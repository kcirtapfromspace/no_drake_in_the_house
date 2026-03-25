import { ConvexError, v } from "convex/values";
import {
  action,
  internalAction,
  internalMutation,
  internalQuery,
  mutation,
  query,
} from "./_generated/server";
import { nowIso, requireCurrentUser } from "./lib/auth";
import {
  decryptToken,
  encryptToken,
  getEncryptionKey,
  isEncrypted,
} from "./lib/crypto";

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

    // --- Encrypt tokens before persisting ---
    const encryptionKey = getEncryptionKey();
    const encryptedAccess = await encryptToken(
      tokenData.access_token,
      encryptionKey,
    );
    const encryptedRefresh = tokenData.refresh_token
      ? await encryptToken(tokenData.refresh_token, encryptionKey)
      : undefined;

    // --- Persist the connection with encrypted tokens ---
    const connectionId = await ctx.runMutation(
      "providerOAuth:_upsertConnection" as any,
      {
        provider: args.provider,
        status: "active",
        providerUserId,
        accessToken: encryptedAccess,
        refreshToken: encryptedRefresh,
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

// ---------------------------------------------------------------------------
// Token refresh: internal functions called by the cron job
// ---------------------------------------------------------------------------

/**
 * Internal query: find all active provider connections whose token expires
 * within the next 10 minutes. Returns the fields needed by the refresh action.
 */
export const _getExpiringConnections = internalQuery({
  args: {},
  handler: async (ctx) => {
    const tenMinutesFromNow = new Date(
      Date.now() + 10 * 60 * 1000,
    ).toISOString();

    const allConnections = await ctx.db
      .query("providerConnections")
      .collect();

    return allConnections
      .filter(
        (c) =>
          c.status === "active" &&
          c.expiresAt != null &&
          c.expiresAt <= tenMinutesFromNow &&
          c.encryptedRefreshToken != null,
      )
      .map((c) => ({
        connectionId: c._id,
        provider: c.provider,
        encryptedRefreshToken: c.encryptedRefreshToken!,
      }));
  },
});

/**
 * Internal mutation: update the token fields on a connection without requiring
 * user auth context (used by the cron refresh action).
 */
export const _updateConnectionTokenInternal = internalMutation({
  args: {
    connectionId: v.id("providerConnections"),
    encryptedAccessToken: v.string(),
    expiresAt: v.optional(v.string()),
  },
  handler: async (ctx, args) => {
    await ctx.db.patch(args.connectionId, {
      encryptedAccessToken: args.encryptedAccessToken,
      expiresAt: args.expiresAt,
      updatedAt: nowIso(),
    });
  },
});

/**
 * Internal action: refresh all expiring OAuth tokens.
 * Called on a 30-minute cron schedule.
 */
export const refreshExpiringTokens = internalAction({
  args: {},
  handler: async (ctx) => {
    const expiring: Array<{
      connectionId: string;
      provider: string;
      encryptedRefreshToken: string;
    }> = await ctx.runQuery(
      "providerOAuth:_getExpiringConnections" as any,
    );

    if (expiring.length === 0) return;

    const encryptionKey = getEncryptionKey();

    for (const conn of expiring) {
      try {
        // Decrypt the stored refresh token (or use as-is for legacy plaintext)
        const refreshToken = isEncrypted(conn.encryptedRefreshToken)
          ? await decryptToken(conn.encryptedRefreshToken, encryptionKey)
          : conn.encryptedRefreshToken;

        // Exchange the refresh token for a new access token
        const { clientId, clientSecret } = getProviderCredentials(
          conn.provider,
        );
        if (!clientId || !clientSecret) continue;

        const tokenUrl = getTokenEndpoint(conn.provider);
        const body = new URLSearchParams({
          grant_type: "refresh_token",
          refresh_token: refreshToken,
        });

        const headers: Record<string, string> = {
          "Content-Type": "application/x-www-form-urlencoded",
        };

        if (conn.provider === "youtube") {
          body.set("client_id", clientId);
          body.set("client_secret", clientSecret);
        } else {
          headers["Authorization"] = `Basic ${btoa(`${clientId}:${clientSecret}`)}`;
        }

        const resp = await fetch(tokenUrl, {
          method: "POST",
          headers,
          body: body.toString(),
        });

        if (!resp.ok) {
          console.error(
            `Token refresh failed for ${conn.provider} connection ${conn.connectionId}: ${resp.status}`,
          );
          continue;
        }

        const tokenData = (await resp.json()) as {
          access_token: string;
          expires_in?: number;
        };

        // Encrypt the new access token
        const encryptedNewAccess = await encryptToken(
          tokenData.access_token,
          encryptionKey,
        );

        const expiresAt = tokenData.expires_in
          ? new Date(Date.now() + tokenData.expires_in * 1000).toISOString()
          : undefined;

        // Persist the encrypted token
        await ctx.runMutation(
          "providerOAuth:_updateConnectionTokenInternal" as any,
          {
            connectionId: conn.connectionId,
            encryptedAccessToken: encryptedNewAccess,
            expiresAt,
          },
        );
      } catch (err: any) {
        console.error(
          `Token refresh error for connection ${conn.connectionId}:`,
          err.message ?? err,
        );
      }
    }
  },
});
