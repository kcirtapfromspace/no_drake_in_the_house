import { ConvexError, v } from "convex/values";
import { action, internalAction, internalMutation, internalQuery, mutation, query } from "./_generated/server";
import { api, internal } from "./_generated/api";
import { Id } from "./_generated/dataModel";
import { nowIso, requireCurrentUser } from "./lib/auth";

/** Check if a sync is already running for a given provider. */
/** Max age for a "running" sync before it's considered stale (35 min). */
const STALE_RUN_MS = 35 * 60 * 1000;

export const _getRunningSync = internalQuery({
  args: {
    platform: v.string(),
  },
  handler: async (ctx, args) => {
    const running = await ctx.db
      .query("platformSyncRuns")
      .withIndex("by_status", (q) => q.eq("status", "running"))
      .collect();

    const now = Date.now();
    const match = running.find((r) => {
      if (r.platform !== args.platform) return false;
      // Ignore stale runs — the action likely crashed or timed out
      const startedMs = new Date(r.startedAt ?? r.createdAt).getTime();
      return now - startedMs < STALE_RUN_MS;
    });
    return match ? { runId: match._id } : null;
  },
});

export const status = query({
  args: {},
  handler: async (ctx) => {
    const { user } = await requireCurrentUser(ctx);
    const runs = await ctx.db
      .query("platformSyncRuns")
      .withIndex("by_status", (q) => q.eq("status", "running"))
      .collect();

    const latestByPlatform = new Map<string, any>();
    for (const run of runs) {
      const existing = latestByPlatform.get(run.platform);
      if (
        !existing ||
        (run.startedAt ?? run.createdAt) >
          (existing.startedAt ?? existing.createdAt)
      ) {
        latestByPlatform.set(run.platform, run);
      }
    }

    return {
      active_syncs: runs.length,
      platforms: Object.fromEntries(
        [...latestByPlatform.entries()].map(([platform, run]) => [
          platform,
          {
            status: run.status,
            started_at: run.startedAt,
            platform: run.platform,
          },
        ]),
      ),
    };
  },
});

export const listRuns = query({
  args: {
    platform: v.optional(v.string()),
    limit: v.optional(v.number()),
  },
  handler: async (ctx, args) => {
    await requireCurrentUser(ctx);
    let runsQuery = args.platform
      ? ctx.db
          .query("platformSyncRuns")
          .withIndex("by_platform", (q) => q.eq("platform", args.platform!))
      : ctx.db.query("platformSyncRuns");

    const allRuns = await runsQuery.collect();
    allRuns.sort((a, b) =>
      (b.startedAt ?? b.createdAt).localeCompare(a.startedAt ?? a.createdAt),
    );

    const limited = allRuns.slice(0, args.limit ?? 20);

    return {
      runs: limited.map((r) => ({
        id: r._id,
        platform: r.platform,
        status: r.status,
        started_at: r.startedAt,
        completed_at: r.completedAt,
        error_log: r.errorLog,
        metadata: r.metadata,
        created_at: r.createdAt,
      })),
      total: allRuns.length,
    };
  },
});

export const getRun = query({
  args: {
    runId: v.id("platformSyncRuns"),
  },
  handler: async (ctx, args) => {
    await requireCurrentUser(ctx);
    const run = await ctx.db.get(args.runId);
    if (!run) return null;

    return {
      id: run._id,
      platform: run.platform,
      status: run.status,
      started_at: run.startedAt,
      completed_at: run.completedAt,
      error_log: run.errorLog,
      checkpoint_data: run.checkpointData,
      metadata: run.metadata,
      created_at: run.createdAt,
    };
  },
});

/**
 * Returns the latest sync run for a given provider, including progress data.
 * Used by the frontend to show meaningful sync status (phase, counts, errors).
 */
export const providerSyncStatus = query({
  args: {
    provider: v.string(),
  },
  handler: async (ctx, args) => {
    const runs = await ctx.db
      .query("platformSyncRuns")
      .withIndex("by_platform", (q) => q.eq("platform", args.provider))
      .collect();

    if (runs.length === 0) {
      return { state: "idle" as const, message: "No sync history" };
    }

    runs.sort((a, b) =>
      (b.startedAt ?? b.createdAt).localeCompare(
        a.startedAt ?? a.createdAt,
      ),
    );

    const latest = runs[0];
    const checkpoint = (latest.checkpointData ?? {}) as Record<string, any>;
    const metadata = (latest.metadata ?? {}) as Record<string, any>;

    const state =
      latest.status === "running"
        ? ("running" as const)
        : latest.status === "completed"
          ? ("completed" as const)
          : latest.status === "failed"
            ? ("failed" as const)
            : ("idle" as const);

    const errorLog = (latest.errorLog ?? []) as Array<Record<string, any>>;
    const lastError = errorLog.length > 0 ? errorLog[errorLog.length - 1] : null;

    const message =
      state === "running"
        ? `Syncing${checkpoint.phase ? ` (${checkpoint.phase})` : ""}...`
        : state === "failed"
          ? lastError?.message ?? "Sync failed"
          : state === "completed"
            ? (metadata.playlistTracksBlocked
            ? "Sync complete (playlist tracks unavailable in Spotify dev mode)"
            : "Sync complete")
            : "No sync history";

    return {
      state,
      message,
      started_at: latest.startedAt ?? latest.createdAt,
      completed_at: latest.completedAt,
      phase: checkpoint.phase ?? null,
      tracks_imported: metadata.tracksImported ?? checkpoint.tracksImported ?? 0,
      liked_count: metadata.likedCount ?? checkpoint.likedCount ?? 0,
      album_count: metadata.albumCount ?? checkpoint.albumCount ?? 0,
      artist_count: metadata.artistCount ?? checkpoint.artistCount ?? 0,
      playlist_track_count:
        metadata.playlistTrackCount ?? checkpoint.playlistTrackCount ?? 0,
      playlists_discovered: metadata.playlistsDiscovered ?? (checkpoint.playlistIds ?? []).length,
      playlist_tracks_blocked: metadata.playlistTracksBlocked ?? false,
      duration_ms: metadata.durationMs ?? null,
      error_message: state === "failed" ? (lastError?.message ?? null) : null,
      run_id: latest._id,
    };
  },
});

export const health = query({
  args: {},
  handler: async (ctx) => {
    await requireCurrentUser(ctx);
    const allRuns = await ctx.db.query("platformSyncRuns").collect();
    const recentRuns = allRuns
      .sort((a, b) =>
        (b.startedAt ?? b.createdAt).localeCompare(
          a.startedAt ?? a.createdAt,
        ),
      )
      .slice(0, 50);

    const failedRecent = recentRuns.filter((r) => r.status === "failed").length;
    const runningCount = allRuns.filter((r) => r.status === "running").length;

    const healthy = failedRecent < 5 && runningCount < 10;
    const overall_status = healthy
      ? "healthy" as const
      : failedRecent < 10
        ? "degraded" as const
        : "unhealthy" as const;

    // Build per-platform health from most recent run per platform
    const latestByPlatform = new Map<string, (typeof recentRuns)[0]>();
    for (const run of recentRuns) {
      if (!latestByPlatform.has(run.platform)) {
        latestByPlatform.set(run.platform, run);
      }
    }

    const platforms = [...latestByPlatform.entries()].map(([platform, run]) => ({
      platform,
      is_healthy: run.status === "completed",
      last_check: run.completedAt ?? run.startedAt ?? run.createdAt,
      error: run.status === "failed"
        ? ((run.errorLog as any)?.[0]?.message ?? "Sync failed")
        : undefined,
    }));

    return {
      overall_status,
      healthy,
      total_runs: allRuns.length,
      running: runningCount,
      recent_failures: failedRecent,
      platforms,
      last_run: recentRuns[0]
        ? {
            platform: recentRuns[0].platform,
            status: recentRuns[0].status,
            started_at: recentRuns[0].startedAt,
          }
        : null,
    };
  },
});

export const cancelRun = mutation({
  args: {
    runId: v.id("platformSyncRuns"),
  },
  handler: async (ctx, args) => {
    const run = await ctx.db.get(args.runId);
    if (!run) {
      throw new ConvexError("Sync run not found.");
    }

    if (run.status !== "running") {
      throw new ConvexError("Can only cancel running sync operations.");
    }

    await ctx.db.patch(run._id, {
      status: "cancelled",
      completedAt: nowIso(),
      updatedAt: nowIso(),
    });

    return { success: true };
  },
});

export const triggerSync = action({
  args: {
    platform: v.string(),
  },
  handler: async (ctx, args): Promise<{
    run_id: Id<"platformSyncRuns">;
    platform: string;
    status: string;
    message: string;
  }> => {
    const provider = args.platform;
    const normalizedProvider =
      provider === "apple-music" ? "apple_music" : provider;

    // Guard: reject if a sync is already running for this provider
    const existingRun: { runId: Id<"platformSyncRuns"> } | null = await ctx.runQuery(
      internal.sync._getRunningSync,
      { platform: normalizedProvider },
    );
    if (existingRun) {
      return {
        run_id: existingRun.runId,
        platform: provider,
        status: "already_running",
        message: `A sync is already running for ${provider}`,
      };
    }

    const { runId, userId }: { runId: Id<"platformSyncRuns">; userId: Id<"users"> } = await ctx.runMutation(
      api.sync._createRunWithUser,
      { platform: normalizedProvider },
    );

    // Schedule the provider-specific sync action
    switch (normalizedProvider) {
      case "spotify":
        await ctx.scheduler.runAfter(0, internal.librarySyncActions.syncSpotifyLibrary, { runId, userId });
        break;
      case "tidal":
        await ctx.scheduler.runAfter(0, internal.librarySyncActions.syncTidalLibrary, { runId, userId });
        break;
      case "youtube":
        await ctx.scheduler.runAfter(0, internal.librarySyncActions.syncYouTubeLibrary, { runId, userId });
        break;
      case "apple_music":
        await ctx.scheduler.runAfter(0, internal.librarySyncActions.syncAppleMusicLibrary, { runId, userId });
        break;
      default:
        throw new ConvexError(`Unsupported provider for library sync: ${provider}`);
    }

    return {
      run_id: runId,
      platform: provider,
      status: "running",
      message: `Sync triggered for ${provider}`,
    };
  },
});

export const triggerProviderSync = action({
  args: {
    provider: v.string(),
  },
  handler: async (ctx, args): Promise<{
    run_id: Id<"platformSyncRuns">;
    provider: string;
    status: string;
    message: string;
  }> => {
    const provider = args.provider;

    // Normalize apple-music → apple_music for DB consistency
    const normalizedProvider =
      provider === "apple-music" ? "apple_music" : provider;

    // Guard: reject if a sync is already running for this provider
    const existingRun: { runId: Id<"platformSyncRuns"> } | null = await ctx.runQuery(
      internal.sync._getRunningSync,
      { platform: normalizedProvider },
    );
    if (existingRun) {
      return {
        run_id: existingRun.runId,
        provider,
        status: "already_running",
        message: `A sync is already running for ${provider}`,
      };
    }

    // Create the sync run record and get the user ID in one mutation
    const { runId, userId }: { runId: Id<"platformSyncRuns">; userId: Id<"users"> } = await ctx.runMutation(
      api.sync._createRunWithUser,
      { platform: normalizedProvider },
    );

    // Schedule the appropriate provider-specific sync action
    switch (normalizedProvider) {
      case "spotify":
        await ctx.scheduler.runAfter(
          0,
          internal.librarySyncActions.syncSpotifyLibrary,
          { runId, userId },
        );
        break;
      case "tidal":
        await ctx.scheduler.runAfter(
          0,
          internal.librarySyncActions.syncTidalLibrary,
          { runId, userId },
        );
        break;
      case "youtube":
        await ctx.scheduler.runAfter(
          0,
          internal.librarySyncActions.syncYouTubeLibrary,
          { runId, userId },
        );
        break;
      case "apple_music":
        await ctx.scheduler.runAfter(
          0,
          internal.librarySyncActions.syncAppleMusicLibrary,
          { runId, userId },
        );
        break;
      default:
        throw new ConvexError(`Unsupported provider for library sync: ${provider}`);
    }

    return {
      run_id: runId,
      provider,
      status: "running",
      message: `Library sync triggered for ${provider}`,
    };
  },
});

/**
 * Trigger a proactive evidence investigation for the current user's library.
 * Resolves unlinked artist names, researches each artist via the Rust backend,
 * promotes new classifications to offenses, and rebuilds the index.
 */
export const triggerInvestigation = action({
  args: {},
  handler: async (ctx): Promise<{
    run_id: Id<"platformSyncRuns">;
    status: string;
    message: string;
  }> => {
    const { runId, userId }: { runId: Id<"platformSyncRuns">; userId: Id<"users"> } = await ctx.runMutation(
      api.sync._createRunWithUser,
      { platform: "evidence_finder" },
    );

    await ctx.scheduler.runAfter(
      0,
      internal.evidenceFinder.investigateLibraryArtists,
      { runId, userId },
    );

    return {
      run_id: runId,
      status: "running",
      message: "Evidence investigation started for your library artists.",
    };
  },
});

/**
 * CLI-friendly: trigger investigation for all users with active connections.
 * No auth required — callable via `npx convex run sync:runInvestigationNow`.
 */
export const runInvestigationNow = internalAction({
  args: {},
  handler: async (ctx) => {
    await ctx.runMutation(internal.evidenceFinder.dailyInvestigation, {});
    return { status: "scheduled", message: "Investigation triggered for all users with active connections." };
  },
});

export const _createRun = mutation({
  args: {
    platform: v.string(),
  },
  handler: async (ctx, args) => {
    const now = nowIso();
    const runId = await ctx.db.insert("platformSyncRuns", {
      legacyKey: `runtime:sync:${args.platform}:${Date.now()}`,
      platform: args.platform,
      status: "running",
      startedAt: now,
      errorLog: [],
      checkpointData: null,
      metadata: {},
      createdAt: now,
      updatedAt: now,
    });
    return runId;
  },
});

/**
 * Creates a sync run record AND returns the current user's ID so the
 * calling action can pass it to a scheduled internal action (which has
 * no auth context of its own).
 */
export const _createRunWithUser = mutation({
  args: {
    platform: v.string(),
  },
  handler: async (ctx, args) => {
    const { user } = await requireCurrentUser(ctx);
    const now = nowIso();
    const runId = await ctx.db.insert("platformSyncRuns", {
      legacyKey: `runtime:sync:${args.platform}:${Date.now()}`,
      platform: args.platform,
      status: "running",
      startedAt: now,
      errorLog: [],
      checkpointData: null,
      metadata: { userId: user._id },
      createdAt: now,
      updatedAt: now,
    });
    return { runId, userId: user._id };
  },
});
