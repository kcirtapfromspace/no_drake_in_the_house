import { v } from "convex/values";
import { query } from "./_generated/server";

/**
 * Read-only queries for the news/research pipeline.
 * Data is ingested by the Rust backend via newsIngestion.ts mutations.
 */

export const listArticles = query({
  args: {
    status: v.optional(v.string()),
    sourceType: v.optional(v.string()),
    limit: v.optional(v.number()),
    cursor: v.optional(v.string()),
  },
  handler: async (ctx, args) => {
    const limit = args.limit ?? 20;

    // Filter by processing status if provided
    let articlesQuery = args.status
      ? ctx.db
          .query("newsArticles")
          .withIndex("by_processingStatus", (q) =>
            q.eq("processingStatus", args.status!),
          )
      : ctx.db.query("newsArticles");

    const allArticles = await articlesQuery.collect();

    // If sourceType is specified, filter by joining with newsSources
    let filtered = allArticles;
    if (args.sourceType) {
      const sourcesOfType = await ctx.db
        .query("newsSources")
        .withIndex("by_sourceType", (q) =>
          q.eq("sourceType", args.sourceType!),
        )
        .collect();
      const sourceIds = new Set(sourcesOfType.map((s) => s._id as string));
      filtered = allArticles.filter(
        (a) => a.sourceId && sourceIds.has(a.sourceId as string),
      );
    }

    // Simple cursor-based pagination using _id
    let startIdx = 0;
    if (args.cursor) {
      const cursorIdx = filtered.findIndex(
        (a) => (a._id as string) === args.cursor,
      );
      if (cursorIdx >= 0) {
        startIdx = cursorIdx + 1;
      }
    }

    const paginated = filtered.slice(startIdx, startIdx + limit);

    // Enrich with source info
    const articles = await Promise.all(
      paginated.map(async (article) => {
        const source = article.sourceId
          ? await ctx.db.get(article.sourceId)
          : null;
        return {
          id: article._id,
          legacyKey: article.legacyKey,
          url: article.url,
          title: article.title,
          summary: article.summary,
          publishedAt: article.publishedAt,
          processingStatus: article.processingStatus,
          embeddingGenerated: article.embeddingGenerated,
          source: source
            ? {
                id: source._id,
                name: source.name,
                sourceType: source.sourceType,
                url: source.url,
                credibilityScore: source.credibilityScore,
              }
            : null,
          createdAt: article.createdAt,
          updatedAt: article.updatedAt,
        };
      }),
    );

    const nextCursor =
      paginated.length === limit
        ? (paginated[paginated.length - 1]._id as string)
        : null;

    return {
      articles,
      total: filtered.length,
      nextCursor,
    };
  },
});

export const getArticle = query({
  args: {
    articleId: v.id("newsArticles"),
  },
  handler: async (ctx, args) => {
    const article = await ctx.db.get(args.articleId);
    if (!article) {
      return null;
    }

    const source = article.sourceId
      ? await ctx.db.get(article.sourceId)
      : null;

    const entities = await ctx.db
      .query("newsArticleEntities")
      .withIndex("by_articleId", (q) => q.eq("articleId", args.articleId))
      .collect();

    const classifications = await ctx.db
      .query("newsOffenseClassifications")
      .withIndex("by_articleId", (q) => q.eq("articleId", args.articleId))
      .collect();

    // Enrich entities with artist info
    const enrichedEntities = await Promise.all(
      entities.map(async (entity) => {
        const artist = entity.artistId
          ? await ctx.db.get(entity.artistId)
          : null;
        return {
          id: entity._id,
          entityName: entity.entityName,
          entityType: entity.entityType,
          confidence: entity.confidence,
          artistId: entity.artistId,
          artistName: artist?.canonicalName ?? null,
          metadata: entity.metadata,
        };
      }),
    );

    // Enrich classifications with artist info
    const enrichedClassifications = await Promise.all(
      classifications.map(async (cls) => {
        const artist = cls.artistId ? await ctx.db.get(cls.artistId) : null;
        return {
          id: cls._id,
          category: cls.category,
          severity: cls.severity,
          confidence: cls.confidence,
          humanVerified: cls.humanVerified,
          artistId: cls.artistId,
          artistName: artist?.canonicalName ?? null,
          entityId: cls.entityId,
          metadata: cls.metadata,
        };
      }),
    );

    return {
      id: article._id,
      legacyKey: article.legacyKey,
      url: article.url,
      title: article.title,
      content: article.content,
      summary: article.summary,
      publishedAt: article.publishedAt,
      processingStatus: article.processingStatus,
      embeddingGenerated: article.embeddingGenerated,
      rawData: article.rawData,
      metadata: article.metadata,
      source: source
        ? {
            id: source._id,
            name: source.name,
            sourceType: source.sourceType,
            url: source.url,
            credibilityScore: source.credibilityScore,
          }
        : null,
      entities: enrichedEntities,
      classifications: enrichedClassifications,
      createdAt: article.createdAt,
      updatedAt: article.updatedAt,
    };
  },
});

export const listArticlesByArtist = query({
  args: {
    artistId: v.id("artists"),
    limit: v.optional(v.number()),
  },
  handler: async (ctx, args) => {
    const limit = args.limit ?? 20;

    // Find all entity links for this artist
    const entityLinks = await ctx.db
      .query("newsArticleEntities")
      .withIndex("by_artistId", (q) => q.eq("artistId", args.artistId))
      .collect();

    // Deduplicate article IDs
    const articleIds = [
      ...new Set(entityLinks.map((e) => e.articleId as string)),
    ];

    // Fetch articles (up to limit)
    const articles = [];
    for (const articleId of articleIds.slice(0, limit)) {
      const article = await ctx.db.get(articleId as any);
      if (article) {
        const source = article.sourceId
          ? await ctx.db.get(article.sourceId)
          : null;

        // Get the entity context for this article-artist pair
        const entityForArticle = entityLinks.find(
          (e) => (e.articleId as string) === articleId,
        );

        articles.push({
          id: article._id,
          url: article.url,
          title: article.title,
          summary: article.summary,
          publishedAt: article.publishedAt,
          processingStatus: article.processingStatus,
          source: source
            ? {
                id: source._id,
                name: source.name,
                sourceType: source.sourceType,
              }
            : null,
          entityContext: entityForArticle
            ? {
                entityName: entityForArticle.entityName,
                entityType: entityForArticle.entityType,
                confidence: entityForArticle.confidence,
              }
            : null,
          createdAt: article.createdAt,
        });
      }
    }

    return {
      articles,
      artistId: args.artistId,
      total: articleIds.length,
    };
  },
});

export const recentClassifications = query({
  args: {
    limit: v.optional(v.number()),
    category: v.optional(v.string()),
  },
  handler: async (ctx, args) => {
    const limit = args.limit ?? 30;

    let classificationsQuery = args.category
      ? ctx.db
          .query("newsOffenseClassifications")
          .withIndex("by_category", (q) => q.eq("category", args.category!))
      : ctx.db.query("newsOffenseClassifications");

    const allClassifications = await classificationsQuery.collect();

    // Sort by creation time descending (most recent first)
    allClassifications.sort((a, b) => b.createdAt.localeCompare(a.createdAt));
    const recent = allClassifications.slice(0, limit);

    // Enrich with article and artist info
    const enriched = await Promise.all(
      recent.map(async (cls) => {
        const article = await ctx.db.get(cls.articleId);
        const artist = cls.artistId ? await ctx.db.get(cls.artistId) : null;

        return {
          id: cls._id,
          category: cls.category,
          severity: cls.severity,
          confidence: cls.confidence,
          humanVerified: cls.humanVerified,
          article: article
            ? {
                id: article._id,
                title: article.title,
                url: article.url,
                publishedAt: article.publishedAt,
              }
            : null,
          artist: artist
            ? {
                id: artist._id,
                canonicalName: artist.canonicalName,
              }
            : null,
          createdAt: cls.createdAt,
          updatedAt: cls.updatedAt,
        };
      }),
    );

    return {
      classifications: enriched,
      total: allClassifications.length,
    };
  },
});

export const pipelineStats = query({
  args: {},
  handler: async (ctx) => {
    // Retrieve sync runs marked as news pipeline
    const newsRuns = await ctx.db
      .query("platformSyncRuns")
      .withIndex("by_platform", (q) => q.eq("platform", "news"))
      .collect();

    const totalRuns = newsRuns.length;
    const statusCounts: Record<string, number> = {};
    let lastRunAt: string | null = null;
    let lastRunStatus: string | null = null;

    for (const run of newsRuns) {
      statusCounts[run.status] = (statusCounts[run.status] ?? 0) + 1;
      const runTime = run.completedAt ?? run.startedAt ?? run.createdAt;
      if (!lastRunAt || (runTime && runTime > lastRunAt)) {
        lastRunAt = runTime;
        lastRunStatus = run.status;
      }
    }

    // Count total articles by status
    const allArticles = await ctx.db.query("newsArticles").collect();
    const articleStatusCounts: Record<string, number> = {};
    for (const article of allArticles) {
      const status = article.processingStatus ?? "unknown";
      articleStatusCounts[status] = (articleStatusCounts[status] ?? 0) + 1;
    }

    const totalClassifications = await ctx.db
      .query("newsOffenseClassifications")
      .collect();

    return {
      pipeline: {
        totalRuns,
        statusCounts,
        lastRunAt,
        lastRunStatus,
      },
      articles: {
        total: allArticles.length,
        byStatus: articleStatusCounts,
      },
      classifications: {
        total: totalClassifications.length,
      },
    };
  },
});
