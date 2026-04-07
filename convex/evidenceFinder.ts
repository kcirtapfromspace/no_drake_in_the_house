import { v } from "convex/values";
import {
  internalAction,
  internalMutation,
  internalQuery,
} from "./_generated/server";
import { internal } from "./_generated/api";
import type { Id } from "./_generated/dataModel";
import { serviceAuthHeaders } from "./lib/serviceAuth";

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
    const seen = new Set<string>();
    const result: string[] = [];
    for await (const track of ctx.db
      .query("userLibraryTracks")
      .withIndex("by_userId", (q) => q.eq("userId", args.userId))) {
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
    const nameCounts = new Map<string, number>();
    for await (const track of ctx.db
      .query("userLibraryTracks")
      .withIndex("by_userId", (q) => q.eq("userId", args.userId))) {
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
    // Use compound index to fetch ONLY tracks matching this artist name,
    // instead of scanning ALL user tracks (was 11.99 GB/month).
    // Cast needed because artistName is v.optional(v.string()) in the schema.
    const tracks = await ctx.db
      .query("userLibraryTracks")
      .withIndex("by_user_artistName", (q) =>
        (q.eq("userId", args.userId) as any).eq(
          "artistName",
          args.artistName,
        ),
      )
      .collect();

    let linked = 0;
    for (const track of tracks) {
      if (!track.artistId) {
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
    const patch: Record<string, string> = {
      investigationStatus: args.status,
      updatedAt: now,
    };
    // Only stamp lastInvestigatedAt on terminal statuses, NOT "in_progress".
    // Setting it early prevents retries from re-investigating failed artists
    // because _filterArtistsNeedingInvestigation skips recently-stamped ones.
    if (args.status !== "in_progress") {
      patch.lastInvestigatedAt = now;
    }
    await ctx.db.patch(args.artistId, patch);
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
      internal.librarySyncActions._getSyncRun,
      { runId },
    );
    if (!run || run.status === "cancelled") return;

    // Restore checkpoint — guard against empty {} from initial run creation
    const raw = run.checkpointData as Record<string, unknown> | null;
    let checkpoint: InvestigationCheckpoint =
      raw && typeof raw.phase === "string"
        ? (raw as unknown as InvestigationCheckpoint)
        : {
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
      await ctx.runMutation(internal.librarySyncActions._updateSyncRun, {
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
            internal.evidenceFinder._getUnresolvedArtistNames,
            { userId },
          );

        for (const { name } of unresolved) {
          if (shouldPause()) {
            await saveCheckpoint();
            await ctx.scheduler.runAfter(
              0,
              internal.evidenceFinder.investigateLibraryArtists,
              { runId, userId },
            );
            return;
          }

          const artistId: Id<"artists"> = await ctx.runMutation(
            internal.evidenceFinder._resolveOrCreateArtist,
            { name },
          );
          await ctx.runMutation(
            internal.evidenceFinder._linkTracksToArtist,
            { userId, artistName: name, artistId },
          );
        }

        // Now get all resolved artist IDs
        const allArtistIds: string[] = await ctx.runQuery(
          internal.evidenceFinder._getLibraryArtistIds,
          { userId },
        );

        // Filter to those needing investigation
        const needsInvestigation: string[] = await ctx.runQuery(
          internal.evidenceFinder._filterArtistsNeedingInvestigation,
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
                internal.evidenceFinder.investigateLibraryArtists,
                { runId, userId },
              );
              return;
            }

            const artistId = artistIds[i] as Id<"artists">;

            // Resolve artist name from Convex before calling backend (US-006)
            const artistInfo: { name: string; canonicalName: string } | null =
              await ctx.runQuery(
                internal.evidenceFinder._getArtistName,
                { artistId },
              );
            if (!artistInfo) {
              checkpoint.skipped++;
              continue;
            }

            // Mark as in_progress
            await ctx.runMutation(
              internal.evidenceFinder._markArtistInvestigated,
              { artistId, status: "in_progress" },
            );

            try {
              // Call the Rust backend research endpoint with artist name (US-005 + US-006)
              const url = `${backendUrl}/api/v1/news/research/trigger`;
              const response = await fetch(url, {
                method: "POST",
                headers: serviceAuthHeaders(),
                body: JSON.stringify({
                  artist_name: artistInfo.canonicalName,
                }),
                redirect: "follow",
              });

              if (response.ok) {
                const result = await response.json();
                checkpoint.investigated++;
                checkpoint.offensesFound +=
                  result.offenses_detected ?? result.total_offenses_detected ?? 0;

                await ctx.runMutation(
                  internal.evidenceFinder._markArtistInvestigated,
                  { artistId, status: "completed" },
                );
              } else {
                const body = await response.text().catch(() => "");
                console.error(
                  `[EvidenceFinder] Backend ${response.status} for artist ${artistId} (${artistInfo.canonicalName}): ${body.slice(0, 200)}`,
                );
                checkpoint.failed++;
                await ctx.runMutation(
                  internal.evidenceFinder._markArtistInvestigated,
                  { artistId, status: "failed" },
                );
              }
            } catch (err: any) {
              console.error(
                `[EvidenceFinder] Exception researching artist ${artistId} (${artistInfo.canonicalName}): ${err?.message ?? err}`,
              );
              checkpoint.failed++;
              await ctx.runMutation(
                internal.evidenceFinder._markArtistInvestigated,
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
          internal.offensePipeline.promoteClassifications,
          {},
        );

        // Rebuild the index
        await ctx.runMutation(
          internal.offensePipeline.rebuildOffendingArtistIndex,
          {},
        );

        // Recompute this user's offense summary (schedule to avoid action→action call)
        await ctx.scheduler.runAfter(
          0,
          internal.offensePipeline.recomputeUserOffenseSummary,
          { userId, triggerReason: "investigation_complete" },
        );

        checkpoint.phase = "done";
        await saveCheckpoint();
      }

      // ── Done ────────────────────────────────────────────────────────
      await ctx.runMutation(internal.librarySyncActions._updateSyncRun, {
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
          totalArtists: checkpoint.artistIds?.length ?? 0,
        },
      });
    } catch (err: any) {
      await ctx.runMutation(internal.librarySyncActions._updateSyncRun, {
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

/**
 * Helper query: get artist name and canonical name for an artist ID.
 * Used by US-006 to resolve the artist name before calling the backend.
 */
export const _getArtistName = internalQuery({
  args: { artistId: v.id("artists") },
  handler: async (ctx, args) => {
    const artist = await ctx.db.get(args.artistId);
    if (!artist) return null;
    return { name: artist.canonicalName, canonicalName: artist.canonicalName };
  },
});

// ---------------------------------------------------------------------------
// Global artist inventory cycling
// ---------------------------------------------------------------------------

/**
 * Get artists ordered by investigation age (never investigated first, then oldest).
 * Used by the cycling worker to process the full inventory.
 */
export const _getArtistsByInvestigationAge = internalQuery({
  args: { limit: v.optional(v.number()) },
  handler: async (ctx, args) => {
    const cutoff = new Date(
      Date.now() - STALE_DAYS * 24 * 60 * 60 * 1000,
    ).toISOString();
    const limit = args.limit ?? BATCH_SIZE;

    // Use async iteration instead of .collect() to avoid loading entire
    // artists table into memory. Stop early once we have enough candidates.
    const neverInvestigated: Array<{
      artistId: Id<"artists">;
      canonicalName: string;
      lastInvestigatedAt: string | null;
    }> = [];
    const stale: Array<{
      artistId: Id<"artists">;
      canonicalName: string;
      lastInvestigatedAt: string | null;
    }> = [];

    for await (const a of ctx.db.query("artists")) {
      if (!a.lastInvestigatedAt) {
        neverInvestigated.push({
          artistId: a._id,
          canonicalName: a.canonicalName,
          lastInvestigatedAt: null,
        });
      } else if (a.lastInvestigatedAt < cutoff) {
        stale.push({
          artistId: a._id,
          canonicalName: a.canonicalName,
          lastInvestigatedAt: a.lastInvestigatedAt,
        });
      }
      // Stop scanning once we have more than enough candidates
      if (neverInvestigated.length >= limit && stale.length >= limit) break;
    }

    // Priority: never investigated first, then oldest stale
    stale.sort((a, b) =>
      (a.lastInvestigatedAt ?? "").localeCompare(b.lastInvestigatedAt ?? ""),
    );
    return [...neverInvestigated, ...stale].slice(0, limit);
  },
});

/**
 * Cycle through the global artist inventory, processing from oldest
 * lastInvestigatedAt timestamp. When the full list is complete, the next
 * cron invocation starts from the oldest again.
 */
export const cycleArtistInventory = internalAction({
  args: {
    runId: v.id("platformSyncRuns"),
  },
  handler: async (ctx, args) => {
    const startTime = Date.now();
    const { runId } = args;

    const shouldPause = () => Date.now() - startTime > SAFE_RUNTIME_MS;

    let investigated = 0;
    let skipped = 0;
    let failed = 0;
    let offensesFound = 0;

    try {
      // Get the next batch of artists to investigate
      const batch: Array<{
        artistId: Id<"artists">;
        canonicalName: string;
        lastInvestigatedAt: string | null;
      }> = await ctx.runQuery(
        internal.evidenceFinder._getArtistsByInvestigationAge,
        { limit: BATCH_SIZE },
      );

      if (batch.length === 0) {
        // All artists are recently investigated — nothing to do
        await ctx.runMutation(internal.librarySyncActions._updateSyncRun, {
          runId,
          status: "completed",
          completedAt: new Date().toISOString(),
          metadata: { investigated: 0, message: "All artists recently investigated" },
        });
        return;
      }

      const backendUrl = process.env.NDITH_BACKEND_URL;

      for (const { artistId, canonicalName } of batch) {
        if (shouldPause()) {
          // Schedule continuation
          await ctx.runMutation(internal.librarySyncActions._updateSyncRun, {
            runId,
            metadata: { investigated, skipped, failed, offensesFound },
          });
          await ctx.scheduler.runAfter(
            0,
            internal.evidenceFinder.cycleArtistInventory,
            { runId },
          );
          return;
        }

        await ctx.runMutation(
          internal.evidenceFinder._markArtistInvestigated,
          { artistId, status: "in_progress" },
        );

        if (backendUrl) {
          try {
            // Call the Rust backend research endpoint with artist name (US-005 + US-006)
            const url = `${backendUrl}/api/v1/news/research/trigger`;
            const response = await fetch(url, {
              method: "POST",
              headers: serviceAuthHeaders(),
              body: JSON.stringify({
                artist_name: canonicalName,
              }),
              redirect: "follow",
            });

            if (response.ok) {
              const result = await response.json();
              investigated++;
              offensesFound +=
                result.offenses_detected ?? result.total_offenses_detected ?? 0;
              await ctx.runMutation(
                internal.evidenceFinder._markArtistInvestigated,
                { artistId, status: "completed" },
              );
            } else {
              const body = await response.text().catch(() => "");
              console.error(
                `[EvidenceFinder:cycle] Backend ${response.status} for artist ${artistId} (${canonicalName}): ${body.slice(0, 200)}`,
              );
              failed++;
              await ctx.runMutation(
                internal.evidenceFinder._markArtistInvestigated,
                { artistId, status: "failed" },
              );
            }
          } catch (err: any) {
            console.error(
              `[EvidenceFinder:cycle] Exception researching artist ${artistId} (${canonicalName}): ${err?.message ?? err}`,
            );
            failed++;
            await ctx.runMutation(
              internal.evidenceFinder._markArtistInvestigated,
              { artistId, status: "failed" },
            );
          }

          // Small delay between research calls
          await new Promise((r) => setTimeout(r, INTER_ARTIST_DELAY_MS));
        } else {
          // No backend URL — mark as investigated so we cycle past this artist
          await ctx.runMutation(
            internal.evidenceFinder._markArtistInvestigated,
            { artistId, status: "no_backend" },
          );
          skipped++;
        }
      }

      // After batch: promote any new classifications and rebuild index
      await ctx.runMutation(
        internal.offensePipeline.promoteClassifications,
        {},
      );
      await ctx.runMutation(
        internal.offensePipeline.rebuildOffendingArtistIndex,
        {},
      );

      // Check if more artists need processing
      const remaining: Array<{ artistId: Id<"artists"> }> = await ctx.runQuery(
        internal.evidenceFinder._getArtistsByInvestigationAge,
        { limit: 1 },
      );

      if (remaining.length > 0) {
        // More artists to process — schedule continuation
        await ctx.scheduler.runAfter(
          0,
          internal.evidenceFinder.cycleArtistInventory,
          { runId },
        );
      } else {
        // Full cycle complete
        await ctx.runMutation(internal.librarySyncActions._updateSyncRun, {
          runId,
          status: "completed",
          completedAt: new Date().toISOString(),
          metadata: { investigated, skipped, failed, offensesFound },
        });
      }
    } catch (err: any) {
      await ctx.runMutation(internal.librarySyncActions._updateSyncRun, {
        runId,
        status: "failed",
        completedAt: new Date().toISOString(),
        errorLog: [{ message: err?.message ?? "Unknown error", ts: new Date().toISOString() }],
        metadata: { investigated, skipped, failed, offensesFound },
      });
    }
  },
});

// ---------------------------------------------------------------------------
// Daily investigation cron handler
// ---------------------------------------------------------------------------

/**
 * Kick off the global artist inventory cycling worker.
 * Creates a tracking run and schedules the first batch.
 */
export const dailyInvestigation = internalMutation({
  args: {},
  handler: async (ctx) => {
    const now = new Date().toISOString();

    // First, resolve any unresolved artist names from all users' library tracks
    const connections = await ctx.db
      .query("providerConnections")
      .collect();

    const userIds = [
      ...new Set(
        connections
          .filter((c) => c.status === "active")
          .map((c) => c.userId as string),
      ),
    ];

    // Schedule per-user resolution (lightweight — just creates artist records)
    for (const userId of userIds) {
      const resolveRunId = await ctx.db.insert("platformSyncRuns", {
        legacyKey: `runtime:sync:resolve:${Date.now()}:${userId}`,
        platform: "artist_resolution",
        status: "running",
        startedAt: now,
        errorLog: [],
        checkpointData: {},
        metadata: { userId },
        createdAt: now,
        updatedAt: now,
      });

      await ctx.scheduler.runAfter(
        0,
        internal.evidenceFinder.investigateLibraryArtists,
        { runId: resolveRunId, userId: userId as Id<"users"> },
      );
    }

    // Then start the global cycling worker to investigate all artists
    const cycleRunId = await ctx.db.insert("platformSyncRuns", {
      legacyKey: `runtime:sync:evidence_cycle:${Date.now()}`,
      platform: "evidence_finder",
      status: "running",
      startedAt: now,
      errorLog: [],
      checkpointData: {},
      metadata: { type: "global_cycle" },
      createdAt: now,
      updatedAt: now,
    });

    // Stagger the cycle start to let resolution complete first (60s)
    await ctx.scheduler.runAfter(
      60_000,
      internal.evidenceFinder.cycleArtistInventory,
      { runId: cycleRunId },
    );

    return { usersResolved: userIds.length, cycleStarted: true };
  },
});
