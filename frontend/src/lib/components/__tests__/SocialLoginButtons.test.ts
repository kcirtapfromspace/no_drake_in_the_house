import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import SocialLoginButtons from '../SocialLoginButtons.svelte';

// Mock the API module
const mockApiPost = vi.fn();
vi.mock('$lib/utils/api', () => ({
  api: {
    post: mockApiPost,
  },
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
  href: '',
};

Object.defineProperty(window, 'location', {
  value: mockLocation,
  writable: true,
});

describe('SocialLoginButtons', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockLocation.href = '';
  });

  it('renders all social login buttons', () => {
    render(SocialLoginButtons);
    
    expect(screen.getByRole('button', { name: /continue with google/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /continue with apple/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /continue with github/i })).toBeInTheDocument();
  });

  it('initiates OAuth flow when Google button is clicked', async () => {
    const mockResponse = {
      success: true,
      data: {
        authorization_url: 'https://accounts.google.com/oauth/authorize?client_id=test',
        state: 'test-state-token',
      },
    };
    
    mockApiPost.mockResolvedValueOnce(mockResponse);
    
    const component = render(SocialLoginButtons);
    let loadingEvent: any = null;
    
    component.component.$on('loading', (event) => {
      loadingEvent = event.detail;
    });
    
    const googleButton = screen.getByRole('button', { name: /continue with google/i });
    await fireEvent.click(googleButton);
    
    await waitFor(() => {
      expect(mockApiPost).toHaveBeenCalledWith('/auth/oauth/google/initiate');
      expect(mockSessionStorage.setItem).toHaveBeenCalledWith('oauth_state_google', 'test-state-token');
      expect(mockLocation.href).toBe('https://accounts.google.com/oauth/authorize?client_id=test');
    });
    
    expect(loadingEvent).toEqual({
      provider: 'google',
      loading: true,
    });
  });

  it('handles API errors gracefully', async () => {
    const mockError = new Error('Network error');
    mockApiPost.mockRejectedValueOnce(mockError);
    
    const component = render(SocialLoginButtons);
    let errorEvent: any = null;
    
    component.component.$on('error', (event) => {
      errorEvent = event.detail;
    });
    
    const googleButton = screen.getByRole('button', { name: /continue with google/i });
    await fireEvent.click(googleButton);
    
    await waitFor(() => {
      expect(errorEvent).toEqual({
        provider: 'google',
        message: 'Network error',
      });
    });
  });

  it('disables buttons when loading', () => {
    render(SocialLoginButtons, { isLoading: true });
    
    const googleButton = screen.getByRole('button', { name: /continue with google/i });
    const appleButton = screen.getByRole('button', { name: /continue with apple/i });
    const githubButton = screen.getByRole('button', { name: /continue with github/i });
    
    expect(googleButton).toBeDisabled();
    expect(appleButton).toBeDisabled();
    expect(githubButton).toBeDisabled();
  });

  it('displays error message when error prop is set', () => {
    render(SocialLoginButtons, { error: 'OAuth authentication failed' });
    
    expect(screen.getByText(/oauth authentication failed/i)).toBeInTheDocument();
  });
});