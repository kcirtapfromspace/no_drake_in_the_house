import { writable, derived } from 'svelte/store';

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
    
    try {
      const token = localStorage.getItem('auth_token');
      const response = await fetch('http://localhost:3000/api/v1/connections', {
        headers: {
          'Authorization': `Bearer ${token}`,
        },
      });

      const result = await response.json();
      
      if (result.success) {
        connectionsStore.update(state => ({
          ...state,
          connections: result.data,
          isLoading: false,
        }));
      } else {
        connectionsStore.update(state => ({
          ...state,
          error: result.message,
          isLoading: false,
        }));
      }
    } catch (error) {
      connectionsStore.update(state => ({
        ...state,
        error: 'Failed to fetch connections',
        isLoading: false,
      }));
    }
  },

  initiateSpotifyAuth: async () => {
    try {
      const token = localStorage.getItem('auth_token');
      const response = await fetch('http://localhost:3000/api/v1/spotify/auth', {
        headers: {
          'Authorization': `Bearer ${token}`,
        },
      });

      const result = await response.json();
      
      if (result.success) {
        // Redirect to Spotify authorization
        window.location.href = result.data.auth_url;
      } else {
        throw new Error(result.message);
      }
    } catch (error) {
      connectionsStore.update(state => ({
        ...state,
        error: `Failed to initiate Spotify auth: ${error instanceof Error ? error.message : 'Unknown error'}`,
      }));
    }
  },

  handleSpotifyCallback: async (code: string, state: string) => {
    try {
      const token = localStorage.getItem('auth_token');
      const response = await fetch('http://localhost:3000/api/v1/spotify/callback', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ code, state }),
      });

      const result = await response.json();
      
      if (result.success) {
        // Refresh connections
        await connectionActions.fetchConnections();
        return { success: true };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error) {
      return { success: false, message: 'Failed to complete Spotify connection' };
    }
  },

  disconnectSpotify: async () => {
    try {
      const token = localStorage.getItem('auth_token');
      const response = await fetch('http://localhost:3000/api/v1/spotify/connection', {
        method: 'DELETE',
        headers: {
          'Authorization': `Bearer ${token}`,
        },
      });

      const result = await response.json();
      
      if (result.success) {
        // Refresh connections
        await connectionActions.fetchConnections();
        return { success: true };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error) {
      return { success: false, message: 'Failed to disconnect Spotify' };
    }
  },

  checkSpotifyHealth: async () => {
    try {
      const token = localStorage.getItem('auth_token');
      const response = await fetch('http://localhost:3000/api/v1/spotify/health', {
        headers: {
          'Authorization': `Bearer ${token}`,
        },
      });

      const result = await response.json();
      
      if (result.success) {
        // Update connection status if needed
        await connectionActions.fetchConnections();
        return result.data;
      }
    } catch (error) {
      console.error('Spotify health check failed:', error);
    }
  },
};