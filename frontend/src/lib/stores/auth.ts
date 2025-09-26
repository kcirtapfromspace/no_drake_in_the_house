import { writable, derived } from 'svelte/store';

export interface User {
  id: string;
  email: string;
  email_verified: boolean;
  totp_enabled: boolean;
  created_at: string;
  last_login?: string;
}

export interface AuthState {
  user: User | null;
  token: string | null;
  refreshToken: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
}

const initialState: AuthState = {
  user: null,
  token: localStorage.getItem('auth_token'),
  refreshToken: localStorage.getItem('refresh_token'),
  isAuthenticated: false,
  isLoading: false,
};

export const authStore = writable<AuthState>(initialState);

export const isAuthenticated = derived(
  authStore,
  ($auth) => $auth.isAuthenticated && $auth.token !== null
);

export const currentUser = derived(
  authStore,
  ($auth) => $auth.user
);

// Auth actions
export const authActions = {
  login: async (email: string, password: string, totpCode?: string) => {
    authStore.update(state => ({ ...state, isLoading: true }));
    
    try {
      const response = await fetch('http://localhost:3000/auth/login', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ email, password, totp_code: totpCode }),
      });

      const result = await response.json();
      
      if (result.success) {
        const { access_token, refresh_token } = result.data;
        localStorage.setItem('auth_token', access_token);
        localStorage.setItem('refresh_token', refresh_token);
        
        authStore.update(state => ({
          ...state,
          token: access_token,
          refreshToken: refresh_token,
          isAuthenticated: true,
          isLoading: false,
        }));
        
        // Fetch user profile
        await authActions.fetchProfile();
        return { success: true };
      } else {
        authStore.update(state => ({ ...state, isLoading: false }));
        return { success: false, message: result.message };
      }
    } catch (error) {
      authStore.update(state => ({ ...state, isLoading: false }));
      return { success: false, message: 'Network error occurred' };
    }
  },

  register: async (email: string, password: string) => {
    authStore.update(state => ({ ...state, isLoading: true }));
    
    try {
      const response = await fetch('http://localhost:3000/auth/register', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ email, password }),
      });

      const result = await response.json();
      authStore.update(state => ({ ...state, isLoading: false }));
      
      return { success: result.success, message: result.message };
    } catch (error) {
      authStore.update(state => ({ ...state, isLoading: false }));
      return { success: false, message: 'Network error occurred' };
    }
  },

  fetchProfile: async () => {
    const token = localStorage.getItem('auth_token');
    if (!token) return;

    try {
      const response = await fetch('http://localhost:3000/auth/profile', {
        headers: {
          'Authorization': `Bearer ${token}`,
        },
      });

      const result = await response.json();
      
      if (result.success) {
        authStore.update(state => ({
          ...state,
          user: result.data,
          isAuthenticated: true,
        }));
      }
    } catch (error) {
      console.error('Failed to fetch profile:', error);
    }
  },

  logout: async () => {
    const token = localStorage.getItem('auth_token');
    
    if (token) {
      try {
        await fetch('http://localhost:3000/auth/logout', {
          method: 'POST',
          headers: {
            'Authorization': `Bearer ${token}`,
          },
        });
      } catch (error) {
        console.error('Logout request failed:', error);
      }
    }

    localStorage.removeItem('auth_token');
    localStorage.removeItem('refresh_token');
    
    authStore.set({
      user: null,
      token: null,
      refreshToken: null,
      isAuthenticated: false,
      isLoading: false,
    });
  },

  refreshToken: async () => {
    const refreshToken = localStorage.getItem('refresh_token');
    if (!refreshToken) return false;

    try {
      const response = await fetch('http://localhost:3000/auth/refresh', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ refresh_token: refreshToken }),
      });

      const result = await response.json();
      
      if (result.success) {
        const { access_token, refresh_token: newRefreshToken } = result.data;
        localStorage.setItem('auth_token', access_token);
        localStorage.setItem('refresh_token', newRefreshToken);
        
        authStore.update(state => ({
          ...state,
          token: access_token,
          refreshToken: newRefreshToken,
          isAuthenticated: true,
        }));
        
        return true;
      }
    } catch (error) {
      console.error('Token refresh failed:', error);
    }
    
    return false;
  },
};

// Initialize auth state on app load
if (typeof window !== 'undefined') {
  const token = localStorage.getItem('auth_token');
  if (token) {
    authStore.update(state => ({
      ...state,
      token,
      isAuthenticated: true,
    }));
    authActions.fetchProfile();
  }
}