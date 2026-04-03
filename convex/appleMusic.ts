import { v } from "convex/values";
import {
  action,
  mutation,
  query,
} from "./_generated/server";
import type { MutationCtx, QueryCtx } from "./_generated/server";
import { nowIso, requireCurrentUser } from "./lib/auth";
import { encryptToken, getEncryptionKey } from "./lib/crypto";

/**
 * Connect Apple Music. This is an action (not mutation) because it encrypts
 * the MusicKit user token via Web Crypto before persisting.
 */
export const connect = action({
  args: {
    musicUserToken: v.string(),
  },
  handler: async (ctx, args) => {
    const encryptionKey = getEncryptionKey();
    const encryptedToken = await encryptToken(args.musicUserToken, encryptionKey);

    const result: { success: boolean; connection_id: string } =
      await ctx.runMutation("appleMusic:_storeConnection" as any, {
        encryptedToken,
      });

    return result;
  },
});

/** Internal mutation: persist the already-encrypted Apple Music token. */
export const _storeConnection = mutation({
  args: {
    encryptedToken: v.string(),
  },
  handler: async (ctx: MutationCtx, args) => {
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
        encryptedAccessToken: args.encryptedToken,
        updatedAt: now,
      });
      return { success: true, connection_id: existing._id };
    }

    const connId = await ctx.db.insert("providerConnections", {
      legacyKey: `runtime:conn:${user._id}:apple_music`,
      userId: user._id,
      provider: "apple_music",
      status: "active",
      encryptedAccessToken: args.encryptedToken,
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
  handler: async (ctx: MutationCtx) => {
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
  handler: async (ctx: QueryCtx) => {
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
    const token = args.musicUserToken;
    const isValid = token.length > 100 && !token.includes(" ");
    return {
      valid: isValid,
      provider: "apple_music",
    };
  },
});
