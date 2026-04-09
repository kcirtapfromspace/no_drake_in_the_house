import { describe, expect, it, vi } from 'vitest';

vi.mock('../../utils/api-client', () => ({
  apiClient: {
    post: vi.fn(),
    authenticatedRequest: vi.fn(),
    setAuthToken: vi.fn(),
    getAuthToken: vi.fn(),
    clearAuthToken: vi.fn(),
  },
}));

vi.mock('../../convex/client', () => ({
  setConvexAuthToken: vi.fn(),
}));

import { hasValidIssuer } from '../auth';

function base64UrlEncode(value: unknown): string {
  return btoa(JSON.stringify(value))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=+$/g, '');
}

function createToken(payload: Record<string, unknown>): string {
  const header = base64UrlEncode({ alg: 'none', typ: 'JWT' });
  const body = base64UrlEncode(payload);
  return `${header}.${body}.`;
}

describe('hasValidIssuer', () => {
  it('returns true when token has a non-empty issuer claim', () => {
    const token = createToken({ iss: 'https://api.nodrakeinthe.house', aud: 'convex' });
    expect(hasValidIssuer(token)).toBe(true);
  });

  it('returns false when issuer claim is missing', () => {
    const token = createToken({ aud: 'convex' });
    expect(hasValidIssuer(token)).toBe(false);
  });

  it('returns false when issuer claim is blank', () => {
    const token = createToken({ iss: '   ', aud: 'convex' });
    expect(hasValidIssuer(token)).toBe(false);
  });

  it('returns false for malformed tokens', () => {
    expect(hasValidIssuer('not-a-jwt')).toBe(false);
  });

  it('returns false when issuer is a non-string value', () => {
    const numericIssuer = createToken({ iss: 12345, aud: 'convex' });
    const nullIssuer = createToken({ iss: null, aud: 'convex' });

    expect(hasValidIssuer(numericIssuer)).toBe(false);
    expect(hasValidIssuer(nullIssuer)).toBe(false);
  });
});
