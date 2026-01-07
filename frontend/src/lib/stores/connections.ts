import { writable, derived } from 'svelte/store';
import { apiClient, type ApiResponse } from '../utils/api-client';

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
    const response = await apiClient.authenticatedRequest<{auth_url: string}>(
      'POST',
      '/api/v1/auth/oauth/spotify/link'
    );
    
    if (response.success && response.data?.auth_url) {
      // Redirect to Spotify authorization
      window.location.href = response.data.auth_url;
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
};