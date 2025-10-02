import { render, screen, waitFor } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import OAuthCallback from '../OAuthCallback.svelte';

// Mock the API module
const mockApi = {
  post: vi.fn(),
};

vi.mock('$lib/utils/api', () => ({
  api: mockApi,
}));

// Mock the auth store
const mockAuthActions = {
  fetchProfile: vi.fn(),
};

vi.mock('$lib/stores/auth', () => ({
  authActions: mockAuthActions,
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

// Mock window.location
const mockLocation = {
  search: '',
  pathname: '/auth/callback/google',
  origin: 'http://localhost:3000',
  href: '',
};

Object.defineProperty(window, 'location', {
  value: mockLocation,
  writable: true,
});

// Mock setTimeout
vi.stubGlobal('setTimeout', (fn: Function, delay: number) => {
  fn();
  return 1;
});

describe('OAuthCallback', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockLocation.search = '';
    mockLocation.href = '';
  });

  it('displays processing state initially', () => {
    mockLocation.search = '?code=test-code&state=test-state';
    mockSessionStorage.getItem.mockReturnValue('test-state');
    
    render(OAuthCallback, { provider: 'google' });
    
    expect(screen.getByText(/completing google sign in/i)).toBeInTheDocument();
    expect(screen.getByText(/please wait while we complete/i)).toBeInTheDocument();
  });

  it('successfully processes OAuth callback', async () => {
    mockLocation.search = '?code=test-code&state=test-state';
    mockSessionStorage.getItem.mockReturnValue('test-state');
    
    const mockResponse = {
      success: true,
      data: {
        access_token: 'test-access-token',
        refresh_token: 'test-refresh-token',
        user: { id: '1', email: 'test@example.com' },
      },
    };
    
    mockApi.post.mockResolvedValueOnce(mockResponse);
    
    render(OAuthCallback, { provider: 'google' });
    
    await waitFor(() => {
      expect(mockApi.post).toHaveBeenCalledWith('/auth/oauth/google/callback', {
        code: 'test-code',
        state: 'test-state',
        redirect_uri: 'http://localhost:3000/auth/callback/google',
      });
    });
    
    await waitFor(() => {
      expect(screen.getByText(/welcome!/i)).toBeInTheDocument();
      expect(screen.getByText(/successfully signed in with google/i)).toBeInTheDocument();
    });
    
    expect(mockAuthActions.fetchProfile).toHaveBeenCalled();
    expect(mockSessionStorage.removeItem).toHaveBeenCalledWith('oauth_state_google');
    expect(mockLocation.href).toBe('/');
  });

  it('handles OAuth provider errors', async () => {
    mockLocation.search = '?error=access_denied&error_description=User%20denied%20access';
    
    render(OAuthCallback, { provider: 'google' });
    
    await waitFor(() => {
      expect(screen.getByText(/authentication failed/i)).toBeInTheDocument();
      expect(screen.getByText(/user denied access/i)).toBeInTheDocument();
    });
    
    expect(mockSessionStorage.removeItem).toHaveBeenCalledWith('oauth_state_google');
  });

  it('handles missing authorization code', async () => {
    mockLocation.search = '?state=test-state';
    
    render(OAuthCallback, { provider: 'google' });
    
    await waitFor(() => {
      expect(screen.getByText(/authentication failed/i)).toBeInTheDocument();
      expect(screen.getByText(/missing authorization code or state parameter/i)).toBeInTheDocument();
    });
  });

  it('handles invalid state parameter (CSRF protection)', async () => {
    mockLocation.search = '?code=test-code&state=invalid-state';
    mockSessionStorage.getItem.mockReturnValue('valid-state');
    
    render(OAuthCallback, { provider: 'google' });
    
    await waitFor(() => {
      expect(screen.getByText(/authentication failed/i)).toBeInTheDocument();
      expect(screen.getByText(/invalid state parameter - possible csrf attack/i)).toBeInTheDocument();
    });
    
    expect(mockSessionStorage.removeItem).toHaveBeenCalledWith('oauth_state_google');
  });

  it('handles API errors during token exchange', async () => {
    mockLocation.search = '?code=test-code&state=test-state';
    mockSessionStorage.getItem.mockReturnValue('test-state');
    
    const mockError = new Error('Network error');
    mockApi.post.mockRejectedValueOnce(mockError);
    
    render(OAuthCallback, { provider: 'google' });
    
    await waitFor(() => {
      expect(screen.getByText(/authentication failed/i)).toBeInTheDocument();
      expect(screen.getByText(/network error/i)).toBeInTheDocument();
    });
    
    expect(mockSessionStorage.removeItem).toHaveBeenCalledWith('oauth_state_google');
  });

  it('handles API failure responses', async () => {
    mockLocation.search = '?code=test-code&state=test-state';
    mockSessionStorage.getItem.mockReturnValue('test-state');
    
    const mockResponse = {
      success: false,
      message: 'Invalid authorization code',
    };
    
    mockApi.post.mockResolvedValueOnce(mockResponse);
    
    render(OAuthCallback, { provider: 'google' });
    
    await waitFor(() => {
      expect(screen.getByText(/authentication failed/i)).toBeInTheDocument();
      expect(screen.getByText(/invalid authorization code/i)).toBeInTheDocument();
    });
  });

  it('displays correct provider name for different providers', () => {
    mockLocation.search = '?code=test-code&state=test-state';
    mockSessionStorage.getItem.mockReturnValue('test-state');
    
    const { rerender } = render(OAuthCallback, { provider: 'apple' });
    expect(screen.getByText(/completing apple sign in/i)).toBeInTheDocument();
    
    rerender({ provider: 'github' });
    expect(screen.getByText(/completing github sign in/i)).toBeInTheDocument();
  });

  it('provides retry functionality on error', async () => {
    mockLocation.search = '?error=server_error';
    
    render(OAuthCallback, { provider: 'google' });
    
    await waitFor(() => {
      expect(screen.getByText(/authentication failed/i)).toBeInTheDocument();
    });
    
    const retryButton = screen.getByRole('button', { name: /try again/i });
    expect(retryButton).toBeInTheDocument();
    
    // Clicking retry should redirect to home
    await retryButton.click();
    expect(mockLocation.href).toBe('/');
  });

  it('stores tokens in localStorage on success', async () => {
    mockLocation.search = '?code=test-code&state=test-state';
    mockSessionStorage.getItem.mockReturnValue('test-state');
    
    const mockResponse = {
      success: true,
      data: {
        access_token: 'test-access-token',
        refresh_token: 'test-refresh-token',
        user: { id: '1', email: 'test@example.com' },
      },
    };
    
    mockApi.post.mockResolvedValueOnce(mockResponse);
    
    // Mock localStorage
    const mockLocalStorage = {
      setItem: vi.fn(),
    };
    Object.defineProperty(window, 'localStorage', {
      value: mockLocalStorage,
    });
    
    render(OAuthCallback, { provider: 'google' });
    
    await waitFor(() => {
      expect(mockLocalStorage.setItem).toHaveBeenCalledWith('auth_token', 'test-access-token');
      expect(mockLocalStorage.setItem).toHaveBeenCalledWith('refresh_token', 'test-refresh-token');
    });
  });

  it('handles missing stored state', async () => {
    mockLocation.search = '?code=test-code&state=test-state';
    mockSessionStorage.getItem.mockReturnValue(null);
    
    render(OAuthCallback, { provider: 'google' });
    
    await waitFor(() => {
      expect(screen.getByText(/authentication failed/i)).toBeInTheDocument();
      expect(screen.getByText(/invalid state parameter - possible csrf attack/i)).toBeInTheDocument();
    });
  });
});