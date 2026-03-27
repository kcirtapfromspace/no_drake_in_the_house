import { writable } from 'svelte/store';
import { apiClient } from '../utils/api-client';

// Types

export interface AdminCatalog {
  total_artists: number;
  total_offenses: number;
  total_evidence: number;
  total_news_articles: number;
  total_classifications: number;
  pending_classifications: number;
}

export interface CategoryCoverage {
  category: string;
  offense_count: number;
  unique_artist_count: number;
  evidence_coverage_pct: number;
}

export interface PipelineRunSummary {
  total: number;
  success: number;
  failed: number;
  offenses_found: number;
}

export interface AdminPipeline {
  investigated: number;
  never_investigated: number;
  stale: number;
  recent_runs_24h: PipelineRunSummary;
  recent_runs_7d: PipelineRunSummary;
}

export interface AdminGrowth {
  artists_delta: number;
  offenses_delta: number;
  evidence_delta: number;
  snapshot_date: string;
}

export interface EvidenceDensity {
  avg_per_offense: number;
  zero_evidence_count: number;
  zero_evidence_pct: number;
}

export interface AdminMetrics {
  catalog: AdminCatalog;
  category_coverage: CategoryCoverage[];
  pipeline: AdminPipeline;
  growth: AdminGrowth | null;
  evidence_density: EvidenceDensity;
}

export interface CatalogSnapshot {
  date: string;
  total_artists: number;
  total_offenses: number;
  total_evidence: number;
  total_news_articles: number;
  total_classifications: number;
}

export interface AdminState {
  metrics: AdminMetrics | null;
  history: CatalogSnapshot[];
  isLoading: boolean;
  error: string | null;
}

const initialState: AdminState = {
  metrics: null,
  history: [],
  isLoading: false,
  error: null,
};

export const adminStore = writable<AdminState>(initialState);

export const adminActions = {
  fetchMetrics: async () => {
    adminStore.update(s => ({ ...s, isLoading: true, error: null }));

    try {
      const result = await apiClient.authenticatedRequest<AdminMetrics>(
        'GET',
        '/api/v1/analytics/admin-metrics',
      );

      if (result.success && result.data) {
        adminStore.update(s => ({
          ...s,
          metrics: result.data!,
          isLoading: false,
        }));
        return { success: true };
      } else {
        adminStore.update(s => ({
          ...s,
          isLoading: false,
          error: result.message || 'Failed to fetch admin metrics',
        }));
        return { success: false };
      }
    } catch (err: any) {
      adminStore.update(s => ({
        ...s,
        isLoading: false,
        error: err.message || 'Failed to fetch admin metrics',
      }));
      return { success: false };
    }
  },

  fetchHistory: async (limit = 30) => {
    try {
      const result = await apiClient.authenticatedRequest<CatalogSnapshot[]>(
        'GET',
        `/api/v1/analytics/catalog-metrics-history?limit=${limit}`,
      );

      if (result.success && result.data) {
        adminStore.update(s => ({
          ...s,
          history: result.data!,
        }));
        return { success: true };
      }
      return { success: false };
    } catch {
      return { success: false };
    }
  },
};
