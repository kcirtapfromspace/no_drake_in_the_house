import { describe, it, expect, beforeEach, vi } from 'vitest';
import { get } from 'svelte/store';

const mocks = vi.hoisted(() => ({
  apiClient: {
    post: vi.fn(),
    authenticatedRequest: vi.fn(),
    setAuthToken: vi.fn(),
    getAuthToken: vi.fn(),
    clearAuthToken: vi.fn(),
  },
}));

vi.mock('../../utils/api-client', () => ({
  apiClient: mocks.apiClient,
}));

type AuthModule = typeof import('../../stores/auth');
let authStore: AuthModule['authStore'];
let authActions: AuthModule['authActions'];

const localData: Record<string, string> = {};
const sessionData: Record<string, string> = {};

const localStorageMock = {
  getItem: vi.fn((key: string) => localData[key] ?? null),
  setItem: vi.fn((key: string, value: string) => {
    localData[key] = String(value);
  }),
  removeItem: vi.fn((key: string) => {
    delete localData[key];
  }),
  clear: vi.fn(() => {
    Object.keys(localData).forEach((key) => delete localData[key]);
  }),
};

const sessionStorageMock = {
  getItem: vi.fn((key: string) => sessionData[key] ?? null),
  setItem: vi.fn((key: string, value: string) => {
    sessionData[key] = String(value);
  }),
  removeItem: vi.fn((key: string) => {
    delete sessionData[key];
  }),
  clear: vi.fn(() => {
    Object.keys(sessionData).forEach((key) => delete sessionData[key]);
  }),
};

const mockLocation = {
  href: '',
  origin: 'http://localhost:3000',
  pathname: '/',
  search: '',
};

Object.defineProperty(window, 'localStorage', {
  value: localStorageMock,
});

Object.defineProperty(window, 'sessionStorage', {
  value: sessionStorageMock,
});

Object.defineProperty(window, 'location', {
  value: mockLocation,
  writable: true,
});

const resetAuthState = () => ({
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

describe('OAuth Integration Tests', () => {
  beforeEach(async () => {
    vi.clearAllMocks();
    vi.resetModules();

    Object.keys(localData).forEach((key) => delete localData[key]);
    Object.keys(sessionData).forEach((key) => delete sessionData[key]);

    mockLocation.href = '';
    mockLocation.origin = 'http://localhost:3000';
    mockLocation.pathname = '/';
    mockLocation.search = '';

    mocks.apiClient.getAuthToken.mockImplementation(() => localStorageMock.getItem('auth_token'));

    const authModule = await import('../../stores/auth');
    authStore = authModule.authStore;
    authActions = authModule.authActions;
    authStore.set(resetAuthState());
  });

  it('initiates an oauth flow and stores the provider state', async () => {
    mocks.apiClient.post.mockResolvedValueOnce({
      success: true,
      data: {
        authorization_url: 'https://accounts.google.com/oauth/authorize?client_id=test',
        state: 'test-state-token',
      },
    });

    const result = await authActions.initiateOAuthFlow('google');

    expect(result).toEqual({ success: true });
    expect(mocks.apiClient.post).toHaveBeenCalledWith(
      '/api/v1/auth/oauth/google/initiate',
      undefined,
      false
    );
    expect(sessionStorageMock.setItem).toHaveBeenCalledWith('oauth_state_google', 'test-state-token');
    expect(mockLocation.href).toBe('https://accounts.google.com/oauth/authorize?client_id=test');

    expect(get(authStore).oauthFlow).toEqual({
      provider: 'google',
      state: 'test-state-token',
      isInProgress: true,
    });
  });

  it('resets oauth state when initiation fails', async () => {
    mocks.apiClient.post.mockRejectedValueOnce(new Error('Network error'));

    const result = await authActions.initiateOAuthFlow('google');

    expect(result).toEqual({
      success: false,
      message: 'Network error',
    });
    expect(get(authStore).oauthFlow).toEqual({
      provider: null,
      state: null,
      isInProgress: false,
    });
  });

  it('completes the oauth callback flow and authenticates the user', async () => {
    sessionStorageMock.setItem('oauth_state_google', 'test-state');
    mocks.apiClient.post.mockResolvedValueOnce({
      success: true,
      data: {
        access_token: 'test-access-token',
        refresh_token: 'test-refresh-token',
      },
    });
    mocks.apiClient.authenticatedRequest.mockResolvedValueOnce({
      success: true,
      data: {
        id: '1',
        email: 'test@example.com',
        email_verified: true,
        totp_enabled: false,
        created_at: '2024-01-01T00:00:00Z',
      },
    });

    const result = await authActions.completeOAuthFlow('google', 'test-code', 'test-state');

    expect(result).toEqual({ success: true });
    expect(mocks.apiClient.post).toHaveBeenCalledWith('/api/v1/auth/oauth/google/callback', {
      code: 'test-code',
      state: 'test-state',
      redirect_uri: 'http://localhost:3000/',
    }, false);
    expect(localStorageMock.setItem).toHaveBeenCalledWith('auth_token', 'test-access-token');
    expect(localStorageMock.setItem).toHaveBeenCalledWith('refresh_token', 'test-refresh-token');
    expect(sessionStorageMock.removeItem).toHaveBeenCalledWith('oauth_state_google');
    expect(get(authStore).isAuthenticated).toBe(true);
    expect(get(authStore).oauthFlow).toEqual({
      provider: null,
      state: null,
      isInProgress: false,
    });
  });

  it('rejects oauth completion when the state parameter does not match', async () => {
    sessionStorageMock.setItem('oauth_state_google', 'valid-state');

    const result = await authActions.completeOAuthFlow('google', 'test-code', 'invalid-state');

    expect(result.success).toBe(false);
    expect(result.message).toContain('Invalid state parameter');
    expect(mocks.apiClient.post).not.toHaveBeenCalled();
    expect(get(authStore).oauthFlow).toEqual({
      provider: null,
      state: null,
      isInProgress: false,
    });
  });

  it('initiates account linking through the authenticated API path', async () => {
    mocks.apiClient.authenticatedRequest.mockResolvedValueOnce({
      success: true,
      data: {
        authorization_url: 'https://accounts.google.com/oauth/authorize?client_id=test',
        state: 'link-state-token',
      },
    });

    const result = await authActions.linkOAuthAccount('google');

    expect(result).toEqual({
      success: true,
      authorization_url: 'https://accounts.google.com/oauth/authorize?client_id=test',
      state: 'link-state-token',
    });
    expect(mocks.apiClient.authenticatedRequest).toHaveBeenCalledWith(
      'POST',
      '/api/v1/auth/oauth/google/link'
    );
    expect(sessionStorageMock.setItem).toHaveBeenCalledWith('oauth_link_state_google', 'link-state-token');
  });

  it('unlinks an oauth account and refreshes the profile', async () => {
    localStorageMock.setItem('auth_token', 'test-access-token');
    mocks.apiClient.authenticatedRequest
      .mockResolvedValueOnce({ success: true })
      .mockResolvedValueOnce({
        success: true,
        data: {
          id: '1',
          email: 'test@example.com',
          email_verified: true,
          totp_enabled: false,
          created_at: '2024-01-01T00:00:00Z',
          oauth_accounts: [],
        },
      });

    const result = await authActions.unlinkOAuthAccount('google');

    expect(result).toEqual({ success: true });
    expect(mocks.apiClient.authenticatedRequest).toHaveBeenNthCalledWith(
      1,
      'DELETE',
      '/api/v1/auth/oauth/google/unlink'
    );
    expect(mocks.apiClient.authenticatedRequest).toHaveBeenNthCalledWith(
      2,
      'GET',
      '/api/v1/users/profile'
    );
  });

  it('returns the linked accounts list from the authenticated API path', async () => {
    const accounts = [
      {
        provider: 'google',
        provider_user_id: 'google123',
        email: 'test@gmail.com',
        display_name: 'Test User',
        linked_at: '2023-01-01T00:00:00Z',
      },
    ];

    mocks.apiClient.authenticatedRequest.mockResolvedValueOnce({
      success: true,
      data: accounts,
    });

    const result = await authActions.getLinkedAccounts();

    expect(result).toEqual({
      success: true,
      data: accounts,
    });
    expect(mocks.apiClient.authenticatedRequest).toHaveBeenCalledWith(
      'GET',
      '/api/v1/auth/oauth/accounts'
    );
  });
});
