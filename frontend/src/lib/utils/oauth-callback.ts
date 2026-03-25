export interface OAuthCallbackLocation {
  origin: string;
  pathname: string;
  search: string;
}

export interface OAuthCallbackRequest {
  code: string;
  state: string;
  redirect_uri?: string;
}

export interface OAuthCallbackResponse {
  success: boolean;
  message?: string;
}

export interface OAuthCallbackResolution {
  errorMessage: string;
  provider: string;
  request?: OAuthCallbackRequest;
  status: 'error' | 'success';
}

export function getProviderFromPath(pathname: string): string {
  const pathParts = pathname.split('/').filter(Boolean);
  return pathParts[pathParts.length - 1] || 'unknown';
}

export function getProviderName(provider: string): string {
  switch (provider) {
    case 'spotify':
      return 'Spotify';
    case 'apple':
      return 'Apple Music';
    case 'youtube':
    case 'youtube_music':
      return 'YouTube Music';
    case 'tidal':
      return 'Tidal';
    case 'google':
      return 'Google';
    case 'github':
      return 'GitHub';
    default:
      return provider;
  }
}

export function isConnectionProvider(provider: string): boolean {
  return (
    provider === 'spotify' ||
    provider === 'youtube' ||
    provider === 'youtube_music' ||
    provider === 'tidal'
  );
}

function getCallbackEndpoint(provider: string): string {
  if (provider === 'spotify' || provider === 'tidal') {
    return `/api/v1/connections/${provider}/callback`;
  }

  if (provider === 'youtube' || provider === 'youtube_music') {
    return '/api/v1/connections/youtube/callback';
  }

  return `/api/v1/auth/oauth/${provider}/link-callback`;
}

/**
 * POST to a connection callback endpoint without retry.
 * OAuth authorization codes are single-use — retrying on 5xx would burn the
 * code and cause "invalid_grant" errors from the provider.
 */
async function postCallbackNoRetry(
  url: string,
  body: OAuthCallbackRequest,
  getAuthToken: () => string | null
): Promise<OAuthCallbackResponse> {
  const headers: Record<string, string> = { 'Content-Type': 'application/json' };
  const token = getAuthToken();
  if (token) headers['Authorization'] = `Bearer ${token}`;

  const res = await fetch(url, {
    method: 'POST',
    headers,
    body: JSON.stringify(body),
  });

  const data = await res.json().catch(() => ({}));

  if (res.ok && data.success !== false) {
    return { success: true, message: data.message };
  }

  return {
    success: false,
    message: data.message || data.error || `Callback failed (${res.status})`,
  };
}

export async function resolveOAuthCallback(
  location: OAuthCallbackLocation,
  post: (url: string, body: OAuthCallbackRequest) => Promise<OAuthCallbackResponse>,
  getAuthToken?: () => string | null
): Promise<OAuthCallbackResolution> {
  const params = new URLSearchParams(location.search);
  const code = params.get('code');
  const state = params.get('state');
  const error = params.get('error');
  const errorDescription = params.get('error_description');
  const provider = getProviderFromPath(location.pathname);

  if (error) {
    return {
      status: 'error',
      provider,
      errorMessage: errorDescription || error || 'Authentication was cancelled or denied',
    };
  }

  if (!code || !state) {
    return {
      status: 'error',
      provider,
      errorMessage: 'Missing authentication parameters',
    };
  }

  const request: OAuthCallbackRequest = {
    code,
    state,
  };

  if (!isConnectionProvider(provider)) {
    request.redirect_uri = location.origin + location.pathname;
  }

  try {
    // Connection-provider callbacks (Spotify, Tidal, YouTube) use single-use
    // OAuth authorization codes that must never be retried.  Use a raw fetch
    // without retry logic for those endpoints.  Non-connection providers
    // (e.g. Google/GitHub account linking) go through the normal post callback
    // which may include retry.
    const result =
      isConnectionProvider(provider) && getAuthToken
        ? await postCallbackNoRetry(getCallbackEndpoint(provider), request, getAuthToken)
        : await post(getCallbackEndpoint(provider), request);

    if (result.success) {
      return {
        status: 'success',
        provider,
        errorMessage: '',
        request,
      };
    }

    return {
      status: 'error',
      provider,
      errorMessage: result.message || 'Failed to link account',
      request,
    };
  } catch (error) {
    return {
      status: 'error',
      provider,
      errorMessage:
        error instanceof Error ? error.message : 'An unexpected error occurred',
      request,
    };
  }
}
