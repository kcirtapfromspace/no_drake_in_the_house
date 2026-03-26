export const HEAVY_LIBRARY_GROUP_TTL_MS = 45_000;

export interface SyncDashboardHeavyCache {
  refreshedAt: number;
  libraryStatsRows: unknown[] | null;
  tasteGrade: unknown | null;
  libraryOffenders: unknown | null;
  libraryOffendersScope: 'songs' | 'all';
  libraryOffendersDays: number;
  libraryItems: unknown[] | null;
  libraryItemsTotal: number;
  libraryItemsOffset: number;
  libraryItemsKey: string | null;
  libraryGroups: unknown[] | null;
  libraryGroupsTotal: number;
  libraryGroupsOffset: number;
  libraryGroupsKey: string | null;
}

export const syncDashboardHeavyCache: SyncDashboardHeavyCache = {
  refreshedAt: 0,
  libraryStatsRows: null,
  tasteGrade: null,
  libraryOffenders: null,
  libraryOffendersScope: 'songs',
  libraryOffendersDays: 0,
  libraryItems: null,
  libraryItemsTotal: 0,
  libraryItemsOffset: 0,
  libraryItemsKey: null,
  libraryGroups: null,
  libraryGroupsTotal: 0,
  libraryGroupsOffset: 0,
  libraryGroupsKey: null,
};
