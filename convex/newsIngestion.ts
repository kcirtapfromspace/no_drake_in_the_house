import { ConvexError, v } from "convex/values";
import { mutation } from "./_generated/server";
import { nowIso } from "./lib/auth";

/**
 * Ingestion mutations called by the Rust backend to sync processed
 * news articles, entities, and classifications into Convex.
 */

export const ingestArticle = mutation({
  args: {
    legacyKey: v.string(),
    url: v.string(),
    title: v.string(),
    content: v.optional(v.string()),
    summary: v.optional(v.string()),
    publishedAt: v.optional(v.string()),
    processingStatus: v.optional(v.string()),
    embeddingGenerated: v.optional(v.boolean()),
    sourceId: v.optional(v.id("newsSources")),
    rawData: v.optional(v.any()),
    metadata: v.optional(v.any()),
  },
  handler: async (ctx, args) => {
    const now = nowIso();

    // Dedup by legacyKey
    const existing = await ctx.db
      .query("newsArticles")
      .withIndex("by_legacyKey", (q) => q.eq("legacyKey", args.legacyKey))
      .unique();

    if (existing) {
      // Update existing article
      await ctx.db.patch(existing._id, {
        url: args.url,
        title: args.title,
        content: args.content,
        summary: args.summary,
        publishedAt: args.publishedAt,
        processingStatus: args.processingStatus,
        embeddingGenerated: args.embeddingGenerated,
        sourceId: args.sourceId,
        rawData: args.rawData ?? existing.rawData,
        metadata: args.metadata ?? existing.metadata,
        updatedAt: now,
      });
      return { id: existing._id, upserted: "updated" as const };
    }

    // Insert new article
    const id = await ctx.db.insert("newsArticles", {
      legacyKey: args.legacyKey,
      url: args.url,
      title: args.title,
      content: args.content,
      summary: args.summary,
      publishedAt: args.publishedAt,
      processingStatus: args.processingStatus ?? "ingested",
      embeddingGenerated: args.embeddingGenerated ?? false,
      sourceId: args.sourceId,
      rawData: args.rawData ?? {},
      metadata: args.metadata ?? {},
      createdAt: now,
      updatedAt: now,
    });

    return { id, upserted: "created" as const };
  },
});

export const ingestEntities = mutation({
  args: {
    articleId: v.id("newsArticles"),
    entities: v.array(
      v.object({
        legacyKey: v.string(),
        entityName: v.string(),
        entityType: v.string(),
        artistId: v.optional(v.id("artists")),
        confidence: v.optional(v.number()),
        metadata: v.optional(v.any()),
      }),
    ),
  },
  handler: async (ctx, args) => {
    const article = await ctx.db.get(args.articleId);
    if (!article) {
      throw new ConvexError("Article not found.");
    }

    const now = nowIso();
    let inserted = 0;
    let updated = 0;

    for (const entity of args.entities) {
      const existing = await ctx.db
        .query("newsArticleEntities")
        .withIndex("by_legacyKey", (q) => q.eq("legacyKey", entity.legacyKey))
        .unique();

      if (existing) {
        await ctx.db.patch(existing._id, {
          entityName: entity.entityName,
          entityType: entity.entityType,
          artistId: entity.artistId,
          confidence: entity.confidence,
          metadata: entity.metadata ?? existing.metadata,
          updatedAt: now,
        });
        updated++;
      } else {
        await ctx.db.insert("newsArticleEntities", {
          legacyKey: entity.legacyKey,
          articleId: args.articleId,
          entityName: entity.entityName,
          entityType: entity.entityType,
          artistId: entity.artistId,
          confidence: entity.confidence,
          metadata: entity.metadata ?? {},
          createdAt: now,
          updatedAt: now,
        });
        inserted++;
      }
    }

    return { articleId: args.articleId, inserted, updated, total: args.entities.length };
  },
});

export const ingestClassification = mutation({
  args: {
    legacyKey: v.string(),
    articleId: v.id("newsArticles"),
    entityId: v.optional(v.id("newsArticleEntities")),
    artistId: v.optional(v.id("artists")),
    category: v.string(),
    severity: v.string(),
    confidence: v.optional(v.number()),
    humanVerified: v.optional(v.boolean()),
    verifiedByUserId: v.optional(v.id("users")),
    metadata: v.optional(v.any()),
  },
  handler: async (ctx, args) => {
    const article = await ctx.db.get(args.articleId);
    if (!article) {
      throw new ConvexError("Article not found.");
    }

    const now = nowIso();

    // Dedup by legacyKey
    const existing = await ctx.db
      .query("newsOffenseClassifications")
      .withIndex("by_legacyKey", (q) => q.eq("legacyKey", args.legacyKey))
      .unique();

    if (existing) {
      await ctx.db.patch(existing._id, {
        articleId: args.articleId,
        entityId: args.entityId,
        artistId: args.artistId,
        category: args.category,
        severity: args.severity,
        confidence: args.confidence,
        humanVerified: args.humanVerified,
        verifiedByUserId: args.verifiedByUserId,
        metadata: args.metadata ?? existing.metadata,
        updatedAt: now,
      });
      return { id: existing._id, upserted: "updated" as const };
    }

    const id = await ctx.db.insert("newsOffenseClassifications", {
      legacyKey: args.legacyKey,
      articleId: args.articleId,
      entityId: args.entityId,
      artistId: args.artistId,
      category: args.category,
      severity: args.severity,
      confidence: args.confidence,
      humanVerified: args.humanVerified ?? false,
      verifiedByUserId: args.verifiedByUserId,
      metadata: args.metadata ?? {},
      createdAt: now,
      updatedAt: now,
    });

    return { id, upserted: "created" as const };
  },
});

export const updateArticleStatus = mutation({
  args: {
    articleId: v.id("newsArticles"),
    processingStatus: v.string(),
    metadata: v.optional(v.any()),
  },
  handler: async (ctx, args) => {
    const article = await ctx.db.get(args.articleId);
    if (!article) {
      throw new ConvexError("Article not found.");
    }

    const now = nowIso();
    const patch: Record<string, any> = {
      processingStatus: args.processingStatus,
      updatedAt: now,
    };

    if (args.metadata !== undefined) {
      patch.metadata = args.metadata;
    }

    await ctx.db.patch(args.articleId, patch);

    return {
      id: args.articleId,
      processingStatus: args.processingStatus,
      updatedAt: now,
    };
  },
});

export const batchIngestArticles = mutation({
  args: {
    articles: v.array(
      v.object({
        legacyKey: v.string(),
        url: v.string(),
        title: v.string(),
        content: v.optional(v.string()),
        summary: v.optional(v.string()),
        publishedAt: v.optional(v.string()),
        processingStatus: v.optional(v.string()),
        embeddingGenerated: v.optional(v.boolean()),
        sourceId: v.optional(v.id("newsSources")),
        rawData: v.optional(v.any()),
        metadata: v.optional(v.any()),
        entities: v.optional(
          v.array(
            v.object({
              legacyKey: v.string(),
              entityName: v.string(),
              entityType: v.string(),
              artistId: v.optional(v.id("artists")),
              confidence: v.optional(v.number()),
              metadata: v.optional(v.any()),
            }),
          ),
        ),
        classifications: v.optional(
          v.array(
            v.object({
              legacyKey: v.string(),
              entityId: v.optional(v.id("newsArticleEntities")),
              artistId: v.optional(v.id("artists")),
              category: v.string(),
              severity: v.string(),
              confidence: v.optional(v.number()),
              humanVerified: v.optional(v.boolean()),
              metadata: v.optional(v.any()),
            }),
          ),
        ),
      }),
    ),
  },
  handler: async (ctx, args) => {
    const now = nowIso();
    let articlesCreated = 0;
    let articlesUpdated = 0;
    let entitiesInserted = 0;
    let classificationsInserted = 0;

    for (const articleData of args.articles) {
      // Upsert article
      const existing = await ctx.db
        .query("newsArticles")
        .withIndex("by_legacyKey", (q) =>
          q.eq("legacyKey", articleData.legacyKey),
        )
        .unique();

      let articleId;

      if (existing) {
        await ctx.db.patch(existing._id, {
          url: articleData.url,
          title: articleData.title,
          content: articleData.content,
          summary: articleData.summary,
          publishedAt: articleData.publishedAt,
          processingStatus: articleData.processingStatus,
          embeddingGenerated: articleData.embeddingGenerated,
          sourceId: articleData.sourceId,
          rawData: articleData.rawData ?? existing.rawData,
          metadata: articleData.metadata ?? existing.metadata,
          updatedAt: now,
        });
        articleId = existing._id;
        articlesUpdated++;
      } else {
        articleId = await ctx.db.insert("newsArticles", {
          legacyKey: articleData.legacyKey,
          url: articleData.url,
          title: articleData.title,
          content: articleData.content,
          summary: articleData.summary,
          publishedAt: articleData.publishedAt,
          processingStatus: articleData.processingStatus ?? "ingested",
          embeddingGenerated: articleData.embeddingGenerated ?? false,
          sourceId: articleData.sourceId,
          rawData: articleData.rawData ?? {},
          metadata: articleData.metadata ?? {},
          createdAt: now,
          updatedAt: now,
        });
        articlesCreated++;
      }

      // Insert entities
      if (articleData.entities) {
        for (const entity of articleData.entities) {
          const existingEntity = await ctx.db
            .query("newsArticleEntities")
            .withIndex("by_legacyKey", (q) =>
              q.eq("legacyKey", entity.legacyKey),
            )
            .unique();

          if (existingEntity) {
            await ctx.db.patch(existingEntity._id, {
              entityName: entity.entityName,
              entityType: entity.entityType,
              artistId: entity.artistId,
              confidence: entity.confidence,
              metadata: entity.metadata ?? existingEntity.metadata,
              updatedAt: now,
            });
          } else {
            await ctx.db.insert("newsArticleEntities", {
              legacyKey: entity.legacyKey,
              articleId,
              entityName: entity.entityName,
              entityType: entity.entityType,
              artistId: entity.artistId,
              confidence: entity.confidence,
              metadata: entity.metadata ?? {},
              createdAt: now,
              updatedAt: now,
            });
            entitiesInserted++;
          }
        }
      }

      // Insert classifications
      if (articleData.classifications) {
        for (const cls of articleData.classifications) {
          const existingCls = await ctx.db
            .query("newsOffenseClassifications")
            .withIndex("by_legacyKey", (q) =>
              q.eq("legacyKey", cls.legacyKey),
            )
            .unique();

          if (existingCls) {
            await ctx.db.patch(existingCls._id, {
              articleId,
              entityId: cls.entityId,
              artistId: cls.artistId,
              category: cls.category,
              severity: cls.severity,
              confidence: cls.confidence,
              humanVerified: cls.humanVerified,
              metadata: cls.metadata ?? existingCls.metadata,
              updatedAt: now,
            });
          } else {
            await ctx.db.insert("newsOffenseClassifications", {
              legacyKey: cls.legacyKey,
              articleId,
              entityId: cls.entityId,
              artistId: cls.artistId,
              category: cls.category,
              severity: cls.severity,
              confidence: cls.confidence,
              humanVerified: cls.humanVerified ?? false,
              metadata: cls.metadata ?? {},
              createdAt: now,
              updatedAt: now,
            });
            classificationsInserted++;
          }
        }
      }
    }

    return {
      articlesCreated,
      articlesUpdated,
      entitiesInserted,
      classificationsInserted,
      totalArticles: args.articles.length,
    };
  },
});
