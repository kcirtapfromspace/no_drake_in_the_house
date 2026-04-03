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
  ],
  tidal: ["user.read", "collection.read", "playlists.read"],
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
  tidal: "https://openapi.tidal.com/v2/users/me",
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
 * Resolve the redirect URI for a provider. Provider-specific env var takes
 * precedence, then YouTube-specific variants, then the caller-supplied value.
 */
export function resolveRedirectUri(
  provider: string,
  callerUri: string | undefined,
): string {
  const key = `${provider.toUpperCase()}_REDIRECT_URI`;
  const envUri = process.env[key];
  if (envUri) return envUri;

  if (provider === "youtube") {
    const alt =
      process.env.YOUTUBE_MUSIC_REDIRECT_URI ||
      process.env.GOOGLE_REDIRECT_URI;
    if (alt) return alt;
  }

  return callerUri || "";
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
  switch (provider) {
    case "spotify":
      return profile.id;
    case "tidal":
      return profile.data?.id ?? profile.id;
    case "youtube": {
      const items = profile.items as Array<Record<string, any>> | undefined;
      return items?.[0]?.id;
    }
    default:
      return undefined;
  }
}
