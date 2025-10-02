import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import Login from '../Login.svelte';
import { authStore, authActions } from '../../stores/auth';

// Mock the API module
const mockApi = {
  post: vi.fn(),
  get: vi.fn(),
};

vi.mock('$lib/utils/api', () => ({
  api: mockApi,
}));

// Mock sessionStorage
const mockSessionStorage = {
  setItem: vi.fn(),
  getItem: vi.fn(),
  removeItem: vi.fn(),
};

Object.defineProperty(window, 'sessionStorage', {
  value: mockSessionStorage,
});

// Mock localStorage
const mockLocalStorage = {
  setItem: vi.fn(),
  getItem: vi.fn(),
  removeItem: vi.fn(),
};

Object.defineProperty(window, 'localStorage', {
  value: mockLocalStorage,
});

// Mock window.location
const mockLocation = {
  href: '',
  origin: 'http://localhost:3000',
  pathname: '/',
  search: '',
};

Object.defineProperty(window, 'location', {
  value: mockLocation,
  writable: true,
});

describe('OAuth Integration Tests', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockLocation.href = '';
    mockLocation.search = '';
    mockLocation.pathname = '/';
    
    // Reset auth store
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
  });

  describe('OAuth Flow Initiation', () => {
    it('initiates Google OAuth flow from login page', async () => {
      const mockResponse = {
        success: true,
        data: {
          authorization_url: 'https://accounts.google.com/oauth/authorize?client_id=test&state=test-state',
          state: 'test-state-token',
        },
      };
      
      mockApi.post.mockResolvedValueOnce(mockResponse);
      
      render(Login);
      
      // Find and click Google login button
      const googleButton = screen.getByRole('button', { name: /continue with google/i });
      await fireEvent.click(googleButton);
      
      await waitFor(() => {
        expect(mockApi.post).toHaveBeenCalledWith('/auth/oauth/google/initiate');
        expect(mockSessionStorage.setItem).toHaveBeenCalledWith('oauth_state_google', 'test-state-token');
        expect(mockLocation.href).toBe('https://accounts.google.com/oauth/authorize?client_id=test&state=test-state');
      });
      
      // Check auth store state
      const authState = get(authStore);
      expect(authState.oauthFlow.provider).toBe('google');
      expect(authState.oauthFlow.isInProgress).toBe(true);
    });

    it('handles OAuth initiation errors', async () => {
      const mockError = new Error('OAuth provider not configured');
      mockApi.post.mockRejectedValueOnce(mockError);
      
      render(Login);
      
      const googleButton = screen.getByRole('button', { name: /continue with google/i });
      await fireEvent.click(googleButton);
      
      await waitFor(() => {
        expect(screen.getByText(/oauth provider not configured/i)).toBeInTheDocument();
      });
      
      // Check auth store state is reset
      const authState = get(authStore);
      expect(authState.oauthFlow.provider).toBe(null);
      expect(authState.oauthFlow.isInProgress).toBe(false);
    });
  });

  describe('OAuth Flow Completion', () => {
    it('completes OAuth flow successfully', async () => {
      const mockResponse = {
        success: true,
        data: {
          access_token: 'test-access-token',
          refresh_token: 'test-refresh-token',
          user: {
            id: '1',
            email: 'test@example.com',
            oauth_accounts: [
              {
                provider: 'google',
                provider_user_id: 'google123',
                email: 'test@gmail.com',
                display_name: 'Test User',
                linked_at: '2023-01-01T00:00:00Z',
              },
            ],
          },
        },
      };
      
      mockApi.post.mockResolvedValueOnce(mockResponse);
      mockApi.get.mockResolvedValueOnce({ success: true, data: mockResponse.data.user });
      mockSessionStorage.getItem.mockReturnValue('test-state');
      
      const result = await authActions.completeOAuthFlow('google', 'test-code', 'test-state');
      
      expect(result.success).toBe(true);
      expect(mockApi.post).toHaveBeenCalledWith('/auth/oauth/google/callback', {
        code: 'test-code',
        state: 'test-state',
        redirect_uri: 'http://localhost:3000/',
      });
      
      expect(mockLocalStorage.setItem).toHaveBeenCalledWith('auth_token', 'test-access-token');
      expect(mockLocalStorage.setItem).toHaveBeenCalledWith('refresh_token', 'test-refresh-token');
      expect(mockSessionStorage.removeItem).toHaveBeenCalledWith('oauth_state_google');
      
      // Check auth store state
      const authState = get(authStore);
      expect(authState.isAuthenticated).toBe(true);
      expect(authState.token).toBe('test-access-token');
      expect(authState.oauthFlow.provider).toBe(null);
      expect(authState.oauthFlow.isInProgress).toBe(false);
    });

    it('handles invalid state parameter (CSRF protection)', async () => {
      mockSessionStorage.getItem.mockReturnValue('valid-state');
      
      const result = await authActions.completeOAuthFlow('google', 'test-code', 'invalid-state');
      
      expect(result.success).toBe(false);
      expect(result.message).toContain('Invalid state parameter');
      expect(mockSessionStorage.removeItem).toHaveBeenCalledWith('oauth_state_google');
      
      // Check auth store state is reset
      const authState = get(authStore);
      expect(authState.oauthFlow.provider).toBe(null);
      expect(authState.oauthFlow.isInProgress).toBe(false);
    });

    it('handles OAuth callback API errors', async () => {
      const mockError = new Error('Invalid authorization code');
      mockApi.post.mockRejectedValueOnce(mockError);
      mockSessionStorage.getItem.mockReturnValue('test-state');
      
      const result = await authActions.completeOAuthFlow('google', 'test-code', 'test-state');
      
      expect(result.success).toBe(false);
      expect(result.message).toBe('Invalid authorization code');
      expect(mockSessionStorage.removeItem).toHaveBeenCalledWith('oauth_state_google');
    });
  });

  describe('Account Linking', () => {
    it('initiates account linking flow', async () => {
      const mockResponse = {
        success: true,
        data: {
          authorization_url: 'https://accounts.google.com/oauth/authorize?client_id=test&state=link-state',
          state: 'link-state-token',
        },
      };
      
      mockApi.post.mockResolvedValueOnce(mockResponse);
      
      const result = await authActions.linkOAuthAccount('google');
      
      expect(result.success).toBe(true);
      expect(result.authorization_url).toBe('https://accounts.google.com/oauth/authorize?client_id=test&state=link-state');
      expect(mockApi.post).toHaveBeenCalledWith('/auth/oauth/google/link');
      expect(mockSessionStorage.setItem).toHaveBeenCalledWith('oauth_link_state_google', 'link-state-token');
    });

    it('unlinks OAuth account successfully', async () => {
      const mockResponse = { success: true };
      mockApi.delete.mockResolvedValueOnce(mockResponse);
      mockApi.get.mockResolvedValueOnce({ 
        success: true, 
        data: { id: '1', email: 'test@example.com', oauth_accounts: [] } 
      });
      
      const result = await authActions.unlinkOAuthAccount('google');
      
      expect(result.success).toBe(true);
      expect(mockApi.delete).toHaveBeenCalledWith('/auth/oauth/google/unlink');
    });

    it('gets linked accounts', async () => {
      const mockAccounts = [
        {
          provider: 'google',
          provider_user_id: 'google123',
          email: 'test@gmail.com',
          display_name: 'Test User',
          linked_at: '2023-01-01T00:00:00Z',
        },
      ];
      
      mockApi.get.mockResolvedValueOnce({ success: true, data: mockAccounts });
      
      const result = await authActions.getLinkedAccounts();
      
      expect(result.success).toBe(true);
      expect(result.data).toEqual(mockAccounts);
      expect(mockApi.get).toHaveBeenCalledWith('/auth/oauth/accounts');
    });
  });

  describe('Error Handling', () => {
    it('handles network errors gracefully', async () => {
      const networkError = new Error('Network error');
      mockApi.post.mockRejectedValueOnce(networkError);
      
      const result = await authActions.initiateOAuthFlow('google');
      
      expect(result.success).toBe(false);
      expect(result.message).toBe('Network error');
      
      // Check auth store state is reset
      const authState = get(authStore);
      expect(authState.oauthFlow.provider).toBe(null);
      expect(authState.oauthFlow.isInProgress).toBe(false);
    });

    it('handles API failure responses', async () => {
      const mockResponse = {
        success: false,
        message: 'OAuth provider temporarily unavailable',
      };
      
      mockApi.post.mockResolvedValueOnce(mockResponse);
      
      const result = await authActions.initiateOAuthFlow('google');
      
      expect(result.success).toBe(false);
      expect(result.message).toBe('OAuth provider temporarily unavailable');
    });
  });

  describe('State Management', () => {
    it('updates auth store correctly during OAuth flow', async () => {
      // Initial state
      let authState = get(authStore);
      expect(authState.oauthFlow.isInProgress).toBe(false);
      
      // Mock successful initiation
      const mockResponse = {
        success: true,
        data: {
          authorization_url: 'https://accounts.google.com/oauth/authorize',
          state: 'test-state',
        },
      };
      mockApi.post.mockResolvedValueOnce(mockResponse);
      
      // Start OAuth flow
      await authActions.initiateOAuthFlow('google');
      
      // Check intermediate state
      authState = get(authStore);
      expect(authState.oauthFlow.provider).toBe('google');
      expect(authState.oauthFlow.state).toBe('test-state');
      expect(authState.oauthFlow.isInProgress).toBe(true);
    });

    it('cleans up state on OAuth completion', async () => {
      // Set up OAuth flow state
      authStore.update(state => ({
        ...state,
        oauthFlow: {
          provider: 'google',
          state: 'test-state',
          isInProgress: true,
        },
      }));
      
      // Mock successful completion
      const mockResponse = {
        success: true,
        data: {
          access_token: 'test-token',
          refresh_token: 'test-refresh',
        },
      };
      mockApi.post.mockResolvedValueOnce(mockResponse);
      mockApi.get.mockResolvedValueOnce({ success: true, data: { id: '1', email: 'test@example.com' } });
      mockSessionStorage.getItem.mockReturnValue('test-state');
      
      await authActions.completeOAuthFlow('google', 'test-code', 'test-state');
      
      // Check final state
      const authState = get(authStore);
      expect(authState.oauthFlow.provider).toBe(null);
      expect(authState.oauthFlow.state).toBe(null);
      expect(authState.oauthFlow.isInProgress).toBe(false);
      expect(authState.isAuthenticated).toBe(true);
    });
  });
});