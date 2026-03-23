import { ConvexError, v } from "convex/values";
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

/** Env var names per provider */
function getProviderCredentials(provider: string) {
  switch (provider) {
    case "spotify":
      return {
        clientId: process.env.SPOTIFY_CLIENT_ID,
        clientSecret: process.env.SPOTIFY_CLIENT_SECRET,
      };
    case "tidal":
      return {
        clientId: process.env.TIDAL_CLIENT_ID,
        clientSecret: process.env.TIDAL_CLIENT_SECRET,
      };
    case "youtube":
      return {
        clientId: process.env.YOUTUBE_CLIENT_ID,
        clientSecret: process.env.YOUTUBE_CLIENT_SECRET,
      };
    default:
      throw new Error(`Unsupported provider: ${provider}`);
  }
}

/** Token endpoint URLs per provider */
function getTokenEndpoint(provider: string) {
  switch (provider) {
    case "spotify":
      return "https://accounts.spotify.com/api/token";
    case "tidal":
      return "https://auth.tidal.com/v1/oauth2/token";
    case "youtube":
      return "https://oauth2.googleapis.com/token";
    default:
      throw new Error(`Unsupported provider: ${provider}`);
  }
}

/** Profile endpoint URLs per provider */
function getProfileEndpoint(provider: string) {
  switch (provider) {
    case "spotify":
      return "https://api.spotify.com/v1/me";
    case "tidal":
      return "https://openapi.tidal.com/v2/users/me";
    case "youtube":
      return "https://www.googleapis.com/youtube/v3/channels?part=snippet&mine=true";
    default:
      return null;
  }
}

export const authorize = action({
  args: {
    provider: v.string(),
    redirectUri: v.optional(v.string()),
    scopes: v.optional(v.array(v.string())),
  },
  handler: async (_ctx, args) => {
    const { clientId } = getProviderCredentials(args.provider);
    if (!clientId) {
      throw new ConvexError(
        `Missing client ID for provider ${args.provider}. ` +
          `Set the ${args.provider.toUpperCase()}_CLIENT_ID environment variable.`,
      );
    }

    const scopeStr = (args.scopes ?? []).join(" ");
    const state = `nodrake_${args.provider}_${Date.now()}`;
    const redirectUri = encodeURIComponent(args.redirectUri ?? "");

    let authUrl = "";
    switch (args.provider) {
      case "spotify":
        authUrl =
          `https://accounts.spotify.com/authorize` +
          `?client_id=${encodeURIComponent(clientId)}` +
          `&response_type=code` +
          `&redirect_uri=${redirectUri}` +
          `&scope=${encodeURIComponent(scopeStr)}` +
          `&state=${state}`;
        break;
      case "tidal":
        authUrl =
          `https://login.tidal.com/authorize` +
          `?client_id=${encodeURIComponent(clientId)}` +
          `&response_type=code` +
          `&redirect_uri=${redirectUri}` +
          `&scope=${encodeURIComponent(scopeStr)}` +
          `&state=${state}`;
        break;
      case "youtube":
        authUrl =
          `https://accounts.google.com/o/oauth2/v2/auth` +
          `?client_id=${encodeURIComponent(clientId)}` +
          `&response_type=code` +
          `&redirect_uri=${redirectUri}` +
          `&scope=${encodeURIComponent(scopeStr)}` +
          `&access_type=offline` +
          `&prompt=consent` +
          `&state=${state}`;
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
    const { clientId, clientSecret } = getProviderCredentials(args.provider);
    if (!clientId || !clientSecret) {
      throw new ConvexError(
        `Missing OAuth credentials for provider ${args.provider}. ` +
          `Set ${args.provider.toUpperCase()}_CLIENT_ID and ${args.provider.toUpperCase()}_CLIENT_SECRET.`,
      );
    }

    // --- Exchange the authorization code for tokens ---
    const tokenUrl = getTokenEndpoint(args.provider);
    const tokenBody = new URLSearchParams({
      grant_type: "authorization_code",
      code: args.code,
      redirect_uri: args.redirectUri ?? "",
    });

    // Spotify and Tidal use HTTP Basic auth; YouTube/Google uses body params
    const headers: Record<string, string> = {
      "Content-Type": "application/x-www-form-urlencoded",
    };

    if (args.provider === "youtube") {
      tokenBody.set("client_id", clientId);
      tokenBody.set("client_secret", clientSecret);
    } else {
      const basicAuth = btoa(`${clientId}:${clientSecret}`);
      headers["Authorization"] = `Basic ${basicAuth}`;
    }

    const tokenResp = await fetch(tokenUrl, {
      method: "POST",
      headers,
      body: tokenBody.toString(),
    });

    if (!tokenResp.ok) {
      const errorText = await tokenResp.text();
      throw new ConvexError(
        `Token exchange failed for ${args.provider}: ${tokenResp.status} ${errorText}`,
      );
    }

    const tokenData = (await tokenResp.json()) as {
      access_token: string;
      refresh_token?: string;
      expires_in?: number;
      token_type?: string;
      scope?: string;
    };

    // --- Optionally fetch the user's profile to get a provider user ID ---
    let providerUserId: string | undefined;
    const profileUrl = getProfileEndpoint(args.provider);

    if (profileUrl && tokenData.access_token) {
      try {
        const profileResp = await fetch(profileUrl, {
          headers: { Authorization: `Bearer ${tokenData.access_token}` },
        });
        if (profileResp.ok) {
          const profile = (await profileResp.json()) as Record<string, any>;
          switch (args.provider) {
            case "spotify":
              providerUserId = profile.id;
              break;
            case "tidal":
              providerUserId = profile.data?.id ?? profile.id;
              break;
            case "youtube": {
              const items = profile.items as Array<Record<string, any>> | undefined;
              providerUserId = items?.[0]?.id;
              break;
            }
          }
        }
      } catch {
        // Profile fetch is best-effort; proceed without providerUserId
      }
    }

    // --- Compute token expiry time ---
    const expiresAt = tokenData.expires_in
      ? new Date(Date.now() + tokenData.expires_in * 1000).toISOString()
      : undefined;

    const scopes = tokenData.scope ? tokenData.scope.split(" ") : [];

    // --- Persist the connection with tokens ---
    const connectionId = await ctx.runMutation(
      "providerOAuth:_upsertConnection" as any,
      {
        provider: args.provider,
        status: "active",
        providerUserId,
        accessToken: tokenData.access_token,
        refreshToken: tokenData.refresh_token,
        expiresAt,
        scopes,
      },
    );

    return {
      success: true,
      provider: args.provider,
      connection_id: connectionId,
      provider_user_id: providerUserId,
      expires_at: expiresAt,
    };
  },
});

export const _upsertConnection = mutation({
  args: {
    provider: v.string(),
    status: v.string(),
    providerUserId: v.optional(v.string()),
    accessToken: v.optional(v.string()),
    refreshToken: v.optional(v.string()),
    expiresAt: v.optional(v.string()),
    scopes: v.optional(v.array(v.string())),
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

    const fields: Record<string, any> = {
      status: args.status,
      providerUserId: args.providerUserId,
      updatedAt: now,
    };

    if (args.accessToken !== undefined) {
      fields.encryptedAccessToken = args.accessToken;
    }
    if (args.refreshToken !== undefined) {
      fields.encryptedRefreshToken = args.refreshToken;
    }
    if (args.expiresAt !== undefined) {
      fields.expiresAt = args.expiresAt;
    }
    if (args.scopes !== undefined) {
      fields.scopes = args.scopes;
    }

    if (existing) {
      await ctx.db.patch(existing._id, fields);
      return existing._id;
    }

    return await ctx.db.insert("providerConnections", {
      legacyKey: `runtime:conn:${user._id}:${args.provider}`,
      userId: user._id,
      provider: args.provider,
      providerUserId: args.providerUserId,
      status: args.status,
      scopes: args.scopes ?? [],
      encryptedAccessToken: args.accessToken,
      encryptedRefreshToken: args.refreshToken,
      expiresAt: args.expiresAt,
      metadata: {},
      createdAt: now,
      updatedAt: now,
    });
  },
});
