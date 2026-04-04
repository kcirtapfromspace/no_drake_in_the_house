import { v } from "convex/values";
import { action } from "./_generated/server";
import { internal } from "./_generated/api";

export const initiateCheckout = action({
  args: {
    plan: v.union(v.literal("pro"), v.literal("team")),
    successUrl: v.string(),
    cancelUrl: v.string(),
  },
  handler: async (ctx, args) => {
    const identity = await ctx.auth.getUserIdentity();
    if (!identity) {
      throw new Error("Authentication required.");
    }

    // Look up the user by their auth subject
    const user: any = await ctx.runQuery(internal.billing_helpers.getCurrentUserByAuth, {
      authSubject: identity.tokenIdentifier,
    });

    if (!user) {
      throw new Error("User not found. Please complete sign-up first.");
    }

    const result: { sessionUrl: string } = await ctx.runAction(
      internal.stripeActions.createCheckoutSession,
      {
        userId: user._id,
        plan: args.plan,
        successUrl: args.successUrl,
        cancelUrl: args.cancelUrl,
      },
    );

    return result;
  },
});

export const initiatePortal = action({
  args: {
    returnUrl: v.string(),
  },
  handler: async (ctx, args) => {
    const identity = await ctx.auth.getUserIdentity();
    if (!identity) {
      throw new Error("Authentication required.");
    }

    const user: any = await ctx.runQuery(internal.billing_helpers.getCurrentUserByAuth, {
      authSubject: identity.tokenIdentifier,
    });

    if (!user) {
      throw new Error("User not found. Please complete sign-up first.");
    }

    const result: { portalUrl: string } = await ctx.runAction(
      internal.stripeActions.createCustomerPortalSession,
      {
        userId: user._id,
        returnUrl: args.returnUrl,
      },
    );

    return result;
  },
});
