/**
 * Unified OAuth token exchange and refresh for all providers.
 *
 * Every provider sends `client_id` and `client_secret` as body parameters
 * (standard OAuth 2.0 / 2.1 form-encoded POST). No Basic Auth headers.
 *
 * Only usable inside Convex **actions** because it performs external fetch().
 */

// -----------------------------------------------------------------------
// Provider configuration
// -----------------------------------------------------------------------

interface ProviderConfig {
  clientId: string;
  clientSecret: string;
  tokenEndpoint: string;
}

const TOKEN_ENDPOINTS: Record<string, string> = {
  spotify: "https://accounts.spotify.com/api/token",
  tidal: "https://auth.tidal.com/v1/oauth2/token",
  youtube: "https://oauth2.googleapis.com/token",
};

const DEFAULT_SCOPES: Record<string, string[]> = {
  spotify: [
    "user-library-read",
    "user-library-modify",
    "playlist-read-private",
    "playlist-read-collaborative",
    "playlist-modify-private",
    "playlist-modify-public",
    "user-follow-read",
    "user-follow-modify",
    "user-read-private",
    "user-read-email",
    "user-read-recently-played",
    "user-top-read",
    "user-read-playback-state",
    "user-read-currently-playing",
  ],
  tidal: [
    "user.read",
    "collection.read",
    "collection.write",
    "playlists.read",
    "playlists.write",
    "playback",
    "recommendations.read",
    "entitlements.read",
    "search.read",
    "search.write",
  ],
  youtube: [
    "openid",
    "email",
    "profile",
    "https://www.googleapis.com/auth/youtube",
    "https://www.googleapis.com/auth/youtube.force-ssl",
    "https://www.googleapis.com/auth/youtube.readonly",
  ],
};

const PROFILE_ENDPOINTS: Record<string, string> = {
  spotify: "https://api.spotify.com/v1/me",
  // Tidal v2 API has no /users/me endpoint; user ID is extracted from
  // the JWT access token during OAuth callback instead.
  youtube:
    "https://www.googleapis.com/youtube/v3/channels?part=id&mine=true",
};

// -----------------------------------------------------------------------
// Credential resolution
// -----------------------------------------------------------------------

export function getProviderConfig(provider: string): ProviderConfig {
  const endpoint = TOKEN_ENDPOINTS[provider];
  if (!endpoint) throw new Error(`Unsupported provider: ${provider}`);

  let clientId: string | undefined;
  let clientSecret: string | undefined;

  switch (provider) {
    case "spotify":
      clientId = process.env.SPOTIFY_CLIENT_ID;
      clientSecret = process.env.SPOTIFY_CLIENT_SECRET;
      break;
    case "tidal":
      clientId = process.env.TIDAL_CLIENT_ID;
      clientSecret = process.env.TIDAL_CLIENT_SECRET;
      break;
    case "youtube":
      clientId =
        process.env.YOUTUBE_MUSIC_CLIENT_ID ||
        process.env.YOUTUBE_CLIENT_ID ||
        process.env.GOOGLE_CLIENT_ID;
      clientSecret =
        process.env.YOUTUBE_MUSIC_CLIENT_SECRET ||
        process.env.YOUTUBE_CLIENT_SECRET ||
        process.env.GOOGLE_CLIENT_SECRET;
      break;
  }

  if (!clientId || !clientSecret) {
    throw new Error(
      `Missing OAuth credentials for ${provider}. ` +
        `Set ${provider.toUpperCase()}_CLIENT_ID and ${provider.toUpperCase()}_CLIENT_SECRET.`,
    );
  }

  return { clientId, clientSecret, tokenEndpoint: endpoint };
}

export function getDefaultScopes(provider: string): string[] {
  return DEFAULT_SCOPES[provider] ?? [];
}

export function getProfileEndpoint(provider: string): string | null {
  return PROFILE_ENDPOINTS[provider] ?? null;
}

/**
 * Resolve the redirect URI for a provider.
 *
 * Priority:
 * 1. Caller-supplied value (frontend sends the correct origin)
 * 2. Provider-specific env var (legacy, may point to old Rust backend)
 * 3. Computed from OAUTH_CALLBACK_BASE_URL + /auth/callback/{provider}
 *
 * The caller-supplied URI takes priority because the OAuth callback must
 * land on the FRONTEND (where OAuthCallback.svelte closes the popup),
 * not on the Rust backend which may redirect elsewhere.
 */
export function resolveRedirectUri(
  provider: string,
  callerUri: string | undefined,
): string {
  // 1. Prefer caller-supplied value (frontend origin)
  if (callerUri) return callerUri;

  // 2. Explicit per-provider override
  const key = `${provider.toUpperCase()}_REDIRECT_URI`;
  const envUri = process.env[key];
  if (envUri) return envUri;

  if (provider === "youtube") {
    const alt =
      process.env.YOUTUBE_MUSIC_REDIRECT_URI ||
      process.env.GOOGLE_REDIRECT_URI;
    if (alt) return alt;
  }

  // 3. Compute from base URL
  const baseUrl = process.env.OAUTH_CALLBACK_BASE_URL;
  if (baseUrl) {
    const providerSlug = provider === "youtube" ? "youtube" : provider;
    return `${baseUrl.replace(/\/+$/, "")}/auth/callback/${providerSlug}`;
  }

  return "";
}

// -----------------------------------------------------------------------
// Token exchange (authorization_code → tokens)
// -----------------------------------------------------------------------

export interface TokenResponse {
  access_token: string;
  refresh_token?: string;
  expires_in?: number;
  token_type?: string;
  scope?: string;
}

export async function exchangeAuthCode(
  provider: string,
  code: string,
  redirectUri: string,
  codeVerifier?: string,
): Promise<TokenResponse> {
  const { clientId, clientSecret, tokenEndpoint } =
    getProviderConfig(provider);

  const body = new URLSearchParams({
    grant_type: "authorization_code",
    code,
    redirect_uri: redirectUri,
    client_id: clientId,
    client_secret: clientSecret,
  });

  // PKCE: include code_verifier for OAuth 2.1 providers (e.g. Tidal)
  if (codeVerifier) {
    body.set("code_verifier", codeVerifier);
  }

  const resp = await fetch(tokenEndpoint, {
    method: "POST",
    headers: { "Content-Type": "application/x-www-form-urlencoded" },
    body: body.toString(),
  });

  if (!resp.ok) {
    const text = await resp.text();
    throw new Error(
      `Token exchange failed for ${provider}: ${resp.status} ${text}`,
    );
  }

  return (await resp.json()) as TokenResponse;
}

// -----------------------------------------------------------------------
// Token refresh (refresh_token → new access token)
// -----------------------------------------------------------------------

export interface RefreshResponse {
  access_token: string;
  expires_in?: number;
}

export async function refreshAccessToken(
  provider: string,
  refreshToken: string,
): Promise<RefreshResponse> {
  const { clientId, clientSecret, tokenEndpoint } =
    getProviderConfig(provider);

  const body = new URLSearchParams({
    grant_type: "refresh_token",
    refresh_token: refreshToken,
    client_id: clientId,
    client_secret: clientSecret,
  });

  const resp = await fetch(tokenEndpoint, {
    method: "POST",
    headers: { "Content-Type": "application/x-www-form-urlencoded" },
    body: body.toString(),
  });

  if (!resp.ok) {
    const text = await resp.text();
    throw new Error(
      `Token refresh failed for ${provider}: ${resp.status} ${text}`,
    );
  }

  return (await resp.json()) as RefreshResponse;
}

// -----------------------------------------------------------------------
// Provider user ID extraction from profile response
// -----------------------------------------------------------------------

export function extractProviderUserId(
  provider: string,
  profile: Record<string, any>,
): string | undefined {
  let raw: unknown;
  switch (provider) {
    case "spotify":
      raw = profile.id;
      break;
    case "tidal":
      raw = profile.data?.id ?? profile.id;
      break;
    case "youtube": {
      const items = profile.items as Array<Record<string, any>> | undefined;
      raw = items?.[0]?.id;
      break;
    }
    default:
      return undefined;
  }
  // Provider IDs can be numbers (e.g. Tidal) — always coerce to string
  return raw != null ? String(raw) : undefined;
}
