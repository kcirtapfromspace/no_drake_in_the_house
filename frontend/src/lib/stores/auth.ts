import { writable, derived } from 'svelte/store';
import { api } from '../utils/api';

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
      const result = await api.post('/auth/login', { 
        email, 
        password, 
        totp_code: totpCode 
      });
      
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
    } catch (error: any) {
      authStore.update(state => ({ ...state, isLoading: false }));
      return { success: false, message: error.message || 'Network error occurred' };
    }
  },

  register: async (email: string, password: string) => {
    authStore.update(state => ({ ...state, isLoading: true }));
    
    try {
      const result = await api.post('/auth/register', { email, password });
      authStore.update(state => ({ ...state, isLoading: false }));
      
      return { success: result.success, message: result.message };
    } catch (error: any) {
      authStore.update(state => ({ ...state, isLoading: false }));
      return { success: false, message: error.message || 'Network error occurred' };
    }
  },

  fetchProfile: async () => {
    const token = localStorage.getItem('auth_token');
    if (!token) return;

    try {
      const result = await api.get('/users/profile');
      
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
        await api.post('/auth/logout');
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
      const result = await api.post('/auth/refresh', { refresh_token: refreshToken });
      
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

  // 2FA Management
  setup2FA: async () => {
    try {
      const result = await api.post('/auth/2fa/setup');
      
      if (result.success) {
        return { 
          success: true, 
          qrCodeUrl: result.data.qr_code_url,
          secret: result.data.secret 
        };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      return { success: false, message: error.message || 'Failed to setup 2FA' };
    }
  },

  verify2FA: async (code: string) => {
    try {
      const result = await api.post('/auth/2fa/verify', { totp_code: code });
      
      if (result.success) {
        // Update user profile to reflect 2FA is now enabled
        await authActions.fetchProfile();
        return { success: true };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      return { success: false, message: error.message || 'Failed to verify 2FA code' };
    }
  },

  disable2FA: async (code: string) => {
    try {
      const result = await api.post('/auth/2fa/disable', { totp_code: code });
      
      if (result.success) {
        // Update user profile to reflect 2FA is now disabled
        await authActions.fetchProfile();
        return { success: true };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      return { success: false, message: error.message || 'Failed to disable 2FA' };
    }
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