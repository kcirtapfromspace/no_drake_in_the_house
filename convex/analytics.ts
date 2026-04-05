import { v } from "convex/values";
import { internalMutation, mutation, query } from "./_generated/server";
import { nowIso, requireCurrentUser, requireOwner } from "./lib/auth";
import { CATEGORY_COPY } from "./categories";

const severityWeight: Record<string, number> = {
  minor: 1,
  moderate: 3,
  severe: 7,
  egregious: 12,
};

const SIMULATED_PAYOUT_PER_STREAM = 0.004;

export const artistOverview = query({
  args: {
    artistId: v.id("artists"),
  },
  handler: async (ctx, args) => {
    await requireCurrentUser(ctx);
    const [offenses, collaborations, blocks] = await Promise.all([
      ctx.db
        .query("artistOffenses")
        .withIndex("by_artistId", (q) => q.eq("artistId", args.artistId))
        .collect(),
      ctx.db
        .query("artistCollaborations")
        .withIndex("by_artistId1", (q) => q.eq("artistId1", args.artistId))
        .collect(),
      ctx.db
        .query("userArtistBlocks")
        .withIndex("by_artistId", (q) => q.eq("artistId", args.artistId))
        .collect(),
    ]);

    // Use indexed lookups per offense instead of scanning ALL evidence
    let evidenceCount = 0;
    for (const offense of offenses) {
      const evidence = await ctx.db
        .query("offenseEvidence")
        .withIndex("by_offenseId", (q) => q.eq("offenseId", offense._id))
        .collect();
      evidenceCount += evidence.length;
    }

    const troubleScore = offenses.reduce(
      (total, offense) => total + (severityWeight[offense.severity] ?? 2),
      0,
    );

    return {
      offense_count: offenses.length,
      evidence_count: evidenceCount,
      collaboration_count: collaborations.length,
      block_count: blocks.length,
      trouble_score: troubleScore,
      severity_breakdown: offenses.reduce<Record<string, number>>((acc, offense) => {
        acc[offense.severity] = (acc[offense.severity] ?? 0) + 1;
        return acc;
      }, {}),
      categories: Array.from(new Set(offenses.map((offense) => offense.category))),
    };
  },
});

export const dashboardSummary = query({
  args: {},
  handler: async (ctx) => {
    const { user } = await requireCurrentUser(ctx);
    const [blocks, scans, subscriptions] = await Promise.all([
      ctx.db
        .query("userArtistBlocks")
        .withIndex("by_userId", (q) => q.eq("userId", user._id))
        .collect(),
      ctx.db
        .query("libraryScans")
        .withIndex("by_userId", (q) => q.eq("userId", user._id))
        .collect(),
      ctx.db
        .query("categorySubscriptions")
        .withIndex("by_userId", (q) => q.eq("userId", user._id))
        .collect(),
    ]);

    const latestScan = scans.sort((left, right) =>
      right.scanStartedAt.localeCompare(left.scanStartedAt),
    )[0];

    return {
      dnp_count: blocks.length,
      category_count: subscriptions.length,
      latest_scan: latestScan ?? null,
    };
  },
});

// --- Phase 3: Extended analytics ---

export const dashboard = query({
  args: {},
  handler: async (ctx) => {
    const { user } = await requireCurrentUser(ctx);
    const [blocks, scans, subscriptions, summary, offenseIndex] =
      await Promise.all([
        ctx.db
          .query("userArtistBlocks")
          .withIndex("by_userId", (q) => q.eq("userId", user._id))
          .collect(),
        ctx.db
          .query("libraryScans")
          .withIndex("by_userId", (q) => q.eq("userId", user._id))
          .collect(),
        ctx.db
          .query("categorySubscriptions")
          .withIndex("by_userId", (q) => q.eq("userId", user._id))
          .collect(),
        // Use precomputed summary instead of scanning ALL user tracks
        ctx.db
          .query("userOffenseSummaries")
          .withIndex("by_userId", (q) => q.eq("userId", user._id))
          .unique(),
        // Use the small index table instead of scanning ALL artistOffenses
        ctx.db.query("offendingArtistIndex").collect(),
      ]);

    const latestScan = scans.sort((a, b) =>
      b.scanStartedAt.localeCompare(a.scanStartedAt),
    )[0];

    // Sum offense counts from the index (much smaller than artistOffenses)
    const totalOffenses = offenseIndex.reduce(
      (sum, row) => sum + row.offenseCount,
      0,
    );

    return {
      dnp_count: blocks.length,
      category_count: subscriptions.length,
      library_track_count: summary?.totalTracks ?? 0,
      total_offenses_in_db: totalOffenses,
      latest_scan: latestScan ?? null,
    };
  },
});

export const userStats = query({
  args: {},
  handler: async (ctx) => {
    const { user } = await requireCurrentUser(ctx);
    const [blocks, subscriptions, summary, connections, scans] =
      await Promise.all([
        ctx.db
          .query("userArtistBlocks")
          .withIndex("by_userId", (q) => q.eq("userId", user._id))
          .collect(),
        ctx.db
          .query("categorySubscriptions")
          .withIndex("by_userId", (q) => q.eq("userId", user._id))
          .collect(),
        // Use precomputed summary for track count
        ctx.db
          .query("userOffenseSummaries")
          .withIndex("by_userId", (q) => q.eq("userId", user._id))
          .unique(),
        // Get providers from connections (tiny table) instead of scanning tracks
        ctx.db
          .query("providerConnections")
          .withIndex("by_userId", (q) => q.eq("userId", user._id))
          .collect(),
        // Use scan data for per-provider counts
        ctx.db
          .query("libraryScans")
          .withIndex("by_userId", (q) => q.eq("userId", user._id))
          .collect(),
      ]);

    const providers = connections
      .filter((c) => c.status === "active")
      .map((c) => c.provider);

    // Build per-provider track counts from latest scans
    const providerCounts: Record<string, number> = {};
    for (const provider of providers) {
      const latestScan = scans
        .filter((s) => s.provider === provider)
        .sort((a, b) => b.scanStartedAt.localeCompare(a.scanStartedAt))[0];
      providerCounts[provider] = latestScan?.totalTracks ?? 0;
    }

    return {
      blocked_artists: blocks.length,
      library_tracks: summary?.totalTracks ?? 0,
      category_subscriptions: subscriptions.length,
      connected_providers: providers,
      provider_track_counts: providerCounts,
    };
  },
});

export const systemHealth = query({
  args: {},
  handler: async (ctx) => {
    await requireOwner(ctx);

    // Use bounded queries and the offendingArtistIndex instead of scanning
    // entire tables. Counts are approximate but avoid massive bandwidth.
    const [offenseIndex, users, failedSyncs, recentSyncs] = await Promise.all([
      ctx.db.query("offendingArtistIndex").collect(),
      ctx.db.query("users").collect(),
      ctx.db
        .query("platformSyncRuns")
        .withIndex("by_status", (q) => q.eq("status", "failed"))
        .take(100),
      ctx.db.query("platformSyncRuns").order("desc").take(100),
    ]);

    const totalOffenses = offenseIndex.reduce(
      (sum, row) => sum + row.offenseCount,
      0,
    );

    return {
      total_artists: offenseIndex.length,
      total_offenses: totalOffenses,
      total_users: users.length,
      total_tracks: null,
      total_sync_runs: recentSyncs.length,
      failed_sync_runs: failedSyncs.length,
      healthy: failedSyncs.length < 10,
    };
  },
});

export const trendSummary = query({
  args: {},
  handler: async (ctx) => {
    await requireCurrentUser(ctx);
    // Use offendingArtistIndex (much smaller) instead of scanning all offenses
    const index = await ctx.db.query("offendingArtistIndex").collect();

    let totalOffenses = 0;
    const byCategory: Record<string, number> = {};
    const bySeverity: Record<string, number> = {};

    for (const row of index) {
      totalOffenses += row.offenseCount;
      for (const cat of row.categories) {
        byCategory[cat] = (byCategory[cat] ?? 0) + 1;
      }
      bySeverity[row.highestSeverity] =
        (bySeverity[row.highestSeverity] ?? 0) + row.offenseCount;
    }

    return {
      total_offenses: totalOffenses,
      by_category: byCategory,
      by_severity: bySeverity,
    };
  },
});

export const risingArtists = query({
  args: {
    limit: v.optional(v.number()),
  },
  handler: async (ctx, args) => {
    await requireCurrentUser(ctx);
    // Use offendingArtistIndex (pre-aggregated) instead of scanning all offenses
    const index = await ctx.db.query("offendingArtistIndex").collect();
    const artistScores = new Map<string, number>();

    for (const row of index) {
      artistScores.set(row.artistId as string, row.severityTotal);
    }

    // Fetch the most recent trouble_scores snapshot for delta comparison
    const snapshots = await ctx.db
      .query("derivedSnapshots")
      .withIndex("by_kind_subjectKey", (q) => q.eq("kind", "trouble_scores"))
      .collect();
    const latestSnapshot = snapshots.sort((a, b) =>
      b.computedAt.localeCompare(a.computedAt),
    )[0];
    const previousScores: Record<string, number> =
      latestSnapshot?.payload?.scores ?? {};

    // Compute delta (current - previous) for each artist
    const deltas: Array<{
      artistId: string;
      score: number;
      delta: number | null;
    }> = [];
    for (const [artistId, score] of artistScores) {
      const prev = previousScores[artistId];
      deltas.push({
        artistId,
        score,
        delta: prev !== undefined ? score - prev : null,
      });
    }

    // Sort by delta descending (new artists without a previous snapshot go last)
    const sorted = deltas
      .sort((a, b) => {
        if (a.delta === null && b.delta === null) return b.score - a.score;
        if (a.delta === null) return 1;
        if (b.delta === null) return -1;
        return b.delta - a.delta;
      })
      .slice(0, args.limit ?? 10);

    const results = await Promise.all(
      sorted.map(async ({ artistId, score, delta }) => {
        const artist = await ctx.db.get(artistId as any);
        return {
          artist_id: artistId,
          artist_name: (artist as any)?.canonicalName ?? "Unknown",
          trouble_score: score,
          delta,
          is_new: delta === null,
        };
      }),
    );

    return results;
  },
});

export const fallingArtists = query({
  args: {
    limit: v.optional(v.number()),
  },
  handler: async (ctx, args) => {
    await requireCurrentUser(ctx);
    // Use offendingArtistIndex (pre-aggregated) instead of scanning all offenses
    const index = await ctx.db.query("offendingArtistIndex").collect();
    const artistScores = new Map<string, number>();

    for (const row of index) {
      artistScores.set(row.artistId as string, row.severityTotal);
    }

    // Fetch the most recent trouble_scores snapshot for delta comparison
    const snapshots = await ctx.db
      .query("derivedSnapshots")
      .withIndex("by_kind_subjectKey", (q) => q.eq("kind", "trouble_scores"))
      .collect();
    const latestSnapshot = snapshots.sort((a, b) =>
      b.computedAt.localeCompare(a.computedAt),
    )[0];
    const previousScores: Record<string, number> =
      latestSnapshot?.payload?.scores ?? {};

    // Compute delta (current - previous) for each artist; exclude new artists
    const deltas: Array<{
      artistId: string;
      score: number;
      delta: number;
    }> = [];
    for (const [artistId, score] of artistScores) {
      const prev = previousScores[artistId];
      if (prev !== undefined) {
        deltas.push({ artistId, score, delta: score - prev });
      }
    }

    // Sort by delta ascending (most negative = biggest fall)
    const sorted = deltas
      .sort((a, b) => a.delta - b.delta)
      .slice(0, args.limit ?? 10);

    const results = await Promise.all(
      sorted.map(async ({ artistId, score, delta }) => {
        const artist = await ctx.db.get(artistId as any);
        return {
          artist_id: artistId,
          artist_name: (artist as any)?.canonicalName ?? "Unknown",
          trouble_score: score,
          delta,
        };
      }),
    );

    return results;
  },
});

export const reportTypes = query({
  args: {},
  handler: async (ctx) => {
    await requireCurrentUser(ctx);
    return [
      { type: "library_scan", name: "Library Scan Report", description: "Full library analysis" },
      { type: "offense_summary", name: "Offense Summary", description: "All tracked offenses" },
      { type: "enforcement_log", name: "Enforcement Log", description: "History of enforcement actions" },
      { type: "taste_grade", name: "Taste Grade Report", description: "Library taste analysis" },
    ];
  },
});

export const generateReport = mutation({
  args: {
    type: v.string(),
    parameters: v.optional(v.any()),
  },
  handler: async (ctx, args) => {
    const { user } = await requireCurrentUser(ctx);
    const now = nowIso();

    const snapshotId = await ctx.db.insert("derivedSnapshots", {
      legacyKey: `runtime:report:${user._id}:${args.type}:${Date.now()}`,
      kind: `report:${args.type}`,
      subjectKey: user._id,
      payload: { type: args.type, parameters: args.parameters, status: "generated" },
      computedAt: now,
      createdAt: now,
      updatedAt: now,
    });

    return { report_id: snapshotId, type: args.type, status: "generated" };
  },
});

export const getReport = query({
  args: {
    reportId: v.id("derivedSnapshots"),
  },
  handler: async (ctx, args) => {
    const { user } = await requireCurrentUser(ctx);
    const report = await ctx.db.get(args.reportId);
    if (!report || report.subjectKey !== user._id) return null;
    return {
      id: report._id,
      kind: report.kind,
      payload: report.payload,
      computed_at: report.computedAt,
    };
  },
});

export const troubleLeaderboard = query({
  args: {
    limit: v.optional(v.number()),
  },
  handler: async (ctx, args) => {
    await requireCurrentUser(ctx);
    // Use offendingArtistIndex (pre-aggregated) instead of scanning all offenses
    const index = await ctx.db.query("offendingArtistIndex").collect();

    const sorted = index
      .map((row) => ({
        artistId: row.artistId as string,
        score: row.severityTotal,
      }))
      .sort((a, b) => b.score - a.score)
      .slice(0, args.limit ?? 20);

    const leaderboard = await Promise.all(
      sorted.map(async ({ artistId, score }, rank) => {
        const artist = await ctx.db.get(artistId as any);
        return {
          rank: rank + 1,
          artist_id: artistId,
          artist_name: (artist as any)?.canonicalName ?? "Unknown",
          trouble_score: score,
        };
      }),
    );

    return { leaderboard, total_artists: index.length };
  },
});

export const troubleDistribution = query({
  args: {},
  handler: async (ctx) => {
    await requireCurrentUser(ctx);
    // Use offendingArtistIndex (pre-aggregated) instead of scanning all offenses
    const index = await ctx.db.query("offendingArtistIndex").collect();

    const buckets: Record<string, number> = {
      "0-5": 0,
      "6-15": 0,
      "16-30": 0,
      "31-50": 0,
      "51+": 0,
    };

    for (const row of index) {
      const score = row.severityTotal;
      if (score <= 5) buckets["0-5"]++;
      else if (score <= 15) buckets["6-15"]++;
      else if (score <= 30) buckets["16-30"]++;
      else if (score <= 50) buckets["31-50"]++;
      else buckets["51+"]++;
    }

    return { distribution: buckets, total_artists: index.length };
  },
});

export const artistTroubleScore = query({
  args: {
    artistId: v.id("artists"),
  },
  handler: async (ctx, args) => {
    await requireCurrentUser(ctx);
    const artist = await ctx.db.get(args.artistId);
    if (!artist) return null;

    const offenses = await ctx.db
      .query("artistOffenses")
      .withIndex("by_artistId", (q) => q.eq("artistId", args.artistId))
      .collect();

    const score = offenses.reduce(
      (total, o) => total + (severityWeight[o.severity] ?? 2),
      0,
    );

    return {
      artist_id: args.artistId,
      artist_name: artist.canonicalName,
      trouble_score: score,
      offense_count: offenses.length,
      severity_breakdown: offenses.reduce<Record<string, number>>((acc, o) => {
        acc[o.severity] = (acc[o.severity] ?? 0) + 1;
        return acc;
      }, {}),
    };
  },
});

export const revenueDistribution = query({
  args: {},
  handler: async (ctx) => {
    const { user } = await requireCurrentUser(ctx);
    // Use precomputed summary instead of scanning ALL tracks + ALL offenses
    const summary = await ctx.db
      .query("userOffenseSummaries")
      .withIndex("by_userId", (q) => q.eq("userId", user._id))
      .unique();

    const totalTracks = summary?.totalTracks ?? 0;
    const flaggedTracks = summary?.flaggedTrackCount ?? 0;
    const cleanTracks = totalTracks - flaggedTracks;

    const cleanRevenue = cleanTracks * SIMULATED_PAYOUT_PER_STREAM;
    const problematicRevenue = flaggedTracks * SIMULATED_PAYOUT_PER_STREAM;

    return {
      clean_revenue: Number(cleanRevenue.toFixed(2)),
      problematic_revenue: Number(problematicRevenue.toFixed(2)),
      total_revenue: Number((cleanRevenue + problematicRevenue).toFixed(2)),
      problematic_percentage:
        totalTracks > 0
          ? Number(((flaggedTracks / totalTracks) * 100).toFixed(1))
          : 0,
    };
  },
});

export const topArtistsByRevenue = query({
  args: {
    limit: v.optional(v.number()),
  },
  handler: async (ctx, args) => {
    const { user } = await requireCurrentUser(ctx);
    // Use precomputed summary for offender data, and scan only user tracks
    const summary = await ctx.db
      .query("userOffenseSummaries")
      .withIndex("by_userId", (q) => q.eq("userId", user._id))
      .unique();

    if (!summary) return [];

    // Offenders already have track counts from the summary
    const sorted = summary.offenders
      .map((o: any) => ({
        artistId: o.artistId as string,
        artistName: o.artistName as string,
        trackCount: o.trackCount as number,
      }))
      .sort((a: any, b: any) => b.trackCount - a.trackCount)
      .slice(0, args.limit ?? 10);

    return sorted.map((o: any) => ({
      artist_id: o.artistId,
      artist_name: o.artistName,
      estimated_revenue: Number(
        (o.trackCount * SIMULATED_PAYOUT_PER_STREAM).toFixed(4),
      ),
    }));
  },
});

export const problematicRevenue = query({
  args: {},
  handler: async (ctx) => {
    const { user } = await requireCurrentUser(ctx);
    // Use precomputed summary instead of scanning ALL tracks + ALL offenses
    const summary = await ctx.db
      .query("userOffenseSummaries")
      .withIndex("by_userId", (q) => q.eq("userId", user._id))
      .unique();

    if (!summary) return { artists: [], total_problematic: 0 };

    const results = summary.offenders.map((o: any) => ({
      artist_id: o.artistId as string,
      artist_name: o.artistName as string,
      estimated_revenue: Number(
        ((o.trackCount as number) * SIMULATED_PAYOUT_PER_STREAM).toFixed(4),
      ),
    }));

    const totalProblematic = Number(
      (summary.flaggedTrackCount * SIMULATED_PAYOUT_PER_STREAM).toFixed(2),
    );

    return { artists: results, total_problematic: totalProblematic };
  },
});

export const payoutRates = query({
  args: {},
  handler: async (ctx) => {
    await requireCurrentUser(ctx);
    return {
      spotify: { per_stream: 0.004, currency: "USD" },
      apple_music: { per_stream: 0.008, currency: "USD" },
      tidal: { per_stream: 0.012, currency: "USD" },
      youtube: { per_stream: 0.002, currency: "USD" },
    };
  },
});

export const artistRevenue = query({
  args: {
    artistId: v.id("artists"),
  },
  handler: async (ctx, args) => {
    await requireCurrentUser(ctx);
    const artist = await ctx.db.get(args.artistId);
    if (!artist) return null;

    const tracks = await ctx.db
      .query("userLibraryTracks")
      .withIndex("by_artistId", (q) => q.eq("artistId", args.artistId))
      .collect();

    const byProvider = new Map<string, number>();
    for (const track of tracks) {
      byProvider.set(
        track.provider,
        (byProvider.get(track.provider) ?? 0) + SIMULATED_PAYOUT_PER_STREAM,
      );
    }

    const total = [...byProvider.values()].reduce((s, v) => s + v, 0);

    return {
      artist_id: args.artistId,
      artist_name: artist.canonicalName,
      total_estimated_revenue: Number(total.toFixed(4)),
      by_provider: Object.fromEntries(
        [...byProvider.entries()].map(([k, v]) => [k, Number(v.toFixed(4))]),
      ),
      track_count: tracks.length,
    };
  },
});

export const globalCategoryRevenue = query({
  args: {},
  handler: async (ctx) => {
    const { user } = await requireCurrentUser(ctx);
    // Use precomputed summary instead of scanning ALL offenses + ALL tracks
    const summary = await ctx.db
      .query("userOffenseSummaries")
      .withIndex("by_userId", (q) => q.eq("userId", user._id))
      .unique();

    if (!summary) return {};

    // Approximate category revenue from offender data in the summary
    const catRevenue = new Map<string, number>();
    for (const o of summary.offenders as any[]) {
      const cats: string[] = o.categories ?? [];
      const trackRev = (o.trackCount as number) * SIMULATED_PAYOUT_PER_STREAM;
      for (const cat of cats) {
        catRevenue.set(
          cat,
          (catRevenue.get(cat) ?? 0) + trackRev / cats.length,
        );
      }
    }

    return Object.fromEntries(
      [...catRevenue.entries()].map(([k, val]) => [k, Number(val.toFixed(4))]),
    );
  },
});

export const offenseCategories = query({
  args: {},
  handler: async (ctx) => {
    await requireCurrentUser(ctx);
    // Use offendingArtistIndex instead of scanning all artistOffenses
    const index = await ctx.db.query("offendingArtistIndex").collect();
    const categories = new Map<string, number>();

    for (const row of index) {
      for (const cat of row.categories) {
        categories.set(cat, (categories.get(cat) ?? 0) + 1);
      }
    }

    return [...categories.entries()]
      .map(([category, count]) => ({ category, offense_count: count }))
      .sort((a, b) => b.offense_count - a.offense_count);
  },
});

export const categoryRevenue = query({
  args: {
    category: v.string(),
  },
  handler: async (ctx, args) => {
    const { user } = await requireCurrentUser(ctx);
    // Use precomputed summary instead of scanning ALL tracks
    const summary = await ctx.db
      .query("userOffenseSummaries")
      .withIndex("by_userId", (q) => q.eq("userId", user._id))
      .unique();

    if (!summary) {
      return {
        category: args.category,
        total_estimated_revenue: 0,
        artist_count: 0,
        top_artists: [],
      };
    }

    // Filter offenders by category from precomputed data
    const categoryOffenders = (summary.offenders as any[]).filter(
      (o: any) => (o.categories as string[])?.includes(args.category),
    );

    const totalRevenue = categoryOffenders.reduce(
      (sum: number, o: any) =>
        sum + (o.trackCount as number) * SIMULATED_PAYOUT_PER_STREAM,
      0,
    );

    const topArtists = categoryOffenders
      .sort((a: any, b: any) => (b.trackCount as number) - (a.trackCount as number))
      .slice(0, 10)
      .map((o: any) => ({
        artist_id: o.artistId as string,
        artist_name: o.artistName as string,
        estimated_revenue: Number(
          ((o.trackCount as number) * SIMULATED_PAYOUT_PER_STREAM).toFixed(4),
        ),
      }));

    return {
      category: args.category,
      total_estimated_revenue: Number(totalRevenue.toFixed(4)),
      artist_count: categoryOffenders.length,
      top_artists: topArtists,
    };
  },
});

export const artistDiscography = query({
  args: {
    artistId: v.id("artists"),
  },
  handler: async (ctx, args) => {
    await requireCurrentUser(ctx);
    const artist = await ctx.db.get(args.artistId);
    if (!artist) return null;

    const albumArtistLinks = await ctx.db
      .query("albumArtists")
      .withIndex("by_artistId", (q) => q.eq("artistId", args.artistId))
      .collect();

    const albums = await Promise.all(
      albumArtistLinks.map(async (link) => {
        const album = await ctx.db.get(link.albumId);
        if (!album) return null;

        const albumTracks = await ctx.db
          .query("tracks")
          .withIndex("by_albumId", (q) => q.eq("albumId", album._id))
          .collect();

        return {
          id: album._id,
          title: album.title,
          release_date: album.releaseDate,
          track_count: albumTracks.length,
        };
      }),
    );

    return {
      artist_id: args.artistId,
      artist_name: artist.canonicalName,
      albums: albums.filter(Boolean),
    };
  },
});

export const userExposure = query({
  args: {},
  handler: async (ctx) => {
    const { user } = await requireCurrentUser(ctx);
    // Use precomputed summary instead of scanning all tracks + all offenses
    const summary = await ctx.db
      .query("userOffenseSummaries")
      .withIndex("by_userId", (q) => q.eq("userId", user._id))
      .unique();

    if (!summary) {
      return { total_tracks: 0, category_exposure: {} };
    }

    const exposure = new Map<string, { tracks: number; revenue: number }>();
    for (const o of summary.offenders as any[]) {
      const cats: string[] = o.categories ?? [];
      const trackCount = o.trackCount as number;
      for (const cat of cats) {
        const existing = exposure.get(cat) ?? { tracks: 0, revenue: 0 };
        existing.tracks += trackCount;
        existing.revenue +=
          (trackCount * SIMULATED_PAYOUT_PER_STREAM) / cats.length;
        exposure.set(cat, existing);
      }
    }

    return {
      total_tracks: summary.totalTracks,
      category_exposure: Object.fromEntries(
        [...exposure.entries()].map(([k, val]) => [
          k,
          {
            tracks: val.tracks,
            estimated_revenue: Number(val.revenue.toFixed(4)),
          },
        ]),
      ),
    };
  },
});

// --- Phase 4: Time-series trend tracking ---

export const snapshotTroubleScores = internalMutation({
  args: {},
  handler: async (ctx) => {
    // Use offendingArtistIndex (pre-aggregated) instead of scanning all offenses
    const index = await ctx.db.query("offendingArtistIndex").collect();
    const artistScores = new Map<string, number>();

    for (const row of index) {
      artistScores.set(row.artistId as string, row.severityTotal);
    }

    const scores: Record<string, number> = Object.fromEntries(artistScores);
    const now = nowIso();

    await ctx.db.insert("derivedSnapshots", {
      legacyKey: `runtime:trouble_scores:${Date.now()}`,
      kind: "trouble_scores",
      subjectKey: "global",
      payload: { scores, artist_count: artistScores.size },
      computedAt: now,
      createdAt: now,
      updatedAt: now,
    });

    return { snapshotted: artistScores.size, computedAt: now };
  },
});

// --- Phase 5: Admin / Owner metrics ---

export const adminMetrics = query({
  args: {},
  handler: async (ctx) => {
    await requireOwner(ctx);

    // Use offendingArtistIndex + bounded queries instead of 6 full table scans
    const [offenseIndex, newsArticles, classifications, syncRuns] =
      await Promise.all([
        ctx.db.query("offendingArtistIndex").collect(),
        ctx.db.query("newsArticles").collect(),
        ctx.db.query("newsOffenseClassifications").collect(),
        ctx.db
          .query("platformSyncRuns")
          .withIndex("by_platform", (q) => q.eq("platform", "evidence_finder"))
          .collect(),
      ]);

    const totalOffenses = offenseIndex.reduce(
      (sum, row) => sum + row.offenseCount,
      0,
    );

    // --- Catalog totals ---
    const pendingClassifications = classifications.filter(
      (c) => c.humanVerified !== true && (c.confidence ?? 0) < 0.7,
    ).length;

    const catalog = {
      total_artists: offenseIndex.length,
      total_offenses: totalOffenses,
      total_evidence: null,
      total_news_articles: newsArticles.length,
      total_classifications: classifications.length,
      pending_classifications: pendingClassifications,
    };

    // --- Category coverage from index ---
    const allCategories = Object.keys(CATEGORY_COPY);
    const categoryCoverage = allCategories.map((cat) => {
      const catArtists = offenseIndex.filter((row) =>
        row.categories.includes(cat),
      );
      return {
        category: cat,
        offense_count: catArtists.reduce((s, r) => s + r.offenseCount, 0),
        unique_artist_count: catArtists.length,
        evidence_coverage_pct: null,
      };
    });

    // --- Backfill pipeline health ---
    const now = Date.now();
    const day1Cutoff = new Date(now - 24 * 60 * 60 * 1000).toISOString();
    const day7Cutoff = new Date(now - 7 * 24 * 60 * 60 * 1000).toISOString();

    function summarizeRuns(runs: typeof syncRuns) {
      let total = 0;
      let success = 0;
      let failed = 0;
      let offensesFound = 0;
      for (const r of runs) {
        total++;
        if (r.status === "completed") success++;
        else if (r.status === "failed") failed++;
        const meta = r.metadata as any;
        offensesFound += meta?.offensesFound ?? 0;
      }
      return { total, success, failed, offenses_found: offensesFound };
    }

    const runs24h = syncRuns.filter(
      (r) => r.startedAt && r.startedAt > day1Cutoff,
    );
    const runs7d = syncRuns.filter(
      (r) => r.startedAt && r.startedAt > day7Cutoff,
    );

    const pipeline = {
      investigated: null,
      never_investigated: null,
      stale: null,
      recent_runs_24h: summarizeRuns(runs24h),
      recent_runs_7d: summarizeRuns(runs7d),
    };

    // --- Growth deltas ---
    const allSnapshots = await ctx.db
      .query("derivedSnapshots")
      .withIndex("by_kind_subjectKey", (q) => q.eq("kind", "catalog_metrics"))
      .collect();

    const snapshots = allSnapshots.filter((s) => s.subjectKey === "global");

    const latestSnapshot = snapshots.sort((a, b) =>
      b.computedAt.localeCompare(a.computedAt),
    )[0];

    let growth: {
      artists_delta: number;
      offenses_delta: number;
      evidence_delta: number | null;
      snapshot_date: string;
    } | null = null;

    if (latestSnapshot?.payload) {
      const prev = latestSnapshot.payload as any;
      growth = {
        artists_delta: catalog.total_artists - (prev.total_artists ?? 0),
        offenses_delta: catalog.total_offenses - (prev.total_offenses ?? 0),
        evidence_delta: null,
        snapshot_date: latestSnapshot.computedAt,
      };
    }

    return {
      catalog,
      category_coverage: categoryCoverage,
      pipeline,
      growth,
      evidence_density: null,
    };
  },
});

export const snapshotCatalogMetrics = internalMutation({
  args: {},
  handler: async (ctx) => {
    // Use offendingArtistIndex + bounded queries instead of full table scans.
    // This runs daily via cron so accuracy is acceptable with approximate counts.
    const [offenseIndex, newsArticles, classifications] = await Promise.all([
      ctx.db.query("offendingArtistIndex").collect(),
      ctx.db.query("newsArticles").collect(),
      ctx.db.query("newsOffenseClassifications").collect(),
    ]);

    const totalOffenses = offenseIndex.reduce(
      (sum, row) => sum + row.offenseCount,
      0,
    );

    const now = nowIso();
    const payload = {
      total_artists: offenseIndex.length,
      total_offenses: totalOffenses,
      total_evidence: null,
      total_news_articles: newsArticles.length,
      total_classifications: classifications.length,
    };

    await ctx.db.insert("derivedSnapshots", {
      legacyKey: `runtime:catalog_metrics:${Date.now()}`,
      kind: "catalog_metrics",
      subjectKey: "global",
      payload,
      computedAt: now,
      createdAt: now,
      updatedAt: now,
    });

    return { snapshotted: true, computedAt: now, ...payload };
  },
});

export const catalogMetricsHistory = query({
  args: {
    limit: v.optional(v.number()),
  },
  handler: async (ctx, args) => {
    await requireOwner(ctx);

    const allSnapshots = await ctx.db
      .query("derivedSnapshots")
      .withIndex("by_kind_subjectKey", (q) => q.eq("kind", "catalog_metrics"))
      .collect();

    const snapshots = allSnapshots.filter((s) => s.subjectKey === "global");

    return snapshots
      .sort((a, b) => b.computedAt.localeCompare(a.computedAt))
      .slice(0, args.limit ?? 30)
      .reverse()
      .map((s) => ({
        date: s.computedAt,
        ...(s.payload as Record<string, number>),
      }));
  },
});
