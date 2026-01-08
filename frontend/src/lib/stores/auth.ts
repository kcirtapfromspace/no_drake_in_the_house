import { writable, derived } from 'svelte/store';
import { apiClient } from '../utils/api-client';

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
    
    const result = await apiClient.post<{access_token: string, refresh_token: string}>(
      '/api/v1/auth/login', 
      { 
        email, 
        password, 
        totp_code: totpCode 
      },
      false // Don't include auth for login
    );
    
    if (result.success && result.data) {
      const { access_token, refresh_token } = result.data;
      apiClient.setAuthToken(access_token);
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
      return { success: false, message: result.message || 'Login failed' };
    }
  },

  register: async (email: string, password: string, confirmPassword: string, termsAccepted: boolean) => {
    authStore.update(state => ({ ...state, isLoading: true }));
    
    const result = await apiClient.post<{access_token?: string, refresh_token?: string, errors?: any}>(
      '/api/v1/auth/register', 
      { 
        email, 
        password, 
        confirm_password: confirmPassword,
        terms_accepted: termsAccepted
      },
      false // Don't include auth for registration
    );
    
    if (result.success) {
      // Check if auto-login was successful (tokens returned)
      if (result.data?.access_token && result.data?.refresh_token) {
        const { access_token, refresh_token } = result.data;
        apiClient.setAuthToken(access_token);
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
        message: result.message || 'Registration failed',
        errors: result.data?.errors || null
      };
    }
  },

  fetchProfile: async () => {
    const token = apiClient.getAuthToken();
    if (!token) return;

    const result = await apiClient.authenticatedRequest<User>(
      'GET',
      '/api/v1/users/profile'
    );
    
    if (result.success && result.data) {
      authStore.update(state => ({
        ...state,
        user: result.data!,
        isAuthenticated: true,
      }));
    } else {
      console.error('Failed to fetch profile:', result.message);
    }
  },

  logout: async () => {
    const token = apiClient.getAuthToken();
    
    if (token) {
      try {
        await apiClient.authenticatedRequest('POST', '/api/v1/auth/logout');
      } catch (error) {
        console.error('Logout request failed:', error);
      }
    }

    apiClient.clearAuthToken();
    localStorage.removeItem('refresh_token');
    
    authStore.set({
      user: null,
      token: null,
      refreshToken: null,
      isAuthenticated: false,
      isLoading: false,
      justRegistered: false,
      oauthFlow: {
        provider: null,
        state: null,
        isInProgress: false,
      },
    });
  },

  refreshToken: async () => {
    const refreshToken = localStorage.getItem('refresh_token');
    if (!refreshToken) return false;

    const result = await apiClient.post<{access_token: string, refresh_token: string}>(
      '/api/v1/auth/refresh', 
      { refresh_token: refreshToken },
      false // Don't include auth for refresh
    );
    
    if (result.success && result.data) {
      const { access_token, refresh_token: newRefreshToken } = result.data;
      apiClient.setAuthToken(access_token);
      localStorage.setItem('refresh_token', newRefreshToken);
      
      authStore.update(state => ({
        ...state,
        token: access_token,
        refreshToken: newRefreshToken,
        isAuthenticated: true,
      }));
      
      return true;
    } else {
      console.error('Token refresh failed:', result.message);
      return false;
    }
  },

  // 2FA Management
  setup2FA: async () => {
    try {
      const result = await apiClient.authenticatedRequest<{qr_code_url: string, secret: string}>('POST', '/api/v1/auth/2fa/setup');

      if (result.success && result.data) {
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
      const result = await apiClient.authenticatedRequest('POST', '/api/v1/auth/2fa/verify', { totp_code: code });

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
      const result = await apiClient.authenticatedRequest('POST', '/api/v1/auth/2fa/disable', { totp_code: code });

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
      const result = await apiClient.post<{authorization_url: string, state: string}>(`/api/v1/auth/oauth/${provider}/initiate`, undefined, false);

      if (result.success && result.data) {
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

      const result = await apiClient.post<{access_token: string, refresh_token: string}>(`/api/v1/auth/oauth/${provider}/callback`, {
        code,
        state,
        redirect_uri: window.location.origin + window.location.pathname,
      }, false);

      if (result.success && result.data) {
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
      const result = await apiClient.authenticatedRequest<{authorization_url: string, state: string}>('POST', `/api/v1/auth/oauth/${provider}/link`);

      if (result.success && result.data) {
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
      const result = await apiClient.authenticatedRequest('DELETE', `/api/v1/auth/oauth/${provider}/unlink`);

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
      const result = await apiClient.authenticatedRequest<LinkedAccount[]>('GET', '/api/v1/auth/oauth/accounts');

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
    apiClient.setAuthToken(token);
    authStore.update(state => ({
      ...state,
      token,
      isAuthenticated: true,
      justRegistered: false, // Reset on app load
    }));
    authActions.fetchProfile();
  }
  
  // Listen for auth logout events from API client
  window.addEventListener('auth:logout', () => {
    authActions.logout();
  });
}