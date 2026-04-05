import { ConvexError, v } from "convex/values";
import { mutation, query } from "./_generated/server";
import { nowIso, requireOwner } from "./lib/auth";

const IMPORTABLE_TABLES = new Set([
  "users",
  "providerConnections",
  "artists",
  "userArtistBlocks",
  "communityLists",
  "communityListItems",
  "userListSubscriptions",
  "categorySubscriptions",
  "artistOffenses",
  "offenseEvidence",
  "userLibraryTracks",
  "libraryScans",
  "actionBatches",
  "actionItems",
  "platformSyncRuns",
  "newsSources",
  "newsArticles",
  "newsArticleEntities",
  "newsOffenseClassifications",
  "socialMediaPosts",
  "socialPostEntities",
  "albums",
  "albumArtists",
  "tracks",
  "trackCredits",
  "artistCollaborations",
  "userTrackBlocks",
  "enforcementRuns",
  "enforcementActions",
  "archivedDatasets",
  "derivedSnapshots",
]);

function assertMigrationKey(candidate: string) {
  const expected = process.env.MIGRATION_API_KEY;
  if (!expected) {
    throw new ConvexError(
      "MIGRATION_API_KEY is not configured for Convex migrations.",
    );
  }
  if (candidate !== expected) {
    throw new ConvexError("Invalid migration API key.");
  }
}

export const importBatch = mutation({
  args: {
    apiKey: v.string(),
    table: v.string(),
    batchKey: v.optional(v.string()),
    rows: v.array(v.any()),
  },
  handler: async (ctx, args) => {
    assertMigrationKey(args.apiKey);

    if (!IMPORTABLE_TABLES.has(args.table)) {
      throw new ConvexError(`Unsupported import table ${args.table}.`);
    }

    let inserted = 0;
    let updated = 0;
    const mappings: Array<{ legacyKey: string; convexId: string }> = [];

    for (const row of args.rows) {
      if (!row || typeof row !== "object" || typeof row.legacyKey !== "string") {
        throw new ConvexError("Each imported row must include a string legacyKey.");
      }

      const existing = await (ctx.db.query(args.table as any) as any)
        .withIndex("by_legacyKey", (q: any) => q.eq("legacyKey", row.legacyKey))
        .unique();

      const normalizedRow = {
        ...row,
        updatedAt: typeof row.updatedAt === "string" ? row.updatedAt : nowIso(),
        createdAt: typeof row.createdAt === "string" ? row.createdAt : nowIso(),
      };

      let convexId: string;

      if (existing) {
        await ctx.db.patch(existing._id, normalizedRow);
        convexId = existing._id;
        updated += 1;
      } else {
        convexId = await ctx.db.insert(args.table as any, normalizedRow);
        inserted += 1;
      }

      const mapping = await ctx.db
        .query("migrationMappings")
        .withIndex("by_sourceTable", (q) => q.eq("sourceTable", args.table))
        .collect()
        .then((entries) =>
          entries.find((entry) => entry.legacyKey === `${args.table}:${row.legacyKey}`) ??
          null,
        );

      if (mapping) {
        await ctx.db.patch(mapping._id, {
          convexTable: args.table,
          convexId,
          batchKey: args.batchKey,
          updatedAt: nowIso(),
        });
      } else {
        await ctx.db.insert("migrationMappings", {
          legacyKey: `${args.table}:${row.legacyKey}`,
          sourceTable: args.table,
          convexTable: args.table,
          convexId,
          batchKey: args.batchKey,
          createdAt: nowIso(),
          updatedAt: nowIso(),
        });
      }

      mappings.push({
        legacyKey: row.legacyKey,
        convexId,
      });
    }

    return {
      table: args.table,
      inserted,
      updated,
      total: inserted + updated,
      mappings,
    };
  },
});

export const tableCounts = query({
  args: {
    table: v.string(),
  },
  handler: async (ctx, args) => {
    await requireOwner(ctx);
    if (!IMPORTABLE_TABLES.has(args.table)) {
      throw new ConvexError(`Unsupported table ${args.table}.`);
    }
    return await (ctx.db.query(args.table as any) as any).count();
  },
});

/**
 * Lightweight pipeline health check — uses .take(1) probes instead of
 * .collect() to avoid transferring full tables on the free plan.
 * Returns whether each table has data and a sample of recent sync runs.
 */
export const pipelineHealth = query({
  args: {},
  handler: async (ctx) => {
    await requireOwner(ctx);

    // Probe each table: take(1) to check if non-empty without full scan
    const [artistSample, offenseSample, evidenceSample, trackSample, connSample, indexSample] =
      await Promise.all([
        ctx.db.query("artists").take(5),
        ctx.db.query("artistOffenses").take(5),
        ctx.db.query("offenseEvidence").take(5),
        ctx.db.query("userLibraryTracks").take(5),
        ctx.db.query("providerConnections").take(5),
        ctx.db.query("offendingArtistIndex").take(5),
      ]);

    // Get 5 most recent sync runs (small table, safe to scan)
    const recentRuns = await ctx.db
      .query("platformSyncRuns")
      .order("desc")
      .take(10);

    return {
      tables: {
        artists: { hasData: artistSample.length > 0, sample: artistSample.map((a) => a.canonicalName) },
        artistOffenses: { hasData: offenseSample.length > 0, sampleCount: offenseSample.length },
        offenseEvidence: { hasData: evidenceSample.length > 0, sampleCount: evidenceSample.length },
        userLibraryTracks: { hasData: trackSample.length > 0, sampleCount: trackSample.length },
        providerConnections: {
          hasData: connSample.length > 0,
          providers: connSample.map((c) => ({ provider: c.provider, status: c.status })),
        },
        offendingArtistIndex: { hasData: indexSample.length > 0, sampleCount: indexSample.length },
      },
      recentRuns: recentRuns.map((r) => ({
        platform: r.platform,
        status: r.status,
        startedAt: r.startedAt,
        completedAt: r.completedAt,
      })),
    };
  },
});
