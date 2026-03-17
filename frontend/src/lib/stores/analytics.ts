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

// Trouble Score Types
export type TroubleTier = 'low' | 'moderate' | 'high' | 'critical';

export interface TroubleScoreComponents {
  severity_score: number;
  evidence_score: number;
  recency_score: number;
  community_score: number;
  revenue_score: number;
}

export interface ArtistTroubleScore {
  id: string;
  artist_id: string;
  artist_name: string;
  components: TroubleScoreComponents;
  total_score: number;
  trouble_tier: TroubleTier;
  offense_count: number;
  verified_offense_count: number;
  block_count: number;
  egregious_count: number;
  severe_count: number;
  moderate_count: number;
  minor_count: number;
  first_offense_date?: string;
  last_offense_date?: string;
  last_calculated_at: string;
}

export interface TroubleLeaderboardEntry {
  rank: number;
  artist_id: string;
  artist_name: string;
  total_score: number;
  trouble_tier: TroubleTier;
  offense_count: number;
  block_count: number;
  most_severe_category?: string;
}

export interface TierDistribution {
  low: number;
  moderate: number;
  high: number;
  critical: number;
  total: number;
}

// Category Revenue Types (Simulated by Offense Category)
export interface CategoryArtistRevenue {
  artist_id: string;
  artist_name: string;
  offense_count: number;
  severity: string;
  simulated_streams: number;
  simulated_revenue: string;
}

export interface CategoryRevenue {
  category: string;
  display_name: string;
  artist_count: number;
  offense_count: number;
  simulated_streams: number;
  simulated_revenue: string;
  percentage_of_total: number;
  top_artists: CategoryArtistRevenue[];
}

export interface GlobalCategoryRevenue {
  period: string;
  total_simulated_revenue: string;
  total_artists_with_offenses: number;
  clean_artist_revenue: string;
  problematic_artist_revenue: string;
  problematic_percentage: number;
  by_category: CategoryRevenue[];
  generated_at: string;
}

export interface AlbumRevenue {
  album_id: string | null;
  title: string;
  release_year: number | null;
  track_count: number;
  simulated_monthly_streams: number;
  simulated_monthly_revenue: string;
}

export interface ArtistOffenseSummary {
  category: string;
  severity: string;
  title: string;
  date: string | null;
}

export interface ArtistDiscographyRevenue {
  artist_id: string;
  artist_name: string;
  offenses: ArtistOffenseSummary[];
  total_albums: number;
  total_tracks: number;
  simulated_monthly_streams: number;
  simulated_monthly_revenue: string;
  simulated_yearly_revenue: string;
  albums: AlbumRevenue[];
}

export interface OffenseCategoryInfo {
  id: string;
  display_name: string;
}

// Revenue Types
export interface PayoutRate {
  platform: string;
  rate_per_stream: string;
  rate_per_minute?: string;
  subscription_monthly?: string;
  rate_tier: string;
  effective_date: string;
  source_url?: string;
}

export interface PlatformRevenue {
  platform: string;
  streams: number;
  listening_time_ms?: number;
  estimated_revenue: string;
  percentage_of_total: number;
  rate_applied: string;
}

export interface ArtistRevenueBreakdown {
  artist_id: string;
  artist_name: string;
  trouble_tier?: TroubleTier;
  trouble_score?: number;
  total_streams: number;
  total_revenue: string;
  percentage_of_user_spend: number;
  by_platform: PlatformRevenue[];
}

export interface UserRevenueDistribution {
  user_id: string;
  platform: string;
  period: string;
  total_streams: number;
  total_revenue: string;
  subscription_cost?: string;
  revenue_to_clean_artists: string;
  revenue_to_problematic_artists: string;
  problematic_percentage: number;
  top_artists: ArtistRevenueBreakdown[];
  top_problematic_artists: ArtistRevenueBreakdown[];
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
  // Trouble scores
  troubleLeaderboard: TroubleLeaderboardEntry[];
  tierDistribution: TierDistribution | null;
  selectedArtistScore: ArtistTroubleScore | null;
  // Revenue
  revenueDistribution: UserRevenueDistribution | null;
  topArtistsByRevenue: ArtistRevenueBreakdown[];
  problematicArtistRevenue: ArtistRevenueBreakdown[];
  payoutRates: PayoutRate[];
  // Category Revenue (Simulated)
  globalCategoryRevenue: GlobalCategoryRevenue | null;
  offenseCategories: OffenseCategoryInfo[];
  selectedCategoryRevenue: CategoryRevenue | null;
  artistDiscographyRevenue: ArtistDiscographyRevenue | null;
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
  // Trouble scores
  troubleLeaderboard: [],
  tierDistribution: null,
  selectedArtistScore: null,
  // Revenue
  revenueDistribution: null,
  topArtistsByRevenue: [],
  problematicArtistRevenue: [],
  payoutRates: [],
  // Category Revenue (Simulated)
  globalCategoryRevenue: null,
  offenseCategories: [],
  selectedCategoryRevenue: null,
  artistDiscographyRevenue: null,
  isLoading: false,
  error: null,
};

function asRecord(value: unknown): Record<string, unknown> | null {
  return value && typeof value === 'object' && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : null;
}

function getNestedRecord(
  value: unknown,
  keys: string[]
): Record<string, unknown> | null {
  const record = asRecord(value);
  if (!record) return null;

  for (const key of keys) {
    const nested = asRecord(record[key]);
    if (nested) return nested;
  }

  return null;
}

function getArray<T>(value: unknown, keys: string[] = []): T[] {
  if (Array.isArray(value)) return value as T[];

  const record = asRecord(value);
  if (!record) return [];

  for (const key of keys) {
    const nested = record[key];
    if (Array.isArray(nested)) {
      return nested as T[];
    }
  }

  return [];
}

function getNumber(
  value: Record<string, unknown> | null,
  keys: string[],
  fallback: number = 0
): number {
  if (!value) return fallback;

  for (const key of keys) {
    const candidate = value[key];
    if (typeof candidate === 'number' && Number.isFinite(candidate)) {
      return candidate;
    }

    if (typeof candidate === 'string') {
      const parsed = Number(candidate);
      if (Number.isFinite(parsed)) {
        return parsed;
      }
    }
  }

  return fallback;
}

function getOptionalNumber(
  value: Record<string, unknown> | null,
  keys: string[]
): number | undefined {
  if (!value) return undefined;

  for (const key of keys) {
    const candidate = value[key];
    if (typeof candidate === 'number' && Number.isFinite(candidate)) {
      return candidate;
    }

    if (typeof candidate === 'string') {
      const parsed = Number(candidate);
      if (Number.isFinite(parsed)) {
        return parsed;
      }
    }
  }

  return undefined;
}

function getString(
  value: Record<string, unknown> | null,
  keys: string[],
  fallback: string = ''
): string {
  if (!value) return fallback;

  for (const key of keys) {
    const candidate = value[key];
    if (typeof candidate === 'string' && candidate.length > 0) {
      return candidate;
    }
  }

  return fallback;
}

function getBoolean(
  value: Record<string, unknown> | null,
  keys: string[],
  fallback: boolean = false
): boolean {
  if (!value) return fallback;

  for (const key of keys) {
    const candidate = value[key];
    if (typeof candidate === 'boolean') {
      return candidate;
    }
  }

  return fallback;
}

function normalizeDashboardMetrics(data: unknown): DashboardMetrics {
  const root = asRecord(data);
  const userMetrics = getNestedRecord(data, ['user_metrics']);
  const communityMetrics = getNestedRecord(data, ['community_list_metrics']);
  const contentMetrics = getNestedRecord(data, ['content_metrics']);
  const recentOffenses = getArray<unknown>(root?.recent_offenses, ['entries', 'items']);

  return {
    total_users: getNumber(root, ['total_users'], getNumber(userMetrics, ['total_users'])),
    active_users_today: getNumber(root, ['active_users_today'], getNumber(userMetrics, ['active_users'])),
    total_blocked_artists: getNumber(root, ['total_blocked_artists'], getNumber(userMetrics, ['total_blocks'])),
    total_subscriptions: getNumber(root, ['total_subscriptions'], getNumber(communityMetrics, ['total_subscriptions'])),
    offense_detections_today: getNumber(
      root,
      ['offense_detections_today'],
      getNumber(contentMetrics, ['recent_offenses'], recentOffenses.length)
    ),
    sync_runs_today: getNumber(root, ['sync_runs_today']),
  };
}

function normalizeUserQuickStats(data: unknown): UserQuickStats {
  const root = asRecord(data);

  return {
    blocked_artists: getNumber(root, ['blocked_artists']),
    subscriptions: getNumber(root, ['subscriptions', 'list_subscriptions']),
    manual_blocks: getNumber(root, ['manual_blocks', 'blocked_tracks']),
    last_sync: getString(root, ['last_sync', 'last_activity'], ''),
  };
}

function normalizeSystemHealth(data: unknown): SystemHealth {
  const root = asRecord(data);
  const latencies = getNestedRecord(data, ['latencies_ms']);

  return {
    overall: (getString(root, ['overall', 'overall_status'], 'unhealthy') as SystemHealth['overall']),
    databases: {
      postgres: getBoolean(root, ['postgres', 'postgres_healthy']),
      redis: getBoolean(root, ['redis', 'redis_healthy']),
      duckdb: getBoolean(root, ['duckdb', 'duckdb_healthy']),
      kuzu: getBoolean(root, ['kuzu', 'kuzu_healthy']),
      lancedb: getBoolean(root, ['lancedb', 'lancedb_healthy']),
    },
    latencies_ms: {
      postgres: getOptionalNumber(latencies, ['postgres']) ?? getOptionalNumber(root, ['postgres_latency_ms']),
      redis: getOptionalNumber(latencies, ['redis']) ?? getOptionalNumber(root, ['redis_latency_ms']),
      duckdb: getOptionalNumber(latencies, ['duckdb']) ?? getOptionalNumber(root, ['duckdb_latency_ms']),
      kuzu: getOptionalNumber(latencies, ['kuzu']) ?? getOptionalNumber(root, ['kuzu_latency_ms']),
      lancedb: getOptionalNumber(latencies, ['lancedb']) ?? getOptionalNumber(root, ['lancedb_latency_ms']),
    },
  };
}

function normalizeTrendDirection(value: string): TrendData['trend'] {
  if (value === 'up' || value === 'down' || value === 'stable') {
    return value;
  }

  if (value === 'rising') return 'up';
  if (value === 'falling') return 'down';
  return 'stable';
}

function normalizeTrendSummary(data: unknown): TrendData {
  const root = asRecord(data);
  const contentTrend = getNestedRecord(data, ['content_volume_trend']);
  const activityTrend = getNestedRecord(data, ['user_activity_trend']);
  const dataPoints = getArray<Record<string, unknown>>(root?.data_points, ['entries', 'items'])
    .map((point) => ({
      date: getString(point, ['date', 'label'], ''),
      value: getNumber(point, ['value', 'count', 'total']),
    }))
    .filter((point) => point.date.length > 0);

  return {
    period: getString(root, ['period'], 'Current selection'),
    data_points: dataPoints,
    change_percent: getNumber(root, ['change_percent'], getNumber(contentTrend, ['change_percentage'], getNumber(activityTrend, ['change_percentage']))),
    trend: normalizeTrendDirection(getString(root, ['trend'], getString(contentTrend, ['direction'], getString(activityTrend, ['direction'], 'stable')))),
  };
}

function normalizeArtistTrends(
  data: unknown,
  fallbackTrend: ArtistTrend['trend']
): ArtistTrend[] {
  return getArray<Record<string, unknown>>(data, ['artists', 'entries', 'items'])
    .map((artist) => ({
      artist_id: getString(artist, ['artist_id', 'id']),
      artist_name: getString(artist, ['artist_name', 'canonical_name', 'name'], 'Unknown Artist'),
      mentions: getNumber(artist, ['mentions', 'mention_count', 'count']),
      sentiment: getNumber(artist, ['sentiment', 'sentiment_score']),
      offense_count: getNumber(artist, ['offense_count', 'offenses']),
      trend: (getString(artist, ['trend'], fallbackTrend) as ArtistTrend['trend']),
    }))
    .filter((artist) => artist.artist_id.length > 0);
}

function normalizeRevenueDistribution(data: unknown): UserRevenueDistribution {
  const root = asRecord(data);
  const subscriptionCost = getString(root, ['subscription_cost'], '');

  return {
    user_id: getString(root, ['user_id']),
    platform: getString(root, ['platform'], 'all'),
    period: getString(root, ['period']),
    total_streams: getNumber(root, ['total_streams']),
    total_revenue: getString(root, ['total_revenue'], '0'),
    subscription_cost: subscriptionCost || undefined,
    revenue_to_clean_artists: getString(root, ['revenue_to_clean_artists'], '0'),
    revenue_to_problematic_artists: getString(root, ['revenue_to_problematic_artists'], '0'),
    problematic_percentage: getNumber(root, ['problematic_percentage']),
    top_artists: getArray<ArtistRevenueBreakdown>(data, ['top_artists', 'artists']),
    top_problematic_artists: getArray<ArtistRevenueBreakdown>(data, ['top_problematic_artists', 'problematic_artists', 'artists']),
  };
}

function normalizePayoutRates(data: unknown): PayoutRate[] {
  return getArray<PayoutRate>(data, ['rates', 'entries', 'items']);
}

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
        const dashboard = normalizeDashboardMetrics(result.data);
        analyticsStore.update(s => ({
          ...s,
          dashboard,
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
        const userStats = normalizeUserQuickStats(result.data);
        analyticsStore.update(s => ({
          ...s,
          userStats,
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
        const health = normalizeSystemHealth(result.data);
        analyticsStore.update(s => ({
          ...s,
          systemHealth: health,
        }));
        return { success: true, health };
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
        const summary = normalizeTrendSummary(result.data);
        const rising = normalizeArtistTrends(
          asRecord(result.data)?.top_rising_artists ?? [],
          'rising'
        );
        const falling = normalizeArtistTrends(
          asRecord(result.data)?.top_falling_artists ?? [],
          'falling'
        );
        analyticsStore.update(s => ({
          ...s,
          trends: {
            ...s.trends,
            summary,
            artists: [
              ...s.trends.artists.filter((artist) => artist.trend !== 'rising' && artist.trend !== 'falling'),
              ...rising,
              ...falling,
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

  fetchRisingArtists: async (limit: number = 10) => {
    try {
      const result = await apiClient.authenticatedRequest<{ artists: ArtistTrend[] }>(
        'GET',
        `/api/v1/analytics/trends/rising?limit=${limit}`
      );

      if (result.success && result.data) {
        const artists = normalizeArtistTrends(result.data, 'rising');
        analyticsStore.update(s => ({
          ...s,
          trends: {
            ...s.trends,
            artists: [
              ...s.trends.artists.filter(a => a.trend !== 'rising'),
              ...artists,
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
        const artists = normalizeArtistTrends(result.data, 'falling');
        analyticsStore.update(s => ({
          ...s,
          trends: {
            ...s.trends,
            artists: [
              ...s.trends.artists.filter(a => a.trend !== 'falling'),
              ...artists,
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

  // Trouble Score Actions
  fetchTroubleLeaderboard: async (minTier?: TroubleTier, limit: number = 20, offset: number = 0) => {
    try {
      const params = new URLSearchParams({ limit: limit.toString(), offset: offset.toString() });
      if (minTier) params.append('min_tier', minTier);

      const result = await apiClient.authenticatedRequest<{ entries: TroubleLeaderboardEntry[] }>(
        'GET',
        `/api/v1/analytics/trouble-scores/leaderboard?${params}`
      );

      if (result.success && result.data) {
        analyticsStore.update(s => ({
          ...s,
          troubleLeaderboard: result.data!.entries,
        }));
        return { success: true };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      return { success: false, message: error.message };
    }
  },

  fetchTierDistribution: async () => {
    try {
      const result = await apiClient.authenticatedRequest<TierDistribution>(
        'GET',
        '/api/v1/analytics/trouble-scores/distribution'
      );

      if (result.success && result.data) {
        analyticsStore.update(s => ({
          ...s,
          tierDistribution: result.data!,
        }));
        return { success: true };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      return { success: false, message: error.message };
    }
  },

  fetchArtistTroubleScore: async (artistId: string) => {
    try {
      const result = await apiClient.authenticatedRequest<ArtistTroubleScore>(
        'GET',
        `/api/v1/analytics/trouble-scores/${artistId}`
      );

      if (result.success && result.data) {
        analyticsStore.update(s => ({
          ...s,
          selectedArtistScore: result.data!,
        }));
        return { success: true, score: result.data };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      return { success: false, message: error.message };
    }
  },

  // Revenue Actions
  fetchRevenueDistribution: async (days: number = 30, platform?: string) => {
    try {
      const params = new URLSearchParams({ days: days.toString() });
      if (platform) params.append('platform', platform);

      const result = await apiClient.authenticatedRequest<UserRevenueDistribution>(
        'GET',
        `/api/v1/analytics/revenue/distribution?${params}`
      );

      if (result.success && result.data) {
        const distribution = normalizeRevenueDistribution(result.data);
        analyticsStore.update(s => ({
          ...s,
          revenueDistribution: distribution,
        }));
        return { success: true };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      return { success: false, message: error.message };
    }
  },

  fetchTopArtistsByRevenue: async (days: number = 30, limit: number = 10) => {
    try {
      const result = await apiClient.authenticatedRequest<{ artists: ArtistRevenueBreakdown[] }>(
        'GET',
        `/api/v1/analytics/revenue/top-artists?days=${days}&limit=${limit}`
      );

      if (result.success && result.data) {
        analyticsStore.update(s => ({
          ...s,
          topArtistsByRevenue: result.data!.artists,
        }));
        return { success: true };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      return { success: false, message: error.message };
    }
  },

  fetchProblematicArtistRevenue: async (days: number = 30, minTier: TroubleTier = 'moderate', limit: number = 10) => {
    try {
      const result = await apiClient.authenticatedRequest<{ artists: ArtistRevenueBreakdown[] }>(
        'GET',
        `/api/v1/analytics/revenue/problematic?days=${days}&min_tier=${minTier}&limit=${limit}`
      );

      if (result.success && result.data) {
        const artists = getArray<ArtistRevenueBreakdown>(result.data, ['artists', 'entries', 'items']);
        analyticsStore.update(s => ({
          ...s,
          problematicArtistRevenue: artists,
        }));
        return { success: true };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      return { success: false, message: error.message };
    }
  },

  fetchPayoutRates: async () => {
    try {
      const result = await apiClient.authenticatedRequest<{ rates: PayoutRate[] }>(
        'GET',
        '/api/v1/analytics/revenue/payout-rates'
      );

      if (result.success && result.data) {
        const rates = normalizePayoutRates(result.data);
        analyticsStore.update(s => ({
          ...s,
          payoutRates: rates,
        }));
        return { success: true };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      return { success: false, message: error.message };
    }
  },

  fetchArtistRevenue: async (artistId: string, days: number = 30) => {
    try {
      const result = await apiClient.authenticatedRequest<ArtistRevenueBreakdown>(
        'GET',
        `/api/v1/analytics/revenue/artist/${artistId}?days=${days}`
      );

      if (result.success && result.data) {
        return { success: true, breakdown: result.data };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      return { success: false, message: error.message };
    }
  },

  // Category Revenue Actions (Simulated by Offense Category)
  fetchGlobalCategoryRevenue: async () => {
    analyticsStore.update(s => ({ ...s, isLoading: true, error: null }));

    try {
      const result = await apiClient.authenticatedRequest<GlobalCategoryRevenue>(
        'GET',
        '/api/v1/analytics/category-revenue'
      );

      if (result.success && result.data) {
        analyticsStore.update(s => ({
          ...s,
          globalCategoryRevenue: result.data!,
          isLoading: false,
        }));
        return { success: true, data: result.data };
      } else {
        analyticsStore.update(s => ({
          ...s,
          isLoading: false,
          error: result.message || 'Failed to fetch category revenue',
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

  fetchOffenseCategories: async () => {
    try {
      const result = await apiClient.authenticatedRequest<{ categories: OffenseCategoryInfo[] }>(
        'GET',
        '/api/v1/analytics/category-revenue/categories'
      );

      if (result.success && result.data) {
        analyticsStore.update(s => ({
          ...s,
          offenseCategories: result.data!.categories,
        }));
        return { success: true, categories: result.data.categories };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      return { success: false, message: error.message };
    }
  },

  fetchCategoryRevenue: async (category: string, topN: number = 10) => {
    try {
      const result = await apiClient.authenticatedRequest<CategoryRevenue>(
        'GET',
        `/api/v1/analytics/category-revenue/${category}?top_n=${topN}`
      );

      if (result.success && result.data) {
        analyticsStore.update(s => ({
          ...s,
          selectedCategoryRevenue: result.data!,
        }));
        return { success: true, data: result.data };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      return { success: false, message: error.message };
    }
  },

  fetchArtistDiscographyRevenue: async (artistId: string) => {
    try {
      const result = await apiClient.authenticatedRequest<ArtistDiscographyRevenue>(
        'GET',
        `/api/v1/analytics/category-revenue/artist/${artistId}/discography`
      );

      if (result.success && result.data) {
        analyticsStore.update(s => ({
          ...s,
          artistDiscographyRevenue: result.data!,
        }));
        return { success: true, data: result.data };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      return { success: false, message: error.message };
    }
  },

  fetchUserCategoryExposure: async (days: number = 30) => {
    try {
      const result = await apiClient.authenticatedRequest<{ categories: CategoryRevenue[] }>(
        'GET',
        `/api/v1/analytics/category-revenue/user/exposure?days=${days}`
      );

      if (result.success && result.data) {
        return { success: true, categories: result.data.categories };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      return { success: false, message: error.message };
    }
  },

  clearCategoryRevenue: () => {
    analyticsStore.update(s => ({
      ...s,
      globalCategoryRevenue: null,
      selectedCategoryRevenue: null,
      artistDiscographyRevenue: null,
    }));
  },
};
