import { defineSchema, defineTable } from "convex/server";
import { v } from "convex/values";

const lifecycleFields = {
  createdAt: v.string(),
  updatedAt: v.string(),
};

const legacyFields = {
  legacyKey: v.string(),
};

const blob = v.optional(v.any());

export default defineSchema({
  users: defineTable({
    ...legacyFields,
    ...lifecycleFields,
    legacyUserId: v.optional(v.string()),
    authSubject: v.optional(v.string()),
    email: v.optional(v.string()),
    emailVerified: v.optional(v.boolean()),
    displayName: v.optional(v.string()),
    avatarUrl: v.optional(v.string()),
    settings: blob,
    roles: v.optional(v.array(v.string())),
    totpEnabled: v.optional(v.boolean()),
    lastLoginAt: v.optional(v.string()),
    metadata: blob,
  })
    .index("by_legacyKey", ["legacyKey"])
    .index("by_legacyUserId", ["legacyUserId"])
    .index("by_authSubject", ["authSubject"])
    .index("by_email", ["email"]),

  providerConnections: defineTable({
    ...legacyFields,
    ...lifecycleFields,
    userId: v.id("users"),
    provider: v.string(),
    providerUserId: v.optional(v.string()),
    scopes: v.optional(v.array(v.string())),
    encryptedAccessToken: v.optional(v.string()),
    encryptedRefreshToken: v.optional(v.string()),
    encryptedDataKey: v.optional(v.string()),
    dataKeyId: v.optional(v.string()),
    tokenVersion: v.optional(v.number()),
    expiresAt: v.optional(v.string()),
    status: v.string(),
    lastHealthCheckAt: v.optional(v.string()),
    errorCode: v.optional(v.string()),
    metadata: blob,
  })
    .index("by_legacyKey", ["legacyKey"])
    .index("by_userId", ["userId"])
    .index("by_user_provider", ["userId", "provider"]),

  artists: defineTable({
    ...legacyFields,
    ...lifecycleFields,
    legacyArtistId: v.optional(v.string()),
    canonicalName: v.string(),
    canonicalArtistId: v.optional(v.id("artists")),
    externalIds: blob,
    metadata: blob,
    aliases: v.optional(v.array(v.string())),
    status: v.optional(v.string()),
    lastInvestigatedAt: v.optional(v.string()),
    investigationStatus: v.optional(v.string()),
    researchQualityScore: v.optional(v.number()),
    sourcesSearched: v.optional(v.array(v.string())),
    researchIterations: v.optional(v.number()),
    catalogEnrichedAt: v.optional(v.string()),
  })
    .index("by_legacyKey", ["legacyKey"])
    .index("by_legacyArtistId", ["legacyArtistId"])
    .index("by_lastInvestigatedAt", ["lastInvestigatedAt"])
    .searchIndex("search_canonicalName", {
      searchField: "canonicalName",
    }),

  userArtistBlocks: defineTable({
    ...legacyFields,
    ...lifecycleFields,
    userId: v.id("users"),
    artistId: v.id("artists"),
    tags: v.optional(v.array(v.string())),
    note: v.optional(v.string()),
    source: v.optional(v.string()),
  })
    .index("by_legacyKey", ["legacyKey"])
    .index("by_userId", ["userId"])
    .index("by_artistId", ["artistId"])
    .index("by_user_artist", ["userId", "artistId"]),

  communityLists: defineTable({
    ...legacyFields,
    ...lifecycleFields,
    ownerUserId: v.optional(v.id("users")),
    name: v.string(),
    description: v.optional(v.string()),
    criteria: v.optional(v.string()),
    governanceUrl: v.optional(v.string()),
    updateCadence: v.optional(v.string()),
    version: v.optional(v.number()),
    visibility: v.optional(v.string()),
    metadata: blob,
  })
    .index("by_legacyKey", ["legacyKey"])
    .index("by_ownerUserId", ["ownerUserId"]),

  communityListItems: defineTable({
    ...legacyFields,
    ...lifecycleFields,
    listId: v.id("communityLists"),
    artistId: v.id("artists"),
    rationaleLink: v.optional(v.string()),
  })
    .index("by_legacyKey", ["legacyKey"])
    .index("by_listId", ["listId"])
    .index("by_artistId", ["artistId"])
    .index("by_list_artist", ["listId", "artistId"]),

  userListSubscriptions: defineTable({
    ...legacyFields,
    ...lifecycleFields,
    userId: v.id("users"),
    listId: v.id("communityLists"),
    versionPinned: v.optional(v.number()),
    autoUpdate: v.optional(v.boolean()),
  })
    .index("by_legacyKey", ["legacyKey"])
    .index("by_userId", ["userId"])
    .index("by_user_list", ["userId", "listId"]),

  categorySubscriptions: defineTable({
    ...legacyFields,
    ...lifecycleFields,
    userId: v.id("users"),
    category: v.string(),
  })
    .index("by_legacyKey", ["legacyKey"])
    .index("by_userId", ["userId"])
    .index("by_user_category", ["userId", "category"]),

  artistOffenses: defineTable({
    ...legacyFields,
    ...lifecycleFields,
    artistId: v.id("artists"),
    category: v.string(),
    severity: v.string(),
    title: v.string(),
    description: v.string(),
    incidentDate: v.optional(v.string()),
    incidentDateApproximate: v.optional(v.boolean()),
    status: v.optional(v.string()),
    proceduralState: v.optional(v.string()),
    arrested: v.optional(v.boolean()),
    charged: v.optional(v.boolean()),
    convicted: v.optional(v.boolean()),
    settled: v.optional(v.boolean()),
    verifiedAt: v.optional(v.string()),
    verifiedByUserId: v.optional(v.id("users")),
    submittedByUserId: v.optional(v.id("users")),
    confidence: v.optional(v.number()),
    sourceArticleUrl: v.optional(v.string()),
    metadata: blob,
  })
    .index("by_legacyKey", ["legacyKey"])
    .index("by_artistId", ["artistId"])
    .index("by_category", ["category"])
    .index("by_status", ["status"])
    .index("by_artistId_and_category", ["artistId", "category"]),

  offenseEvidence: defineTable({
    ...legacyFields,
    ...lifecycleFields,
    offenseId: v.id("artistOffenses"),
    url: v.string(),
    sourceName: v.optional(v.string()),
    sourceType: v.optional(v.string()),
    title: v.optional(v.string()),
    excerpt: v.optional(v.string()),
    publishedDate: v.optional(v.string()),
    archivedUrl: v.optional(v.string()),
    isPrimarySource: v.optional(v.boolean()),
    credibilityScore: v.optional(v.number()),
    submittedByUserId: v.optional(v.id("users")),
    metadata: blob,
  })
    .index("by_legacyKey", ["legacyKey"])
    .index("by_offenseId", ["offenseId"])
    .index("by_offenseId_and_url", ["offenseId", "url"]),

  userLibraryTracks: defineTable({
    ...legacyFields,
    ...lifecycleFields,
    userId: v.id("users"),
    provider: v.string(),
    providerTrackId: v.string(),
    trackName: v.optional(v.string()),
    albumName: v.optional(v.string()),
    artistId: v.optional(v.id("artists")),
    artistName: v.optional(v.string()),
    sourceType: v.optional(v.string()),
    playlistName: v.optional(v.string()),
    addedAt: v.optional(v.string()),
    lastSyncedAt: v.optional(v.string()),
    canonicalTrackId: v.optional(v.id("tracks")),
    metadata: blob,
  })
    .index("by_legacyKey", ["legacyKey"])
    .index("by_userId", ["userId"])
    .index("by_user_provider", ["userId", "provider"])
    .index("by_artistId", ["artistId"])
    .index("by_user_artistName", ["userId", "artistName"])
    .index("by_canonicalTrackId", ["canonicalTrackId"]),

  libraryScans: defineTable({
    ...legacyFields,
    ...lifecycleFields,
    userId: v.id("users"),
    provider: v.string(),
    totalTracks: v.number(),
    totalArtists: v.number(),
    flaggedArtists: v.number(),
    flaggedTracks: v.number(),
    severityCounts: blob,
    flaggedArtistsJson: v.optional(v.array(v.any())),
    scanStartedAt: v.string(),
    scanCompletedAt: v.optional(v.string()),
  })
    .index("by_legacyKey", ["legacyKey"])
    .index("by_userId", ["userId"])
    .index("by_user_provider", ["userId", "provider"]),

  actionBatches: defineTable({
    ...legacyFields,
    ...lifecycleFields,
    userId: v.id("users"),
    provider: v.string(),
    idempotencyKey: v.optional(v.string()),
    dryRun: v.optional(v.boolean()),
    status: v.string(),
    options: blob,
    summary: blob,
    completedAt: v.optional(v.string()),
  })
    .index("by_legacyKey", ["legacyKey"])
    .index("by_userId", ["userId"])
    .index("by_status", ["status"]),

  actionItems: defineTable({
    ...legacyFields,
    ...lifecycleFields,
    batchId: v.id("actionBatches"),
    entityType: v.string(),
    entityId: v.string(),
    action: v.string(),
    idempotencyKey: v.optional(v.string()),
    beforeState: blob,
    afterState: blob,
    status: v.string(),
    errorMessage: v.optional(v.string()),
  })
    .index("by_legacyKey", ["legacyKey"])
    .index("by_batchId", ["batchId"]),

  platformSyncRuns: defineTable({
    ...legacyFields,
    ...lifecycleFields,
    platform: v.string(),
    status: v.string(),
    startedAt: v.optional(v.string()),
    completedAt: v.optional(v.string()),
    errorLog: v.optional(v.array(v.any())),
    checkpointData: blob,
    metadata: blob,
  })
    .index("by_legacyKey", ["legacyKey"])
    .index("by_platform", ["platform"])
    .index("by_status", ["status"]),

  newsSources: defineTable({
    ...legacyFields,
    ...lifecycleFields,
    name: v.string(),
    sourceType: v.string(),
    url: v.string(),
    config: blob,
    credibilityScore: v.optional(v.number()),
    category: v.optional(v.string()),
    pollIntervalMinutes: v.optional(v.number()),
    isActive: v.optional(v.boolean()),
    lastPolledAt: v.optional(v.string()),
  })
    .index("by_legacyKey", ["legacyKey"])
    .index("by_sourceType", ["sourceType"]),

  newsArticles: defineTable({
    ...legacyFields,
    ...lifecycleFields,
    sourceId: v.optional(v.id("newsSources")),
    url: v.string(),
    title: v.string(),
    content: v.optional(v.string()),
    summary: v.optional(v.string()),
    publishedAt: v.optional(v.string()),
    processingStatus: v.optional(v.string()),
    embeddingGenerated: v.optional(v.boolean()),
    rawData: blob,
    metadata: blob,
  })
    .index("by_legacyKey", ["legacyKey"])
    .index("by_sourceId", ["sourceId"])
    .index("by_processingStatus", ["processingStatus"]),

  newsArticleEntities: defineTable({
    ...legacyFields,
    ...lifecycleFields,
    articleId: v.id("newsArticles"),
    artistId: v.optional(v.id("artists")),
    entityName: v.string(),
    entityType: v.string(),
    confidence: v.optional(v.number()),
    metadata: blob,
  })
    .index("by_legacyKey", ["legacyKey"])
    .index("by_articleId", ["articleId"])
    .index("by_artistId", ["artistId"]),

  newsOffenseClassifications: defineTable({
    ...legacyFields,
    ...lifecycleFields,
    articleId: v.id("newsArticles"),
    entityId: v.optional(v.id("newsArticleEntities")),
    artistId: v.optional(v.id("artists")),
    category: v.string(),
    severity: v.string(),
    confidence: v.optional(v.number()),
    humanVerified: v.optional(v.boolean()),
    verifiedByUserId: v.optional(v.id("users")),
    metadata: blob,
  })
    .index("by_legacyKey", ["legacyKey"])
    .index("by_articleId", ["articleId"])
    .index("by_artistId", ["artistId"])
    .index("by_category", ["category"]),

  socialMediaPosts: defineTable({
    ...legacyFields,
    ...lifecycleFields,
    platform: v.string(),
    externalId: v.string(),
    url: v.optional(v.string()),
    content: v.optional(v.string()),
    authorName: v.optional(v.string()),
    postedAt: v.optional(v.string()),
    engagementMetrics: blob,
    processingStatus: v.optional(v.string()),
    metadata: blob,
  })
    .index("by_legacyKey", ["legacyKey"])
    .index("by_platform_externalId", ["platform", "externalId"]),

  socialPostEntities: defineTable({
    ...legacyFields,
    ...lifecycleFields,
    postId: v.id("socialMediaPosts"),
    artistId: v.optional(v.id("artists")),
    entityName: v.string(),
    entityType: v.string(),
    confidence: v.optional(v.number()),
    metadata: blob,
  })
    .index("by_legacyKey", ["legacyKey"])
    .index("by_postId", ["postId"])
    .index("by_artistId", ["artistId"]),

  albums: defineTable({
    ...legacyFields,
    ...lifecycleFields,
    title: v.string(),
    normalizedTitle: v.optional(v.string()),
    releaseDate: v.optional(v.string()),
    appleMusicId: v.optional(v.string()),
    spotifyId: v.optional(v.string()),
    tidalId: v.optional(v.string()),
    deezerId: v.optional(v.string()),
    metadata: blob,
  })
    .index("by_legacyKey", ["legacyKey"])
    .index("by_normalizedTitle", ["normalizedTitle"])
    .index("by_spotifyId", ["spotifyId"])
    .index("by_appleMusicId", ["appleMusicId"]),

  albumArtists: defineTable({
    ...legacyFields,
    ...lifecycleFields,
    albumId: v.id("albums"),
    artistId: v.id("artists"),
  })
    .index("by_legacyKey", ["legacyKey"])
    .index("by_albumId", ["albumId"])
    .index("by_artistId", ["artistId"]),

  tracks: defineTable({
    ...legacyFields,
    ...lifecycleFields,
    title: v.string(),
    normalizedKey: v.optional(v.string()),
    albumId: v.optional(v.id("albums")),
    artistId: v.optional(v.id("artists")),
    appleMusicId: v.optional(v.string()),
    spotifyId: v.optional(v.string()),
    tidalId: v.optional(v.string()),
    deezerId: v.optional(v.string()),
    isrc: v.optional(v.string()),
    duration: v.optional(v.number()),
    trackNumber: v.optional(v.number()),
    metadata: blob,
  })
    .index("by_legacyKey", ["legacyKey"])
    .index("by_albumId", ["albumId"])
    .index("by_artistId", ["artistId"])
    .index("by_normalizedKey", ["normalizedKey"])
    .index("by_spotifyId", ["spotifyId"])
    .index("by_appleMusicId", ["appleMusicId"]),

  trackCredits: defineTable({
    ...legacyFields,
    ...lifecycleFields,
    trackId: v.id("tracks"),
    artistId: v.optional(v.id("artists")),
    creditedName: v.string(),
    role: v.string(),
    metadata: blob,
  })
    .index("by_legacyKey", ["legacyKey"])
    .index("by_trackId", ["trackId"])
    .index("by_artistId", ["artistId"]),

  artistCollaborations: defineTable({
    ...legacyFields,
    ...lifecycleFields,
    artistId1: v.id("artists"),
    artistId2: v.id("artists"),
    collaborationType: v.string(),
    collaborationCount: v.optional(v.number()),
    recentTracks: v.optional(v.array(v.string())),
    metadata: blob,
  })
    .index("by_legacyKey", ["legacyKey"])
    .index("by_artistId1", ["artistId1"])
    .index("by_artistId2", ["artistId2"]),

  userTrackBlocks: defineTable({
    ...legacyFields,
    ...lifecycleFields,
    userId: v.id("users"),
    artistId: v.id("artists"),
    trackId: v.optional(v.id("tracks")),
    reason: v.optional(v.string()),
    metadata: blob,
  })
    .index("by_legacyKey", ["legacyKey"])
    .index("by_userId", ["userId"])
    .index("by_artistId", ["artistId"])
    .index("by_trackId", ["trackId"]),

  enforcementRuns: defineTable({
    ...legacyFields,
    ...lifecycleFields,
    userId: v.id("users"),
    provider: v.string(),
    status: v.string(),
    options: blob,
    errorDetails: blob,
    startedAt: v.optional(v.string()),
    completedAt: v.optional(v.string()),
  })
    .index("by_legacyKey", ["legacyKey"])
    .index("by_userId", ["userId"])
    .index("by_status", ["status"]),

  enforcementActions: defineTable({
    ...legacyFields,
    ...lifecycleFields,
    runId: v.id("enforcementRuns"),
    userId: v.id("users"),
    resourceType: v.string(),
    resourceId: v.string(),
    status: v.string(),
    beforeState: blob,
    afterState: blob,
    metadata: blob,
  })
    .index("by_legacyKey", ["legacyKey"])
    .index("by_runId", ["runId"])
    .index("by_userId", ["userId"]),

  bloomFilterSnapshots: defineTable({
    ...legacyFields,
    ...lifecycleFields,
    scope: v.string(),
    version: v.number(),
    artistCount: v.number(),
    payload: v.any(),
    signature: v.optional(v.string()),
    generatedAt: v.string(),
  })
    .index("by_legacyKey", ["legacyKey"])
    .index("by_scope", ["scope"]),

  migrationMappings: defineTable({
    ...legacyFields,
    ...lifecycleFields,
    sourceTable: v.string(),
    convexTable: v.string(),
    convexId: v.string(),
    batchKey: v.optional(v.string()),
  })
    .index("by_legacyKey", ["legacyKey"])
    .index("by_sourceTable", ["sourceTable"])
    .index("by_sourceTable_legacyKey", ["sourceTable", "legacyKey"]),

  archivedDatasets: defineTable({
    ...legacyFields,
    ...lifecycleFields,
    dataset: v.string(),
    storageUri: v.string(),
    checksum: v.optional(v.string()),
    rowCount: v.optional(v.number()),
    metadata: blob,
  })
    .index("by_legacyKey", ["legacyKey"])
    .index("by_dataset", ["dataset"]),

  derivedSnapshots: defineTable({
    ...legacyFields,
    ...lifecycleFields,
    kind: v.string(),
    subjectKey: v.string(),
    payload: v.any(),
    computedAt: v.string(),
  })
    .index("by_legacyKey", ["legacyKey"])
    .index("by_kind_subjectKey", ["kind", "subjectKey"]),

  subscriptions: defineTable({
    ...legacyFields,
    ...lifecycleFields,
    userId: v.optional(v.id("users")),
    stripeCustomerId: v.string(),
    stripeSubscriptionId: v.string(),
    stripePriceId: v.string(),
    plan: v.union(
      v.literal("free"),
      v.literal("pro"),
      v.literal("team"),
    ),
    status: v.union(
      v.literal("active"),
      v.literal("past_due"),
      v.literal("canceled"),
      v.literal("trialing"),
      v.literal("incomplete"),
    ),
    currentPeriodStart: v.string(),
    currentPeriodEnd: v.string(),
    cancelAtPeriodEnd: v.optional(v.boolean()),
    seats: v.optional(v.number()),
    metadata: blob,
  })
    .index("by_legacyKey", ["legacyKey"])
    .index("by_userId", ["userId"])
    .index("by_stripeCustomerId", ["stripeCustomerId"])
    .index("by_stripeSubscriptionId", ["stripeSubscriptionId"]),

  offendingArtistIndex: defineTable({
    ...legacyFields,
    ...lifecycleFields,
    artistId: v.id("artists"),
    offenseCount: v.number(),
    highestSeverity: v.string(),
    severityTotal: v.number(),
    categories: v.array(v.string()),
  })
    .index("by_legacyKey", ["legacyKey"])
    .index("by_artistId", ["artistId"]),

  userOffenseSummaries: defineTable({
    ...legacyFields,
    ...lifecycleFields,
    userId: v.id("users"),
    totalTracks: v.number(),
    totalArtists: v.number(),
    flaggedArtistCount: v.number(),
    flaggedTrackCount: v.number(),
    offenderRatio: v.number(),
    grade: v.string(),
    offenders: v.array(v.any()),
    computedAt: v.string(),
    triggerReason: v.optional(v.string()),
  })
    .index("by_legacyKey", ["legacyKey"])
    .index("by_userId", ["userId"]),

  billingEvents: defineTable({
    ...legacyFields,
    ...lifecycleFields,
    userId: v.optional(v.id("users")),
    stripeEventId: v.string(),
    eventType: v.string(),
    payload: v.any(),
    processedAt: v.optional(v.string()),
  })
    .index("by_legacyKey", ["legacyKey"])
    .index("by_stripeEventId", ["stripeEventId"])
    .index("by_userId", ["userId"]),
});
