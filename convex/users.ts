import { ConvexError, v } from "convex/values";
import { mutation, query } from "./_generated/server";
import { getCurrentUser, nowIso, upsertCurrentUserFromIdentity } from "./lib/auth";

export const current = query({
  args: {},
  handler: async (ctx) => {
    const { user } = await getCurrentUser(ctx);
    return user ?? null;
  },
});

export const upsertCurrent = mutation({
  args: {},
  handler: async (ctx) => {
    return await upsertCurrentUserFromIdentity(ctx);
  },
});

export const linkedAccounts = query({
  args: {},
  handler: async (ctx) => {
    const { user } = await getCurrentUser(ctx);
    if (!user) {
      return [];
    }

    const connections = await ctx.db
      .query("providerConnections")
      .withIndex("by_userId", (q) => q.eq("userId", user._id))
      .collect();

    return connections.map((connection) => ({
      provider: connection.provider,
      provider_user_id: connection.providerUserId ?? "",
      linked_at: connection.createdAt,
      status: connection.status,
      expires_at: connection.expiresAt,
    }));
  },
});

export const attachAuthSubject = mutation({
  args: {
    legacyUserId: v.string(),
    authSubject: v.string(),
  },
  handler: async (ctx, args) => {
    const user = await ctx.db
      .query("users")
      .withIndex("by_legacyUserId", (q) => q.eq("legacyUserId", args.legacyUserId))
      .unique();

    if (!user) {
      throw new ConvexError(`User ${args.legacyUserId} not found.`);
    }

    await ctx.db.patch(user._id, {
      authSubject: args.authSubject,
      updatedAt: nowIso(),
    });

    return await ctx.db.get(user._id);
  },
});
