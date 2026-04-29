import { describe, expect, it } from 'vitest';
import type { ServiceConnection } from '../../stores/connections';
import {
  classifyOAuthFailure,
  deriveCanonicalOAuthState,
  getOAuthStateCopy,
  isAlreadyConnectedMessage,
  mapOAuthActionError,
} from '../oauth-state';

function connection(overrides: Partial<ServiceConnection>): ServiceConnection {
  return {
    id: 'conn-1',
    provider: 'spotify',
    scopes: [],
    status: 'active',
    created_at: '2026-01-01T00:00:00.000Z',
    ...overrides,
  };
}

describe('oauth-state helpers', () => {
  it('classifies auth and rate-limit errors', () => {
    expect(classifyOAuthFailure('invalid_grant from provider')).toBe('failed_auth');
    expect(classifyOAuthFailure('HTTP_429 too many requests')).toBe('rate_limited');
  });

  it('derives connected and disconnected states', () => {
    expect(deriveCanonicalOAuthState(null)).toBe('disconnected');
    expect(deriveCanonicalOAuthState(connection({ status: 'active' }))).toBe('connected');
  });

  it('prefers explicit canonical state from backend payloads', () => {
    expect(
      deriveCanonicalOAuthState(connection({ status: 'active', oauth_state: 'failed_auth' }))
    ).toBe('failed_auth');
  });

  it('maps non-active connection statuses to canonical failures', () => {
    expect(deriveCanonicalOAuthState(connection({ status: 'expired' }))).toBe('failed_auth');
    expect(
      deriveCanonicalOAuthState(
        connection({ status: 'error', error_code: 'HTTP_429 from provider sync' })
      )
    ).toBe('rate_limited');
  });

  it('returns provider-agnostic copy and reconnect affordance', () => {
    const copy = getOAuthStateCopy('failed_auth', 'Spotify');
    expect(copy.label).toBe('Reconnect Required');
    expect(copy.reconnectCta).toBe(true);
    expect(copy.message).toContain('Reconnect');
  });

  it('detects already-connected messages and normalizes action errors', () => {
    expect(isAlreadyConnectedMessage('You already have an active spotify connection')).toBe(true);
    expect(
      mapOAuthActionError('Spotify', 'connect', 'already connected')
    ).toContain('already connected');
  });
});
