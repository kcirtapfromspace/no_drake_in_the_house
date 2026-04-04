import { v } from "convex/values";
import {
  action,
  internalAction,
  internalMutation,
  internalQuery,
  mutation,
  query,
} from "./_generated/server";
import type { MutationCtx, QueryCtx } from "./_generated/server";
import { api, internal } from "./_generated/api";
import { nowIso, requireCurrentUser } from "./lib/auth";
import {
  decryptToken,
  encryptToken,
  getEncryptionKey,
  isEncrypted,
} from "./lib/crypto";
import {
  exchangeAuthCode,
  extractProviderUserId,
  getDefaultScopes,
  getProfileEndpoint,
  getProviderConfig,
  refreshAccessToken,
  resolveRedirectUri,
} from "./lib/oauth";

export const status = query({
  args: {
    provider: v.string(),
  },
  handler: async (ctx: QueryCtx, args) => {
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

/** Authorization endpoint URLs per provider */
const AUTH_ENDPOINTS: Record<string, string> = {
  spotify: "https://accounts.spotify.com/authorize",
  tidal: "https://login.tidal.com/authorize",
  youtube: "https://accounts.google.com/o/oauth2/v2/auth",
};

/** Generate a PKCE code verifier and challenge (S256). */
async function generatePkce(): Promise<{
  codeVerifier: string;
  codeChallenge: string;
}> {
  const array = new Uint8Array(32);
  crypto.getRandomValues(array);
  const codeVerifier = Array.from(array, (b) =>
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~".charAt(
      b % 66,
    ),
  ).join("");

  const digest = await crypto.subtle.digest(
    "SHA-256",
    new TextEncoder().encode(codeVerifier),
  );
  const codeChallenge = btoa(String.fromCharCode(...new Uint8Array(digest)))
    .replace(/\+/g, "-")
    .replace(/\//g, "_")
    .replace(/=+$/, "");

  return { codeVerifier, codeChallenge };
}

/** Providers that require PKCE (OAuth 2.1). */
const PKCE_PROVIDERS = new Set(["tidal"]);

export const authorize = action({
  args: {
    provider: v.string(),
    redirectUri: v.optional(v.string()),
    scopes: v.optional(v.array(v.string())),
  },
  handler: async (_ctx, args) => {
    const { clientId } = getProviderConfig(args.provider);

    const scopes = args.scopes?.length ? args.scopes : getDefaultScopes(args.provider);
    const scopeStr = scopes.join(" ");
    const state = `nodrake_${args.provider}_${Date.now()}`;
    const rawRedirectUri = resolveRedirectUri(args.provider, args.redirectUri);
    const redirectUri = encodeURIComponent(rawRedirectUri);

    const baseUrl = AUTH_ENDPOINTS[args.provider];
    if (!baseUrl) throw new Error(`Unsupported provider: ${args.provider}`);

    let authUrl =
      `${baseUrl}` +
      `?client_id=${encodeURIComponent(clientId)}` +
      `&response_type=code` +
      `&redirect_uri=${redirectUri}` +
      `&scope=${encodeURIComponent(scopeStr)}` +
      `&state=${state}`;

    // YouTube/Google requires offline access + consent prompt
    if (args.provider === "youtube") {
      authUrl += `&access_type=offline&prompt=consent`;
    }

    // Tidal (OAuth 2.1) requires PKCE
    let codeVerifier: string | undefined;
    if (PKCE_PROVIDERS.has(args.provider)) {
      const pkce = await generatePkce();
      codeVerifier = pkce.codeVerifier;
      authUrl += `&code_challenge=${pkce.codeChallenge}&code_challenge_method=S256`;
    }

    return {
      authorization_url: authUrl,
      auth_url: authUrl,
      state,
      scopes,
      code_verifier: codeVerifier,
    };
  },
});

export const callback = action({
  args: {
    provider: v.string(),
    code: v.string(),
    state: v.optional(v.string()),
    redirectUri: v.optional(v.string()),
    codeVerifier: v.optional(v.string()),
  },
  handler: async (ctx, args) => {
    // --- Exchange the authorization code for tokens (unified, no Basic Auth) ---
    const redirectUri = resolveRedirectUri(args.provider, args.redirectUri);
    const tokenData = await exchangeAuthCode(
      args.provider,
      args.code,
      redirectUri,
      args.codeVerifier,
    );

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
          providerUserId = extractProviderUserId(args.provider, profile);
        }
      } catch {
        // Profile fetch is best-effort; proceed without providerUserId
      }
    }

    // For providers without a profile endpoint (e.g. Tidal), extract user ID
    // from the JWT access token's sub claim.
    if (!providerUserId && tokenData.access_token) {
      try {
        const parts = tokenData.access_token.split(".");
        if (parts.length === 3) {
          const payload = JSON.parse(atob(parts[1]));
          const raw = payload.sub ?? payload.uid;
          providerUserId = raw != null ? String(raw) : undefined;
        }
      } catch {
        // Not a JWT — ignore
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
      api.providerOAuth._upsertConnection,
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

    // --- Auto-trigger library sync after successful connection ---
    const syncProviders: Record<string, typeof internal.librarySyncActions.syncSpotifyLibrary> = {
      spotify: internal.librarySyncActions.syncSpotifyLibrary,
      tidal: internal.librarySyncActions.syncTidalLibrary,
      youtube: internal.librarySyncActions.syncYouTubeLibrary,
      apple_music: internal.librarySyncActions.syncAppleMusicLibrary,
    };
    const syncActionRef = syncProviders[args.provider];
    if (syncActionRef) {
      try {
        const { runId, userId } = (await ctx.runMutation(
          api.sync._createRunWithUser,
          { platform: args.provider },
        )) as { runId: string; userId: string };
        await ctx.scheduler.runAfter(0, syncActionRef, { runId, userId });
      } catch (syncErr: any) {
        console.warn(
          `Auto-sync scheduling failed for ${args.provider}:`,
          syncErr?.message ?? syncErr,
        );
      }
    }

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
  handler: async (ctx: MutationCtx, args) => {
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
      internal.providerOAuth._getExpiringConnections,
    );

    if (expiring.length === 0) return;

    const encryptionKey = getEncryptionKey();

    for (const conn of expiring) {
      try {
        const plainRefresh = isEncrypted(conn.encryptedRefreshToken)
          ? await decryptToken(conn.encryptedRefreshToken, encryptionKey)
          : conn.encryptedRefreshToken;

        // Unified refresh: body params for all providers, no Basic Auth
        const tokenData = await refreshAccessToken(conn.provider, plainRefresh);

        const encryptedNewAccess = await encryptToken(
          tokenData.access_token,
          encryptionKey,
        );

        const expiresAt = tokenData.expires_in
          ? new Date(Date.now() + tokenData.expires_in * 1000).toISOString()
          : undefined;

        await ctx.runMutation(
          internal.providerOAuth._updateConnectionTokenInternal,
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
