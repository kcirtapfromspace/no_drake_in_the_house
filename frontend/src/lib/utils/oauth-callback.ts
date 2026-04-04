export interface OAuthCallbackLocation {
  origin: string;
  pathname: string;
  search: string;
}

export interface OAuthCallbackRequest {
  code: string;
  state: string;
  redirect_uri?: string;
  codeVerifier?: string;
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
  // Connection providers route through Convex OAuth so tokens are stored
  // where the Convex library sync can read them.
  if (provider === 'spotify' || provider === 'tidal') {
    return `/api/v1/oauth/${provider}/callback`;
  }

  if (provider === 'youtube' || provider === 'youtube_music') {
    return '/api/v1/oauth/youtube/callback';
  }

  return `/api/v1/auth/oauth/${provider}/link-callback`;
}

export async function resolveOAuthCallback(
  location: OAuthCallbackLocation,
  post: (url: string, body: OAuthCallbackRequest) => Promise<OAuthCallbackResponse>,
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

  // Retrieve PKCE code_verifier if stored during authorize (e.g. Tidal)
  const codeVerifier =
    (typeof sessionStorage !== 'undefined' &&
      sessionStorage.getItem(`oauth_code_verifier_${provider}`)) ||
    undefined;
  if (codeVerifier && typeof sessionStorage !== 'undefined') {
    sessionStorage.removeItem(`oauth_code_verifier_${provider}`);
  }

  const request: OAuthCallbackRequest = {
    code,
    state,
    redirect_uri: location.origin + location.pathname,
    codeVerifier,
  };

  try {
    // Connection providers route through the Convex bridge (which calls
    // the action directly, no HTTP retry).  The API client only retries
    // on 5xx (never 4xx), so single-use OAuth codes are safe.
    const result = await post(getCallbackEndpoint(provider), request);

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
