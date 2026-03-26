<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { get } from 'svelte/store';
  import { syncStore, syncActions, isAnySyncRunning, platformsStatus, recentRuns } from '../stores/sync';
  import type { TriggerSyncRequest } from '../stores/sync';
  import { navigateTo, navigateToArtist } from '../utils/simple-router';
  import { connectionsStore, connectionActions, type ServiceConnection } from '../stores/connections';
  import { apiClient } from '../utils/api-client';
  import { blockingStore } from '../stores/blocking';
  import { timeAgo } from '../utils/time-ago';
  import { HEAVY_LIBRARY_GROUP_TTL_MS, syncDashboardHeavyCache } from '../utils/sync-dashboard-cache';
  import ServiceConnector from './ServiceConnector.svelte';
  import { billingActions } from '../stores/billing';

  let selectedPlatforms: string[] = [];
  let syncType: 'full' | 'incremental' = 'incremental';
  let priority: 'low' | 'normal' | 'high' | 'critical' = 'normal';
  let showTriggerModal = false;

  // Connection states
  let connectingPlatform: string | null = null;
  let syncingLibrary: string | null = null;
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
  let appleLibraryLoading = false;
  let appleLibraryError: string | null = null;
  let appleLibraryRequested = false;
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
    status: 'ready' | 'error';
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
  let connectionsByProvider = new Map<string, ServiceConnection>();
  let activeProviders = new Set<string>();
  let connectedPlatforms: Platform[] = [];

  $: connectionsByProvider = new Map<string, ServiceConnection>(
    ($connectionsStore.connections ?? []).map((conn) => [conn.provider, conn])
  );

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

  function getPlatformStatusColor(status: PlatformStatus): string {
    switch (status) {
      case 'ready': return 'bg-green-500/20 text-green-400 border-green-500/30';
      case 'paused': return 'bg-amber-500/20 text-amber-400 border-amber-500/30';
      case 'catalog-only': return 'bg-zinc-500/20 text-zinc-400 border-zinc-500/30';
      default: return 'bg-zinc-500/20 text-zinc-400 border-zinc-500/30';
    }
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

  function formatSeverity(value: OffenderArtist['severity']): string {
    switch (value) {
      case 'egregious': return 'Egregious';
      case 'severe': return 'Severe';
      case 'moderate': return 'Moderate';
      case 'minor': return 'Minor';
      default: return value;
    }
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

  function buildLibraryItemsCacheKey(): string {
    return buildLibraryItemsEndpoint(0, libraryItemsLimit);
  }

  function buildLibraryGroupsCacheKey(): string {
    return buildLibraryGroupsEndpoint(0, libraryGroupsLimit);
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
    if (syncDashboardHeavyCache.libraryItemsKey !== buildLibraryItemsCacheKey()) return false;
    if (syncDashboardHeavyCache.libraryGroupsKey !== buildLibraryGroupsCacheKey()) return false;
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
    syncDashboardHeavyCache.libraryItemsKey = buildLibraryItemsCacheKey();
    syncDashboardHeavyCache.libraryGroups = libraryGroups;
    syncDashboardHeavyCache.libraryGroupsTotal = libraryGroupsTotal;
    syncDashboardHeavyCache.libraryGroupsOffset = libraryGroupsOffset;
    syncDashboardHeavyCache.libraryGroupsKey = buildLibraryGroupsCacheKey();
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

  async function loadAppleLibraryPreview(force = false): Promise<AppleLibraryPreview | null> {
    if (!hasActiveConnection('apple_music')) return null;
    if (appleLibraryLoading) return appleLibrary;
    if (!force && appleLibraryRequested) return appleLibrary;

    appleLibraryRequested = true;
    appleLibraryLoading = true;
    appleLibraryError = null;

    try {
      const result = await apiClient.authenticatedRequest<any>(
        'GET',
        '/api/v1/apple-music/library?limit=100'
      );
      if (result.success) {
        const payload = (result.data ?? result) as any;
        const tracks = Array.isArray(payload?.tracks) ? payload.tracks : [];
        const albums = Array.isArray(payload?.albums) ? payload.albums : [];
        const playlists = Array.isArray(payload?.playlists) ? payload.playlists : [];
        const tracksTotal = Number(payload?.tracks_total ?? tracks.length);
        const albumsTotal = Number(payload?.albums_total ?? albums.length);
        const artistsTotal = Number(payload?.artists_total ?? 0);
        const playlistsTotal = Number(payload?.playlists_total ?? playlists.length);

        appleLibrary = {
          tracks,
          albums,
          playlists,
          tracksCount: Number.isFinite(tracksTotal) ? tracksTotal : tracks.length,
          albumsCount: Number.isFinite(albumsTotal) ? albumsTotal : albums.length,
          artistsCount: Number.isFinite(artistsTotal) && artistsTotal > 0 ? artistsTotal : 0,
          playlistsCount: Number.isFinite(playlistsTotal) ? playlistsTotal : playlists.length,
          scannedAt: payload?.scanned_at,
        };
        return appleLibrary;
      } else {
        const isServerError = result.error_code?.startsWith('HTTP_5');
        appleLibraryError = isServerError
          ? 'Apple Music connection may need to be refreshed. Try disconnecting and reconnecting.'
          : (result.message || 'Failed to load Apple Music library');
        return null;
      }
    } catch (error) {
      appleLibraryError = error instanceof Error ? error.message : 'Failed to load Apple Music library';
      return null;
    } finally {
      appleLibraryLoading = false;
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
      status: hasImportedData ? 'ready' : 'error',
      message: hasImportedData ? undefined : 'No library data synced yet. Click "Sync Library" to import your favorites and playlists.',
    };
  }

  async function buildLibraryStatsRow(platform: Platform): Promise<ProviderLibraryStatsRow> {
    try {
      if (platform.id === 'apple') {
        let importedTracks: ImportedLibraryTrack[] = [];
        try {
          importedTracks = await fetchImportedLibraryTracks('apple_music');
        } catch (error) {
          console.warn('Failed to load Apple imported library cache:', error);
        }

        if (importedTracks.length > 0) {
          return summarizeImportedLibrary(platform, importedTracks);
        }

        const preview = appleLibrary;

        if (!preview) {
          return {
            provider: platform.connectionProvider || platform.id,
            providerName: platform.name,
            songs: null,
            albums: null,
            artists: null,
            playlists: null,
            totalItems: null,
            source: 'live_api',
            status: 'error',
            message: appleLibraryError || 'Apple Music library is unavailable',
          };
        }

        const uniqueArtists = new Set(
          preview.tracks
            .map(track => track.artist?.trim())
            .filter((artist): artist is string => Boolean(artist))
        ).size;

        return {
          provider: platform.connectionProvider || platform.id,
          providerName: platform.name,
          songs: preview.tracksCount,
          albums: preview.albumsCount,
          artists: preview.artistsCount > 0 ? preview.artistsCount : uniqueArtists,
          playlists: preview.playlistsCount,
          totalItems: preview.tracksCount + preview.albumsCount + preview.playlistsCount,
          lastSynced: preview.scannedAt,
          source: 'live_api',
          status: 'ready',
          message:
            appleLibrarySyncStatus?.state === 'running'
              ? 'Apple Music preview is loaded and the cached import is still running. The sections below will populate when the import finishes.'
              : 'Preview loaded from Apple Music. Use "Sync Library" or "Sync All" to import items into the cached library views.',
        };
      }

      const provider = platform.connectionProvider || platform.id;
      const importedTracks = await fetchImportedLibraryTracks(provider);
      return summarizeImportedLibrary(platform, importedTracks);
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
        source: platform.id === 'apple' ? 'live_api' : 'imported_cache',
        status: 'error',
        message,
      };
    }
  }

  function getAppleLibraryStatsRow(): ProviderLibraryStatsRow | undefined {
    return libraryStatsByProvider.get('apple_music');
  }

  function hasAppleImportedCache(): boolean {
    const row = getAppleLibraryStatsRow();
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

  // Connect to a platform
  function getProviderHints(platform: Platform): string[] {
    const hints = [platform.id, platform.name, platform.connectionProvider ?? '']
      .map(value => value.toLowerCase())
      .filter(Boolean);

    if (platform.id === 'apple') hints.push('apple');
    if (platform.id === 'youtube') hints.push('youtube', 'youtube music');

    return hints;
  }

  function isAlreadyConnectedError(message: string | undefined, platform: Platform): boolean {
    if (!message) return false;

    const normalized = message.toLowerCase();
    if (!normalized.includes('already have an active')) return false;
    if (normalized.includes('disconnect first to reconnect')) return true;

    return getProviderHints(platform).some(hint => {
      const normalizedHint = hint.replace(/[_-]/g, ' ');
      return normalized.includes(hint) || normalized.includes(normalizedHint);
    });
  }

  function showAlreadyConnectedMessage(platformName: string): void {
    showConnectionSuccess(
      `${platformName} is already connected. Use "Sync Library" or disconnect first to reconnect.`
    );
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

  function isSuccessLikeMessage(message: string | undefined): boolean {
    if (!message) return false;
    const normalized = message.toLowerCase();
    return (
      normalized.includes('connected successfully') ||
      normalized.includes('library imported') ||
      normalized.includes('synced successfully') ||
      normalized.includes('sync complete')
    );
  }

  async function connectPlatform(platform: Platform) {
    if (!platform.connectionProvider || platform.disabled) return;

    connectingPlatform = platform.id;
    connectionError = null;
    connectionSuccess = null;

    try {
      // Refresh connection state before triggering OAuth to avoid reconnect loops
      // when the page is refreshed and local UI state is stale.
      await connectionActions.fetchConnections();
      if (hasActiveConnection(platform.connectionProvider)) {
        showAlreadyConnectedMessage(platform.name);
        return;
      }

      if (platform.id === 'apple') {
        const result = await connectionActions.connectAppleMusic();
        if (result.success || isSuccessLikeMessage(result.message)) {
          showConnectionSuccess(
            `${platform.name} connected successfully. Click "Sync Library" to fetch your library.`
          );
          appleLibraryRequested = false;
        } else if (isAlreadyConnectedError(result.message, platform)) {
          await connectionActions.fetchConnections();
          showAlreadyConnectedMessage(platform.name);
        } else {
          showConnectionError(result.message || `Failed to connect ${platform.name}`);
        }
      } else if (platform.id === 'spotify') {
        await connectionActions.initiateSpotifyAuth();
      } else if (platform.id === 'youtube') {
        // YouTube Music OAuth flow
        const result: any = await connectionActions.initiateYouTubeAuth();
        if (!result.success && isAlreadyConnectedError(result.message, platform)) {
          await connectionActions.fetchConnections();
          showAlreadyConnectedMessage(platform.name);
        } else if (!result.success && isSuccessLikeMessage(result.message)) {
          showConnectionSuccess(result.message || `${platform.name} connected successfully.`);
        } else if (!result.success) {
          showConnectionError(result.message || 'Failed to initiate YouTube auth');
        }
        // Note: If successful, page will redirect to Google OAuth
      } else if (platform.id === 'tidal') {
        // Tidal OAuth flow - uses GET endpoint with auth token
        const result = await connectionActions.initiateTidalAuth();
        if (result.alreadyConnected) {
          await connectionActions.fetchConnections();
          showAlreadyConnectedMessage(platform.name);
        } else if (!result.success) {
          if (isAlreadyConnectedError(result.message, platform)) {
            await connectionActions.fetchConnections();
            showAlreadyConnectedMessage(platform.name);
          } else if (isSuccessLikeMessage(result.message)) {
            showConnectionSuccess(result.message || `${platform.name} connected successfully.`);
          } else {
            showConnectionError(result.message || 'Failed to initiate Tidal auth');
          }
        }
        // Note: If successful, page will redirect to Tidal OAuth
      }
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : `Failed to connect ${platform.name}`;
      if (isAlreadyConnectedError(errorMessage, platform)) {
        await connectionActions.fetchConnections();
        showAlreadyConnectedMessage(platform.name);
      } else {
        showConnectionError(errorMessage);
      }
    } finally {
      await refreshHeavyLibraryViews({ force: true });
      connectingPlatform = null;
    }
  }

  // Disconnect from a platform
  async function disconnectPlatform(platform: Platform) {
    if (!platform.connectionProvider) return;

    connectingPlatform = platform.id;

    try {
      if (platform.id === 'apple') {
        await connectionActions.disconnectAppleMusic();
        appleLibrary = null;
        appleLibraryError = null;
        appleLibraryRequested = false;
      } else if (platform.id === 'spotify') {
        await connectionActions.disconnectSpotify();
      } else if (platform.id === 'tidal') {
        await connectionActions.disconnectTidal();
      } else if (platform.id === 'youtube') {
        await connectionActions.disconnectYouTube();
      } else {
        await connectionActions.fetchConnections();
      }
    } catch (error) {
      console.error(`Failed to disconnect ${platform.name}:`, error);
    } finally {
      await refreshHeavyLibraryViews({ force: true });
      connectingPlatform = null;
    }
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

    syncingLibrary = platform.id;
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
            payload?.message || `${platform.name} library synced successfully.`
          );
        } else if (
          syncResult.error_code === 'HTTP_404' ||
          syncResult.error_code === 'HTTP_405'
        ) {
          await triggerGenericSync();
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
      syncingLibrary = null;
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

  function getStatusColor(status: string): string {
    switch (status) {
      case 'running': return 'bg-rose-500/10 text-rose-400';
      case 'completed': return 'bg-green-500/20 text-green-400';
      case 'error': case 'failed': return 'bg-red-500/20 text-red-400';
      case 'cancelled': return 'bg-zinc-500/20 text-zinc-300';
      default: return 'bg-zinc-500/20 text-zinc-300';
    }
  }

  function getStatusIcon(status: string): string {
    switch (status) {
      case 'running': return '\u21BB';
      case 'completed': return '\u2713';
      case 'error': case 'failed': return '\u2717';
      case 'cancelled': return '\u25A0';
      default: return '\u2026';
    }
  }

  function formatDuration(ms?: number): string {
    if (!ms) return '-';
    if (ms < 1000) return `${ms}ms`;
    if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
    return `${(ms / 60000).toFixed(1)}m`;
  }

  function formatDate(dateStr?: string): string {
    return timeAgo(dateStr);
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

  function getStatsSourceLabel(source: ProviderLibraryStatsRow['source']): string {
    return source === 'live_api' ? 'Live API' : 'Imported Cache';
  }

  function openTriggerModal() {
    selectedPlatforms = connectedPlatforms
      .filter(
        (platform) =>
          Boolean(platform.connectionProvider) &&
          hasActiveConnection(platform.connectionProvider) &&
          !platform.disabled
      )
      .map((platform) => platform.id);
    syncType = 'incremental';
    priority = 'normal';
    showTriggerModal = true;
  }

  function closeTriggerModal() {
    showTriggerModal = false;
  }

  function togglePlatform(platformId: string) {
    if (selectedPlatforms.includes(platformId)) {
      selectedPlatforms = selectedPlatforms.filter(p => p !== platformId);
    } else {
      selectedPlatforms = [...selectedPlatforms, platformId];
    }
  }

  async function handleTriggerSync() {
    if (selectedPlatforms.length === 0) return;
    closeTriggerModal();

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
        sync_type: syncType,
        priority,
      };

      const result = await syncActions.triggerSync(request);
      if (!result.success && result.message) {
        showConnectionError(result.message);
      } else if (result.success) {
        queueGenericSyncPolling();
      }
    }
  }

  async function handleCancelRun(runId: string) {
    await syncActions.cancelRun(runId);
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
              on:click={openTriggerModal}
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

    <!-- Legacy Platform Status Grid (hidden — replaced by ServiceConnector above) -->
    <div class="mb-8 hidden">
      <h2 class="text-xl font-semibold text-white mb-4">Your Music Services</h2>
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {#each platforms as platform}
          {@const syncStatus = $platformsStatus.find(s => s.platform === platform.id)}
          {@const provider = platform.connectionProvider || platform.id}
          {@const connection = connectionsByProvider.get(provider) ?? null}
          {@const libraryRow = libraryStatsByProvider.get(provider) ?? null}
          {@const connected = connection?.status === 'active'}
          {@const needsReconnect = connection && connection.status !== 'active'}
          <div class="surface-card rounded-xl p-5 {platform.disabled ? 'opacity-60' : ''} transition-all hover:border-zinc-600" >
            <!-- Header with icon and status -->
            <div class="flex items-start justify-between mb-4">
              <div class="flex items-center gap-3">
                <div class="w-12 h-12 rounded-xl flex items-center justify-center" style="background-color: {platform.color}20; border: 1px solid {platform.color}40;">
                  {#if platform.icon === 'spotify'}
                    <svg class="w-6 h-6" fill="{platform.color}" viewBox="0 0 24 24">
                      <path d="M12 0C5.4 0 0 5.4 0 12s5.4 12 12 12 12-5.4 12-12S18.66 0 12 0zm5.521 17.34c-.24.359-.66.48-1.021.24-2.82-1.74-6.36-2.101-10.561-1.141-.418.122-.779-.179-.899-.539-.12-.421.18-.78.54-.9 4.56-1.021 8.52-.6 11.64 1.32.42.18.479.659.301 1.02zm1.44-3.3c-.301.42-.841.6-1.262.3-3.239-1.98-8.159-2.58-11.939-1.38-.479.12-1.02-.12-1.14-.6-.12-.48.12-1.021.6-1.141C9.6 9.9 15 10.561 18.72 12.84c.361.181.54.78.241 1.2zm.12-3.36C15.24 8.4 8.82 8.16 5.16 9.301c-.6.179-1.2-.181-1.38-.721-.18-.601.18-1.2.72-1.381 4.26-1.26 11.28-1.02 15.721 1.621.539.3.719 1.02.42 1.56-.299.421-1.02.599-1.559.3z"/>
                    </svg>
                  {:else if platform.icon === 'apple'}
                    <svg class="w-6 h-6" fill="{platform.color}" viewBox="0 0 24 24">
                      <path d="M18.71 19.5c-.83 1.24-1.71 2.45-3.05 2.47-1.34.03-1.77-.79-3.29-.79-1.53 0-2 .77-3.27.82-1.31.05-2.3-1.32-3.14-2.53C4.25 17 2.94 12.45 4.7 9.39c.87-1.52 2.43-2.48 4.12-2.51 1.28-.02 2.5.87 3.29.87.78 0 2.26-1.07 3.81-.91.65.03 2.47.26 3.64 1.98-.09.06-2.17 1.28-2.15 3.81.03 3.02 2.65 4.03 2.68 4.04-.03.07-.42 1.44-1.38 2.83M13 3.5c.73-.83 1.94-1.46 2.94-1.5.13 1.17-.34 2.35-1.04 3.19-.69.85-1.83 1.51-2.95 1.42-.15-1.15.41-2.35 1.05-3.11z"/>
                    </svg>
                  {:else if platform.icon === 'youtube'}
                    <svg class="w-6 h-6" fill="{platform.color}" viewBox="0 0 24 24">
                      <path d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z"/>
                    </svg>
                  {:else if platform.icon === 'tidal'}
                    <svg class="w-6 h-6" fill="white" viewBox="0 0 24 24">
                      <path d="M12.012 3.992L8.008 7.996 4.004 3.992 0 7.996 4.004 12l4.004-4.004L12.012 12l4.004-4.004L12.012 3.992zM12.012 12l-4.004 4.004L12.012 20.008l4.004-4.004L12.012 12zM20.02 7.996L16.016 3.992l-4.004 4.004 4.004 4.004 4.004-4.004L24.024 3.992 20.02 7.996z"/>
                    </svg>
                  {:else if platform.icon === 'deezer'}
                    <svg class="w-6 h-6" fill="{platform.color}" viewBox="0 0 24 24">
                      <path d="M18.81 4.16v3.03H24V4.16h-5.19zM6.27 8.38v3.027h5.189V8.38h-5.19zm12.54 0v3.027H24V8.38h-5.19zM6.27 12.594v3.027h5.189v-3.027h-5.19zm6.271 0v3.027h5.19v-3.027h-5.19zm6.27 0v3.027H24v-3.027h-5.19zM0 16.81v3.028h5.19v-3.027H0zm6.27 0v3.028h5.189v-3.027h-5.19zm6.271 0v3.028h5.19v-3.027h-5.19zm6.27 0v3.028H24v-3.027h-5.19z"/>
                    </svg>
                  {/if}
                </div>
                <div>
                  <div class="flex items-center gap-2">
                    <span class="font-semibold text-white">{platform.name}</span>
                  </div>
                  <div class="flex items-center gap-2 mt-1">
                    <span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium border {getPlatformStatusColor(platform.status)}">
                      {platform.statusLabel}
                    </span>
                    {#if connected}
                      <span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-green-500/20 text-green-400 border border-green-500/30">
                        Connected
                      </span>
                    {:else if needsReconnect}
                      <span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-amber-500/20 text-amber-300 border border-amber-500/30">
                        Reconnect Required
                      </span>
                    {/if}
                  </div>
                </div>
              </div>
            </div>

            <!-- Connection/Sync Status -->
            {#if platform.disabled}
              <div class="text-sm text-zinc-500 italic mb-4">
                Developer portal unavailable — integration paused
              </div>
              <button
                type="button"
                disabled
                class="w-full px-4 py-2.5 rounded-lg text-sm font-medium text-zinc-500 bg-zinc-800 border border-zinc-700 cursor-not-allowed"
              >
                {platform.statusLabel}
              </button>
            {:else if connected}
              <!-- Connected state -->
              <div class="space-y-2 text-sm text-zinc-400 mb-4">
                {#if platform.id === 'apple' && appleLibrary}
                  <div class="flex justify-between">
                    <span>Preview songs:</span>
                    <span class="font-medium text-zinc-300">{appleLibrary.tracksCount.toLocaleString()}</span>
                  </div>
                  <div class="flex justify-between">
                    <span>Preview albums:</span>
                    <span class="font-medium text-zinc-300">{appleLibrary.albumsCount.toLocaleString()}</span>
                  </div>
                  <div class="flex justify-between">
                    <span>Preview playlists:</span>
                    <span class="font-medium text-zinc-300">{appleLibrary.playlistsCount.toLocaleString()}</span>
                  </div>
                  <div class="flex justify-between">
                    <span>Preview refreshed:</span>
                    <span class="font-medium text-zinc-300">{formatDate(appleLibrary.scannedAt)}</span>
                  </div>
                  {#if libraryRow && libraryRow.source === 'imported_cache'}
                    <div class="flex justify-between">
                      <span>Imported items:</span>
                      <span class="font-medium text-zinc-300">{formatMetric(libraryRow.totalItems)}</span>
                    </div>
                    <div class="flex justify-between">
                      <span>Imported songs:</span>
                      <span class="font-medium text-zinc-300">{formatMetric(libraryRow.songs)}</span>
                    </div>
                    <div class="flex justify-between">
                      <span>Imported albums:</span>
                      <span class="font-medium text-zinc-300">{formatMetric(libraryRow.albums)}</span>
                    </div>
                    <div class="flex justify-between">
                      <span>Imported playlists:</span>
                      <span class="font-medium text-zinc-300">{formatMetric(libraryRow.playlists)}</span>
                    </div>
                    <div class="flex justify-between">
                      <span>Imported cache:</span>
                      <span class="font-medium text-zinc-300">{formatDate(libraryRow.lastSynced)}</span>
                    </div>
                  {:else}
                    <div class="rounded-lg border border-amber-500/20 bg-amber-500/10 px-3 py-2 text-xs text-amber-200">
                      Apple preview is loaded, but the cached library is still empty. Use <span class="font-semibold text-white">Sync Library</span> or <span class="font-semibold text-white">Sync All</span> to import items for the sections below.
                    </div>
                  {/if}
                  <button
                    type="button"
                    on:click={async () => {
                      await loadAppleLibraryPreview(true);
                      await refreshLibraryStats();
                    }}
                    class="text-xs text-zinc-400 hover:text-white transition-colors"
                    disabled={appleLibraryLoading}
                  >
                    {appleLibraryLoading ? 'Refreshing library preview...' : 'Refresh library preview'}
                  </button>
                  {#if appleLibrary.playlists.length > 0}
                    <div class="text-xs text-zinc-500 pt-2 border-t border-zinc-700">
                      <div class="mb-1 font-medium text-zinc-400">Playlists</div>
                      <div class="text-zinc-400">{appleLibrary.playlists.slice(0, 3).map(p => p.name).filter(Boolean).join(' • ')}</div>
                    </div>
                  {/if}
                  {#if appleLibrary.tracks.length > 0}
                    <div class="text-xs text-zinc-500">
                      <div class="mb-1 font-medium text-zinc-400">Songs</div>
                      <div class="text-zinc-400">{appleLibrary.tracks.slice(0, 3).map(t => t.name).filter(Boolean).join(' • ')}</div>
                    </div>
                  {/if}
                {:else if libraryRow && libraryRow.status === 'ready'}
                  <div class="flex justify-between">
                    <span>Library songs:</span>
                    <span class="font-medium text-zinc-300">{formatMetric(libraryRow.songs)}</span>
                  </div>
                  <div class="flex justify-between">
                    <span>Library albums:</span>
                    <span class="font-medium text-zinc-300">{formatMetric(libraryRow.albums)}</span>
                  </div>
                  <div class="flex justify-between">
                    <span>Library artists:</span>
                    <span class="font-medium text-zinc-300">{formatMetric(libraryRow.artists)}</span>
                  </div>
                  <div class="flex justify-between">
                    <span>Library playlists:</span>
                    <span class="font-medium text-zinc-300">{formatMetric(libraryRow.playlists)}</span>
                  </div>
                  <div class="flex justify-between">
                    <span>Last synced:</span>
                    <span class="font-medium text-zinc-300">{formatDate(libraryRow.lastSynced)}</span>
                  </div>
                  <div class="flex justify-between">
                    <span>Source:</span>
                    <span class="font-medium text-zinc-300">{getStatsSourceLabel(libraryRow.source)}</span>
                  </div>
                {:else if platform.id === 'apple' && appleLibraryLoading}
                  <div class="text-zinc-500">Loading Apple Music library...</div>
                {:else if platform.id === 'apple' && appleLibraryError}
                  <div class="text-xs text-red-400 bg-red-500/10 rounded p-2 border border-red-500/20">
                    {appleLibraryError}
                  </div>
                {:else if syncStatus}
                  <div class="flex justify-between">
                    <span>Catalog artists synced:</span>
                    <span class="font-medium text-zinc-300">{syncStatus.artists_count?.toLocaleString() ?? 0}</span>
                  </div>
                  <div class="flex justify-between">
                    <span>Last synced:</span>
                    <span class="font-medium text-zinc-300">{formatDate(syncStatus.last_sync)}</span>
                  </div>
                {:else}
                  <div class="text-zinc-500">No library data synced yet</div>
                {/if}
                {#if connection?.created_at || connection?.last_health_check}
                  <div class="pt-2 border-t border-zinc-700 space-y-1">
                    {#if connection?.created_at}
                      <div class="flex justify-between">
                        <span>Connected:</span>
                        <span class="font-medium text-zinc-300">{formatDate(connection.created_at)}</span>
                      </div>
                    {/if}
                    {#if connection?.last_health_check}
                      <div class="flex justify-between">
                        <span>Last check:</span>
                        <span class="font-medium text-zinc-300">{formatDate(connection.last_health_check)}</span>
                      </div>
                    {/if}
                  </div>
                {/if}
                {#if syncStatus?.error_message}
                  <div class="text-xs text-red-400 bg-red-500/10 rounded p-2 border border-red-500/20">
                    {syncStatus.error_message}
                  </div>
                {/if}
              </div>
              <!-- Action buttons -->
              <div class="flex gap-2">
                <button
                  type="button"
                  on:click={() => syncLibrary(platform)}
                  disabled={syncingLibrary === platform.id || $isAnySyncRunning}
                  class="flex-1 px-4 py-2.5 rounded-lg text-sm font-medium text-white transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
                  style="background-color: {platform.color}; opacity: {syncingLibrary === platform.id || $isAnySyncRunning ? 0.5 : 1};"
                >
                  {#if syncingLibrary === platform.id}
                    <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
                    Syncing...
                  {:else}
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                    </svg>
                    Sync Library
                  {/if}
                </button>
                <button
                  type="button"
                  on:click={() => disconnectPlatform(platform)}
                  disabled={connectingPlatform === platform.id}
                  class="px-4 py-2.5 rounded-lg text-sm font-medium text-zinc-400 bg-zinc-800 border border-zinc-700 hover:bg-zinc-700 hover:text-white transition-all disabled:opacity-50"
                >
                  {#if connectingPlatform === platform.id}
                    <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
                  {:else}
                    Disconnect
                  {/if}
                </button>
              </div>
            {:else if platform.status === 'catalog-only'}
              <!-- Catalog only - no connection needed -->
              <div class="text-sm text-zinc-500 mb-4">
                Public API — no account connection required
              </div>
              <button
                type="button"
                on:click={() => navigateTo('home')}
                class="w-full px-4 py-2.5 rounded-lg text-sm font-medium text-zinc-200 bg-zinc-800 border border-zinc-700 hover:bg-zinc-700 transition-colors"
              >
                No account needed &mdash; search from Home
              </button>
            {:else}
              <!-- Not connected state -->
              <div class="text-sm text-zinc-500 mb-4">
                {#if needsReconnect && connection?.error_code}
                  <div class="text-xs text-amber-300 bg-amber-500/10 rounded p-2 border border-amber-500/20">
                    {connection.error_code}
                  </div>
                {:else}
                  Connect your account to sync your playlists and favorites
                {/if}
              </div>
              <button
                type="button"
                on:click={() => connectPlatform(platform)}
                disabled={connectingPlatform === platform.id || $connectionsStore.isLoading}
                class="w-full px-4 py-2.5 rounded-lg text-sm font-medium text-white transition-all disabled:opacity-50 flex items-center justify-center gap-2"
                style="background-color: {platform.color};"
              >
                {#if connectingPlatform === platform.id}
                  <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                  </svg>
                  Connecting...
                {:else if $connectionsStore.isLoading}
                  <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                  </svg>
                  Checking...
                {:else}
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
                  </svg>
                  {needsReconnect ? 'Reconnect Account' : 'Connect Account'}
                {/if}
              </button>
            {/if}
          </div>
        {/each}
      </div>
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
                  <td class="px-4 py-3 font-medium text-white">{row.providerName}</td>
                  <td class="px-4 py-3 text-zinc-300">{formatMetric(row.songs)}</td>
                  <td class="px-4 py-3 text-zinc-300">{formatMetric(row.albums)}</td>
                  <td class="px-4 py-3 text-zinc-300">{formatMetric(row.artists)}</td>
                  <td class="px-4 py-3 text-zinc-300">{formatMetric(row.playlists)}</td>
                  <td class="px-4 py-3 text-zinc-300">{formatMetric(row.totalItems)}</td>
                  <td class="px-4 py-3 text-zinc-300">{formatDate(row.lastSynced)}</td>
                  <td class="px-4 py-3 text-zinc-300">{getStatsSourceLabel(row.source)}</td>
                  <td class="px-4 py-3">
                    {#if row.status === 'ready'}
                      <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium bg-green-500/20 text-green-400">
                        ✓ Ready
                      </span>
                    {:else}
                      <div>
                        <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium bg-red-500/20 text-red-400">
                          ✗ Error
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
        {@const overallGrade = tasteGrade.overall_grade}
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
                <div class="text-3xl font-extrabold leading-none">{tasteGrade.overall_grade}</div>
                <div class="text-xs opacity-80 mt-1">{tasteGrade.overall_score.toFixed(2)}</div>
              </div>
            </div>

            <div class="flex-1">
              <div class="flex items-center justify-between gap-4">
                <div>
                  <div class="text-sm text-zinc-400">Overall</div>
                  <div class="text-lg font-semibold text-white">Music Taste Score</div>
                </div>
                <div class="text-xs text-zinc-500">
                  Computed {formatDate(tasteGrade.computed_at)}
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
                    {#each tasteGrade.components as component (component.id)}
                      <tr class="hover:bg-zinc-800/60">
                        <td class="px-4 py-3 text-white font-medium">{component.label}</td>
                        <td class="px-4 py-3 text-zinc-300">{component.grade}</td>
                        <td class="px-4 py-3 text-zinc-300">{component.score.toFixed(2)}</td>
                        <td class="px-4 py-3 text-zinc-400">{Math.round(component.weight * 100)}%</td>
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
              {#if tasteGrade.signals.length === 0}
                <div class="text-sm text-zinc-500">No signals yet.</div>
              {:else}
                <div class="space-y-1 text-sm text-zinc-300">
                  {#each tasteGrade.signals as s}
                    <div>• {s}</div>
                  {/each}
                </div>
              {/if}
            </div>

            <div class="bg-zinc-950/40 rounded-xl border border-zinc-700 p-4">
              <div class="text-xs font-medium text-zinc-400 uppercase tracking-wide mb-2">Recommendations</div>
              {#if tasteGrade.recommendations.length === 0}
                <div class="text-sm text-zinc-500">Looking good. No recommendations right now.</div>
              {:else}
                <div class="space-y-1 text-sm text-zinc-300">
                  {#each tasteGrade.recommendations as r}
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
      {:else if !libraryOffenders || libraryOffenders.offenders.length === 0}
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
              Top {libraryOffenders.offenders.length} of {libraryOffenders.total_flagged_artists.toLocaleString()} flagged artists
              ({libraryOffenders.total_flagged_tracks.toLocaleString()} {unitLabel} impacted)
            </div>
            <div class="text-xs text-zinc-500">
              Computed {formatDate(libraryOffenders.computed_at)}
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
            {#each libraryOffenders.offenders as offender (offender.id)}
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
                      <span>{offender.track_count.toLocaleString()} {unitLabel} in your library</span>
                      {#if libraryOffenders.playcounts_available}
                        <span class="text-zinc-600">•</span>
                        <span>
                          {offender.play_count !== null ? offender.play_count.toLocaleString() : '--'} plays
                          <span class="text-zinc-500">({playWindowLabel})</span>
                        </span>
                        <span class="text-zinc-600">•</span>
                        <span>{formatCurrency(offender.estimated_revenue)} est. payout</span>
                        {#if offender.percentage_of_user_spend !== null}
                          <span class="text-zinc-600">•</span>
                          <span>{offender.percentage_of_user_spend.toFixed(1)}% of your est. payout</span>
                        {/if}
                      {/if}
                    </div>
                  </div>

                  <span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium border {severityBadgeClass(offender.severity)}">
                    {formatSeverity(offender.severity)}
                  </span>
                </div>

                {#if offender.offenses.length > 0}
                  <div class="mt-3 space-y-1 text-xs text-zinc-300">
                    {#each offender.offenses as o (o.title)}
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
                            <div class="text-xs text-zinc-500">Provider: {getProviderName(group.provider)}</div>
                          {/if}
                        </td>
                        <td class="py-2 pr-4 text-zinc-300 whitespace-nowrap">{group.count.toLocaleString()}</td>
                        <td class="py-2 pr-4 text-zinc-300 whitespace-nowrap">{formatDate(group.last_synced)}</td>
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
              <div class="py-8 text-center">
                <div class="text-zinc-400 mb-2">No matching items yet.</div>
                <div class="text-xs text-zinc-500">
                  Click "Sync Library" above to cache your library items. Apple Music imports your full library, while Tidal/YouTube Music import favorites and playlists.
                </div>
              </div>
            {:else}
              <div class="overflow-x-auto">
                <table class="w-full">
                  <thead class="border-b border-zinc-700">
                    <tr class="text-left text-xs font-medium text-zinc-400 uppercase tracking-wide">
                      <th class="py-2 pr-4">Provider</th>
                      <th class="py-2 pr-4">Title</th>
                      <th class="py-2 pr-4">Artist</th>
                      <th class="py-2 pr-4">Album / Notes</th>
                      <th class="py-2 pr-4">Type</th>
                      <th class="py-2 pr-4">Source</th>
                      <th class="py-2 pr-4">Added</th>
                      <th class="py-2 pr-0">Synced</th>
                    </tr>
                  </thead>
                  <tbody class="border-t border-zinc-700">
                    {#each libraryItems as item (item.id ?? `${item.provider ?? 'unknown'}:${item.provider_track_id ?? 'unknown'}`)}
                      {@const provider = item.provider || 'unknown'}
                      {@const providerName = getProviderName(provider)}
                      {@const kind = kindFromImportedItem(item)}
                      {@const title = item.track_name || item.playlist_name || item.provider_track_id || '(Untitled)'}
                      <tr class="hover:bg-zinc-800">
                        <td class="py-2 pr-4 text-zinc-300 whitespace-nowrap">{providerName}</td>
                        <td class="py-2 pr-4 text-white font-medium max-w-md truncate" title={title}>{title}</td>
                        <td class="py-2 pr-4 text-zinc-300 max-w-xs truncate" title={item.artist_name || ''}>{item.artist_name || '--'}</td>
                        <td class="py-2 pr-4 text-zinc-300 max-w-xs truncate" title={item.album_name || item.playlist_name || ''}>
                          {item.album_name || item.playlist_name || '--'}
                        </td>
                        <td class="py-2 pr-4 text-zinc-300 whitespace-nowrap">{kind}</td>
                        <td class="py-2 pr-4 text-zinc-300 whitespace-nowrap">{item.source_type || '--'}</td>
                        <td class="py-2 pr-4 text-zinc-300 whitespace-nowrap">{item.added_at ? formatDate(item.added_at) : '--'}</td>
                        <td class="py-2 pr-0 text-zinc-300 whitespace-nowrap">{item.last_synced ? formatDate(item.last_synced) : '--'}</td>
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
                    <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium {getStatusColor(run.status)}">
                      {getStatusIcon(run.status)} {run.status}
                    </span>
                  </td>
                  <td class="px-4 py-3 text-zinc-300">
                    {run.artists_processed.toLocaleString()}
                    {#if run.errors_count > 0}
                      <span class="text-red-500 ml-1">({run.errors_count} errors)</span>
                    {/if}
                  </td>
                  <td class="px-4 py-3 text-zinc-300">
                    {formatDuration(run.duration_ms)}
                  </td>
                  <td class="px-4 py-3 text-zinc-300">
                    {formatDate(run.started_at)}
                  </td>
                  <td class="px-4 py-3">
                    {#if run.status === 'running' || run.status === 'pending'}
                      <button
                        type="button"
                        on:click={() => handleCancelRun(run.id)}
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

<!-- Trigger Sync Modal -->
{#if showTriggerModal}
  <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center p-4 z-50" on:click={closeTriggerModal} role="dialog" aria-modal="true">
    <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
    <div class="bg-zinc-900 rounded-2xl max-w-lg w-full p-6 shadow-xl" on:click|stopPropagation role="document">
      <div class="flex items-center mb-6">
        <div class="w-14 h-14 bg-indigo-900/50 rounded-full flex items-center justify-center mr-4">
          <svg class="w-7 h-7 text-indigo-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/><path d="M3.51 9a9 9 0 0114.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0020.49 15"/></svg>
        </div>
        <div>
          <h3 class="text-xl font-bold text-white">Sync Multiple Libraries</h3>
          <p class="text-zinc-400">Select platforms to sync in batch</p>
        </div>
      </div>

      <!-- Platform Selection -->
      <div class="mb-6">
        <label class="block text-sm font-medium text-white mb-3">Platforms</label>
        <div class="grid grid-cols-2 gap-2">
          {#each platforms as platform}
            <button
              type="button"
              on:click={() => !platform.disabled && togglePlatform(platform.id)}
              disabled={platform.disabled}
              class="p-3 rounded-xl border-2 transition-all text-left flex items-center gap-2 {
                platform.disabled
                  ? 'border-zinc-700 bg-zinc-800/50 cursor-not-allowed opacity-50'
                  : selectedPlatforms.includes(platform.id)
                    ? 'border-indigo-500 bg-indigo-900 text-zinc-300'
                    : 'border-zinc-600 hover:border-zinc-500 text-zinc-300'
              }"
            >
              <span class="text-xl font-bold text-zinc-500">{platform.abbr}</span>
              <div class="flex flex-col">
                <span class="font-medium">{platform.name}</span>
                {#if platform.disabled}
                  <span class="text-xs text-amber-400">{platform.statusLabel}</span>
                {/if}
              </div>
            </button>
          {/each}
        </div>
      </div>

      <!-- Sync Type -->
      <div class="mb-6">
        <label class="block text-sm font-medium text-white mb-3">Sync Type</label>
        <div class="grid grid-cols-2 gap-2">
          <button
            type="button"
            on:click={() => syncType = 'incremental'}
            class="p-3 rounded-xl border-2 transition-all text-left text-zinc-300 {
              syncType === 'incremental' ? 'border-indigo-500 bg-indigo-900' : 'border-zinc-600 hover:border-zinc-500'
            }"
          >
            <div class="font-medium">Incremental</div>
            <div class="text-xs text-zinc-400">Only new/changed artists</div>
          </button>
          <button
            type="button"
            on:click={() => syncType = 'full'}
            class="p-3 rounded-xl border-2 transition-all text-left text-zinc-300 {
              syncType === 'full' ? 'border-indigo-500 bg-indigo-900' : 'border-zinc-600 hover:border-zinc-500'
            }"
          >
            <div class="font-medium">Full</div>
            <div class="text-xs text-zinc-400">Complete catalog refresh</div>
          </button>
        </div>
      </div>

      <!-- Priority -->
      <div class="mb-6">
        <label for="priority" class="block text-sm font-medium text-white mb-2">Priority</label>
        <select id="priority" bind:value={priority} class="w-full px-4 py-3 rounded-xl focus:border-indigo-500 focus:ring-2 focus:ring-indigo-200 text-zinc-300 bg-zinc-800 border border-zinc-700" >
          <option value="low">Low</option>
          <option value="normal">Normal</option>
          <option value="high">High</option>
          <option value="critical">Critical</option>
        </select>
      </div>

      <!-- Actions -->
      <div class="flex gap-3">
        <button type="button" on:click={closeTriggerModal} class="flex-1 px-4 py-3 text-white rounded-xl hover:bg-zinc-700 font-medium transition-colors border border-zinc-700" >
          Cancel
        </button>
        <button
          type="button"
          on:click={handleTriggerSync}
          disabled={selectedPlatforms.length === 0 || $syncStore.isTriggering}
          class="flex-1 px-4 py-3 bg-indigo-600 text-white rounded-xl hover:bg-indigo-700 font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {#if $syncStore.isTriggering}
            Starting...
          {:else}
            Start Sync
          {/if}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
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

  .sync-dashboard-page .space-y-2 > * + * {
    margin-top: 0.5rem;
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
