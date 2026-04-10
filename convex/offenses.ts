import { ConvexError, v } from "convex/values";
import { internalMutation, mutation, query } from "./_generated/server";
import { internal } from "./_generated/api";
import { nowIso, requireCurrentUser } from "./lib/auth";

export const listByArtist = query({
  args: {
    artistId: v.id("artists"),
  },
  handler: async (ctx, args) => {
    const offenses = await ctx.db
      .query("artistOffenses")
      .withIndex("by_artistId", (q) => q.eq("artistId", args.artistId))
      .collect();

    const withEvidence = await Promise.all(
      offenses.map(async (offense) => {
        const evidence = await ctx.db
          .query("offenseEvidence")
          .withIndex("by_offenseId", (q) => q.eq("offenseId", offense._id))
          .collect();

        return {
          id: offense._id,
          artist_id: offense.artistId,
          category: offense.category,
          severity: offense.severity,
          title: offense.title,
          description: offense.description,
          incident_date: offense.incidentDate,
          procedural_state: offense.proceduralState,
          status: offense.status,
          evidence,
          created_at: offense.createdAt,
          updated_at: offense.updatedAt,
        };
      }),
    );

    return { offenses: withEvidence };
  },
});

export const getOne = query({
  args: {
    offenseId: v.id("artistOffenses"),
  },
  handler: async (ctx, args) => {
    const offense = await ctx.db.get(args.offenseId);
    if (!offense) {
      return null;
    }

    const evidence = await ctx.db
      .query("offenseEvidence")
      .withIndex("by_offenseId", (q) => q.eq("offenseId", offense._id))
      .collect();

    return {
      ...offense,
      evidence,
    };
  },
});

export const listPaginated = query({
  args: {
    category: v.optional(v.string()),
    status: v.optional(v.string()),
    limit: v.optional(v.number()),
    offset: v.optional(v.number()),
  },
  handler: async (ctx, args) => {
    let offensesQuery = args.category
      ? ctx.db
          .query("artistOffenses")
          .withIndex("by_category", (q) => q.eq("category", args.category!))
      : args.status
        ? ctx.db
            .query("artistOffenses")
            .withIndex("by_status", (q) => q.eq("status", args.status!))
        : ctx.db.query("artistOffenses");

    const allOffenses = await offensesQuery.collect();
    const offset = args.offset ?? 0;
    const limit = args.limit ?? 20;
    const paginated = allOffenses.slice(offset, offset + limit);

    const withArtists = await Promise.all(
      paginated.map(async (offense) => {
        const artist = await ctx.db.get(offense.artistId);
        const evidence = await ctx.db
          .query("offenseEvidence")
          .withIndex("by_offenseId", (q) => q.eq("offenseId", offense._id))
          .collect();

        return {
          id: offense._id,
          artist_id: offense.artistId,
          artist_name: artist?.canonicalName ?? "Unknown",
          category: offense.category,
          severity: offense.severity,
          title: offense.title,
          description: offense.description,
          incident_date: offense.incidentDate,
          procedural_state: offense.proceduralState,
          status: offense.status,
          evidence_count: evidence.length,
          created_at: offense.createdAt,
          updated_at: offense.updatedAt,
        };
      }),
    );

    return {
      offenses: withArtists,
      total: allOffenses.length,
      offset,
      limit,
    };
  },
});

export const submit = mutation({
  args: {
    artistId: v.id("artists"),
    category: v.string(),
    severity: v.string(),
    title: v.string(),
    description: v.string(),
    incidentDate: v.optional(v.string()),
    proceduralState: v.optional(v.string()),
  },
  handler: async (ctx, args) => {
    const { user } = await requireCurrentUser(ctx);
    const offenseId = await ctx.db.insert("artistOffenses", {
      legacyKey: `runtime:offense:${args.artistId}:${Date.now()}`,
      artistId: args.artistId,
      category: args.category,
      severity: args.severity,
      title: args.title,
      description: args.description,
      incidentDate: args.incidentDate,
      incidentDateApproximate: false,
      status: "pending",
      proceduralState: args.proceduralState,
      arrested: false,
      charged: false,
      convicted: false,
      settled: false,
      verifiedAt: undefined,
      verifiedByUserId: undefined,
      submittedByUserId: user._id,
      metadata: {},
      createdAt: nowIso(),
      updatedAt: nowIso(),
    });

    return await ctx.db.get(offenseId);
  },
});

export const addEvidence = mutation({
  args: {
    offenseId: v.id("artistOffenses"),
    url: v.string(),
    sourceName: v.optional(v.string()),
    sourceType: v.optional(v.string()),
    title: v.optional(v.string()),
    excerpt: v.optional(v.string()),
    publishedDate: v.optional(v.string()),
    archivedUrl: v.optional(v.string()),
    credibilityScore: v.optional(v.number()),
  },
  handler: async (ctx, args) => {
    const { user } = await requireCurrentUser(ctx);
    const offense = await ctx.db.get(args.offenseId);
    if (!offense) {
      throw new ConvexError("Offense not found.");
    }

    const evidenceId = await ctx.db.insert("offenseEvidence", {
      legacyKey: `runtime:evidence:${args.offenseId}:${Date.now()}`,
      offenseId: args.offenseId,
      url: args.url,
      sourceName: args.sourceName,
      sourceType: args.sourceType,
      title: args.title,
      excerpt: args.excerpt,
      publishedDate: args.publishedDate,
      archivedUrl: args.archivedUrl,
      isPrimarySource: false,
      credibilityScore: args.credibilityScore,
      submittedByUserId: user._id,
      metadata: {},
      createdAt: nowIso(),
      updatedAt: nowIso(),
    });

    return await ctx.db.get(evidenceId);
  },
});

/**
 * One-time fix: correct malformed Wayback Machine URLs in evidence records.
 * Changes `web/YYYY/` to `web/YYYY* /` so archive.org resolves to the nearest snapshot.
 */
export const fixMalformedArchiveUrls = internalMutation({
  args: {},
  handler: async (ctx) => {
    const evidence = await ctx.db.query("offenseEvidence").take(2000);
    const archiveUrlPattern = /web\.archive\.org\/web\/(\d{4})\//;
    let fixed = 0;

    for (const record of evidence) {
      let patched = false;
      const patch: Record<string, string> = {};

      if (record.url && archiveUrlPattern.test(record.url)) {
        patch.url = record.url.replace(archiveUrlPattern, 'web.archive.org/web/$1*/');
        patched = true;
      }
      if (record.archivedUrl && archiveUrlPattern.test(record.archivedUrl)) {
        patch.archivedUrl = record.archivedUrl.replace(archiveUrlPattern, 'web.archive.org/web/$1*/');
        patched = true;
      }

      if (patched) {
        await ctx.db.patch(record._id, patch);
        fixed++;
      }
    }

    return { fixed, total: evidence.length };
  },
});

export const verifyOffense = mutation({
  args: {
    offenseId: v.id("artistOffenses"),
  },
  handler: async (ctx, args) => {
    const { user } = await requireCurrentUser(ctx);
    const offense = await ctx.db.get(args.offenseId);
    if (!offense) {
      throw new ConvexError("Offense not found.");
    }

    const now = nowIso();

    // Mark offense as verified
    await ctx.db.patch(offense._id, {
      status: "verified",
      verifiedAt: now,
      verifiedByUserId: user._id,
      updatedAt: now,
    });

    // Rebuild offending artist index for this artist
    await ctx.scheduler.runAfter(
      0,
      internal.offensePipeline.rebuildOffendingArtistIndex,
      {},
    );

    // Fan out to recompute affected users
    await ctx.scheduler.runAfter(
      100,
      internal.offensePipeline.recomputeAffectedUsers,
      { artistId: offense.artistId },
    );

    return await ctx.db.get(offense._id);
  },
});
