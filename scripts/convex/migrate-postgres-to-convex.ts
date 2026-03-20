import type { ConvexHttpClient } from "convex/browser";
import type { Client } from "pg";
import {
  buildLifecycle,
  chunk,
  createConvexClient,
  createPostgresClient,
  ensureLegacyKey,
  MappingStore,
  requireEnv,
  sanitizeJson,
  tableExists,
  toIsoString,
} from "./shared";

type LegacyRow = Record<string, unknown>;
type ImportRow = Record<string, unknown> & { legacyKey: string };

interface TableDefinition {
  sourceTable: string;
  convexTable: string;
  transform: (row: LegacyRow, mappings: MappingStore) => ImportRow;
}

const BATCH_SIZE = Number(process.env.CONVEX_IMPORT_BATCH_SIZE ?? "100");
const migrationKey = requireEnv("MIGRATION_API_KEY");

const definitions: TableDefinition[] = [
  {
    sourceTable: "users",
    convexTable: "users",
    transform: (row) => ({
      legacyKey: ensureLegacyKey(row, [row.id]),
      legacyUserId: row.id,
      authSubject: undefined,
      email: typeof row.email === "string" ? row.email.toLowerCase() : undefined,
      emailVerified: Boolean(row.email_verified),
      displayName:
        typeof row.email === "string" ? row.email.split("@")[0] : undefined,
      avatarUrl: undefined,
      settings: sanitizeJson(row.settings) ?? {},
      roles: [],
      totpEnabled: Boolean(row.totp_enabled),
      lastLoginAt: row.last_login ? toIsoString(row.last_login) : undefined,
      metadata: {
        password_hash_present: Boolean(row.password_hash),
      },
      ...buildLifecycle(row),
    }),
  },
  {
    sourceTable: "artists",
    convexTable: "artists",
    transform: (row, mappings) => ({
      legacyKey: ensureLegacyKey(row, [row.id]),
      legacyArtistId: row.id,
      canonicalName: row.canonical_name,
      canonicalArtistId: mappings.get("artists", row.canonical_artist_id),
      externalIds: sanitizeJson(row.external_ids) ?? {},
      metadata: sanitizeJson(row.metadata) ?? {},
      aliases: Object.values((sanitizeJson(row.aliases) as Record<string, string>) ?? {}),
      status: "imported",
      ...buildLifecycle(row),
    }),
  },
  {
    sourceTable: "connections",
    convexTable: "providerConnections",
    transform: (row, mappings) => ({
      legacyKey: ensureLegacyKey(row, [row.id]),
      userId: mappings.require("users", row.user_id),
      provider: row.provider,
      providerUserId: row.provider_user_id,
      scopes: Array.isArray(row.scopes) ? row.scopes : [],
      encryptedAccessToken: row.access_token_encrypted,
      encryptedRefreshToken: row.refresh_token_encrypted,
      encryptedDataKey: undefined,
      dataKeyId: row.data_key_id,
      tokenVersion: Number(row.token_version ?? 1),
      expiresAt: row.expires_at ? toIsoString(row.expires_at) : undefined,
      status: row.status ?? "active",
      lastHealthCheckAt: row.last_health_check
        ? toIsoString(row.last_health_check)
        : undefined,
      errorCode: row.error_code,
      metadata: {},
      ...buildLifecycle(row),
    }),
  },
  {
    sourceTable: "community_lists",
    convexTable: "communityLists",
    transform: (row, mappings) => ({
      legacyKey: ensureLegacyKey(row, [row.id]),
      ownerUserId: mappings.get("users", row.owner_user_id),
      name: row.name,
      description: row.description,
      criteria: row.criteria,
      governanceUrl: row.governance_url,
      updateCadence: row.update_cadence,
      version: Number(row.version ?? 1),
      visibility: row.visibility ?? "public",
      metadata: {},
      ...buildLifecycle(row),
    }),
  },
  {
    sourceTable: "community_list_items",
    convexTable: "communityListItems",
    transform: (row, mappings) => ({
      legacyKey: ensureLegacyKey(row, [row.list_id, row.artist_id]),
      listId: mappings.require("communityLists", row.list_id),
      artistId: mappings.require("artists", row.artist_id),
      rationaleLink: row.rationale_link,
      ...buildLifecycle(row),
    }),
  },
  {
    sourceTable: "user_list_subscriptions",
    convexTable: "userListSubscriptions",
    transform: (row, mappings) => ({
      legacyKey: ensureLegacyKey(row, [row.user_id, row.list_id]),
      userId: mappings.require("users", row.user_id),
      listId: mappings.require("communityLists", row.list_id),
      versionPinned: row.version_pinned ? Number(row.version_pinned) : undefined,
      autoUpdate: row.auto_update ?? true,
      ...buildLifecycle(row),
    }),
  },
  {
    sourceTable: "category_subscriptions",
    convexTable: "categorySubscriptions",
    transform: (row, mappings) => ({
      legacyKey: ensureLegacyKey(row, [row.user_id, row.category]),
      userId: mappings.require("users", row.user_id),
      category: row.category,
      ...buildLifecycle(row),
    }),
  },
  {
    sourceTable: "user_artist_blocks",
    convexTable: "userArtistBlocks",
    transform: (row, mappings) => ({
      legacyKey: ensureLegacyKey(row, [row.user_id, row.artist_id]),
      userId: mappings.require("users", row.user_id),
      artistId: mappings.require("artists", row.artist_id),
      tags: Array.isArray(row.tags) ? row.tags : [],
      note: row.note,
      source: "postgres",
      ...buildLifecycle(row),
    }),
  },
  {
    sourceTable: "artist_offenses",
    convexTable: "artistOffenses",
    transform: (row, mappings) => ({
      legacyKey: ensureLegacyKey(row, [row.id]),
      artistId: mappings.require("artists", row.artist_id),
      category: row.category,
      severity: row.severity ?? "moderate",
      title: row.title,
      description: row.description,
      incidentDate: row.incident_date ? toIsoString(row.incident_date) : undefined,
      incidentDateApproximate: Boolean(row.incident_date_approximate),
      status: row.status ?? "pending",
      proceduralState: row.procedural_state,
      arrested: Boolean(row.arrested),
      charged: Boolean(row.charged),
      convicted: Boolean(row.convicted),
      settled: Boolean(row.settled),
      verifiedAt: row.verified_at ? toIsoString(row.verified_at) : undefined,
      verifiedByUserId: mappings.get("users", row.verified_by),
      submittedByUserId: mappings.get("users", row.submitted_by),
      metadata: {},
      ...buildLifecycle(row),
    }),
  },
  {
    sourceTable: "offense_evidence",
    convexTable: "offenseEvidence",
    transform: (row, mappings) => ({
      legacyKey: ensureLegacyKey(row, [row.id]),
      offenseId: mappings.require("artistOffenses", row.offense_id),
      url: row.url,
      sourceName: row.source_name,
      sourceType: row.source_type,
      title: row.title,
      excerpt: row.excerpt,
      publishedDate: row.published_date ? toIsoString(row.published_date) : undefined,
      archivedUrl: row.archived_url,
      isPrimarySource: Boolean(row.is_primary_source),
      credibilityScore: row.credibility_score
        ? Number(row.credibility_score)
        : undefined,
      submittedByUserId: mappings.get("users", row.submitted_by),
      metadata: {},
      ...buildLifecycle(row),
    }),
  },
  {
    sourceTable: "user_library_tracks",
    convexTable: "userLibraryTracks",
    transform: (row, mappings) => ({
      legacyKey: ensureLegacyKey(row, [row.id, row.user_id, row.provider_track_id]),
      userId: mappings.require("users", row.user_id),
      provider: row.provider,
      providerTrackId: row.provider_track_id,
      trackName: row.track_name,
      albumName: row.album_name,
      artistId: mappings.get("artists", row.artist_id),
      artistName: row.artist_name,
      sourceType: row.source_type,
      playlistName: row.playlist_name,
      addedAt: row.added_at ? toIsoString(row.added_at) : undefined,
      lastSyncedAt: row.last_synced ? toIsoString(row.last_synced) : undefined,
      metadata: {},
      ...buildLifecycle(row),
    }),
  },
  {
    sourceTable: "library_scan_results",
    convexTable: "libraryScans",
    transform: (row, mappings) => ({
      legacyKey: ensureLegacyKey(row, [row.id, row.user_id, row.provider]),
      userId: mappings.require("users", row.user_id),
      provider: row.provider,
      totalTracks: Number(row.total_tracks ?? 0),
      totalArtists: Number(row.total_artists ?? 0),
      flaggedArtists: Number(row.flagged_artists ?? 0),
      flaggedTracks: Number(row.flagged_tracks ?? 0),
      severityCounts: {
        egregious: Number(row.egregious_count ?? 0),
        severe: Number(row.severe_count ?? 0),
        moderate: Number(row.moderate_count ?? 0),
        minor: Number(row.minor_count ?? 0),
      },
      flaggedArtistsJson: Array.isArray(row.flagged_artists_json)
        ? sanitizeJson(row.flagged_artists_json)
        : [],
      scanStartedAt: toIsoString(row.scan_started_at),
      scanCompletedAt: row.scan_completed_at
        ? toIsoString(row.scan_completed_at)
        : undefined,
      ...buildLifecycle(row),
    }),
  },
  {
    sourceTable: "albums",
    convexTable: "albums",
    transform: (row) => ({
      legacyKey: ensureLegacyKey(row, [row.id]),
      title: row.title ?? row.name,
      releaseDate: row.release_date ? toIsoString(row.release_date) : undefined,
      appleMusicId: row.apple_music_id,
      spotifyId: row.spotify_id,
      deezerId: row.deezer_id,
      metadata: sanitizeJson(row.metadata) ?? {},
      ...buildLifecycle(row),
    }),
  },
  {
    sourceTable: "album_artists",
    convexTable: "albumArtists",
    transform: (row, mappings) => ({
      legacyKey: ensureLegacyKey(row, [row.album_id, row.artist_id]),
      albumId: mappings.require("albums", row.album_id),
      artistId: mappings.require("artists", row.artist_id),
      ...buildLifecycle(row),
    }),
  },
  {
    sourceTable: "tracks",
    convexTable: "tracks",
    transform: (row, mappings) => ({
      legacyKey: ensureLegacyKey(row, [row.id]),
      title: row.title ?? row.track_name,
      albumId: mappings.get("albums", row.album_id),
      appleMusicId: row.apple_music_id,
      spotifyId: row.spotify_id,
      deezerId: row.deezer_id,
      isrc: row.isrc,
      metadata: sanitizeJson(row.metadata) ?? {},
      ...buildLifecycle(row),
    }),
  },
  {
    sourceTable: "track_credits",
    convexTable: "trackCredits",
    transform: (row, mappings) => ({
      legacyKey: ensureLegacyKey(row, [row.id, row.track_id, row.credited_name, row.role]),
      trackId: mappings.require("tracks", row.track_id),
      artistId: mappings.get("artists", row.artist_id),
      creditedName: row.credited_name,
      role: row.role,
      metadata: sanitizeJson(row.metadata) ?? {},
      ...buildLifecycle(row),
    }),
  },
  {
    sourceTable: "artist_collaborations",
    convexTable: "artistCollaborations",
    transform: (row, mappings) => ({
      legacyKey: ensureLegacyKey(row, [row.id, row.artist_id_1, row.artist_id_2, row.collaboration_type]),
      artistId1: mappings.require("artists", row.artist_id_1),
      artistId2: mappings.require("artists", row.artist_id_2),
      collaborationType: row.collaboration_type ?? "featured",
      collaborationCount: row.collaboration_count
        ? Number(row.collaboration_count)
        : undefined,
      recentTracks: Array.isArray(row.recent_tracks)
        ? sanitizeJson(row.recent_tracks)
        : [],
      metadata: {},
      ...buildLifecycle(row),
    }),
  },
  {
    sourceTable: "user_track_blocks",
    convexTable: "userTrackBlocks",
    transform: (row, mappings) => ({
      legacyKey: ensureLegacyKey(row, [row.user_id, row.track_id ?? row.artist_id]),
      userId: mappings.require("users", row.user_id),
      artistId: mappings.require("artists", row.artist_id),
      trackId: mappings.get("tracks", row.track_id),
      reason: row.reason ?? row.note,
      metadata: sanitizeJson(row.flagged_artist_details) ?? {},
      ...buildLifecycle(row),
    }),
  },
  {
    sourceTable: "platform_sync_runs",
    convexTable: "platformSyncRuns",
    transform: (row) => ({
      legacyKey: ensureLegacyKey(row, [row.id]),
      platform: row.platform,
      status: row.status ?? "pending",
      startedAt: row.started_at ? toIsoString(row.started_at) : undefined,
      completedAt: row.completed_at ? toIsoString(row.completed_at) : undefined,
      errorLog: Array.isArray(row.error_log) ? sanitizeJson(row.error_log) : [],
      checkpointData: sanitizeJson(row.checkpoint_data) ?? {},
      metadata: {},
      ...buildLifecycle(row),
    }),
  },
  {
    sourceTable: "action_batches",
    convexTable: "actionBatches",
    transform: (row, mappings) => ({
      legacyKey: ensureLegacyKey(row, [row.id]),
      userId: mappings.require("users", row.user_id),
      provider: row.provider,
      idempotencyKey: row.idempotency_key,
      dryRun: Boolean(row.dry_run),
      status: row.status ?? "pending",
      options: sanitizeJson(row.options) ?? {},
      summary: sanitizeJson(row.summary) ?? {},
      completedAt: row.completed_at ? toIsoString(row.completed_at) : undefined,
      ...buildLifecycle(row),
    }),
  },
  {
    sourceTable: "action_items",
    convexTable: "actionItems",
    transform: (row, mappings) => ({
      legacyKey: ensureLegacyKey(row, [row.id, row.batch_id, row.entity_id, row.action]),
      batchId: mappings.require("actionBatches", row.batch_id),
      entityType: row.entity_type,
      entityId: row.entity_id,
      action: row.action,
      idempotencyKey: row.idempotency_key,
      beforeState: sanitizeJson(row.before_state),
      afterState: sanitizeJson(row.after_state),
      status: row.status ?? "pending",
      errorMessage: row.error_message,
      ...buildLifecycle(row),
    }),
  },
  {
    sourceTable: "news_sources",
    convexTable: "newsSources",
    transform: (row) => ({
      legacyKey: ensureLegacyKey(row, [row.id]),
      name: row.name,
      sourceType: row.source_type,
      url: row.url,
      config: sanitizeJson(row.config) ?? {},
      credibilityScore: row.credibility_score
        ? Number(row.credibility_score)
        : undefined,
      category: row.category,
      pollIntervalMinutes: row.poll_interval_minutes
        ? Number(row.poll_interval_minutes)
        : undefined,
      isActive: row.is_active ?? true,
      lastPolledAt: row.last_polled_at ? toIsoString(row.last_polled_at) : undefined,
      ...buildLifecycle(row),
    }),
  },
  {
    sourceTable: "news_articles",
    convexTable: "newsArticles",
    transform: (row, mappings) => ({
      legacyKey: ensureLegacyKey(row, [row.id]),
      sourceId: mappings.get("newsSources", row.source_id),
      url: row.url,
      title: row.title,
      content: row.content,
      summary: row.summary,
      publishedAt: row.published_at ? toIsoString(row.published_at) : undefined,
      processingStatus: row.processing_status ?? "pending",
      embeddingGenerated: Boolean(row.embedding_generated),
      rawData: sanitizeJson(row.raw_data) ?? {},
      metadata: {},
      ...buildLifecycle(row),
    }),
  },
  {
    sourceTable: "news_article_entities",
    convexTable: "newsArticleEntities",
    transform: (row, mappings) => ({
      legacyKey: ensureLegacyKey(row, [row.id]),
      articleId: mappings.require("newsArticles", row.article_id),
      artistId: mappings.get("artists", row.artist_id),
      entityName: row.entity_name ?? row.name,
      entityType: row.entity_type ?? "artist",
      confidence: row.confidence ? Number(row.confidence) : undefined,
      metadata: {},
      ...buildLifecycle(row),
    }),
  },
  {
    sourceTable: "news_offense_classifications",
    convexTable: "newsOffenseClassifications",
    transform: (row, mappings) => ({
      legacyKey: ensureLegacyKey(row, [row.id]),
      articleId: mappings.require("newsArticles", row.article_id),
      entityId: mappings.get("newsArticleEntities", row.entity_id),
      artistId: mappings.get("artists", row.artist_id),
      category: row.category,
      severity: row.severity ?? "moderate",
      confidence: row.confidence ? Number(row.confidence) : undefined,
      humanVerified: Boolean(row.human_verified),
      verifiedByUserId: mappings.get("users", row.verified_by),
      metadata: {},
      ...buildLifecycle(row),
    }),
  },
  {
    sourceTable: "social_media_posts",
    convexTable: "socialMediaPosts",
    transform: (row) => ({
      legacyKey: ensureLegacyKey(row, [row.id]),
      platform: row.platform,
      externalId: row.external_id,
      url: row.url,
      content: row.content,
      authorName: row.author_name,
      postedAt: row.posted_at ? toIsoString(row.posted_at) : undefined,
      engagementMetrics: sanitizeJson(row.engagement_metrics) ?? {},
      processingStatus: row.processing_status ?? "pending",
      metadata: {},
      ...buildLifecycle(row),
    }),
  },
  {
    sourceTable: "social_post_entities",
    convexTable: "socialPostEntities",
    transform: (row, mappings) => ({
      legacyKey: ensureLegacyKey(row, [row.id]),
      postId: mappings.require("socialMediaPosts", row.post_id),
      artistId: mappings.get("artists", row.artist_id),
      entityName: row.entity_name ?? row.name,
      entityType: row.entity_type ?? "artist",
      confidence: row.confidence ? Number(row.confidence) : undefined,
      metadata: {},
      ...buildLifecycle(row),
    }),
  },
  {
    sourceTable: "apple_music_enforcement_runs",
    convexTable: "enforcementRuns",
    transform: (row, mappings) => ({
      legacyKey: ensureLegacyKey(row, [row.id]),
      userId: mappings.require("users", row.user_id),
      provider: "apple_music",
      status: row.status ?? "pending",
      options: sanitizeJson(row.options) ?? {},
      errorDetails: sanitizeJson(row.error_details),
      startedAt: row.started_at ? toIsoString(row.started_at) : undefined,
      completedAt: row.completed_at ? toIsoString(row.completed_at) : undefined,
      ...buildLifecycle(row),
    }),
  },
  {
    sourceTable: "apple_music_enforcement_actions",
    convexTable: "enforcementActions",
    transform: (row, mappings) => ({
      legacyKey: ensureLegacyKey(row, [row.id]),
      runId: mappings.require("enforcementRuns", row.run_id),
      userId: mappings.require("users", row.user_id),
      resourceType: row.resource_type ?? "track",
      resourceId: row.resource_id,
      status: row.status ?? "pending",
      beforeState: sanitizeJson(row.before_state),
      afterState: sanitizeJson(row.after_state),
      metadata: {},
      ...buildLifecycle(row),
    }),
  },
];

async function importTable(
  postgres: Client,
  convex: ConvexHttpClient,
  mappings: MappingStore,
  definition: TableDefinition,
) {
  const exists = await tableExists(postgres, definition.sourceTable);
  if (!exists) {
    console.log(`Skipping ${definition.sourceTable}; table does not exist.`);
    return;
  }

  const result = await postgres.query(`select * from ${definition.sourceTable}`);
  const transformed = result.rows.map((row: LegacyRow) =>
    definition.transform(row, mappings),
  );

  console.log(
    `Importing ${definition.sourceTable} -> ${definition.convexTable} (${transformed.length} rows)`,
  );

  for (const [index, batch] of chunk(transformed, BATCH_SIZE).entries()) {
    const response = await convex.mutation("migration:importBatch" as any, {
      apiKey: migrationKey,
      table: definition.convexTable,
      batchKey: `${definition.convexTable}:${index}`,
      rows: batch,
    }) as {
      inserted: number;
      updated: number;
      mappings?: Array<{ legacyKey: string; convexId: string }>;
    };

    mappings.rememberBatch(definition.convexTable, response.mappings ?? []);
    console.log(
      `  batch ${index + 1}: inserted=${response.inserted} updated=${response.updated}`,
    );
  }
}

async function main() {
  const postgres = createPostgresClient();
  const convex = createConvexClient();
  const mappings = new MappingStore();

  await postgres.connect();

  try {
    for (const definition of definitions) {
      await importTable(postgres, convex, mappings, definition);
    }
  } finally {
    await postgres.end();
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
