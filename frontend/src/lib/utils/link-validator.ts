/**
 * Link validator with Wayback Machine fallback.
 * Checks evidence source URLs and resolves archived versions for broken links.
 */

export type LinkStatus = 'valid' | 'broken' | 'archived' | 'checking';

export interface LinkCheckResult {
  originalUrl: string;
  status: LinkStatus;
  resolvedUrl: string; // The URL to actually navigate to
  archivedUrl?: string;
  checkedAt: number;
}

// Cache validated links to avoid re-checking
const linkCache = new Map<string, LinkCheckResult>();

// In-flight requests to avoid duplicate checks
const pendingChecks = new Map<string, Promise<LinkCheckResult>>();

const CACHE_TTL_MS = 30 * 60 * 1000; // 30 minutes

/**
 * Fix malformed Wayback Machine URLs.
 * `web/2024/` (bare year) is invalid — needs `web/2024* /` (wildcard) to
 * auto-redirect to the closest snapshot.
 */
function fixArchiveUrl(url: string): string {
  return url.replace(
    /web\.archive\.org\/web\/(\d{4})\//,
    'web.archive.org/web/$1*/',
  );
}

/** Clear the internal link cache — useful for testing. */
export function clearLinkCache(): void {
  linkCache.clear();
  pendingChecks.clear();
}

/**
 * Check if a URL is reachable via a lightweight HEAD/fetch probe.
 * We use opaque fetch mode as a signal — if the request doesn't throw
 * a network error, the server responded (even if CORS blocks reading it).
 */
async function probeUrl(url: string): Promise<boolean> {
  try {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 8000);
    const response = await fetch(url, {
      method: 'HEAD',
      mode: 'no-cors',
      signal: controller.signal,
    });
    clearTimeout(timeoutId);
    // In no-cors mode, opaque responses have type 'opaque' and status 0.
    // A network error would throw, so reaching here means the server responded.
    // For same-origin or CORS-enabled endpoints we can check status directly.
    if (response.type === 'opaque') return true;
    return response.ok || response.status === 301 || response.status === 302;
  } catch {
    return false;
  }
}

/**
 * Query the Wayback Machine Availability API for an archived snapshot.
 * Returns the archived URL if available, or null.
 */
async function findWaybackSnapshot(url: string): Promise<string | null> {
  try {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 10000);
    const apiUrl = `https://archive.org/wayback/available?url=${encodeURIComponent(url)}`;
    const response = await fetch(apiUrl, { signal: controller.signal });
    clearTimeout(timeoutId);

    if (!response.ok) return null;

    const data = await response.json();
    const snapshot = data?.archived_snapshots?.closest;
    if (snapshot?.available && snapshot?.url) {
      // Ensure HTTPS
      return snapshot.url.replace(/^http:/, 'https:');
    }
    return null;
  } catch {
    return null;
  }
}

/**
 * Validate a single evidence URL. Returns cached result if available.
 * If the URL is broken, attempts to find a Wayback Machine archive.
 */
export async function validateLink(
  url: string,
  existingArchivedUrl?: string,
): Promise<LinkCheckResult> {
  if (!url) {
    return {
      originalUrl: url,
      status: 'broken',
      resolvedUrl: '',
    } as LinkCheckResult;
  }

  // Check cache first
  const cached = linkCache.get(url);
  if (cached && Date.now() - cached.checkedAt < CACHE_TTL_MS) {
    return cached;
  }

  // De-duplicate in-flight requests
  const pending = pendingChecks.get(url);
  if (pending) return pending;

  const check = (async (): Promise<LinkCheckResult> => {
    // Fix malformed archive URLs and normalize the known archived fallback
    const knownArchive = existingArchivedUrl ? fixArchiveUrl(existingArchivedUrl) : null;

    // If this URL is itself an archive.org URL, fix its format and use directly
    if (url.includes('web.archive.org/web/')) {
      const fixedUrl = fixArchiveUrl(url);
      const result: LinkCheckResult = {
        originalUrl: url,
        status: 'archived',
        resolvedUrl: fixedUrl,
        archivedUrl: fixedUrl,
        checkedAt: Date.now(),
      };
      linkCache.set(url, result);
      return result;
    }

    const isReachable = await probeUrl(url);

    if (isReachable) {
      const result: LinkCheckResult = {
        originalUrl: url,
        status: 'valid',
        resolvedUrl: url,
        archivedUrl: knownArchive || undefined,
        checkedAt: Date.now(),
      };
      linkCache.set(url, result);
      return result;
    }

    // URL is broken — try known archived URL first
    if (knownArchive) {
      const result: LinkCheckResult = {
        originalUrl: url,
        status: 'archived',
        resolvedUrl: knownArchive,
        archivedUrl: knownArchive,
        checkedAt: Date.now(),
      };
      linkCache.set(url, result);
      return result;
    }

    // Try Wayback Machine
    const waybackUrl = await findWaybackSnapshot(url);
    if (waybackUrl) {
      const result: LinkCheckResult = {
        originalUrl: url,
        status: 'archived',
        resolvedUrl: waybackUrl,
        archivedUrl: waybackUrl,
        checkedAt: Date.now(),
      };
      linkCache.set(url, result);
      return result;
    }

    // Truly broken — no archive found
    const result: LinkCheckResult = {
      originalUrl: url,
      status: 'broken',
      resolvedUrl: url,
      checkedAt: Date.now(),
    };
    linkCache.set(url, result);
    return result;
  })();

  pendingChecks.set(url, check);
  try {
    return await check;
  } finally {
    pendingChecks.delete(url);
  }
}

/**
 * Batch-validate multiple URLs. Returns a Map of url → result.
 * Runs checks in parallel with a concurrency limit.
 */
export async function validateLinks(
  urls: { url: string; archivedUrl?: string }[],
): Promise<Map<string, LinkCheckResult>> {
  const results = new Map<string, LinkCheckResult>();
  const CONCURRENCY = 4;

  for (let i = 0; i < urls.length; i += CONCURRENCY) {
    const batch = urls.slice(i, i + CONCURRENCY);
    const batchResults = await Promise.all(
      batch.map((entry) => validateLink(entry.url, entry.archivedUrl)),
    );
    for (const result of batchResults) {
      results.set(result.originalUrl, result);
    }
  }

  return results;
}
