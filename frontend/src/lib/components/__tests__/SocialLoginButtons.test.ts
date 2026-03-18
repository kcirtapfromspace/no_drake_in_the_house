import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import SocialLoginButtons from '../SocialLoginButtons.svelte';

const { mockApiPost } = vi.hoisted(() => ({
  mockApiPost: vi.fn(),
}));

vi.mock('../../utils/api', () => ({
  api: {
    post: mockApiPost,
  },
}));

const mockSessionStorage = {
  setItem: vi.fn(),
  getItem: vi.fn(),
  removeItem: vi.fn(),
};

const mockLocation = {
  href: '',
};

Object.defineProperty(window, 'sessionStorage', {
  value: mockSessionStorage,
});

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

  it('initiates the google oauth flow and emits loading lifecycle events', async () => {
    mockApiPost.mockResolvedValueOnce({
      success: true,
      data: {
        authorization_url: 'https://accounts.google.com/oauth/authorize?client_id=test',
        state: 'test-state-token',
      },
    });

    const { component } = render(SocialLoginButtons);
    const loadingEvents: Array<{ provider: string; loading: boolean }> = [];

    component.$on('loading', (event) => {
      loadingEvents.push(event.detail);
    });

    await fireEvent.click(screen.getByRole('button', { name: /continue with google/i }));

    await waitFor(() => {
      expect(mockApiPost).toHaveBeenCalledWith('/auth/oauth/google/initiate');
      expect(mockSessionStorage.setItem).toHaveBeenCalledWith('oauth_state_google', 'test-state-token');
      expect(mockLocation.href).toBe('https://accounts.google.com/oauth/authorize?client_id=test');
    });

    expect(loadingEvents).toEqual([
      { provider: 'google', loading: true },
      { provider: 'google', loading: false },
    ]);
  });

  it('emits an error event when the oauth API returns a failure response', async () => {
    mockApiPost.mockResolvedValueOnce({
      success: false,
      message: 'OAuth provider not configured',
    });

    const { component } = render(SocialLoginButtons);
    let errorEvent: { provider: string; message: string } | null = null;

    component.$on('error', (event) => {
      errorEvent = event.detail;
    });

    await fireEvent.click(screen.getByRole('button', { name: /continue with google/i }));

    await waitFor(() => {
      expect(errorEvent).toEqual({
        provider: 'google',
        message: 'OAuth provider not configured',
      });
    });
  });

  it('emits an error event when the oauth request throws', async () => {
    mockApiPost.mockRejectedValueOnce(new Error('Network error'));

    const { component } = render(SocialLoginButtons);
    let errorEvent: { provider: string; message: string } | null = null;

    component.$on('error', (event) => {
      errorEvent = event.detail;
    });

    await fireEvent.click(screen.getByRole('button', { name: /continue with google/i }));

    await waitFor(() => {
      expect(errorEvent).toEqual({
        provider: 'google',
        message: 'Network error',
      });
    });
  });

  it('disables all buttons when loading is controlled externally', () => {
    render(SocialLoginButtons, { isLoading: true });

    expect(screen.getByRole('button', { name: /continue with google/i })).toBeDisabled();
    expect(screen.getByRole('button', { name: /continue with apple/i })).toBeDisabled();
    expect(screen.getByRole('button', { name: /continue with github/i })).toBeDisabled();
  });

  it('renders an inline error message from props', () => {
    render(SocialLoginButtons, { error: 'OAuth authentication failed' });

    expect(screen.getByText(/oauth authentication failed/i)).toBeInTheDocument();
  });
});
