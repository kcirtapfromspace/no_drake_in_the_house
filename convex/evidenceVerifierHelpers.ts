import { v } from "convex/values";
import { internalQuery } from "./_generated/server";

/**
 * Look up a user by their auth tokenIdentifier.
 * Used by the evidence submission action for auth.
 */
export const _getCurrentUserByToken = internalQuery({
  args: { tokenIdentifier: v.string() },
  handler: async (ctx, args) => {
    const user = await ctx.db
      .query("users")
      .withIndex("by_authSubject", (q) =>
        q.eq("authSubject", args.tokenIdentifier),
      )
      .unique();
    return user;
  },
});

/**
 * Count how many evidence records a user has submitted today.
 * Uses offenseEvidence.submittedByUserId + _creationTime.
 */
export const _countUserSubmissionsToday = internalQuery({
  args: { userId: v.id("users") },
  handler: async (ctx, args) => {
    const startOfDay = new Date();
    startOfDay.setHours(0, 0, 0, 0);
    const startOfDayMs = startOfDay.getTime();

    // Scan offenseEvidence for records submitted by this user today.
    // There's no index on submittedByUserId, so we scan by offenseId index
    // and filter. With the 10/day rate limit, this is bounded.
    let count = 0;
    for await (const evidence of ctx.db
      .query("offenseEvidence")
      .order("desc")) {
      // Stop once we pass today's records
      if (evidence._creationTime < startOfDayMs) break;
      if (evidence.submittedByUserId === args.userId) {
        count++;
      }
    }
    return count;
  },
});
