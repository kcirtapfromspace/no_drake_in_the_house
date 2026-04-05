import { writable, derived } from 'svelte/store';
import { apiClient } from '../utils/api-client';
import { maybeHandleConvexRoute } from '../convex/bridge';
import * as musicKit from '../utils/musickit';

/** Channel name used by the OAuth popup to signal completion back to the opener. */
export const OAUTH_BROADCAST_CHANNEL = 'oauth-callback';

/** Open a centered popup window over the current browser window. */
function openCenteredPopup(url: string, name: string, w = 500, h = 700): Window | null {
  const left = window.screenX + Math.round((window.outerWidth - w) / 2);
  const top = window.screenY + Math.round((window.outerHeight - h) / 2);
  return window.open(
    url, name,
    `width=${w},height=${h},left=${left},top=${top},menubar=no,toolbar=no,location=yes,status=no`
  );
}

/**
 * Run an OAuth popup flow with COOP-safe completion detection.
 *
 * Google's OAuth pages set Cross-Origin-Opener-Policy: same-origin which
 * severs the window.opener link, making popup.closed polling unreliable.
 * We use BroadcastChannel as the primary signal (same-origin, unaffected
 * by COOP) and fall back to popup.closed polling.
 */
function runOAuthPopup(
  authUrl: string,
  popupName: string,
  onSuccess: () => void,
): Promise<{ success: boolean; message?: string }> {
  const popup = openCenteredPopup(authUrl, popupName);
  if (!popup) {
    window.location.href = authUrl;
    return Promise.resolve({ success: true });
  }

  return new Promise((resolve) => {
    let settled = false;
    const settle = (result: { success: boolean; message?: string }) => {
      if (settled) return;
      settled = true;
      clearInterval(closedPoll);
      clearTimeout(timeout);
      channel.close();
      resolve(result);
    };

    // Primary: BroadcastChannel message from the callback page
    const channel = new BroadcastChannel(OAUTH_BROADCAST_CHANNEL);
    channel.onmessage = (event) => {
      if (event.data?.type === 'oauth-complete') {
        onSuccess();
        settle({ success: true });
      }
    };

    // Fallback: poll popup.closed (works when COOP doesn't block it)
    const closedPoll = setInterval(() => {
      try {
        if (popup.closed) {
          onSuccess();
          settle({ success: true });
        }
      } catch {
        // Cross-origin access error — ignore, rely on BroadcastChannel
      }
    }, 500);

    // Timeout after 5 minutes
    const timeout = setTimeout(() => {
      try { if (!popup.closed) popup.close(); } catch { /* ignore */ }
      settle({ success: false, message: 'OAuth timed out' });
    }, 300000);
  });
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
const spotifyRefreshAttempts = new Set<string>();

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

async function fetchConnectionsDirect(): Promise<ConnectionsFetchResponse> {
  // Route through the apiClient so the Convex bridge can intercept.
  // Convex is the canonical data store for connections.
  try {
    const response = await apiClient.authenticatedRequest<ConnectionsHealthPayload>(
      'GET',
      '/api/v1/connections',
    );

    if (response.success && response.data) {
      return { success: true, data: response.data };
    }

    return {
      success: false,
      message: response.message || 'Failed to fetch connections',
      error_code: response.error_code,
    };
  } catch (error) {
    return {
      success: false,
      message: error instanceof Error ? error.message : 'Failed to fetch connections',
      error_code: 'NETWORK_ERROR',
    };
  }
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

      // If Spotify recovered to active, allow future refresh attempts for the same row.
      for (const connection of connections) {
        if (connection.provider === 'spotify' && connection.status === 'active') {
          spotifyRefreshAttempts.delete(connection.id);
        }
      }

      // Auto-refresh disabled: stale connections should be disconnected and
      // reconnected via the OAuth flow rather than silently refreshed in the
      // background (which 502s when the backend token vault is out of sync).
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
        'playlist-modify-public',
        'user-follow-read',
        'user-follow-modify',
        'user-read-private',
        'user-read-email',
        'user-read-recently-played',
        'user-top-read',
        'user-read-playback-state',
        'user-read-currently-playing',
      ],
    });

    const authUrl = response.data?.auth_url ?? response.data?.authorization_url;
    if (response.success && authUrl) {
      if (response.data?.state) {
        sessionStorage.setItem('oauth_link_state_spotify', response.data.state);
      }
      return runOAuthPopup(authUrl, 'spotify-auth', () => {
        connectionActions.fetchConnections().then(() => {
          apiClient.authenticatedRequest('POST', '/api/v1/connections/spotify/library/sync')
            .catch(() => {}); // fire-and-forget; SyncDashboard polls status
        });
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
    // Route through Convex bridge first so tokens are stored in Convex
    // (the bridge does not retry, so the single-use code is safe).
    // Fall back to raw fetch only if the bridge is not available.
    try {
      const bridged = await maybeHandleConvexRoute<any>('POST', '/api/v1/connections/spotify/callback', { code, state });
      if (bridged) {
        if (bridged.success) {
          await connectionActions.fetchConnections();
          return { success: true };
        }
        return { success: false, message: bridged.message || 'Spotify callback failed via Convex' };
      }

      // Fallback: raw fetch to Rust backend (legacy path)
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
  initiateTidalAuth: async (): Promise<{ success: boolean; message?: string }> => {
    const callbackUrl = `${window.location.origin}/auth/callback/tidal`;
    const response = await apiClient.authenticatedRequest<{
      authorization_url?: string;
      auth_url?: string;
      state?: string;
      code_verifier?: string;
      already_connected?: boolean;
      message?: string;
    }>(
      'POST',
      '/api/v1/oauth/tidal/authorize',
      {
        redirectUri: callbackUrl,
        scopes: ['user.read', 'collection.read', 'collection.write', 'playlists.read', 'playlists.write'],
      }
    );

    const authUrl = response.data?.auth_url ?? response.data?.authorization_url;
    if (response.success && authUrl) {
      if (response.data?.state) {
        sessionStorage.setItem('oauth_link_state_tidal', response.data.state);
      }
      // Store PKCE code_verifier for the callback.
      // Use localStorage (not sessionStorage) because the OAuth popup is a
      // separate window whose sessionStorage is isolated from the opener.
      if (response.data?.code_verifier) {
        localStorage.setItem('oauth_code_verifier_tidal', response.data.code_verifier);
      }
      return runOAuthPopup(authUrl, 'tidal-auth', () => {
        connectionActions.fetchConnections().then(() => {
          apiClient.authenticatedRequest('POST', '/api/v1/connections/tidal/library/sync')
            .catch(() => {}); // fire-and-forget; SyncDashboard polls status
        });
      });
    } else {
      const message =
        response.data?.message || response.message || 'Failed to initiate Tidal auth';
      connectionsStore.update(state => ({
        ...state,
        error: message,
      }));
      return { success: false, message };
    }
  },

  handleTidalCallback: async (code: string, state: string) => {
    // OAuth authorization codes are single-use — route through Convex bridge first.
    const codeVerifier = localStorage.getItem('oauth_code_verifier_tidal') || undefined;
    localStorage.removeItem('oauth_code_verifier_tidal');
    try {
      const bridged = await maybeHandleConvexRoute<any>('POST', '/api/v1/connections/tidal/callback', { code, state, codeVerifier });
      if (bridged) {
        if (bridged.success) {
          await connectionActions.fetchConnections();
          return { success: true };
        }
        return { success: false, message: bridged.message || 'Tidal callback failed via Convex' };
      }

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
    const callbackUrl = `${window.location.origin}/auth/callback/youtube`;
    const response = await apiClient.authenticatedRequest<{
      authorization_url?: string;
      auth_url?: string;
      state?: string;
      already_connected?: boolean;
      message?: string;
    }>(
      'POST',
      '/api/v1/oauth/youtube/authorize',
      {
        redirectUri: callbackUrl,
        scopes: [
          'openid', 'email', 'profile',
          'https://www.googleapis.com/auth/youtube',
          'https://www.googleapis.com/auth/youtube.force-ssl',
          'https://www.googleapis.com/auth/youtube.readonly',
        ],
      }
    );

    const authUrl = response.data?.auth_url ?? response.data?.authorization_url;
    if (response.success && authUrl) {
      if (response.data?.state) {
        sessionStorage.setItem('oauth_link_state_youtube', response.data.state);
      }
      return runOAuthPopup(authUrl, 'youtube-auth', () => {
        connectionActions.fetchConnections().then(() => {
          apiClient.authenticatedRequest('POST', '/api/v1/connections/youtube/library/sync')
            .catch(() => {}); // fire-and-forget; SyncDashboard polls status
        });
      });
    } else {
      const message =
        response.data?.message || response.message || 'Failed to initiate YouTube Music auth';
      connectionsStore.update(state => ({
        ...state,
        error: message,
      }));
      return { success: false, message };
    }
  },

  handleYouTubeCallback: async (code: string, state: string) => {
    // OAuth authorization codes are single-use — route through Convex bridge first.
    try {
      const bridged = await maybeHandleConvexRoute<any>('POST', '/api/v1/connections/youtube/callback', { code, state });
      if (bridged) {
        if (bridged.success) {
          await connectionActions.fetchConnections();
          return { success: true };
        }
        return { success: false, message: bridged.message || 'YouTube callback failed via Convex' };
      }

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
