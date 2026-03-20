import { ConvexError, v } from "convex/values";
import { mutation, query } from "./_generated/server";
import { nowIso, requireCurrentUser } from "./lib/auth";

export const listCurrentUser = query({
  args: {},
  handler: async (ctx) => {
    const { user } = await requireCurrentUser(ctx);
    return await ctx.db
      .query("providerConnections")
      .withIndex("by_userId", (q) => q.eq("userId", user._id))
      .collect();
  },
});

export const disconnect = mutation({
  args: {
    provider: v.string(),
  },
  handler: async (ctx, args) => {
    const { user } = await requireCurrentUser(ctx);
    const connection = await ctx.db
      .query("providerConnections")
      .withIndex("by_userId", (q) => q.eq("userId", user._id))
      .collect()
      .then((connections) =>
        connections.find((entry) => entry.provider === args.provider) ?? null,
      );

    if (!connection) {
      throw new ConvexError(`No ${args.provider} connection found.`);
    }

    await ctx.db.patch(connection._id, {
      status: "revoked",
      updatedAt: nowIso(),
    });

    return await ctx.db.get(connection._id);
  },
});
