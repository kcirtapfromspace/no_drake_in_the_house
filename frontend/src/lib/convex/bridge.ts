import { anyApi, convexMutation, convexQuery, hasConvexAuth, isConvexEnabled } from './client';

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

export async function maybeHandleConvexRoute<T = unknown>(
  method: 'GET' | 'POST' | 'PUT' | 'DELETE',
  endpoint: string,
  data?: any,
): Promise<BridgedApiResponse<T> | null> {
  if (!isConvexEnabled() || !hasConvexAuth()) {
    return null;
  }

  const url = toUrl(endpoint);
  const { pathname } = url;

  try {
    await ensureConvexSession();

    if (method === 'GET' && pathname === '/api/v1/users/profile') {
      return (await handleProfile()) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/auth/oauth/accounts') {
      const linkedAccounts = await convexQuery<any[]>(anyApi.users.linkedAccounts, {});
      return ok(linkedAccounts) as BridgedApiResponse<T>;
    }

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

    const dnpEntryMatch = matchPath(pathname, /^\/api\/v1\/dnp\/list\/([^/]+)$/);
    if (dnpEntryMatch && method === 'DELETE') {
      const artistId = decodeURIComponent(dnpEntryMatch[1]);
      const result = await convexMutation<any>(anyApi.dnp.removeArtistBlock, {
        artistId,
      });
      return ok(result) as BridgedApiResponse<T>;
    }

    if (dnpEntryMatch && method === 'PUT') {
      const artistId = decodeURIComponent(dnpEntryMatch[1]);
      const result = await convexMutation<any>(anyApi.dnp.updateArtistBlock, {
        artistId,
        tags: Array.isArray(data?.tags) ? data.tags : [],
        note: typeof data?.note === 'string' ? data.note : undefined,
      });
      return ok(result ? mapDnpEntry(result) : result) as BridgedApiResponse<T>;
    }

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
      const category = decodeURIComponent(categorySubscribeMatch[1]);
      const result = await convexMutation<any>(anyApi.categories.subscribe, { category });
      return ok(result) as BridgedApiResponse<T>;
    }

    if (categorySubscribeMatch && method === 'DELETE') {
      const category = decodeURIComponent(categorySubscribeMatch[1]);
      const result = await convexMutation<any>(anyApi.categories.unsubscribe, { category });
      return ok(result) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/offenses/query') {
      const category = url.searchParams.get('category') ?? undefined;
      const artists = await convexQuery<any[]>(anyApi.categories.blockedArtists, { category });
      return ok({ artists }) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/connections') {
      const connections = await convexQuery<any[]>(anyApi.providerConnections.listCurrentUser, {});
      return ok({ connections: connections.map(mapConnection) }) as BridgedApiResponse<T>;
    }

    const connectionMatch = matchPath(pathname, /^\/api\/v1\/connections\/([^/]+)$/);
    if (connectionMatch && method === 'DELETE') {
      const provider = decodeURIComponent(connectionMatch[1]);
      const result = await convexMutation<any>(anyApi.providerConnections.disconnect, {
        provider,
      });
      return ok(mapConnection(result)) as BridgedApiResponse<T>;
    }

    if (method === 'GET' && pathname === '/api/v1/graph/search') {
      return (await handleGraphSearch(url)) as BridgedApiResponse<T>;
    }

    const graphCollaboratorsMatch = matchPath(
      pathname,
      /^\/api\/v1\/graph\/artists\/([^/]+)\/collaborators$/,
    );
    if (graphCollaboratorsMatch && method === 'GET') {
      const artistId = decodeURIComponent(graphCollaboratorsMatch[1]);
      const limit = Number(url.searchParams.get('limit') ?? '20');
      const result = await convexQuery<any>(anyApi.graph.collaboratorsForArtist, {
        artistId,
        limit: Number.isFinite(limit) ? limit : 20,
      });
      return ok(result) as BridgedApiResponse<T>;
    }

    const graphNetworkMatch = matchPath(
      pathname,
      /^\/api\/v1\/graph\/artists\/([^/]+)\/network$/,
    );
    if (graphNetworkMatch && method === 'GET') {
      const artistId = decodeURIComponent(graphNetworkMatch[1]);
      const depth = Number(url.searchParams.get('depth') ?? '2');
      const result = await convexQuery<any>(anyApi.graph.artistNetwork, {
        artistId,
        depth: Number.isFinite(depth) ? depth : 2,
      });
      return ok(result) as BridgedApiResponse<T>;
    }

    const graphPathMatch = matchPath(
      pathname,
      /^\/api\/v1\/graph\/artists\/([^/]+)\/path-to\/([^/]+)$/,
    );
    if (graphPathMatch && method === 'GET') {
      const sourceArtistId = decodeURIComponent(graphPathMatch[1]);
      const targetArtistId = decodeURIComponent(graphPathMatch[2]);
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
  } catch (error) {
    const message = error instanceof Error ? error.message : 'Convex bridge request failed.';
    return fail(message, 'HTTP_500') as BridgedApiResponse<T>;
  }

  return null;
}
