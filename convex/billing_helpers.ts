import { v } from "convex/values";
import { internalQuery } from "./_generated/server";

export const getCurrentUserByAuth = internalQuery({
  args: {
    authSubject: v.string(),
  },
  handler: async (ctx, args) => {
    const user = await ctx.db
      .query("users")
      .withIndex("by_authSubject", (q) =>
        q.eq("authSubject", args.authSubject),
      )
      .unique();

    return user;
  },
});
