import { describe, it, expect, vi } from 'vitest';
import {
  getProviderFromPath,
  getProviderName,
  resolveOAuthCallback,
} from '../../utils/oauth-callback';

describe('oauth callback helpers', () => {
  it('extracts the provider from the callback route', () => {
    expect(getProviderFromPath('/auth/callback/google')).toBe('google');
    expect(getProviderFromPath('/auth/callback/apple')).toBe('apple');
  });

  it('maps provider names for the callback UI', () => {
    expect(getProviderName('google')).toBe('Google');
    expect(getProviderName('apple')).toBe('Apple Music');
    expect(getProviderName('github')).toBe('GitHub');
    expect(getProviderName('youtube')).toBe('YouTube Music');
    expect(getProviderName('tidal')).toBe('Tidal');
  });

  it('completes a successful callback with the expected request payload', async () => {
    const post = vi.fn().mockResolvedValue({ success: true });
    const location = {
      origin: 'http://localhost:3000',
      pathname: '/auth/callback/google',
      search: '?code=test-code&state=test-state',
    };

    const result = await resolveOAuthCallback(location, post);

    expect(post).toHaveBeenCalledWith('/api/v1/auth/oauth/google/link-callback', {
      code: 'test-code',
      state: 'test-state',
      redirect_uri: 'http://localhost:3000/auth/callback/google',
    });
    expect(result).toEqual({
      status: 'success',
      provider: 'google',
      errorMessage: '',
      request: {
        code: 'test-code',
        state: 'test-state',
        redirect_uri: 'http://localhost:3000/auth/callback/google',
      },
    });
  });

  it('surfaces provider errors returned in the callback URL', async () => {
    const result = await resolveOAuthCallback(
      {
        origin: 'http://localhost:3000',
        pathname: '/auth/callback/google',
        search: '?error=access_denied&error_description=User%20denied%20access',
      },
      vi.fn()
    );

    expect(result).toEqual({
      status: 'error',
      provider: 'google',
      errorMessage: 'User denied access',
    });
  });

  it('returns an error when required auth parameters are missing', async () => {
    const result = await resolveOAuthCallback(
      {
        origin: 'http://localhost:3000',
        pathname: '/auth/callback/google',
        search: '?state=test-state',
      },
      vi.fn()
    );

    expect(result).toEqual({
      status: 'error',
      provider: 'google',
      errorMessage: 'Missing authentication parameters',
    });
  });

  it('surfaces backend failure responses', async () => {
    const post = vi.fn().mockResolvedValue({
      success: false,
      message: 'Invalid authorization code',
    });

    const result = await resolveOAuthCallback(
      {
        origin: 'http://localhost:3000',
        pathname: '/auth/callback/google',
        search: '?code=test-code&state=test-state',
      },
      post
    );

    expect(result.status).toBe('error');
    expect(result.provider).toBe('google');
    expect(result.errorMessage).toBe('Invalid authorization code');
  });

  it('routes streaming providers to the connection callback endpoints', async () => {
    const post = vi.fn().mockResolvedValue({ success: true });

    const result = await resolveOAuthCallback(
      {
        origin: 'https://nodrakeinthe.house',
        pathname: '/auth/callback/youtube',
        search: '?code=test-code&state=test-state',
      },
      post
    );

    expect(post).toHaveBeenCalledWith('/api/v1/connections/youtube/callback', {
      code: 'test-code',
      state: 'test-state',
    });
    expect(result.status).toBe('success');
    expect(result.provider).toBe('youtube');
  });

  it('surfaces thrown request errors', async () => {
    const post = vi.fn().mockRejectedValue(new Error('Network error'));

    const result = await resolveOAuthCallback(
      {
        origin: 'http://localhost:3000',
        pathname: '/auth/callback/google',
        search: '?code=test-code&state=test-state',
      },
      post
    );

    expect(result.status).toBe('error');
    expect(result.provider).toBe('google');
    expect(result.errorMessage).toBe('Network error');
  });
});
