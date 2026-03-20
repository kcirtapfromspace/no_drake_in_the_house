import { ConvexError, v } from "convex/values";
import { mutation, query } from "./_generated/server";
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
