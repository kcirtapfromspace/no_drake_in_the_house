import { v } from "convex/values";
import { action, mutation, query } from "./_generated/server";
import { nowIso, requireCurrentUser } from "./lib/auth";

export const status = query({
  args: {
    provider: v.string(),
  },
  handler: async (ctx, args) => {
    const { user } = await requireCurrentUser(ctx);
    const connection = await ctx.db
      .query("providerConnections")
      .withIndex("by_user_provider", (q) =>
        q.eq("userId", user._id).eq("provider", args.provider),
      )
      .unique();

    if (!connection) {
      return {
        connected: false,
        provider: args.provider,
        status: "not_connected",
      };
    }

    return {
      connected: connection.status === "active",
      provider: args.provider,
      status: connection.status,
      provider_user_id: connection.providerUserId,
      expires_at: connection.expiresAt,
      scopes: connection.scopes ?? [],
      last_health_check: connection.lastHealthCheckAt,
    };
  },
});

export const authorize = action({
  args: {
    provider: v.string(),
    redirectUri: v.optional(v.string()),
    scopes: v.optional(v.array(v.string())),
  },
  handler: async (_ctx, args) => {
    const scopeStr = (args.scopes ?? []).join(" ");
    const state = `nodrake_${args.provider}_${Date.now()}`;

    let authUrl = "";
    switch (args.provider) {
      case "spotify":
        authUrl = `https://accounts.spotify.com/authorize?client_id=PLACEHOLDER&response_type=code&redirect_uri=${encodeURIComponent(args.redirectUri ?? "")}&scope=${encodeURIComponent(scopeStr)}&state=${state}`;
        break;
      case "tidal":
        authUrl = `https://login.tidal.com/authorize?client_id=PLACEHOLDER&response_type=code&redirect_uri=${encodeURIComponent(args.redirectUri ?? "")}&scope=${encodeURIComponent(scopeStr)}&state=${state}`;
        break;
      case "youtube":
        authUrl = `https://accounts.google.com/o/oauth2/v2/auth?client_id=PLACEHOLDER&response_type=code&redirect_uri=${encodeURIComponent(args.redirectUri ?? "")}&scope=${encodeURIComponent(scopeStr)}&state=${state}`;
        break;
      default:
        throw new Error(`Unsupported provider: ${args.provider}`);
    }

    return { auth_url: authUrl, state };
  },
});

export const callback = action({
  args: {
    provider: v.string(),
    code: v.string(),
    state: v.optional(v.string()),
    redirectUri: v.optional(v.string()),
  },
  handler: async (ctx, args) => {
    const connectionId = await ctx.runMutation(
      // @ts-expect-error -- internal reference via string
      "providerOAuth:_upsertConnection" as any,
      {
        provider: args.provider,
        status: "active",
      },
    );

    return {
      success: true,
      provider: args.provider,
      connection_id: connectionId,
    };
  },
});

export const _upsertConnection = mutation({
  args: {
    provider: v.string(),
    status: v.string(),
    providerUserId: v.optional(v.string()),
  },
  handler: async (ctx, args) => {
    const { user } = await requireCurrentUser(ctx);
    const now = nowIso();

    const existing = await ctx.db
      .query("providerConnections")
      .withIndex("by_user_provider", (q) =>
        q.eq("userId", user._id).eq("provider", args.provider),
      )
      .unique();

    if (existing) {
      await ctx.db.patch(existing._id, {
        status: args.status,
        providerUserId: args.providerUserId,
        updatedAt: now,
      });
      return existing._id;
    }

    return await ctx.db.insert("providerConnections", {
      legacyKey: `runtime:conn:${user._id}:${args.provider}`,
      userId: user._id,
      provider: args.provider,
      providerUserId: args.providerUserId,
      status: args.status,
      scopes: [],
      metadata: {},
      createdAt: now,
      updatedAt: now,
    });
  },
});
