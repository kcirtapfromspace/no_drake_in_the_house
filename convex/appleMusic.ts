import { v } from "convex/values";
import { action, mutation, query } from "./_generated/server";
import { internal } from "./_generated/api";
import { nowIso, requireCurrentUser } from "./lib/auth";

export const connect = mutation({
  args: {
    musicUserToken: v.string(),
  },
  handler: async (ctx, args) => {
    const { user } = await requireCurrentUser(ctx);
    const now = nowIso();

    const existing = await ctx.db
      .query("providerConnections")
      .withIndex("by_user_provider", (q) =>
        q.eq("userId", user._id).eq("provider", "apple_music"),
      )
      .unique();

    if (existing) {
      await ctx.db.patch(existing._id, {
        status: "active",
        encryptedAccessToken: args.musicUserToken,
        updatedAt: now,
      });
      return { success: true, connection_id: existing._id };
    }

    const connId = await ctx.db.insert("providerConnections", {
      legacyKey: `runtime:conn:${user._id}:apple_music`,
      userId: user._id,
      provider: "apple_music",
      status: "active",
      encryptedAccessToken: args.musicUserToken,
      scopes: ["music-library"],
      metadata: {},
      createdAt: now,
      updatedAt: now,
    });

    return { success: true, connection_id: connId };
  },
});

export const disconnect = mutation({
  args: {},
  handler: async (ctx) => {
    const { user } = await requireCurrentUser(ctx);

    const existing = await ctx.db
      .query("providerConnections")
      .withIndex("by_user_provider", (q) =>
        q.eq("userId", user._id).eq("provider", "apple_music"),
      )
      .unique();

    if (existing) {
      await ctx.db.patch(existing._id, {
        status: "revoked",
        encryptedAccessToken: undefined,
        updatedAt: nowIso(),
      });
    }

    return { success: true };
  },
});

export const status = query({
  args: {},
  handler: async (ctx) => {
    const { user } = await requireCurrentUser(ctx);

    const connection = await ctx.db
      .query("providerConnections")
      .withIndex("by_user_provider", (q) =>
        q.eq("userId", user._id).eq("provider", "apple_music"),
      )
      .unique();

    if (!connection || connection.status !== "active") {
      return {
        connected: false,
        provider: "apple_music",
        status: connection?.status ?? "not_connected",
      };
    }

    return {
      connected: true,
      provider: "apple_music",
      status: "active",
      scopes: connection.scopes ?? [],
      last_health_check: connection.lastHealthCheckAt,
    };
  },
});

export const verify = action({
  args: {
    musicUserToken: v.string(),
  },
  handler: async (_ctx, args) => {
    const isValid = args.musicUserToken.length > 0;
    return {
      valid: isValid,
      provider: "apple_music",
    };
  },
});
