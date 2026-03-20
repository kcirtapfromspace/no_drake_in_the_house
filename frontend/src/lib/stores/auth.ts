import { writable, derived } from 'svelte/store';
import { apiClient } from '../utils/api-client';
import config from '../utils/config';
import {
  clearAuthSession,
  initializeAuthSession,
  isAuth0Mode,
  loginWithAuth0,
  logoutFromAuth0,
  refreshAuthSession,
  syncAuthToken,
} from '../auth/auth0';

export interface LinkedAccount {
  provider: string;
  provider_user_id: string;
  email?: string;
  display_name?: string;
  avatar_url?: string;
  linked_at: string;
  status?: string;
  expires_at?: string;
}

export interface User {
  id: string;
  email: string;
  email_verified: boolean;
  totp_enabled: boolean;
  created_at: string;
  last_login?: string;
  oauth_accounts?: LinkedAccount[];
  display_name?: string;
  avatar_url?: string;
  legacy_user_id?: string;
}

export interface AuthState {
  user: User | null;
  token: string | null;
  refreshToken: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  justRegistered: boolean;
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
  ($auth) => $auth.isAuthenticated && $auth.token !== null,
);

export const currentUser = derived(
  authStore,
  ($auth) => $auth.user,
);

export const justRegistered = derived(
  authStore,
  ($auth) => $auth.justRegistered,
);

function resetState() {
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
}

async function fetchLinkedAccounts(): Promise<LinkedAccount[]> {
  const result = await apiClient.authenticatedRequest<LinkedAccount[]>(
    'GET',
    '/api/v1/auth/oauth/accounts',
  );
  return result.success && Array.isArray(result.data) ? result.data : [];
}

async function setAuthenticatedUser(user: User | null) {
  const token = localStorage.getItem('auth_token');
  authStore.update((state) => ({
    ...state,
    token,
    refreshToken: localStorage.getItem('refresh_token'),
    user,
    isAuthenticated: Boolean(token && user),
    isLoading: false,
  }));
}

async function beginAuth0Flow(mode: 'login' | 'register', email?: string, provider?: string) {
  authStore.update((state) => ({ ...state, isLoading: true }));
  await loginWithAuth0({
    mode,
    email,
    provider,
    returnTo: window.location.pathname,
  });
  return { success: true };
}

export const authActions = {
  login: async (email: string, password: string, totpCode?: string) => {
    if (isAuth0Mode()) {
      void password;
      void totpCode;
      return await beginAuth0Flow('login', email);
    }

    authStore.update((state) => ({ ...state, isLoading: true }));

    const result = await apiClient.post<{ access_token: string; refresh_token: string }>(
      '/api/v1/auth/login',
      {
        email,
        password,
        totp_code: totpCode,
      },
      false,
    );

    if (result.success && result.data) {
      const { access_token, refresh_token } = result.data;
      apiClient.setAuthToken(access_token);
      localStorage.setItem('refresh_token', refresh_token);

      authStore.update((state) => ({
        ...state,
        token: access_token,
        refreshToken: refresh_token,
        isAuthenticated: true,
        isLoading: false,
        justRegistered: false,
      }));

      await authActions.fetchProfile();
      return { success: true };
    }

    authStore.update((state) => ({ ...state, isLoading: false }));
    return { success: false, message: result.message || 'Login failed' };
  },

  register: async (
    email: string,
    password: string,
    confirmPassword: string,
    termsAccepted: boolean,
  ) => {
    if (isAuth0Mode()) {
      void password;
      void confirmPassword;
      void termsAccepted;
      authStore.update((state) => ({ ...state, justRegistered: true }));
      return await beginAuth0Flow('register', email);
    }

    authStore.update((state) => ({ ...state, isLoading: true }));

    const result = await apiClient.post<{ access_token?: string; refresh_token?: string; errors?: any }>(
      '/api/v1/auth/register',
      {
        email,
        password,
        confirm_password: confirmPassword,
        terms_accepted: termsAccepted,
      },
      false,
    );

    if (result.success) {
      if (result.data?.access_token && result.data?.refresh_token) {
        const { access_token, refresh_token } = result.data;
        apiClient.setAuthToken(access_token);
        localStorage.setItem('refresh_token', refresh_token);

        authStore.update((state) => ({
          ...state,
          token: access_token,
          refreshToken: refresh_token,
          isAuthenticated: true,
          isLoading: false,
          justRegistered: true,
        }));

        await authActions.fetchProfile();
        return { success: true, autoLogin: true };
      }

      authStore.update((state) => ({ ...state, isLoading: false }));
      return { success: true, autoLogin: false, message: result.message };
    }

    authStore.update((state) => ({ ...state, isLoading: false }));
    return {
      success: false,
      message: result.message || 'Registration failed',
      errors: result.data?.errors || null,
    };
  },

  fetchProfile: async () => {
    try {
      if (isAuth0Mode()) {
        const authenticated = await initializeAuthSession();
        if (!authenticated) {
          resetState();
          return;
        }

        await syncAuthToken();
      } else if (!apiClient.getAuthToken()) {
        resetState();
        return;
      }

      const result = await apiClient.authenticatedRequest<User>('GET', '/api/v1/users/profile');

      if (result.success && result.data) {
        await setAuthenticatedUser(result.data);
      } else if (isAuth0Mode()) {
        resetState();
      } else {
        console.error('Failed to fetch profile:', result.message);
      }
    } catch (error) {
      console.error('Failed to fetch profile:', error);
      if (isAuth0Mode()) {
        resetState();
      }
    }
  },

  logout: async () => {
    if (isAuth0Mode()) {
      resetState();
      await logoutFromAuth0(`${window.location.origin}${config.auth.auth0.redirectPath}`);
      return;
    }

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
    resetState();
  },

  refreshToken: async () => {
    if (isAuth0Mode()) {
      const refreshed = await refreshAuthSession();
      if (refreshed) {
        authStore.update((state) => ({
          ...state,
          token: localStorage.getItem('auth_token'),
          isAuthenticated: true,
        }));
      }
      return refreshed;
    }

    const refreshToken = localStorage.getItem('refresh_token');
    if (!refreshToken) return false;

    const result = await apiClient.post<{ access_token: string; refresh_token: string }>(
      '/api/v1/auth/refresh',
      { refresh_token: refreshToken },
      false,
    );

    if (result.success && result.data) {
      const { access_token, refresh_token: newRefreshToken } = result.data;
      apiClient.setAuthToken(access_token);
      localStorage.setItem('refresh_token', newRefreshToken);

      authStore.update((state) => ({
        ...state,
        token: access_token,
        refreshToken: newRefreshToken,
        isAuthenticated: true,
      }));

      return true;
    }

    console.error('Token refresh failed:', result.message);
    return false;
  },

  setup2FA: async () => {
    if (isAuth0Mode()) {
      return {
        success: false,
        message: 'MFA is managed in Auth0 during the Convex migration.',
      };
    }

    try {
      const result = await apiClient.authenticatedRequest<{ qr_code_url: string; secret: string }>(
        'POST',
        '/api/v1/auth/2fa/setup',
      );

      if (result.success && result.data) {
        return {
          success: true,
          qrCodeUrl: result.data.qr_code_url,
          secret: result.data.secret,
        };
      }

      return { success: false, message: result.message };
    } catch (error: any) {
      return { success: false, message: error.message || 'Failed to setup 2FA' };
    }
  },

  verify2FA: async (code: string) => {
    if (isAuth0Mode()) {
      void code;
      return {
        success: false,
        message: 'MFA verification is handled by Auth0 during sign-in.',
      };
    }

    try {
      const result = await apiClient.authenticatedRequest(
        'POST',
        '/api/v1/auth/2fa/verify',
        { totp_code: code },
      );

      if (result.success) {
        await authActions.fetchProfile();
        return { success: true };
      }

      return { success: false, message: result.message };
    } catch (error: any) {
      return { success: false, message: error.message || 'Failed to verify 2FA code' };
    }
  },

  disable2FA: async (code: string) => {
    if (isAuth0Mode()) {
      void code;
      return {
        success: false,
        message: 'MFA disablement is managed in Auth0 during the Convex migration.',
      };
    }

    try {
      const result = await apiClient.authenticatedRequest(
        'POST',
        '/api/v1/auth/2fa/disable',
        { totp_code: code },
      );

      if (result.success) {
        await authActions.fetchProfile();
        return { success: true };
      }

      return { success: false, message: result.message };
    } catch (error: any) {
      return { success: false, message: error.message || 'Failed to disable 2FA' };
    }
  },

  clearJustRegistered: () => {
    authStore.update((state) => ({
      ...state,
      justRegistered: false,
    }));
  },

  initiateOAuthFlow: async (provider: string) => {
    if (isAuth0Mode()) {
      authStore.update((state) => ({
        ...state,
        oauthFlow: {
          provider,
          state: null,
          isInProgress: true,
        },
      }));

      await loginWithAuth0({
        mode: 'login',
        provider,
        returnTo: window.location.pathname,
      });

      return { success: true };
    }

    authStore.update((state) => ({
      ...state,
      oauthFlow: {
        provider,
        state: null,
        isInProgress: true,
      },
    }));

    try {
      const result = await apiClient.post<{ authorization_url: string; state: string }>(
        `/api/v1/auth/oauth/${provider}/initiate`,
        undefined,
        false,
      );

      if (result.success && result.data) {
        const { authorization_url, state } = result.data;
        sessionStorage.setItem(`oauth_state_${provider}`, state);

        authStore.update((authState) => ({
          ...authState,
          oauthFlow: {
            ...authState.oauthFlow,
            state,
          },
        }));

        window.location.href = authorization_url;
        return { success: true };
      }

      authStore.update((state) => ({
        ...state,
        oauthFlow: {
          provider: null,
          state: null,
          isInProgress: false,
        },
      }));
      return { success: false, message: result.message };
    } catch (error: any) {
      authStore.update((state) => ({
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
      const storedState = sessionStorage.getItem(`oauth_state_${provider}`);
      if (!storedState || storedState !== state) {
        throw new Error('Invalid state parameter - possible CSRF attack');
      }

      const result = await apiClient.post<{ access_token: string; refresh_token: string }>(
        `/api/v1/auth/oauth/${provider}/callback`,
        {
          code,
          state,
          redirect_uri: window.location.origin + window.location.pathname,
        },
        false,
      );

      if (result.success && result.data) {
        const { access_token, refresh_token } = result.data;
        localStorage.setItem('auth_token', access_token);
        localStorage.setItem('refresh_token', refresh_token);

        authStore.update((authState) => ({
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

        sessionStorage.removeItem(`oauth_state_${provider}`);
        await authActions.fetchProfile();
        return { success: true };
      }

      throw new Error(result.message || 'OAuth authentication failed');
    } catch (error: any) {
      sessionStorage.removeItem(`oauth_state_${provider}`);

      authStore.update((state) => ({
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
      const result = await apiClient.authenticatedRequest<{ authorization_url: string; state: string }>(
        'POST',
        `/api/v1/auth/oauth/${provider}/link`,
      );

      if (result.success && result.data) {
        const { authorization_url, state } = result.data;
        sessionStorage.setItem(`oauth_link_state_${provider}`, state);

        return {
          success: true,
          authorization_url,
          state,
        };
      }

      return { success: false, message: result.message };
    } catch (error: any) {
      return { success: false, message: error.message || 'Network error occurred' };
    }
  },

  unlinkOAuthAccount: async (provider: string) => {
    try {
      const result = await apiClient.authenticatedRequest(
        'DELETE',
        `/api/v1/auth/oauth/${provider}/unlink`,
      );

      if (result.success) {
        await authActions.fetchProfile();
        return { success: true };
      }

      return { success: false, message: result.message };
    } catch (error: any) {
      return { success: false, message: error.message || 'Network error occurred' };
    }
  },

  getLinkedAccounts: async () => {
    try {
      const linkedAccounts = await fetchLinkedAccounts();
      return { success: true, data: linkedAccounts };
    } catch (error: any) {
      return { success: false, message: error.message || 'Network error occurred' };
    }
  },
};

if (typeof window !== 'undefined') {
  if (isAuth0Mode()) {
    initializeAuthSession()
      .then((authenticated) => {
        if (authenticated) {
          return authActions.fetchProfile();
        }
        return clearAuthSession();
      })
      .catch((error) => {
        console.error('Failed to initialize Auth0 session:', error);
      });
  } else {
    const token = localStorage.getItem('auth_token');
    if (token) {
      apiClient.setAuthToken(token);
      authStore.update((state) => ({
        ...state,
        token,
        isAuthenticated: true,
        justRegistered: false,
      }));
      void authActions.fetchProfile();
    }
  }

  window.addEventListener('auth:logout', () => {
    void authActions.logout();
  });
}
