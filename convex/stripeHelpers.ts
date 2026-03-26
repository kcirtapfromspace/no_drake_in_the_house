import { v } from "convex/values";
import { internalQuery } from "./_generated/server";

export const getSubscriptionByUserId = internalQuery({
  args: {
    userId: v.id("users"),
  },
  handler: async (ctx, args) => {
    const subscription = await ctx.db
      .query("subscriptions")
      .withIndex("by_userId", (q) => q.eq("userId", args.userId))
      .order("desc")
      .first();

    return subscription;
  },
});
