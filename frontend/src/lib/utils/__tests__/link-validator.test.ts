/**
 * Link Validator Tests
 * Tests for URL validation, Wayback Machine fallback, caching, and batch validation
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { validateLink, validateLinks, clearLinkCache } from '../link-validator';
import type { LinkCheckResult as _LinkCheckResult } from '../link-validator';

describe('Link Validator', () => {
  let fetchMock: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    fetchMock = vi.fn();
    globalThis.fetch = fetchMock as unknown as typeof globalThis.fetch;
    clearLinkCache();
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.restoreAllMocks();
  });

  describe('validateLink', () => {
    it('should return valid for reachable URLs', async () => {
      // First call: HEAD probe (no-cors) succeeds
      fetchMock.mockResolvedValueOnce({
        type: 'opaque',
        ok: false,
        status: 0,
      });

      const result = await validateLink('https://example.com/article');

      expect(result.status).toBe('valid');
      expect(result.originalUrl).toBe('https://example.com/article');
      expect(result.resolvedUrl).toBe('https://example.com/article');
    });

    it('should return valid for CORS-enabled URLs with ok response', async () => {
      fetchMock.mockResolvedValueOnce({
        type: 'cors',
        ok: true,
        status: 200,
      });

      const result = await validateLink('https://example.com/article');

      expect(result.status).toBe('valid');
    });

    it('should accept 301/302 redirects as valid', async () => {
      fetchMock.mockResolvedValueOnce({
        type: 'cors',
        ok: false,
        status: 301,
      });

      const result = await validateLink('https://example.com/old-article');

      expect(result.status).toBe('valid');
    });

    it('should use existing archived URL when original is broken', async () => {
      // HEAD probe fails (network error)
      fetchMock.mockRejectedValueOnce(new Error('Network error'));

      const result = await validateLink(
        'https://deadlink.example.com/article',
        'https://web.archive.org/web/2024/https://deadlink.example.com/article',
      );

      expect(result.status).toBe('archived');
      expect(result.resolvedUrl).toBe(
        'https://web.archive.org/web/2024*/https://deadlink.example.com/article',
      );
      expect(result.archivedUrl).toBe(
        'https://web.archive.org/web/2024*/https://deadlink.example.com/article',
      );
    });

    it('should query Wayback Machine when URL is broken and no existing archive', async () => {
      // HEAD probe fails
      fetchMock.mockRejectedValueOnce(new Error('Network error'));

      // Wayback Machine API returns snapshot
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: () =>
          Promise.resolve({
            archived_snapshots: {
              closest: {
                available: true,
                url: 'http://web.archive.org/web/20240101/https://broken.example.com/page',
              },
            },
          }),
      });

      const result = await validateLink('https://broken.example.com/page');

      expect(result.status).toBe('archived');
      // Should convert http to https
      expect(result.resolvedUrl).toBe(
        'https://web.archive.org/web/20240101/https://broken.example.com/page',
      );
      expect(result.archivedUrl).toBe(
        'https://web.archive.org/web/20240101/https://broken.example.com/page',
      );
    });

    it('should return broken when URL is dead and no Wayback snapshot exists', async () => {
      // HEAD probe fails
      fetchMock.mockRejectedValueOnce(new Error('Network error'));

      // Wayback Machine returns no snapshot
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: () =>
          Promise.resolve({
            archived_snapshots: {},
          }),
      });

      const result = await validateLink('https://gone-forever.example.com/page');

      expect(result.status).toBe('broken');
      expect(result.resolvedUrl).toBe('https://gone-forever.example.com/page');
      expect(result.archivedUrl).toBeUndefined();
    });

    it('should return broken when Wayback Machine API fails', async () => {
      // HEAD probe fails
      fetchMock.mockRejectedValueOnce(new Error('Network error'));

      // Wayback Machine API errors
      fetchMock.mockRejectedValueOnce(new Error('API unavailable'));

      const result = await validateLink('https://broken.example.com/page');

      expect(result.status).toBe('broken');
    });

    it('should return broken for empty URL', async () => {
      const result = await validateLink('');

      expect(result.status).toBe('broken');
      expect(result.resolvedUrl).toBe('');
    });

    it('should cache results and return cached on subsequent calls', async () => {
      fetchMock.mockResolvedValueOnce({ type: 'opaque', ok: false, status: 0 });

      const result1 = await validateLink('https://cached.example.com');
      expect(result1.status).toBe('valid');

      // Second call should not trigger another fetch
      const result2 = await validateLink('https://cached.example.com');
      expect(result2.status).toBe('valid');

      // Only 1 fetch call (the HEAD probe), not 2
      expect(fetchMock).toHaveBeenCalledTimes(1);
    });

    it('should preserve archivedUrl on valid links when provided', async () => {
      fetchMock.mockResolvedValueOnce({ type: 'opaque', ok: false, status: 0 });

      const result = await validateLink(
        'https://valid.example.com',
        'https://web.archive.org/web/2024/https://valid.example.com',
      );

      expect(result.status).toBe('valid');
      expect(result.archivedUrl).toBe(
        'https://web.archive.org/web/2024*/https://valid.example.com',
      );
    });

    it('should handle fetch timeout (abort)', async () => {
      // Simulate abort
      fetchMock.mockRejectedValueOnce(new DOMException('Aborted', 'AbortError'));

      // No Wayback snapshot
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve({ archived_snapshots: {} }),
      });

      const result = await validateLink('https://slow.example.com');

      expect(result.status).toBe('broken');
    });
  });

  describe('validateLinks (batch)', () => {
    it('should validate multiple URLs in parallel', async () => {
      // URL 1: valid
      fetchMock.mockResolvedValueOnce({ type: 'opaque', ok: false, status: 0 });
      // URL 2: valid
      fetchMock.mockResolvedValueOnce({ type: 'opaque', ok: false, status: 0 });
      // URL 3: valid
      fetchMock.mockResolvedValueOnce({ type: 'opaque', ok: false, status: 0 });

      const urls = [
        { url: 'https://a.example.com' },
        { url: 'https://b.example.com' },
        { url: 'https://c.example.com' },
      ];

      const results = await validateLinks(urls);

      expect(results.size).toBe(3);
      expect(results.get('https://a.example.com')?.status).toBe('valid');
      expect(results.get('https://b.example.com')?.status).toBe('valid');
      expect(results.get('https://c.example.com')?.status).toBe('valid');
    });

    it('should return empty map for empty input', async () => {
      const results = await validateLinks([]);
      expect(results.size).toBe(0);
    });

    it('should handle mixed valid/broken/archived URLs', async () => {
      // Use URL-based mock to handle parallel execution ordering
      fetchMock.mockImplementation((input: any, options: any) => {
        const url = typeof input === 'string' ? input : input.url;

        // HEAD probes (no-cors)
        if (options?.method === 'HEAD') {
          if (url === 'https://valid.example.com') {
            return Promise.resolve({ type: 'opaque', ok: false, status: 0 });
          }
          // broken + gone both fail
          return Promise.reject(new Error('Network error'));
        }

        // Wayback API calls
        if (url.includes('archive.org/wayback')) {
          if (url.includes('broken.example.com')) {
            return Promise.resolve({
              ok: true,
              json: () =>
                Promise.resolve({
                  archived_snapshots: {
                    closest: {
                      available: true,
                      url: 'https://web.archive.org/web/2024/https://broken.example.com',
                    },
                  },
                }),
            });
          }
          // gone.example.com → no archive
          return Promise.resolve({
            ok: true,
            json: () => Promise.resolve({ archived_snapshots: {} }),
          });
        }

        return Promise.reject(new Error('Unexpected fetch'));
      });

      const urls = [
        { url: 'https://valid.example.com' },
        { url: 'https://broken.example.com' },
        { url: 'https://gone.example.com' },
      ];

      const results = await validateLinks(urls);

      expect(results.get('https://valid.example.com')?.status).toBe('valid');
      expect(results.get('https://broken.example.com')?.status).toBe('archived');
      expect(results.get('https://gone.example.com')?.status).toBe('broken');
    });

    it('should pass through existing archivedUrl to validator', async () => {
      // HEAD probe fails
      fetchMock.mockRejectedValueOnce(new Error('Network error'));

      const urls = [
        {
          url: 'https://dead.example.com',
          archivedUrl: 'https://web.archive.org/saved-copy',
        },
      ];

      const results = await validateLinks(urls);
      const result = results.get('https://dead.example.com');

      expect(result?.status).toBe('archived');
      expect(result?.resolvedUrl).toBe('https://web.archive.org/saved-copy');
    });

    it('should respect concurrency limit of 4', async () => {
      // Create 6 URLs - should process in 2 batches of 4 + 2
      const urls = Array.from({ length: 6 }, (_, i) => ({
        url: `https://example${i}.com`,
      }));

      // All succeed
      for (let i = 0; i < 6; i++) {
        fetchMock.mockResolvedValueOnce({ type: 'opaque', ok: false, status: 0 });
      }

      const results = await validateLinks(urls);
      expect(results.size).toBe(6);
    });
  });

  describe('Wayback Machine URL normalization', () => {
    it('should convert http Wayback URLs to https', async () => {
      fetchMock.mockRejectedValueOnce(new Error('Network error'));
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: () =>
          Promise.resolve({
            archived_snapshots: {
              closest: {
                available: true,
                url: 'http://web.archive.org/web/20240101/https://example.com',
              },
            },
          }),
      });

      const result = await validateLink('https://example.com');
      expect(result.archivedUrl).toMatch(/^https:\/\//);
    });

    it('should handle Wayback snapshot with available=false', async () => {
      fetchMock.mockRejectedValueOnce(new Error('Network error'));
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: () =>
          Promise.resolve({
            archived_snapshots: {
              closest: {
                available: false,
                url: 'http://web.archive.org/web/20240101/https://example.com',
              },
            },
          }),
      });

      const result = await validateLink('https://example.com');
      expect(result.status).toBe('broken');
    });

    it('should handle non-200 Wayback API response', async () => {
      fetchMock.mockRejectedValueOnce(new Error('Network error'));
      fetchMock.mockResolvedValueOnce({
        ok: false,
        status: 503,
      });

      const result = await validateLink('https://example.com');
      expect(result.status).toBe('broken');
    });
  });
});
