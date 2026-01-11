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

const initialState: ConnectionsState = {
  connections: [],
  isLoading: false,
  error: null,
};

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

// Connection actions
export const connectionActions = {
  fetchConnections: async () => {
    connectionsStore.update(state => ({ ...state, isLoading: true, error: null }));
    
    const response = await apiClient.authenticatedRequest<ServiceConnection[]>(
      'GET',
      '/api/v1/auth/oauth/accounts'
    );
    
    if (response.success && response.data) {
      // Ensure data is an array
      const connections = Array.isArray(response.data) ? response.data : [];
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
    const response = await apiClient.authenticatedRequest<{authorization_url: string; state: string}>(
      'POST',
      '/api/v1/auth/oauth/spotify/link'
    );

    if (response.success && response.data?.authorization_url) {
      // Store state for callback validation
      sessionStorage.setItem('oauth_link_state_spotify', response.data.state);
      // Redirect to Spotify authorization
      window.location.href = response.data.authorization_url;
    } else {
      connectionsStore.update(state => ({
        ...state,
        error: response.message || 'Failed to initiate Spotify auth',
      }));
    }
  },

  handleSpotifyCallback: async (code: string, state: string) => {
    const response = await apiClient.authenticatedRequest<any>(
      'POST',
      '/api/v1/auth/oauth/spotify/link-callback',
      {
        code,
        state,
        redirect_uri: window.location.origin + window.location.pathname
      }
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
      '/api/v1/auth/oauth/spotify/unlink'
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
    // Use the OAuth health endpoint instead of Spotify-specific one
    const response = await apiClient.authenticatedRequest<any>(
      'GET',
      '/oauth/health/spotify'
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
  connectAppleMusic: async () => {
    console.log('[connections.ts] connectAppleMusic called');
    connectionsStore.update(state => ({ ...state, isLoading: true, error: null }));

    console.log('[connections.ts] Calling musicKit.connectAppleMusic()...');
    const result = await musicKit.connectAppleMusic();
    console.log('[connections.ts] musicKit.connectAppleMusic() result:', result);

    if (result.success) {
      // Refresh connections to get the new Apple Music connection
      await connectionActions.fetchConnections();
      return { success: true, connectionId: result.connectionId };
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
};