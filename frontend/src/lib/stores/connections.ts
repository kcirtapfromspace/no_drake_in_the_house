import { writable, derived } from 'svelte/store';
import { apiClient } from '../utils/api-client';
import * as musicKit from '../utils/musickit';

export interface ServiceConnection {
  id: string;
  provider: string;
  provider_user_id?: string;
  scopes: string[];
  status: 'active' | 'expired' | 'error';
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

export const connectionsStore = writable<ConnectionsState>(initialState);

export const connectedServices = derived(
  connectionsStore,
  ($connections) => $connections.connections.filter(conn => conn.status === 'active')
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

    const response = await apiClient.authenticatedRequest<ConnectionsHealthPayload>(
      'GET',
      '/api/v1/connections'
    );

    if (response.success && response.data) {
      const payload = response.data as ConnectionsHealthPayload;
      const records = Array.isArray(payload.connections) ? payload.connections : [];
      const connections: ServiceConnection[] = records.map((connection) => ({
        id: connection.id,
        provider: normalizeProvider(connection.provider),
        provider_user_id: connection.provider_user_id,
        scopes: connection.scopes ?? [],
        status: mapConnectionStatus(connection.health_status),
        expires_at: connection.expires_at,
        last_health_check: connection.last_used_at,
        created_at: connection.last_used_at ?? connection.expires_at ?? new Date().toISOString(),
        error_code: connection.error_message,
      }));

      connectionsStore.update(state => ({
        ...state,
        connections,
        isLoading: false,
      }));
    } else {
      connectionsStore.update(state => ({
        ...state,
        connections: [], // Reset to empty array on error
        error: response.message || 'Failed to fetch connections',
        isLoading: false,
      }));
    }
  },

  initiateSpotifyAuth: async () => {
    const response = await apiClient.authenticatedRequest<{
      authorization_url?: string;
      state?: string;
      already_connected?: boolean;
      message?: string;
    }>('GET', '/api/v1/connections/spotify/authorize');

    if (response.success && response.data?.authorization_url) {
      if (response.data.state) {
        sessionStorage.setItem('oauth_link_state_spotify', response.data.state);
      }
      window.location.href = response.data.authorization_url;
      return { success: true };
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
    const response = await apiClient.authenticatedRequest<any>(
      'POST',
      '/api/v1/connections/spotify/callback',
      { code, state }
    );
    
    if (response.success) {
      // Refresh connections
      await connectionActions.fetchConnections();
      return { success: true };
    } else {
      return { success: false, message: response.message || 'Failed to complete Spotify connection' };
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
      window.location.href = response.data.authorization_url;
      return { success: true };
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
    const response = await apiClient.authenticatedRequest<any>(
      'POST',
      '/api/v1/connections/tidal/callback',
      { code, state }
    );

    if (response.success) {
      await connectionActions.fetchConnections();
      return { success: true };
    }

    return {
      success: false,
      message: response.message || 'Failed to complete Tidal connection',
    };
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
  initiateYouTubeAuth: async () => {
    const response = await apiClient.authenticatedRequest<{
      authorization_url: string;
      state: string;
    }>(
      'GET',
      '/api/v1/connections/youtube/authorize'
    );

    if (response.success && response.data?.authorization_url) {
      sessionStorage.setItem('oauth_link_state_youtube', response.data.state);
      window.location.href = response.data.authorization_url;
      return { success: true };
    }

    connectionsStore.update(state => ({
      ...state,
      error: response.message || 'Failed to initiate YouTube Music auth',
    }));
    return { success: false, message: response.message };
  },

  handleYouTubeCallback: async (code: string, state: string) => {
    const response = await apiClient.authenticatedRequest<any>(
      'POST',
      '/api/v1/connections/youtube/callback',
      { code, state }
    );

    if (response.success) {
      await connectionActions.fetchConnections();
      return { success: true };
    }

    return {
      success: false,
      message: response.message || 'Failed to complete YouTube Music connection',
    };
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
