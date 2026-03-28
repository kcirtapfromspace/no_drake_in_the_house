import { writable, derived } from 'svelte/store';
import { apiClient } from '../utils/api-client';
import config from '../utils/config';
import * as musicKit from '../utils/musickit';

/** Open a centered popup window over the current browser window. */
function openCenteredPopup(url: string, name: string, w = 500, h = 700): Window | null {
  const left = window.screenX + Math.round((window.outerWidth - w) / 2);
  const top = window.screenY + Math.round((window.outerHeight - h) / 2);
  return window.open(
    url, name,
    `width=${w},height=${h},left=${left},top=${top},menubar=no,toolbar=no,location=yes,status=no`
  );
}

export interface ServiceConnection {
  id: string;
  provider: string;
  provider_user_id?: string;
  scopes: string[];
  status: 'active' | 'expired' | 'error';
  health_status?: 'active' | 'expiring_soon' | 'needs_reauth' | 'error';
  expires_at?: string;
  last_health_check?: string;
  created_at: string;
  error_code?: string;
}

export interface ConnectionsState {
  connections: ServiceConnection[];
  isLoading: boolean;
  error: string | null;
}

export interface AppleLibrarySyncSummary {
  tracksCount: number;
  albumsCount: number;
  playlistsCount: number;
  message?: string;
}

interface ConnectionHealthRecord {
  id: string;
  provider: string;
  provider_user_id?: string;
  health_status?: 'active' | 'expiring_soon' | 'needs_reauth' | 'error';
  expires_at?: string;
  last_used_at?: string;
  error_message?: string;
  scopes?: string[];
}

interface ConnectionsHealthPayload {
  connections?: ConnectionHealthRecord[];
}

interface ConnectionsFetchResponse {
  success: boolean;
  data?: ConnectionsHealthPayload;
  message?: string;
  error_code?: string;
}

const initialState: ConnectionsState = {
  connections: [],
  isLoading: false,
  error: null,
};

const PROVIDER_ALIASES: Record<string, string> = {
  apple_music: 'apple_music',
  spotify: 'spotify',
  tidal: 'tidal',
  youtube: 'youtube_music',
  youtube_music: 'youtube_music',
};

function normalizeProvider(provider: string): string {
  const normalized = provider.toLowerCase();
  return PROVIDER_ALIASES[normalized] ?? normalized;
}

function mapConnectionStatus(
  status?: ConnectionHealthRecord['health_status']
): ServiceConnection['status'] {
  switch (status) {
    case 'active':
    case 'expiring_soon':
      return 'active';
    case 'needs_reauth':
      return 'expired';
    case 'error':
    default:
      return 'error';
  }
}

function mapConnectionRecord(connection: ConnectionHealthRecord): ServiceConnection {
  return {
    id: connection.id,
    provider: normalizeProvider(connection.provider),
    provider_user_id: connection.provider_user_id,
    scopes: connection.scopes ?? [],
    status: mapConnectionStatus(connection.health_status),
    health_status: connection.health_status,
    expires_at: connection.expires_at,
    last_health_check: connection.last_used_at,
    created_at: connection.last_used_at ?? connection.expires_at ?? new Date().toISOString(),
    error_code: connection.error_message,
  };
}

function normalizeConnectionsPayload(payload?: ConnectionsHealthPayload | null): ServiceConnection[] {
  const records = Array.isArray(payload?.connections) ? payload.connections : [];
  return records.map(mapConnectionRecord);
}

function emitAuthLogout(): void {
  if (typeof window === 'undefined') return;

  window.dispatchEvent(
    new CustomEvent('auth:logout', {
      detail: { reason: 'token_refresh_failed' },
    })
  );
}

async function parseConnectionsResponse(response: Response): Promise<ConnectionsFetchResponse> {
  const contentType = response.headers.get('content-type') ?? '';

  if (contentType.includes('application/json')) {
    const payload = (await response.json().catch(() => null)) as
      | ConnectionsHealthPayload
      | { message?: string }
      | null;

    if (response.ok) {
      return {
        success: true,
        data: payload && typeof payload === 'object' ? (payload as ConnectionsHealthPayload) : undefined,
      };
    }

    return {
      success: false,
      message:
        payload && typeof payload === 'object' && 'message' in payload
          ? payload.message || `HTTP ${response.status}`
          : `HTTP ${response.status}`,
      error_code: `HTTP_${response.status}`,
    };
  }

  const message = await response.text().catch(() => '');
  return {
    success: response.ok,
    message: message || `HTTP ${response.status}`,
    error_code: response.ok ? undefined : `HTTP_${response.status}`,
  };
}

async function fetchConnectionsDirect(): Promise<ConnectionsFetchResponse> {
  const requestConnections = async (): Promise<ConnectionsFetchResponse> => {
    const token = apiClient.getAuthToken();
    const headers: Record<string, string> = {};

    if (token) {
      headers.Authorization = `Bearer ${token}`;
    }

    try {
      const response = await fetch(config.resolveUrl('/api/v1/connections'), {
        method: 'GET',
        headers,
      });

      return await parseConnectionsResponse(response);
    } catch (error) {
      return {
        success: false,
        message: error instanceof Error ? error.message : 'Failed to fetch connections',
        error_code: 'NETWORK_ERROR',
      };
    }
  };

  let result = await requestConnections();

  if (!result.success && result.error_code === 'HTTP_401') {
    const refreshed = await apiClient.handleAuthError();

    if (refreshed) {
      result = await requestConnections();
    } else {
      apiClient.clearAuthToken();
      localStorage.removeItem('refresh_token');
      emitAuthLogout();
    }
  }

  return result;
}

export function isConnectionActive(connection: Pick<ServiceConnection, 'status'> | null | undefined): boolean {
  return connection?.status === 'active';
}

export function countActiveConnections(connections: ServiceConnection[]): number {
  return connections.filter((connection) => isConnectionActive(connection)).length;
}

export const connectionsStore = writable<ConnectionsState>(initialState);

export const connectedServices = derived(
  connectionsStore,
  ($connections) => $connections.connections.filter((connection) => isConnectionActive(connection))
);

export const activeConnectionCount = derived(
  connectionsStore,
  ($connections) => countActiveConnections($connections.connections)
);

export const spotifyConnection = derived(
  connectionsStore,
  ($connections) => $connections.connections.find(conn => conn.provider === 'spotify')
);

export const hasActiveSpotifyConnection = derived(
  spotifyConnection,
  ($spotify) => $spotify?.status === 'active'
);

export const appleMusicConnection = derived(
  connectionsStore,
  ($connections) => $connections.connections.find(conn => conn.provider === 'apple_music')
);

export const hasActiveAppleMusicConnection = derived(
  appleMusicConnection,
  ($apple) => $apple?.status === 'active'
);

export const tidalConnection = derived(
  connectionsStore,
  ($connections) => $connections.connections.find(conn => conn.provider === 'tidal')
);

export const hasActiveTidalConnection = derived(
  tidalConnection,
  ($tidal) => $tidal?.status === 'active'
);

export const youtubeConnection = derived(
  connectionsStore,
  ($connections) => $connections.connections.find(conn => conn.provider === 'youtube_music')
);

export const hasActiveYoutubeConnection = derived(
  youtubeConnection,
  ($youtube) => $youtube?.status === 'active'
);

// Connection actions
export const connectionActions = {
  prepareAppleMusic: async () => {
    try {
      await musicKit.configureMusicKit();
      return { success: true };
    } catch (error) {
      console.warn('Apple Music prewarm skipped:', error);
      return {
        success: false,
        message: error instanceof Error ? error.message : 'Failed to prepare Apple Music',
      };
    }
  },

  fetchConnections: async () => {
    connectionsStore.update(state => ({ ...state, isLoading: true, error: null }));

    const response = await fetchConnectionsDirect();

    if (response.success && response.data) {
      const connections = normalizeConnectionsPayload(response.data);

      connectionsStore.update(state => ({
        ...state,
        connections,
        isLoading: false,
      }));
    } else {
      const shouldClearConnections =
        response.error_code === 'HTTP_401' || response.error_code === 'HTTP_403';

      connectionsStore.update(state => ({
        ...state,
        connections: shouldClearConnections ? [] : state.connections,
        error: response.message || 'Failed to fetch connections',
        isLoading: false,
      }));
    }
  },

  initiateSpotifyAuth: async (): Promise<{ success: boolean; message?: string }> => {
    // Route through Convex OAuth so tokens are stored in Convex
    // (required for the Convex library sync to read them)
    const callbackUrl = `${window.location.origin}/auth/callback/spotify`;
    const response = await apiClient.authenticatedRequest<{
      authorization_url?: string;
      auth_url?: string;
      state?: string;
      already_connected?: boolean;
      message?: string;
    }>('POST', '/api/v1/oauth/spotify/authorize', {
      redirectUri: callbackUrl,
      scopes: [
        'user-library-read',
        'user-library-modify',
        'playlist-read-private',
        'playlist-read-collaborative',
        'playlist-modify-private',
        'user-follow-read',
        'user-follow-modify',
      ],
    });

    const authUrl = response.data?.auth_url ?? response.data?.authorization_url;
    if (response.success && authUrl) {
      if (response.data?.state) {
        sessionStorage.setItem('oauth_link_state_spotify', response.data.state);
      }
      // Popup OAuth — user stays on current page
      const popup = openCenteredPopup(authUrl, 'spotify-auth');
      if (!popup) {
        // Popup blocked — fall back to redirect
        window.location.href = authUrl;
        return { success: true };
      }
      // Poll for popup closure (callback page will close the popup)
      return new Promise((resolve) => {
        const interval = setInterval(() => {
          if (popup.closed) {
            clearInterval(interval);
            connectionActions.fetchConnections().then(() => {
              // Auto-trigger library sync after successful OAuth
              apiClient.authenticatedRequest('POST', '/api/v1/connections/spotify/library/sync')
                .catch(() => {}); // fire-and-forget; SyncDashboard polls status
            });
            resolve({ success: true });
          }
        }, 500);
        // Timeout after 5 minutes
        setTimeout(() => {
          clearInterval(interval);
          if (!popup.closed) popup.close();
          resolve({ success: false, message: 'OAuth timed out' });
        }, 300000);
      });
    } else {
      const message =
        response.data?.message || response.message || 'Failed to initiate Spotify auth';
      connectionsStore.update(state => ({
        ...state,
        error: message,
      }));
      return { success: false, message };
    }
  },

  handleSpotifyCallback: async (code: string, state: string) => {
    // OAuth authorization codes are single-use — never retry this request.
    // The default apiClient retries on 5xx which burns the single-use code.
    try {
      const token = apiClient.getAuthToken();
      const headers: Record<string, string> = { 'Content-Type': 'application/json' };
      if (token) headers['Authorization'] = `Bearer ${token}`;

      const res = await fetch('/api/v1/connections/spotify/callback', {
        method: 'POST',
        headers,
        body: JSON.stringify({ code, state }),
      });
      const data = await res.json().catch(() => ({}));
      if (res.ok && data.success !== false) {
        await connectionActions.fetchConnections();
        return { success: true };
      }
      return { success: false, message: data.message || data.error || `Spotify callback failed (${res.status})` };
    } catch (err: any) {
      return { success: false, message: err.message || 'Network error during Spotify callback' };
    }
  },

  disconnectSpotify: async () => {
    const response = await apiClient.authenticatedRequest<any>(
      'DELETE',
      '/api/v1/connections/spotify'
    );
    
    if (response.success) {
      // Refresh connections
      await connectionActions.fetchConnections();
      return { success: true };
    } else {
      return { success: false, message: response.message || 'Failed to disconnect Spotify' };
    }
  },

  checkSpotifyHealth: async () => {
    const response = await apiClient.authenticatedRequest<any>(
      'GET',
      '/api/v1/connections/spotify/status'
    );

    if (response.success) {
      // Update connection status if needed
      await connectionActions.fetchConnections();
      return response.data;
    } else {
      console.error('Spotify health check failed:', response.message);
      return null;
    }
  },

  // Apple Music actions
  connectAppleMusic: async (): Promise<{
    success: boolean;
    message?: string;
    connectionId?: string;
    syncSummary?: AppleLibrarySyncSummary;
    syncWarning?: string;
  }> => {
    console.log('[connections.ts] connectAppleMusic called');
    connectionsStore.update(state => ({ ...state, isLoading: true, error: null }));

    console.log('[connections.ts] Calling musicKit.connectAppleMusic()...');
    const result = await musicKit.connectAppleMusic();
    console.log('[connections.ts] musicKit.connectAppleMusic() result:', result);

    if (result.success) {
      // Refresh connections to get the new Apple Music connection
      await connectionActions.fetchConnections();
      connectionsStore.update(state => ({
        ...state,
        isLoading: false,
        error: null,
      }));

      return {
        success: true,
        connectionId: result.connectionId,
        message: result.message,
      };
    } else {
      connectionsStore.update(state => ({
        ...state,
        error: result.message || 'Failed to connect Apple Music',
        isLoading: false,
      }));
      return { success: false, message: result.message };
    }
  },

  disconnectAppleMusic: async () => {
    const result = await musicKit.disconnectAppleMusic();

    if (result.success) {
      // Refresh connections
      await connectionActions.fetchConnections();
      return { success: true };
    } else {
      return { success: false, message: result.message || 'Failed to disconnect Apple Music' };
    }
  },

  checkAppleMusicHealth: async () => {
    const result = await musicKit.verifyAppleMusicConnection();

    if (result.healthy) {
      // Update connection status if needed
      await connectionActions.fetchConnections();
    }

    return result;
  },

  getAppleMusicStatus: async () => {
    return await musicKit.getAppleMusicStatus();
  },

  // Tidal actions
  initiateTidalAuth: async (options?: {
    attemptReconnect?: boolean;
  }): Promise<{
    success: boolean;
    message?: string;
    alreadyConnected?: boolean;
  }> => {
    const response = await apiClient.authenticatedRequest<{
      authorization_url?: string;
      state?: string;
      already_connected?: boolean;
      message?: string;
    }>(
      'GET',
      '/api/v1/connections/tidal/authorize'
    );

    if (response.success && response.data?.authorization_url) {
      if (response.data.state) {
        sessionStorage.setItem('oauth_link_state_tidal', response.data.state);
      }
      // Popup OAuth
      const popup = openCenteredPopup(response.data.authorization_url, 'tidal-auth');
      if (!popup) {
        window.location.href = response.data.authorization_url;
        return { success: true };
      }
      return new Promise((resolve) => {
        const interval = setInterval(() => {
          if (popup.closed) {
            clearInterval(interval);
            connectionActions.fetchConnections().then(() => {
              apiClient.authenticatedRequest('POST', '/api/v1/connections/tidal/library/sync')
                .catch(() => {});
            });
            resolve({ success: true });
          }
        }, 500);
        setTimeout(() => { clearInterval(interval); if (!popup.closed) popup.close(); resolve({ success: false, message: 'OAuth timed out' }); }, 300000);
      });
    }

    const message =
      response.data?.message || response.message || 'Failed to initiate Tidal auth';
    const normalizedMessage = message.toLowerCase();
    const alreadyConnected =
      response.data?.already_connected === true ||
      (normalizedMessage.includes('already have an active') &&
        normalizedMessage.includes('tidal'));

    if (alreadyConnected) {
      const shouldAttemptReconnect = options?.attemptReconnect !== false;
      if (shouldAttemptReconnect) {
        const disconnectResult = await connectionActions.disconnectTidal();
        if (disconnectResult.success) {
          return connectionActions.initiateTidalAuth({ attemptReconnect: false });
        }
      }

      await connectionActions.fetchConnections();
      connectionsStore.update(state => ({
        ...state,
        error: null,
      }));
      return { success: false, alreadyConnected: true, message };
    }

    connectionsStore.update(state => ({
      ...state,
      error: message,
    }));
    return { success: false, message };
  },

  handleTidalCallback: async (code: string, state: string) => {
    // OAuth authorization codes are single-use — never retry this request.
    // The default apiClient retries on 5xx which burns the single-use code.
    try {
      const token = apiClient.getAuthToken();
      const headers: Record<string, string> = { 'Content-Type': 'application/json' };
      if (token) headers['Authorization'] = `Bearer ${token}`;

      const res = await fetch('/api/v1/connections/tidal/callback', {
        method: 'POST',
        headers,
        body: JSON.stringify({ code, state }),
      });
      const data = await res.json().catch(() => ({}));
      if (res.ok && data.success !== false) {
        await connectionActions.fetchConnections();
        return { success: true };
      }
      return { success: false, message: data.message || data.error || `Tidal callback failed (${res.status})` };
    } catch (err: any) {
      return { success: false, message: err.message || 'Network error during Tidal callback' };
    }
  },

  disconnectTidal: async () => {
    const response = await apiClient.authenticatedRequest<any>(
      'DELETE',
      '/api/v1/connections/tidal'
    );

    if (response.success) {
      await connectionActions.fetchConnections();
      return { success: true };
    }

    return {
      success: false,
      message: response.message || 'Failed to disconnect Tidal',
    };
  },

  getTidalStatus: async () => {
    const response = await apiClient.authenticatedRequest<any>(
      'GET',
      '/api/v1/connections/tidal/status'
    );

    if (response.success) {
      return response.data;
    }

    return { connected: false };
  },

  // YouTube Music actions
  initiateYouTubeAuth: async (): Promise<{ success: boolean; message?: string }> => {
    const response = await apiClient.authenticatedRequest<{
      authorization_url: string;
      state: string;
    }>(
      'GET',
      '/api/v1/connections/youtube/authorize'
    );

    if (response.success && response.data?.authorization_url) {
      sessionStorage.setItem('oauth_link_state_youtube', response.data.state);
      // Popup OAuth
      const popup = openCenteredPopup(response.data.authorization_url, 'youtube-auth');
      if (!popup) {
        window.location.href = response.data.authorization_url;
        return { success: true };
      }
      return new Promise((resolve) => {
        const interval = setInterval(() => {
          if (popup.closed) {
            clearInterval(interval);
            connectionActions.fetchConnections().then(() => {
              apiClient.authenticatedRequest('POST', '/api/v1/connections/youtube/library/sync')
                .catch(() => {});
            });
            resolve({ success: true });
          }
        }, 500);
        setTimeout(() => { clearInterval(interval); if (!popup.closed) popup.close(); resolve({ success: false, message: 'OAuth timed out' }); }, 300000);
      });
    }

    connectionsStore.update(state => ({
      ...state,
      error: response.message || 'Failed to initiate YouTube Music auth',
    }));
    return { success: false, message: response.message };
  },

  handleYouTubeCallback: async (code: string, state: string) => {
    // OAuth authorization codes are single-use — never retry this request.
    // The default apiClient retries on 5xx which burns the single-use code.
    try {
      const token = apiClient.getAuthToken();
      const headers: Record<string, string> = { 'Content-Type': 'application/json' };
      if (token) headers['Authorization'] = `Bearer ${token}`;

      const res = await fetch('/api/v1/connections/youtube/callback', {
        method: 'POST',
        headers,
        body: JSON.stringify({ code, state }),
      });
      const data = await res.json().catch(() => ({}));
      if (res.ok && data.success !== false) {
        await connectionActions.fetchConnections();
        return { success: true };
      }
      return { success: false, message: data.message || data.error || `YouTube callback failed (${res.status})` };
    } catch (err: any) {
      return { success: false, message: err.message || 'Network error during YouTube callback' };
    }
  },

  disconnectYouTube: async () => {
    const response = await apiClient.authenticatedRequest<any>(
      'DELETE',
      '/api/v1/connections/youtube'
    );

    if (response.success) {
      await connectionActions.fetchConnections();
      return { success: true };
    }

    return {
      success: false,
      message: response.message || 'Failed to disconnect YouTube Music',
    };
  },

  getYouTubeStatus: async () => {
    const response = await apiClient.authenticatedRequest<any>(
      'GET',
      '/api/v1/connections/youtube/status'
    );

    if (response.success) {
      return response.data;
    }

    return { connected: false };
  },
};
