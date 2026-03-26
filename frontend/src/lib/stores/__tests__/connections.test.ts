import { describe, it, expect, beforeEach, vi } from 'vitest';
import { get } from 'svelte/store';

const { mockApiClient } = vi.hoisted(() => ({
  mockApiClient: {
    authenticatedRequest: vi.fn(),
  },
}));

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
    mockApiClient.authenticatedRequest.mockResolvedValue({
      success: false,
      error_code: 'HTTP_500',
      message: 'backend unavailable',
    });

    await connectionActions.fetchConnections();

    expect(get(activeConnectionCount)).toBe(1);
    expect(get(connectionsStore).connections).toEqual(existingState.connections);
    expect(get(connectionsStore).error).toBe('backend unavailable');
  });

  it('clears connections on auth failures', async () => {
    connectionsStore.set(existingState);
    mockApiClient.authenticatedRequest.mockResolvedValue({
      success: false,
      error_code: 'HTTP_401',
      message: 'Unauthorized',
    });

    await connectionActions.fetchConnections();

    expect(get(activeConnectionCount)).toBe(0);
    expect(get(connectionsStore).connections).toEqual([]);
  });
});
