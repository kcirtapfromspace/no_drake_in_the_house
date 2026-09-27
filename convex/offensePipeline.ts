import { v } from "convex/values";
import { internalAction, internalMutation, internalQuery, type MutationCtx } from "./_generated/server";
import { internal } from "./_generated/api";
import type { Id } from "./_generated/dataModel";
import { normalizedSeverity } from "./lib/evaluationPolicy";
import { enqueueEvidence } from "./evaluations";

const severityWeight: Record<string, number> = {
  minor: 1,
  moderate: 3,
  severe: 7,
  egregious: 12,
};

const severityRank: Record<string, number> = {
  minor: 0,
  moderate: 1,
  severe: 2,
  egregious: 3,
};

function highestSeverity(a: string, b: string): string {
  return (severityRank[a] ?? 0) >= (severityRank[b] ?? 0) ? a : b;
}

function computeGrade(offenderRatio: number): string {
  if (offenderRatio > 0.5) return "F";
  if (offenderRatio > 0.3) return "D";
  if (offenderRatio > 0.2) return "C";
  if (offenderRatio > 0.1) return "B";
  if (offenderRatio > 0.05) return "A";
  return "A+";
}

/** Update one artist atomically with a review. Only explicitly approved records qualify. */
export async function refreshArtistIndex(ctx: MutationCtx, artistId: Id<"artists">) {
  let offenseCount = 0;
  let highest = "minor";
  let severityTotal = 0;
  const categories = new Set<string>();
  for await (const offense of ctx.db.query("artistOffenses").withIndex("by_artistId_and_status", q => q.eq("artistId", artistId).eq("status", "verified"))) {
    const severity = normalizedSeverity(offense.severity);
    offenseCount++;
    highest = highestSeverity(highest, severity);
    severityTotal += severityWeight[severity];
    categories.add(offense.category);
  }
  const existing = await ctx.db.query("offendingArtistIndex").withIndex("by_artistId", q => q.eq("artistId", artistId)).unique();
  if (!offenseCount) {
    if (existing) await ctx.db.delete(existing._id);
    return;
  }
  const now = new Date().toISOString();
  const values = { offenseCount, highestSeverity: highest, severityTotal, categories: [...categories], updatedAt: now };
  if (existing) await ctx.db.patch(existing._id, values);
  else await ctx.db.insert("offendingArtistIndex", { ...values, artistId, legacyKey: `offense_idx:${artistId}`, createdAt: now });
}

export const rebuildArtistIndex = internalMutation({
  args: { artistId: v.id("artists") },
  handler: async (ctx, args) => { await refreshArtistIndex(ctx, args.artistId); },
});

/** Paginated scheduling avoids silently dropping artists after a fixed scan limit. */
export const rebuildOffendingArtistIndex = internalMutation({
  args: { cursor: v.optional(v.string()) },
  handler: async (ctx, args) => {
    const page = await ctx.db.query("artists").paginate({ cursor: args.cursor ?? null, numItems: 25 });
    for (const artist of page.page) await ctx.scheduler.runAfter(0, internal.offensePipeline.rebuildArtistIndex, { artistId: artist._id });
    if (!page.isDone) await ctx.scheduler.runAfter(1000, internal.offensePipeline.rebuildOffendingArtistIndex, { cursor: page.continueCursor });
    return { scheduled: page.page.length, isDone: page.isDone };
  },
});

/**
 * Recompute the offense summary for a single user.
 *
 * Split into query → action → mutation to eliminate write conflicts:
 * - _readTrackArtistCounts (query): reads userLibraryTracks without holding
 *   a transaction write lock, so concurrent imports/cleanups don't conflict.
 * - recomputeUserOffenseSummary (action): orchestrates read → compute → write.
 * - _writeOffenseSummary (mutation): only touches userOffenseSummaries +
 *   offendingArtistIndex + artists — never reads userLibraryTracks.
 */
/** Minimum interval between recomputes for the same user (30 minutes). */
const RECOMPUTE_COOLDOWN_MS = 30 * 60 * 1000;

/** Phase 1: Read track data per provider as a QUERY (no write-conflict participation).
 *  Scans per-provider via by_user_provider to stay within read limits. */
export const _readTrackArtistCounts = internalQuery({
  args: { userId: v.id("users"), provider: v.string() },
  handler: async (ctx, args) => {
    const tracksByArtist = new Map<string, number>();
    let totalTrackCount = 0;
    for await (const track of ctx.db
      .query("userLibraryTracks")
      .withIndex("by_user_provider", (q) =>
        q.eq("userId", args.userId).eq("provider", args.provider),
      )) {
      totalTrackCount++;
      if (track.artistId) {
        const aid = track.artistId as string;
        tracksByArtist.set(aid, (tracksByArtist.get(aid) ?? 0) + 1);
      }
    }
    return {
      totalTrackCount,
      artistCounts: Object.fromEntries(tracksByArtist),
    };
  },
});

/** Phase 3: Write-only mutation — never reads userLibraryTracks. */
export const _writeOffenseSummary = internalMutation({
  args: {
    userId: v.id("users"),
    totalTrackCount: v.number(),
    artistCounts: v.any(),
    triggerReason: v.optional(v.string()),
    force: v.optional(v.boolean()),
  },
  handler: async (ctx, args) => {
    // ── Throttle: skip if recently computed ────────────────────────────
    if (!args.force) {
      const existing = await ctx.db
        .query("userOffenseSummaries")
        .withIndex("by_userId", (q) => q.eq("userId", args.userId))
        .unique();

      if (existing?.computedAt) {
        const lastComputed = new Date(existing.computedAt).getTime();
        if (Date.now() - lastComputed < RECOMPUTE_COOLDOWN_MS) {
          return {
            grade: existing.grade,
            flaggedArtistCount: existing.flaggedArtistCount,
            totalArtists: existing.totalArtists,
            skipped: true,
          };
        }
      }
    }

    const artistCounts = args.artistCounts as Record<string, number>;
    const artistIds = Object.keys(artistCounts);

    // Batch-lookup offending artist index
    const indexLookups = await Promise.all(
      artistIds.map(async (artistId) => {
        const row = await ctx.db
          .query("offendingArtistIndex")
          .withIndex("by_artistId", (q) =>
            q.eq("artistId", artistId as Id<"artists">),
          )
          .unique();
        return { artistId, row };
      }),
    );

    // Build offenders list
    const offenders: Array<{
      artistId: Id<"artists">;
      artistName: string;
      trackCount: number;
      highestSeverity: string;
      severityTotal: number;
      categories: string[];
    }> = [];

    let flaggedTrackCount = 0;

    for (const { artistId, row } of indexLookups) {
      if (!row) continue;

      const trackCount = artistCounts[artistId] ?? 0;
      flaggedTrackCount += trackCount;

      const artist = await ctx.db.get(artistId as Id<"artists">);
      offenders.push({
        artistId: artistId as Id<"artists">,
        artistName: artist?.canonicalName ?? "Unknown",
        trackCount,
        highestSeverity: row.highestSeverity,
        severityTotal: row.severityTotal,
        categories: row.categories,
      });
    }

    // Sort by severity total descending
    offenders.sort((a, b) => b.severityTotal - a.severityTotal);

    const totalArtists = artistIds.length;
    const flaggedArtistCount = offenders.length;
    const offenderRatio =
      totalArtists > 0 ? flaggedArtistCount / totalArtists : 0;
    const grade = computeGrade(offenderRatio);
    const now = new Date().toISOString();

    // Upsert: find existing summary for this user
    const existing = await ctx.db
      .query("userOffenseSummaries")
      .withIndex("by_userId", (q) => q.eq("userId", args.userId))
      .unique();

    if (existing) {
      await ctx.db.replace(existing._id, {
        legacyKey: existing.legacyKey,
        createdAt: existing.createdAt,
        updatedAt: now,
        userId: args.userId,
        totalTracks: args.totalTrackCount,
        totalArtists,
        flaggedArtistCount,
        flaggedTrackCount,
        offenderRatio,
        grade,
        offenders,
        computedAt: now,
        triggerReason: args.triggerReason ?? "manual",
      });
    } else {
      await ctx.db.insert("userOffenseSummaries", {
        legacyKey: `offense_summary:${args.userId}`,
        createdAt: now,
        updatedAt: now,
        userId: args.userId,
        totalTracks: args.totalTrackCount,
        totalArtists,
        flaggedArtistCount,
        flaggedTrackCount,
        offenderRatio,
        grade,
        offenders,
        computedAt: now,
        triggerReason: args.triggerReason ?? "manual",
      });
    }

    return { grade, flaggedArtistCount, totalArtists };
  },
});

/** Phase 2: Action orchestrator — query (no conflict) then mutation (no track reads). */
export const recomputeUserOffenseSummary = internalAction({
  args: {
    userId: v.id("users"),
    triggerReason: v.optional(v.string()),
    force: v.optional(v.boolean()),
  },
  handler: async (ctx, args) => {
    // Get user's active providers to split scans (stays within read limits)
    const providers: string[] = await ctx.runQuery(
      internal.evidenceFinder._getUserActiveProviders,
      { userId: args.userId },
    );

    // Phase 1: Read tracks per provider then merge. Does not participate in OCC,
    // so concurrent mutations on userLibraryTracks won't conflict.
    let totalTrackCount = 0;
    const mergedArtistCounts: Record<string, number> = {};
    for (const provider of providers) {
      const data: { totalTrackCount: number; artistCounts: Record<string, number> } =
        await ctx.runQuery(
          internal.offensePipeline._readTrackArtistCounts,
          { userId: args.userId, provider },
        );
      totalTrackCount += data.totalTrackCount;
      for (const [artistId, count] of Object.entries(data.artistCounts)) {
        mergedArtistCounts[artistId] = (mergedArtistCounts[artistId] ?? 0) + count;
      }
    }
    const trackData = { totalTrackCount, artistCounts: mergedArtistCounts };

    // Phase 2: Write summary — only touches userOffenseSummaries,
    // offendingArtistIndex, and artists tables. Zero reads on userLibraryTracks.
    const result: any = await ctx.runMutation(
      internal.offensePipeline._writeOffenseSummary,
      {
        userId: args.userId,
        totalTrackCount: trackData.totalTrackCount,
        artistCounts: trackData.artistCounts,
        triggerReason: args.triggerReason,
        force: args.force,
      },
    );

    return result;
  },
});

/**
 * Fan-out: given an artistId, find all users who have that artist in their
 * library and schedule recomputeUserOffenseSummary for each.
 */
export const recomputeAffectedUsers = internalAction({
  args: {
    artistId: v.id("artists"),
  },
  handler: async (ctx, args) => {
    // Find all library tracks referencing this artist
    const tracks: Array<{ userId: string }> = await ctx.runQuery(
      internal.offensePipeline._getTracksByArtist,
      { artistId: args.artistId },
    );

    // Deduplicate user IDs
    const userIds = [...new Set(tracks.map((t) => t.userId))];

    // Schedule recompute in batches of 50
    const BATCH = 50;
    for (let i = 0; i < userIds.length; i += BATCH) {
      const batch = userIds.slice(i, i + BATCH);
      for (const userId of batch) {
        await ctx.scheduler.runAfter(
          0,
          internal.offensePipeline.recomputeUserOffenseSummary,
          { userId: userId as Id<"users">, triggerReason: "offense_verified" },
        );
      }
    }

    return { usersScheduled: userIds.length };
  },
});

/** Helper query: get user IDs from tracks by artist */
export const _getTracksByArtist = internalQuery({
  args: { artistId: v.id("artists") },
  handler: async (ctx, args) => {
    const tracks = await ctx.db
      .query("userLibraryTracks")
      .withIndex("by_artistId", (q) => q.eq("artistId", args.artistId))
      .take(500);
    return tracks.map((t) => ({ userId: t.userId as string }));
  },
});

/**
 * Promote high-confidence newsOffenseClassifications into artistOffenses.
 * This is the missing link: the news pipeline writes classifications but
 * nothing was converting them into the artistOffenses table that drives
 * all block list counts and the offendingArtistIndex.
 */
/** Legacy classifications are candidates for Jev, never automatically promoted offenses. */
export const promoteClassifications = internalMutation({
  args: { cursor: v.optional(v.string()) },
  handler: async (ctx, args) => {
    const page = await ctx.db.query("newsOffenseClassifications").paginate({ cursor: args.cursor ?? null, numItems: 25 });
    let queued = 0;
    for (const candidate of page.page) {
      if (!candidate.artistId) continue;
      const article = await ctx.db.get(candidate.articleId);
      if (!article) continue;
      await enqueueEvidence(ctx, { artistId: candidate.artistId, url: article.url, sourceText: article.content, sourceTitle: article.title });
      queued++;
    }
    if (!page.isDone) await ctx.scheduler.runAfter(1000, internal.offensePipeline.promoteClassifications, { cursor: page.continueCursor });
    return { queued, promoted: 0, isDone: page.isDone };
  },
});

/**
 * Daily sweep: find summaries older than 24h and schedule recompute.
 */
/** Stagger delay between scheduled recomputes to avoid thundering herd (ms). */
const SWEEP_STAGGER_MS = 60_000;

export const dailySweep = internalMutation({
  args: {},
  handler: async (ctx) => {
    const cutoff = new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString();
    const allSummaries = await ctx.db.query("userOffenseSummaries").take(500);

    let scheduled = 0;
    for (const summary of allSummaries) {
      if (summary.computedAt < cutoff) {
        // Stagger recomputes over time to avoid bandwidth spikes
        await ctx.scheduler.runAfter(
          scheduled * SWEEP_STAGGER_MS,
          internal.offensePipeline.recomputeUserOffenseSummary,
          { userId: summary.userId, triggerReason: "scheduled" },
        );
        scheduled++;
      }
    }

    return { scheduled };
  },
});
