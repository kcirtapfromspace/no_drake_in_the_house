import { v } from "convex/values";
import {
  query,
  internalMutation,
} from "./_generated/server";
import { getCurrentUser } from "./lib/auth";
import { nowIso } from "./lib/auth";
import { getUserPlan, checkFeatureAccess as checkFeatureAccessHelper } from "./lib/entitlements";

// --- Internal Mutations ---

export const syncSubscription = internalMutation({
  args: {
    stripeCustomerId: v.string(),
    stripeSubscriptionId: v.string(),
    plan: v.union(
      v.literal("free"),
      v.literal("pro"),
      v.literal("team"),
    ),
    status: v.union(
      v.literal("active"),
      v.literal("past_due"),
      v.literal("canceled"),
      v.literal("trialing"),
      v.literal("incomplete"),
    ),
    currentPeriodStart: v.string(),
    currentPeriodEnd: v.string(),
    stripePriceId: v.string(),
    cancelAtPeriodEnd: v.optional(v.boolean()),
  },
  handler: async (ctx, args) => {
    const now = nowIso();

    // Look for existing subscription by stripeSubscriptionId
    const existing = await ctx.db
      .query("subscriptions")
      .withIndex("by_stripeSubscriptionId", (q) =>
        q.eq("stripeSubscriptionId", args.stripeSubscriptionId),
      )
      .unique();

    if (existing) {
      // Update existing subscription
      await ctx.db.patch(existing._id, {
        plan: args.plan,
        status: args.status,
        currentPeriodStart: args.currentPeriodStart,
        currentPeriodEnd: args.currentPeriodEnd,
        stripePriceId: args.stripePriceId,
        cancelAtPeriodEnd: args.cancelAtPeriodEnd,
        updatedAt: now,
      });
      return existing._id;
    }

    // Try to find a user by stripeCustomerId from existing subscriptions
    let userId: any = undefined;
    const existingByCustomer = await ctx.db
      .query("subscriptions")
      .withIndex("by_stripeCustomerId", (q) =>
        q.eq("stripeCustomerId", args.stripeCustomerId),
      )
      .first();

    if (existingByCustomer) {
      userId = existingByCustomer.userId;
    }

    // Create new subscription record
    const subId = await ctx.db.insert("subscriptions", {
      legacyKey: `stripe:sub:${args.stripeSubscriptionId}`,
      createdAt: now,
      updatedAt: now,
      userId: userId as any, // May be undefined if no user mapping exists yet
      stripeCustomerId: args.stripeCustomerId,
      stripeSubscriptionId: args.stripeSubscriptionId,
      stripePriceId: args.stripePriceId,
      plan: args.plan,
      status: args.status,
      currentPeriodStart: args.currentPeriodStart,
      currentPeriodEnd: args.currentPeriodEnd,
      cancelAtPeriodEnd: args.cancelAtPeriodEnd,
    });

    return subId;
  },
});

export const recordBillingEvent = internalMutation({
  args: {
    stripeEventId: v.string(),
    eventType: v.string(),
    payload: v.any(),
    userId: v.optional(v.id("users")),
  },
  handler: async (ctx, args) => {
    const now = nowIso();

    // Idempotency check: skip if event already recorded
    const existing = await ctx.db
      .query("billingEvents")
      .withIndex("by_stripeEventId", (q) =>
        q.eq("stripeEventId", args.stripeEventId),
      )
      .unique();

    if (existing) {
      return existing._id;
    }

    const eventId = await ctx.db.insert("billingEvents", {
      legacyKey: `stripe:event:${args.stripeEventId}`,
      createdAt: now,
      updatedAt: now,
      stripeEventId: args.stripeEventId,
      eventType: args.eventType,
      payload: args.payload,
      userId: args.userId,
      processedAt: now,
    });

    return eventId;
  },
});

// --- Public Queries ---

export const getSubscription = query({
  args: {},
  handler: async (ctx) => {
    const { user } = await getCurrentUser(ctx);
    if (!user) {
      return null;
    }

    const { plan, isActive, subscription } = await getUserPlan(ctx, user._id);

    if (!subscription) {
      return null;
    }

    return {
      _id: subscription._id,
      plan: subscription.plan,
      status: subscription.status,
      currentPeriodStart: subscription.currentPeriodStart,
      currentPeriodEnd: subscription.currentPeriodEnd,
      cancelAtPeriodEnd: subscription.cancelAtPeriodEnd,
      seats: subscription.seats,
      isActive,
      activePlan: plan,
    };
  },
});

export const getFeatureAccess = query({
  args: {
    feature: v.string(),
  },
  handler: async (ctx, args) => {
    const { user } = await getCurrentUser(ctx);
    if (!user) {
      return {
        allowed: false,
        reason: "User not found. Please sign in.",
      };
    }

    return checkFeatureAccessHelper(ctx, user._id, args.feature);
  },
});
