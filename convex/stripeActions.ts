"use node";

import { v } from "convex/values";
import { internalAction } from "./_generated/server";
import { internal } from "./_generated/api";

function getStripeClient() {
  // Dynamic import to avoid issues in test environments
  const Stripe = require("stripe");
  const stripeSecretKey = process.env.STRIPE_SECRET_KEY;
  if (!stripeSecretKey) {
    throw new Error("STRIPE_SECRET_KEY environment variable is not set.");
  }
  return new Stripe(stripeSecretKey, {
    apiVersion: "2024-12-18.acacia",
  });
}

export const createCheckoutSession = internalAction({
  args: {
    userId: v.id("users"),
    plan: v.union(v.literal("pro"), v.literal("team")),
    successUrl: v.string(),
    cancelUrl: v.string(),
  },
  handler: async (ctx, args) => {
    const stripe = getStripeClient();

    const priceId =
      args.plan === "pro"
        ? process.env.STRIPE_PRO_PRICE_ID
        : process.env.STRIPE_TEAM_PRICE_ID;

    if (!priceId) {
      throw new Error(
        `STRIPE_${args.plan.toUpperCase()}_PRICE_ID environment variable is not set.`,
      );
    }

    const session = await stripe.checkout.sessions.create({
      mode: "subscription",
      line_items: [{ price: priceId, quantity: 1 }],
      success_url: args.successUrl,
      cancel_url: args.cancelUrl,
      metadata: {
        convexUserId: args.userId,
        plan: args.plan,
      },
      subscription_data: {
        metadata: {
          convexUserId: args.userId,
          plan: args.plan,
        },
      },
    });

    return { sessionUrl: session.url };
  },
});

export const createCustomerPortalSession = internalAction({
  args: {
    userId: v.id("users"),
    returnUrl: v.string(),
  },
  handler: async (ctx, args) => {
    const stripe = getStripeClient();

    // Look up the user's Stripe customer ID from their subscription
    const subscription: any = await ctx.runQuery(
      internal.stripeHelpers.getSubscriptionByUserId,
      { userId: args.userId },
    );

    if (!subscription) {
      throw new Error("No active subscription found for this user.");
    }

    const session = await stripe.billingPortal.sessions.create({
      customer: subscription.stripeCustomerId,
      return_url: args.returnUrl,
    });

    return { portalUrl: session.url };
  },
});

export const handleWebhookEvent = internalAction({
  args: {
    payload: v.string(),
    signature: v.string(),
  },
  handler: async (ctx, args) => {
    const stripe = getStripeClient();

    const webhookSecret = process.env.STRIPE_WEBHOOK_SECRET;
    if (!webhookSecret) {
      throw new Error(
        "STRIPE_WEBHOOK_SECRET environment variable is not set.",
      );
    }

    const event = stripe.webhooks.constructEvent(
      args.payload,
      args.signature,
      webhookSecret,
    );

    // Record the billing event (idempotent)
    await ctx.runMutation(internal.subscriptions.recordBillingEvent, {
      stripeEventId: event.id,
      eventType: event.type,
      payload: event.data.object,
    });

    // Process subscription-related events
    switch (event.type) {
      case "customer.subscription.created":
      case "customer.subscription.updated":
      case "customer.subscription.deleted": {
        const subscription = event.data.object;
        const plan = subscription.metadata?.plan ?? "pro";
        const statusMap: Record<string, string> = {
          active: "active",
          past_due: "past_due",
          canceled: "canceled",
          trialing: "trialing",
          incomplete: "incomplete",
          incomplete_expired: "incomplete",
          unpaid: "past_due",
        };

        await ctx.runMutation(internal.subscriptions.syncSubscription, {
          stripeCustomerId:
            typeof subscription.customer === "string"
              ? subscription.customer
              : subscription.customer.id,
          stripeSubscriptionId: subscription.id,
          plan: plan as "free" | "pro" | "team",
          status: (statusMap[subscription.status] ?? "incomplete") as any,
          currentPeriodStart: new Date(
            subscription.current_period_start * 1000,
          ).toISOString(),
          currentPeriodEnd: new Date(
            subscription.current_period_end * 1000,
          ).toISOString(),
          stripePriceId: subscription.items.data[0]?.price.id ?? "",
          cancelAtPeriodEnd: subscription.cancel_at_period_end,
        });
        break;
      }

      case "invoice.paid":
      case "invoice.payment_failed":
        // Already recorded as billing event above
        break;

      default:
        // Unhandled event type - already recorded
        break;
    }

    return { received: true };
  },
});
