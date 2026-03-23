import { ConvexError, v } from "convex/values";
import type { Doc } from "./_generated/dataModel";
import { action, mutation, query } from "./_generated/server";
import { nowIso, requireCurrentUser } from "./lib/auth";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Retrieve the stored access token for a provider connection. */
async function getAccessToken(
  ctx: any,
  provider: string,
): Promise<{ accessToken: string; refreshToken?: string; connection: any }> {
  const connection = await ctx.runQuery(
    "enforcement:_getConnection" as any,
    { provider },
  );
  if (!connection || !connection.accessToken) {
    throw new ConvexError(
      `No active ${provider} connection found. Please connect your account first.`,
    );
  }
  return connection;
}

/**
 * Try to refresh an expired Spotify token.
 * Returns the new access token or throws.
 */
async function refreshSpotifyToken(
  refreshToken: string,
): Promise<{ access_token: string; expires_in: number }> {
  const clientId = process.env.SPOTIFY_CLIENT_ID;
  const clientSecret = process.env.SPOTIFY_CLIENT_SECRET;
  if (!clientId || !clientSecret) {
    throw new ConvexError("Spotify credentials not configured.");
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
    throw new ConvexError(`Spotify token refresh failed: ${resp.status} ${text}`);
  }

  return (await resp.json()) as { access_token: string; expires_in: number };
}

/**
 * Wrapper around fetch that auto-retries once on 401 by refreshing the token.
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

    // Persist the new token
    await ctx.runMutation("enforcement:_updateConnectionToken" as any, {
      provider,
      accessToken: refreshed.access_token,
      expiresAt,
    });

    // Retry the original request
    headers.Authorization = `Bearer ${refreshed.access_token}`;
    resp = await fetch(url, { ...opts, headers });
  }

  return resp;
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
      "enforcement:_createBatch" as any,
      {
        provider,
        options: args.options,
        dryRun: args.dryRun ?? true,
      },
    );

    // Compute the real impact by cross-referencing the user's library with blocks
    const impact = await ctx.runQuery(
      "enforcement:_computeImpact" as any,
      { provider },
    );

    // Create action items for each flagged track / artist
    if (impact.flaggedTracks.length > 0 || impact.blockedArtistIds.length > 0) {
      await ctx.runMutation("enforcement:_createPlanItems" as any, {
        batchId,
        flaggedTracks: impact.flaggedTracks,
        blockedArtistIds: impact.blockedArtistIds,
      });
    }

    // Update the batch summary with real counts
    await ctx.runMutation("enforcement:_updateBatchSummary" as any, {
      batchId,
      summary: {
        totalItems:
          impact.flaggedTracks.length + impact.blockedArtistIds.length,
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
        flaggedArtistNames: impact.flaggedArtistNames,
      },
      capabilities: {
        removeLibraryTracks: true,
        removePlaylistTracks: provider === "spotify",
        unfollowArtists: provider === "spotify",
      },
      estimatedDuration:
        impact.flaggedTracks.length > 50 ? "~5 minutes" : "~2 minutes",
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
    const dryRun = args.dryRun ?? false;

    // Mark batch as running
    await ctx.runMutation("enforcement:_updateBatchStatus" as any, {
      batchId: args.planId,
      status: "running",
    });

    // Fetch all pending action items for this batch
    const items: Array<{
      id: string;
      entityType: string;
      entityId: string;
      action: string;
    }> = await ctx.runQuery("enforcement:_getPendingItems" as any, {
      batchId: args.planId,
    });

    // Fetch the batch to know the provider
    const batch: { provider: string } | null = await ctx.runQuery(
      "enforcement:_getBatch" as any,
      { batchId: args.planId },
    );

    const provider = batch?.provider ?? "spotify";

    let completedCount = 0;
    let failedCount = 0;

    if (!dryRun && provider === "spotify") {
      // Get access token for Spotify API calls
      let tokenInfo: {
        accessToken: string;
        refreshToken?: string;
      };

      try {
        tokenInfo = await getAccessToken(ctx, provider);
      } catch {
        await ctx.runMutation("enforcement:_updateBatchStatus" as any, {
          batchId: args.planId,
          status: "failed",
        });
        return {
          id: args.planId,
          status: "failed",
          message: `No active ${provider} connection. Please connect your account first.`,
          completed: 0,
          failed: items.length,
        };
      }

      // Process items in batches of up to 50 (Spotify API limit)
      const trackIdsToRemove: string[] = [];
      const artistIdsToUnfollow: string[] = [];

      for (const item of items) {
        if (item.entityType === "track" && item.action === "remove") {
          trackIdsToRemove.push(item.entityId);
        } else if (item.entityType === "artist" && item.action === "unfollow") {
          artistIdsToUnfollow.push(item.entityId);
        }
      }

      // Remove tracks from library in chunks of 50
      for (let i = 0; i < trackIdsToRemove.length; i += 50) {
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
            // Mark these items as completed
            for (const trackId of chunk) {
              const item = items.find(
                (it) => it.entityId === trackId && it.entityType === "track",
              );
              if (item) {
                await ctx.runMutation(
                  "enforcement:_updateItemStatus" as any,
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
                  "enforcement:_updateItemStatus" as any,
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
                "enforcement:_updateItemStatus" as any,
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

      // Unfollow artists in chunks of 50
      for (let i = 0; i < artistIdsToUnfollow.length; i += 50) {
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
                  "enforcement:_updateItemStatus" as any,
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
                  "enforcement:_updateItemStatus" as any,
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
                "enforcement:_updateItemStatus" as any,
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
    } else if (dryRun) {
      // In dry-run mode, mark all items as skipped
      for (const item of items) {
        await ctx.runMutation("enforcement:_updateItemStatus" as any, {
          itemId: item.id,
          status: "skipped",
          afterState: { dryRun: true },
        });
      }
      completedCount = 0;
      failedCount = 0;
    }

    // Mark batch as completed or failed
    const finalStatus = failedCount > 0 && completedCount === 0
      ? "failed"
      : "completed";

    await ctx.runMutation("enforcement:_updateBatchStatus" as any, {
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
    }> = await ctx.runQuery("enforcement:_getCompletedItems" as any, {
      batchId: args.batchId,
    });

    const batch: { provider: string } | null = await ctx.runQuery(
      "enforcement:_getBatch" as any,
      { batchId: args.batchId },
    );

    const provider = batch?.provider ?? "spotify";
    let rolledBack = 0;
    let rollbackFailed = 0;

    if (provider === "spotify" && completedItems.length > 0) {
      let tokenInfo: { accessToken: string; refreshToken?: string };

      try {
        tokenInfo = await getAccessToken(ctx, provider);
      } catch {
        await ctx.runMutation("enforcement:_updateBatchStatus" as any, {
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
                  "enforcement:_updateItemStatus" as any,
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
                  "enforcement:_updateItemStatus" as any,
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
    }

    await ctx.runMutation("enforcement:_updateBatchStatus" as any, {
      batchId: args.batchId,
      status: "rolled_back",
    });

    return {
      success: rollbackFailed === 0,
      message:
        completedItems.length === 0
          ? "Nothing to roll back."
          : `Rollback completed: ${rolledBack} restored, ${rollbackFailed} failed.`,
      rolledBack,
      failed: rollbackFailed,
    };
  },
});

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
  handler: async (ctx, args) => {
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

    // Also check artist offenses (global blocklist)
    const allOffenses = await ctx.db.query("artistOffenses").collect();
    const offendingArtistIdSet = new Set(
      allOffenses.map((o) => o.artistId as string),
    );

    // Merge blocked + offending
    const allFlaggedIds = new Set([
      ...blockedArtistIdSet,
      ...offendingArtistIdSet,
    ]);

    // Find flagged tracks
    const flaggedTracks: Array<{
      trackId: string;
      providerTrackId: string;
      trackName: string;
      artistId: string;
      artistName: string;
      playlistName: string | undefined;
    }> = [];

    const flaggedArtistNames = new Set<string>();
    const playlistsAffected = new Set<string>();
    const blockedArtistProviderIds = new Set<string>();

    for (const track of tracks) {
      const aid = track.artistId as string | undefined;
      if (aid && allFlaggedIds.has(aid)) {
        flaggedTracks.push({
          trackId: track._id as string,
          providerTrackId: track.providerTrackId,
          trackName: track.trackName ?? "Unknown",
          artistId: aid,
          artistName: track.artistName ?? "Unknown",
          playlistName: track.playlistName ?? undefined,
        });
        if (track.artistName) flaggedArtistNames.add(track.artistName);
        if (track.playlistName) playlistsAffected.add(track.playlistName);

        // Collect unique blocked artist IDs that have external provider IDs
        // For Spotify, the providerTrackId can help us but we need artist external IDs
        // We store the internal artist IDs and look them up
        blockedArtistProviderIds.add(aid);
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
        const spotifyArtistId = externalIds?.spotify ?? externalIds?.spotifyId;
        if (spotifyArtistId) {
          blockedArtistIds.push(spotifyArtistId);
        }
      }
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
  },
  handler: async (ctx, args) => {
    const now = nowIso();
    const batchId = args.batchId as any;

    // Create action items for tracks to remove
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
  },
});

export const _getConnection = query({
  args: {
    provider: v.string(),
  },
  handler: async (ctx, args) => {
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
    };
  },
});

export const _updateConnectionToken = mutation({
  args: {
    provider: v.string(),
    accessToken: v.string(),
    expiresAt: v.optional(v.string()),
  },
  handler: async (ctx, args) => {
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
    const batch = await ctx.db.get(args.batchId as any);
    if (!batch) return null;
    return { provider: batch.provider };
  },
});
