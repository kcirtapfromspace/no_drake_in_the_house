import { ConvexError, v } from "convex/values";
import type { Doc, Id } from "./_generated/dataModel";
import { action, mutation, query } from "./_generated/server";
import type { MutationCtx, QueryCtx } from "./_generated/server";
import { api, internal } from "./_generated/api";
import { nowIso, requireCurrentUser } from "./lib/auth";
import {
  decryptToken,
  encryptToken,
  getEncryptionKey,
  isEncrypted,
} from "./lib/crypto";

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/** Maximum wall-clock time (ms) before we schedule a continuation. */
const EXECUTION_TIME_LIMIT_MS = 24 * 60 * 1000; // 24 minutes (buffer before 25 min Convex limit)

/** Tidal rate-limit budget: 500 req / 5 min. We use a conservative ceiling. */
const TIDAL_RATE_LIMIT_WINDOW_MS = 5 * 60 * 1000;
const TIDAL_RATE_LIMIT_MAX = 480; // leave headroom

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/**
 * Retrieve the stored access token for a provider connection.
 * Transparently decrypts the token if it was encrypted, or returns it as-is
 * for backward-compatible plaintext legacy tokens.
 */
async function getAccessToken(
  ctx: any,
  provider: string,
): Promise<{ accessToken: string; refreshToken?: string; connection: any }> {
  const connection = await ctx.runQuery(
    api.enforcement._getConnection,
    { provider },
  );
  if (!connection || !connection.accessToken) {
    throw new ConvexError(
      `No active ${provider} connection found. Please connect your account first.`,
    );
  }

  const encryptionKey = getEncryptionKey();

  // Decrypt the access token (or use as-is for legacy plaintext)
  const accessToken = isEncrypted(connection.accessToken)
    ? await decryptToken(connection.accessToken, encryptionKey)
    : connection.accessToken;

  // Decrypt the refresh token if present
  let refreshToken: string | undefined;
  if (connection.refreshToken) {
    refreshToken = isEncrypted(connection.refreshToken)
      ? await decryptToken(connection.refreshToken, encryptionKey)
      : connection.refreshToken;
  }

  return { accessToken, refreshToken, connection };
}

/**
 * Try to refresh an expired Spotify token.
 * Delegates to the unified OAuth module (body params, no Basic Auth).
 */
async function refreshSpotifyToken(
  refreshToken: string,
): Promise<{ access_token: string; expires_in: number }> {
  const { refreshAccessToken } = await import("./lib/oauth");
  const data = await refreshAccessToken("spotify", refreshToken);
  return { access_token: data.access_token, expires_in: data.expires_in ?? 3600 };
}

/**
 * Wrapper around fetch that auto-retries once on 401 by refreshing the token,
 * and respects Spotify 429 rate-limit responses with Retry-After backoff.
 */
async function spotifyFetch(
  url: string,
  opts: RequestInit,
  accessToken: string,
  refreshToken: string | undefined,
  ctx: any,
  provider: string,
): Promise<Response> {
  const headers = {
    ...(opts.headers as Record<string, string>),
    Authorization: `Bearer ${accessToken}`,
  };

  let resp = await fetch(url, { ...opts, headers });

  if (resp.status === 401 && refreshToken) {
    // Attempt token refresh
    const refreshed = await refreshSpotifyToken(refreshToken);
    const expiresAt = new Date(
      Date.now() + refreshed.expires_in * 1000,
    ).toISOString();

    // Encrypt the new token before persisting
    const encryptionKey = getEncryptionKey();
    const encryptedNewToken = await encryptToken(
      refreshed.access_token,
      encryptionKey,
    );

    // Persist the encrypted token
    await ctx.runMutation(api.enforcement._updateConnectionToken, {
      provider,
      accessToken: encryptedNewToken,
      expiresAt,
    });

    // Retry the original request with the plaintext token
    headers.Authorization = `Bearer ${refreshed.access_token}`;
    resp = await fetch(url, { ...opts, headers });
  }

  // Handle Spotify 429 rate limiting with Retry-After backoff
  if (resp.status === 429) {
    const retryAfter = resp.headers.get("Retry-After");
    const waitMs = retryAfter
      ? (parseInt(retryAfter, 10) || 1) * 1000
      : 2000;
    await new Promise((resolve) => setTimeout(resolve, Math.min(waitMs, 30000)));
    resp = await fetch(url, { ...opts, headers });
  }

  return resp;
}

/**
 * Generic provider fetch with bearer token and optional Retry-After handling.
 * Used for Tidal and YouTube Music where we do not have a token-refresh cycle
 * built in (callers should handle 401 separately if needed).
 */
async function providerFetch(
  url: string,
  opts: RequestInit,
  accessToken: string,
): Promise<Response> {
  const headers = {
    ...(opts.headers as Record<string, string>),
    Authorization: `Bearer ${accessToken}`,
  };

  let resp = await fetch(url, { ...opts, headers });

  // Respect Retry-After on 429
  if (resp.status === 429) {
    const retryAfter = resp.headers.get("Retry-After");
    const waitMs = retryAfter
      ? (parseInt(retryAfter, 10) || 1) * 1000
      : 2000;
    await new Promise((resolve) => setTimeout(resolve, Math.min(waitMs, 30000)));
    resp = await fetch(url, { ...opts, headers });
  }

  return resp;
}

/**
 * Check whether we are approaching the Convex action time limit.
 */
function isApproachingTimeLimit(startTime: number): boolean {
  return Date.now() - startTime >= EXECUTION_TIME_LIMIT_MS;
}

// ---------------------------------------------------------------------------
// Queries
// ---------------------------------------------------------------------------

export const getProgress = query({
  args: {
    batchId: v.id("actionBatches"),
  },
  handler: async (ctx, args) => {
    const batch = await ctx.db.get(args.batchId);
    if (!batch) return null;

    const items = await ctx.db
      .query("actionItems")
      .withIndex("by_batchId", (q) => q.eq("batchId", args.batchId))
      .collect();

    const completed = items.filter((i) => i.status === "completed").length;
    const failed = items.filter((i) => i.status === "failed").length;
    const skipped = items.filter((i) => i.status === "skipped").length;

    return {
      id: batch._id,
      provider: batch.provider,
      status: batch.status,
      options: batch.options,
      summary: {
        totalItems: items.length,
        completedItems: completed,
        failedItems: failed,
        skippedItems: skipped,
      },
      items: items.map((i) => ({
        id: i._id,
        entityType: i.entityType,
        entityId: i.entityId,
        action: i.action,
        beforeState: i.beforeState,
        afterState: i.afterState,
        status: i.status,
        errorMessage: i.errorMessage,
      })),
      createdAt: batch.createdAt,
      completedAt: batch.completedAt,
    };
  },
});

export const history = query({
  args: {},
  handler: async (ctx) => {
    const { user } = await requireCurrentUser(ctx);
    const batches = await ctx.db
      .query("actionBatches")
      .withIndex("by_userId", (q) => q.eq("userId", user._id))
      .collect();

    batches.sort((a, b) => b.createdAt.localeCompare(a.createdAt));

    const withItems = await Promise.all(
      batches.map(async (batch) => {
        const items = await ctx.db
          .query("actionItems")
          .withIndex("by_batchId", (q) => q.eq("batchId", batch._id))
          .collect();

        const completed = items.filter((i) => i.status === "completed").length;
        const failed = items.filter((i) => i.status === "failed").length;
        const skipped = items.filter((i) => i.status === "skipped").length;

        return {
          id: batch._id,
          provider: batch.provider,
          status: batch.status,
          options: batch.options,
          summary: {
            totalItems: items.length,
            completedItems: completed,
            failedItems: failed,
            skippedItems: skipped,
          },
          items: items.map((i) => ({
            id: i._id,
            entityType: i.entityType,
            entityId: i.entityId,
            action: i.action,
            status: i.status,
            errorMessage: i.errorMessage,
          })),
          createdAt: batch.createdAt,
          completedAt: batch.completedAt,
        };
      }),
    );

    return withItems;
  },
});

// ---------------------------------------------------------------------------
// Actions
// ---------------------------------------------------------------------------

export const planEnforcement = action({
  args: {
    providers: v.array(v.string()),
    options: v.any(),
    dryRun: v.optional(v.boolean()),
  },
  handler: async (ctx, args) => {
    const provider = args.providers[0] ?? "spotify";

    // Create the batch record first
    const batchId = await ctx.runMutation(
      api.enforcement._createBatch,
      {
        provider,
        options: args.options,
        dryRun: args.dryRun ?? true,
      },
    );

    // Compute the real impact by cross-referencing the user's library with blocks
    const impact = await ctx.runQuery(
      api.enforcement._computeImpact,
      { provider },
    );

    // Create action items for each flagged track / artist / playlist track
    if (
      impact.flaggedTracks.length > 0 ||
      impact.blockedArtistIds.length > 0 ||
      impact.playlistTrackRemovals.length > 0
    ) {
      await ctx.runMutation(api.enforcement._createPlanItems, {
        batchId,
        flaggedTracks: impact.flaggedTracks,
        blockedArtistIds: impact.blockedArtistIds,
        playlistTrackRemovals: impact.playlistTrackRemovals,
      });
    }

    // Update the batch summary with real counts
    const totalItems =
      impact.flaggedTracks.length +
      impact.blockedArtistIds.length +
      impact.playlistTrackRemovals.length;

    await ctx.runMutation(api.enforcement._updateBatchSummary, {
      batchId,
      summary: {
        totalItems,
        completedItems: 0,
        failedItems: 0,
        skippedItems: 0,
      },
    });

    return {
      planId: batchId,
      idempotencyKey: `plan-${batchId}`,
      impact: {
        tracksToRemove: impact.flaggedTracks.length,
        artistsToUnfollow: impact.blockedArtistIds.length,
        playlistsAffected: impact.playlistsAffected,
        playlistTracksToRemove: impact.playlistTrackRemovals.length,
        playlistDetails: impact.playlistDetails,
        flaggedArtistNames: impact.flaggedArtistNames,
      },
      capabilities: {
        removeLibraryTracks: true,
        removePlaylistTracks: ["spotify", "tidal", "youtube"].includes(provider),
        unfollowArtists: ["spotify", "tidal", "youtube"].includes(provider),
      },
      estimatedDuration:
        totalItems > 50 ? "~5 minutes" : "~2 minutes",
      resumable: true,
    };
  },
});

export const executePlan = action({
  args: {
    planId: v.string(),
    dryRun: v.optional(v.boolean()),
  },
  handler: async (ctx, args) => {
    const startTime = Date.now();
    const dryRun = args.dryRun ?? false;

    // Mark batch as running
    await ctx.runMutation(api.enforcement._updateBatchStatus, {
      batchId: args.planId,
      status: "running",
    });

    // Fetch all pending action items for this batch
    const items: Array<{
      id: string;
      entityType: string;
      entityId: string;
      action: string;
      beforeState: any;
    }> = await ctx.runQuery(api.enforcement._getPendingItems, {
      batchId: args.planId,
    });

    // Fetch the batch to know the provider
    const batch: { provider: string } | null = await ctx.runQuery(
      api.enforcement._getBatch,
      { batchId: args.planId },
    );

    const provider = batch?.provider ?? "spotify";

    let completedCount = 0;
    let failedCount = 0;
    let suspended = false;

    if (!dryRun && provider === "spotify") {
      const result = await executeSpotifyEnforcement(ctx, items, provider, startTime);
      completedCount = result.completed;
      failedCount = result.failed;
      suspended = result.suspended;
    } else if (!dryRun && provider === "tidal") {
      const result = await executeTidalEnforcement(ctx, items, provider, startTime);
      completedCount = result.completed;
      failedCount = result.failed;
      suspended = result.suspended;
    } else if (!dryRun && provider === "youtube") {
      const result = await executeYouTubeEnforcement(ctx, items, provider, startTime);
      completedCount = result.completed;
      failedCount = result.failed;
      suspended = result.suspended;
    } else if (dryRun) {
      // In dry-run mode, mark all items as skipped
      for (const item of items) {
        await ctx.runMutation(api.enforcement._updateItemStatus, {
          itemId: item.id,
          status: "skipped",
          afterState: { dryRun: true },
        });
      }
      completedCount = 0;
      failedCount = 0;
    }

    // If we suspended due to time limit, schedule a continuation
    if (suspended) {
      await ctx.runMutation(api.enforcement._updateBatchStatus, {
        batchId: args.planId,
        status: "suspended",
      });

      // Schedule immediate continuation
      await ctx.scheduler.runAfter(
        0,
        api.enforcement.resumeExecution,
        { batchId: args.planId },
      );

      return {
        id: args.planId,
        status: "suspended",
        message: `Enforcement paused for continuation: ${completedCount} succeeded so far, ${failedCount} failed. Resuming automatically.`,
        completed: completedCount,
        failed: failedCount,
        resuming: true,
      };
    }

    // Mark batch as completed or failed
    const finalStatus = failedCount > 0 && completedCount === 0
      ? "failed"
      : "completed";

    await ctx.runMutation(api.enforcement._updateBatchStatus, {
      batchId: args.planId,
      status: finalStatus,
    });

    return {
      id: args.planId,
      status: finalStatus,
      message: dryRun
        ? "Dry run completed. No changes were made."
        : `Enforcement completed: ${completedCount} succeeded, ${failedCount} failed.`,
      completed: completedCount,
      failed: failedCount,
    };
  },
});

/**
 * Resume a suspended enforcement execution.
 * Reads the batch, fetches remaining pending items, and continues processing.
 */
export const resumeExecution = action({
  args: {
    batchId: v.string(),
  },
  handler: async (ctx, args) => {
    const startTime = Date.now();

    // Mark batch as running again
    await ctx.runMutation(api.enforcement._updateBatchStatus, {
      batchId: args.batchId,
      status: "running",
    });

    // Fetch remaining pending items
    const items: Array<{
      id: string;
      entityType: string;
      entityId: string;
      action: string;
      beforeState: any;
    }> = await ctx.runQuery(api.enforcement._getPendingItems, {
      batchId: args.batchId,
    });

    if (items.length === 0) {
      // Nothing left to process
      await ctx.runMutation(api.enforcement._updateBatchStatus, {
        batchId: args.batchId,
        status: "completed",
      });
      return {
        id: args.batchId,
        status: "completed",
        message: "Resume found no pending items. Enforcement complete.",
        completed: 0,
        failed: 0,
      };
    }

    const batch: { provider: string } | null = await ctx.runQuery(
      api.enforcement._getBatch,
      { batchId: args.batchId },
    );

    const provider = batch?.provider ?? "spotify";
    let completedCount = 0;
    let failedCount = 0;
    let suspended = false;

    if (provider === "spotify") {
      const result = await executeSpotifyEnforcement(ctx, items, provider, startTime);
      completedCount = result.completed;
      failedCount = result.failed;
      suspended = result.suspended;
    } else if (provider === "tidal") {
      const result = await executeTidalEnforcement(ctx, items, provider, startTime);
      completedCount = result.completed;
      failedCount = result.failed;
      suspended = result.suspended;
    } else if (provider === "youtube") {
      const result = await executeYouTubeEnforcement(ctx, items, provider, startTime);
      completedCount = result.completed;
      failedCount = result.failed;
      suspended = result.suspended;
    }

    if (suspended) {
      await ctx.runMutation(api.enforcement._updateBatchStatus, {
        batchId: args.batchId,
        status: "suspended",
      });

      await ctx.scheduler.runAfter(
        0,
        api.enforcement.resumeExecution,
        { batchId: args.batchId },
      );

      return {
        id: args.batchId,
        status: "suspended",
        message: `Enforcement paused again: ${completedCount} succeeded, ${failedCount} failed. Resuming automatically.`,
        completed: completedCount,
        failed: failedCount,
        resuming: true,
      };
    }

    const finalStatus = failedCount > 0 && completedCount === 0
      ? "failed"
      : "completed";

    await ctx.runMutation(api.enforcement._updateBatchStatus, {
      batchId: args.batchId,
      status: finalStatus,
    });

    return {
      id: args.batchId,
      status: finalStatus,
      message: `Resume completed: ${completedCount} succeeded, ${failedCount} failed.`,
      completed: completedCount,
      failed: failedCount,
    };
  },
});

// ---------------------------------------------------------------------------
// Provider-specific enforcement executors
// ---------------------------------------------------------------------------

type ActionItem = {
  id: string;
  entityType: string;
  entityId: string;
  action: string;
  beforeState: any;
};

type ExecutionResult = {
  completed: number;
  failed: number;
  suspended: boolean;
};

/**
 * Execute Spotify enforcement: remove liked songs, unfollow artists,
 * and remove tracks from user-owned playlists.
 */
async function executeSpotifyEnforcement(
  ctx: any,
  items: ActionItem[],
  provider: string,
  startTime: number,
): Promise<ExecutionResult> {
  let completedCount = 0;
  let failedCount = 0;

  // Get access token for Spotify API calls
  let tokenInfo: { accessToken: string; refreshToken?: string };

  try {
    tokenInfo = await getAccessToken(ctx, provider);
  } catch {
    // Mark all items as failed
    for (const item of items) {
      await ctx.runMutation(api.enforcement._updateItemStatus, {
        itemId: item.id,
        status: "failed",
        errorMessage: `No active ${provider} connection.`,
      });
    }
    return { completed: 0, failed: items.length, suspended: false };
  }

  // Categorize items
  const trackIdsToRemove: string[] = [];
  const artistIdsToUnfollow: string[] = [];
  // playlistTrackItems: grouped by playlistId for batch removal
  const playlistTrackItems: Map<string, Array<{ itemRef: ActionItem; trackUri: string }>> = new Map();

  for (const item of items) {
    if (item.entityType === "track" && item.action === "remove") {
      trackIdsToRemove.push(item.entityId);
    } else if (item.entityType === "artist" && item.action === "unfollow") {
      artistIdsToUnfollow.push(item.entityId);
    } else if (item.entityType === "playlist_track" && item.action === "remove_playlist_track") {
      const playlistId = item.beforeState?.playlistId;
      const trackUri = item.beforeState?.trackUri ?? `spotify:track:${item.entityId}`;
      if (playlistId) {
        if (!playlistTrackItems.has(playlistId)) {
          playlistTrackItems.set(playlistId, []);
        }
        playlistTrackItems.get(playlistId)!.push({ itemRef: item, trackUri });
      }
    }
  }

  // --- Step 1: Remove tracks from library in chunks of 50 ---
  for (let i = 0; i < trackIdsToRemove.length; i += 50) {
    if (isApproachingTimeLimit(startTime)) {
      return { completed: completedCount, failed: failedCount, suspended: true };
    }

    const chunk = trackIdsToRemove.slice(i, i + 50);
    try {
      const resp = await spotifyFetch(
        "https://api.spotify.com/v1/me/tracks",
        {
          method: "DELETE",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ ids: chunk }),
        },
        tokenInfo.accessToken,
        tokenInfo.refreshToken,
        ctx,
        provider,
      );

      if (resp.ok || resp.status === 200) {
        for (const trackId of chunk) {
          const item = items.find(
            (it) => it.entityId === trackId && it.entityType === "track",
          );
          if (item) {
            await ctx.runMutation(
              api.enforcement._updateItemStatus,
              {
                itemId: item.id,
                status: "completed",
                afterState: { removed: true },
              },
            );
            completedCount++;
          }
        }
      } else {
        const errText = await resp.text();
        for (const trackId of chunk) {
          const item = items.find(
            (it) => it.entityId === trackId && it.entityType === "track",
          );
          if (item) {
            await ctx.runMutation(
              api.enforcement._updateItemStatus,
              {
                itemId: item.id,
                status: "failed",
                errorMessage: `Spotify API error ${resp.status}: ${errText.substring(0, 200)}`,
              },
            );
            failedCount++;
          }
        }
      }
    } catch (err: any) {
      for (const trackId of chunk) {
        const item = items.find(
          (it) => it.entityId === trackId && it.entityType === "track",
        );
        if (item) {
          await ctx.runMutation(
            api.enforcement._updateItemStatus,
            {
              itemId: item.id,
              status: "failed",
              errorMessage: err.message?.substring(0, 200) ?? "Unknown error",
            },
          );
          failedCount++;
        }
      }
    }
  }

  // --- Step 2: Unfollow artists in chunks of 50 ---
  for (let i = 0; i < artistIdsToUnfollow.length; i += 50) {
    if (isApproachingTimeLimit(startTime)) {
      return { completed: completedCount, failed: failedCount, suspended: true };
    }

    const chunk = artistIdsToUnfollow.slice(i, i + 50);
    try {
      const resp = await spotifyFetch(
        `https://api.spotify.com/v1/me/following?type=artist&ids=${chunk.join(",")}`,
        { method: "DELETE" },
        tokenInfo.accessToken,
        tokenInfo.refreshToken,
        ctx,
        provider,
      );

      if (resp.ok || resp.status === 200 || resp.status === 204) {
        for (const artistId of chunk) {
          const item = items.find(
            (it) =>
              it.entityId === artistId && it.entityType === "artist",
          );
          if (item) {
            await ctx.runMutation(
              api.enforcement._updateItemStatus,
              {
                itemId: item.id,
                status: "completed",
                afterState: { unfollowed: true },
              },
            );
            completedCount++;
          }
        }
      } else {
        const errText = await resp.text();
        for (const artistId of chunk) {
          const item = items.find(
            (it) =>
              it.entityId === artistId && it.entityType === "artist",
          );
          if (item) {
            await ctx.runMutation(
              api.enforcement._updateItemStatus,
              {
                itemId: item.id,
                status: "failed",
                errorMessage: `Spotify API error ${resp.status}: ${errText.substring(0, 200)}`,
              },
            );
            failedCount++;
          }
        }
      }
    } catch (err: any) {
      for (const artistId of chunk) {
        const item = items.find(
          (it) =>
            it.entityId === artistId && it.entityType === "artist",
        );
        if (item) {
          await ctx.runMutation(
            api.enforcement._updateItemStatus,
            {
              itemId: item.id,
              status: "failed",
              errorMessage: err.message?.substring(0, 200) ?? "Unknown error",
            },
          );
          failedCount++;
        }
      }
    }
  }

  // --- Step 3: Remove tracks from user-owned playlists ---
  for (const [playlistId, trackEntries] of playlistTrackItems) {
    if (isApproachingTimeLimit(startTime)) {
      return { completed: completedCount, failed: failedCount, suspended: true };
    }

    // Spotify DELETE /v1/playlists/{id}/tracks accepts up to 100 tracks per request
    for (let i = 0; i < trackEntries.length; i += 100) {
      if (isApproachingTimeLimit(startTime)) {
        return { completed: completedCount, failed: failedCount, suspended: true };
      }

      const chunk = trackEntries.slice(i, i + 100);
      const tracks = chunk.map((entry) => ({ uri: entry.trackUri }));

      try {
        const resp = await spotifyFetch(
          `https://api.spotify.com/v1/playlists/${playlistId}/tracks`,
          {
            method: "DELETE",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ tracks }),
          },
          tokenInfo.accessToken,
          tokenInfo.refreshToken,
          ctx,
          provider,
        );

        if (resp.ok || resp.status === 200) {
          for (const entry of chunk) {
            await ctx.runMutation(
              api.enforcement._updateItemStatus,
              {
                itemId: entry.itemRef.id,
                status: "completed",
                afterState: { removedFromPlaylist: playlistId },
              },
            );
            completedCount++;
          }
        } else {
          const errText = await resp.text();
          for (const entry of chunk) {
            await ctx.runMutation(
              api.enforcement._updateItemStatus,
              {
                itemId: entry.itemRef.id,
                status: "failed",
                errorMessage: `Spotify playlist API error ${resp.status}: ${errText.substring(0, 200)}`,
              },
            );
            failedCount++;
          }
        }
      } catch (err: any) {
        for (const entry of chunk) {
          await ctx.runMutation(
            api.enforcement._updateItemStatus,
            {
              itemId: entry.itemRef.id,
              status: "failed",
              errorMessage: err.message?.substring(0, 200) ?? "Unknown error",
            },
          );
          failedCount++;
        }
      }
    }
  }

  return { completed: completedCount, failed: failedCount, suspended: false };
}

/**
 * Execute Tidal enforcement: remove favorite tracks, unfollow artists,
 * and remove tracks from playlists.
 *
 * Tidal API endpoints:
 * - Remove favorite track:  DELETE /v1/users/{userId}/favorites/tracks/{trackId}
 * - Unfollow artist:        DELETE /v1/users/{userId}/favorites/artists/{artistId}
 * - Remove playlist track:  DELETE /v1/playlists/{playlistId}/items/{itemId}
 *
 * Rate limits: 500 req / 5 min. We track and respect Retry-After headers.
 */
async function executeTidalEnforcement(
  ctx: any,
  items: ActionItem[],
  provider: string,
  startTime: number,
): Promise<ExecutionResult> {
  let completedCount = 0;
  let failedCount = 0;

  let tokenInfo: { accessToken: string; refreshToken?: string; connection: any };

  try {
    tokenInfo = await getAccessToken(ctx, provider);
  } catch {
    for (const item of items) {
      await ctx.runMutation(api.enforcement._updateItemStatus, {
        itemId: item.id,
        status: "failed",
        errorMessage: `No active ${provider} connection.`,
      });
    }
    return { completed: 0, failed: items.length, suspended: false };
  }

  // Tidal requires the user's Tidal ID for favorites endpoints
  const tidalUserId = tokenInfo.connection?.providerUserId;
  if (!tidalUserId) {
    for (const item of items) {
      await ctx.runMutation(api.enforcement._updateItemStatus, {
        itemId: item.id,
        status: "failed",
        errorMessage: "Tidal user ID not found on connection record.",
      });
    }
    return { completed: 0, failed: items.length, suspended: false };
  }

  const TIDAL_BASE = "https://openapi.tidal.com/v1";
  let requestCount = 0;
  const windowStart = Date.now();

  async function tidalFetchWithRateLimit(
    url: string,
    opts: RequestInit,
  ): Promise<Response> {
    // Simple rate-limit guard
    requestCount++;
    if (requestCount >= TIDAL_RATE_LIMIT_MAX) {
      const elapsed = Date.now() - windowStart;
      if (elapsed < TIDAL_RATE_LIMIT_WINDOW_MS) {
        const waitMs = TIDAL_RATE_LIMIT_WINDOW_MS - elapsed + 1000;
        await new Promise((resolve) => setTimeout(resolve, waitMs));
      }
      // Reset after waiting
      requestCount = 0;
    }
    return providerFetch(url, opts, tokenInfo.accessToken);
  }

  for (const item of items) {
    if (isApproachingTimeLimit(startTime)) {
      return { completed: completedCount, failed: failedCount, suspended: true };
    }

    try {
      let resp: Response;

      if (item.entityType === "track" && item.action === "remove") {
        // Remove favorite track
        resp = await tidalFetchWithRateLimit(
          `${TIDAL_BASE}/users/${tidalUserId}/favorites/tracks/${item.entityId}`,
          { method: "DELETE" },
        );
      } else if (item.entityType === "artist" && item.action === "unfollow") {
        // Unfollow artist
        resp = await tidalFetchWithRateLimit(
          `${TIDAL_BASE}/users/${tidalUserId}/favorites/artists/${item.entityId}`,
          { method: "DELETE" },
        );
      } else if (item.entityType === "playlist_track" && item.action === "remove_playlist_track") {
        // Remove track from playlist
        const playlistId = item.beforeState?.playlistId;
        const playlistItemId = item.beforeState?.playlistItemId ?? item.entityId;
        if (!playlistId) {
          await ctx.runMutation(api.enforcement._updateItemStatus, {
            itemId: item.id,
            status: "failed",
            errorMessage: "Missing playlistId in beforeState.",
          });
          failedCount++;
          continue;
        }
        resp = await tidalFetchWithRateLimit(
          `${TIDAL_BASE}/playlists/${playlistId}/items/${playlistItemId}`,
          { method: "DELETE" },
        );
      } else {
        // Unknown action type, skip
        await ctx.runMutation(api.enforcement._updateItemStatus, {
          itemId: item.id,
          status: "skipped",
          afterState: { reason: "unsupported_action" },
        });
        continue;
      }

      if (resp.ok || resp.status === 200 || resp.status === 204) {
        const afterState =
          item.entityType === "playlist_track"
            ? { removedFromPlaylist: item.beforeState?.playlistId }
            : item.entityType === "artist"
              ? { unfollowed: true }
              : { removed: true };

        await ctx.runMutation(api.enforcement._updateItemStatus, {
          itemId: item.id,
          status: "completed",
          afterState,
        });
        completedCount++;
      } else {
        const errText = await resp.text();
        await ctx.runMutation(api.enforcement._updateItemStatus, {
          itemId: item.id,
          status: "failed",
          errorMessage: `Tidal API error ${resp.status}: ${errText.substring(0, 200)}`,
        });
        failedCount++;
      }
    } catch (err: any) {
      await ctx.runMutation(api.enforcement._updateItemStatus, {
        itemId: item.id,
        status: "failed",
        errorMessage: err.message?.substring(0, 200) ?? "Unknown error",
      });
      failedCount++;
    }
  }

  return { completed: completedCount, failed: failedCount, suspended: false };
}

/**
 * Execute YouTube Music enforcement: unlike videos, remove playlist items,
 * and unsubscribe from channels.
 *
 * YouTube API endpoints:
 * - Remove liked video:     POST /youtube/v3/videos/rate?id={id}&rating=none
 * - Remove playlist item:   DELETE /youtube/v3/playlistItems?id={itemId}
 * - Unsubscribe:            DELETE /youtube/v3/subscriptions?id={subscriptionId}
 */
async function executeYouTubeEnforcement(
  ctx: any,
  items: ActionItem[],
  provider: string,
  startTime: number,
): Promise<ExecutionResult> {
  let completedCount = 0;
  let failedCount = 0;

  let tokenInfo: { accessToken: string; refreshToken?: string; connection: any };

  try {
    tokenInfo = await getAccessToken(ctx, provider);
  } catch {
    for (const item of items) {
      await ctx.runMutation(api.enforcement._updateItemStatus, {
        itemId: item.id,
        status: "failed",
        errorMessage: `No active ${provider} connection.`,
      });
    }
    return { completed: 0, failed: items.length, suspended: false };
  }

  const YT_BASE = "https://www.googleapis.com/youtube/v3";

  for (const item of items) {
    if (isApproachingTimeLimit(startTime)) {
      return { completed: completedCount, failed: failedCount, suspended: true };
    }

    try {
      let resp: Response;

      if (item.entityType === "track" && item.action === "remove") {
        // Remove liked video (set rating to "none")
        resp = await providerFetch(
          `${YT_BASE}/videos/rate?id=${encodeURIComponent(item.entityId)}&rating=none`,
          {
            method: "POST",
            headers: { "Content-Type": "application/json" },
          },
          tokenInfo.accessToken,
        );
      } else if (item.entityType === "artist" && item.action === "unfollow") {
        // Unsubscribe from channel
        // entityId is the subscription ID (not the channel ID)
        resp = await providerFetch(
          `${YT_BASE}/subscriptions?id=${encodeURIComponent(item.entityId)}`,
          { method: "DELETE" },
          tokenInfo.accessToken,
        );
      } else if (item.entityType === "playlist_track" && item.action === "remove_playlist_track") {
        // Remove playlist item
        // entityId is the playlistItem ID
        const playlistItemId = item.beforeState?.playlistItemId ?? item.entityId;
        resp = await providerFetch(
          `${YT_BASE}/playlistItems?id=${encodeURIComponent(playlistItemId)}`,
          { method: "DELETE" },
          tokenInfo.accessToken,
        );
      } else {
        await ctx.runMutation(api.enforcement._updateItemStatus, {
          itemId: item.id,
          status: "skipped",
          afterState: { reason: "unsupported_action" },
        });
        continue;
      }

      if (resp.ok || resp.status === 200 || resp.status === 204) {
        const afterState =
          item.entityType === "playlist_track"
            ? { removedFromPlaylist: item.beforeState?.playlistId }
            : item.entityType === "artist"
              ? { unsubscribed: true }
              : { unliked: true };

        await ctx.runMutation(api.enforcement._updateItemStatus, {
          itemId: item.id,
          status: "completed",
          afterState,
        });
        completedCount++;
      } else {
        const errText = await resp.text();
        await ctx.runMutation(api.enforcement._updateItemStatus, {
          itemId: item.id,
          status: "failed",
          errorMessage: `YouTube API error ${resp.status}: ${errText.substring(0, 200)}`,
        });
        failedCount++;
      }
    } catch (err: any) {
      await ctx.runMutation(api.enforcement._updateItemStatus, {
        itemId: item.id,
        status: "failed",
        errorMessage: err.message?.substring(0, 200) ?? "Unknown error",
      });
      failedCount++;
    }
  }

  return { completed: completedCount, failed: failedCount, suspended: false };
}

// ---------------------------------------------------------------------------
// Rollback
// ---------------------------------------------------------------------------

export const rollback = action({
  args: {
    batchId: v.string(),
  },
  handler: async (ctx, args) => {
    // Fetch completed action items for this batch
    const completedItems: Array<{
      id: string;
      entityType: string;
      entityId: string;
      action: string;
      beforeState: any;
    }> = await ctx.runQuery(api.enforcement._getCompletedItems, {
      batchId: args.batchId,
    });

    const batch: { provider: string } | null = await ctx.runQuery(
      api.enforcement._getBatch,
      { batchId: args.batchId },
    );

    const provider = batch?.provider ?? "spotify";
    let rolledBack = 0;
    let rollbackFailed = 0;

    if (completedItems.length === 0) {
      await ctx.runMutation(api.enforcement._updateBatchStatus, {
        batchId: args.batchId,
        status: "rolled_back",
      });
      return {
        success: true,
        message: "Nothing to roll back.",
        rolledBack: 0,
        failed: 0,
      };
    }

    let tokenInfo: { accessToken: string; refreshToken?: string; connection: any };

    try {
      tokenInfo = await getAccessToken(ctx, provider);
    } catch {
      await ctx.runMutation(api.enforcement._updateBatchStatus, {
        batchId: args.batchId,
        status: "failed",
      });
      return {
        success: false,
        message: `No active ${provider} connection for rollback.`,
        rolledBack: 0,
        failed: completedItems.length,
      };
    }

    if (provider === "spotify") {
      const result = await rollbackSpotify(ctx, completedItems, tokenInfo, provider);
      rolledBack = result.rolledBack;
      rollbackFailed = result.failed;
    } else if (provider === "tidal") {
      const result = await rollbackTidal(ctx, completedItems, tokenInfo, provider);
      rolledBack = result.rolledBack;
      rollbackFailed = result.failed;
    } else if (provider === "youtube") {
      const result = await rollbackYouTube(ctx, completedItems, tokenInfo, provider);
      rolledBack = result.rolledBack;
      rollbackFailed = result.failed;
    }

    await ctx.runMutation(api.enforcement._updateBatchStatus, {
      batchId: args.batchId,
      status: "rolled_back",
    });

    return {
      success: rollbackFailed === 0,
      message: `Rollback completed: ${rolledBack} restored, ${rollbackFailed} failed.`,
      rolledBack,
      failed: rollbackFailed,
    };
  },
});

// ---------------------------------------------------------------------------
// Provider-specific rollback executors
// ---------------------------------------------------------------------------

type RollbackResult = { rolledBack: number; failed: number };

async function rollbackSpotify(
  ctx: any,
  completedItems: ActionItem[],
  tokenInfo: { accessToken: string; refreshToken?: string },
  provider: string,
): Promise<RollbackResult> {
  let rolledBack = 0;
  let rollbackFailed = 0;

  // Re-add removed tracks to library
  const trackIdsToRestore = completedItems
    .filter((i) => i.entityType === "track" && i.action === "remove")
    .map((i) => i.entityId);

  for (let i = 0; i < trackIdsToRestore.length; i += 50) {
    const chunk = trackIdsToRestore.slice(i, i + 50);
    try {
      const resp = await spotifyFetch(
        "https://api.spotify.com/v1/me/tracks",
        {
          method: "PUT",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ ids: chunk }),
        },
        tokenInfo.accessToken,
        tokenInfo.refreshToken,
        ctx,
        provider,
      );

      if (resp.ok || resp.status === 200) {
        rolledBack += chunk.length;
        for (const trackId of chunk) {
          const item = completedItems.find(
            (it) => it.entityId === trackId && it.entityType === "track",
          );
          if (item) {
            await ctx.runMutation(
              api.enforcement._updateItemStatus,
              {
                itemId: item.id,
                status: "rolled_back",
                afterState: { restored: true },
              },
            );
          }
        }
      } else {
        rollbackFailed += chunk.length;
      }
    } catch {
      rollbackFailed += chunk.length;
    }
  }

  // Re-follow unfollowed artists
  const artistIdsToRefollow = completedItems
    .filter((i) => i.entityType === "artist" && i.action === "unfollow")
    .map((i) => i.entityId);

  for (let i = 0; i < artistIdsToRefollow.length; i += 50) {
    const chunk = artistIdsToRefollow.slice(i, i + 50);
    try {
      const resp = await spotifyFetch(
        `https://api.spotify.com/v1/me/following?type=artist&ids=${chunk.join(",")}`,
        { method: "PUT" },
        tokenInfo.accessToken,
        tokenInfo.refreshToken,
        ctx,
        provider,
      );

      if (resp.ok || resp.status === 200 || resp.status === 204) {
        rolledBack += chunk.length;
        for (const artistId of chunk) {
          const item = completedItems.find(
            (it) =>
              it.entityId === artistId && it.entityType === "artist",
          );
          if (item) {
            await ctx.runMutation(
              api.enforcement._updateItemStatus,
              {
                itemId: item.id,
                status: "rolled_back",
                afterState: { refollowed: true },
              },
            );
          }
        }
      } else {
        rollbackFailed += chunk.length;
      }
    } catch {
      rollbackFailed += chunk.length;
    }
  }

  // Re-add tracks to playlists
  const playlistTrackItems = completedItems.filter(
    (i) => i.entityType === "playlist_track" && i.action === "remove_playlist_track",
  );

  // Group by playlistId for batch re-addition
  const playlistGroups: Map<string, ActionItem[]> = new Map();
  for (const item of playlistTrackItems) {
    const playlistId = item.beforeState?.playlistId;
    if (!playlistId) continue;
    if (!playlistGroups.has(playlistId)) {
      playlistGroups.set(playlistId, []);
    }
    playlistGroups.get(playlistId)!.push(item);
  }

  for (const [playlistId, groupItems] of playlistGroups) {
    // Spotify POST /v1/playlists/{id}/tracks to re-add, up to 100 URIs per request
    for (let i = 0; i < groupItems.length; i += 100) {
      const chunk = groupItems.slice(i, i + 100);
      const uris = chunk.map(
        (item) => item.beforeState?.trackUri ?? `spotify:track:${item.entityId}`,
      );

      try {
        const resp = await spotifyFetch(
          `https://api.spotify.com/v1/playlists/${playlistId}/tracks`,
          {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ uris }),
          },
          tokenInfo.accessToken,
          tokenInfo.refreshToken,
          ctx,
          provider,
        );

        if (resp.ok || resp.status === 201 || resp.status === 200) {
          rolledBack += chunk.length;
          for (const item of chunk) {
            await ctx.runMutation(
              api.enforcement._updateItemStatus,
              {
                itemId: item.id,
                status: "rolled_back",
                afterState: { restoredToPlaylist: playlistId },
              },
            );
          }
        } else {
          rollbackFailed += chunk.length;
        }
      } catch {
        rollbackFailed += chunk.length;
      }
    }
  }

  return { rolledBack, failed: rollbackFailed };
}

async function rollbackTidal(
  ctx: any,
  completedItems: ActionItem[],
  tokenInfo: { accessToken: string; refreshToken?: string; connection: any },
  provider: string,
): Promise<RollbackResult> {
  let rolledBack = 0;
  let rollbackFailed = 0;

  const tidalUserId = tokenInfo.connection?.providerUserId;
  if (!tidalUserId) {
    return { rolledBack: 0, failed: completedItems.length };
  }

  const TIDAL_BASE = "https://openapi.tidal.com/v1";

  for (const item of completedItems) {
    try {
      let resp: Response;

      if (item.entityType === "track" && item.action === "remove") {
        // Re-favorite the track
        resp = await providerFetch(
          `${TIDAL_BASE}/users/${tidalUserId}/favorites/tracks`,
          {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ trackId: item.entityId }),
          },
          tokenInfo.accessToken,
        );
      } else if (item.entityType === "artist" && item.action === "unfollow") {
        // Re-follow the artist
        resp = await providerFetch(
          `${TIDAL_BASE}/users/${tidalUserId}/favorites/artists`,
          {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ artistId: item.entityId }),
          },
          tokenInfo.accessToken,
        );
      } else if (item.entityType === "playlist_track" && item.action === "remove_playlist_track") {
        // Re-add track to playlist
        const playlistId = item.beforeState?.playlistId;
        if (!playlistId) {
          rollbackFailed++;
          continue;
        }
        resp = await providerFetch(
          `${TIDAL_BASE}/playlists/${playlistId}/items`,
          {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ trackId: item.entityId }),
          },
          tokenInfo.accessToken,
        );
      } else {
        continue;
      }

      if (resp.ok || resp.status === 200 || resp.status === 201 || resp.status === 204) {
        rolledBack++;
        await ctx.runMutation(api.enforcement._updateItemStatus, {
          itemId: item.id,
          status: "rolled_back",
          afterState: { restored: true },
        });
      } else {
        rollbackFailed++;
      }
    } catch {
      rollbackFailed++;
    }
  }

  return { rolledBack, failed: rollbackFailed };
}

async function rollbackYouTube(
  ctx: any,
  completedItems: ActionItem[],
  tokenInfo: { accessToken: string; refreshToken?: string },
  provider: string,
): Promise<RollbackResult> {
  let rolledBack = 0;
  let rollbackFailed = 0;

  const YT_BASE = "https://www.googleapis.com/youtube/v3";

  for (const item of completedItems) {
    try {
      let resp: Response;

      if (item.entityType === "track" && item.action === "remove") {
        // Re-like the video
        resp = await providerFetch(
          `${YT_BASE}/videos/rate?id=${encodeURIComponent(item.entityId)}&rating=like`,
          {
            method: "POST",
            headers: { "Content-Type": "application/json" },
          },
          tokenInfo.accessToken,
        );
      } else if (item.entityType === "artist" && item.action === "unfollow") {
        // Re-subscribe to channel
        const channelId = item.beforeState?.channelId;
        if (!channelId) {
          rollbackFailed++;
          continue;
        }
        resp = await providerFetch(
          `${YT_BASE}/subscriptions?part=snippet`,
          {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              snippet: {
                resourceId: {
                  kind: "youtube#channel",
                  channelId,
                },
              },
            }),
          },
          tokenInfo.accessToken,
        );
      } else if (item.entityType === "playlist_track" && item.action === "remove_playlist_track") {
        // Re-add to playlist
        const playlistId = item.beforeState?.playlistId;
        const videoId = item.beforeState?.videoId ?? item.entityId;
        if (!playlistId) {
          rollbackFailed++;
          continue;
        }
        resp = await providerFetch(
          `${YT_BASE}/playlistItems?part=snippet`,
          {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              snippet: {
                playlistId,
                resourceId: {
                  kind: "youtube#video",
                  videoId,
                },
              },
            }),
          },
          tokenInfo.accessToken,
        );
      } else {
        continue;
      }

      if (resp.ok || resp.status === 200 || resp.status === 204) {
        rolledBack++;
        await ctx.runMutation(api.enforcement._updateItemStatus, {
          itemId: item.id,
          status: "rolled_back",
          afterState: { restored: true },
        });
      } else {
        rollbackFailed++;
      }
    } catch {
      rollbackFailed++;
    }
  }

  return { rolledBack, failed: rollbackFailed };
}

// ---------------------------------------------------------------------------
// Internal mutations and queries
// ---------------------------------------------------------------------------

export const _createBatch = mutation({
  args: {
    provider: v.string(),
    options: v.any(),
    dryRun: v.optional(v.boolean()),
  },
  handler: async (ctx, args) => {
    const { user } = await requireCurrentUser(ctx);
    const now = nowIso();

    const batchId = await ctx.db.insert("actionBatches", {
      legacyKey: `runtime:batch:${user._id}:${Date.now()}`,
      userId: user._id,
      provider: args.provider,
      idempotencyKey: `batch-${Date.now()}`,
      dryRun: args.dryRun,
      status: "pending",
      options: args.options,
      summary: {
        totalItems: 0,
        completedItems: 0,
        failedItems: 0,
        skippedItems: 0,
      },
      createdAt: now,
      updatedAt: now,
    });

    return batchId;
  },
});

export const _updateBatchStatus = mutation({
  args: {
    batchId: v.string(),
    status: v.string(),
  },
  handler: async (ctx, args) => {
    const batch = await ctx.db.get(args.batchId as any);
    if (!batch) {
      throw new ConvexError("Batch not found.");
    }

    const update: any = {
      status: args.status,
      updatedAt: nowIso(),
    };

    if (
      args.status === "completed" ||
      args.status === "failed" ||
      args.status === "rolled_back"
    ) {
      update.completedAt = nowIso();
    }

    await ctx.db.patch(batch._id, update);
    return { success: true };
  },
});

export const _updateBatchSummary = mutation({
  args: {
    batchId: v.string(),
    summary: v.any(),
  },
  handler: async (ctx, args) => {
    const batch = await ctx.db.get(args.batchId as any);
    if (!batch) throw new ConvexError("Batch not found.");
    await ctx.db.patch(batch._id, {
      summary: args.summary,
      updatedAt: nowIso(),
    });
  },
});

export const _computeImpact = query({
  args: {
    provider: v.string(),
  },
  handler: async (ctx: QueryCtx, args) => {
    const { user } = await requireCurrentUser(ctx);

    // Get user's library tracks for this provider
    const tracks = await ctx.db
      .query("userLibraryTracks")
      .withIndex("by_user_provider", (q) =>
        q.eq("userId", user._id).eq("provider", args.provider),
      )
      .collect();

    // Get user's blocked artists
    const blocks = await ctx.db
      .query("userArtistBlocks")
      .withIndex("by_userId", (q) => q.eq("userId", user._id))
      .collect();
    const blockedArtistIdSet = new Set(blocks.map((b) => b.artistId as string));

    // Get user's category subscriptions to filter relevant offenses
    const catSubs = await ctx.db
      .query("categorySubscriptions")
      .withIndex("by_userId", (q) => q.eq("userId", user._id))
      .collect();
    const subscribedCategories = new Set(catSubs.map((s) => s.category));

    // Only include offenses matching the user's subscribed categories.
    // If the user has no category subscriptions, fall back to all offenses
    // so the default experience still flags known offenders.
    const allOffenses = await ctx.db.query("artistOffenses").collect();
    const relevantOffenses = subscribedCategories.size > 0
      ? allOffenses.filter((o) => subscribedCategories.has(o.category))
      : allOffenses;
    const offendingArtistIdSet = new Set(
      relevantOffenses.map((o) => o.artistId as string),
    );

    // Merge blocked + offending
    const allFlaggedIds = new Set([
      ...blockedArtistIdSet,
      ...offendingArtistIdSet,
    ]);

    // Find flagged tracks (library liked songs)
    const flaggedTracks: Array<{
      trackId: string;
      providerTrackId: string;
      trackName: string;
      artistId: string;
      artistName: string;
      playlistName: string | undefined;
    }> = [];

    // Find playlist tracks to remove (separate from library liked songs)
    const playlistTrackRemovals: Array<{
      providerTrackId: string;
      trackName: string;
      artistName: string;
      playlistName: string;
      playlistId: string;
      trackUri: string;
    }> = [];

    const flaggedArtistNames = new Set<string>();
    const playlistsAffected = new Set<string>();
    const blockedArtistProviderIds = new Set<string>();

    // Track playlist-level details: which playlists and how many tracks in each
    const playlistTrackCounts: Map<string, number> = new Map();

    for (const track of tracks) {
      const aid = track.artistId as string | undefined;
      if (aid && allFlaggedIds.has(aid)) {
        if (track.artistName) flaggedArtistNames.add(track.artistName);
        blockedArtistProviderIds.add(aid);

        if (track.playlistName) {
          // This is a playlist track
          playlistsAffected.add(track.playlistName);

          // Extract playlist ID from metadata if available, else use playlistName as identifier
          const meta = track.metadata as Record<string, any> | undefined;
          const playlistId = meta?.playlistId ?? meta?.playlist_id ?? track.playlistName;

          playlistTrackRemovals.push({
            providerTrackId: track.providerTrackId,
            trackName: track.trackName ?? "Unknown",
            artistName: track.artistName ?? "Unknown",
            playlistName: track.playlistName,
            playlistId,
            trackUri: meta?.trackUri ?? `spotify:track:${track.providerTrackId}`,
          });

          // Count tracks per playlist for details
          const currentCount = playlistTrackCounts.get(track.playlistName) ?? 0;
          playlistTrackCounts.set(track.playlistName, currentCount + 1);
        } else {
          // This is a library liked song (no playlist context)
          flaggedTracks.push({
            trackId: track._id as string,
            providerTrackId: track.providerTrackId,
            trackName: track.trackName ?? "Unknown",
            artistId: aid,
            artistName: track.artistName ?? "Unknown",
            playlistName: undefined,
          });
        }
      }
    }

    // Resolve external artist IDs for unfollowing
    const blockedArtistIds: string[] = [];
    for (const artistId of blockedArtistProviderIds) {
      const artist = await ctx.db.get(artistId as any);
      if (artist) {
        const externalIds = (artist as any).externalIds as
          | Record<string, string>
          | undefined;

        // Support multiple providers
        let externalArtistId: string | undefined;
        if (args.provider === "spotify") {
          externalArtistId = externalIds?.spotify ?? externalIds?.spotifyId;
        } else if (args.provider === "tidal") {
          externalArtistId = externalIds?.tidal ?? externalIds?.tidalId;
        } else if (args.provider === "youtube") {
          externalArtistId = externalIds?.youtube ?? externalIds?.youtubeId;
        }

        if (externalArtistId) {
          blockedArtistIds.push(externalArtistId);
        }
      }
    }

    // Build playlist details for the response
    const playlistDetails: Array<{ playlistName: string; tracksToRemove: number }> = [];
    for (const [name, count] of playlistTrackCounts) {
      playlistDetails.push({ playlistName: name, tracksToRemove: count });
    }

    return {
      flaggedTracks: flaggedTracks.map((t) => ({
        providerTrackId: t.providerTrackId,
        trackName: t.trackName,
        artistName: t.artistName,
        playlistName: t.playlistName,
      })),
      blockedArtistIds,
      flaggedArtistNames: [...flaggedArtistNames],
      playlistsAffected: playlistsAffected.size,
      playlistTracksToRemove: playlistTrackRemovals.length,
      playlistTrackRemovals: playlistTrackRemovals.map((t) => ({
        providerTrackId: t.providerTrackId,
        trackName: t.trackName,
        artistName: t.artistName,
        playlistName: t.playlistName,
        playlistId: t.playlistId,
        trackUri: t.trackUri,
      })),
      playlistDetails,
    };
  },
});

export const _createPlanItems = mutation({
  args: {
    batchId: v.string(),
    flaggedTracks: v.array(
      v.object({
        providerTrackId: v.string(),
        trackName: v.string(),
        artistName: v.string(),
        playlistName: v.optional(v.string()),
      }),
    ),
    blockedArtistIds: v.array(v.string()),
    playlistTrackRemovals: v.optional(
      v.array(
        v.object({
          providerTrackId: v.string(),
          trackName: v.string(),
          artistName: v.string(),
          playlistName: v.string(),
          playlistId: v.string(),
          trackUri: v.string(),
        }),
      ),
    ),
  },
  handler: async (ctx, args) => {
    const now = nowIso();
    const batchId = args.batchId as any;

    // Create action items for tracks to remove from library
    for (const track of args.flaggedTracks) {
      await ctx.db.insert("actionItems", {
        legacyKey: `runtime:item:${args.batchId}:track:${track.providerTrackId}`,
        batchId,
        entityType: "track",
        entityId: track.providerTrackId,
        action: "remove",
        idempotencyKey: `remove-track-${track.providerTrackId}`,
        beforeState: {
          trackName: track.trackName,
          artistName: track.artistName,
          playlistName: track.playlistName,
          inLibrary: true,
        },
        afterState: {},
        status: "pending",
        createdAt: now,
        updatedAt: now,
      });
    }

    // Create action items for artists to unfollow
    for (const artistId of args.blockedArtistIds) {
      await ctx.db.insert("actionItems", {
        legacyKey: `runtime:item:${args.batchId}:artist:${artistId}`,
        batchId,
        entityType: "artist",
        entityId: artistId,
        action: "unfollow",
        idempotencyKey: `unfollow-artist-${artistId}`,
        beforeState: { followed: true },
        afterState: {},
        status: "pending",
        createdAt: now,
        updatedAt: now,
      });
    }

    // Create action items for tracks to remove from playlists
    const playlistTrackRemovals = args.playlistTrackRemovals ?? [];
    for (const pt of playlistTrackRemovals) {
      await ctx.db.insert("actionItems", {
        legacyKey: `runtime:item:${args.batchId}:playlist_track:${pt.playlistId}:${pt.providerTrackId}`,
        batchId,
        entityType: "playlist_track",
        entityId: pt.providerTrackId,
        action: "remove_playlist_track",
        idempotencyKey: `remove-playlist-track-${pt.playlistId}-${pt.providerTrackId}`,
        beforeState: {
          trackName: pt.trackName,
          artistName: pt.artistName,
          playlistName: pt.playlistName,
          playlistId: pt.playlistId,
          trackUri: pt.trackUri,
        },
        afterState: {},
        status: "pending",
        createdAt: now,
        updatedAt: now,
      });
    }
  },
});

export const _getConnection = query({
  args: {
    provider: v.string(),
  },
  handler: async (ctx: QueryCtx, args) => {
    const { user } = await requireCurrentUser(ctx);

    const connection = await ctx.db
      .query("providerConnections")
      .withIndex("by_user_provider", (q) =>
        q.eq("userId", user._id).eq("provider", args.provider),
      )
      .unique();

    if (!connection || connection.status !== "active") {
      return null;
    }

    return {
      connectionId: connection._id,
      accessToken: connection.encryptedAccessToken ?? null,
      refreshToken: connection.encryptedRefreshToken ?? null,
      expiresAt: connection.expiresAt ?? null,
      providerUserId: connection.providerUserId ?? null,
    };
  },
});

export const _updateConnectionToken = mutation({
  args: {
    provider: v.string(),
    accessToken: v.string(),
    expiresAt: v.optional(v.string()),
  },
  handler: async (ctx: MutationCtx, args) => {
    const { user } = await requireCurrentUser(ctx);

    const connection = await ctx.db
      .query("providerConnections")
      .withIndex("by_user_provider", (q) =>
        q.eq("userId", user._id).eq("provider", args.provider),
      )
      .unique();

    if (!connection) {
      throw new ConvexError("Connection not found.");
    }

    await ctx.db.patch(connection._id, {
      encryptedAccessToken: args.accessToken,
      expiresAt: args.expiresAt,
      updatedAt: nowIso(),
    });
  },
});

export const _updateItemStatus = mutation({
  args: {
    itemId: v.string(),
    status: v.string(),
    afterState: v.optional(v.any()),
    errorMessage: v.optional(v.string()),
  },
  handler: async (ctx, args) => {
    const item = await ctx.db.get(args.itemId as any);
    if (!item) return;

    const update: any = {
      status: args.status,
      updatedAt: nowIso(),
    };
    if (args.afterState !== undefined) update.afterState = args.afterState;
    if (args.errorMessage !== undefined) update.errorMessage = args.errorMessage;

    await ctx.db.patch(item._id, update);
  },
});

export const _getPendingItems = query({
  args: {
    batchId: v.string(),
  },
  handler: async (ctx, args) => {
    const items = await ctx.db
      .query("actionItems")
      .withIndex("by_batchId", (q) => q.eq("batchId", args.batchId as any))
      .collect();

    return items
      .filter((i) => i.status === "pending")
      .map((i) => ({
        id: i._id as string,
        entityType: i.entityType,
        entityId: i.entityId,
        action: i.action,
        beforeState: i.beforeState,
      }));
  },
});

export const _getCompletedItems = query({
  args: {
    batchId: v.string(),
  },
  handler: async (ctx, args) => {
    const items = await ctx.db
      .query("actionItems")
      .withIndex("by_batchId", (q) => q.eq("batchId", args.batchId as any))
      .collect();

    return items
      .filter((i) => i.status === "completed")
      .map((i) => ({
        id: i._id as string,
        entityType: i.entityType,
        entityId: i.entityId,
        action: i.action,
        beforeState: i.beforeState,
      }));
  },
});

export const _getBatch = query({
  args: {
    batchId: v.string(),
  },
  handler: async (ctx, args) => {
    const batch = await ctx.db.get(args.batchId as Id<"actionBatches">);
    if (!batch) return null;
    return { provider: batch.provider };
  },
});
