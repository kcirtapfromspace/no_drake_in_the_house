import { v } from "convex/values";
import { mutation, query } from "./_generated/server";
import { nowIso, requireCurrentUser } from "./lib/auth";

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
    const [offenses, evidence, collaborations, blocks] = await Promise.all([
      ctx.db
        .query("artistOffenses")
        .withIndex("by_artistId", (q) => q.eq("artistId", args.artistId))
        .collect(),
      ctx.db.query("offenseEvidence").collect(),
      ctx.db
        .query("artistCollaborations")
        .withIndex("by_artistId1", (q) => q.eq("artistId1", args.artistId))
        .collect(),
      ctx.db
        .query("userArtistBlocks")
        .withIndex("by_artistId", (q) => q.eq("artistId", args.artistId))
        .collect(),
    ]);

    const offenseIds = new Set(offenses.map((offense) => offense._id));
    const evidenceCount = evidence.filter((item) => offenseIds.has(item.offenseId)).length;
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
    const [blocks, scans, subscriptions, tracks, offenses] = await Promise.all([
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
      ctx.db
        .query("userLibraryTracks")
        .withIndex("by_userId", (q) => q.eq("userId", user._id))
        .collect(),
      ctx.db.query("artistOffenses").collect(),
    ]);

    const latestScan = scans.sort((a, b) =>
      b.scanStartedAt.localeCompare(a.scanStartedAt),
    )[0];

    return {
      dnp_count: blocks.length,
      category_count: subscriptions.length,
      library_track_count: tracks.length,
      total_offenses_in_db: offenses.length,
      latest_scan: latestScan ?? null,
    };
  },
});

export const userStats = query({
  args: {},
  handler: async (ctx) => {
    const { user } = await requireCurrentUser(ctx);
    const [blocks, tracks, subscriptions] = await Promise.all([
      ctx.db
        .query("userArtistBlocks")
        .withIndex("by_userId", (q) => q.eq("userId", user._id))
        .collect(),
      ctx.db
        .query("userLibraryTracks")
        .withIndex("by_userId", (q) => q.eq("userId", user._id))
        .collect(),
      ctx.db
        .query("categorySubscriptions")
        .withIndex("by_userId", (q) => q.eq("userId", user._id))
        .collect(),
    ]);

    const providers = [...new Set(tracks.map((t) => t.provider))];

    return {
      blocked_artists: blocks.length,
      library_tracks: tracks.length,
      category_subscriptions: subscriptions.length,
      connected_providers: providers,
      provider_track_counts: providers.reduce<Record<string, number>>(
        (acc, p) => {
          acc[p] = tracks.filter((t) => t.provider === p).length;
          return acc;
        },
        {},
      ),
    };
  },
});

export const systemHealth = query({
  args: {},
  handler: async (ctx) => {
    const [artists, offenses, users, tracks, syncRuns] = await Promise.all([
      ctx.db.query("artists").collect(),
      ctx.db.query("artistOffenses").collect(),
      ctx.db.query("users").collect(),
      ctx.db.query("userLibraryTracks").collect(),
      ctx.db.query("platformSyncRuns").collect(),
    ]);

    const failedSyncs = syncRuns.filter((r) => r.status === "failed").length;

    return {
      total_artists: artists.length,
      total_offenses: offenses.length,
      total_users: users.length,
      total_tracks: tracks.length,
      total_sync_runs: syncRuns.length,
      failed_sync_runs: failedSyncs,
      healthy: failedSyncs < 10,
    };
  },
});

export const trendSummary = query({
  args: {},
  handler: async (ctx) => {
    const offenses = await ctx.db.query("artistOffenses").collect();

    const byCategory = offenses.reduce<Record<string, number>>((acc, o) => {
      acc[o.category] = (acc[o.category] ?? 0) + 1;
      return acc;
    }, {});

    const bySeverity = offenses.reduce<Record<string, number>>((acc, o) => {
      acc[o.severity] = (acc[o.severity] ?? 0) + 1;
      return acc;
    }, {});

    return {
      total_offenses: offenses.length,
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
    const offenses = await ctx.db.query("artistOffenses").collect();
    const artistScores = new Map<string, number>();

    for (const o of offenses) {
      const id = o.artistId as string;
      artistScores.set(
        id,
        (artistScores.get(id) ?? 0) + (severityWeight[o.severity] ?? 2),
      );
    }

    const sorted = [...artistScores.entries()]
      .sort((a, b) => b[1] - a[1])
      .slice(0, args.limit ?? 10);

    const results = await Promise.all(
      sorted.map(async ([artistId, score]) => {
        const artist = await ctx.db.get(artistId as any);
        return {
          artist_id: artistId,
          artist_name: (artist as any)?.canonicalName ?? "Unknown",
          trouble_score: score,
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
    const offenses = await ctx.db.query("artistOffenses").collect();
    const artistScores = new Map<string, number>();

    for (const o of offenses) {
      const id = o.artistId as string;
      artistScores.set(
        id,
        (artistScores.get(id) ?? 0) + (severityWeight[o.severity] ?? 2),
      );
    }

    const sorted = [...artistScores.entries()]
      .sort((a, b) => a[1] - b[1])
      .slice(0, args.limit ?? 10);

    const results = await Promise.all(
      sorted.map(async ([artistId, score]) => {
        const artist = await ctx.db.get(artistId as any);
        return {
          artist_id: artistId,
          artist_name: (artist as any)?.canonicalName ?? "Unknown",
          trouble_score: score,
        };
      }),
    );

    return results;
  },
});

export const reportTypes = query({
  args: {},
  handler: async () => {
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
    const report = await ctx.db.get(args.reportId);
    if (!report) return null;
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
    const offenses = await ctx.db.query("artistOffenses").collect();
    const artistScores = new Map<string, number>();

    for (const o of offenses) {
      const id = o.artistId as string;
      artistScores.set(
        id,
        (artistScores.get(id) ?? 0) + (severityWeight[o.severity] ?? 2),
      );
    }

    const sorted = [...artistScores.entries()]
      .sort((a, b) => b[1] - a[1])
      .slice(0, args.limit ?? 20);

    const leaderboard = await Promise.all(
      sorted.map(async ([artistId, score], rank) => {
        const artist = await ctx.db.get(artistId as any);
        return {
          rank: rank + 1,
          artist_id: artistId,
          artist_name: (artist as any)?.canonicalName ?? "Unknown",
          trouble_score: score,
        };
      }),
    );

    return { leaderboard, total_artists: artistScores.size };
  },
});

export const troubleDistribution = query({
  args: {},
  handler: async (ctx) => {
    const offenses = await ctx.db.query("artistOffenses").collect();
    const artistScores = new Map<string, number>();

    for (const o of offenses) {
      const id = o.artistId as string;
      artistScores.set(
        id,
        (artistScores.get(id) ?? 0) + (severityWeight[o.severity] ?? 2),
      );
    }

    const buckets: Record<string, number> = {
      "0-5": 0,
      "6-15": 0,
      "16-30": 0,
      "31-50": 0,
      "51+": 0,
    };

    for (const score of artistScores.values()) {
      if (score <= 5) buckets["0-5"]++;
      else if (score <= 15) buckets["6-15"]++;
      else if (score <= 30) buckets["16-30"]++;
      else if (score <= 50) buckets["31-50"]++;
      else buckets["51+"]++;
    }

    return { distribution: buckets, total_artists: artistScores.size };
  },
});

export const artistTroubleScore = query({
  args: {
    artistId: v.id("artists"),
  },
  handler: async (ctx, args) => {
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
    const tracks = await ctx.db.query("userLibraryTracks").collect();
    const offenses = await ctx.db.query("artistOffenses").collect();
    const offendingArtistIds = new Set(offenses.map((o) => o.artistId as string));

    let cleanRevenue = 0;
    let problematicRevenue = 0;

    for (const track of tracks) {
      const revenue = SIMULATED_PAYOUT_PER_STREAM;
      if (track.artistId && offendingArtistIds.has(track.artistId as string)) {
        problematicRevenue += revenue;
      } else {
        cleanRevenue += revenue;
      }
    }

    return {
      clean_revenue: Number(cleanRevenue.toFixed(2)),
      problematic_revenue: Number(problematicRevenue.toFixed(2)),
      total_revenue: Number((cleanRevenue + problematicRevenue).toFixed(2)),
      problematic_percentage:
        cleanRevenue + problematicRevenue > 0
          ? Number(
              (
                (problematicRevenue / (cleanRevenue + problematicRevenue)) *
                100
              ).toFixed(1),
            )
          : 0,
    };
  },
});

export const topArtistsByRevenue = query({
  args: {
    limit: v.optional(v.number()),
  },
  handler: async (ctx, args) => {
    const tracks = await ctx.db.query("userLibraryTracks").collect();
    const artistRevMap = new Map<string, number>();

    for (const track of tracks) {
      if (track.artistId) {
        const id = track.artistId as string;
        artistRevMap.set(id, (artistRevMap.get(id) ?? 0) + SIMULATED_PAYOUT_PER_STREAM);
      }
    }

    const sorted = [...artistRevMap.entries()]
      .sort((a, b) => b[1] - a[1])
      .slice(0, args.limit ?? 10);

    const results = await Promise.all(
      sorted.map(async ([artistId, rev]) => {
        const artist = await ctx.db.get(artistId as any);
        return {
          artist_id: artistId,
          artist_name: (artist as any)?.canonicalName ?? "Unknown",
          estimated_revenue: Number(rev.toFixed(4)),
        };
      }),
    );

    return results;
  },
});

export const problematicRevenue = query({
  args: {},
  handler: async (ctx) => {
    const tracks = await ctx.db.query("userLibraryTracks").collect();
    const offenses = await ctx.db.query("artistOffenses").collect();
    const offendingArtistIds = new Set(offenses.map((o) => o.artistId as string));

    const byArtist = new Map<string, number>();
    for (const track of tracks) {
      if (track.artistId && offendingArtistIds.has(track.artistId as string)) {
        const id = track.artistId as string;
        byArtist.set(id, (byArtist.get(id) ?? 0) + SIMULATED_PAYOUT_PER_STREAM);
      }
    }

    const sorted = [...byArtist.entries()].sort((a, b) => b[1] - a[1]);

    const results = await Promise.all(
      sorted.map(async ([artistId, rev]) => {
        const artist = await ctx.db.get(artistId as any);
        return {
          artist_id: artistId,
          artist_name: (artist as any)?.canonicalName ?? "Unknown",
          estimated_revenue: Number(rev.toFixed(4)),
        };
      }),
    );

    return { artists: results, total_problematic: Number(sorted.reduce((s, [, r]) => s + r, 0).toFixed(2)) };
  },
});

export const payoutRates = query({
  args: {},
  handler: async () => {
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
    const offenses = await ctx.db.query("artistOffenses").collect();
    const tracks = await ctx.db.query("userLibraryTracks").collect();

    const artistCategories = new Map<string, Set<string>>();
    for (const o of offenses) {
      const id = o.artistId as string;
      if (!artistCategories.has(id)) artistCategories.set(id, new Set());
      artistCategories.get(id)!.add(o.category);
    }

    const catRevenue = new Map<string, number>();
    for (const track of tracks) {
      if (track.artistId) {
        const cats = artistCategories.get(track.artistId as string);
        if (cats) {
          for (const cat of cats) {
            catRevenue.set(
              cat,
              (catRevenue.get(cat) ?? 0) + SIMULATED_PAYOUT_PER_STREAM / cats.size,
            );
          }
        }
      }
    }

    return Object.fromEntries(
      [...catRevenue.entries()].map(([k, v]) => [k, Number(v.toFixed(4))]),
    );
  },
});

export const offenseCategories = query({
  args: {},
  handler: async (ctx) => {
    const offenses = await ctx.db.query("artistOffenses").collect();
    const categories = new Map<string, number>();

    for (const o of offenses) {
      categories.set(o.category, (categories.get(o.category) ?? 0) + 1);
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
    const offenses = await ctx.db
      .query("artistOffenses")
      .withIndex("by_category", (q) => q.eq("category", args.category))
      .collect();

    const artistIds = new Set(offenses.map((o) => o.artistId as string));
    const tracks = await ctx.db.query("userLibraryTracks").collect();

    let totalRevenue = 0;
    const byArtist = new Map<string, number>();

    for (const track of tracks) {
      if (track.artistId && artistIds.has(track.artistId as string)) {
        totalRevenue += SIMULATED_PAYOUT_PER_STREAM;
        const id = track.artistId as string;
        byArtist.set(id, (byArtist.get(id) ?? 0) + SIMULATED_PAYOUT_PER_STREAM);
      }
    }

    const topArtists = await Promise.all(
      [...byArtist.entries()]
        .sort((a, b) => b[1] - a[1])
        .slice(0, 10)
        .map(async ([artistId, rev]) => {
          const artist = await ctx.db.get(artistId as any);
          return {
            artist_id: artistId,
            artist_name: (artist as any)?.canonicalName ?? "Unknown",
            estimated_revenue: Number(rev.toFixed(4)),
          };
        }),
    );

    return {
      category: args.category,
      total_estimated_revenue: Number(totalRevenue.toFixed(4)),
      artist_count: artistIds.size,
      top_artists: topArtists,
    };
  },
});

export const artistDiscography = query({
  args: {
    artistId: v.id("artists"),
  },
  handler: async (ctx, args) => {
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
    const tracks = await ctx.db
      .query("userLibraryTracks")
      .withIndex("by_userId", (q) => q.eq("userId", user._id))
      .collect();

    const offenses = await ctx.db.query("artistOffenses").collect();
    const artistCategories = new Map<string, Set<string>>();
    for (const o of offenses) {
      const id = o.artistId as string;
      if (!artistCategories.has(id)) artistCategories.set(id, new Set());
      artistCategories.get(id)!.add(o.category);
    }

    const exposure = new Map<string, { tracks: number; revenue: number }>();
    for (const track of tracks) {
      if (track.artistId) {
        const cats = artistCategories.get(track.artistId as string);
        if (cats) {
          for (const cat of cats) {
            const existing = exposure.get(cat) ?? { tracks: 0, revenue: 0 };
            existing.tracks++;
            existing.revenue += SIMULATED_PAYOUT_PER_STREAM / cats.size;
            exposure.set(cat, existing);
          }
        }
      }
    }

    return {
      total_tracks: tracks.length,
      category_exposure: Object.fromEntries(
        [...exposure.entries()].map(([k, v]) => [
          k,
          { tracks: v.tracks, estimated_revenue: Number(v.revenue.toFixed(4)) },
        ]),
      ),
    };
  },
});
