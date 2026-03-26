import { v } from "convex/values";
import { internalAction, internalMutation, internalQuery } from "./_generated/server";
import { internal } from "./_generated/api";
import type { Id } from "./_generated/dataModel";

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

/**
 * Rebuild the offendingArtistIndex from all verified/active artistOffenses.
 * Idempotent — deletes stale rows and upserts current ones.
 */
export const rebuildOffendingArtistIndex = internalMutation({
  args: {},
  handler: async (ctx) => {
    // Load all offenses
    const allOffenses = await ctx.db.query("artistOffenses").collect();

    // Group by artist
    const byArtist = new Map<
      string,
      { offenseCount: number; highest: string; severityTotal: number; categories: Set<string> }
    >();

    for (const offense of allOffenses) {
      const aid = offense.artistId as string;
      const existing = byArtist.get(aid);
      const weight = severityWeight[offense.severity] ?? 2;

      if (existing) {
        existing.offenseCount++;
        existing.highest = highestSeverity(existing.highest, offense.severity);
        existing.severityTotal += weight;
        existing.categories.add(offense.category);
      } else {
        byArtist.set(aid, {
          offenseCount: 1,
          highest: offense.severity,
          severityTotal: weight,
          categories: new Set([offense.category]),
        });
      }
    }

    // Delete all existing index rows
    const existingRows = await ctx.db.query("offendingArtistIndex").collect();
    for (const row of existingRows) {
      await ctx.db.delete(row._id);
    }

    // Insert fresh rows
    const now = new Date().toISOString();
    let inserted = 0;
    for (const [artistId, data] of byArtist) {
      await ctx.db.insert("offendingArtistIndex", {
        legacyKey: `offense_idx:${artistId}`,
        createdAt: now,
        updatedAt: now,
        artistId: artistId as Id<"artists">,
        offenseCount: data.offenseCount,
        highestSeverity: data.highest,
        severityTotal: data.severityTotal,
        categories: [...data.categories],
      });
      inserted++;
    }

    return { indexed: inserted };
  },
});

/**
 * Recompute the offense summary for a single user.
 * Reads their library tracks, batch-lookups offendingArtistIndex, and upserts the summary.
 */
export const recomputeUserOffenseSummary = internalMutation({
  args: {
    userId: v.id("users"),
    triggerReason: v.optional(v.string()),
  },
  handler: async (ctx, args) => {
    const tracks = await ctx.db
      .query("userLibraryTracks")
      .withIndex("by_userId", (q) => q.eq("userId", args.userId))
      .collect();

    // Collect unique artist IDs and count tracks per artist
    const tracksByArtist = new Map<string, number>();
    for (const track of tracks) {
      if (track.artistId) {
        const aid = track.artistId as string;
        tracksByArtist.set(aid, (tracksByArtist.get(aid) ?? 0) + 1);
      }
    }

    const artistIds = [...tracksByArtist.keys()];

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

      const trackCount = tracksByArtist.get(artistId) ?? 0;
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
        totalTracks: tracks.length,
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
        totalTracks: tracks.length,
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
      .collect();
    return tracks.map((t) => ({ userId: t.userId as string }));
  },
});

/**
 * Promote high-confidence newsOffenseClassifications into artistOffenses.
 * This is the missing link: the news pipeline writes classifications but
 * nothing was converting them into the artistOffenses table that drives
 * all block list counts and the offendingArtistIndex.
 */
export const promoteClassifications = internalMutation({
  args: {},
  handler: async (ctx) => {
    const CONFIDENCE_THRESHOLD = 0.7;
    const now = new Date().toISOString();

    // Load all classifications that are high-confidence or human-verified
    const allClassifications = await ctx.db
      .query("newsOffenseClassifications")
      .collect();

    const eligible = allClassifications.filter(
      (c) =>
        c.artistId &&
        ((c.confidence ?? 0) >= CONFIDENCE_THRESHOLD || c.humanVerified === true),
    );

    // Load existing artistOffenses for dedup
    const existingOffenses = await ctx.db.query("artistOffenses").collect();
    const offenseKeys = new Set(
      existingOffenses.map((o) => `${o.artistId}:${o.category}`),
    );

    let promoted = 0;
    let skipped = 0;

    for (const cls of eligible) {
      const dedupKey = `${cls.artistId}:${cls.category}`;
      if (offenseKeys.has(dedupKey)) {
        skipped++;
        continue;
      }

      // Fetch article for title/description
      const article = await ctx.db.get(cls.articleId);
      const title = article?.title ?? `${cls.category} offense`;
      const description =
        article?.summary ??
        article?.content?.slice(0, 500) ??
        `Auto-detected from news classification.`;

      // Create the artistOffenses record
      const offenseId = await ctx.db.insert("artistOffenses", {
        legacyKey: `auto:offense:${cls.artistId}:${cls.category}:${Date.now()}`,
        createdAt: now,
        updatedAt: now,
        artistId: cls.artistId! as Id<"artists">,
        category: cls.category,
        severity: cls.severity,
        title,
        description,
        incidentDate: article?.publishedAt,
        incidentDateApproximate: true,
        status: "auto_detected",
        proceduralState: undefined,
        arrested: false,
        charged: false,
        convicted: false,
        settled: false,
        verifiedAt: cls.humanVerified ? now : undefined,
        verifiedByUserId: cls.verifiedByUserId,
        submittedByUserId: undefined,
        metadata: {
          sourceClassificationId: cls._id,
          confidence: cls.confidence,
          autoPromoted: true,
        },
      });

      // Link article as evidence
      if (article) {
        await ctx.db.insert("offenseEvidence", {
          legacyKey: `auto:evidence:${offenseId}:${cls.articleId}`,
          createdAt: now,
          updatedAt: now,
          offenseId,
          url: article.url,
          sourceName: undefined,
          sourceType: "news_article",
          title: article.title,
          excerpt: article.summary ?? article.content?.slice(0, 300),
          publishedDate: article.publishedAt,
          archivedUrl: undefined,
          isPrimarySource: true,
          credibilityScore: cls.confidence,
          submittedByUserId: undefined,
          metadata: {},
        });
      }

      offenseKeys.add(dedupKey);
      promoted++;
    }

    // If we promoted anything, rebuild the index
    if (promoted > 0) {
      await ctx.scheduler.runAfter(
        0,
        internal.offensePipeline.rebuildOffendingArtistIndex,
        {},
      );
    }

    return { promoted, skipped, totalEligible: eligible.length };
  },
});

/**
 * Daily sweep: find summaries older than 24h and schedule recompute.
 */
export const dailySweep = internalMutation({
  args: {},
  handler: async (ctx) => {
    const cutoff = new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString();
    const allSummaries = await ctx.db.query("userOffenseSummaries").collect();

    let scheduled = 0;
    for (const summary of allSummaries) {
      if (summary.computedAt < cutoff) {
        await ctx.scheduler.runAfter(
          0,
          internal.offensePipeline.recomputeUserOffenseSummary,
          { userId: summary.userId, triggerReason: "scheduled" },
        );
        scheduled++;
      }
    }

    return { scheduled };
  },
});
