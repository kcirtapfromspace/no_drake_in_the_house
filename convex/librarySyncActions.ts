import { v } from "convex/values";
import { internalAction, internalMutation, internalQuery, type QueryCtx, type MutationCtx } from "./_generated/server";
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
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const BATCH_SIZE = 50;
const SPOTIFY_PAGE_SIZE = 50;
const PLAYLIST_TRACK_PAGE_SIZE = 100;
const MAX_RETRIES = 6;
const MAX_RETRY_WAIT_SECS = 120;
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
  handler: async (ctx: QueryCtx, args) => {
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
      providerUserId: connection.providerUserId ?? null,
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
    let inserted = 0;
    let updated = 0;

    // Upsert: existing data stays visible while sync runs.
    // After sync completes, _cleanupStaleTracks removes items not seen.
    for (const track of args.tracks) {
      const legacyKey = `runtime:track:${args.userId}:${args.provider}:${track.providerTrackId}`;
      const existing = await ctx.db
        .query("userLibraryTracks")
        .withIndex("by_legacyKey", (q) => q.eq("legacyKey", legacyKey))
        .unique();

      if (existing) {
        await ctx.db.patch(existing._id, {
          trackName: track.trackName,
          albumName: track.albumName,
          artistName: track.artistName,
          sourceType: track.sourceType,
          playlistName: track.playlistName,
          lastSyncedAt: now,
          updatedAt: now,
        });
        updated++;
      } else {
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
        inserted++;
      }
    }

    return { imported: inserted + updated, inserted, updated };
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
  handler: async (ctx: MutationCtx, args) => {
    const batch = await ctx.db
      .query("userLibraryTracks")
      .withIndex("by_user_provider", (q) =>
        q.eq("userId", args.userId).eq("provider", args.provider),
      )
      .take(500);

    for (const track of batch) {
      await ctx.db.delete(track._id);
    }

    return { deleted: batch.length, hasMore: batch.length === 500 };
  },
});

/**
 * Remove tracks that were NOT updated during this sync run.
 * Called AFTER a successful sync so old data remains visible until replaced.
 */
export const _cleanupStaleTracks = internalMutation({
  args: {
    userId: v.id("users"),
    provider: v.string(),
    syncStartedAt: v.string(),
  },
  handler: async (ctx: MutationCtx, args) => {
    const tracks = await ctx.db
      .query("userLibraryTracks")
      .withIndex("by_user_provider", (q) =>
        q.eq("userId", args.userId).eq("provider", args.provider),
      )
      .take(500);

    let deleted = 0;
    for (const track of tracks) {
      if ((track.lastSyncedAt ?? "") < args.syncStartedAt) {
        await ctx.db.delete(track._id);
        deleted++;
      }
    }

    const hasMore = tracks.length === 500 && deleted > 0;
    return { deleted, hasMore };
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
// Shared helpers
// ---------------------------------------------------------------------------

/**
 * After a successful sync, remove tracks not touched in this run.
 * Old data stays visible throughout the sync and is only pruned after
 * the new data is fully imported — like a catalog that's always available.
 */
async function cleanupStaleTracks(
  ctx: { runMutation: (ref: any, args: any) => Promise<any> },
  userId: string,
  provider: string,
  syncStartedAt: string,
): Promise<number> {
  let totalDeleted = 0;
  let hasMore = true;
  while (hasMore) {
    const result: { deleted: number; hasMore: boolean } =
      await ctx.runMutation(internal.librarySyncActions._cleanupStaleTracks, {
        userId,
        provider,
        syncStartedAt,
      });
    totalDeleted += result.deleted;
    hasMore = result.hasMore;
  }
  return totalDeleted;
}

/**
 * Resolve unresolved artist names from library tracks into artist records.
 * Called after a successful sync to populate the artists table.
 */
async function resolveTrackArtists(
  ctx: { runQuery: (ref: any, args: any) => Promise<any>; runMutation: (ref: any, args: any) => Promise<any> },
  userId: string,
): Promise<number> {
  const unresolved: Array<{ name: string; count: number }> =
    await ctx.runQuery(
      internal.evidenceFinder._getUnresolvedArtistNames,
      { userId },
    );

  let resolved = 0;
  for (const { name } of unresolved) {
    const artistId = await ctx.runMutation(
      internal.evidenceFinder._resolveOrCreateArtist,
      { name },
    );
    await ctx.runMutation(
      internal.evidenceFinder._linkTracksToArtist,
      { userId, artistName: name, artistId },
    );
    resolved++;
  }
  return resolved;
}

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

/**
 * Generic fetch with 429 retry for Tidal / YouTube / Apple Music.
 */
async function apiFetchWithRetry(
  url: string,
  headers: Record<string, string>,
  label: string,
): Promise<Response> {
  for (let attempt = 0; attempt <= MAX_RETRIES; attempt++) {
    const res = await fetch(url, { headers });

    if (res.status === 429) {
      if (attempt === MAX_RETRIES) {
        throw new Error(`${label} rate limited after ${MAX_RETRIES} retries on ${url}`);
      }
      const retryAfter = res.headers.get("retry-after");
      const waitSecs = Math.min(
        parseInt(retryAfter || "5", 10) || 5,
        MAX_RETRY_WAIT_SECS,
      );
      await new Promise((r) => setTimeout(r, waitSecs * 1000));
      continue;
    }

    return res;
  }
  throw new Error(`${label} apiFetchWithRetry: unreachable`);
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
 * Delegates to the unified OAuth module (body params, no Basic Auth).
 */
async function refreshSpotifyAccessToken(
  refreshToken: string,
): Promise<{ accessToken: string; expiresIn: number }> {
  const { refreshAccessToken } = await import("./lib/oauth");
  const data = await refreshAccessToken("spotify", refreshToken);
  return { accessToken: data.access_token, expiresIn: data.expires_in ?? 3600 };
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
      internal.librarySyncActions._getSyncRun,
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
    };

    // ── Get the connection tokens ──────────────────────────────────────
    const conn = await ctx.runQuery(
      internal.librarySyncActions._getConnectionTokens,
      { userId, provider: "spotify" },
    );

    if (!conn || !conn.encryptedAccessToken) {
      await ctx.runMutation(internal.librarySyncActions._updateSyncRun, {
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
      await ctx.runMutation(internal.librarySyncActions._updateSyncRun, {
        runId,
        status: "failed",
        completedAt: new Date().toISOString(),
        errorLog: [{ message: `Token decryption failed: ${err.message}`, ts: new Date().toISOString() }],
      });
      return;
    }

    // Proactive refresh if token expires within 5 minutes
    if (conn.expiresAt) {
      const expiryMs = new Date(conn.expiresAt).getTime();
      if (expiryMs - Date.now() < 5 * 60 * 1000) {
        if (refreshToken) {
          try {
            const refreshed = await refreshSpotifyAccessToken(refreshToken);
            accessToken = refreshed.accessToken;
            const expiresAt = new Date(Date.now() + refreshed.expiresIn * 1000).toISOString();
            const encryptionKey = getEncryptionKey();
            const encrypted = await encryptToken(refreshed.accessToken, encryptionKey);
            await ctx.runMutation(internal.librarySyncActions._updateConnectionTokenFromSync, {
              connectionId: conn.connectionId,
              encryptedAccessToken: encrypted,
              expiresAt,
            });
          } catch {
            // Fall through — the reactive 401 handler will retry
          }
        }
      }
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
        await ctx.runMutation(internal.librarySyncActions._updateConnectionTokenFromSync, {
          connectionId: conn!.connectionId,
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
      await ctx.runMutation(internal.librarySyncActions._updateSyncRun, {
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
          internal.librarySyncActions.syncSpotifyLibrary,
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
        internal.librarySyncActions._batchImportTracks,
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

    // Record sync start time for stale track cleanup after success
    const syncStartedAt = new Date().toISOString();

    try {

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

          let page: SpotifyPaging<SpotifyPlaylist>;
          try {
            const url = `https://api.spotify.com/v1/me/playlists?limit=${SPOTIFY_PAGE_SIZE}&offset=${offset}`;
            page = await spotifyGet<SpotifyPaging<SpotifyPlaylist>>(url);
          } catch (err: any) {
            // 403 = token lacks playlist scope. Skip playlists, don't fail the whole sync.
            if (err.message?.includes("403")) {
              console.warn("[Spotify sync] Playlists returned 403 — skipping. Reconnect Spotify to grant playlist scope.");
              break;
            }
            throw err;
          }

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

      // ── Clean up tracks not seen in this sync ──────────────────────────
      await cleanupStaleTracks(ctx, userId, "spotify", syncStartedAt);

      // ── Mark sync as completed ─────────────────────────────────────────
      await ctx.runMutation(internal.librarySyncActions._updateSyncRun, {
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

      // ── Resolve artist names into artist records ─────────────────────
      // Delay to avoid write conflicts with the batch import / cleanup
      await new Promise((r) => setTimeout(r, 5_000));
      await resolveTrackArtists(ctx, userId);

      // ── Trigger offense summary recompute (delay to avoid conflicts) ──
      await ctx.scheduler.runAfter(
        60_000,
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

      await ctx.runMutation(internal.librarySyncActions._updateSyncRun, {
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
// Tidal sync action (checkpoint-based pagination)
// ---------------------------------------------------------------------------

export const syncTidalLibrary = internalAction({
  args: {
    runId: v.id("platformSyncRuns"),
    userId: v.id("users"),
  },
  handler: async (ctx, args) => {
    const startTime = Date.now();
    const { runId, userId } = args;

    // ── Retrieve and check the run status ──────────────────────────────
    const run = await ctx.runQuery(
      internal.librarySyncActions._getSyncRun,
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
    };

    // ── Get the connection tokens ──────────────────────────────────────
    const conn = await ctx.runQuery(
      internal.librarySyncActions._getConnectionTokens,
      { userId, provider: "tidal" },
    );

    if (!conn || !conn.encryptedAccessToken) {
      await ctx.runMutation(internal.librarySyncActions._updateSyncRun, {
        runId,
        status: "failed",
        completedAt: new Date().toISOString(),
        errorLog: [{ message: "No active Tidal connection found.", ts: new Date().toISOString() }],
      });
      return;
    }

    let accessToken: string;
    let refreshToken: string | undefined;

    try {
      accessToken = await decryptAccessToken(conn.encryptedAccessToken);
      refreshToken = await decryptRefreshTokenValue(conn.encryptedRefreshToken);
    } catch (err: any) {
      await ctx.runMutation(internal.librarySyncActions._updateSyncRun, {
        runId,
        status: "failed",
        completedAt: new Date().toISOString(),
        errorLog: [{ message: `Token decryption failed: ${err.message}`, ts: new Date().toISOString() }],
      });
      return;
    }

    // Proactive refresh if token expires within 5 minutes
    if (conn.expiresAt && refreshToken) {
      const expiryMs = new Date(conn.expiresAt).getTime();
      if (expiryMs - Date.now() < 5 * 60 * 1000) {
        try {
          const { refreshAccessToken } = await import("./lib/oauth");
          const data = await refreshAccessToken("tidal", refreshToken);
          accessToken = data.access_token;
          const expiresAt = new Date(Date.now() + (data.expires_in ?? 3600) * 1000).toISOString();
          const encryptionKey = getEncryptionKey();
          const encrypted = await encryptToken(data.access_token, encryptionKey);
          await ctx.runMutation(internal.librarySyncActions._updateConnectionTokenFromSync, {
            connectionId: conn.connectionId,
            encryptedAccessToken: encrypted,
            expiresAt,
          });
        } catch {
          // Fall through — use existing token, 401 handler will retry
        }
      }
    }

    // Helper: refresh token on 401
    async function handleTokenRefresh(): Promise<boolean> {
      if (!refreshToken) return false;
      try {
        const { refreshAccessToken } = await import("./lib/oauth");
        const data = await refreshAccessToken("tidal", refreshToken);
        accessToken = data.access_token;
        const expiresAt = new Date(Date.now() + (data.expires_in ?? 3600) * 1000).toISOString();
        const encryptionKey = getEncryptionKey();
        const encrypted = await encryptToken(data.access_token, encryptionKey);
        await ctx.runMutation(internal.librarySyncActions._updateConnectionTokenFromSync, {
          connectionId: conn!.connectionId,
          encryptedAccessToken: encrypted,
          expiresAt,
        });
        return true;
      } catch {
        return false;
      }
    }

    // Helper: Tidal v1 GET with auto-refresh on 401
    async function tidalGet(url: string): Promise<any> {
      let res = await apiFetchWithRetry(url, {
        Authorization: `Bearer ${accessToken}`,
      }, "Tidal");

      if (res.status === 401) {
        const refreshed = await handleTokenRefresh();
        if (!refreshed) {
          throw new Error(`Tidal API returned 401 and token refresh failed for ${url}`);
        }
        res = await apiFetchWithRetry(url, {
          Authorization: `Bearer ${accessToken}`,
        }, "Tidal");
      }

      if (!res.ok) {
        const text = await res.text();
        throw new Error(`Tidal API error ${res.status}: ${text.substring(0, 300)}`);
      }

      return await res.json();
    }

    // Helper: Tidal v2 GET (JSON:API) with auto-refresh on 401
    async function tidalGetV2(url: string): Promise<any> {
      let res = await apiFetchWithRetry(url, {
        Authorization: `Bearer ${accessToken}`,
        Accept: "application/vnd.api+json",
      }, "Tidal");

      if (res.status === 401) {
        const refreshed = await handleTokenRefresh();
        if (!refreshed) {
          throw new Error(`Tidal v2 API returned 401 and token refresh failed for ${url}`);
        }
        res = await apiFetchWithRetry(url, {
          Authorization: `Bearer ${accessToken}`,
          Accept: "application/vnd.api+json",
        }, "Tidal");
      }

      if (!res.ok) {
        const text = await res.text();
        throw new Error(`Tidal v2 API error ${res.status}: ${text.substring(0, 300)}`);
      }

      return await res.json();
    }

    function shouldPause(): boolean {
      return Date.now() - startTime > SAFE_RUNTIME_MS;
    }

    async function saveCheckpoint(pause: boolean) {
      await ctx.runMutation(internal.librarySyncActions._updateSyncRun, {
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
        await ctx.scheduler.runAfter(
          0,
          internal.librarySyncActions.syncTidalLibrary,
          { runId, userId },
        );
      }
    }

    const pendingTracks: TrackImport[] = [];

    async function flushTracks() {
      if (pendingTracks.length === 0) return;
      const batch = pendingTracks.splice(0, pendingTracks.length);
      const result: { imported: number } = await ctx.runMutation(
        internal.librarySyncActions._batchImportTracks,
        { userId, provider: "tidal", tracks: batch },
      );
      checkpoint.tracksImported += result.imported;
    }

    async function addTrack(track: TrackImport) {
      pendingTracks.push(track);
      if (pendingTracks.length >= BATCH_SIZE) {
        await flushTracks();
      }
    }

    const syncStartedAt = new Date().toISOString();
    const TIDAL_PAGE_SIZE = 50;

    try {
      // ── Resolve Tidal user ID ──────────────────────────────────────────
      let tidalUserId: string | null = conn.providerUserId ? String(conn.providerUserId) : null;
      let countryCode = "US";

      if (!tidalUserId) {
        try {
          const parts = accessToken.split(".");
          if (parts.length === 3) {
            const payload = JSON.parse(atob(parts[1]));
            const raw = payload.sub ?? payload.uid;
            tidalUserId = raw != null ? String(raw) : null;
            if (payload.countryCode) countryCode = payload.countryCode;
            console.log(`[Tidal sync] Got user from JWT: id=${tidalUserId}, country=${countryCode}`);
          }
        } catch {
          // Not a JWT or malformed
        }
      }

      if (!tidalUserId) {
        throw new Error(
          "Could not determine Tidal user ID. Try disconnecting and reconnecting Tidal.",
        );
      }

      // ── Phase: Liked tracks (v1 API — v2 does not support tracks yet) ─
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

          const url =
            `https://api.tidal.com/v1/users/${tidalUserId}/favorites/tracks` +
            `?countryCode=${countryCode}&limit=${TIDAL_PAGE_SIZE}&offset=${offset}`;

          let body: any;
          try {
            body = await tidalGet(url);
          } catch (err: any) {
            if (err.message?.includes("404")) {
              console.warn("[Tidal sync] favorites/tracks returned 404 — skipping");
              break;
            }
            throw err;
          }

          const items = body.items ?? [];
          console.log(
            `[Tidal sync] Liked tracks page: ${items.length} items (offset=${offset}, total=${body.totalNumberOfItems ?? "?"})`,
          );

          for (const entry of items) {
            const track = entry.item;
            if (!track) continue;

            await addTrack({
              providerTrackId: `liked:${track.id}`,
              trackName: track.title,
              albumName: track.album?.title,
              artistName: track.artists?.[0]?.name ?? track.artist?.name ?? "Unknown Artist",
              sourceType: "liked",
            });
            checkpoint.likedCount++;
          }

          hasMore = items.length >= TIDAL_PAGE_SIZE;
          offset += items.length;
        }

        checkpoint.phase = "albums";
        checkpoint.offset = 0;
      }

      // ── Phase: Saved albums (v2 API — userCollections) ────────────────
      if (checkpoint.phase === "albums") {
        let cursor: string | undefined = undefined;
        let hasMore = true;

        while (hasMore) {
          if (shouldPause()) {
            await flushTracks();
            await saveCheckpoint(true);
            return;
          }

          let url =
            `https://openapi.tidal.com/v2/userCollections/${tidalUserId}/relationships/albums` +
            `?countryCode=${countryCode}&include=albums`;
          if (cursor) {
            url += `&page[cursor]=${encodeURIComponent(cursor)}`;
          }

          let body: any;
          try {
            body = await tidalGetV2(url);
          } catch (err: any) {
            // If v2 userCollections fails, fall back to v1
            if (err.message?.includes("404") || err.message?.includes("400")) {
              console.warn("[Tidal sync] v2 userCollections/albums failed, falling back to v1");
              try {
                let v1Offset = 0;
                let v1HasMore = true;
                while (v1HasMore) {
                  const v1Url =
                    `https://api.tidal.com/v1/users/${tidalUserId}/favorites/albums` +
                    `?countryCode=${countryCode}&limit=${TIDAL_PAGE_SIZE}&offset=${v1Offset}`;
                  const v1Body = await tidalGet(v1Url);
                  const v1Items = v1Body.items ?? [];
                  for (const entry of v1Items) {
                    const album = entry.item;
                    if (!album) continue;
                    await addTrack({
                      providerTrackId: `album:${album.id}`,
                      trackName: `[Album] ${album.title}`,
                      albumName: album.title,
                      artistName: album.artists?.[0]?.name ?? album.artist?.name ?? "Unknown Artist",
                      sourceType: "saved_album",
                    });
                    checkpoint.albumCount++;
                  }
                  v1HasMore = v1Items.length >= TIDAL_PAGE_SIZE;
                  v1Offset += v1Items.length;
                }
              } catch (v1Err: any) {
                console.warn(`[Tidal sync] v1 fallback for albums also failed: ${v1Err.message}`);
              }
              break;
            }
            throw err;
          }

          const items = body.data ?? [];
          const includedById = new Map<string, any>();
          for (const inc of body.included ?? []) {
            includedById.set(`${inc.type}:${inc.id}`, inc);
          }

          for (const item of items) {
            const album = includedById.get(`albums:${item.id}`);
            const attrs = album?.attributes;
            if (!attrs) continue;

            const artistRel = album?.relationships?.artists?.data?.[0];
            const artistResource = artistRel
              ? includedById.get(`artists:${artistRel.id}`)
              : undefined;
            const artistName = artistResource?.attributes?.name ?? "Unknown Artist";

            await addTrack({
              providerTrackId: `album:${item.id}`,
              trackName: `[Album] ${attrs.title}`,
              albumName: attrs.title,
              artistName,
              sourceType: "saved_album",
            });
            checkpoint.albumCount++;
          }

          const nextCursor = body.links?.meta?.nextCursor;
          if (nextCursor && items.length > 0) {
            cursor = nextCursor;
          } else {
            hasMore = false;
          }
        }

        checkpoint.phase = "artists";
      }

      // ── Phase: Followed artists (v2 API — userCollections) ────────────
      if (checkpoint.phase === "artists") {
        let cursor: string | undefined = undefined;
        let hasMore = true;

        while (hasMore) {
          if (shouldPause()) {
            await flushTracks();
            await saveCheckpoint(true);
            return;
          }

          let url =
            `https://openapi.tidal.com/v2/userCollections/${tidalUserId}/relationships/artists` +
            `?countryCode=${countryCode}&include=artists`;
          if (cursor) {
            url += `&page[cursor]=${encodeURIComponent(cursor)}`;
          }

          let body: any;
          try {
            body = await tidalGetV2(url);
          } catch (err: any) {
            // If v2 userCollections fails, fall back to v1
            if (err.message?.includes("404") || err.message?.includes("400")) {
              console.warn("[Tidal sync] v2 userCollections/artists failed, falling back to v1");
              try {
                let v1Offset = 0;
                let v1HasMore = true;
                while (v1HasMore) {
                  const v1Url =
                    `https://api.tidal.com/v1/users/${tidalUserId}/favorites/artists` +
                    `?countryCode=${countryCode}&limit=${TIDAL_PAGE_SIZE}&offset=${v1Offset}`;
                  const v1Body = await tidalGet(v1Url);
                  const v1Items = v1Body.items ?? [];
                  for (const entry of v1Items) {
                    const artist = entry.item;
                    if (!artist) continue;
                    await addTrack({
                      providerTrackId: `artist:${artist.id}`,
                      trackName: `[Artist] ${artist.name}`,
                      artistName: artist.name,
                      sourceType: "followed_artist",
                    });
                    checkpoint.artistCount++;
                  }
                  v1HasMore = v1Items.length >= TIDAL_PAGE_SIZE;
                  v1Offset += v1Items.length;
                }
              } catch (v1Err: any) {
                console.warn(`[Tidal sync] v1 fallback for artists also failed: ${v1Err.message}`);
              }
              break;
            }
            throw err;
          }

          const items = body.data ?? [];
          const includedById = new Map<string, any>();
          for (const inc of body.included ?? []) {
            includedById.set(`${inc.type}:${inc.id}`, inc);
          }

          for (const item of items) {
            const artist = includedById.get(`artists:${item.id}`);
            const name = artist?.attributes?.name ?? "Unknown Artist";

            await addTrack({
              providerTrackId: `artist:${item.id}`,
              trackName: `[Artist] ${name}`,
              artistName: name,
              sourceType: "followed_artist",
            });
            checkpoint.artistCount++;
          }

          const nextCursor = body.links?.meta?.nextCursor;
          if (nextCursor && items.length > 0) {
            cursor = nextCursor;
          } else {
            hasMore = false;
          }
        }

        checkpoint.phase = "playlists";
        checkpoint.offset = 0;
        checkpoint.playlistIds = [];
        checkpoint.playlistNames = [];
      }

      // ── Phase: Discover playlists (v1 API) ────────────────────────────
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

          let body: any;
          try {
            const url =
              `https://api.tidal.com/v1/users/${tidalUserId}/playlists` +
              `?countryCode=${countryCode}&limit=${TIDAL_PAGE_SIZE}&offset=${offset}`;
            body = await tidalGet(url);
          } catch (err: any) {
            if (err.message?.includes("404") || err.message?.includes("403")) {
              console.warn("[Tidal sync] Playlists endpoint not available — skipping");
              break;
            }
            throw err;
          }

          const items = body.items ?? [];
          for (const playlist of items) {
            playlistIds.push(String(playlist.uuid ?? playlist.id));
            playlistNames.push(playlist.title ?? "Unknown Playlist");
          }

          hasMore = items.length >= TIDAL_PAGE_SIZE;
          offset += items.length;
        }

        checkpoint.playlistIds = playlistIds;
        checkpoint.playlistNames = playlistNames;
        checkpoint.phase = "playlist_tracks";
        checkpoint.playlistIndex = 0;
        checkpoint.playlistTrackOffset = 0;
      }

      // ── Phase: Fetch tracks for each playlist (v1 API) ────────────────
      if (checkpoint.phase === "playlist_tracks") {
        const playlistIds = checkpoint.playlistIds ?? [];
        const playlistNames = checkpoint.playlistNames ?? [];
        let plIdx = checkpoint.playlistIndex ?? 0;

        while (plIdx < playlistIds.length) {
          const playlistId = playlistIds[plIdx];
          const playlistName = playlistNames[plIdx] ?? "Unknown Playlist";
          let trackOffset = checkpoint.playlistTrackOffset ?? 0;
          let hasMore = true;

          while (hasMore) {
            if (shouldPause()) {
              checkpoint.playlistIndex = plIdx;
              checkpoint.playlistTrackOffset = trackOffset;
              await flushTracks();
              await saveCheckpoint(true);
              return;
            }

            let body: any;
            try {
              const url =
                `https://api.tidal.com/v1/playlists/${playlistId}/tracks` +
                `?countryCode=${countryCode}&limit=${TIDAL_PAGE_SIZE}&offset=${trackOffset}`;
              body = await tidalGet(url);
            } catch (err: any) {
              if (err.message?.includes("404") || err.message?.includes("403")) {
                console.warn(`[Tidal sync] Playlist ${playlistId} tracks not accessible — skipping`);
                break;
              }
              throw err;
            }

            const items = body.items ?? [];
            for (let i = 0; i < items.length; i++) {
              const track = items[i];
              if (!track?.id) continue;

              await addTrack({
                providerTrackId: `playlist:${playlistId}:${track.id}:${trackOffset + i}`,
                trackName: track.title,
                albumName: track.album?.title,
                artistName: track.artists?.[0]?.name ?? track.artist?.name ?? "Unknown Artist",
                sourceType: "playlist_track",
                playlistName,
              });
              checkpoint.playlistTrackCount++;
            }

            hasMore = items.length >= TIDAL_PAGE_SIZE;
            trackOffset += items.length;
          }

          plIdx++;
          checkpoint.playlistTrackOffset = 0;
        }

        checkpoint.phase = "done";
      }

      // ── Flush remaining + cleanup ──────────────────────────────────────
      await flushTracks();
      await cleanupStaleTracks(ctx, userId, "tidal", syncStartedAt);

      await ctx.runMutation(internal.librarySyncActions._updateSyncRun, {
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

      await new Promise((r) => setTimeout(r, 5_000));
      await resolveTrackArtists(ctx, userId);

      await ctx.scheduler.runAfter(
        60_000,
        internal.offensePipeline.recomputeUserOffenseSummary,
        { userId, triggerReason: "sync_complete" },
      );
    } catch (err: any) {
      try { await flushTracks(); } catch { /* ignore */ }

      await ctx.runMutation(internal.librarySyncActions._updateSyncRun, {
        runId,
        status: "failed",
        completedAt: new Date().toISOString(),
        errorLog: [{ message: err.message?.substring(0, 500) ?? "Unknown error", ts: new Date().toISOString() }],
        checkpointData: checkpoint,
        metadata: {
          tracksImported: checkpoint.tracksImported,
          likedCount: checkpoint.likedCount,
          albumCount: checkpoint.albumCount,
          artistCount: checkpoint.artistCount,
          playlistTrackCount: checkpoint.playlistTrackCount,
        },
      });
    }
  },
});

// ---------------------------------------------------------------------------
// YouTube Music sync action (checkpoint-based pagination)
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Apple Music sync action
// ---------------------------------------------------------------------------

export const syncAppleMusicLibrary = internalAction({
  args: {
    runId: v.id("platformSyncRuns"),
    userId: v.id("users"),
  },
  handler: async (ctx, args) => {
    const { runId, userId } = args;

    // Get stored Music User Token
    const conn = await ctx.runQuery(
      internal.librarySyncActions._getConnectionTokens,
      { userId, provider: "apple_music" },
    );

    if (!conn || !conn.encryptedAccessToken) {
      await ctx.runMutation(internal.librarySyncActions._updateSyncRun, {
        runId,
        status: "failed",
        completedAt: new Date().toISOString(),
        errorLog: [{ message: "No active Apple Music connection found.", ts: new Date().toISOString() }],
      });
      return;
    }

    let musicUserToken: string;
    try {
      musicUserToken = await decryptAccessToken(conn.encryptedAccessToken);
    } catch (err: any) {
      await ctx.runMutation(internal.librarySyncActions._updateSyncRun, {
        runId,
        status: "failed",
        completedAt: new Date().toISOString(),
        errorLog: [{ message: `Token decryption failed: ${err.message}`, ts: new Date().toISOString() }],
      });
      return;
    }

    // Get the Apple Developer Token (cross-runtime call: default → Node.js)
    const devTokenResult: { developer_token: string | null; error?: string } =
      await ctx.runAction(internal.signing.getDeveloperToken, {});

    if (!devTokenResult.developer_token) {
      await ctx.runMutation(internal.librarySyncActions._updateSyncRun, {
        runId,
        status: "failed",
        completedAt: new Date().toISOString(),
        errorLog: [{
          message: devTokenResult.error ?? "Apple Music developer token not available.",
          ts: new Date().toISOString(),
        }],
      });
      return;
    }

    const developerToken = devTokenResult.developer_token;

    const syncStartedAt = new Date().toISOString();
    const pendingTracks: TrackImport[] = [];
    let tracksImported = 0;

    async function flushTracks() {
      if (pendingTracks.length === 0) return;
      const batch = pendingTracks.splice(0, pendingTracks.length);
      const result: { imported: number } = await ctx.runMutation(
        internal.librarySyncActions._batchImportTracks,
        { userId, provider: "apple_music", tracks: batch },
      );
      tracksImported += result.imported;
    }

    try {
      // Apple Music API: GET /v1/me/library/songs (paginated)
      let nextUrl: string | null =
        "https://api.music.apple.com/v1/me/library/songs?limit=100";

      while (nextUrl) {
        const res = await apiFetchWithRetry(nextUrl, {
          Authorization: `Bearer ${developerToken}`,
          "Music-User-Token": musicUserToken,
          Accept: "application/json",
        }, "Apple Music");

        if (!res.ok) {
          throw new Error(
            `Apple Music API error ${res.status}: ${(await res.text()).substring(0, 300)}`,
          );
        }

        const data = (await res.json()) as {
          data?: Array<{
            id: string;
            attributes?: {
              name?: string;
              albumName?: string;
              artistName?: string;
            };
          }>;
          next?: string;
        };

        for (const item of data.data ?? []) {
          const attrs = item.attributes;
          pendingTracks.push({
            providerTrackId: `liked:${item.id}`,
            trackName: attrs?.name,
            albumName: attrs?.albumName,
            artistName: attrs?.artistName ?? "Unknown Artist",
            sourceType: "library",
          });
          if (pendingTracks.length >= BATCH_SIZE) {
            await flushTracks();
            await ctx.runMutation(internal.librarySyncActions._updateSyncRun, {
              runId,
              checkpointData: { tracksImported, phase: "library" },
              metadata: { tracksImported },
            });
          }
        }

        // Apple Music returns an absolute path for `next`
        nextUrl = data.next
          ? `https://api.music.apple.com${data.next}`
          : null;
      }

      await flushTracks();

      // ── Phase 2: Fetch playlists and their tracks ──────────────────────
      const appleHeaders = {
        Authorization: `Bearer ${developerToken}`,
        "Music-User-Token": musicUserToken,
        Accept: "application/json",
      };

      try {
        let playlistUrl: string | null =
          "https://api.music.apple.com/v1/me/library/playlists?limit=100";

        while (playlistUrl) {
          const plRes = await apiFetchWithRetry(playlistUrl, appleHeaders, "Apple Music");
          if (!plRes.ok) {
            console.warn(`[Apple Music sync] Playlists returned ${plRes.status} — skipping`);
            break;
          }

          const plData = (await plRes.json()) as {
            data?: Array<{ id: string; attributes?: { name?: string } }>;
            next?: string;
          };

          for (const playlist of plData.data ?? []) {
            const playlistName = playlist.attributes?.name ?? "Unknown Playlist";
            let trackUrl: string | null =
              `https://api.music.apple.com/v1/me/library/playlists/${playlist.id}/tracks?limit=100`;

            while (trackUrl) {
              const trRes = await apiFetchWithRetry(trackUrl, appleHeaders, "Apple Music");
              if (!trRes.ok) break;

              const trData = (await trRes.json()) as {
                data?: Array<{
                  id: string;
                  attributes?: { name?: string; albumName?: string; artistName?: string };
                }>;
                next?: string;
              };

              for (const item of trData.data ?? []) {
                const attrs = item.attributes;
                pendingTracks.push({
                  providerTrackId: `playlist:${playlist.id}:${item.id}`,
                  trackName: attrs?.name,
                  albumName: attrs?.albumName,
                  artistName: attrs?.artistName ?? "Unknown Artist",
                  sourceType: "playlist_track",
                  playlistName,
                });
                if (pendingTracks.length >= BATCH_SIZE) {
                  await flushTracks();
                  await ctx.runMutation(internal.librarySyncActions._updateSyncRun, {
                    runId,
                    checkpointData: { tracksImported, phase: "playlists" },
                    metadata: { tracksImported },
                  });
                }
              }

              trackUrl = trData.next
                ? `https://api.music.apple.com${trData.next}`
                : null;
            }
          }

          playlistUrl = plData.next
            ? `https://api.music.apple.com${plData.next}`
            : null;
        }
      } catch (plErr: any) {
        console.warn(`[Apple Music sync] Playlist sync error (non-fatal): ${plErr.message}`);
      }

      await flushTracks();
      await cleanupStaleTracks(ctx, userId, "apple_music", syncStartedAt);

      await ctx.runMutation(internal.librarySyncActions._updateSyncRun, {
        runId,
        status: "completed",
        completedAt: new Date().toISOString(),
        metadata: { tracksImported },
      });

      await new Promise((r) => setTimeout(r, 5_000));
      await resolveTrackArtists(ctx, userId);

      await ctx.scheduler.runAfter(
        60_000,
        internal.offensePipeline.recomputeUserOffenseSummary,
        { userId, triggerReason: "sync_complete" },
      );
    } catch (err: any) {
      try { await flushTracks(); } catch { /* ignore */ }

      await ctx.runMutation(internal.librarySyncActions._updateSyncRun, {
        runId,
        status: "failed",
        completedAt: new Date().toISOString(),
        errorLog: [{ message: err.message?.substring(0, 500) ?? "Unknown error", ts: new Date().toISOString() }],
        metadata: { tracksImported },
      });
    }
  },
});

export const syncYouTubeLibrary = internalAction({
  args: {
    runId: v.id("platformSyncRuns"),
    userId: v.id("users"),
  },
  handler: async (ctx, args) => {
    const { runId, userId } = args;

    const conn = await ctx.runQuery(
      internal.librarySyncActions._getConnectionTokens,
      { userId, provider: "youtube" },
    );

    if (!conn || !conn.encryptedAccessToken) {
      await ctx.runMutation(internal.librarySyncActions._updateSyncRun, {
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
      await ctx.runMutation(internal.librarySyncActions._updateSyncRun, {
        runId,
        status: "failed",
        completedAt: new Date().toISOString(),
        errorLog: [{ message: `Token decryption failed: ${err.message}`, ts: new Date().toISOString() }],
      });
      return;
    }

    const syncStartedAt = new Date().toISOString();
    const pendingTracks: TrackImport[] = [];
    let tracksImported = 0;

    async function flushTracks() {
      if (pendingTracks.length === 0) return;
      const batch = pendingTracks.splice(0, pendingTracks.length);
      const result: { imported: number } = await ctx.runMutation(
        internal.librarySyncActions._batchImportTracks,
        { userId, provider: "youtube", tracks: batch },
      );
      tracksImported += result.imported;
    }

    try {
      // YouTube Data API: GET /youtube/v3/playlistItems for "liked" playlist
      // First, get the user's "liked videos" playlist ID
      const channelUrl =
        "https://www.googleapis.com/youtube/v3/channels?part=contentDetails&mine=true";
      const channelRes = await apiFetchWithRetry(channelUrl, {
        Authorization: `Bearer ${accessToken}`,
      }, "YouTube");

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

          const res = await apiFetchWithRetry(url, {
            Authorization: `Bearer ${accessToken}`,
          }, "YouTube");

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
              await ctx.runMutation(internal.librarySyncActions._updateSyncRun, {
                runId,
                checkpointData: { tracksImported, phase: "liked" },
                metadata: { tracksImported },
              });
            }
          }

          pageToken = data.nextPageToken;
          hasMore = pageToken !== undefined;
        }
      }

      await flushTracks();
      await cleanupStaleTracks(ctx, userId, "youtube", syncStartedAt);

      await ctx.runMutation(internal.librarySyncActions._updateSyncRun, {
        runId,
        status: "completed",
        completedAt: new Date().toISOString(),
        metadata: { tracksImported },
      });

      await new Promise((r) => setTimeout(r, 5_000));
      await resolveTrackArtists(ctx, userId);

      await ctx.scheduler.runAfter(
        60_000,
        internal.offensePipeline.recomputeUserOffenseSummary,
        { userId, triggerReason: "sync_complete" },
      );
    } catch (err: any) {
      try { await flushTracks(); } catch { /* ignore */ }

      await ctx.runMutation(internal.librarySyncActions._updateSyncRun, {
        runId,
        status: "failed",
        completedAt: new Date().toISOString(),
        errorLog: [{ message: err.message?.substring(0, 500) ?? "Unknown error", ts: new Date().toISOString() }],
        metadata: { tracksImported },
      });
    }
  },
});
