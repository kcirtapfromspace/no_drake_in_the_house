import { ConvexError, v } from "convex/values";
import { action, internalAction, internalMutation, internalQuery } from "./_generated/server";
import { internal } from "./_generated/api";
import {
  decryptToken,
  encryptToken,
  getEncryptionKey,
  isEncrypted,
} from "./lib/crypto";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

interface TrackImport {
  providerTrackId: string;
  trackName?: string;
  albumName?: string;
  artistName?: string;
  sourceType?: string;
  playlistName?: string;
}

interface CheckpointData {
  phase: "liked" | "albums" | "artists" | "playlists" | "playlist_tracks" | "done";
  offset?: number;
  artistCursor?: string;
  playlistIndex?: number;
  playlistTrackOffset?: number;
  playlistIds?: string[];
  playlistNames?: string[];
  tracksImported: number;
  likedCount: number;
  albumCount: number;
  artistCount: number;
  playlistTrackCount: number;
  clearedExisting: boolean;
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const BATCH_SIZE = 200;
const SPOTIFY_PAGE_SIZE = 50;
const PLAYLIST_TRACK_PAGE_SIZE = 100;
const MAX_RETRIES = 3;
const MAX_RETRY_WAIT_SECS = 60;
// Leave a 5-minute buffer before the 30-minute Convex action limit
const SAFE_RUNTIME_MS = 25 * 60 * 1000;

// ---------------------------------------------------------------------------
// Internal queries / mutations used by sync actions
// ---------------------------------------------------------------------------

/**
 * Fetch the encrypted access + refresh tokens for a user+provider connection.
 * Runs without user auth context so the scheduled action can access it.
 */
export const _getConnectionTokens = internalQuery({
  args: {
    userId: v.id("users"),
    provider: v.string(),
  },
  handler: async (ctx, args) => {
    const connection = await ctx.db
      .query("providerConnections")
      .withIndex("by_user_provider", (q) =>
        q.eq("userId", args.userId).eq("provider", args.provider),
      )
      .unique();

    if (!connection || connection.status !== "active") {
      return null;
    }

    return {
      connectionId: connection._id,
      encryptedAccessToken: connection.encryptedAccessToken ?? null,
      encryptedRefreshToken: connection.encryptedRefreshToken ?? null,
      expiresAt: connection.expiresAt ?? null,
    };
  },
});

/**
 * Batch-import tracks into userLibraryTracks without requiring user auth
 * context. Designed to be called from scheduled sync actions.
 */
export const _batchImportTracks = internalMutation({
  args: {
    userId: v.id("users"),
    provider: v.string(),
    tracks: v.array(
      v.object({
        providerTrackId: v.string(),
        trackName: v.optional(v.string()),
        albumName: v.optional(v.string()),
        artistName: v.optional(v.string()),
        sourceType: v.optional(v.string()),
        playlistName: v.optional(v.string()),
      }),
    ),
  },
  handler: async (ctx, args) => {
    const now = new Date().toISOString();
    let imported = 0;

    for (const track of args.tracks) {
      const legacyKey = `runtime:track:${args.userId}:${args.provider}:${track.providerTrackId}`;
      const existing = await ctx.db
        .query("userLibraryTracks")
        .withIndex("by_legacyKey", (q) => q.eq("legacyKey", legacyKey))
        .unique();

      if (!existing) {
        await ctx.db.insert("userLibraryTracks", {
          legacyKey,
          userId: args.userId,
          provider: args.provider,
          providerTrackId: track.providerTrackId,
          trackName: track.trackName,
          albumName: track.albumName,
          artistName: track.artistName,
          sourceType: track.sourceType,
          playlistName: track.playlistName,
          lastSyncedAt: now,
          metadata: {},
          createdAt: now,
          updatedAt: now,
        });
        imported++;
      }
    }

    return { imported };
  },
});

/**
 * Delete all userLibraryTracks for a given user + provider.
 * Called once at the start of a full sync.
 */
export const _clearProviderTracks = internalMutation({
  args: {
    userId: v.id("users"),
    provider: v.string(),
  },
  handler: async (ctx, args) => {
    const tracks = await ctx.db
      .query("userLibraryTracks")
      .withIndex("by_user_provider", (q) =>
        q.eq("userId", args.userId).eq("provider", args.provider),
      )
      .collect();

    let deleted = 0;
    for (const track of tracks) {
      await ctx.db.delete(track._id);
      deleted++;
    }

    return { deleted };
  },
});

/**
 * Update a platformSyncRuns record with checkpoint data, stats, or completion.
 */
export const _updateSyncRun = internalMutation({
  args: {
    runId: v.id("platformSyncRuns"),
    status: v.optional(v.string()),
    checkpointData: v.optional(v.any()),
    metadata: v.optional(v.any()),
    completedAt: v.optional(v.string()),
    errorLog: v.optional(v.array(v.any())),
  },
  handler: async (ctx, args) => {
    const run = await ctx.db.get(args.runId);
    if (!run) return;

    const patch: Record<string, any> = {
      updatedAt: new Date().toISOString(),
    };

    if (args.status !== undefined) patch.status = args.status;
    if (args.checkpointData !== undefined) patch.checkpointData = args.checkpointData;
    if (args.metadata !== undefined) patch.metadata = args.metadata;
    if (args.completedAt !== undefined) patch.completedAt = args.completedAt;
    if (args.errorLog !== undefined) patch.errorLog = args.errorLog;

    await ctx.db.patch(run._id, patch);
  },
});

/**
 * Read the current state of a sync run (to check if it was cancelled).
 */
export const _getSyncRun = internalQuery({
  args: {
    runId: v.id("platformSyncRuns"),
  },
  handler: async (ctx, args) => {
    const run = await ctx.db.get(args.runId);
    if (!run) return null;
    return {
      status: run.status,
      checkpointData: run.checkpointData,
    };
  },
});

/**
 * Persist refreshed access token from within the sync action.
 */
export const _updateConnectionTokenFromSync = internalMutation({
  args: {
    connectionId: v.id("providerConnections"),
    encryptedAccessToken: v.string(),
    expiresAt: v.optional(v.string()),
  },
  handler: async (ctx, args) => {
    await ctx.db.patch(args.connectionId, {
      encryptedAccessToken: args.encryptedAccessToken,
      expiresAt: args.expiresAt,
      updatedAt: new Date().toISOString(),
    });
  },
});

// ---------------------------------------------------------------------------
// Spotify HTTP helper with retry + rate-limit handling
// ---------------------------------------------------------------------------

async function spotifyFetchWithRetry(
  url: string,
  token: string,
): Promise<Response> {
  for (let attempt = 0; attempt <= MAX_RETRIES; attempt++) {
    const res = await fetch(url, {
      headers: {
        Authorization: `Bearer ${token}`,
        Accept: "application/json",
      },
    });

    if (res.status === 429) {
      if (attempt === MAX_RETRIES) {
        throw new Error(`Spotify rate limited after ${MAX_RETRIES} retries on ${url}`);
      }
      const retryAfter = res.headers.get("retry-after");
      const waitSecs = Math.min(
        parseInt(retryAfter || "2", 10) || 2,
        MAX_RETRY_WAIT_SECS,
      );
      await new Promise((r) => setTimeout(r, waitSecs * 1000));
      continue;
    }

    return res;
  }
  throw new Error("spotifyFetchWithRetry: unreachable");
}

// ---------------------------------------------------------------------------
// Token helpers
// ---------------------------------------------------------------------------

async function decryptAccessToken(
  encryptedToken: string,
): Promise<string> {
  const encryptionKey = getEncryptionKey();
  return isEncrypted(encryptedToken)
    ? await decryptToken(encryptedToken, encryptionKey)
    : encryptedToken;
}

async function decryptRefreshTokenValue(
  encryptedToken: string | null,
): Promise<string | undefined> {
  if (!encryptedToken) return undefined;
  const encryptionKey = getEncryptionKey();
  return isEncrypted(encryptedToken)
    ? await decryptToken(encryptedToken, encryptionKey)
    : encryptedToken;
}

/**
 * Attempt to refresh a Spotify access token using the refresh token.
 * Returns the new plaintext access token and expiry, or throws.
 */
async function refreshSpotifyAccessToken(
  refreshToken: string,
): Promise<{ accessToken: string; expiresIn: number }> {
  const clientId = process.env.SPOTIFY_CLIENT_ID;
  const clientSecret = process.env.SPOTIFY_CLIENT_SECRET;
  if (!clientId || !clientSecret) {
    throw new Error("Spotify OAuth credentials not configured.");
  }

  const body = new URLSearchParams({
    grant_type: "refresh_token",
    refresh_token: refreshToken,
  });

  const resp = await fetch("https://accounts.spotify.com/api/token", {
    method: "POST",
    headers: {
      "Content-Type": "application/x-www-form-urlencoded",
      Authorization: `Basic ${btoa(`${clientId}:${clientSecret}`)}`,
    },
    body: body.toString(),
  });

  if (!resp.ok) {
    const text = await resp.text();
    throw new Error(`Spotify token refresh failed: ${resp.status} ${text}`);
  }

  const data = (await resp.json()) as {
    access_token: string;
    expires_in: number;
  };

  return { accessToken: data.access_token, expiresIn: data.expires_in };
}

// ---------------------------------------------------------------------------
// Spotify response types
// ---------------------------------------------------------------------------

interface SpotifyPaging<T> {
  items: T[];
  next: string | null;
  total: number;
}

interface SpotifyArtistRef {
  id?: string;
  name: string;
}

interface SpotifyTrack {
  id?: string;
  name: string;
  artists?: SpotifyArtistRef[];
  album?: { id?: string; name: string; artists?: SpotifyArtistRef[] };
}

interface SpotifySavedTrack {
  added_at?: string;
  track?: SpotifyTrack;
}

interface SpotifySavedAlbum {
  added_at?: string;
  album?: {
    id?: string;
    name: string;
    artists?: SpotifyArtistRef[];
  };
}

interface SpotifyFollowedArtistsResponse {
  artists: {
    items: Array<{ id?: string; name: string }>;
    cursors?: { after?: string };
    total?: number;
  };
}

interface SpotifyPlaylist {
  id: string;
  name: string;
  tracks?: { total?: number };
}

interface SpotifyPlaylistTrackItem {
  added_at?: string;
  track?: SpotifyTrack;
}

// ---------------------------------------------------------------------------
// Main Spotify sync action
// ---------------------------------------------------------------------------

export const syncSpotifyLibrary = internalAction({
  args: {
    runId: v.id("platformSyncRuns"),
    userId: v.id("users"),
  },
  handler: async (ctx, args) => {
    const startTime = Date.now();
    const { runId, userId } = args;

    // ── Retrieve and check the run status ──────────────────────────────
    const run = await ctx.runQuery(
      "librarySyncActions:_getSyncRun" as any,
      { runId },
    );
    if (!run || run.status === "cancelled") return;

    // ── Restore checkpoint (for continuation) ──────────────────────────
    let checkpoint: CheckpointData = (run.checkpointData as CheckpointData | null) ?? {
      phase: "liked",
      offset: 0,
      tracksImported: 0,
      likedCount: 0,
      albumCount: 0,
      artistCount: 0,
      playlistTrackCount: 0,
      clearedExisting: false,
    };

    // ── Get the connection tokens ──────────────────────────────────────
    const conn = await ctx.runQuery(
      "librarySyncActions:_getConnectionTokens" as any,
      { userId, provider: "spotify" },
    );

    if (!conn || !conn.encryptedAccessToken) {
      await ctx.runMutation("librarySyncActions:_updateSyncRun" as any, {
        runId,
        status: "failed",
        completedAt: new Date().toISOString(),
        errorLog: [{ message: "No active Spotify connection found.", ts: new Date().toISOString() }],
      });
      return;
    }

    let accessToken: string;
    let refreshToken: string | undefined;

    try {
      accessToken = await decryptAccessToken(conn.encryptedAccessToken);
      refreshToken = await decryptRefreshTokenValue(conn.encryptedRefreshToken);
    } catch (err: any) {
      await ctx.runMutation("librarySyncActions:_updateSyncRun" as any, {
        runId,
        status: "failed",
        completedAt: new Date().toISOString(),
        errorLog: [{ message: `Token decryption failed: ${err.message}`, ts: new Date().toISOString() }],
      });
      return;
    }

    // Helper: handle 401 by refreshing the token
    async function handleTokenRefresh(): Promise<boolean> {
      if (!refreshToken) return false;
      try {
        const refreshed = await refreshSpotifyAccessToken(refreshToken);
        accessToken = refreshed.accessToken;
        const expiresAt = new Date(Date.now() + refreshed.expiresIn * 1000).toISOString();
        const encryptionKey = getEncryptionKey();
        const encrypted = await encryptToken(refreshed.accessToken, encryptionKey);
        await ctx.runMutation("librarySyncActions:_updateConnectionTokenFromSync" as any, {
          connectionId: conn.connectionId,
          encryptedAccessToken: encrypted,
          expiresAt,
        });
        return true;
      } catch {
        return false;
      }
    }

    // Helper: make a Spotify GET request, auto-refresh on 401
    async function spotifyGet<T>(url: string): Promise<T> {
      let res = await spotifyFetchWithRetry(url, accessToken);

      if (res.status === 401) {
        const refreshed = await handleTokenRefresh();
        if (!refreshed) {
          throw new Error(`Spotify API returned 401 and token refresh failed for ${url}`);
        }
        res = await spotifyFetchWithRetry(url, accessToken);
      }

      if (!res.ok) {
        const text = await res.text();
        throw new Error(`Spotify API error ${res.status}: ${text.substring(0, 300)}`);
      }

      return (await res.json()) as T;
    }

    // Helper: check if we need to pause for time limit
    function shouldPause(): boolean {
      return Date.now() - startTime > SAFE_RUNTIME_MS;
    }

    // Helper: save checkpoint and optionally schedule continuation
    async function saveCheckpoint(pause: boolean) {
      await ctx.runMutation("librarySyncActions:_updateSyncRun" as any, {
        runId,
        checkpointData: checkpoint,
        metadata: {
          tracksImported: checkpoint.tracksImported,
          likedCount: checkpoint.likedCount,
          albumCount: checkpoint.albumCount,
          artistCount: checkpoint.artistCount,
          playlistTrackCount: checkpoint.playlistTrackCount,
        },
      });

      if (pause) {
        // Schedule a continuation
        await ctx.scheduler.runAfter(
          0,
          "librarySyncActions:syncSpotifyLibrary" as any,
          { runId, userId },
        );
      }
    }

    // Helper: flush a batch of tracks to the DB
    const pendingTracks: TrackImport[] = [];

    async function flushTracks() {
      if (pendingTracks.length === 0) return;
      const batch = pendingTracks.splice(0, pendingTracks.length);
      const result: { imported: number } = await ctx.runMutation(
        "librarySyncActions:_batchImportTracks" as any,
        { userId, provider: "spotify", tracks: batch },
      );
      checkpoint.tracksImported += result.imported;
    }

    async function addTrack(track: TrackImport) {
      pendingTracks.push(track);
      if (pendingTracks.length >= BATCH_SIZE) {
        await flushTracks();
      }
    }

    // ── Clear existing tracks (once per sync) ──────────────────────────
    try {
      if (!checkpoint.clearedExisting) {
        await ctx.runMutation("librarySyncActions:_clearProviderTracks" as any, {
          userId,
          provider: "spotify",
        });
        checkpoint.clearedExisting = true;
      }

      // ── Phase: Liked songs ─────────────────────────────────────────────
      if (checkpoint.phase === "liked") {
        let offset = checkpoint.offset ?? 0;
        let hasMore = true;

        while (hasMore) {
          if (shouldPause()) {
            checkpoint.offset = offset;
            await flushTracks();
            await saveCheckpoint(true);
            return;
          }

          const url = `https://api.spotify.com/v1/me/tracks?limit=${SPOTIFY_PAGE_SIZE}&offset=${offset}`;
          const page = await spotifyGet<SpotifyPaging<SpotifySavedTrack>>(url);

          for (const item of page.items) {
            if (!item.track?.id) continue;
            const track = item.track;
            await addTrack({
              providerTrackId: `liked:${track.id}`,
              trackName: track.name,
              albumName: track.album?.name,
              artistName: track.artists?.[0]?.name ?? "Unknown Artist",
              sourceType: "liked",
            });
            checkpoint.likedCount++;
          }

          hasMore = page.next !== null;
          offset += SPOTIFY_PAGE_SIZE;
        }

        checkpoint.phase = "albums";
        checkpoint.offset = 0;
      }

      // ── Phase: Saved albums ────────────────────────────────────────────
      if (checkpoint.phase === "albums") {
        let offset = checkpoint.offset ?? 0;
        let hasMore = true;

        while (hasMore) {
          if (shouldPause()) {
            checkpoint.offset = offset;
            await flushTracks();
            await saveCheckpoint(true);
            return;
          }

          const url = `https://api.spotify.com/v1/me/albums?limit=${SPOTIFY_PAGE_SIZE}&offset=${offset}`;
          const page = await spotifyGet<SpotifyPaging<SpotifySavedAlbum>>(url);

          for (const item of page.items) {
            if (!item.album?.id) continue;
            const album = item.album;
            const albumArtist = album.artists?.[0]?.name ?? "Unknown Artist";

            await addTrack({
              providerTrackId: `album:${album.id}`,
              trackName: `[Album] ${album.name}`,
              albumName: album.name,
              artistName: albumArtist,
              sourceType: "saved_album",
            });
            checkpoint.albumCount++;
          }

          hasMore = page.next !== null;
          offset += SPOTIFY_PAGE_SIZE;
        }

        checkpoint.phase = "artists";
        checkpoint.artistCursor = undefined;
      }

      // ── Phase: Followed artists (cursor-based) ─────────────────────────
      if (checkpoint.phase === "artists") {
        let cursor = checkpoint.artistCursor;
        let hasMore = true;

        while (hasMore) {
          if (shouldPause()) {
            checkpoint.artistCursor = cursor;
            await flushTracks();
            await saveCheckpoint(true);
            return;
          }

          let url = `https://api.spotify.com/v1/me/following?type=artist&limit=${SPOTIFY_PAGE_SIZE}`;
          if (cursor) {
            url += `&after=${cursor}`;
          }

          const response = await spotifyGet<SpotifyFollowedArtistsResponse>(url);

          for (const artist of response.artists.items) {
            if (!artist.id) continue;
            await addTrack({
              providerTrackId: `artist:${artist.id}`,
              trackName: `[Artist] ${artist.name}`,
              artistName: artist.name,
              sourceType: "followed_artist",
            });
            checkpoint.artistCount++;
          }

          cursor = response.artists.cursors?.after ?? undefined;
          hasMore = cursor !== undefined;
        }

        checkpoint.phase = "playlists";
        checkpoint.offset = 0;
        checkpoint.playlistIds = [];
        checkpoint.playlistNames = [];
      }

      // ── Phase: Discover playlists ──────────────────────────────────────
      if (checkpoint.phase === "playlists") {
        let offset = checkpoint.offset ?? 0;
        let hasMore = true;
        const playlistIds = checkpoint.playlistIds ?? [];
        const playlistNames = checkpoint.playlistNames ?? [];

        while (hasMore) {
          if (shouldPause()) {
            checkpoint.offset = offset;
            checkpoint.playlistIds = playlistIds;
            checkpoint.playlistNames = playlistNames;
            await flushTracks();
            await saveCheckpoint(true);
            return;
          }

          const url = `https://api.spotify.com/v1/me/playlists?limit=${SPOTIFY_PAGE_SIZE}&offset=${offset}`;
          const page = await spotifyGet<SpotifyPaging<SpotifyPlaylist>>(url);

          for (const playlist of page.items) {
            playlistIds.push(playlist.id);
            playlistNames.push(playlist.name);
          }

          hasMore = page.next !== null;
          offset += SPOTIFY_PAGE_SIZE;
        }

        checkpoint.playlistIds = playlistIds;
        checkpoint.playlistNames = playlistNames;
        checkpoint.phase = "playlist_tracks";
        checkpoint.playlistIndex = 0;
        checkpoint.playlistTrackOffset = 0;
      }

      // ── Phase: Fetch tracks for each playlist ──────────────────────────
      if (checkpoint.phase === "playlist_tracks") {
        const playlistIds = checkpoint.playlistIds ?? [];
        const playlistNames = checkpoint.playlistNames ?? [];
        let plIdx = checkpoint.playlistIndex ?? 0;

        while (plIdx < playlistIds.length) {
          const playlistId = playlistIds[plIdx];
          const playlistName = playlistNames[plIdx] ?? "Unknown Playlist";
          let trackOffset = checkpoint.playlistTrackOffset ?? 0;
          let hasMore = true;
          let positionIndex = trackOffset;

          while (hasMore) {
            if (shouldPause()) {
              checkpoint.playlistIndex = plIdx;
              checkpoint.playlistTrackOffset = trackOffset;
              await flushTracks();
              await saveCheckpoint(true);
              return;
            }

            const url =
              `https://api.spotify.com/v1/playlists/${playlistId}/tracks` +
              `?limit=${PLAYLIST_TRACK_PAGE_SIZE}&offset=${trackOffset}` +
              `&fields=next,items(added_at,track(id,name,artists(id,name),album(id,name)))`;

            const page = await spotifyGet<SpotifyPaging<SpotifyPlaylistTrackItem>>(url);

            for (const item of page.items) {
              if (!item.track?.id) continue;
              const track = item.track;
              await addTrack({
                providerTrackId: `playlist:${playlistId}:${track.id}:${positionIndex}`,
                trackName: track.name,
                albumName: track.album?.name,
                artistName: track.artists?.[0]?.name ?? "Unknown Artist",
                sourceType: "playlist_track",
                playlistName,
              });
              checkpoint.playlistTrackCount++;
              positionIndex++;
            }

            hasMore = page.next !== null;
            trackOffset += PLAYLIST_TRACK_PAGE_SIZE;
          }

          plIdx++;
          checkpoint.playlistTrackOffset = 0;
        }

        checkpoint.phase = "done";
      }

      // ── Flush remaining tracks ─────────────────────────────────────────
      await flushTracks();

      // ── Mark sync as completed ─────────────────────────────────────────
      await ctx.runMutation("librarySyncActions:_updateSyncRun" as any, {
        runId,
        status: "completed",
        completedAt: new Date().toISOString(),
        checkpointData: checkpoint,
        metadata: {
          tracksImported: checkpoint.tracksImported,
          likedCount: checkpoint.likedCount,
          albumCount: checkpoint.albumCount,
          artistCount: checkpoint.artistCount,
          playlistTrackCount: checkpoint.playlistTrackCount,
          durationMs: Date.now() - startTime,
        },
      });

      // ── Trigger offense summary recompute ─────────────────────────────
      await ctx.scheduler.runAfter(
        0,
        internal.offensePipeline.recomputeUserOffenseSummary,
        { userId, triggerReason: "sync_complete" },
      );
    } catch (err: any) {
      // Flush whatever we have so far before marking as failed
      try {
        await flushTracks();
      } catch {
        // Ignore flush errors during failure handling
      }

      await ctx.runMutation("librarySyncActions:_updateSyncRun" as any, {
        runId,
        status: "failed",
        completedAt: new Date().toISOString(),
        checkpointData: checkpoint,
        errorLog: [
          {
            message: err.message?.substring(0, 500) ?? "Unknown sync error",
            phase: checkpoint.phase,
            ts: new Date().toISOString(),
          },
        ],
        metadata: {
          tracksImported: checkpoint.tracksImported,
          likedCount: checkpoint.likedCount,
          albumCount: checkpoint.albumCount,
          artistCount: checkpoint.artistCount,
          playlistTrackCount: checkpoint.playlistTrackCount,
          durationMs: Date.now() - startTime,
        },
      });
    }
  },
});

// ---------------------------------------------------------------------------
// Tidal sync action (stub — same checkpoint pattern)
// ---------------------------------------------------------------------------

export const syncTidalLibrary = internalAction({
  args: {
    runId: v.id("platformSyncRuns"),
    userId: v.id("users"),
  },
  handler: async (ctx, args) => {
    const { runId, userId } = args;

    const conn = await ctx.runQuery(
      "librarySyncActions:_getConnectionTokens" as any,
      { userId, provider: "tidal" },
    );

    if (!conn || !conn.encryptedAccessToken) {
      await ctx.runMutation("librarySyncActions:_updateSyncRun" as any, {
        runId,
        status: "failed",
        completedAt: new Date().toISOString(),
        errorLog: [{ message: "No active Tidal connection found.", ts: new Date().toISOString() }],
      });
      return;
    }

    let accessToken: string;
    try {
      accessToken = await decryptAccessToken(conn.encryptedAccessToken);
    } catch (err: any) {
      await ctx.runMutation("librarySyncActions:_updateSyncRun" as any, {
        runId,
        status: "failed",
        completedAt: new Date().toISOString(),
        errorLog: [{ message: `Token decryption failed: ${err.message}`, ts: new Date().toISOString() }],
      });
      return;
    }

    // Clear existing tracks
    await ctx.runMutation("librarySyncActions:_clearProviderTracks" as any, {
      userId,
      provider: "tidal",
    });

    const pendingTracks: TrackImport[] = [];
    let tracksImported = 0;

    async function flushTracks() {
      if (pendingTracks.length === 0) return;
      const batch = pendingTracks.splice(0, pendingTracks.length);
      const result: { imported: number } = await ctx.runMutation(
        "librarySyncActions:_batchImportTracks" as any,
        { userId, provider: "tidal", tracks: batch },
      );
      tracksImported += result.imported;
    }

    try {
      // Tidal API: GET /v2/favorites/tracks (paginated)
      let offset = 0;
      let hasMore = true;

      while (hasMore) {
        const url = `https://openapi.tidal.com/v2/favorites/tracks?limit=50&offset=${offset}`;
        const res = await fetch(url, {
          headers: {
            Authorization: `Bearer ${accessToken}`,
            Accept: "application/json",
          },
        });

        if (!res.ok) {
          throw new Error(`Tidal API error ${res.status}: ${(await res.text()).substring(0, 300)}`);
        }

        const data = (await res.json()) as {
          data: Array<{
            id: string;
            resource: {
              id: string;
              title: string;
              artists?: Array<{ name: string }>;
              album?: { title: string };
            };
          }>;
          metadata?: { total?: number };
        };

        for (const item of data.data ?? []) {
          const resource = item.resource;
          pendingTracks.push({
            providerTrackId: `liked:${resource.id}`,
            trackName: resource.title,
            albumName: resource.album?.title,
            artistName: resource.artists?.[0]?.name ?? "Unknown Artist",
            sourceType: "liked",
          });
          if (pendingTracks.length >= BATCH_SIZE) {
            await flushTracks();
          }
        }

        hasMore = (data.data ?? []).length >= 50;
        offset += 50;
      }

      await flushTracks();

      await ctx.runMutation("librarySyncActions:_updateSyncRun" as any, {
        runId,
        status: "completed",
        completedAt: new Date().toISOString(),
        metadata: { tracksImported },
      });

      await ctx.scheduler.runAfter(
        0,
        internal.offensePipeline.recomputeUserOffenseSummary,
        { userId, triggerReason: "sync_complete" },
      );
    } catch (err: any) {
      try { await flushTracks(); } catch { /* ignore */ }

      await ctx.runMutation("librarySyncActions:_updateSyncRun" as any, {
        runId,
        status: "failed",
        completedAt: new Date().toISOString(),
        errorLog: [{ message: err.message?.substring(0, 500) ?? "Unknown error", ts: new Date().toISOString() }],
        metadata: { tracksImported },
      });
    }
  },
});

// ---------------------------------------------------------------------------
// YouTube Music sync action (stub — same checkpoint pattern)
// ---------------------------------------------------------------------------

export const syncYouTubeLibrary = internalAction({
  args: {
    runId: v.id("platformSyncRuns"),
    userId: v.id("users"),
  },
  handler: async (ctx, args) => {
    const { runId, userId } = args;

    const conn = await ctx.runQuery(
      "librarySyncActions:_getConnectionTokens" as any,
      { userId, provider: "youtube" },
    );

    if (!conn || !conn.encryptedAccessToken) {
      await ctx.runMutation("librarySyncActions:_updateSyncRun" as any, {
        runId,
        status: "failed",
        completedAt: new Date().toISOString(),
        errorLog: [{ message: "No active YouTube connection found.", ts: new Date().toISOString() }],
      });
      return;
    }

    let accessToken: string;
    try {
      accessToken = await decryptAccessToken(conn.encryptedAccessToken);
    } catch (err: any) {
      await ctx.runMutation("librarySyncActions:_updateSyncRun" as any, {
        runId,
        status: "failed",
        completedAt: new Date().toISOString(),
        errorLog: [{ message: `Token decryption failed: ${err.message}`, ts: new Date().toISOString() }],
      });
      return;
    }

    // Clear existing tracks
    await ctx.runMutation("librarySyncActions:_clearProviderTracks" as any, {
      userId,
      provider: "youtube",
    });

    const pendingTracks: TrackImport[] = [];
    let tracksImported = 0;

    async function flushTracks() {
      if (pendingTracks.length === 0) return;
      const batch = pendingTracks.splice(0, pendingTracks.length);
      const result: { imported: number } = await ctx.runMutation(
        "librarySyncActions:_batchImportTracks" as any,
        { userId, provider: "youtube", tracks: batch },
      );
      tracksImported += result.imported;
    }

    try {
      // YouTube Data API: GET /youtube/v3/playlistItems for "liked" playlist
      // First, get the user's "liked videos" playlist ID
      const channelUrl =
        "https://www.googleapis.com/youtube/v3/channels?part=contentDetails&mine=true";
      const channelRes = await fetch(channelUrl, {
        headers: { Authorization: `Bearer ${accessToken}` },
      });

      if (!channelRes.ok) {
        throw new Error(
          `YouTube Channels API error ${channelRes.status}: ${(await channelRes.text()).substring(0, 300)}`,
        );
      }

      const channelData = (await channelRes.json()) as {
        items?: Array<{
          contentDetails?: {
            relatedPlaylists?: { likes?: string };
          };
        }>;
      };

      const likedPlaylistId =
        channelData.items?.[0]?.contentDetails?.relatedPlaylists?.likes;

      if (likedPlaylistId) {
        let pageToken: string | undefined;
        let hasMore = true;

        while (hasMore) {
          let url =
            `https://www.googleapis.com/youtube/v3/playlistItems` +
            `?part=snippet&maxResults=50&playlistId=${likedPlaylistId}`;
          if (pageToken) {
            url += `&pageToken=${pageToken}`;
          }

          const res = await fetch(url, {
            headers: { Authorization: `Bearer ${accessToken}` },
          });

          if (!res.ok) {
            throw new Error(
              `YouTube PlaylistItems API error ${res.status}: ${(await res.text()).substring(0, 300)}`,
            );
          }

          const data = (await res.json()) as {
            items?: Array<{
              snippet?: {
                resourceId?: { videoId?: string };
                title?: string;
                channelTitle?: string;
                publishedAt?: string;
              };
            }>;
            nextPageToken?: string;
          };

          for (const item of data.items ?? []) {
            const snippet = item.snippet;
            const videoId = snippet?.resourceId?.videoId;
            if (!videoId) continue;

            pendingTracks.push({
              providerTrackId: `liked:${videoId}`,
              trackName: snippet?.title,
              artistName: snippet?.channelTitle ?? "Unknown Artist",
              sourceType: "liked",
            });
            if (pendingTracks.length >= BATCH_SIZE) {
              await flushTracks();
            }
          }

          pageToken = data.nextPageToken;
          hasMore = pageToken !== undefined;
        }
      }

      await flushTracks();

      await ctx.runMutation("librarySyncActions:_updateSyncRun" as any, {
        runId,
        status: "completed",
        completedAt: new Date().toISOString(),
        metadata: { tracksImported },
      });

      await ctx.scheduler.runAfter(
        0,
        internal.offensePipeline.recomputeUserOffenseSummary,
        { userId, triggerReason: "sync_complete" },
      );
    } catch (err: any) {
      try { await flushTracks(); } catch { /* ignore */ }

      await ctx.runMutation("librarySyncActions:_updateSyncRun" as any, {
        runId,
        status: "failed",
        completedAt: new Date().toISOString(),
        errorLog: [{ message: err.message?.substring(0, 500) ?? "Unknown error", ts: new Date().toISOString() }],
        metadata: { tracksImported },
      });
    }
  },
});
