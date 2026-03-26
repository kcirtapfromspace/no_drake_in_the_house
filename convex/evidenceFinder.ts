import { v } from "convex/values";
import {
  internalAction,
  internalMutation,
  internalQuery,
} from "./_generated/server";
import { internal } from "./_generated/api";
import type { Id } from "./_generated/dataModel";

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/** Max artists to research per continuation before checkpointing. */
const BATCH_SIZE = 15;
/** Skip artists investigated within this many days. */
const STALE_DAYS = 30;
/** Leave a 5-minute buffer before the 30-minute Convex action limit. */
const SAFE_RUNTIME_MS = 25 * 60 * 1000;
/** Delay between research calls to avoid hammering the backend (ms). */
const INTER_ARTIST_DELAY_MS = 2000;

// ---------------------------------------------------------------------------
// Checkpoint types
// ---------------------------------------------------------------------------

interface InvestigationCheckpoint {
  phase: "resolve" | "investigate" | "promote" | "done";
  artistIds: string[];
  currentIndex: number;
  investigated: number;
  skipped: number;
  failed: number;
  offensesFound: number;
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/**
 * Get unique artist IDs from a user's library tracks.
 * Only returns tracks that have an artistId resolved.
 */
export const _getLibraryArtistIds = internalQuery({
  args: { userId: v.id("users") },
  handler: async (ctx, args) => {
    const tracks = await ctx.db
      .query("userLibraryTracks")
      .withIndex("by_userId", (q) => q.eq("userId", args.userId))
      .collect();

    const seen = new Set<string>();
    const result: string[] = [];
    for (const track of tracks) {
      if (track.artistId && !seen.has(track.artistId as string)) {
        seen.add(track.artistId as string);
        result.push(track.artistId as string);
      }
    }
    return result;
  },
});

/**
 * Get unique artist names from library tracks that don't have artistId resolved.
 * Returns { name, count } pairs sorted by track count descending.
 */
export const _getUnresolvedArtistNames = internalQuery({
  args: { userId: v.id("users") },
  handler: async (ctx, args) => {
    const tracks = await ctx.db
      .query("userLibraryTracks")
      .withIndex("by_userId", (q) => q.eq("userId", args.userId))
      .collect();

    const nameCounts = new Map<string, number>();
    for (const track of tracks) {
      if (!track.artistId && track.artistName) {
        const name = track.artistName;
        nameCounts.set(name, (nameCounts.get(name) ?? 0) + 1);
      }
    }

    return [...nameCounts.entries()]
      .map(([name, count]) => ({ name, count }))
      .sort((a, b) => b.count - a.count);
  },
});

/**
 * Resolve an artist name to an artist record, creating one if needed.
 * Returns the artist ID.
 */
export const _resolveOrCreateArtist = internalMutation({
  args: { name: v.string() },
  handler: async (ctx, args) => {
    // Try exact match via search index
    const matches = await ctx.db
      .query("artists")
      .withSearchIndex("search_canonicalName", (q) =>
        q.search("canonicalName", args.name),
      )
      .take(10);

    const exact = matches.find(
      (a) => a.canonicalName.toLowerCase() === args.name.toLowerCase(),
    );
    if (exact) return exact._id;
    if (matches.length > 0) return matches[0]._id;

    // Create a new artist record
    const now = new Date().toISOString();
    return await ctx.db.insert("artists", {
      legacyKey: `auto:artist:${args.name.toLowerCase().replace(/\s+/g, "_")}`,
      createdAt: now,
      updatedAt: now,
      canonicalName: args.name,
      status: "unverified",
    });
  },
});

/**
 * Link unresolved library tracks to a resolved artist ID.
 */
export const _linkTracksToArtist = internalMutation({
  args: {
    userId: v.id("users"),
    artistName: v.string(),
    artistId: v.id("artists"),
  },
  handler: async (ctx, args) => {
    const tracks = await ctx.db
      .query("userLibraryTracks")
      .withIndex("by_userId", (q) => q.eq("userId", args.userId))
      .collect();

    let linked = 0;
    for (const track of tracks) {
      if (
        !track.artistId &&
        track.artistName?.toLowerCase() === args.artistName.toLowerCase()
      ) {
        await ctx.db.patch(track._id, { artistId: args.artistId });
        linked++;
      }
    }
    return { linked };
  },
});

/**
 * Filter a list of artist IDs to those needing investigation.
 * Returns IDs sorted by priority: never investigated first, then stale.
 */
export const _filterArtistsNeedingInvestigation = internalQuery({
  args: { artistIds: v.array(v.string()) },
  handler: async (ctx, args) => {
    const cutoff = new Date(
      Date.now() - STALE_DAYS * 24 * 60 * 60 * 1000,
    ).toISOString();

    const neverInvestigated: string[] = [];
    const stale: string[] = [];

    for (const id of args.artistIds) {
      const artist = await ctx.db.get(id as Id<"artists">);
      if (!artist) continue;

      if (!artist.lastInvestigatedAt) {
        neverInvestigated.push(id);
      } else if (artist.lastInvestigatedAt < cutoff) {
        stale.push(id);
      }
      // else: recently investigated, skip
    }

    // Priority: never investigated first, then stale
    return [...neverInvestigated, ...stale];
  },
});

/**
 * Mark an artist as investigated.
 */
export const _markArtistInvestigated = internalMutation({
  args: {
    artistId: v.id("artists"),
    status: v.string(),
  },
  handler: async (ctx, args) => {
    const now = new Date().toISOString();
    await ctx.db.patch(args.artistId, {
      lastInvestigatedAt: now,
      investigationStatus: args.status,
      updatedAt: now,
    });
  },
});

// ---------------------------------------------------------------------------
// Main investigation action
// ---------------------------------------------------------------------------

/**
 * Investigate library artists by calling the Rust backend research endpoint.
 * Uses checkpoint/continuation pattern for large libraries.
 */
export const investigateLibraryArtists = internalAction({
  args: {
    runId: v.id("platformSyncRuns"),
    userId: v.id("users"),
  },
  handler: async (ctx, args) => {
    const startTime = Date.now();
    const { runId, userId } = args;

    // Check if run was cancelled
    const run: any = await ctx.runQuery(
      "librarySyncActions:_getSyncRun" as any,
      { runId },
    );
    if (!run || run.status === "cancelled") return;

    // Restore checkpoint
    let checkpoint: InvestigationCheckpoint = (run.checkpointData as InvestigationCheckpoint | null) ?? {
      phase: "resolve",
      artistIds: [],
      currentIndex: 0,
      investigated: 0,
      skipped: 0,
      failed: 0,
      offensesFound: 0,
    };

    const shouldPause = () => Date.now() - startTime > SAFE_RUNTIME_MS;

    const saveCheckpoint = async () => {
      await ctx.runMutation("librarySyncActions:_updateSyncRun" as any, {
        runId,
        checkpointData: checkpoint,
        metadata: {
          userId,
          investigated: checkpoint.investigated,
          skipped: checkpoint.skipped,
          failed: checkpoint.failed,
        },
      });
    };

    try {
      // ── Phase: resolve ──────────────────────────────────────────────
      if (checkpoint.phase === "resolve") {
        // First resolve unresolved artist names
        const unresolved: Array<{ name: string; count: number }> =
          await ctx.runQuery(
            "evidenceFinder:_getUnresolvedArtistNames" as any,
            { userId },
          );

        for (const { name } of unresolved) {
          if (shouldPause()) {
            await saveCheckpoint();
            await ctx.scheduler.runAfter(
              0,
              "evidenceFinder:investigateLibraryArtists" as any,
              { runId, userId },
            );
            return;
          }

          const artistId: Id<"artists"> = await ctx.runMutation(
            "evidenceFinder:_resolveOrCreateArtist" as any,
            { name },
          );
          await ctx.runMutation(
            "evidenceFinder:_linkTracksToArtist" as any,
            { userId, artistName: name, artistId },
          );
        }

        // Now get all resolved artist IDs
        const allArtistIds: string[] = await ctx.runQuery(
          "evidenceFinder:_getLibraryArtistIds" as any,
          { userId },
        );

        // Filter to those needing investigation
        const needsInvestigation: string[] = await ctx.runQuery(
          "evidenceFinder:_filterArtistsNeedingInvestigation" as any,
          { artistIds: allArtistIds },
        );

        checkpoint.artistIds = needsInvestigation;
        checkpoint.currentIndex = 0;
        checkpoint.phase = "investigate";
        await saveCheckpoint();
      }

      // ── Phase: investigate ──────────────────────────────────────────
      if (checkpoint.phase === "investigate") {
        const backendUrl = process.env.NDITH_BACKEND_URL;

        if (!backendUrl) {
          // No backend URL configured — skip research, go straight to promote
          checkpoint.phase = "promote";
          await saveCheckpoint();
        } else {
          const { artistIds, currentIndex } = checkpoint;

          for (let i = currentIndex; i < artistIds.length; i++) {
            if (shouldPause()) {
              checkpoint.currentIndex = i;
              await saveCheckpoint();
              await ctx.scheduler.runAfter(
                0,
                "evidenceFinder:investigateLibraryArtists" as any,
                { runId, userId },
              );
              return;
            }

            const artistId = artistIds[i] as Id<"artists">;

            // Get artist name for research
            const artist: any = await ctx.runQuery(
              "evidenceFinder:_getArtistById" as any,
              { artistId },
            );
            if (!artist) {
              checkpoint.skipped++;
              continue;
            }

            // Mark as in_progress
            await ctx.runMutation(
              "evidenceFinder:_markArtistInvestigated" as any,
              { artistId, status: "in_progress" },
            );

            try {
              // Call the Rust backend research endpoint
              const response = await fetch(
                `${backendUrl}/news/research/artists/${artistId}/trigger`,
                {
                  method: "POST",
                  headers: { "Content-Type": "application/json" },
                  body: JSON.stringify({
                    artist_name: artist.canonicalName,
                    artist_id: artistId,
                  }),
                },
              );

              if (response.ok) {
                const result = await response.json();
                checkpoint.investigated++;
                checkpoint.offensesFound +=
                  result.offenses_detected ?? result.total_offenses_detected ?? 0;

                await ctx.runMutation(
                  "evidenceFinder:_markArtistInvestigated" as any,
                  { artistId, status: "completed" },
                );
              } else {
                checkpoint.failed++;
                await ctx.runMutation(
                  "evidenceFinder:_markArtistInvestigated" as any,
                  { artistId, status: "failed" },
                );
              }
            } catch {
              checkpoint.failed++;
              await ctx.runMutation(
                "evidenceFinder:_markArtistInvestigated" as any,
                { artistId, status: "failed" },
              );
            }

            // Small delay between requests
            if (i < artistIds.length - 1) {
              await new Promise((r) => setTimeout(r, INTER_ARTIST_DELAY_MS));
            }
          }

          checkpoint.phase = "promote";
          await saveCheckpoint();
        }
      }

      // ── Phase: promote ──────────────────────────────────────────────
      if (checkpoint.phase === "promote") {
        // Convert any new high-confidence classifications into offenses
        await ctx.runMutation(
          "offensePipeline:promoteClassifications" as any,
          {},
        );

        // Rebuild the index
        await ctx.runMutation(
          internal.offensePipeline.rebuildOffendingArtistIndex,
          {},
        );

        // Recompute this user's offense summary
        await ctx.runMutation(
          internal.offensePipeline.recomputeUserOffenseSummary,
          { userId, triggerReason: "investigation_complete" },
        );

        checkpoint.phase = "done";
        await saveCheckpoint();
      }

      // ── Done ────────────────────────────────────────────────────────
      await ctx.runMutation("librarySyncActions:_updateSyncRun" as any, {
        runId,
        status: "completed",
        completedAt: new Date().toISOString(),
        checkpointData: checkpoint,
        metadata: {
          userId,
          investigated: checkpoint.investigated,
          skipped: checkpoint.skipped,
          failed: checkpoint.failed,
          offensesFound: checkpoint.offensesFound,
          totalArtists: checkpoint.artistIds.length,
        },
      });
    } catch (err: any) {
      await ctx.runMutation("librarySyncActions:_updateSyncRun" as any, {
        runId,
        status: "failed",
        completedAt: new Date().toISOString(),
        checkpointData: checkpoint,
        errorLog: [
          {
            message: err?.message ?? "Unknown error during investigation",
            ts: new Date().toISOString(),
          },
        ],
      });
    }
  },
});

/**
 * Helper query: get artist by ID.
 */
export const _getArtistById = internalQuery({
  args: { artistId: v.id("artists") },
  handler: async (ctx, args) => {
    return await ctx.db.get(args.artistId);
  },
});

// ---------------------------------------------------------------------------
// Daily investigation cron handler
// ---------------------------------------------------------------------------

/**
 * Find all users with connected providers and schedule investigation for each.
 */
export const dailyInvestigation = internalMutation({
  args: {},
  handler: async (ctx) => {
    const now = new Date().toISOString();

    // Find all active provider connections
    const connections = await ctx.db
      .query("providerConnections")
      .collect();

    // Get unique user IDs with active connections
    const userIds = [
      ...new Set(
        connections
          .filter((c) => c.status === "active")
          .map((c) => c.userId as string),
      ),
    ];

    let scheduled = 0;
    for (const userId of userIds) {
      // Create a sync run for tracking
      const runId = await ctx.db.insert("platformSyncRuns", {
        legacyKey: `runtime:sync:evidence_finder:${Date.now()}:${userId}`,
        platform: "evidence_finder",
        status: "running",
        startedAt: now,
        errorLog: [],
        checkpointData: {},
        metadata: { userId },
        createdAt: now,
        updatedAt: now,
      });

      // Stagger by 30 seconds per user to avoid thundering herd
      await ctx.scheduler.runAfter(
        scheduled * 30_000,
        "evidenceFinder:investigateLibraryArtists" as any,
        { runId, userId: userId as Id<"users"> },
      );
      scheduled++;
    }

    return { scheduled };
  },
});
