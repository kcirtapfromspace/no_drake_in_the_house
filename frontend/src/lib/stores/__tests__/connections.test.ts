import { describe, it, expect, beforeEach, vi } from 'vitest';
import { get } from 'svelte/store';

const { mockApiClient } = vi.hoisted(() => ({
  mockApiClient: {
    getAuthToken: vi.fn(),
    handleAuthError: vi.fn(),
    clearAuthToken: vi.fn(),
    authenticatedRequest: vi.fn(),
  },
}));

const mockFetch = vi.fn();

vi.mock('../../utils/api-client', () => ({
  apiClient: mockApiClient,
}));

vi.mock('../../utils/musickit', () => ({
  configureMusicKit: vi.fn(),
  connectAppleMusic: vi.fn(),
  disconnectAppleMusic: vi.fn(),
  verifyAppleMusicConnection: vi.fn(),
  getAppleMusicStatus: vi.fn(),
}));

import {
  activeConnectionCount,
  connectionActions,
  connectionsStore,
  countActiveConnections,
  isConnectionActive,
  type ConnectionsState,
} from '../connections';

const existingState: ConnectionsState = {
  connections: [
    {
      id: 'apple-1',
      provider: 'apple_music',
      provider_user_id: 'apple-user',
      scopes: ['library-read'],
      status: 'active',
      health_status: 'active',
      created_at: '2026-03-26T00:00:00.000Z',
    },
  ],
  isLoading: false,
  error: null,
};

describe('connections store', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockApiClient.getAuthToken.mockReturnValue('test-token');
    mockApiClient.handleAuthError.mockResolvedValue(false);
    mockApiClient.authenticatedRequest.mockResolvedValue({ success: false });
    vi.stubGlobal('fetch', mockFetch);

    connectionsStore.set({
      connections: [],
      isLoading: false,
      error: null,
    });
  });

  it('counts only active connections', () => {
    expect(
      countActiveConnections([
        { ...existingState.connections[0], status: 'active' },
        { ...existingState.connections[0], id: 'spotify-1', provider: 'spotify', status: 'expired' },
        { ...existingState.connections[0], id: 'tidal-1', provider: 'tidal', status: 'error' },
      ])
    ).toBe(1);

    expect(isConnectionActive(existingState.connections[0])).toBe(true);
    expect(isConnectionActive({ ...existingState.connections[0], status: 'expired' })).toBe(false);
  });

  it('preserves previous connections on non-auth fetch failures', async () => {
    connectionsStore.set(existingState);
    mockFetch.mockResolvedValue(
      new Response(JSON.stringify({ message: 'backend unavailable' }), {
        status: 500,
        headers: { 'content-type': 'application/json' },
      })
    );

    await connectionActions.fetchConnections();

    expect(get(activeConnectionCount)).toBe(1);
    expect(get(connectionsStore).connections).toEqual(existingState.connections);
    expect(get(connectionsStore).error).toBe('backend unavailable');
  });

  it('clears connections on auth failures', async () => {
    connectionsStore.set(existingState);
    mockFetch.mockResolvedValue(
      new Response(JSON.stringify({ message: 'Unauthorized' }), {
        status: 401,
        headers: { 'content-type': 'application/json' },
      })
    );

    await connectionActions.fetchConnections();

    expect(get(activeConnectionCount)).toBe(0);
    expect(get(connectionsStore).connections).toEqual([]);
  });

  it('maps health_status payloads into active connections', async () => {
    mockFetch.mockResolvedValue(
      new Response(
        JSON.stringify({
          connections: [
            {
              id: 'apple-1',
              provider: 'apple_music',
              provider_user_id: 'apple-user',
              health_status: 'active',
              expires_at: '2026-09-15T19:29:25.415161Z',
              last_used_at: null,
              error_message: null,
              scopes: ['library-read'],
            },
            {
              id: 'spotify-1',
              provider: 'spotify',
              provider_user_id: 'spotify-user',
              health_status: 'needs_reauth',
              expires_at: '2026-03-26T21:42:16.487656Z',
              last_used_at: null,
              error_message: 'reauth required',
              scopes: ['user-library-read'],
            },
          ],
        }),
        {
          status: 200,
          headers: { 'content-type': 'application/json' },
        }
      )
    );

    await connectionActions.fetchConnections();

    expect(get(activeConnectionCount)).toBe(1);
    expect(get(connectionsStore).connections).toEqual([
      {
        id: 'apple-1',
        provider: 'apple_music',
        provider_user_id: 'apple-user',
        scopes: ['library-read'],
        status: 'active',
        health_status: 'active',
        expires_at: '2026-09-15T19:29:25.415161Z',
        last_health_check: null,
        created_at: '2026-09-15T19:29:25.415161Z',
        error_code: null,
      },
      {
        id: 'spotify-1',
        provider: 'spotify',
        provider_user_id: 'spotify-user',
        scopes: ['user-library-read'],
        status: 'expired',
        health_status: 'needs_reauth',
        expires_at: '2026-03-26T21:42:16.487656Z',
        last_health_check: null,
        created_at: '2026-03-26T21:42:16.487656Z',
        error_code: 'reauth required',
      },
    ]);
  });
});
