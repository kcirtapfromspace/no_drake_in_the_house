import {
  anyApi,
  convexAction,
  convexMutation,
  convexQuery,
  hasConvexAuth,
  isConvexEnabled,
} from './client';

export interface BridgedApiResponse<T = unknown> {
  success: boolean;
  data?: T;
  message?: string;
  error_code?: string;
  timestamp?: string;
}

function ok<T>(data: T): BridgedApiResponse<T> {
  return {
    success: true,
    data,
    timestamp: new Date().toISOString(),
  };
}

function fail(message: string, errorCode = 'BRIDGE_ERROR'): BridgedApiResponse<never> {
  return {
    success: false,
    message,
    error_code: errorCode,
    timestamp: new Date().toISOString(),
  };
}

async function ensureConvexSession(): Promise<void> {
  if (!isConvexEnabled()) {
    throw new Error('Convex is not configured.');
  }
}

function toUrl(endpoint: string): URL {
  const base = typeof window !== 'undefined' ? window.location.origin : 'https://nodrake.invalid';
  return new URL(endpoint, base);
}

function matchPath(pathname: string, pattern: RegExp): RegExpMatchArray | null {
  return pathname.match(pattern);
}

function mapConvexUser(user: any, linkedAccounts: any[] = []) {
  return {
    id: user?._id ?? '',
    email: user?.email ?? '',
    email_verified: Boolean(user?.emailVerified),
    totp_enabled: Boolean(user?.totpEnabled),
    created_at: user?.createdAt ?? new Date().toISOString(),
    last_login: user?.lastLoginAt,
    oauth_accounts: linkedAccounts,
    display_name: user?.displayName,
    avatar_url: user?.avatarUrl,
    legacy_user_id: user?.legacyUserId,
    roles: user?.roles ?? [],
  };
}

function mapDnpEntry(entry: any) {
  return {
    ...entry,
    id: entry?.artist?.id,
    artist_id: entry?.artist?.id,
  };
}

function mapConnection(record: any) {
  const status =
    record?.status === 'active'
      ? 'active'
      : record?.status === 'expired'
        ? 'needs_reauth'
        : record?.status === 'revoked'
          ? 'error'
          : record?.status ?? 'error';

  return {
    id: record?._id ?? record?.provider,
    provider: record?.provider,
    provider_user_id: record?.providerUserId,
    health_status: status,
    expires_at: record?.expiresAt,
    last_used_at: record?.lastRefreshedAt ?? record?.updatedAt ?? record?.createdAt,
    error_message: record?.errorMessage,
    scopes: Array.isArray(record?.scopes) ? record.scopes : [],
  };
}

async function handleProfile(): Promise<BridgedApiResponse> {
  await convexMutation(anyApi.users.upsertCurrent, {});
  const [user, linkedAccounts] = await Promise.all([
    convexQuery<any>(anyApi.users.current, {}),
    convexQuery<any[]>(anyApi.users.linkedAccounts, {}),
  ]);

  if (!user) {
    return fail('Authenticated user is not available in Convex.', 'HTTP_404');
  }

  return ok(mapConvexUser(user, linkedAccounts));
}

async function handleDnpSearch(url: URL): Promise<BridgedApiResponse> {
  const query = url.searchParams.get('q')?.trim() ?? '';
  const limit = Number(url.searchParams.get('limit') ?? '20');
  const result = await convexQuery<any>(anyApi.dnp.searchArtists, {
    query,
    limit: Number.isFinite(limit) ? limit : 20,
  });
  return ok(result);
}

async function handleGraphSearch(url: URL): Promise<BridgedApiResponse> {
  const query = url.searchParams.get('q')?.trim() ?? '';
  const limit = Number(url.searchParams.get('limit') ?? '20');
  const result = await convexQuery<any>(anyApi.graph.search, {
    query,
    limit: Number.isFinite(limit) ? limit : 20,
  });
  return ok(result);
}

// --- Route handler helpers ---

function parseId(raw: string): string {
  return decodeURIComponent(raw);
}

function parseIntParam(url: URL, name: string, fallback: number): number {
  const val = Number(url.searchParams.get(name) ?? fallback);
  return Number.isFinite(val) ? val : fallback;
}

function isUnauthenticatedRoute(pathname: string): boolean {
  return (
    /^\/api\/v1\/(oauth|connections)\/[^/]+\/(authorize|callback)$/.test(pathname) ||
    pathname === '/api/v1/apple-music/auth/developer-token'
  );
}

export async function maybeHandleConvexRoute<T = unknown>(
  method: 'GET' | 'POST' | 'PUT' | 'DELETE',
  endpoint: string,
  data?: any,
): Promise<BridgedApiResponse<T> | null> {
  if (!isConvexEnabled()) {
    return null;
  }

  const url = toUrl(endpoint);
  const { pathname } = url;

  if (!hasConvexAuth() && !isUnauthenticatedRoute(pathname)) {
    return null;
  }

  try {
    await ensureConvexSession();

    // =============================================
    // Users / Auth
    // =============================================

    if (method === 'GET' && pathname === '/api/v1/users/profile') {
      return (await handleProfile()) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/auth/oauth/accounts') {
      const linkedAccounts = await convexQuery<any[]>(anyApi.users.linkedAccounts, {});
      return ok(linkedAccounts) as BridgedApiResponse<T>;
    }

    // =============================================
    // DNP (Do Not Play)
    // =============================================

    if (method === 'GET' && pathname === '/api/v1/dnp/list') {
      const entries = await convexQuery<any[]>(anyApi.dnp.listCurrentUser, {});
      return ok(entries.map(mapDnpEntry)) as BridgedApiResponse<T>;
    }

    if (method === 'POST' && pathname === '/api/v1/dnp/list') {
      const entry = await convexMutation<any>(anyApi.dnp.addArtistBlock, {
        query: typeof data?.query === 'string' ? data.query : undefined,
        tags: Array.isArray(data?.tags) ? data.tags : [],
        note: typeof data?.note === 'string' ? data.note : undefined,
      });
      return ok(entry ? mapDnpEntry(entry) : entry) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/dnp/search') {
      return (await handleDnpSearch(url)) as BridgedApiResponse<T>;
    }

    if (method === 'POST' && pathname === '/api/v1/dnp/import') {
      const result = await convexMutation<any>(anyApi.dnp.importBlocklist, {
        entries: Array.isArray(data?.entries) ? data.entries : [],
      });
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/dnp/export') {
      const result = await convexQuery<any>(anyApi.dnp.exportBlocklist, {});
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'POST' && pathname === '/api/v1/dnp/tracks') {
      const result = await convexMutation<any>(anyApi.artistProfile.blockTrack, {
        artistId: data?.artistId ?? data?.artist_id,
        trackId: data?.trackId ?? data?.track_id,
        reason: data?.reason,
      });
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'POST' && pathname === '/api/v1/dnp/tracks/batch') {
      const result = await convexMutation<any>(anyApi.artistProfile.batchTrackBlock, {
        artistId: data?.artistId ?? data?.artist_id,
        trackIds: data?.trackIds ?? data?.track_ids ?? [],
        reason: data?.reason,
      });
      return ok(result) as BridgedApiResponse<T>;
    }

    const dnpTrackDeleteMatch = matchPath(pathname, /^\/api\/v1\/dnp\/tracks\/([^/]+)$/);
    if (dnpTrackDeleteMatch && method === 'DELETE') {
      const trackId = parseId(dnpTrackDeleteMatch[1]);
      const result = await convexMutation<any>(anyApi.artistProfile.unblockTrack, { trackId });
      return ok(result) as BridgedApiResponse<T>;
    }

    const dnpEntryMatch = matchPath(pathname, /^\/api\/v1\/dnp\/list\/([^/]+)$/);
    if (dnpEntryMatch && method === 'DELETE') {
      const artistId = parseId(dnpEntryMatch[1]);
      const result = await convexMutation<any>(anyApi.dnp.removeArtistBlock, { artistId });
      return ok(result) as BridgedApiResponse<T>;
    }

    if (dnpEntryMatch && method === 'PUT') {
      const artistId = parseId(dnpEntryMatch[1]);
      const result = await convexMutation<any>(anyApi.dnp.updateArtistBlock, {
        artistId,
        tags: Array.isArray(data?.tags) ? data.tags : [],
        note: typeof data?.note === 'string' ? data.note : undefined,
      });
      return ok(result ? mapDnpEntry(result) : result) as BridgedApiResponse<T>;
    }

    // =============================================
    // Categories
    // =============================================

    if (method === 'GET' && pathname === '/api/v1/categories') {
      const categories = await convexQuery<any[]>(anyApi.categories.list, {});
      return ok(categories) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/categories/blocked-artists') {
      const blockedArtists = await convexQuery<any[]>(anyApi.categories.blockedArtists, {});
      return ok(blockedArtists) as BridgedApiResponse<T>;
    }

    const categorySubscribeMatch = matchPath(
      pathname,
      /^\/api\/v1\/categories\/([^/]+)\/subscribe$/,
    );
    if (categorySubscribeMatch && method === 'POST') {
      const category = parseId(categorySubscribeMatch[1]);
      const result = await convexMutation<any>(anyApi.categories.subscribe, { category });
      return ok(result) as BridgedApiResponse<T>;
    }

    if (categorySubscribeMatch && method === 'DELETE') {
      const category = parseId(categorySubscribeMatch[1]);
      const result = await convexMutation<any>(anyApi.categories.unsubscribe, { category });
      return ok(result) as BridgedApiResponse<T>;
    }

    // =============================================
    // Offenses
    // =============================================

    if (method === 'GET' && pathname === '/api/v1/offenses/query') {
      const artistIdParam = url.searchParams.get('artist_id');
      if (artistIdParam) {
        const result = await convexQuery<any>(anyApi.offenses.listByArtist, {
          artistId: artistIdParam,
        });
        return ok(result) as BridgedApiResponse<T>;
      }
      const category = url.searchParams.get('category') ?? undefined;
      const artists = await convexQuery<any[]>(anyApi.categories.blockedArtists, { category });
      return ok({ artists }) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/offenses') {
      const category = url.searchParams.get('category') ?? undefined;
      const status = url.searchParams.get('status') ?? undefined;
      const limit = parseIntParam(url, 'limit', 20);
      const offset = parseIntParam(url, 'offset', 0);
      const result = await convexQuery<any>(anyApi.offenses.listPaginated, {
        category,
        status,
        limit,
        offset,
      });
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'POST' && pathname === '/api/v1/offenses/submit') {
      const result = await convexMutation<any>(anyApi.offenses.submit, {
        artistId: data?.artistId ?? data?.artist_id,
        category: data?.category,
        severity: data?.severity,
        title: data?.title,
        description: data?.description,
        incidentDate: data?.incidentDate ?? data?.incident_date,
        proceduralState: data?.proceduralState ?? data?.procedural_state,
      });
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'POST' && pathname === '/api/v1/offenses/evidence') {
      const result = await convexMutation<any>(anyApi.offenses.addEvidence, {
        offenseId: data?.offenseId ?? data?.offense_id,
        url: data?.url,
        sourceName: data?.sourceName ?? data?.source_name,
        sourceType: data?.sourceType ?? data?.source_type,
        title: data?.title,
        excerpt: data?.excerpt,
        publishedDate: data?.publishedDate ?? data?.published_date,
        archivedUrl: data?.archivedUrl ?? data?.archived_url,
        credibilityScore: data?.credibilityScore ?? data?.credibility_score,
      });
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'POST' && pathname === '/api/v1/offenses/report-error') {
      const result = await convexMutation<any>(anyApi.artistProfile.reportError, {
        offenseId: data?.offenseId ?? data?.offense_id,
        reason: data?.reason,
        details: data?.details,
      });
      return ok(result) as BridgedApiResponse<T>;
    }

    const offenseContestMatch = matchPath(pathname, /^\/api\/v1\/offenses\/([^/]+)\/contest$/);
    if (offenseContestMatch && method === 'POST') {
      const offenseId = parseId(offenseContestMatch[1]);
      const result = await convexMutation<any>(anyApi.offenses.contestOffense, {
        offenseId,
        reason: data?.reason ?? '',
        reasonCategory: data?.reasonCategory ?? data?.reason_category ?? 'other',
      });
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'POST' && pathname === '/api/v1/offenses/submit-evidence') {
      const result = await convexAction<any>(anyApi.evidenceVerifier.submitEvidence, {
        artistId: data?.artistId ?? data?.artist_id,
        url: data?.url,
        category: data?.category,
      });
      return ok(result) as BridgedApiResponse<T>;
    }

    const offenseByIdMatch = matchPath(pathname, /^\/api\/v1\/offenses\/([^/]+)$/);
    if (offenseByIdMatch && method === 'GET') {
      const offenseId = parseId(offenseByIdMatch[1]);
      const result = await convexQuery<any>(anyApi.offenses.getOne, { offenseId });
      return ok(result) as BridgedApiResponse<T>;
    }

    // =============================================
    // Artist search & profile
    // =============================================

    if (method === 'GET' && pathname === '/api/v1/artists/search') {
      return (await handleDnpSearch(url)) as BridgedApiResponse<T>;
    }

    const artistProfileMatch = matchPath(pathname, /^\/api\/v1\/artists\/([^/]+)\/profile$/);
    if (artistProfileMatch && method === 'GET') {
      const artistId = parseId(artistProfileMatch[1]);
      const result = await convexQuery<any>(anyApi.artistProfile.getProfile, { artistId });
      return ok(result) as BridgedApiResponse<T>;
    }

    const artistAnalyticsMatch = matchPath(pathname, /^\/api\/v1\/artists\/([^/]+)\/analytics$/);
    if (artistAnalyticsMatch && method === 'GET') {
      const artistId = parseId(artistAnalyticsMatch[1]);
      const result = await convexQuery<any>(anyApi.artistProfile.getStreamingMetrics, { artistId });
      return ok(result) as BridgedApiResponse<T>;
    }

    const artistCatalogMatch = matchPath(pathname, /^\/api\/v1\/artists\/([^/]+)\/catalog$/);
    if (artistCatalogMatch && method === 'GET') {
      const artistId = parseId(artistCatalogMatch[1]);
      const result = await convexQuery<any>(anyApi.artistProfile.getCatalog, { artistId });
      return ok(result) as BridgedApiResponse<T>;
    }

    const artistCreditsMatch = matchPath(pathname, /^\/api\/v1\/artists\/([^/]+)\/credits$/);
    if (artistCreditsMatch && method === 'GET') {
      const artistId = parseId(artistCreditsMatch[1]);
      const result = await convexQuery<any>(anyApi.artistProfile.getCredits, { artistId });
      return ok(result) as BridgedApiResponse<T>;
    }

    // =============================================
    // Connections
    // =============================================

    if (method === 'GET' && pathname === '/api/v1/connections') {
      const connections = await convexQuery<any[]>(anyApi.providerConnections.listCurrentUser, {});
      return ok({ connections: connections.map(mapConnection) }) as BridgedApiResponse<T>;
    }

    const connectionMatch = matchPath(pathname, /^\/api\/v1\/connections\/([^/]+)$/);
    if (connectionMatch && method === 'DELETE') {
      const provider = parseId(connectionMatch[1]);
      const result = await convexMutation<any>(anyApi.providerConnections.disconnect, { provider });
      return ok(mapConnection(result)) as BridgedApiResponse<T>;
    }

    // =============================================
    // Graph
    // =============================================

    if (method === 'GET' && pathname === '/api/v1/graph/search') {
      return (await handleGraphSearch(url)) as BridgedApiResponse<T>;
    }

    const graphCollaboratorsMatch = matchPath(
      pathname,
      /^\/api\/v1\/graph\/artists\/([^/]+)\/collaborators$/,
    );
    if (graphCollaboratorsMatch && method === 'GET') {
      const artistId = parseId(graphCollaboratorsMatch[1]);
      const limit = parseIntParam(url, 'limit', 20);
      const result = await convexQuery<any>(anyApi.graph.collaboratorsForArtist, {
        artistId,
        limit,
      });
      return ok(result) as BridgedApiResponse<T>;
    }

    const graphNetworkMatch = matchPath(
      pathname,
      /^\/api\/v1\/graph\/artists\/([^/]+)\/network$/,
    );
    if (graphNetworkMatch && method === 'GET') {
      const artistId = parseId(graphNetworkMatch[1]);
      const depth = parseIntParam(url, 'depth', 2);
      const result = await convexQuery<any>(anyApi.graph.artistNetwork, {
        artistId,
        depth,
      });
      return ok(result) as BridgedApiResponse<T>;
    }

    const graphPathMatch = matchPath(
      pathname,
      /^\/api\/v1\/graph\/artists\/([^/]+)\/path-to\/([^/]+)$/,
    );
    if (graphPathMatch && method === 'GET') {
      const sourceArtistId = parseId(graphPathMatch[1]);
      const targetArtistId = parseId(graphPathMatch[2]);
      const result = await convexQuery<any>(anyApi.graph.pathBetweenArtists, {
        sourceArtistId,
        targetArtistId,
      });
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'POST' && pathname === '/api/v1/graph/blocked-network') {
      const result = await convexQuery<any>(anyApi.graph.blockedNetworkAnalysis, {});
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/graph/health') {
      const result = await convexQuery<any>(anyApi.graph.health, {});
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/graph/stats') {
      const result = await convexQuery<any>(anyApi.graph.stats, {});
      return ok(result) as BridgedApiResponse<T>;
    }

    // Community Lists — UI removed in refactor; backend (community.ts) preserved
    // for future re-enablement. Bridge routes removed to avoid dead code.

    // =============================================
    // Library (Phase 2a)
    // =============================================

    if (method === 'GET' && pathname === '/api/v1/library/scan') {
      const result = await convexQuery<any>(anyApi.library.scanLibrary, {});
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/library/scan/cached') {
      const result = await convexQuery<any>(anyApi.library.scanLibrary, {});
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/library/tracks') {
      const provider = url.searchParams.get('provider') ?? undefined;
      const limit = parseIntParam(url, 'limit', 50);
      const offset = parseIntParam(url, 'offset', 0);
      const result = await convexQuery<any>(anyApi.library.listTracks, { provider, limit, offset });
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/library/stats') {
      const provider = url.searchParams.get('provider');
      if (!provider) return fail('provider query param is required') as BridgedApiResponse<T>;
      const result = await convexQuery<any>(anyApi.library.getLibraryStats, { provider });
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/library/items') {
      const provider = url.searchParams.get('provider') ?? undefined;
      const result = await convexQuery<any>(anyApi.library.listItems, { provider });
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/library/groups') {
      const result = await convexQuery<any>(anyApi.library.listGroups, {});
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/library/taste-grade') {
      const result = await convexQuery<any>(anyApi.library.tasteGrade, {});
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'POST' && pathname === '/api/v1/library/taste-grade/refresh') {
      await convexMutation<any>(anyApi.library.refreshTasteGrade, {});
      return ok({ scheduled: true }) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/library/offenders') {
      const result = await convexQuery<any>(anyApi.library.listOffenders, {});
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/library/playlists/tracks') {
      const provider = url.searchParams.get('provider') ?? 'spotify';
      const playlistName = url.searchParams.get('playlistName') ?? url.searchParams.get('playlist_name') ?? '';
      const result = await convexQuery<any>(anyApi.library.getPlaylistTracks, { provider, playlistName });
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/library/playlists') {
      const provider = url.searchParams.get('provider') ?? undefined;
      const result = await convexQuery<any>(anyApi.library.listPlaylists, { provider });
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'POST' && pathname === '/api/v1/library/import') {
      const result = await convexMutation<any>(anyApi.library.importTracks, {
        provider: data?.provider,
        tracks: data?.tracks ?? [],
      });
      return ok(result) as BridgedApiResponse<T>;
    }

    // =============================================
    // Analytics (Phase 3)
    // =============================================

    if (method === 'GET' && pathname === '/api/v1/analytics/dashboard') {
      const result = await convexQuery<any>(anyApi.analytics.dashboard, {});
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/analytics/dashboard/user-stats') {
      const result = await convexQuery<any>(anyApi.analytics.userStats, {});
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/analytics/health') {
      const result = await convexQuery<any>(anyApi.analytics.systemHealth, {});
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/analytics/trends') {
      const result = await convexQuery<any>(anyApi.analytics.trendSummary, {});
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/analytics/trends/rising') {
      const limit = parseIntParam(url, 'limit', 10);
      const result = await convexQuery<any>(anyApi.analytics.risingArtists, { limit });
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/analytics/trends/falling') {
      const limit = parseIntParam(url, 'limit', 10);
      const result = await convexQuery<any>(anyApi.analytics.fallingArtists, { limit });
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/analytics/reports/types') {
      const result = await convexQuery<any>(anyApi.analytics.reportTypes, {});
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'POST' && pathname === '/api/v1/analytics/reports') {
      const result = await convexMutation<any>(anyApi.analytics.generateReport, {
        type: data?.type,
        parameters: data?.parameters,
      });
      return ok(result) as BridgedApiResponse<T>;
    }

    const reportMatch = matchPath(pathname, /^\/api\/v1\/analytics\/reports\/([^/]+)$/);
    if (reportMatch && method === 'GET') {
      const reportId = parseId(reportMatch[1]);
      const result = await convexQuery<any>(anyApi.analytics.getReport, { reportId });
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/analytics/trouble-scores/leaderboard') {
      const limit = parseIntParam(url, 'limit', 20);
      const result = await convexQuery<any>(anyApi.analytics.troubleLeaderboard, { limit });
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/analytics/trouble-scores/distribution') {
      const result = await convexQuery<any>(anyApi.analytics.troubleDistribution, {});
      return ok(result) as BridgedApiResponse<T>;
    }

    const troubleScoreArtistMatch = matchPath(
      pathname,
      /^\/api\/v1\/analytics\/trouble-scores\/([^/]+)$/,
    );
    if (troubleScoreArtistMatch && method === 'GET') {
      const artistId = parseId(troubleScoreArtistMatch[1]);
      const result = await convexQuery<any>(anyApi.analytics.artistTroubleScore, { artistId });
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/analytics/revenue/distribution') {
      const result = await convexQuery<any>(anyApi.analytics.revenueDistribution, {});
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/analytics/revenue/top-artists') {
      const limit = parseIntParam(url, 'limit', 10);
      const result = await convexQuery<any>(anyApi.analytics.topArtistsByRevenue, { limit });
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/analytics/revenue/problematic') {
      const result = await convexQuery<any>(anyApi.analytics.problematicRevenue, {});
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/analytics/revenue/payout-rates') {
      const result = await convexQuery<any>(anyApi.analytics.payoutRates, {});
      return ok(result) as BridgedApiResponse<T>;
    }

    const artistRevenueMatch = matchPath(
      pathname,
      /^\/api\/v1\/analytics\/revenue\/artist\/([^/]+)$/,
    );
    if (artistRevenueMatch && method === 'GET') {
      const artistId = parseId(artistRevenueMatch[1]);
      const result = await convexQuery<any>(anyApi.analytics.artistRevenue, { artistId });
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/analytics/category-revenue') {
      const result = await convexQuery<any>(anyApi.analytics.globalCategoryRevenue, {});
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/analytics/category-revenue/categories') {
      const result = await convexQuery<any>(anyApi.analytics.offenseCategories, {});
      return ok(result) as BridgedApiResponse<T>;
    }

    const categoryRevenueMatch = matchPath(
      pathname,
      /^\/api\/v1\/analytics\/category-revenue\/([^/]+)$/,
    );
    if (categoryRevenueMatch && method === 'GET') {
      const category = parseId(categoryRevenueMatch[1]);
      // Avoid matching sub-paths like "artist" or "user"
      if (category !== 'artist' && category !== 'user' && category !== 'categories') {
        const result = await convexQuery<any>(anyApi.analytics.categoryRevenue, { category });
        return ok(result) as BridgedApiResponse<T>;
      }
    }

    const artistDiscographyMatch = matchPath(
      pathname,
      /^\/api\/v1\/analytics\/category-revenue\/artist\/([^/]+)\/discography$/,
    );
    if (artistDiscographyMatch && method === 'GET') {
      const artistId = parseId(artistDiscographyMatch[1]);
      const result = await convexQuery<any>(anyApi.analytics.artistDiscography, { artistId });
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/analytics/category-revenue/user/exposure') {
      const result = await convexQuery<any>(anyApi.analytics.userExposure, {});
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/analytics/admin-metrics') {
      const result = await convexQuery<any>(anyApi.analytics.adminMetrics, {});
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/analytics/catalog-metrics-history') {
      const limit = parseIntParam(url, 'limit', 30);
      const result = await convexQuery<any>(anyApi.analytics.catalogMetricsHistory, { limit });
      return ok(result) as BridgedApiResponse<T>;
    }

    // =============================================
    // Sync (Phase 4)
    // =============================================

    if (method === 'GET' && pathname === '/api/v1/sync/status') {
      const result = await convexQuery<any>(anyApi.sync.status, {});
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/sync/runs') {
      const platform = url.searchParams.get('platform') ?? undefined;
      const limit = parseIntParam(url, 'limit', 20);
      const result = await convexQuery<any>(anyApi.sync.listRuns, { platform, limit });
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/sync/health') {
      const result = await convexQuery<any>(anyApi.sync.health, {});
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'POST' && pathname === '/api/v1/sync/trigger') {
      // The sync store sends { platforms: string[] } (plural) but Convex
      // triggerSync expects { platform: string } (singular). Handle both shapes.
      const platform = data?.platform ?? data?.platforms?.[0];
      const result = await convexAction<any>(anyApi.sync.triggerSync, {
        platform,
      });
      return ok(result) as BridgedApiResponse<T>;
    }

    const syncRunMatch = matchPath(pathname, /^\/api\/v1\/sync\/runs\/([^/]+)$/);
    if (syncRunMatch && method === 'GET') {
      const runId = parseId(syncRunMatch[1]);
      const result = await convexQuery<any>(anyApi.sync.getRun, { runId });
      return ok(result) as BridgedApiResponse<T>;
    }

    const syncCancelMatch = matchPath(pathname, /^\/api\/v1\/sync\/runs\/([^/]+)\/cancel$/);
    if (syncCancelMatch && method === 'POST') {
      const runId = parseId(syncCancelMatch[1]);
      const result = await convexMutation<any>(anyApi.sync.cancelRun, { runId });
      return ok(result) as BridgedApiResponse<T>;
    }

    // Provider-specific sync status (replaces Rust backend endpoint)
    const providerSyncStatusMatch = matchPath(pathname, /^\/api\/v1\/connections\/([^/]+)\/library\/sync-status$/);
    if (providerSyncStatusMatch && method === 'GET') {
      const provider = parseId(providerSyncStatusMatch[1]);
      const result = await convexQuery<any>(anyApi.sync.providerSyncStatus, { provider });
      return ok(result) as BridgedApiResponse<T>;
    }

    // Provider-specific sync trigger (both /api/v1/{provider}/library/sync and
    // /api/v1/connections/{provider}/library/sync route to the same Convex action)
    const providerSyncMatch = matchPath(pathname, /^\/api\/v1\/([^/]+)\/library\/sync$/);
    const connectionSyncMatch = matchPath(pathname, /^\/api\/v1\/connections\/([^/]+)\/library\/sync$/);
    const syncMatch = providerSyncMatch || connectionSyncMatch;
    if (syncMatch && method === 'POST') {
      const provider = parseId(syncMatch[1]);
      const result = await convexAction<any>(anyApi.sync.triggerProviderSync, { provider });
      return ok(result) as BridgedApiResponse<T>;
    }

    // =============================================
    // Enforcement (Phase 5)
    // =============================================

    if (method === 'POST' && pathname === '/api/v1/spotify/library/plan') {
      const result = await convexAction<any>(anyApi.enforcement.planEnforcement, {
        providers: data?.providers ?? ['spotify'],
        options: data?.options,
        dryRun: data?.dryRun ?? data?.dry_run ?? true,
      });
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'POST' && pathname === '/api/v1/spotify/enforcement/execute') {
      const result = await convexAction<any>(anyApi.enforcement.executePlan, {
        planId: data?.planId ?? data?.plan_id,
        dryRun: data?.dryRun ?? data?.dry_run,
      });
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'POST' && pathname === '/api/v1/spotify/enforcement/rollback') {
      const result = await convexAction<any>(anyApi.enforcement.rollback, {
        batchId: data?.batchId ?? data?.batch_id,
      });
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/spotify/enforcement/history') {
      const result = await convexQuery<any>(anyApi.enforcement.history, {});
      return ok(result) as BridgedApiResponse<T>;
    }

    const enforcementProgressMatch = matchPath(
      pathname,
      /^\/api\/v1\/spotify\/enforcement\/progress\/([^/]+)$/,
    );
    if (enforcementProgressMatch && method === 'GET') {
      const batchId = parseId(enforcementProgressMatch[1]);
      const result = await convexQuery<any>(anyApi.enforcement.getProgress, { batchId });
      return ok(result) as BridgedApiResponse<T>;
    }

    // =============================================
    // Provider OAuth (Phase 6a)
    // =============================================

    const oauthStatusMatch = matchPath(pathname, /^\/api\/v1\/oauth\/([^/]+)\/status$/);
    if (oauthStatusMatch && method === 'GET') {
      const provider = parseId(oauthStatusMatch[1]);
      const result = await convexQuery<any>(anyApi.providerOAuth.status, { provider });
      return ok(result) as BridgedApiResponse<T>;
    }

    const oauthAuthorizeMatch = matchPath(pathname, /^\/api\/v1\/oauth\/([^/]+)\/authorize$/);
    const connectionAuthorizeMatch = matchPath(pathname, /^\/api\/v1\/connections\/([^/]+)\/authorize$/);
    const authMatch = oauthAuthorizeMatch || connectionAuthorizeMatch;
    if (authMatch && (method === 'POST' || method === 'GET')) {
      const provider = parseId(authMatch[1]);
      if (['spotify', 'tidal', 'youtube'].includes(provider)) {
        const result = await convexAction<any>(anyApi.providerOAuth.authorize, {
          provider,
          redirectUri: data?.redirect_uri ?? data?.redirectUri,
          scopes: data?.scopes,
        });
        return ok(result) as BridgedApiResponse<T>;
      }
    }

    const oauthCallbackMatch = matchPath(pathname, /^\/api\/v1\/oauth\/([^/]+)\/callback$/);
    const connectionCallbackMatch = matchPath(pathname, /^\/api\/v1\/connections\/([^/]+)\/callback$/);
    const cbMatch = oauthCallbackMatch || connectionCallbackMatch;
    if (cbMatch && method === 'POST') {
      const provider = parseId(cbMatch[1]);
      const result = await convexAction<any>(anyApi.providerOAuth.callback, {
        provider,
        code: data?.code,
        state: data?.state,
        redirectUri: data?.redirect_uri ?? data?.redirectUri,
        codeVerifier: data?.codeVerifier ?? data?.code_verifier,
      });
      return ok(result) as BridgedApiResponse<T>;
    }

    // Provider-specific status shortcuts (both /api/v1/{provider}/status and
    // /api/v1/connections/{provider}/status)
    const providerStatusMatch = matchPath(pathname, /^\/api\/v1\/([^/]+)\/status$/);
    const connectionStatusMatch = matchPath(pathname, /^\/api\/v1\/connections\/([^/]+)\/status$/);
    const statusMatch = providerStatusMatch || connectionStatusMatch;
    if (statusMatch && method === 'GET') {
      const provider = parseId(statusMatch[1]);
      if (['spotify', 'tidal', 'youtube'].includes(provider)) {
        const result = await convexQuery<any>(anyApi.providerOAuth.status, { provider });
        return ok(result) as BridgedApiResponse<T>;
      }
    }

    // =============================================
    // Apple Music (Phase 6b)
    // =============================================

    if (method === 'GET' && pathname === '/api/v1/apple-music/auth/developer-token') {
      const result = await convexAction<any>(anyApi.appleMusic.getDeveloperToken, {});
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'POST' && pathname === '/api/v1/apple-music/auth/connect') {
      const result = await convexAction<any>(anyApi.appleMusic.connect, {
        musicUserToken: data?.musicUserToken ?? data?.music_user_token,
      });
      return ok(result) as BridgedApiResponse<T>;
    }

    if ((method === 'POST' || method === 'DELETE') && pathname === '/api/v1/apple-music/auth/disconnect') {
      const result = await convexMutation<any>(anyApi.appleMusic.disconnect, {});
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/apple-music/auth/status') {
      const result = await convexQuery<any>(anyApi.appleMusic.status, {});
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'POST' && pathname === '/api/v1/apple-music/auth/verify') {
      const result = await convexAction<any>(anyApi.appleMusic.verify, {
        musicUserToken: data?.musicUserToken ?? data?.music_user_token,
      });
      return ok(result) as BridgedApiResponse<T>;
    }

    // Apple Music library sync (same action as generic provider sync)
    if (method === 'POST' && pathname === '/api/v1/apple-music/library/sync') {
      const result = await convexAction<any>(anyApi.sync.triggerProviderSync, { provider: 'apple_music' });
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/apple-music/library/sync-status') {
      const result = await convexQuery<any>(anyApi.sync.providerSyncStatus, { provider: 'apple_music' });
      return ok(result) as BridgedApiResponse<T>;
    }

    // =============================================
    // Sanitizer (Phase 7)
    // =============================================

    if (method === 'POST' && pathname === '/api/v1/sanitizer/grade') {
      const result = await convexAction<any>(anyApi.sanitizer.gradePlaylist, {
        provider: data?.provider,
        playlistId: data?.playlistId ?? data?.playlist_id,
      });
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'POST' && pathname === '/api/v1/sanitizer/suggest') {
      const result = await convexAction<any>(anyApi.sanitizer.suggestReplacements, {
        provider: data?.provider,
        playlistId: data?.playlistId ?? data?.playlist_id,
        flaggedTrackIds: data?.flaggedTrackIds ?? data?.flagged_track_ids,
      });
      return ok(result) as BridgedApiResponse<T>;
    }

    const sanitizerPlanMatch = matchPath(pathname, /^\/api\/v1\/sanitizer\/plan\/([^/]+)$/);
    if (sanitizerPlanMatch && method === 'PUT') {
      const planId = parseId(sanitizerPlanMatch[1]);
      const result = await convexMutation<any>(anyApi.sanitizer.updatePlan, {
        planId,
        acceptedReplacements: data?.acceptedReplacements ?? data?.accepted_replacements,
        rejectedReplacements: data?.rejectedReplacements ?? data?.rejected_replacements,
      });
      return ok(result) as BridgedApiResponse<T>;
    }

    const sanitizerPublishMatch = matchPath(pathname, /^\/api\/v1\/sanitizer\/publish\/([^/]+)$/);
    if (sanitizerPublishMatch && method === 'POST') {
      const planId = parseId(sanitizerPublishMatch[1]);
      const result = await convexAction<any>(anyApi.sanitizer.publishPlan, { planId });
      return ok(result) as BridgedApiResponse<T>;
    }

    // =============================================
    // News / Research Pipeline (Phase 5 - read layer)
    // =============================================

    if (method === 'GET' && pathname === '/api/v1/news/articles') {
      const status = url.searchParams.get('status') ?? undefined;
      const sourceType = url.searchParams.get('source_type') ?? url.searchParams.get('sourceType') ?? undefined;
      const limit = parseIntParam(url, 'limit', 20);
      const cursor = url.searchParams.get('cursor') ?? undefined;
      const result = await convexQuery<any>(anyApi.news.listArticles, {
        status,
        sourceType,
        limit,
        cursor,
      });
      return ok(result) as BridgedApiResponse<T>;
    }

    const newsArticleMatch = matchPath(pathname, /^\/api\/v1\/news\/articles\/([^/]+)$/);
    if (newsArticleMatch && method === 'GET') {
      const articleId = parseId(newsArticleMatch[1]);
      const result = await convexQuery<any>(anyApi.news.getArticle, { articleId });
      return ok(result) as BridgedApiResponse<T>;
    }

    const newsArtistMatch = matchPath(pathname, /^\/api\/v1\/news\/artist\/([^/]+)$/);
    if (newsArtistMatch && method === 'GET') {
      const artistId = parseId(newsArtistMatch[1]);
      const limit = parseIntParam(url, 'limit', 20);
      const result = await convexQuery<any>(anyApi.news.listArticlesByArtist, {
        artistId,
        limit,
      });
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/news/classifications') {
      const limit = parseIntParam(url, 'limit', 30);
      const category = url.searchParams.get('category') ?? undefined;
      const result = await convexQuery<any>(anyApi.news.recentClassifications, {
        limit,
        category,
      });
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/news/pipeline/stats') {
      const result = await convexQuery<any>(anyApi.news.pipelineStats, {});
      return ok(result) as BridgedApiResponse<T>;
    }

  } catch (error) {
    const message = error instanceof Error ? error.message : 'Convex bridge request failed.';
    return fail(message, 'HTTP_500') as BridgedApiResponse<T>;
  }

  return null;
}
