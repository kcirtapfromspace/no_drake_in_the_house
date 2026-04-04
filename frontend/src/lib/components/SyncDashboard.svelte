<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { get } from 'svelte/store';
  import { syncStore, syncActions, isAnySyncRunning, platformsStatus, recentRuns } from '../stores/sync';
  import type { TriggerSyncRequest } from '../stores/sync';
  import { navigateTo, navigateToArtist } from '../utils/simple-router';
  import { connectionsStore, connectionActions } from '../stores/connections';
  import { apiClient } from '../utils/api-client';
  import { blockingStore } from '../stores/blocking';
  import { timeAgo } from '../utils/time-ago';
  import { HEAVY_LIBRARY_GROUP_TTL_MS, syncDashboardHeavyCache } from '../utils/sync-dashboard-cache';
  import ServiceConnector from './ServiceConnector.svelte';
  import ProviderIcon from './ProviderIcon.svelte';
  import { billingActions } from '../stores/billing';


  // Connection states
  let connectionSuccess: string | null = null;
  let connectionError: string | null = null;
  let connectionBannerTimeout: ReturnType<typeof setTimeout> | null = null;

  interface AppleLibraryPreview {
    tracks: Array<{ id: string; name?: string; artist?: string; album?: string }>;
    albums: Array<{ id: string; name?: string; artist?: string }>;
    playlists: Array<{ id: string; name?: string; track_count?: number }>;
    tracksCount: number;
    albumsCount: number;
    artistsCount: number;
    playlistsCount: number;
    scannedAt?: string;
  }

  let appleLibrary: AppleLibraryPreview | null = null;
  let appleLibraryError: string | null = null;
  let appleLibrarySyncPolling = false;

  interface AppleLibrarySyncStatus {
    state: 'idle' | 'running' | 'completed' | 'failed';
    message: string;
    started_at: string;
    completed_at?: string;
    tracks_count?: number;
    albums_count?: number;
    playlists_count?: number;
    imported_items_count?: number;
    added?: number;
    removed?: number;
    unchanged?: number;
  }

  let appleLibrarySyncStatus: AppleLibrarySyncStatus | null = null;

  type ProviderSyncPlatform = 'spotify' | 'tidal' | 'youtube';

  interface ProviderLibrarySyncStatus {
    state: 'idle' | 'running' | 'completed' | 'failed';
    message: string;
    started_at: string;
    completed_at?: string;
    tracks_count?: number;
    albums_count?: number;
    artists_count?: number;
    playlists_count?: number;
    imported_items_count?: number;
    // Progress fields from Convex providerSyncStatus query
    phase?: string | null;
    tracks_imported?: number;
    liked_count?: number;
    album_count?: number;
    artist_count?: number;
    playlist_track_count?: number;
    duration_ms?: number | null;
    error_message?: string | null;
    run_id?: string;
  }

  interface ImportedLibraryTrack {
    id?: string;
    user_id?: string;
    provider?: string;
    provider_track_id?: string;
    track_name?: string;
    album_name?: string;
    artist_id?: string;
    artist_name?: string;
    source_type?: string;
    playlist_name?: string;
    last_synced?: string;
    added_at?: string;
  }

  interface ProviderLibraryStatsRow {
    provider: string;
    providerName: string;
    songs: number | null;
    albums: number | null;
    artists: number | null;
    playlists: number | null;
    totalItems: number | null;
    lastSynced?: string;
    source: 'live_api' | 'imported_cache';
    status: 'ready' | 'error' | 'syncing' | 'not_synced';
    message?: string;
  }

  let libraryStatsRows: ProviderLibraryStatsRow[] = [];
  let libraryStatsLoading = false;
  let libraryStatsError: string | null = null;
  let libraryStatsByProvider = new Map<string, ProviderLibraryStatsRow>();

  type LibraryItemKind = 'songs' | 'albums' | 'artists' | 'playlists' | 'all';

  let libraryItemsProviderFilter = 'all';
  // Default to "Everything" so users immediately see albums/playlists when a provider has 0 songs.
  let libraryItemsKindFilter: LibraryItemKind = 'all';
  let libraryItemsQuery = '';

  type LibraryBrowseView = 'items' | 'grouped';
  type LibraryGroupBy = 'artist' | 'album' | 'playlist' | 'provider' | 'kind';

  type LibraryItemSort = 'last_synced' | 'added_at' | 'title' | 'artist' | 'album' | 'provider';
  type LibraryGroupSort = 'count' | 'name' | 'last_synced';

  interface LibraryItemsPage {
    items: ImportedLibraryTrack[];
    total: number;
    limit: number;
    offset: number;
  }

  interface LibraryGroup {
    value: string;
    secondary: string | null;
    provider: string | null;
    count: number;
    last_synced: string;
  }

  interface LibraryGroupsPage {
    group_by: string;
    groups: LibraryGroup[];
    total: number;
    limit: number;
    offset: number;
  }

  interface TasteGradeComponent {
    id: string;
    label: string;
    weight: number;
    score: number;
    grade: string;
    summary: string;
  }

  interface TasteGradeResponse {
    computed_at: string;
    overall_score: number;
    overall_grade: string;
    components: TasteGradeComponent[];
    signals: string[];
    recommendations: string[];
  }

  interface OffenderOffenseSummary {
    category: string;
    title: string;
    date: string;
    evidence_count: number;
  }

  interface OffenderArtist {
    id: string;
    name: string;
    track_count: number;
    severity: 'minor' | 'moderate' | 'severe' | 'egregious';
    offenses: OffenderOffenseSummary[];
    play_count: number | null;
    estimated_revenue: string | null;
    percentage_of_user_spend: number | null;
  }

  interface LibraryOffendersResponse {
    computed_at: string;
    total_flagged_artists: number;
    total_flagged_tracks: number;
    playcount_window_days: number;
    playcounts_available: boolean;
    offenders: OffenderArtist[];
  }

  let libraryBrowseView: LibraryBrowseView = 'items';
  let libraryGroupBy: LibraryGroupBy = 'artist';
  let librarySort: LibraryItemSort = 'last_synced';
  let librarySortDir: 'asc' | 'desc' = 'desc';
  let libraryGroupSort: LibraryGroupSort = 'count';
  let libraryGroupSortDir: 'asc' | 'desc' = 'desc';

  let libraryFilterArtist = '';
  let libraryFilterAlbum = '';
  let libraryFilterPlaylist = '';

  let libraryItems: ImportedLibraryTrack[] = [];
  let libraryItemsTotal = 0;
  let libraryItemsLoading = false;
  let libraryItemsError: string | null = null;
  let libraryItemsLimit = 50;
  let libraryItemsOffset = 0;

  let libraryGroups: LibraryGroup[] = [];
  let libraryGroupsTotal = 0;
  let libraryGroupsLoading = false;
  let libraryGroupsError: string | null = null;
  let libraryGroupsLimit = 50;
  let libraryGroupsOffset = 0;

  let tasteGrade: TasteGradeResponse | null = null;
  let tasteGradeLoading = false;
  let tasteGradeError: string | null = null;

  let libraryOffenders: LibraryOffendersResponse | null = null;
  let libraryOffendersLoading = false;
  let libraryOffendersError: string | null = null;
  let libraryOffendersScope: 'songs' | 'all' = 'songs';
  let libraryOffendersDays = 0;
  let libraryOffendersHasPlaycounts = false;

  const libraryOffendersPeriodOptions = [
    { value: 0, label: 'All time' },
    { value: 7, label: 'Last 7d' },
    { value: 30, label: 'Last 30d' },
    { value: 90, label: 'Last 90d' },
    { value: 365, label: 'Last year' },
  ];

  let libraryQueryDebounce: ReturnType<typeof setTimeout> | null = null;
  const GENERIC_SYNC_POLL_INTERVAL_MS = 4_000;
  const GENERIC_SYNC_SETTLE_WINDOW_MS = 12_000;
  const PROVIDER_SYNC_POLL_INTERVAL_MS = 4_000;
  const PROVIDER_SYNC_MAX_WAIT_MS = 1_200_000;
  const POLLED_SYNC_PLATFORMS = new Set(['spotify', 'tidal', 'youtube', 'youtube_music']);
  const PROVIDER_SYNC_ENDPOINTS: Record<ProviderSyncPlatform, string> = {
    spotify: '/api/v1/connections/spotify/library/sync-status',
    tidal: '/api/v1/connections/tidal/library/sync-status',
    youtube: '/api/v1/connections/youtube/library/sync-status',
  };

  let heavyLibraryGroupRefreshedAt = syncDashboardHeavyCache.refreshedAt;
  let genericSyncPolling = false;
  let genericSyncPollTimer: ReturnType<typeof setTimeout> | null = null;
  let genericSyncPollDeadline = 0;
  let genericSyncRefreshPending = false;
  let activeGenericSyncProviders: string[] = [];
  let providerSyncStatusByPlatform: Partial<Record<ProviderSyncPlatform, ProviderLibrarySyncStatus | null>> = {};
  let providerSyncPolling = false;
  let providerSyncPollTimer: ReturnType<typeof setTimeout> | null = null;
  let providerSyncPollDeadline = 0;
  let providerSyncRefreshPending = false;
  let polledProviderSyncs = new Set<ProviderSyncPlatform>();

  type PlatformStatus = 'ready' | 'paused' | 'catalog-only';

  interface Platform {
    id: string;
    name: string;
    abbr: string;
    icon: string;
    color: string;
    status: PlatformStatus;
    statusLabel: string;
    disabled?: boolean;
    connectionProvider?: string; // Maps to connection provider name
  }

  const platforms: Platform[] = [
    { id: 'spotify', name: 'Spotify', abbr: 'SP', icon: 'spotify', color: '#1DB954', status: 'ready', statusLabel: 'Ready', connectionProvider: 'spotify' },
    { id: 'apple', name: 'Apple Music', abbr: 'AM', icon: 'apple', color: '#FA2D48', status: 'ready', statusLabel: 'Ready', connectionProvider: 'apple_music' },
    { id: 'youtube', name: 'YouTube Music', abbr: 'YT', icon: 'youtube', color: '#FF0000', status: 'ready', statusLabel: 'Ready', connectionProvider: 'youtube_music' },
    { id: 'tidal', name: 'Tidal', abbr: 'TI', icon: 'tidal', color: '#000000', status: 'ready', statusLabel: 'Ready', connectionProvider: 'tidal' },
    { id: 'deezer', name: 'Deezer', abbr: 'DZ', icon: 'deezer', color: '#FEAA2D', status: 'catalog-only', statusLabel: 'Catalog Only' },
  ];

  // Derived connection state used by the template.
  //
  // Important: avoid relying on helper functions (e.g. getConnectedPlatforms()) inside markup
  // because Svelte's dependency tracking is static and won't see store usage inside function
  // bodies, which can leave the UI stuck in a stale branch.
  let activeProviders = new Set<string>();
  let connectedPlatforms: Platform[] = [];

  $: activeProviders = new Set<string>(
    ($connectionsStore.connections ?? [])
      .filter((conn) => conn.status === 'active')
      .map((conn) => conn.provider)
  );

  $: connectedPlatforms = platforms.filter((platform) => {
    const provider = platform.connectionProvider;
    return provider != null && !platform.disabled && activeProviders.has(provider);
  });

  $: activeGenericSyncProviders = $platformsStatus
    .filter((status) => POLLED_SYNC_PLATFORMS.has((status.platform || '').toLowerCase()) && status.status === 'running')
    .map((status) => status.platform);

  $: if (activeGenericSyncProviders.length > 0) {
    queueGenericSyncPolling();
  }

  $: libraryStatsByProvider = new Map<string, ProviderLibraryStatsRow>(
    (libraryStatsRows ?? []).map((row) => [row.provider, row])
  );

  function getPlatformById(platformId: string): Platform | undefined {
    return platforms.find((platform) => platform.id === platformId);
  }

  // Use when you need the latest connection state immediately after an async fetch.
  // Avoid using this in markup (it is not reactive).
  function hasActiveConnection(provider?: string): boolean {
    if (!provider) return false;
    const state = get(connectionsStore);
    return state.connections.some((conn) => conn.provider === provider && conn.status === 'active');
  }

  function getProviderName(provider: string): string {
    return platforms.find(platform => (platform.connectionProvider || platform.id) === provider)?.name ?? provider;
  }

  function kindFromImportedItem(item: ImportedLibraryTrack): Exclude<LibraryItemKind, 'all'> {
    const sourceType = (item.source_type ?? '').toLowerCase();
    const providerTrackId = (item.provider_track_id ?? '').toLowerCase();

    if (sourceType.includes('playlist') || providerTrackId.startsWith('playlist:')) return 'playlists';
    if (sourceType.includes('artist') || providerTrackId.startsWith('artist:')) return 'artists';
    if (sourceType.includes('album') || providerTrackId.startsWith('album:')) return 'albums';
    return 'songs';
  }

  function formatOffenseCategory(value: string): string {
    const normalized = (value || '').trim().replace(/_/g, ' ');
    if (!normalized) return 'Other';
    return normalized.replace(/\b\w/g, (c) => c.toUpperCase());
  }

  function formatSeverity(value: string): string {
    return value.charAt(0).toUpperCase() + value.slice(1);
  }

  function severityBadgeClass(value: OffenderArtist['severity']): string {
    switch (value) {
      case 'egregious': return 'border-rose-500/30 bg-rose-500/10 text-rose-300';
      case 'severe': return 'border-red-500/30 bg-red-500/10 text-red-300';
      case 'moderate': return 'border-orange-500/30 bg-orange-500/10 text-orange-300';
      case 'minor': return 'border-amber-500/30 bg-amber-500/10 text-amber-300';
      default: return 'border-zinc-500/30 bg-zinc-500/10 text-zinc-300';
    }
  }

  function hasRunningGenericSync(): boolean {
    return get(platformsStatus).some(
      (status) => POLLED_SYNC_PLATFORMS.has((status.platform || '').toLowerCase()) && status.status === 'running'
    );
  }

  function getProviderSyncPlatform(platformId: string): ProviderSyncPlatform | null {
    if (platformId === 'spotify' || platformId === 'tidal' || platformId === 'youtube') {
      return platformId;
    }

    return null;
  }

  function getProviderSyncStatusForPlatform(platformId: string): ProviderLibrarySyncStatus | null {
    const key = getProviderSyncPlatform(platformId);
    return key ? (providerSyncStatusByPlatform[key] ?? null) : null;
  }

  function getConnectedProviderSyncPlatforms(): ProviderSyncPlatform[] {
    return platforms
      .filter((platform) => {
        const providerSyncPlatform = getProviderSyncPlatform(platform.id);
        return (
          providerSyncPlatform !== null &&
          platform.connectionProvider != null &&
          hasActiveConnection(platform.connectionProvider)
        );
      })
      .map((platform) => getProviderSyncPlatform(platform.id))
      .filter((platform): platform is ProviderSyncPlatform => platform !== null);
  }

  function hasRunningProviderSync(): boolean {
    if (polledProviderSyncs.size > 0) return true;

    return Object.values(providerSyncStatusByPlatform).some(
      (status) => status?.state === 'running'
    );
  }

  function clearGenericSyncPollTimer(): void {
    if (genericSyncPollTimer) {
      clearTimeout(genericSyncPollTimer);
      genericSyncPollTimer = null;
    }
  }

  function scheduleGenericSyncPoll(delayMs: number): void {
    clearGenericSyncPollTimer();
    genericSyncPollTimer = setTimeout(() => {
      void runGenericSyncPoll();
    }, delayMs);
  }

  function queueGenericSyncPolling(): void {
    genericSyncRefreshPending = true;
    genericSyncPollDeadline = Date.now() + GENERIC_SYNC_SETTLE_WINDOW_MS;

    if (genericSyncPolling || genericSyncPollTimer) return;

    scheduleGenericSyncPoll(0);
  }

  function clearProviderSyncPollTimer(): void {
    if (providerSyncPollTimer) {
      clearTimeout(providerSyncPollTimer);
      providerSyncPollTimer = null;
    }
  }

  function scheduleProviderSyncPoll(delayMs: number): void {
    clearProviderSyncPollTimer();
    providerSyncPollTimer = setTimeout(() => {
      void runProviderSyncPoll();
    }, delayMs);
  }

  async function refreshProviderLibrarySyncStatus(
    platform: ProviderSyncPlatform
  ): Promise<ProviderLibrarySyncStatus | null> {
    try {
      const result = await apiClient.authenticatedRequest<ProviderLibrarySyncStatus>(
        'GET',
        PROVIDER_SYNC_ENDPOINTS[platform],
        undefined
      );

      const status = result.success && result.data ? result.data : null;
      providerSyncStatusByPlatform = {
        ...providerSyncStatusByPlatform,
        [platform]: status,
      };

      return status;
    } catch {
      providerSyncStatusByPlatform = {
        ...providerSyncStatusByPlatform,
        [platform]: null,
      };
      return null;
    }
  }

  async function refreshConnectedProviderSyncStatuses(
    options: { queueRunning?: boolean } = {}
  ): Promise<void> {
    const providerSyncPlatforms = getConnectedProviderSyncPlatforms();
    if (providerSyncPlatforms.length === 0) return;

    const statuses = await Promise.all(
      providerSyncPlatforms.map((platform) => refreshProviderLibrarySyncStatus(platform))
    );

    if (options.queueRunning !== true) return;

    const runningProviders = providerSyncPlatforms.filter(
      (_, index) => statuses[index]?.state === 'running'
    );

    if (runningProviders.length > 0) {
      queueProviderSyncPolling(runningProviders, true);
    }
  }

  function queueProviderSyncPolling(
    platformsToPoll: ProviderSyncPlatform[],
    refreshAfterCompletion = true
  ): void {
    if (platformsToPoll.length === 0) return;

    polledProviderSyncs = new Set([...polledProviderSyncs, ...platformsToPoll]);
    if (refreshAfterCompletion) {
      providerSyncRefreshPending = true;
      providerSyncPollDeadline = Date.now() + PROVIDER_SYNC_MAX_WAIT_MS;
    }

    if (providerSyncPolling || providerSyncPollTimer) return;

    scheduleProviderSyncPoll(0);
  }

  function hydrateHeavyLibraryViewsFromCache(options: { allowStale?: boolean } = {}): boolean {
    const { allowStale = false } = options;

    if (syncDashboardHeavyCache.refreshedAt <= 0) return false;
    if (
      !allowStale &&
      Date.now() - syncDashboardHeavyCache.refreshedAt >= HEAVY_LIBRARY_GROUP_TTL_MS
    ) {
      return false;
    }
    if (syncDashboardHeavyCache.libraryItemsKey !== buildLibraryItemsEndpoint(0, libraryItemsLimit)) return false;
    if (syncDashboardHeavyCache.libraryGroupsKey !== buildLibraryGroupsEndpoint(0, libraryGroupsLimit)) return false;
    if (syncDashboardHeavyCache.libraryOffendersScope !== libraryOffendersScope) return false;
    if (syncDashboardHeavyCache.libraryOffendersDays !== libraryOffendersDays) return false;

    libraryStatsRows = (syncDashboardHeavyCache.libraryStatsRows ?? []) as ProviderLibraryStatsRow[];
    tasteGrade = (syncDashboardHeavyCache.tasteGrade ?? null) as TasteGradeResponse | null;
    libraryOffenders = (syncDashboardHeavyCache.libraryOffenders ?? null) as LibraryOffendersResponse | null;
    libraryItems = (syncDashboardHeavyCache.libraryItems ?? []) as ImportedLibraryTrack[];
    libraryItemsTotal = syncDashboardHeavyCache.libraryItemsTotal;
    libraryItemsOffset = syncDashboardHeavyCache.libraryItemsOffset;
    libraryGroups = (syncDashboardHeavyCache.libraryGroups ?? []) as LibraryGroup[];
    libraryGroupsTotal = syncDashboardHeavyCache.libraryGroupsTotal;
    libraryGroupsOffset = syncDashboardHeavyCache.libraryGroupsOffset;
    heavyLibraryGroupRefreshedAt = syncDashboardHeavyCache.refreshedAt;

    libraryStatsError = null;
    tasteGradeError = null;
    libraryOffendersError = null;
    libraryItemsError = null;
    libraryGroupsError = null;

    return true;
  }

  function persistHeavyLibraryViewsToCache(): void {
    syncDashboardHeavyCache.refreshedAt = heavyLibraryGroupRefreshedAt;
    syncDashboardHeavyCache.libraryStatsRows = libraryStatsRows;
    syncDashboardHeavyCache.tasteGrade = tasteGrade;
    syncDashboardHeavyCache.libraryOffenders = libraryOffenders;
    syncDashboardHeavyCache.libraryOffendersScope = libraryOffendersScope;
    syncDashboardHeavyCache.libraryOffendersDays = libraryOffendersDays;
    syncDashboardHeavyCache.libraryItems = libraryItems;
    syncDashboardHeavyCache.libraryItemsTotal = libraryItemsTotal;
    syncDashboardHeavyCache.libraryItemsOffset = libraryItemsOffset;
    syncDashboardHeavyCache.libraryItemsKey = buildLibraryItemsEndpoint(0, libraryItemsLimit);
    syncDashboardHeavyCache.libraryGroups = libraryGroups;
    syncDashboardHeavyCache.libraryGroupsTotal = libraryGroupsTotal;
    syncDashboardHeavyCache.libraryGroupsOffset = libraryGroupsOffset;
    syncDashboardHeavyCache.libraryGroupsKey = buildLibraryGroupsEndpoint(0, libraryGroupsLimit);
  }

  async function runProviderSyncPoll(): Promise<void> {
    if (providerSyncPolling) return;

    clearProviderSyncPollTimer();
    providerSyncPolling = true;

    try {
      const platformsToPoll = [...polledProviderSyncs];
      if (platformsToPoll.length === 0) {
        providerSyncRefreshPending = false;
        return;
      }

      const statuses = await Promise.all(
        platformsToPoll.map((platform) => refreshProviderLibrarySyncStatus(platform))
      );
      const runningProviders = platformsToPoll.filter(
        (_, index) => statuses[index]?.state === 'running'
      );
      const failedStatus = statuses.find((status) => status?.state === 'failed');

      polledProviderSyncs = new Set(runningProviders);

      if (failedStatus?.message) {
        showConnectionError(failedStatus.message);
      }

      if (runningProviders.length > 0 && Date.now() < providerSyncPollDeadline) {
        scheduleProviderSyncPoll(PROVIDER_SYNC_POLL_INTERVAL_MS);
        return;
      }

      await connectionActions.fetchConnections();

      if (providerSyncRefreshPending) {
        providerSyncRefreshPending = false;
        await refreshHeavyLibraryViews({ force: true });
      }
    } finally {
      providerSyncPolling = false;
    }
  }

  function shouldSkipHeavyLibraryGroupRefresh(force = false): boolean {
    if (force) return false;
    if (hasRunningGenericSync() || hasRunningProviderSync()) return true;
    return (
      heavyLibraryGroupRefreshedAt > 0 &&
      Date.now() - heavyLibraryGroupRefreshedAt < HEAVY_LIBRARY_GROUP_TTL_MS
    );
  }

  async function refreshHeavyLibraryViews(options: { force?: boolean } = {}): Promise<void> {
    if (!options.force && hydrateHeavyLibraryViewsFromCache()) {
      return;
    }

    if (!options.force && (hasRunningGenericSync() || hasRunningProviderSync())) {
      hydrateHeavyLibraryViewsFromCache({ allowStale: true });
      return;
    }

    if (shouldSkipHeavyLibraryGroupRefresh(options.force === true)) {
      return;
    }

    await Promise.all([
      refreshLibraryStats(),
      refreshTasteGrade(),
      refreshLibraryOffenders(),
      refreshLibraryExplorer(true),
    ]);
    heavyLibraryGroupRefreshedAt = Date.now();
    persistHeavyLibraryViewsToCache();
  }

  async function runGenericSyncPoll(): Promise<void> {
    if (genericSyncPolling) return;

    clearGenericSyncPollTimer();
    genericSyncPolling = true;

    try {
      await syncActions.fetchStatus();

      if (hasRunningGenericSync()) {
        scheduleGenericSyncPoll(GENERIC_SYNC_POLL_INTERVAL_MS);
        return;
      }

      if (genericSyncRefreshPending && Date.now() < genericSyncPollDeadline) {
        scheduleGenericSyncPoll(1500);
        return;
      }

      if (genericSyncRefreshPending) {
        genericSyncRefreshPending = false;
        await refreshHeavyLibraryViews({ force: true });
      }
    } finally {
      genericSyncPolling = false;
    }
  }

  function getLatestTimestamp(values: Array<string | undefined>): string | undefined {
    let latest: string | undefined;

    for (const value of values) {
      if (!value) continue;
      const timestamp = new Date(value).getTime();
      if (Number.isNaN(timestamp)) continue;

      if (!latest || timestamp > new Date(latest).getTime()) {
        latest = value;
      }
    }

    return latest;
  }

  async function fetchImportedLibraryTracks(provider: string): Promise<ImportedLibraryTrack[]> {
    const result = await apiClient.authenticatedRequest<any>(
      'GET',
      `/api/v1/library/tracks?provider=${encodeURIComponent(provider)}`,
      undefined
    );

    if (!result.success) {
      throw new Error(result.message || `Failed to load ${provider} library stats`);
    }

    const payload = result.data;
    if (Array.isArray(payload)) {
      return payload as ImportedLibraryTrack[];
    }

    if (Array.isArray((payload as any)?.tracks)) {
      return (payload as any).tracks as ImportedLibraryTrack[];
    }

    return [];
  }

  async function fetchLibraryStats(
    platform: Platform,
    provider: string
  ): Promise<ProviderLibraryStatsRow> {
    const result = await apiClient.authenticatedRequest<any>(
      'GET',
      `/api/v1/library/stats?provider=${encodeURIComponent(provider)}`,
      undefined
    );

    if (result.success && result.data) {
      const stats = result.data;
      return {
        provider,
        providerName: platform.name,
        songs: stats.songs ?? 0,
        albums: stats.albums ?? 0,
        artists: stats.artists ?? 0,
        playlists: stats.playlists ?? 0,
        totalItems: stats.totalItems ?? 0,
        lastSynced: stats.lastSynced ?? 'Never',
        source: 'imported_cache',
        status: stats.totalItems > 0 ? 'ready' : 'not_synced',
        message: stats.totalItems > 0
          ? undefined
          : 'No library data synced yet. Click "Sync Library" to import your favorites and playlists.',
      };
    }

    // Fallback: fetch tracks the old way
    const importedTracks = await fetchImportedLibraryTracks(provider);
    return summarizeImportedLibrary(platform, importedTracks);
  }

  function summarizeImportedLibrary(
    platform: Platform,
    tracks: ImportedLibraryTrack[]
  ): ProviderLibraryStatsRow {
    let songs = 0;
    let albums = 0;
    let artists = 0;
    let playlistEntries = 0;
    const playlistNames = new Set<string>();

    for (const item of tracks) {
      const sourceType = (item.source_type ?? '').toLowerCase();
      if (item.playlist_name) playlistNames.add(item.playlist_name);

      if (
        sourceType === 'favorite_track' ||
        sourceType === 'liked' ||
        sourceType === 'playlist_track' ||
        sourceType === 'liked_video' ||
        sourceType === 'playlist_item' ||
        sourceType === 'library_song'
      ) {
        songs += 1;
      } else if (
        sourceType === 'favorite_album' ||
        sourceType === 'saved_album' ||
        sourceType === 'library_album'
      ) {
        albums += 1;
      } else if (
        sourceType === 'favorite_artist' ||
        sourceType === 'followed_artist' ||
        sourceType === 'subscription'
      ) {
        artists += 1;
      } else if (!sourceType) {
        songs += 1;
      }

      if (sourceType.includes('playlist') || sourceType === 'library_playlist') {
        playlistEntries += 1;
      }
    }

    const playlists = playlistNames.size > 0 ? playlistNames.size : playlistEntries;
    const lastSynced = getLatestTimestamp(tracks.map(track => track.last_synced || track.added_at));
    const hasImportedData = tracks.length > 0;

    return {
      provider: platform.connectionProvider || platform.id,
      providerName: platform.name,
      songs,
      albums,
      artists,
      playlists,
      totalItems: tracks.length,
      lastSynced,
      source: 'imported_cache',
      status: hasImportedData ? 'ready' : 'not_synced',
      message: hasImportedData ? undefined : 'No library data synced yet. Click "Sync Library" to import your favorites and playlists.',
    };
  }

  async function buildLibraryStatsRow(platform: Platform): Promise<ProviderLibraryStatsRow> {
    try {
      const provider = platform.connectionProvider || platform.id;
      const row = await fetchLibraryStats(platform, provider);

      // Overlay sync status if a provider-specific sync is running or has failed
      const activeSyncStatus = getProviderSyncStatusForPlatform(platform.id);
      if (activeSyncStatus?.state === 'running') {
        row.status = 'syncing';
        row.message = activeSyncStatus.message || 'Syncing...';
      } else if (activeSyncStatus?.state === 'failed' && row.status === 'not_synced') {
        row.status = 'error';
        row.message = activeSyncStatus.error_message || activeSyncStatus.message || 'Sync failed';
      }

      return row;
    } catch (error) {
      const message =
        error instanceof Error ? error.message : `Failed to load ${platform.name} library stats`;

      return {
        provider: platform.connectionProvider || platform.id,
        providerName: platform.name,
        songs: null,
        albums: null,
        artists: null,
        playlists: null,
        totalItems: null,
        source: 'imported_cache',
        status: 'error',
        message,
      };
    }
  }

  function hasAppleImportedCache(): boolean {
    const row = libraryStatsByProvider.get('apple_music');
    return Boolean(row && row.source === 'imported_cache' && (row.totalItems ?? 0) > 0);
  }

  function isAppleImportPendingForLibraryView(): boolean {
    const providerFilter = normalizeLibraryParam(libraryItemsProviderFilter);
    const providerMatches = providerFilter === null || providerFilter === 'apple_music';
    return providerMatches && appleLibrarySyncStatus?.state === 'running' && !hasAppleImportedCache();
  }

  async function refreshAppleLibrarySyncStatus(): Promise<AppleLibrarySyncStatus | null> {
    const result = await apiClient.authenticatedRequest<AppleLibrarySyncStatus>(
      'GET',
      '/api/v1/apple-music/library/sync-status',
      undefined
    );

    if (result.success && result.data) {
      appleLibrarySyncStatus = result.data;
      return result.data;
    }

    appleLibrarySyncStatus = null;
    return null;
  }

  async function waitForAppleLibraryImport(
    maxWaitMs = 1_200_000
  ): Promise<AppleLibrarySyncStatus | null> {
    if (appleLibrarySyncPolling) {
      return appleLibrarySyncStatus;
    }

    appleLibrarySyncPolling = true;
    const deadline = Date.now() + maxWaitMs;

    try {
      while (Date.now() < deadline) {
        const status = await refreshAppleLibrarySyncStatus();
        await refreshLibraryStats();

        if (status?.state === 'failed') {
          throw new Error(status.message || 'Apple Music library sync failed');
        }

        if ((status?.state === 'completed' && status.imported_items_count !== undefined) || hasAppleImportedCache()) {
          return status;
        }

        await new Promise((resolve) => setTimeout(resolve, 4000));
      }

      return appleLibrarySyncStatus;
    } finally {
      appleLibrarySyncPolling = false;
    }
  }

  async function refreshLibraryStats() {
    libraryStatsLoading = true;
    libraryStatsError = null;

    try {
      if (connectedPlatforms.length === 0) {
        libraryStatsRows = [];
        return;
      }

      const rows = await Promise.all(connectedPlatforms.map(platform => buildLibraryStatsRow(platform)));
      const providerOrder = new Map(
        platforms.map((platform, index) => [platform.connectionProvider || platform.id, index])
      );

      libraryStatsRows = rows.sort((a, b) => {
        const aOrder = providerOrder.get(a.provider) ?? 999;
        const bOrder = providerOrder.get(b.provider) ?? 999;
        return aOrder - bOrder;
      });

      if (rows.every(row => row.status === 'error')) {
        libraryStatsError = 'Could not fetch library stats from connected services.';
      }
    } catch (error) {
      libraryStatsRows = [];
      libraryStatsError =
        error instanceof Error ? error.message : 'Failed to refresh library stats';
    } finally {
      libraryStatsLoading = false;
    }
  }

  function clearConnectionBannerTimer(): void {
    if (connectionBannerTimeout) {
      clearTimeout(connectionBannerTimeout);
      connectionBannerTimeout = null;
    }
  }

  function showConnectionSuccess(message: string, duration = 5000): void {
    clearConnectionBannerTimer();
    connectionError = null;
    connectionSuccess = message;
    if (duration > 0) {
      connectionBannerTimeout = setTimeout(() => {
        connectionSuccess = null;
        connectionBannerTimeout = null;
      }, duration);
    }
  }

  function showConnectionError(message: string): void {
    clearConnectionBannerTimer();
    connectionSuccess = null;
    connectionError = message;
  }

  // Sync library from a platform
  async function syncLibrary(platform: Platform) {
    if (!platform.connectionProvider) return;
    if (!hasActiveConnection(platform.connectionProvider)) return;

    // Check scan limit before syncing
    const scanAccess = await billingActions.checkFeature('scans');
    if (!scanAccess.allowed) {
      showConnectionError(scanAccess.reason || 'Scan limit reached. Upgrade your plan in Settings.');
      return;
    }

    connectionError = null;
    connectionSuccess = null;
    let shouldQueueGenericSyncRefresh = false;
    const providerSyncPlatform = getProviderSyncPlatform(platform.id);

    try {
      const triggerGenericSync = async () => {
        const request: TriggerSyncRequest = {
          platforms: [platform.id],
          sync_type: 'incremental',
          priority: 'normal',
        };
        await syncActions.triggerSync(request);
        showConnectionSuccess(
          `${platform.name} sync started. Provider-specific sync endpoint unavailable, using catalog pipeline.`
        );
        shouldQueueGenericSyncRefresh = true;
      };

      // Use platform-specific library sync endpoints
      if (platform.id === 'apple') {
        const inProgressToastId = blockingStore.addToast({
          type: 'info',
          message: 'Starting Apple Music library import...',
          dismissible: false,
        });

        try {
          const syncResult = await apiClient.authenticatedRequest<any>(
            'POST',
            '/api/v1/apple-music/library/sync',
            undefined
          );

          blockingStore.removeToast(inProgressToastId);

          if (syncResult.success) {
            const payload = (syncResult.data ?? syncResult) as any;
            const msg =
              payload?.message ||
              'Apple Music library sync started. Refresh will update once cached rows are ready.';
            blockingStore.addToast({
              type: 'info',
              message: msg,
              dismissible: true,
              duration: 6000,
            });
            await refreshAppleLibrarySyncStatus();

            const completedStatus = await waitForAppleLibraryImport();
            if (completedStatus?.state === 'completed') {
              showConnectionSuccess(
                completedStatus.message || 'Apple Music library imported successfully.'
              );
            }
          } else {
            const errMsg = syncResult.message || 'Failed to sync Apple Music library';
            blockingStore.addToast({
              type: 'error',
              message: errMsg,
              dismissible: true,
              duration: 8000,
            });
            connectionError = errMsg;
          }
        } catch (innerError) {
          blockingStore.removeToast(inProgressToastId);
          throw innerError; // re-throw to outer catch
        }
      } else if (platform.id === 'spotify' || platform.id === 'youtube' || platform.id === 'tidal') {
        const endpoint =
          platform.id === 'spotify'
            ? '/api/v1/connections/spotify/library/sync'
            : platform.id === 'youtube'
              ? '/api/v1/connections/youtube/library/sync'
              : '/api/v1/connections/tidal/library/sync';

        const syncResult = await apiClient.authenticatedRequest<any>(
          'POST',
          endpoint,
          undefined
        );

        if (syncResult.success) {
          const payload = (syncResult.data ?? syncResult) as any;
          showConnectionSuccess(
            payload?.message || `${platform.name} library sync started. Check status for progress.`
          );
        } else if (
          syncResult.error_code === 'HTTP_404' ||
          syncResult.error_code === 'HTTP_405'
        ) {
          await triggerGenericSync();
        } else if (syncResult.error_code === 'HTTP_502' || syncResult.error_code === 'HTTP_504') {
          showConnectionError(
            `${platform.name} sync timed out. The sync may still be running — check back in a minute.`
          );
        } else if (syncResult.error_code === 'HTTP_429') {
          showConnectionError(
            `${platform.name} is rate-limiting requests. Please wait a few minutes and try again.`
          );
        } else {
          showConnectionError(
            syncResult.message || `Failed to sync ${platform.name} library`
          );
        }
      } else {
        // Fallback to generic catalog sync for other platforms
        await triggerGenericSync();
      }
    } catch (error) {
      const message =
        error instanceof Error ? error.message : `Failed to sync library from ${platform.name}`;
      console.error(`Failed to sync library from ${platform.name}:`, error);
      showConnectionError(message);
      if (platform.id === 'apple') {
        blockingStore.addToast({
          type: 'error',
          message,
          dismissible: true,
          duration: 8000,
        });
      }
    } finally {
      // Refresh connection statuses (sync endpoints may mark needs_reauth).
      await connectionActions.fetchConnections();
      // Refresh status after sync
      await syncActions.fetchStatus();
      if (platform.id !== 'apple' && providerSyncPlatform) {
        const providerStatus = await refreshProviderLibrarySyncStatus(providerSyncPlatform);

        if (providerStatus?.state === 'running') {
          queueProviderSyncPolling([providerSyncPlatform], true);
        } else if (providerStatus?.state === 'completed') {
          await refreshHeavyLibraryViews({ force: true });
        } else if (providerStatus?.state === 'failed') {
          showConnectionError(
            providerStatus.message || `Failed to sync ${platform.name} library`
          );
        } else {
          await refreshHeavyLibraryViews({ force: true });
        }
      } else if (platform.id !== 'apple' && shouldQueueGenericSyncRefresh) {
        queueGenericSyncPolling();
      }
    }
  }

  onMount(async () => {
    await Promise.all([
      refreshAppleLibrarySyncStatus(),
      syncActions.fetchStatus(),
      syncActions.fetchRuns(),
      syncActions.fetchHealth(),
      connectionActions.fetchConnections(),
    ]);
    await refreshConnectedProviderSyncStatuses({ queueRunning: true });
    await refreshHeavyLibraryViews();
  });

  onDestroy(() => {
    clearGenericSyncPollTimer();
    clearProviderSyncPollTimer();
    if (libraryQueryDebounce) {
      clearTimeout(libraryQueryDebounce);
    }
  });

  const STATUS_COLORS: Record<string, string> = {
    running: 'bg-rose-500/10 text-rose-400',
    completed: 'bg-green-500/20 text-green-400',
    error: 'bg-red-500/20 text-red-400',
    failed: 'bg-red-500/20 text-red-400',
    cancelled: 'bg-zinc-500/20 text-zinc-300',
  };
  const STATUS_ICONS: Record<string, string> = {
    running: '\u21BB',
    completed: '\u2713',
    error: '\u2717',
    failed: '\u2717',
    cancelled: '\u25A0',
  };
  const DEFAULT_STATUS_COLOR = 'bg-zinc-500/20 text-zinc-300';
  const DEFAULT_STATUS_ICON = '\u2026';

  function formatDuration(ms?: number): string {
    if (!ms) return '-';
    if (ms < 1000) return `${ms}ms`;
    if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
    return `${(ms / 60000).toFixed(1)}m`;
  }

  function formatMetric(value: number | null): string {
    if (value === null) return '--';
    return value.toLocaleString();
  }

  function formatCurrency(value: string | null | undefined): string {
    if (!value) return '--';
    const num = Number.parseFloat(value);
    if (!Number.isFinite(num)) return '--';
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 2,
      maximumFractionDigits: 4,
    }).format(num);
  }

  async function handleSyncAll() {
    const selectedPlatforms = connectedPlatforms
      .filter(
        (platform) =>
          Boolean(platform.connectionProvider) &&
          hasActiveConnection(platform.connectionProvider) &&
          !platform.disabled
      )
      .map((platform) => platform.id);

    if (selectedPlatforms.length === 0) return;

    const directSyncPlatforms = selectedPlatforms
      .map((platformId) => getPlatformById(platformId))
      .filter((platform): platform is Platform => Boolean(platform))
      .filter(
        (platform) =>
          Boolean(platform.connectionProvider) &&
          hasActiveConnection(platform.connectionProvider) &&
          ['apple', 'spotify', 'youtube', 'tidal'].includes(platform.id)
      );

    const directSyncPlatformIds = new Set(directSyncPlatforms.map((platform) => platform.id));
    const pipelinePlatforms = selectedPlatforms.filter(
      (platformId) => !directSyncPlatformIds.has(platformId)
    );

    if (directSyncPlatforms.length > 0) {
      for (const platform of directSyncPlatforms) {
        await syncLibrary(platform);
      }
    }

    if (pipelinePlatforms.length > 0) {
      const request: TriggerSyncRequest = {
        platforms: pipelinePlatforms,
        sync_type: 'incremental',
        priority: 'normal',
      };

      const result = await syncActions.triggerSync(request);
      if (!result.success && result.message) {
        showConnectionError(result.message);
      } else if (result.success) {
        queueGenericSyncPolling();
      }
    }
  }

  $: healthStatus = $syncStore.health?.overall_status ?? 'unknown';
  $: healthColor = healthStatus === 'healthy' ? 'text-green-600' :
                   healthStatus === 'degraded' ? 'text-yellow-600' : 'text-red-600';

  function normalizeLibraryParam(value: string): string | null {
    const trimmed = value.trim();
    if (!trimmed || trimmed === 'all') return null;
    return trimmed;
  }

  function buildLibraryItemsEndpoint(offset: number, limit: number): string {
    const params = new URLSearchParams();
    const provider = normalizeLibraryParam(libraryItemsProviderFilter);
    if (provider) params.set('provider', provider);
    if (libraryItemsKindFilter !== 'all') params.set('kind', libraryItemsKindFilter);
    if (libraryItemsQuery.trim()) params.set('q', libraryItemsQuery.trim());
    if (libraryFilterArtist.trim()) params.set('artist', libraryFilterArtist.trim());
    if (libraryFilterAlbum.trim()) params.set('album', libraryFilterAlbum.trim());
    if (libraryFilterPlaylist.trim()) params.set('playlist', libraryFilterPlaylist.trim());
    params.set('sort', librarySort);
    params.set('dir', librarySortDir);
    params.set('limit', String(limit));
    params.set('offset', String(offset));
    return `/api/v1/library/items?${params.toString()}`;
  }

  function buildLibraryGroupsEndpoint(offset: number, limit: number): string {
    const params = new URLSearchParams();
    params.set('group_by', libraryGroupBy);
    const provider = normalizeLibraryParam(libraryItemsProviderFilter);
    if (provider) params.set('provider', provider);
    if (libraryItemsKindFilter !== 'all') params.set('kind', libraryItemsKindFilter);
    if (libraryItemsQuery.trim()) params.set('q', libraryItemsQuery.trim());
    params.set('sort', libraryGroupSort);
    params.set('dir', libraryGroupSortDir);
    params.set('limit', String(limit));
    params.set('offset', String(offset));
    return `/api/v1/library/groups?${params.toString()}`;
  }

  async function refreshLibraryItems(options: { reset?: boolean; append?: boolean } = {}) {
    const reset = options.reset === true;
    const append = options.append === true && !reset;
    const requestOffset = append ? libraryItemsOffset : 0;

    libraryItemsLoading = true;
    libraryItemsError = null;

    if (!append) {
      libraryItemsOffset = 0;
      libraryItems = [];
    }

    if (isAppleImportPendingForLibraryView()) {
      libraryItemsError =
        'Apple Music library import is still running. The cached library list will populate automatically when the import completes.';
      libraryItemsLoading = false;
      libraryItemsTotal = 0;
      return;
    }

    try {
      const endpoint = buildLibraryItemsEndpoint(requestOffset, libraryItemsLimit);
      const result = await apiClient.authenticatedRequest<LibraryItemsPage>(
        'GET',
        endpoint,
        undefined
      );

      if (!result.success || !result.data) {
        libraryItemsError = result.message || 'Failed to load library items';
        libraryItems = append ? libraryItems : [];
        libraryItemsTotal = 0;
        return;
      }

      const page = result.data as any;
      const items: ImportedLibraryTrack[] = Array.isArray(page?.items) ? page.items : [];
      const total = Number(page?.total ?? items.length);

      libraryItemsTotal = Number.isFinite(total) ? total : items.length;
      libraryItemsOffset = requestOffset + items.length;
      libraryItems = append ? [...libraryItems, ...items] : items;
    } catch (error) {
      libraryItemsError = error instanceof Error ? error.message : 'Failed to load library items';
      libraryItems = append ? libraryItems : [];
      libraryItemsTotal = 0;
    } finally {
      libraryItemsLoading = false;
    }
  }

  async function refreshLibraryGroups(options: { reset?: boolean; append?: boolean } = {}) {
    const reset = options.reset === true;
    const append = options.append === true && !reset;
    const requestOffset = append ? libraryGroupsOffset : 0;

    libraryGroupsLoading = true;
    libraryGroupsError = null;

    if (!append) {
      libraryGroupsOffset = 0;
      libraryGroups = [];
    }

    if (isAppleImportPendingForLibraryView()) {
      libraryGroupsError =
        'Apple Music library import is still running. Grouped library views will populate when cached rows are ready.';
      libraryGroupsLoading = false;
      libraryGroupsTotal = 0;
      return;
    }

    try {
      const endpoint = buildLibraryGroupsEndpoint(requestOffset, libraryGroupsLimit);
      const result = await apiClient.authenticatedRequest<LibraryGroupsPage>(
        'GET',
        endpoint,
        undefined
      );

      if (!result.success || !result.data) {
        libraryGroupsError = result.message || 'Failed to load library groups';
        libraryGroups = append ? libraryGroups : [];
        libraryGroupsTotal = 0;
        return;
      }

      const page = result.data as any;
      const groups: LibraryGroup[] = Array.isArray(page?.groups) ? page.groups : [];
      const total = Number(page?.total ?? groups.length);

      libraryGroupsTotal = Number.isFinite(total) ? total : groups.length;
      libraryGroupsOffset = requestOffset + groups.length;
      libraryGroups = append ? [...libraryGroups, ...groups] : groups;
    } catch (error) {
      libraryGroupsError = error instanceof Error ? error.message : 'Failed to load library groups';
      libraryGroups = append ? libraryGroups : [];
      libraryGroupsTotal = 0;
    } finally {
      libraryGroupsLoading = false;
    }
  }

  async function refreshLibraryExplorer(reset = true) {
    if (libraryBrowseView === 'grouped') {
      await refreshLibraryGroups({ reset });
      return;
    }

    await refreshLibraryItems({ reset });
  }

  function scheduleLibraryExplorerRefresh() {
    if (libraryQueryDebounce) clearTimeout(libraryQueryDebounce);
    libraryQueryDebounce = setTimeout(() => {
      void refreshLibraryExplorer(true);
    }, 250);
  }

  function clearLibraryEntityFilters() {
    libraryFilterArtist = '';
    libraryFilterAlbum = '';
    libraryFilterPlaylist = '';
  }

  function applyLibraryGroupFilter(group: LibraryGroup) {
    clearLibraryEntityFilters();

    if (libraryGroupBy === 'artist') {
      libraryFilterArtist = group.value;
    } else if (libraryGroupBy === 'album') {
      libraryFilterAlbum = group.value;
      if (group.secondary) libraryFilterArtist = group.secondary;
    } else if (libraryGroupBy === 'playlist') {
      libraryFilterPlaylist = group.value;
      if (group.provider) libraryItemsProviderFilter = group.provider;
    } else if (libraryGroupBy === 'provider') {
      libraryItemsProviderFilter = group.value;
    } else if (libraryGroupBy === 'kind') {
      if (['songs', 'albums', 'artists', 'playlists'].includes(group.value)) {
        libraryItemsKindFilter = group.value as LibraryItemKind;
      }
    }

    libraryBrowseView = 'items';
    void refreshLibraryItems({ reset: true });
  }

  async function refreshTasteGrade() {
    tasteGradeLoading = true;
    tasteGradeError = null;

    if (appleLibrarySyncStatus?.state === 'running' && !hasAppleImportedCache()) {
      tasteGrade = null;
      tasteGradeError =
        'Apple Music library import is still running. Taste grade will populate when cached rows are ready.';
      tasteGradeLoading = false;
      return;
    }

    try {
      const result = await apiClient.authenticatedRequest<TasteGradeResponse>(
        'GET',
        '/api/v1/library/taste-grade',
        undefined
      );

      if (result.success && result.data) {
        tasteGrade = result.data;
      } else {
        tasteGrade = null;
        tasteGradeError = result.message || 'Failed to load taste grade';
      }
    } catch (error) {
      tasteGrade = null;
      tasteGradeError = error instanceof Error ? error.message : 'Failed to load taste grade';
    } finally {
      tasteGradeLoading = false;
    }
  }

  async function refreshLibraryOffenders() {
    libraryOffendersLoading = true;
    libraryOffendersError = null;

    if (appleLibrarySyncStatus?.state === 'running' && !hasAppleImportedCache()) {
      libraryOffenders = null;
      libraryOffendersError =
        'Apple Music library import is still running. Offender analysis will populate when cached rows are ready.';
      libraryOffendersLoading = false;
      return;
    }

    try {
      const params = new URLSearchParams();
      params.set('limit', '5');
      if (libraryOffendersScope === 'all') params.set('kind', 'all');
      params.set('days', String(libraryOffendersDays));

      const result = await apiClient.authenticatedRequest<LibraryOffendersResponse>(
        'GET',
        `/api/v1/library/offenders?${params.toString()}`,
        undefined
      );

      if (result.success && result.data) {
        libraryOffenders = result.data;
        libraryOffendersHasPlaycounts ||= Boolean(result.data.playcounts_available);
      } else {
        libraryOffenders = null;
        libraryOffendersError = result.message || 'Failed to load worst offenders';
      }
    } catch (error) {
      libraryOffenders = null;
      libraryOffendersError = error instanceof Error ? error.message : 'Failed to load worst offenders';
    } finally {
      libraryOffendersLoading = false;
    }
  }

</script>

<div class="brand-page surface-page sync-dashboard-page">
  <div class="brand-page__inner brand-page__stack">
    <section class="brand-hero">
      <div class="brand-hero__header">
        <div class="brand-hero__copy">
          <button
            type="button"
            on:click={() => navigateTo('home')}
            class="brand-back"
          >
            <svg fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
            </svg>
            Back to Home
          </button>
          <div class="brand-kickers">
            <span class="brand-kicker">Library Control</span>
            <span class="brand-kicker brand-kicker--accent">Cross-platform Sync</span>
          </div>
          <h1 class="brand-title brand-title--compact">Keep your library in step with your policy.</h1>
          <p class="brand-subtitle">
            Connect streaming services, monitor sync health, and push changes without losing visibility into what is actually imported.
          </p>
        </div>

        <div class="brand-hero__aside">
          <div class="brand-stat-grid brand-stat-grid--compact" aria-label="Sync overview">
            <div class="brand-stat">
              <span class="brand-stat__value">{connectedPlatforms.length}</span>
              <span class="brand-stat__label">Connected services</span>
            </div>
            <div class="brand-stat">
              <span class="brand-stat__value">{libraryStatsRows.length}</span>
              <span class="brand-stat__label">Tracked libraries</span>
            </div>
          </div>

          <div class="brand-actions">
            <button
              type="button"
              on:click={handleSyncAll}
              disabled={$isAnySyncRunning || $syncStore.isTriggering}
              class="brand-button brand-button--primary"
            >
              {#if $syncStore.isTriggering}
                <span class="brand-button__spinner"></span>
                Starting...
              {:else}
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/><path d="M3.51 9a9 9 0 0114.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0020.49 15"/></svg>
                Sync All
              {/if}
            </button>
          </div>
        </div>
      </div>
    </section>

    <div>
    <!-- Health Status -->
    {#if $syncStore.health}
      <div class="surface-card rounded-xl p-4 mb-6" >
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <span class="text-2xl">
              {healthStatus === 'healthy' ? '\u2713' : healthStatus === 'degraded' ? '\u26A0' : '\u2717'}
            </span>
            <div>
              <span class="font-medium text-white">Overall Health:</span>
              <span class="{healthColor} font-semibold ml-2 capitalize">{healthStatus}</span>
            </div>
          </div>
          <div class="flex items-center gap-4 text-sm text-zinc-400">
            {#each ($syncStore.health?.platforms || []) as platform}
              <div class="flex items-center gap-1">
                <span class={platform?.is_healthy ? 'text-green-500' : 'text-red-500'}>
                  {platform?.is_healthy ? '●' : '○'}
                </span>
                <span class="capitalize">{platform?.platform || 'unknown'}</span>
              </div>
            {/each}
          </div>
        </div>
      </div>
    {/if}

    <!-- Error display -->
    {#if $syncStore.error}
      <div class="brand-alert brand-alert--error mb-6">
        <div class="flex items-center gap-2">
          <span aria-hidden="true">✕</span>
          <span>{$syncStore.error}</span>
          <button type="button" on:click={syncActions.clearError} class="brand-alert__dismiss">
            Dismiss
          </button>
        </div>
      </div>
    {/if}

    {#if connectionError}
      <div class="brand-alert brand-alert--error mb-6">
        <div class="flex items-center gap-2">
          <span aria-hidden="true">✕</span>
          <span>{connectionError}</span>
          <button type="button" on:click={() => connectionError = null} class="brand-alert__dismiss">
            Dismiss
          </button>
        </div>
      </div>
    {/if}

    <!-- Connection success notification -->
    {#if connectionSuccess}
      <div class="brand-alert brand-alert--success mb-6">
        <div class="flex items-center gap-2">
          <svg class="w-5 h-5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
          <span>{connectionSuccess}</span>
          <button type="button" on:click={() => connectionSuccess = null} class="brand-alert__dismiss">
            Dismiss
          </button>
        </div>
      </div>
    {/if}

    <!-- Service Connector Grid -->
    <div class="mb-8">
      <h2 class="text-xl font-semibold text-white mb-4">Connect Your Services</h2>
      <ServiceConnector />
    </div>


    <!-- Connected library stats -->
    <div class="mb-8">
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-xl font-semibold text-white">Connected Library Stats</h2>
        <button
          type="button"
          on:click={refreshLibraryStats}
          disabled={libraryStatsLoading || $connectionsStore.isLoading}
          class="brand-button brand-button--secondary brand-button--compact"
        >
          <svg class="w-4 h-4 inline" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="23 4 23 10 17 10"/>
            <polyline points="1 20 1 14 7 14"/>
            <path d="M3.51 9a9 9 0 0114.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0020.49 15"/>
          </svg>
          {libraryStatsLoading ? 'Refreshing...' : 'Refresh Stats'}
        </button>
      </div>

      {#if libraryStatsLoading}
        <div class="bg-zinc-900 rounded-xl border border-zinc-700 p-8 text-center">
          <div class="w-8 h-8 border-4 border-indigo-500 border-t-transparent rounded-full animate-spin mx-auto"></div>
          <p class="text-zinc-400 mt-3">Collecting stats from connected services...</p>
        </div>
      {:else if libraryStatsRows.length === 0}
        <div class="bg-zinc-900 rounded-xl border border-zinc-700 p-8 text-center">
          <p class="text-zinc-400">Connect and sync a streaming service to populate library stats.</p>
        </div>
      {:else}
        <div class="bg-zinc-900 rounded-xl shadow-sm overflow-hidden border border-zinc-700">
          <table class="w-full">
            <thead class="bg-zinc-700 border-b border-zinc-700">
              <tr>
                <th class="px-4 py-3 text-left text-xs font-medium text-zinc-400 uppercase">Provider</th>
                <th class="px-4 py-3 text-left text-xs font-medium text-zinc-400 uppercase">Songs</th>
                <th class="px-4 py-3 text-left text-xs font-medium text-zinc-400 uppercase">Albums</th>
                <th class="px-4 py-3 text-left text-xs font-medium text-zinc-400 uppercase">Artists</th>
                <th class="px-4 py-3 text-left text-xs font-medium text-zinc-400 uppercase">Playlists</th>
                <th class="px-4 py-3 text-left text-xs font-medium text-zinc-400 uppercase">Total</th>
                <th class="px-4 py-3 text-left text-xs font-medium text-zinc-400 uppercase">Last Updated</th>
                <th class="px-4 py-3 text-left text-xs font-medium text-zinc-400 uppercase">Source</th>
                <th class="px-4 py-3 text-left text-xs font-medium text-zinc-400 uppercase">Status</th>
              </tr>
            </thead>
            <tbody class="border-t border-zinc-700">
              {#each libraryStatsRows as row}
                <tr class="hover:bg-zinc-800">
                  <td class="px-4 py-3 font-medium text-white">
                    <span class="inline-flex items-center gap-2">
                      <ProviderIcon provider={row.provider} size={16} />
                      {row.providerName}
                    </span>
                  </td>
                  <td class="px-4 py-3 text-zinc-300">{formatMetric(row.songs)}</td>
                  <td class="px-4 py-3 text-zinc-300">{formatMetric(row.albums)}</td>
                  <td class="px-4 py-3 text-zinc-300">{formatMetric(row.artists)}</td>
                  <td class="px-4 py-3 text-zinc-300">{formatMetric(row.playlists)}</td>
                  <td class="px-4 py-3 text-zinc-300">{formatMetric(row.totalItems)}</td>
                  <td class="px-4 py-3 text-zinc-300">{timeAgo(row.lastSynced)}</td>
                  <td class="px-4 py-3 text-zinc-300">{row.source === 'live_api' ? 'Live API' : 'Imported Cache'}</td>
                  <td class="px-4 py-3">
                    {#if row.status === 'ready'}
                      <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium bg-green-500/20 text-green-400">
                        ✓ Ready
                      </span>
                    {:else if row.status === 'syncing'}
                      <div>
                        <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium bg-indigo-500/20 text-indigo-400">
                          <span class="w-2 h-2 rounded-full bg-indigo-400 animate-pulse"></span>
                          Syncing
                        </span>
                        {#if row.message}
                          <div class="text-xs text-indigo-300 mt-1">{row.message}</div>
                        {/if}
                      </div>
                    {:else if row.status === 'not_synced'}
                      <div>
                        <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium bg-zinc-500/20 text-zinc-400">
                          Not synced
                        </span>
                        {#if row.message}
                          <div class="text-xs text-zinc-400 mt-1">{row.message}</div>
                        {/if}
                      </div>
                    {:else}
                      <div>
                        <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium bg-red-500/20 text-red-400">
                          ✗ Failed
                        </span>
                        {#if row.message}
                          <div class="text-xs text-red-400 mt-1">{row.message}</div>
                        {/if}
                      </div>
                    {/if}
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}

      {#if libraryStatsError}
        <p class="text-xs text-red-400 mt-2">{libraryStatsError}</p>
      {/if}
    </div>

    <!-- Taste grade -->
    <div class="mb-8">
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-xl font-semibold text-white">Taste Grade</h2>
        <button
          type="button"
          on:click={refreshTasteGrade}
          disabled={tasteGradeLoading || $connectionsStore.isLoading}
          class="brand-button brand-button--secondary brand-button--compact"
        >
          <svg class="w-4 h-4 inline" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="23 4 23 10 17 10"/>
            <polyline points="1 20 1 14 7 14"/>
            <path d="M3.51 9a9 9 0 0114.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0020.49 15"/>
          </svg>
          {tasteGradeLoading ? 'Refreshing...' : 'Refresh Grade'}
        </button>
      </div>

      {#if tasteGradeLoading}
        <div class="bg-zinc-900 rounded-xl border border-zinc-700 p-8 text-center">
          <div class="w-8 h-8 border-4 border-indigo-500 border-t-transparent rounded-full animate-spin mx-auto"></div>
          <p class="text-zinc-400 mt-3">Grading your library...</p>
        </div>
      {:else if tasteGradeError}
        <div class="bg-red-500/10 border border-red-500/20 rounded-xl p-4 text-sm text-red-300">
          {tasteGradeError}
        </div>
      {:else if !tasteGrade}
        <div class="bg-zinc-900 rounded-xl border border-zinc-700 p-8 text-center">
          <p class="text-zinc-400">Sync a library to generate a taste grade.</p>
        </div>
      {:else}
        {@const overallGrade = tasteGrade.overall_grade ?? '?'}
        {@const overallColor =
          overallGrade.startsWith('A') ? 'border-green-500/40 bg-green-500/10 text-green-300' :
          overallGrade.startsWith('B') ? 'border-lime-500/40 bg-lime-500/10 text-lime-300' :
          overallGrade.startsWith('C') ? 'border-amber-500/40 bg-amber-500/10 text-amber-300' :
          overallGrade.startsWith('D') ? 'border-orange-500/40 bg-orange-500/10 text-orange-300' :
          'border-red-500/40 bg-red-500/10 text-red-300'
        }

        <div class="bg-zinc-900 rounded-xl border border-zinc-700 p-6">
          <div class="flex flex-col md:flex-row gap-6 md:items-center">
            <div class="flex items-center justify-center w-28 h-28 rounded-full border-2 {overallColor}">
              <div class="text-center">
                <div class="text-3xl font-extrabold leading-none">{overallGrade}</div>
                <div class="text-xs opacity-80 mt-1">{(tasteGrade.overall_score ?? 0).toFixed(2)}</div>
              </div>
            </div>

            <div class="flex-1">
              <div class="flex items-center justify-between gap-4">
                <div>
                  <div class="text-sm text-zinc-400">Overall</div>
                  <div class="text-lg font-semibold text-white">Music Taste Score</div>
                </div>
                <div class="text-xs text-zinc-500">
                  Computed {timeAgo(tasteGrade.computed_at)}
                </div>
              </div>

              <div class="mt-4 bg-zinc-950/40 rounded-xl border border-zinc-700 overflow-hidden">
                <table class="w-full">
                  <thead class="bg-zinc-800/70 border-b border-zinc-700">
                    <tr class="text-left text-xs font-medium text-zinc-400 uppercase tracking-wide">
                      <th class="px-4 py-3">Category</th>
                      <th class="px-4 py-3">Grade</th>
                      <th class="px-4 py-3">Score</th>
                      <th class="px-4 py-3">Weight</th>
                      <th class="px-4 py-3">Signal</th>
                    </tr>
                  </thead>
                  <tbody class="border-t border-zinc-700">
                    {#each (tasteGrade.components ?? []) as component (component.id)}
                      <tr class="hover:bg-zinc-800/60">
                        <td class="px-4 py-3 text-white font-medium">{component.label}</td>
                        <td class="px-4 py-3 text-zinc-300">{component.grade}</td>
                        <td class="px-4 py-3 text-zinc-300">{(component.score ?? 0).toFixed(2)}</td>
                        <td class="px-4 py-3 text-zinc-400">{Math.round((component.weight ?? 0) * 100)}%</td>
                        <td class="px-4 py-3 text-zinc-400">{component.summary}</td>
                      </tr>
                    {/each}
                  </tbody>
                </table>
              </div>
            </div>
          </div>

          <div class="mt-6 grid grid-cols-1 md:grid-cols-2 gap-4">
            <div class="bg-zinc-950/40 rounded-xl border border-zinc-700 p-4">
              <div class="text-xs font-medium text-zinc-400 uppercase tracking-wide mb-2">Signals</div>
              {#if (tasteGrade.signals ?? []).length === 0}
                <div class="text-sm text-zinc-500">No signals yet.</div>
              {:else}
                <div class="space-y-1 text-sm text-zinc-300">
                  {#each (tasteGrade.signals ?? []) as s}
                    <div>• {s}</div>
                  {/each}
                </div>
              {/if}
            </div>

            <div class="bg-zinc-950/40 rounded-xl border border-zinc-700 p-4">
              <div class="text-xs font-medium text-zinc-400 uppercase tracking-wide mb-2">Recommendations</div>
              {#if (tasteGrade.recommendations ?? []).length === 0}
                <div class="text-sm text-zinc-500">Looking good. No recommendations right now.</div>
              {:else}
                <div class="space-y-1 text-sm text-zinc-300">
                  {#each (tasteGrade.recommendations ?? []) as r}
                    <div>• {r}</div>
                  {/each}
                </div>
              {/if}
            </div>
          </div>
        </div>
      {/if}
    </div>

    <!-- Worst offenders -->
    <div class="mb-8">
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-xl font-semibold text-white">Worst Offenders</h2>
        <div class="flex items-center gap-3">
          <select
            bind:value={libraryOffendersScope}
            on:change={() => void refreshLibraryOffenders()}
            class="px-3 py-1.5 rounded-lg bg-zinc-900 text-zinc-200 text-sm border border-zinc-700 hover:border-zinc-600"
          >
            <option value="songs">Songs</option>
            <option value="all">All items</option>
          </select>
          {#if libraryOffendersHasPlaycounts}
            <select
              bind:value={libraryOffendersDays}
              on:change={() => void refreshLibraryOffenders()}
              class="px-3 py-1.5 rounded-lg bg-zinc-900 text-zinc-200 text-sm border border-zinc-700 hover:border-zinc-600"
            >
              {#each libraryOffendersPeriodOptions as option}
                <option value={option.value}>{option.label}</option>
              {/each}
            </select>
          {/if}
          <button
            type="button"
            on:click={refreshLibraryOffenders}
            disabled={libraryOffendersLoading || $connectionsStore.isLoading}
            class="brand-button brand-button--secondary brand-button--compact"
          >
            <svg class="w-4 h-4 inline" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="23 4 23 10 17 10"/>
              <polyline points="1 20 1 14 7 14"/>
              <path d="M3.51 9a9 9 0 0114.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0020.49 15"/>
            </svg>
            {libraryOffendersLoading ? 'Refreshing...' : 'Refresh'}
          </button>
          <button
            type="button"
            on:click={() => navigateTo('library-scan')}
            class="brand-button brand-button--secondary brand-button--compact"
          >
            View Full Scan
          </button>
        </div>
      </div>

      {#if libraryOffendersLoading}
        <div class="bg-zinc-900 rounded-xl border border-zinc-700 p-8 text-center">
          <div class="w-8 h-8 border-4 border-indigo-500 border-t-transparent rounded-full animate-spin mx-auto"></div>
          <p class="text-zinc-400 mt-3">Finding offenders in your library...</p>
        </div>
      {:else if libraryOffendersError}
        <div class="bg-red-500/10 border border-red-500/20 rounded-xl p-4 text-sm text-red-300">
          {libraryOffendersError}
        </div>
      {:else if !libraryOffenders || (libraryOffenders.offenders ?? []).length === 0}
        <div class="bg-zinc-900 rounded-xl border border-zinc-700 p-8 text-center">
          <p class="text-zinc-400">No verified offenders detected in your library.</p>
          <p class="text-xs text-zinc-500 mt-2">This card only considers artists with verified incidents in our offense database.</p>
        </div>
      {:else}
        {@const unitLabel = libraryOffendersScope === 'songs' ? 'songs' : 'items'}
        {@const playWindowLabel = libraryOffenders.playcount_window_days === 0 ? 'All time' : `${libraryOffenders.playcount_window_days}d`}
        <div class="bg-zinc-900 rounded-xl border border-zinc-700 p-6">
          <div class="flex items-center justify-between gap-4 mb-4">
            <div class="text-sm text-zinc-400">
              Top {(libraryOffenders.offenders ?? []).length} of {(libraryOffenders.total_flagged_artists ?? 0).toLocaleString()} flagged artists
              ({(libraryOffenders.total_flagged_tracks ?? 0).toLocaleString()} {unitLabel} impacted)
            </div>
            <div class="text-xs text-zinc-500">
              Computed {timeAgo(libraryOffenders.computed_at)}
            </div>
          </div>

          {#if !libraryOffenders.playcounts_available}
            <div class="mb-4 bg-zinc-950/40 rounded-xl border border-zinc-700 p-3 text-xs text-zinc-400 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2">
              <div>
                Plays and payout estimates aren’t available yet for the selected period. Import listening history to enable play counts and estimated payout.
              </div>
              <button
                type="button"
                on:click={() => navigateTo('revenue-impact')}
                class="brand-button brand-button--secondary brand-button--compact"
              >
                Open Streaming Impact
              </button>
            </div>
          {/if}

          <div class="space-y-3">
            {#each (libraryOffenders.offenders ?? []) as offender (offender.id)}
              <div class="bg-zinc-950/40 rounded-xl border border-zinc-700 p-4">
                <div class="flex items-start justify-between gap-4">
                  <div class="min-w-0">
                    <button
                      type="button"
                      on:click={() => navigateToArtist(offender.id)}
                      class="text-white font-semibold hover:underline truncate"
                      title={offender.name}
                    >
                      {offender.name}
                    </button>
                    <div class="text-xs text-zinc-400 mt-1 flex flex-wrap gap-x-2 gap-y-1">
                      <span>{(offender.track_count ?? 0).toLocaleString()} {unitLabel} in your library</span>
                      {#if libraryOffenders.playcounts_available}
                        <span class="text-zinc-600">•</span>
                        <span>
                          {offender.play_count !== null ? offender.play_count.toLocaleString() : '--'} plays
                          <span class="text-zinc-500">({playWindowLabel})</span>
                        </span>
                        <span class="text-zinc-600">•</span>
                        <span>{formatCurrency(offender.estimated_revenue)} est. payout</span>
                        {#if offender.percentage_of_user_spend != null}
                          <span class="text-zinc-600">•</span>
                          <span>{(offender.percentage_of_user_spend ?? 0).toFixed(1)}% of your est. payout</span>
                        {/if}
                      {/if}
                    </div>
                  </div>

                  <span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium border {severityBadgeClass(offender.severity)}">
                    {formatSeverity(offender.severity)}
                  </span>
                </div>

                {#if (offender.offenses ?? []).length > 0}
                  <div class="mt-3 space-y-1 text-xs text-zinc-300">
                    {#each (offender.offenses ?? []) as o (o.title)}
                      <div class="flex gap-2">
                        <span class="text-zinc-500">•</span>
                        <div class="min-w-0">
                          <span class="text-zinc-400">{formatOffenseCategory(o.category)}</span>
                          <span class="text-zinc-500"> · </span>
                          <span class="text-zinc-200">{o.title}</span>
                          <span class="text-zinc-500"> ({o.date})</span>
                          {#if o.evidence_count > 0}
                            <span class="text-zinc-500"> · {o.evidence_count} sources</span>
                          {/if}
                        </div>
                      </div>
                    {/each}
                  </div>
                {:else}
                  <div class="mt-3 text-xs text-zinc-500">
                    Verified offenses exist, but summaries were unavailable.
                  </div>
                {/if}
              </div>
            {/each}
          </div>
        </div>
      {/if}
    </div>

    <!-- Personal library items -->
    <div class="mb-8">
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-xl font-semibold text-white">Your Library</h2>
        <button
          type="button"
          on:click={() => void refreshLibraryExplorer(true)}
          disabled={(libraryBrowseView === 'items' ? libraryItemsLoading : libraryGroupsLoading) || $connectionsStore.isLoading}
          class="brand-button brand-button--secondary brand-button--compact"
        >
          <svg class="w-4 h-4 inline" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="23 4 23 10 17 10"/>
            <polyline points="1 20 1 14 7 14"/>
            <path d="M3.51 9a9 9 0 0114.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0020.49 15"/>
          </svg>
          {libraryBrowseView === 'items'
            ? (libraryItemsLoading ? 'Refreshing...' : 'Refresh')
            : (libraryGroupsLoading ? 'Refreshing...' : 'Refresh')}
        </button>
      </div>

      <div class="bg-zinc-900 rounded-xl shadow-sm overflow-hidden border border-zinc-700 p-4">
        <div class="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between mb-4">
          <div class="flex flex-col gap-2 sm:flex-row sm:items-center sm:flex-wrap">
            <label class="text-xs font-medium text-zinc-400 uppercase tracking-wide">View</label>
            <select
              bind:value={libraryBrowseView}
              on:change={() => void refreshLibraryExplorer(true)}
              class="px-3 py-2 rounded-lg text-sm text-zinc-300 bg-zinc-800 border border-zinc-700 focus:outline-none focus:border-indigo-500"
            >
              <option value="items">Items</option>
              <option value="grouped">Grouped</option>
            </select>

            <label class="text-xs font-medium text-zinc-400 uppercase tracking-wide">Provider</label>
            <select
              bind:value={libraryItemsProviderFilter}
              on:change={() => void refreshLibraryExplorer(true)}
              class="px-3 py-2 rounded-lg text-sm text-zinc-300 bg-zinc-800 border border-zinc-700 focus:outline-none focus:border-indigo-500"
            >
              <option value="all">All connected</option>
              {#each connectedPlatforms as platform}
                {@const provider = platform.connectionProvider || platform.id}
                <option value={provider}>{platform.name}</option>
              {/each}
            </select>

            <label class="text-xs font-medium text-zinc-400 uppercase tracking-wide sm:ml-2">Type</label>
            <select
              bind:value={libraryItemsKindFilter}
              on:change={() => void refreshLibraryExplorer(true)}
              class="px-3 py-2 rounded-lg text-sm text-zinc-300 bg-zinc-800 border border-zinc-700 focus:outline-none focus:border-indigo-500"
            >
              <option value="songs">Songs</option>
              <option value="albums">Albums</option>
              <option value="artists">Artists</option>
              <option value="playlists">Playlists</option>
              <option value="all">Everything</option>
            </select>

            {#if libraryBrowseView === 'items'}
              <label class="text-xs font-medium text-zinc-400 uppercase tracking-wide sm:ml-2">Sort</label>
              <select
                bind:value={librarySort}
                on:change={() => void refreshLibraryItems({ reset: true })}
                class="px-3 py-2 rounded-lg text-sm text-zinc-300 bg-zinc-800 border border-zinc-700 focus:outline-none focus:border-indigo-500"
              >
                <option value="last_synced">Synced</option>
                <option value="added_at">Added</option>
                <option value="title">Title</option>
                <option value="artist">Artist</option>
                <option value="album">Album</option>
                <option value="provider">Provider</option>
              </select>
              <select
                bind:value={librarySortDir}
                on:change={() => void refreshLibraryItems({ reset: true })}
                class="px-3 py-2 rounded-lg text-sm text-zinc-300 bg-zinc-800 border border-zinc-700 focus:outline-none focus:border-indigo-500"
              >
                <option value="desc">Desc</option>
                <option value="asc">Asc</option>
              </select>
            {:else}
              <label class="text-xs font-medium text-zinc-400 uppercase tracking-wide sm:ml-2">Group</label>
              <select
                bind:value={libraryGroupBy}
                on:change={() => void refreshLibraryGroups({ reset: true })}
                class="px-3 py-2 rounded-lg text-sm text-zinc-300 bg-zinc-800 border border-zinc-700 focus:outline-none focus:border-indigo-500"
              >
                <option value="artist">Artist</option>
                <option value="album">Album</option>
                <option value="playlist">Playlist</option>
                <option value="provider">Provider</option>
                <option value="kind">Type</option>
              </select>

              <label class="text-xs font-medium text-zinc-400 uppercase tracking-wide sm:ml-2">Sort</label>
              <select
                bind:value={libraryGroupSort}
                on:change={() => void refreshLibraryGroups({ reset: true })}
                class="px-3 py-2 rounded-lg text-sm text-zinc-300 bg-zinc-800 border border-zinc-700 focus:outline-none focus:border-indigo-500"
              >
                <option value="count">Count</option>
                <option value="name">Name</option>
                <option value="last_synced">Updated</option>
              </select>
              <select
                bind:value={libraryGroupSortDir}
                on:change={() => void refreshLibraryGroups({ reset: true })}
                class="px-3 py-2 rounded-lg text-sm text-zinc-300 bg-zinc-800 border border-zinc-700 focus:outline-none focus:border-indigo-500"
              >
                <option value="desc">Desc</option>
                <option value="asc">Asc</option>
              </select>
            {/if}
          </div>

          <div class="flex items-center gap-2">
            <input
              type="text"
              bind:value={libraryItemsQuery}
              on:input={scheduleLibraryExplorerRefresh}
              placeholder="Search your library..."
              class="w-full md:w-72 px-3 py-2 rounded-lg text-sm text-zinc-200 placeholder-zinc-500 bg-zinc-800 border border-zinc-700 focus:outline-none focus:border-indigo-500"
            />
            {#if libraryItemsQuery.trim().length > 0}
              <button
                type="button"
                on:click={() => {
                  libraryItemsQuery = '';
                  void refreshLibraryExplorer(true);
                }}
                class="px-3 py-2 rounded-lg text-sm font-medium text-zinc-300 bg-zinc-800 border border-zinc-700 hover:bg-zinc-700"
              >
                Clear
              </button>
            {/if}
          </div>
        </div>

        {#if connectedPlatforms.length === 0}
          <div class="py-8 text-center text-zinc-400">
            Connect a streaming service to see your personal catalog here.
          </div>
        {:else if libraryFilterArtist || libraryFilterAlbum || libraryFilterPlaylist}
          <div class="flex flex-wrap items-center gap-2 mb-4 text-xs text-zinc-300">
            {#if libraryFilterArtist}
              <span class="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-zinc-800 border border-zinc-700">
                Artist: <span class="text-white">{libraryFilterArtist}</span>
                <button
                  type="button"
                  class="text-zinc-400 hover:text-white"
                  on:click={() => {
                    libraryFilterArtist = '';
                    void refreshLibraryExplorer(true);
                  }}
                  aria-label="Clear artist filter"
                >
                  ×
                </button>
              </span>
            {/if}
            {#if libraryFilterAlbum}
              <span class="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-zinc-800 border border-zinc-700">
                Album: <span class="text-white">{libraryFilterAlbum}</span>
                <button
                  type="button"
                  class="text-zinc-400 hover:text-white"
                  on:click={() => {
                    libraryFilterAlbum = '';
                    void refreshLibraryExplorer(true);
                  }}
                  aria-label="Clear album filter"
                >
                  ×
                </button>
              </span>
            {/if}
            {#if libraryFilterPlaylist}
              <span class="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-zinc-800 border border-zinc-700">
                Playlist: <span class="text-white">{libraryFilterPlaylist}</span>
                <button
                  type="button"
                  class="text-zinc-400 hover:text-white"
                  on:click={() => {
                    libraryFilterPlaylist = '';
                    void refreshLibraryExplorer(true);
                  }}
                  aria-label="Clear playlist filter"
                >
                  ×
                </button>
              </span>
            {/if}
            <button
              type="button"
              class="px-3 py-1 rounded-full bg-zinc-800 border border-zinc-700 hover:bg-zinc-700 text-zinc-300"
              on:click={() => {
                clearLibraryEntityFilters();
                void refreshLibraryExplorer(true);
              }}
            >
              Clear all
            </button>
          </div>
        {/if}

        {#if connectedPlatforms.length !== 0}
          {#if libraryBrowseView === 'grouped'}
            {#if libraryGroupsLoading && libraryGroups.length === 0}
              <div class="py-10 text-center">
                <div class="w-8 h-8 border-4 border-indigo-500 border-t-transparent rounded-full animate-spin mx-auto"></div>
                <p class="text-zinc-400 mt-3">Building groupings...</p>
              </div>
            {:else if libraryGroupsError}
              <div class="py-8 text-center text-red-300">
                {libraryGroupsError}
              </div>
            {:else if libraryGroups.length === 0}
              <div class="py-8 text-center">
                <div class="text-zinc-400 mb-2">No groups found.</div>
                <div class="text-xs text-zinc-500">
                  Sync a library, then try grouping by Artist or Album.
                </div>
              </div>
            {:else}
              <div class="overflow-x-auto">
                <table class="w-full">
                  <thead class="border-b border-zinc-700">
                    <tr class="text-left text-xs font-medium text-zinc-400 uppercase tracking-wide">
                      <th class="py-2 pr-4">Group</th>
                      <th class="py-2 pr-4">Count</th>
                      <th class="py-2 pr-4">Updated</th>
                      <th class="py-2 pr-0">Action</th>
                    </tr>
                  </thead>
                  <tbody class="border-t border-zinc-700">
                    {#each libraryGroups as group (group.value + '::' + (group.secondary ?? '') + '::' + (group.provider ?? ''))}
                      <tr class="hover:bg-zinc-800">
                        <td class="py-2 pr-4">
                          <div class="text-white font-medium">{group.value}</div>
                          {#if group.secondary}
                            <div class="text-xs text-zinc-500">{group.secondary}</div>
                          {/if}
                          {#if group.provider && libraryGroupBy === 'playlist'}
                            <div class="text-xs text-zinc-500 inline-flex items-center gap-1">
                              <ProviderIcon provider={group.provider} size={12} />
                              {getProviderName(group.provider)}
                            </div>
                          {/if}
                        </td>
                        <td class="py-2 pr-4 text-zinc-300 whitespace-nowrap">{(group.count ?? 0).toLocaleString()}</td>
                        <td class="py-2 pr-4 text-zinc-300 whitespace-nowrap">{timeAgo(group.last_synced)}</td>
                        <td class="py-2 pr-0">
                          <div class="flex items-center gap-2">
                            <button
                              type="button"
                              class="px-3 py-1.5 rounded-lg text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 transition-colors"
                              on:click={() => applyLibraryGroupFilter(group)}
                            >
                              View items
                            </button>
                            {#if libraryGroupBy === 'playlist'}
                              <button
                                type="button"
                                class="px-3 py-1.5 rounded-lg text-sm font-medium text-rose-400 bg-rose-500/10 hover:bg-rose-500/20 border border-rose-500/30 transition-colors"
                                on:click={() => navigateTo('playlist-sanitizer')}
                                title="Sanitize this playlist"
                              >
                                Sanitize
                              </button>
                            {/if}
                          </div>
                        </td>
                      </tr>
                    {/each}
                  </tbody>
                </table>
              </div>

              {#if libraryGroups.length < libraryGroupsTotal}
                <div class="pt-4 flex items-center justify-between">
                  <div class="text-xs text-zinc-500">
                    Showing {libraryGroups.length.toLocaleString()} of {libraryGroupsTotal.toLocaleString()}
                  </div>
                  <button
                    type="button"
                    on:click={() => void refreshLibraryGroups({ append: true })}
                    disabled={libraryGroupsLoading}
                    class="px-4 py-2 rounded-lg text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 transition-colors disabled:opacity-50"
                  >
                    {libraryGroupsLoading ? 'Loading...' : 'Load more'}
                  </button>
                </div>
              {/if}
            {/if}
          {:else}
            {#if libraryItemsLoading && libraryItems.length === 0}
              <div class="py-10 text-center">
                <div class="w-8 h-8 border-4 border-indigo-500 border-t-transparent rounded-full animate-spin mx-auto"></div>
                <p class="text-zinc-400 mt-3">Loading library items...</p>
              </div>
            {:else if libraryItemsError}
              <div class="py-8 text-center text-red-300">
                {libraryItemsError}
              </div>
            {:else if libraryItems.length === 0}
              <div class="py-16 text-center">
                <div class="mx-auto mb-4 w-16 h-16 rounded-full bg-zinc-800 flex items-center justify-center">
                  <svg class="w-8 h-8 text-zinc-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M9 9l10.5-3m0 6.553v3.75a2.25 2.25 0 01-1.632 2.163l-1.32.377a1.803 1.803 0 11-.99-3.467l2.31-.66a2.25 2.25 0 001.632-2.163zm0 0V2.25L9 5.25v10.303m0 0v3.75a2.25 2.25 0 01-1.632 2.163l-1.32.377a1.803 1.803 0 01-.99-3.467l2.31-.66A2.25 2.25 0 009 15.553z" />
                  </svg>
                </div>
                <div class="text-lg font-medium text-zinc-300 mb-1">No library items yet</div>
                <div class="text-sm text-zinc-500 max-w-sm mx-auto">
                  Connect a streaming service above, then click "Sync Library" to import your favorites and playlists.
                </div>
              </div>
            {:else}
              <div class="overflow-x-auto">
                <table class="w-full library-table">
                  <thead>
                    <tr class="text-left text-[11px] font-semibold text-zinc-500 uppercase tracking-wider border-b border-zinc-700/60">
                      <th class="py-2.5 pr-3 w-10"></th>
                      <th class="py-2.5 pr-4">Title</th>
                      <th class="py-2.5 pr-4">Artist</th>
                      <th class="py-2.5 pr-4">Album</th>
                      <th class="py-2.5 pr-4">Type</th>
                      <th class="py-2.5 pr-0">Added</th>
                    </tr>
                  </thead>
                  <tbody>
                    {#each libraryItems as item (item.id ?? `${item.provider ?? 'unknown'}:${item.provider_track_id ?? 'unknown'}`)}
                      {@const provider = item.provider || 'unknown'}
                      {@const providerName = getProviderName(provider)}
                      {@const kind = kindFromImportedItem(item)}
                      {@const title = item.track_name || item.playlist_name || item.provider_track_id || '(Untitled)'}
                      <tr class="library-table__row">
                        <td class="py-3 pr-3" title={providerName}>
                          <ProviderIcon provider={provider} size={16} />
                        </td>
                        <td class="py-3 pr-4 max-w-md truncate" title={title}>
                          <span class="text-white font-medium text-sm">{title}</span>
                        </td>
                        <td class="py-3 pr-4 max-w-xs truncate text-sm text-zinc-400" title={item.artist_name || ''}>
                          {item.artist_name || '--'}
                        </td>
                        <td class="py-3 pr-4 max-w-xs truncate text-xs text-zinc-500" title={item.album_name || item.playlist_name || ''}>
                          {item.album_name || item.playlist_name || '--'}
                        </td>
                        <td class="py-3 pr-4 text-xs text-zinc-500 whitespace-nowrap">{kind}</td>
                        <td class="py-3 pr-0 text-xs text-zinc-500 whitespace-nowrap">{item.added_at ? timeAgo(item.added_at) : '--'}</td>
                      </tr>
                    {/each}
                  </tbody>
                </table>
              </div>

              <div class="pt-4 flex items-center justify-between">
                <div class="text-xs text-zinc-500">
                  Showing {libraryItems.length.toLocaleString()} of {libraryItemsTotal.toLocaleString()}
                </div>
                {#if libraryItems.length < libraryItemsTotal}
                  <button
                    type="button"
                    on:click={() => void refreshLibraryItems({ append: true })}
                    disabled={libraryItemsLoading}
                    class="px-4 py-2 rounded-lg text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 transition-colors disabled:opacity-50"
                  >
                    {libraryItemsLoading ? 'Loading...' : 'Load more'}
                  </button>
                {/if}
              </div>
            {/if}
          {/if}
        {/if}
      </div>

      <p class="text-xs text-zinc-500 mt-2">
        Note: “Downloaded/offline” items usually are not exposed by provider APIs. We can show library additions, favorites, and playlists.
      </p>
    </div>

    <!-- Recent Sync Runs -->
    <div>
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-xl font-semibold text-white">Recent Sync Runs</h2>
        <button
          type="button"
          on:click={() => syncActions.fetchRuns()}
          class="brand-button brand-button--secondary brand-button--compact"
        >
          <svg class="w-4 h-4 inline" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/><path d="M3.51 9a9 9 0 0114.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0020.49 15"/></svg>
          Refresh
        </button>
      </div>

      {#if $syncStore.isLoading}
        <div class="flex justify-center py-12">
          <div class="w-8 h-8 border-4 border-indigo-500 border-t-transparent rounded-full animate-spin"></div>
        </div>
      {:else if $recentRuns.length === 0}
        <div class="rounded-xl p-8 text-center bg-zinc-700" >
          <span class="text-4xl mb-3 block text-zinc-500">--</span>
          <p class="text-zinc-300">No sync runs yet. Trigger your first sync above.</p>
        </div>
      {:else}
        <div class="bg-zinc-900 rounded-xl shadow-sm overflow-hidden border border-zinc-700" >
          <table class="w-full">
            <thead class="bg-zinc-700 border-b border-zinc-700">
              <tr>
                <th class="px-4 py-3 text-left text-xs font-medium text-zinc-400 uppercase">Platform</th>
                <th class="px-4 py-3 text-left text-xs font-medium text-zinc-400 uppercase">Type</th>
                <th class="px-4 py-3 text-left text-xs font-medium text-zinc-400 uppercase">Status</th>
                <th class="px-4 py-3 text-left text-xs font-medium text-zinc-400 uppercase">Artists</th>
                <th class="px-4 py-3 text-left text-xs font-medium text-zinc-400 uppercase">Duration</th>
                <th class="px-4 py-3 text-left text-xs font-medium text-zinc-400 uppercase">Started</th>
                <th class="px-4 py-3 text-left text-xs font-medium text-zinc-400 uppercase">Actions</th>
              </tr>
            </thead>
            <tbody class="border-t border-zinc-700">
              {#each $recentRuns as run}
                <tr class="hover:bg-zinc-800">
                  <td class="px-4 py-3">
                    <span class="capitalize font-medium">{run.platform}</span>
                  </td>
                  <td class="px-4 py-3">
                    <span class="px-2 py-0.5 rounded text-xs {run.sync_type === 'full' ? 'bg-purple-100 text-purple-700' : 'bg-indigo-100 text-indigo-300'}">
                      {run.sync_type}
                    </span>
                  </td>
                  <td class="px-4 py-3">
                    <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium {STATUS_COLORS[run.status] || DEFAULT_STATUS_COLOR}">
                      {STATUS_ICONS[run.status] || DEFAULT_STATUS_ICON} {run.status}
                    </span>
                  </td>
                  <td class="px-4 py-3 text-zinc-300">
                    {(run.artists_processed ?? 0).toLocaleString()}
                    {#if (run.errors_count ?? 0) > 0}
                      <span class="text-red-500 ml-1">({run.errors_count} errors)</span>
                    {/if}
                  </td>
                  <td class="px-4 py-3 text-zinc-300">
                    {formatDuration(run.duration_ms)}
                  </td>
                  <td class="px-4 py-3 text-zinc-300">
                    {timeAgo(run.started_at)}
                  </td>
                  <td class="px-4 py-3">
                    {#if run.status === 'running' || run.status === 'pending'}
                      <button
                        type="button"
                        on:click={() => syncActions.cancelRun(run.id)}
                        class="brand-text-action brand-text-action--danger"
                      >
                        Cancel
                      </button>
                    {:else}
                      <button
                        type="button"
                        on:click={() => syncActions.fetchRun(run.id)}
                        class="brand-text-action"
                      >
                        Details
                      </button>
                    {/if}
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </div>
    </div>
  </div>
</div>


<style>
  .library-table {
    border-collapse: separate;
    border-spacing: 0;
  }

  .library-table :global(.library-table__row) {
    transition: background-color 0.15s ease;
    border-bottom: 1px solid rgba(63, 63, 70, 0.4);
  }

  .library-table :global(.library-table__row:hover) {
    background-color: rgba(63, 63, 70, 0.35);
  }

  .sync-dashboard-page {
    --sync-panel-bg: linear-gradient(155deg, rgba(18, 20, 28, 0.94), rgba(11, 13, 19, 0.96));
    --sync-border: rgba(82, 88, 112, 0.46);
  }

  .sync-dashboard-page > .brand-page__inner > div {
    display: flex;
    flex-direction: column;
    gap: 1.9rem;
  }

  .sync-dashboard-page .surface-card {
    background: var(--sync-panel-bg);
    border: 1px solid var(--sync-border);
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04);
  }

  .sync-dashboard-page button svg {
    width: 1rem;
    height: 1rem;
    flex: 0 0 auto;
  }

  .sync-dashboard-page .brand-button--compact {
    min-height: 2.3rem;
    padding: 0.45rem 0.85rem;
    font-size: 0.82rem;
    gap: 0.45rem;
  }

  .sync-dashboard-page .brand-button--compact svg {
    width: 0.95rem;
    height: 0.95rem;
  }

  .sync-dashboard-page .brand-alert {
    border-radius: 0.85rem;
  }

  .sync-dashboard-page .grid {
    display: grid;
  }

  .sync-dashboard-page .flex {
    display: flex;
  }

  .sync-dashboard-page .inline-flex {
    display: inline-flex;
  }

  .sync-dashboard-page .items-center {
    align-items: center;
  }

  .sync-dashboard-page .items-start {
    align-items: flex-start;
  }

  .sync-dashboard-page .justify-between {
    justify-content: space-between;
  }

  .sync-dashboard-page .justify-center {
    justify-content: center;
  }

  .sync-dashboard-page .flex-col {
    flex-direction: column;
  }

  .sync-dashboard-page .flex-wrap {
    flex-wrap: wrap;
  }

  .sync-dashboard-page .flex-1 {
    flex: 1 1 0%;
  }

  .sync-dashboard-page .min-w-0 {
    min-width: 0;
  }

  .sync-dashboard-page .w-full {
    width: 100%;
  }

  .sync-dashboard-page .w-4,
  .sync-dashboard-page .h-4 {
    width: 1rem;
    height: 1rem;
  }

  .sync-dashboard-page .w-5,
  .sync-dashboard-page .h-5 {
    width: 1.25rem;
    height: 1.25rem;
  }

  .sync-dashboard-page .w-6,
  .sync-dashboard-page .h-6 {
    width: 1.5rem;
    height: 1.5rem;
  }

  .sync-dashboard-page .w-7,
  .sync-dashboard-page .h-7 {
    width: 1.75rem;
    height: 1.75rem;
  }

  .sync-dashboard-page .w-8,
  .sync-dashboard-page .h-8 {
    width: 2rem;
    height: 2rem;
  }

  .sync-dashboard-page .w-12,
  .sync-dashboard-page .h-12 {
    width: 3rem;
    height: 3rem;
  }

  .sync-dashboard-page .w-14,
  .sync-dashboard-page .h-14 {
    width: 3.5rem;
    height: 3.5rem;
  }

  .sync-dashboard-page .w-28,
  .sync-dashboard-page .h-28 {
    width: 7rem;
    height: 7rem;
  }

  .sync-dashboard-page .gap-1 {
    gap: 0.25rem;
  }

  .sync-dashboard-page .gap-2 {
    gap: 0.5rem;
  }

  .sync-dashboard-page .gap-3 {
    gap: 0.75rem;
  }

  .sync-dashboard-page .gap-4 {
    gap: 1rem;
  }

  .sync-dashboard-page .gap-6 {
    gap: 1.5rem;
  }

  .sync-dashboard-page .gap-x-2 {
    column-gap: 0.5rem;
  }

  .sync-dashboard-page .gap-y-1 {
    row-gap: 0.25rem;
  }

  .sync-dashboard-page .mb-2 {
    margin-bottom: 0.5rem;
  }

  .sync-dashboard-page .mb-3 {
    margin-bottom: 0.75rem;
  }

  .sync-dashboard-page .mb-4 {
    margin-bottom: 1rem;
  }

  .sync-dashboard-page .mb-6 {
    margin-bottom: 1.5rem;
  }

  .sync-dashboard-page .mb-8 {
    margin-bottom: 2rem;
  }

  .sync-dashboard-page .mt-1 {
    margin-top: 0.25rem;
  }

  .sync-dashboard-page .mt-2 {
    margin-top: 0.5rem;
  }

  .sync-dashboard-page .mt-3 {
    margin-top: 0.75rem;
  }

  .sync-dashboard-page .mt-4 {
    margin-top: 1rem;
  }

  .sync-dashboard-page .mt-6 {
    margin-top: 1.5rem;
  }

  .sync-dashboard-page .ml-1 {
    margin-left: 0.25rem;
  }

  .sync-dashboard-page .ml-2 {
    margin-left: 0.5rem;
  }

  .sync-dashboard-page .p-2 {
    padding: 0.5rem;
  }

  .sync-dashboard-page .p-3 {
    padding: 0.75rem;
  }

  .sync-dashboard-page .p-4 {
    padding: 1rem;
  }

  .sync-dashboard-page .p-5 {
    padding: 1.25rem;
  }

  .sync-dashboard-page .p-6 {
    padding: 1.5rem;
  }

  .sync-dashboard-page .p-8 {
    padding: 2rem;
  }

  .sync-dashboard-page .px-2 {
    padding-left: 0.5rem;
    padding-right: 0.5rem;
  }

  .sync-dashboard-page .px-3 {
    padding-left: 0.75rem;
    padding-right: 0.75rem;
  }

  .sync-dashboard-page .px-4 {
    padding-left: 1rem;
    padding-right: 1rem;
  }

  .sync-dashboard-page .py-0\.5 {
    padding-top: 0.125rem;
    padding-bottom: 0.125rem;
  }

  .sync-dashboard-page .py-1 {
    padding-top: 0.25rem;
    padding-bottom: 0.25rem;
  }

  .sync-dashboard-page .py-1\.5 {
    padding-top: 0.375rem;
    padding-bottom: 0.375rem;
  }

  .sync-dashboard-page .py-2 {
    padding-top: 0.5rem;
    padding-bottom: 0.5rem;
  }

  .sync-dashboard-page .py-2\.5 {
    padding-top: 0.625rem;
    padding-bottom: 0.625rem;
  }

  .sync-dashboard-page .py-3 {
    padding-top: 0.75rem;
    padding-bottom: 0.75rem;
  }

  .sync-dashboard-page .py-8 {
    padding-top: 2rem;
    padding-bottom: 2rem;
  }

  .sync-dashboard-page .py-10 {
    padding-top: 2.5rem;
    padding-bottom: 2.5rem;
  }

  .sync-dashboard-page .py-12 {
    padding-top: 3rem;
    padding-bottom: 3rem;
  }

  .sync-dashboard-page .pt-2 {
    padding-top: 0.5rem;
  }

  .sync-dashboard-page .pt-4 {
    padding-top: 1rem;
  }

  .sync-dashboard-page .rounded {
    border-radius: 0.25rem;
  }

  .sync-dashboard-page .rounded-lg {
    border-radius: 0.65rem;
  }

  .sync-dashboard-page .rounded-xl {
    border-radius: 0.95rem;
  }

  .sync-dashboard-page .rounded-2xl {
    border-radius: 1.15rem;
  }

  .sync-dashboard-page .rounded-full {
    border-radius: 9999px;
  }

  .sync-dashboard-page .border {
    border-width: 1px;
    border-style: solid;
  }

  .sync-dashboard-page .border-2 {
    border-width: 2px;
    border-style: solid;
  }

  .sync-dashboard-page .border-4 {
    border-width: 4px;
    border-style: solid;
  }

  .sync-dashboard-page .border-t {
    border-top-width: 1px;
    border-top-style: solid;
  }

  .sync-dashboard-page .grid-cols-1 {
    grid-template-columns: repeat(1, minmax(0, 1fr));
  }

  .sync-dashboard-page .grid-cols-2 {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .sync-dashboard-page .space-y-1 > * + * {
    margin-top: 0.25rem;
  }

  .sync-dashboard-page .space-y-3 > * + * {
    margin-top: 0.75rem;
  }

  .sync-dashboard-page .animate-spin {
    animation: sync-spin 1s linear infinite;
  }

  .sync-dashboard-page table {
    width: 100%;
    border-collapse: collapse;
  }

  .sync-dashboard-page .overflow-x-auto {
    overflow-x: auto;
  }

  .sync-dashboard-page .truncate {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .sync-dashboard-page .whitespace-nowrap {
    white-space: nowrap;
  }

  .sync-dashboard-page .text-2xl {
    font-size: 1.5rem;
    line-height: 2rem;
  }

  .sync-dashboard-page .text-xl {
    font-size: 1.25rem;
    line-height: 1.75rem;
  }

  .sync-dashboard-page .text-3xl {
    font-size: 1.875rem;
    line-height: 2.2rem;
  }

  @keyframes sync-spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  @media (min-width: 640px) {
    .sync-dashboard-page .sm\:flex-row {
      flex-direction: row;
    }

    .sync-dashboard-page .sm\:items-center {
      align-items: center;
    }

    .sync-dashboard-page .sm\:justify-between {
      justify-content: space-between;
    }

    .sync-dashboard-page .sm\:flex-wrap {
      flex-wrap: wrap;
    }

    .sync-dashboard-page .sm\:ml-2 {
      margin-left: 0.5rem;
    }
  }

  @media (min-width: 768px) {
    .sync-dashboard-page .md\:grid-cols-2 {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }

    .sync-dashboard-page .md\:flex-row {
      flex-direction: row;
    }

    .sync-dashboard-page .md\:items-center {
      align-items: center;
    }

    .sync-dashboard-page .md\:w-72 {
      width: 18rem;
    }
  }

  @media (min-width: 1024px) {
    .sync-dashboard-page .lg\:grid-cols-3 {
      grid-template-columns: repeat(3, minmax(0, 1fr));
    }

    .sync-dashboard-page .lg\:flex-row {
      flex-direction: row;
    }

    .sync-dashboard-page .lg\:items-center {
      align-items: center;
    }

    .sync-dashboard-page .lg\:justify-between {
      justify-content: space-between;
    }
  }

  @media (max-width: 900px) {
    .sync-dashboard-page .brand-hero__header {
      gap: 1.2rem;
    }

    .sync-dashboard-page .brand-actions {
      width: 100%;
      justify-content: flex-start;
    }
  }

  @media (max-width: 640px) {
    .sync-dashboard-page {
      overflow-x: hidden;
    }

    .sync-dashboard-page > .brand-page__inner {
      gap: 1.25rem;
    }

    .sync-dashboard-page .brand-title {
      font-size: clamp(1.8rem, 6.5vw, 2.35rem);
      line-height: 1.04;
    }
  }
</style>
