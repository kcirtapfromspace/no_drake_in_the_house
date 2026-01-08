import { writable, derived } from 'svelte/store';
import { apiClient } from '../utils/api-client';

// Types
export interface DashboardMetrics {
  total_users: number;
  active_users_today: number;
  total_blocked_artists: number;
  total_subscriptions: number;
  offense_detections_today: number;
  sync_runs_today: number;
}

export interface UserQuickStats {
  blocked_artists: number;
  subscriptions: number;
  manual_blocks: number;
  last_sync?: string;
}

export interface SystemHealth {
  overall: 'healthy' | 'degraded' | 'unhealthy';
  databases: {
    postgres: boolean;
    redis: boolean;
    duckdb: boolean;
    kuzu: boolean;
    lancedb: boolean;
  };
  latencies_ms?: {
    postgres?: number;
    redis?: number;
    duckdb?: number;
    kuzu?: number;
    lancedb?: number;
  };
}

export interface TrendData {
  period: string;
  data_points: {
    date: string;
    value: number;
  }[];
  change_percent: number;
  trend: 'up' | 'down' | 'stable';
}

export interface ArtistTrend {
  artist_id: string;
  artist_name: string;
  mentions: number;
  sentiment: number;
  offense_count: number;
  trend: 'rising' | 'falling' | 'stable';
}

export interface ReportType {
  id: string;
  name: string;
  description: string;
  formats: string[];
}

export interface ReportRequest {
  report_type: string;
  format: 'json' | 'csv' | 'parquet' | 'html';
  time_range: string;
  include_details: boolean;
}

export interface ReportStatus {
  id: string;
  status: 'pending' | 'processing' | 'completed' | 'failed';
  progress_percent?: number;
  download_url?: string;
  error?: string;
  created_at: string;
  completed_at?: string;
}

// State
export interface AnalyticsState {
  dashboard: DashboardMetrics | null;
  userStats: UserQuickStats | null;
  systemHealth: SystemHealth | null;
  trends: {
    summary: TrendData | null;
    artists: ArtistTrend[];
    platforms: TrendData[];
  };
  reportTypes: ReportType[];
  reports: ReportStatus[];
  currentReport: ReportStatus | null;
  isLoading: boolean;
  error: string | null;
}

const initialState: AnalyticsState = {
  dashboard: null,
  userStats: null,
  systemHealth: null,
  trends: {
    summary: null,
    artists: [],
    platforms: [],
  },
  reportTypes: [],
  reports: [],
  currentReport: null,
  isLoading: false,
  error: null,
};

// Store
export const analyticsStore = writable<AnalyticsState>(initialState);

// Derived stores
export const isSystemHealthy = derived(
  analyticsStore,
  ($analytics) => $analytics.systemHealth?.overall === 'healthy'
);

export const risingArtists = derived(
  analyticsStore,
  ($analytics) => $analytics.trends.artists.filter(a => a.trend === 'rising')
);

export const fallingArtists = derived(
  analyticsStore,
  ($analytics) => $analytics.trends.artists.filter(a => a.trend === 'falling')
);

// Actions
export const analyticsActions = {
  fetchDashboard: async (timeRange: string = 'last7days') => {
    analyticsStore.update(s => ({ ...s, isLoading: true, error: null }));

    try {
      const result = await apiClient.authenticatedRequest<DashboardMetrics>(
        'GET',
        `/api/v1/analytics/dashboard?time_range=${timeRange}`
      );

      if (result.success && result.data) {
        analyticsStore.update(s => ({
          ...s,
          dashboard: result.data!,
          isLoading: false,
        }));
        return { success: true };
      } else {
        analyticsStore.update(s => ({
          ...s,
          isLoading: false,
          error: result.message || 'Failed to fetch dashboard',
        }));
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      analyticsStore.update(s => ({
        ...s,
        isLoading: false,
        error: error.message || 'Network error',
      }));
      return { success: false, message: error.message };
    }
  },

  fetchUserStats: async () => {
    try {
      const result = await apiClient.authenticatedRequest<UserQuickStats>(
        'GET',
        '/api/v1/analytics/dashboard/user-stats'
      );

      if (result.success && result.data) {
        analyticsStore.update(s => ({
          ...s,
          userStats: result.data!,
        }));
        return { success: true };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      return { success: false, message: error.message };
    }
  },

  fetchSystemHealth: async () => {
    try {
      const result = await apiClient.authenticatedRequest<SystemHealth>(
        'GET',
        '/api/v1/analytics/health'
      );

      if (result.success && result.data) {
        analyticsStore.update(s => ({
          ...s,
          systemHealth: result.data!,
        }));
        return { success: true, health: result.data };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      return { success: false, message: error.message };
    }
  },

  fetchTrendSummary: async (periodDays: number = 7) => {
    try {
      const result = await apiClient.authenticatedRequest<TrendData>(
        'GET',
        `/api/v1/analytics/trends?period_days=${periodDays}`
      );

      if (result.success && result.data) {
        analyticsStore.update(s => ({
          ...s,
          trends: {
            ...s.trends,
            summary: result.data!,
          },
        }));
        return { success: true };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      return { success: false, message: error.message };
    }
  },

  fetchRisingArtists: async (limit: number = 10) => {
    try {
      const result = await apiClient.authenticatedRequest<{ artists: ArtistTrend[] }>(
        'GET',
        `/api/v1/analytics/trends/rising?limit=${limit}`
      );

      if (result.success && result.data) {
        analyticsStore.update(s => ({
          ...s,
          trends: {
            ...s.trends,
            artists: [
              ...s.trends.artists.filter(a => a.trend !== 'rising'),
              ...result.data!.artists.map(a => ({ ...a, trend: 'rising' as const })),
            ],
          },
        }));
        return { success: true };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      return { success: false, message: error.message };
    }
  },

  fetchFallingArtists: async (limit: number = 10) => {
    try {
      const result = await apiClient.authenticatedRequest<{ artists: ArtistTrend[] }>(
        'GET',
        `/api/v1/analytics/trends/falling?limit=${limit}`
      );

      if (result.success && result.data) {
        analyticsStore.update(s => ({
          ...s,
          trends: {
            ...s.trends,
            artists: [
              ...s.trends.artists.filter(a => a.trend !== 'falling'),
              ...result.data!.artists.map(a => ({ ...a, trend: 'falling' as const })),
            ],
          },
        }));
        return { success: true };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      return { success: false, message: error.message };
    }
  },

  fetchReportTypes: async () => {
    try {
      const result = await apiClient.authenticatedRequest<{ report_types: ReportType[] }>(
        'GET',
        '/api/v1/analytics/reports/types'
      );

      if (result.success && result.data) {
        analyticsStore.update(s => ({
          ...s,
          reportTypes: result.data!.report_types,
        }));
        return { success: true };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      return { success: false, message: error.message };
    }
  },

  generateReport: async (request: ReportRequest) => {
    try {
      const result = await apiClient.authenticatedRequest<{ report_id: string }>(
        'POST',
        '/api/v1/analytics/reports',
        request
      );

      if (result.success && result.data) {
        return { success: true, reportId: result.data.report_id };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      return { success: false, message: error.message };
    }
  },

  fetchReportStatus: async (reportId: string) => {
    try {
      const result = await apiClient.authenticatedRequest<ReportStatus>(
        'GET',
        `/api/v1/analytics/reports/${reportId}`
      );

      if (result.success && result.data) {
        analyticsStore.update(s => ({
          ...s,
          currentReport: result.data!,
        }));
        return { success: true, report: result.data };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      return { success: false, message: error.message };
    }
  },

  clearError: () => {
    analyticsStore.update(s => ({ ...s, error: null }));
  },
};
