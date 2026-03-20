import { ConvexError, v } from "convex/values";
import { mutation, query } from "./_generated/server";
import { nowIso } from "./lib/auth";

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
    if (!IMPORTABLE_TABLES.has(args.table)) {
      throw new ConvexError(`Unsupported table ${args.table}.`);
    }
    return await (ctx.db.query(args.table as any) as any).count();
  },
});
