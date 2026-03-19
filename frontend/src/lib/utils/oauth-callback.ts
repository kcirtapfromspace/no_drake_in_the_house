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

export async function resolveOAuthCallback(
  location: OAuthCallbackLocation,
  post: (url: string, body: OAuthCallbackRequest) => Promise<OAuthCallbackResponse>
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

  const request = {
    code,
    state,
  };

  if (!isConnectionProvider(provider)) {
    request.redirect_uri = location.origin + location.pathname;
  }

  try {
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
