import { writable, derived } from 'svelte/store';
import { api } from '../utils/api';

export interface LinkedAccount {
  provider: string;
  provider_user_id: string;
  email?: string;
  display_name?: string;
  avatar_url?: string;
  linked_at: string;
}

export interface User {
  id: string;
  email: string;
  email_verified: boolean;
  totp_enabled: boolean;
  created_at: string;
  last_login?: string;
  oauth_accounts?: LinkedAccount[];
}

export interface AuthState {
  user: User | null;
  token: string | null;
  refreshToken: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  justRegistered: boolean; // Track if user just completed registration
  oauthFlow: {
    provider: string | null;
    state: string | null;
    isInProgress: boolean;
  };
}

const initialState: AuthState = {
  user: null,
  token: localStorage.getItem('auth_token'),
  refreshToken: localStorage.getItem('refresh_token'),
  isAuthenticated: false,
  isLoading: false,
  justRegistered: false,
  oauthFlow: {
    provider: null,
    state: null,
    isInProgress: false,
  },
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

export const justRegistered = derived(
  authStore,
  ($auth) => $auth.justRegistered
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
          justRegistered: false, // Reset on login
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

  register: async (email: string, password: string, confirmPassword: string, termsAccepted: boolean) => {
    authStore.update(state => ({ ...state, isLoading: true }));
    
    try {
      const result = await api.post('/auth/register', { 
        email, 
        password, 
        confirm_password: confirmPassword,
        terms_accepted: termsAccepted
      });
      
      if (result.success) {
        // Check if auto-login was successful (tokens returned)
        if (result.data?.access_token && result.data?.refresh_token) {
          const { access_token, refresh_token } = result.data;
          localStorage.setItem('auth_token', access_token);
          localStorage.setItem('refresh_token', refresh_token);
          
          authStore.update(state => ({
            ...state,
            token: access_token,
            refreshToken: refresh_token,
            isAuthenticated: true,
            isLoading: false,
            justRegistered: true, // Mark as just registered for better UX
          }));
          
          // Fetch user profile
          await authActions.fetchProfile();
          return { success: true, autoLogin: true };
        } else {
          authStore.update(state => ({ ...state, isLoading: false }));
          return { success: true, autoLogin: false, message: result.message };
        }
      } else {
        authStore.update(state => ({ ...state, isLoading: false }));
        return { 
          success: false, 
          message: result.message,
          errors: result.data?.errors || null
        };
      }
    } catch (error: any) {
      authStore.update(state => ({ ...state, isLoading: false }));
      
      // Handle structured error responses
      if (error.status === 400 && error.message) {
        try {
          const errorData = JSON.parse(error.message);
          if (errorData.errors) {
            return { 
              success: false, 
              message: errorData.message || 'Registration validation failed',
              errors: errorData.errors
            };
          }
        } catch (parseError) {
          // Fall through to generic error handling
        }
      }
      
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
      justRegistered: false,
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

  // Clear the just registered flag (useful for onboarding flows)
  clearJustRegistered: () => {
    authStore.update(state => ({
      ...state,
      justRegistered: false,
    }));
  },

  // OAuth-specific actions
  initiateOAuthFlow: async (provider: string) => {
    authStore.update(state => ({
      ...state,
      oauthFlow: {
        provider,
        state: null,
        isInProgress: true,
      },
    }));

    try {
      const result = await api.post(`/auth/oauth/${provider}/initiate`);
      
      if (result.success) {
        const { authorization_url, state } = result.data;
        
        // Store state for validation
        sessionStorage.setItem(`oauth_state_${provider}`, state);
        
        authStore.update(authState => ({
          ...authState,
          oauthFlow: {
            ...authState.oauthFlow,
            state,
          },
        }));
        
        // Redirect to OAuth provider
        window.location.href = authorization_url;
        
        return { success: true };
      } else {
        authStore.update(state => ({
          ...state,
          oauthFlow: {
            provider: null,
            state: null,
            isInProgress: false,
          },
        }));
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      authStore.update(state => ({
        ...state,
        oauthFlow: {
          provider: null,
          state: null,
          isInProgress: false,
        },
      }));
      return { success: false, message: error.message || 'Network error occurred' };
    }
  },

  completeOAuthFlow: async (provider: string, code: string, state: string) => {
    try {
      // Validate state parameter
      const storedState = sessionStorage.getItem(`oauth_state_${provider}`);
      if (!storedState || storedState !== state) {
        throw new Error('Invalid state parameter - possible CSRF attack');
      }

      const result = await api.post(`/auth/oauth/${provider}/callback`, {
        code,
        state,
        redirect_uri: window.location.origin + window.location.pathname,
      });

      if (result.success) {
        const { access_token, refresh_token } = result.data;
        
        // Store tokens
        localStorage.setItem('auth_token', access_token);
        localStorage.setItem('refresh_token', refresh_token);
        
        // Update auth store
        authStore.update(authState => ({
          ...authState,
          token: access_token,
          refreshToken: refresh_token,
          isAuthenticated: true,
          oauthFlow: {
            provider: null,
            state: null,
            isInProgress: false,
          },
        }));
        
        // Clean up stored state
        sessionStorage.removeItem(`oauth_state_${provider}`);
        
        // Fetch user profile
        await authActions.fetchProfile();
        
        return { success: true };
      } else {
        throw new Error(result.message || 'OAuth authentication failed');
      }
    } catch (error: any) {
      // Clean up on error
      sessionStorage.removeItem(`oauth_state_${provider}`);
      
      authStore.update(state => ({
        ...state,
        oauthFlow: {
          provider: null,
          state: null,
          isInProgress: false,
        },
      }));
      
      return { success: false, message: error.message || 'Authentication failed' };
    }
  },

  linkOAuthAccount: async (provider: string) => {
    try {
      const result = await api.post(`/auth/oauth/${provider}/link`);
      
      if (result.success) {
        const { authorization_url, state } = result.data;
        
        // Store state for validation
        sessionStorage.setItem(`oauth_link_state_${provider}`, state);
        
        return { 
          success: true, 
          authorization_url,
          state 
        };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      return { success: false, message: error.message || 'Network error occurred' };
    }
  },

  unlinkOAuthAccount: async (provider: string) => {
    try {
      const result = await api.delete(`/auth/oauth/${provider}/unlink`);
      
      if (result.success) {
        // Refresh user profile to update linked accounts
        await authActions.fetchProfile();
        return { success: true };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      return { success: false, message: error.message || 'Network error occurred' };
    }
  },

  getLinkedAccounts: async () => {
    try {
      const result = await api.get('/auth/oauth/accounts');
      
      if (result.success) {
        return { success: true, data: result.data };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      return { success: false, message: error.message || 'Network error occurred' };
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
      justRegistered: false, // Reset on app load
    }));
    authActions.fetchProfile();
  }
}