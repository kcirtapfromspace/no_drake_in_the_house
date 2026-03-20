import { v } from "convex/values";
import { query } from "./_generated/server";
import { requireCurrentUser } from "./lib/auth";

const severityWeight: Record<string, number> = {
  minor: 1,
  moderate: 3,
  severe: 7,
  egregious: 12,
};

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
