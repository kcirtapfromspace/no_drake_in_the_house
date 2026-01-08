import { writable, derived } from 'svelte/store';
import { apiClient } from '../utils/api-client';

// Types
export interface PlatformStatus {
  platform: string;
  status: 'idle' | 'running' | 'error' | 'completed';
  last_sync?: string;
  artists_count?: number;
  error_message?: string;
}

export interface SyncRun {
  id: string;
  platform: string;
  sync_type: 'full' | 'incremental';
  status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';
  artists_processed: number;
  errors_count: number;
  started_at: string;
  completed_at?: string;
  duration_ms?: number;
}

export interface TriggerSyncRequest {
  platforms: string[];
  sync_type: 'full' | 'incremental';
  priority: 'low' | 'normal' | 'high' | 'critical';
}

export interface SyncHealthResponse {
  overall_status: 'healthy' | 'degraded' | 'unhealthy';
  platforms: {
    platform: string;
    is_healthy: boolean;
    last_check: string;
    error?: string;
  }[];
}

export interface CanonicalArtist {
  id: string;
  canonical_name: string;
  platform_ids: {
    platform: string;
    platform_id: string;
    confidence_score: number;
  }[];
  metadata?: {
    genres?: string[];
    image_url?: string;
    country?: string;
  };
}

// State
export interface SyncState {
  status: PlatformStatus[];
  runs: SyncRun[];
  currentRun: SyncRun | null;
  health: SyncHealthResponse | null;
  canonicalArtists: CanonicalArtist[];
  isLoading: boolean;
  isTriggering: boolean;
  error: string | null;
}

const initialState: SyncState = {
  status: [],
  runs: [],
  currentRun: null,
  health: null,
  canonicalArtists: [],
  isLoading: false,
  isTriggering: false,
  error: null,
};

// Store
export const syncStore = writable<SyncState>(initialState);

// Derived stores
export const isAnySyncRunning = derived(
  syncStore,
  ($sync) => $sync.status.some(s => s.status === 'running')
);

export const platformsStatus = derived(
  syncStore,
  ($sync) => $sync.status
);

export const recentRuns = derived(
  syncStore,
  ($sync) => $sync.runs.slice(0, 10)
);

// Actions
export const syncActions = {
  fetchStatus: async () => {
    syncStore.update(s => ({ ...s, isLoading: true, error: null }));

    try {
      const result = await apiClient.authenticatedRequest<{ platforms: PlatformStatus[] }>(
        'GET',
        '/api/v1/sync/status'
      );

      if (result.success && result.data) {
        syncStore.update(s => ({
          ...s,
          status: result.data!.platforms,
          isLoading: false,
        }));
        return { success: true };
      } else {
        syncStore.update(s => ({
          ...s,
          isLoading: false,
          error: result.message || 'Failed to fetch sync status',
        }));
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      syncStore.update(s => ({
        ...s,
        isLoading: false,
        error: error.message || 'Network error',
      }));
      return { success: false, message: error.message };
    }
  },

  triggerSync: async (request: TriggerSyncRequest) => {
    syncStore.update(s => ({ ...s, isTriggering: true, error: null }));

    try {
      const result = await apiClient.authenticatedRequest<{ run_id: string }>(
        'POST',
        '/api/v1/sync/trigger',
        request
      );

      if (result.success && result.data) {
        syncStore.update(s => ({ ...s, isTriggering: false }));
        // Refresh status after triggering
        await syncActions.fetchStatus();
        await syncActions.fetchRuns();
        return { success: true, runId: result.data.run_id };
      } else {
        syncStore.update(s => ({
          ...s,
          isTriggering: false,
          error: result.message || 'Failed to trigger sync',
        }));
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      syncStore.update(s => ({
        ...s,
        isTriggering: false,
        error: error.message || 'Network error',
      }));
      return { success: false, message: error.message };
    }
  },

  fetchRuns: async (limit: number = 20) => {
    syncStore.update(s => ({ ...s, isLoading: true, error: null }));

    try {
      const result = await apiClient.authenticatedRequest<{ runs: SyncRun[] }>(
        'GET',
        `/api/v1/sync/runs?limit=${limit}`
      );

      if (result.success && result.data) {
        syncStore.update(s => ({
          ...s,
          runs: result.data!.runs,
          isLoading: false,
        }));
        return { success: true };
      } else {
        syncStore.update(s => ({
          ...s,
          isLoading: false,
          error: result.message || 'Failed to fetch sync runs',
        }));
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      syncStore.update(s => ({
        ...s,
        isLoading: false,
        error: error.message || 'Network error',
      }));
      return { success: false, message: error.message };
    }
  },

  fetchRun: async (runId: string) => {
    try {
      const result = await apiClient.authenticatedRequest<SyncRun>(
        'GET',
        `/api/v1/sync/runs/${runId}`
      );

      if (result.success && result.data) {
        syncStore.update(s => ({
          ...s,
          currentRun: result.data!,
        }));
        return { success: true, run: result.data };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      return { success: false, message: error.message };
    }
  },

  cancelRun: async (runId: string) => {
    try {
      const result = await apiClient.authenticatedRequest(
        'POST',
        `/api/v1/sync/runs/${runId}/cancel`
      );

      if (result.success) {
        await syncActions.fetchRuns();
        return { success: true };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      return { success: false, message: error.message };
    }
  },

  fetchHealth: async () => {
    try {
      const result = await apiClient.authenticatedRequest<SyncHealthResponse>(
        'GET',
        '/api/v1/sync/health'
      );

      if (result.success && result.data) {
        syncStore.update(s => ({
          ...s,
          health: result.data!,
        }));
        return { success: true, health: result.data };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      return { success: false, message: error.message };
    }
  },

  searchArtists: async (query: string) => {
    try {
      const result = await apiClient.authenticatedRequest<{ artists: CanonicalArtist[] }>(
        'GET',
        `/api/v1/sync/search?q=${encodeURIComponent(query)}`
      );

      if (result.success && result.data) {
        return { success: true, artists: result.data.artists };
      } else {
        return { success: false, message: result.message, artists: [] };
      }
    } catch (error: any) {
      return { success: false, message: error.message, artists: [] };
    }
  },

  clearError: () => {
    syncStore.update(s => ({ ...s, error: null }));
  },
};
