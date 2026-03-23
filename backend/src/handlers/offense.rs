use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, QueryBuilder};
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::models::offense::{
    AddEvidenceRequest, CreateOffenseRequest, FlaggedArtist, ImportLibraryRequest,
    LibraryScanResponse, OffenseSeverity, OffenseSummary, OffenseWithEvidence,
};
use crate::models::{AuthenticatedUser, Claims};
use crate::services::offense::OffenseService;
use crate::AppState;

/// Query parameters for listing flagged artists
#[derive(Debug, Deserialize)]
pub struct FlaggedArtistsQuery {
    pub severity: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Get all flagged artists in the database
pub async fn get_flagged_artists(
    State(state): State<AppState>,
    Query(query): Query<FlaggedArtistsQuery>,
) -> Result<Json<Vec<FlaggedArtist>>> {
    let offense_service = OffenseService::new(&state.db_pool);

    let severity = query.severity.as_ref().and_then(|s| match s.as_str() {
        "minor" => Some(OffenseSeverity::Minor),
        "moderate" => Some(OffenseSeverity::Moderate),
        "severe" => Some(OffenseSeverity::Severe),
        "egregious" => Some(OffenseSeverity::Egregious),
        _ => None,
    });

    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);

    let artists = offense_service
        .get_flagged_artists(severity, limit, offset)
        .await?;

    Ok(Json(artists))
}

/// Get offense details with evidence
pub async fn get_offense(
    State(state): State<AppState>,
    Path(offense_id): Path<Uuid>,
) -> Result<Json<OffenseWithEvidence>> {
    let offense_service = OffenseService::new(&state.db_pool);
    let offense = offense_service
        .get_offense_with_evidence(offense_id)
        .await?;
    Ok(Json(offense))
}

/// Create a new offense (requires authentication)
pub async fn create_offense(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(request): Json<CreateOffenseRequest>,
) -> Result<impl IntoResponse> {
    let offense_service = OffenseService::new(&state.db_pool);
    let offense = offense_service
        .create_offense(request, Some(user.id))
        .await?;
    Ok((StatusCode::CREATED, Json(offense)))
}

/// Add evidence to an offense (requires authentication)
pub async fn add_evidence(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(request): Json<AddEvidenceRequest>,
) -> Result<impl IntoResponse> {
    let offense_service = OffenseService::new(&state.db_pool);
    let evidence = offense_service.add_evidence(request, Some(user.id)).await?;
    Ok((StatusCode::CREATED, Json(evidence)))
}

/// Verify an offense (moderator only)
/// Requires moderator or admin role to verify offenses
pub async fn verify_offense(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    claims: Claims,
    Path(offense_id): Path<Uuid>,
) -> Result<StatusCode> {
    // Check if user has moderator role
    if !claims.has_moderator_access() {
        tracing::warn!(
            user_id = %user.id,
            offense_id = %offense_id,
            role = ?claims.role,
            "Unauthorized attempt to verify offense - moderator role required"
        );
        return Err(AppError::InsufficientPermissions);
    }

    tracing::info!(
        user_id = %user.id,
        offense_id = %offense_id,
        role = ?claims.role,
        "Moderator verifying offense"
    );

    let offense_service = OffenseService::new(&state.db_pool);
    offense_service.verify_offense(offense_id, user.id).await?;
    Ok(StatusCode::OK)
}

/// Import library tracks from file upload
pub async fn import_library(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(request): Json<ImportLibraryRequest>,
) -> Result<Json<serde_json::Value>> {
    let offense_service = OffenseService::new(&state.db_pool);
    let imported = offense_service.import_library(user.id, request).await?;
    Ok(Json(serde_json::json!({
        "imported": imported,
        "message": format!("Successfully imported {} tracks", imported)
    })))
}

/// Scan user's library against offense database
pub async fn scan_library(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<LibraryScanResponse>> {
    let offense_service = OffenseService::new(&state.db_pool);
    let result = offense_service.scan_library(user.id).await?;
    Ok(Json(result))
}

/// Get cached library scan results (if any previous scan exists)
pub async fn get_cached_scan(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<Option<LibraryScanResponse>>> {
    let offense_service = OffenseService::new(&state.db_pool);
    let result = offense_service.get_cached_scan(user.id).await?;
    Ok(Json(result))
}

#[derive(Debug, Deserialize)]
pub struct LibraryOffendersQuery {
    pub provider: Option<String>,
    pub kind: Option<String>,
    pub days: Option<i32>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct LibraryOffender {
    pub id: Uuid,
    pub name: String,
    /// Count of matching items in the user's library (items, not plays).
    pub track_count: i32,
    pub severity: OffenseSeverity,
    pub offenses: Vec<OffenseSummary>,
    /// Streams/plays in the selected window, if playcount data exists for the user.
    pub play_count: Option<i64>,
    /// Estimated payout generated by those plays, in USD (string-serialized decimal).
    pub estimated_revenue: Option<String>,
    /// Share of the user's estimated payout in the selected window.
    pub percentage_of_user_spend: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct LibraryOffendersResponse {
    pub computed_at: DateTime<Utc>,
    pub total_flagged_artists: i64,
    pub total_flagged_tracks: i64,
    pub playcount_window_days: i32,
    pub playcounts_available: bool,
    pub offenders: Vec<LibraryOffender>,
}

/// Return the "favorite worst offenders" in a user's library (verified offenses only).
///
/// Designed for a small UI card: fast, small payload, and avoids scan N+1 queries.
pub async fn get_library_offenders(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<LibraryOffendersQuery>,
) -> Result<Json<LibraryOffendersResponse>> {
    #[derive(Debug, sqlx::FromRow)]
    struct OffenderRow {
        artist_id: Uuid,
        artist_name: String,
        track_count: i64,
        highest_severity: OffenseSeverity,
        play_count: i64,
        estimated_revenue: Decimal,
        total_flagged_artists: i64,
        total_flagged_tracks: i64,
    }

    let provider = normalize_optional(query.provider.clone());
    let kind = normalize_optional(query.kind.clone());
    // `days=0` means "all time". Any negative values are treated as all time as well.
    let raw_days = query.days.unwrap_or(0);
    let days = if raw_days <= 0 {
        0
    } else {
        raw_days.clamp(1, 3650)
    };
    let limit = query.limit.unwrap_or(5).clamp(1, 20);

    // Default to songs-only unless explicitly set to "all".
    let songs_only = !matches!(kind.as_deref(), Some("all"));

    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"
        WITH offenders AS (
          SELECT
            a.id AS artist_id,
            a.canonical_name AS artist_name,
            COUNT(DISTINCT ult.id)::bigint AS track_count,
            MAX(ao.severity) AS highest_severity
          FROM user_library_tracks ult
          JOIN artists a ON (
            a.id = ult.artist_id
            OR (
              ult.artist_id IS NULL
              AND ult.artist_name IS NOT NULL
              AND LOWER(ult.artist_name) = LOWER(a.canonical_name)
            )
          )
          JOIN artist_offenses ao ON ao.artist_id = a.id AND ao.status = 'verified'
          WHERE ult.user_id =
        "#,
    );
    qb.push_bind(user.id);

    if let Some(provider) = provider.as_deref() {
        qb.push(" AND ult.provider = ");
        qb.push_bind(provider);
    }

    if songs_only {
        qb.push(
            " AND ult.provider_track_id NOT LIKE 'album:%'
              AND ult.provider_track_id NOT LIKE 'artist:%'
              AND ult.provider_track_id NOT LIKE 'subscription:%'
              AND ult.provider_track_id NOT LIKE 'playlist:%'",
        );
    }

    qb.push(
        r#"
          GROUP BY a.id, a.canonical_name
        ),
        plays AS (
          SELECT
            pc.artist_id,
            COALESCE(SUM(pc.play_count), 0)::bigint AS play_count,
            COALESCE(SUM(pc.estimated_revenue), 0) AS estimated_revenue
          FROM user_artist_playcounts pc
          WHERE pc.user_id =
        "#,
    );
    qb.push_bind(user.id);
    if days > 0 {
        qb.push(" AND pc.period_start >= CURRENT_DATE - ");
        qb.push_bind(days);
        qb.push("::integer");
    }
    if let Some(provider) = provider.as_deref() {
        qb.push(" AND pc.platform = ");
        qb.push_bind(provider);
    }
    qb.push(
        r#"
          GROUP BY pc.artist_id
        ),
        ranked AS (
          SELECT
            o.artist_id,
            o.artist_name,
            o.track_count,
            o.highest_severity,
            COALESCE(p.play_count, 0)::bigint AS play_count,
            COALESCE(p.estimated_revenue, 0) AS estimated_revenue,
            COUNT(*) OVER()::bigint AS total_flagged_artists,
            COALESCE(SUM(o.track_count) OVER(), 0)::bigint AS total_flagged_tracks
          FROM offenders o
          LEFT JOIN plays p ON p.artist_id = o.artist_id
        )
        SELECT
          artist_id,
          artist_name,
          track_count,
          highest_severity,
          play_count,
          estimated_revenue,
          total_flagged_artists,
          total_flagged_tracks
        FROM ranked
        ORDER BY play_count DESC, track_count DESC, highest_severity DESC, artist_name ASC
        LIMIT
        "#,
    );
    qb.push_bind(limit);

    let offender_rows: Vec<OffenderRow> = qb
        .build_query_as::<OffenderRow>()
        .fetch_all(&state.db_pool)
        .await
        .map_err(AppError::DatabaseQueryFailed)?;

    let computed_at = Utc::now();
    if offender_rows.is_empty() {
        return Ok(Json(LibraryOffendersResponse {
            computed_at,
            total_flagged_artists: 0,
            total_flagged_tracks: 0,
            playcount_window_days: days,
            playcounts_available: false,
            offenders: Vec::new(),
        }));
    }

    let total_flagged_artists = offender_rows
        .first()
        .map(|r| r.total_flagged_artists)
        .unwrap_or(0);
    let total_flagged_tracks = offender_rows
        .first()
        .map(|r| r.total_flagged_tracks)
        .unwrap_or(0);

    #[derive(Debug, sqlx::FromRow)]
    struct UserPlaycountTotalsRow {
        total_streams: i64,
        total_revenue: Decimal,
    }

    let mut totals_qb: QueryBuilder<Postgres> = QueryBuilder::new(
        "SELECT COALESCE(SUM(play_count), 0)::bigint AS total_streams, COALESCE(SUM(estimated_revenue), 0) AS total_revenue FROM user_artist_playcounts WHERE user_id = ",
    );
    totals_qb.push_bind(user.id);
    if days > 0 {
        totals_qb.push(" AND period_start >= CURRENT_DATE - ");
        totals_qb.push_bind(days);
        totals_qb.push("::integer");
    }
    if let Some(provider) = provider.as_deref() {
        totals_qb.push(" AND platform = ");
        totals_qb.push_bind(provider);
    }

    let user_totals: UserPlaycountTotalsRow = totals_qb
        .build_query_as()
        .fetch_one(&state.db_pool)
        .await
        .map_err(AppError::DatabaseQueryFailed)?;

    let playcounts_available =
        user_totals.total_streams > 0 || user_totals.total_revenue > Decimal::ZERO;

    let artist_ids: Vec<Uuid> = offender_rows.iter().map(|r| r.artist_id).collect();
    let offenses_per_artist: i64 = 3;

    #[derive(Debug, sqlx::FromRow)]
    struct OffenseSummaryRow {
        artist_id: Uuid,
        category: crate::models::offense::OffenseCategory,
        title: String,
        incident_date: Option<chrono::NaiveDate>,
        evidence_count: i64,
    }

    let offense_rows: Vec<OffenseSummaryRow> = sqlx::query_as(
        r#"
        WITH ranked AS (
          SELECT
            ao.artist_id,
            ao.category,
            ao.title,
            ao.incident_date,
            COUNT(oe.id)::bigint AS evidence_count,
            ROW_NUMBER() OVER (
              PARTITION BY ao.artist_id
              ORDER BY ao.severity DESC, ao.verified_at DESC NULLS LAST, ao.created_at DESC
            ) AS rn
          FROM artist_offenses ao
          LEFT JOIN offense_evidence oe ON oe.offense_id = ao.id
          WHERE ao.status = 'verified'
            AND ao.artist_id = ANY($1)
          GROUP BY ao.id, ao.artist_id, ao.category, ao.title, ao.incident_date, ao.severity, ao.verified_at, ao.created_at
        )
        SELECT
          artist_id,
          category,
          title,
          incident_date,
          evidence_count
        FROM ranked
        WHERE rn <= $2
        ORDER BY artist_id, rn
        "#,
    )
    .bind(&artist_ids)
    .bind(offenses_per_artist)
    .fetch_all(&state.db_pool)
    .await
    .map_err(AppError::DatabaseQueryFailed)?;

    let mut offenses_by_artist: HashMap<Uuid, Vec<OffenseSummary>> = HashMap::new();
    for row in offense_rows {
        offenses_by_artist
            .entry(row.artist_id)
            .or_default()
            .push(OffenseSummary {
                category: row.category,
                title: row.title,
                date: row
                    .incident_date
                    .map(|d| d.format("%Y").to_string())
                    .unwrap_or_else(|| "Unknown".to_string()),
                evidence_count: row.evidence_count as i32,
            });
    }

    let offenders: Vec<LibraryOffender> = offender_rows
        .into_iter()
        .map(|row| {
            let percentage = if playcounts_available && user_totals.total_revenue > Decimal::ZERO {
                let pct = (row.estimated_revenue / user_totals.total_revenue * Decimal::from(100))
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0);
                Some(pct)
            } else {
                None
            };

            LibraryOffender {
                id: row.artist_id,
                name: row.artist_name,
                track_count: row.track_count.clamp(0, i32::MAX as i64) as i32,
                severity: row.highest_severity,
                offenses: offenses_by_artist
                    .remove(&row.artist_id)
                    .unwrap_or_default(),
                play_count: if playcounts_available {
                    Some(row.play_count.max(0))
                } else {
                    None
                },
                estimated_revenue: if playcounts_available {
                    Some(row.estimated_revenue.to_string())
                } else {
                    None
                },
                percentage_of_user_spend: percentage,
            }
        })
        .collect();

    Ok(Json(LibraryOffendersResponse {
        computed_at,
        total_flagged_artists,
        total_flagged_tracks,
        playcount_window_days: days,
        playcounts_available,
        offenders,
    }))
}

/// Get user's imported library tracks
pub async fn get_library(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<LibraryQuery>,
) -> Result<Json<Vec<crate::models::offense::UserLibraryTrack>>> {
    let offense_service = OffenseService::new(&state.db_pool);
    let tracks = offense_service
        .get_user_library(user.id, query.provider)
        .await?;
    Ok(Json(tracks))
}

#[derive(Debug, Deserialize)]
pub struct LibraryQuery {
    pub provider: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LibraryItemsQuery {
    pub provider: Option<String>,
    pub kind: Option<String>,
    pub q: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub playlist: Option<String>,
    pub source_type: Option<String>,
    pub sort: Option<String>,
    pub dir: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct LibraryItemsResponse {
    pub items: Vec<crate::models::offense::UserLibraryTrack>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    let trimmed = value?.trim().to_string();
    if trimmed.is_empty() || trimmed == "all" {
        None
    } else {
        Some(trimmed)
    }
}

fn apply_library_filters(qb: &mut QueryBuilder<'_, Postgres>, query: &LibraryItemsQuery) {
    if let Some(provider) = normalize_optional(query.provider.clone()) {
        qb.push(" AND ult.provider = ");
        qb.push_bind(provider);
    }

    if let Some(kind) = normalize_optional(query.kind.clone()) {
        match kind.as_str() {
            "albums" => {
                qb.push(" AND ult.provider_track_id LIKE 'album:%'");
            }
            "artists" => {
                qb.push(
                " AND (ult.provider_track_id LIKE 'artist:%' OR ult.provider_track_id LIKE 'subscription:%')",
            );
            }
            "playlists" => {
                qb.push(" AND ult.provider_track_id LIKE 'playlist:%'");
            }
            "songs" => {
                qb.push(
                " AND ult.provider_track_id NOT LIKE 'album:%' AND ult.provider_track_id NOT LIKE 'artist:%' AND ult.provider_track_id NOT LIKE 'subscription:%' AND ult.provider_track_id NOT LIKE 'playlist:%'",
            );
            }
            _ => {}
        }
    }

    if let Some(source_type) = normalize_optional(query.source_type.clone()) {
        qb.push(" AND ult.source_type = ");
        qb.push_bind(source_type);
    }

    if let Some(artist) = normalize_optional(query.artist.clone()) {
        qb.push(" AND ult.artist_name IS NOT NULL AND LOWER(ult.artist_name) = LOWER(");
        qb.push_bind(artist);
        qb.push(")");
    }

    if let Some(album) = normalize_optional(query.album.clone()) {
        qb.push(" AND ult.album_name IS NOT NULL AND LOWER(ult.album_name) = LOWER(");
        qb.push_bind(album);
        qb.push(")");
    }

    if let Some(playlist) = normalize_optional(query.playlist.clone()) {
        qb.push(" AND ult.playlist_name IS NOT NULL AND LOWER(ult.playlist_name) = LOWER(");
        qb.push_bind(playlist);
        qb.push(")");
    }

    if let Some(q) = normalize_optional(query.q.clone()) {
        let pattern = format!("%{}%", q);
        qb.push(" AND (");
        qb.push("COALESCE(ult.track_name, '') ILIKE ");
        qb.push_bind(pattern.clone());
        qb.push(" OR COALESCE(ult.artist_name, '') ILIKE ");
        qb.push_bind(pattern.clone());
        qb.push(" OR COALESCE(ult.album_name, '') ILIKE ");
        qb.push_bind(pattern.clone());
        qb.push(" OR COALESCE(ult.playlist_name, '') ILIKE ");
        qb.push_bind(pattern.clone());
        qb.push(" OR COALESCE(ult.source_type, '') ILIKE ");
        qb.push_bind(pattern);
        qb.push(")");
    }
}

/// Browse user's library items with fast server-side filtering + sorting.
pub async fn get_library_items(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<LibraryItemsQuery>,
) -> Result<Json<LibraryItemsResponse>> {
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let offset = query.offset.unwrap_or(0).max(0);

    let mut count_qb: QueryBuilder<Postgres> =
        QueryBuilder::new("SELECT COUNT(*) FROM user_library_tracks ult WHERE ult.user_id = ");
    count_qb.push_bind(user.id);
    apply_library_filters(&mut count_qb, &query);

    let total: i64 = count_qb
        .build_query_scalar()
        .fetch_one(&state.db_pool)
        .await
        .map_err(AppError::DatabaseQueryFailed)?;

    let sort_key = query.sort.as_deref().unwrap_or("last_synced");
    let sort_dir = query.dir.as_deref().unwrap_or("desc");

    let (order_expr, nulls_last) = match sort_key {
        "title" => ("COALESCE(ult.track_name, '')", false),
        "artist" => ("COALESCE(ult.artist_name, '')", false),
        "album" => ("COALESCE(ult.album_name, '')", false),
        "provider" => ("ult.provider", false),
        "added_at" => ("ult.added_at", true),
        _ => ("ult.last_synced", true),
    };

    let dir_sql = if sort_dir.eq_ignore_ascii_case("asc") {
        "ASC"
    } else {
        "DESC"
    };

    let mut items_qb: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"
        SELECT id, user_id, provider, provider_track_id, track_name, album_name,
               artist_id, artist_name, source_type, playlist_name, added_at, last_synced
        FROM user_library_tracks ult
        WHERE ult.user_id =
        "#,
    );
    items_qb.push_bind(user.id);
    apply_library_filters(&mut items_qb, &query);
    items_qb.push(" ORDER BY ");
    items_qb.push(order_expr);
    items_qb.push(" ");
    items_qb.push(dir_sql);
    if nulls_last {
        items_qb.push(" NULLS LAST");
    }
    items_qb.push(", ult.id DESC");
    items_qb.push(" LIMIT ");
    items_qb.push_bind(limit);
    items_qb.push(" OFFSET ");
    items_qb.push_bind(offset);

    let items = items_qb
        .build_query_as::<crate::models::offense::UserLibraryTrack>()
        .fetch_all(&state.db_pool)
        .await
        .map_err(AppError::DatabaseQueryFailed)?;

    Ok(Json(LibraryItemsResponse {
        items,
        total,
        limit,
        offset,
    }))
}

#[derive(Debug, Deserialize)]
pub struct LibraryGroupsQuery {
    pub group_by: Option<String>,
    pub provider: Option<String>,
    pub kind: Option<String>,
    pub q: Option<String>,
    pub sort: Option<String>,
    pub dir: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct LibraryGroup {
    pub value: String,
    pub secondary: Option<String>,
    pub provider: Option<String>,
    pub count: i64,
    pub last_synced: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct LibraryGroupsResponse {
    pub group_by: String,
    pub groups: Vec<LibraryGroup>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// List grouped library facets (artist/album/playlist/provider/kind).
pub async fn get_library_groups(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<LibraryGroupsQuery>,
) -> Result<Json<LibraryGroupsResponse>> {
    let group_by =
        normalize_optional(query.group_by.clone()).unwrap_or_else(|| "artist".to_string());
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let offset = query.offset.unwrap_or(0).max(0);

    // Reuse the item filter logic by mapping groups query -> items query.
    let items_query = LibraryItemsQuery {
        provider: query.provider.clone(),
        kind: query.kind.clone(),
        q: query.q.clone(),
        artist: None,
        album: None,
        playlist: None,
        source_type: None,
        sort: None,
        dir: None,
        limit: None,
        offset: None,
    };

    let (select_expr, secondary_expr, provider_expr): (&str, Option<&str>, Option<&str>) =
        match group_by.as_str() {
            "album" => (
                "COALESCE(NULLIF(TRIM(ult.album_name), ''), 'Unknown Album')",
                Some("COALESCE(NULLIF(TRIM(ult.artist_name), ''), 'Unknown Artist')"),
                None,
            ),
            "playlist" => (
                "COALESCE(NULLIF(TRIM(ult.playlist_name), ''), 'Unknown Playlist')",
                None,
                Some("ult.provider"),
            ),
            "provider" => ("ult.provider", None, None),
            "kind" => (
                r#"
                CASE
                  WHEN ult.provider_track_id LIKE 'album:%' THEN 'albums'
                  WHEN ult.provider_track_id LIKE 'playlist:%' THEN 'playlists'
                  WHEN ult.provider_track_id LIKE 'artist:%' OR ult.provider_track_id LIKE 'subscription:%' THEN 'artists'
                  ELSE 'songs'
                END
                "#,
                None,
                None,
            ),
            _ => (
                "COALESCE(NULLIF(TRIM(ult.artist_name), ''), 'Unknown Artist')",
                None,
                None,
            ),
        };

    // Always group by the first 3 projections: value, secondary, provider.
    // (secondary/provider may be NULL constants depending on group type.)
    let group_cols = "1,2,3";

    let mut count_qb: QueryBuilder<Postgres> = QueryBuilder::new("SELECT COUNT(*) FROM (SELECT ");
    count_qb.push(select_expr);
    count_qb.push(" AS value, ");
    if let Some(secondary) = secondary_expr {
        count_qb.push(secondary);
    } else {
        count_qb.push("NULL::text");
    }
    count_qb.push(" AS secondary, ");
    if let Some(provider) = provider_expr {
        count_qb.push(provider);
    } else {
        count_qb.push("NULL::text");
    }
    count_qb.push(" AS provider FROM user_library_tracks ult WHERE ult.user_id = ");
    count_qb.push_bind(user.id);
    apply_library_filters(&mut count_qb, &items_query);
    count_qb.push(" GROUP BY ");
    count_qb.push(group_cols);
    count_qb.push(") grouped");

    let total: i64 = count_qb
        .build_query_scalar()
        .fetch_one(&state.db_pool)
        .await
        .map_err(AppError::DatabaseQueryFailed)?;

    let sort_key = query.sort.as_deref().unwrap_or("count");
    let sort_dir = query.dir.as_deref().unwrap_or("desc");
    let dir_sql = if sort_dir.eq_ignore_ascii_case("asc") {
        "ASC"
    } else {
        "DESC"
    };

    let order_expr = match sort_key {
        "name" => "value",
        "last_synced" => "last_synced",
        _ => "count",
    };

    let mut groups_qb: QueryBuilder<Postgres> = QueryBuilder::new("SELECT ");
    groups_qb.push(select_expr);
    groups_qb.push(" AS value, ");

    if let Some(secondary) = secondary_expr {
        groups_qb.push(secondary);
        groups_qb.push(" AS secondary, ");
    } else {
        groups_qb.push("NULL::text AS secondary, ");
    }

    if let Some(provider) = provider_expr {
        groups_qb.push(provider);
        groups_qb.push(" AS provider, ");
    } else {
        groups_qb.push("NULL::text AS provider, ");
    }

    groups_qb.push("COUNT(*)::bigint AS count, MAX(ult.last_synced) AS last_synced ");
    groups_qb.push("FROM user_library_tracks ult WHERE ult.user_id = ");
    groups_qb.push_bind(user.id);
    apply_library_filters(&mut groups_qb, &items_query);
    groups_qb.push(" GROUP BY ");
    groups_qb.push(group_cols);
    groups_qb.push(" ORDER BY ");
    groups_qb.push(order_expr);
    groups_qb.push(" ");
    groups_qb.push(dir_sql);
    groups_qb.push(", last_synced DESC");
    groups_qb.push(" LIMIT ");
    groups_qb.push_bind(limit);
    groups_qb.push(" OFFSET ");
    groups_qb.push_bind(offset);

    let groups = groups_qb
        .build_query_as::<LibraryGroup>()
        .fetch_all(&state.db_pool)
        .await
        .map_err(AppError::DatabaseQueryFailed)?;

    Ok(Json(LibraryGroupsResponse {
        group_by,
        groups,
        total,
        limit,
        offset,
    }))
}

#[derive(Debug, Serialize)]
pub struct TasteGradeComponent {
    pub id: String,
    pub label: String,
    pub weight: f64,
    pub score: f64,
    pub grade: String,
    pub summary: String,
}

#[derive(Debug, Serialize)]
pub struct TasteGradeResponse {
    pub computed_at: DateTime<Utc>,
    pub overall_score: f64,
    pub overall_grade: String,
    pub components: Vec<TasteGradeComponent>,
    pub signals: Vec<String>,
    pub recommendations: Vec<String>,
}

fn grade_from_score(score: f64) -> &'static str {
    match score {
        s if s >= 97.0 => "A+",
        s if s >= 93.0 => "A",
        s if s >= 90.0 => "A-",
        s if s >= 87.0 => "B+",
        s if s >= 83.0 => "B",
        s if s >= 80.0 => "B-",
        s if s >= 77.0 => "C+",
        s if s >= 73.0 => "C",
        s if s >= 70.0 => "C-",
        s if s >= 60.0 => "D",
        _ => "F",
    }
}

fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

/// Grade "music taste" using library metadata and the offense database (fun + informative).
pub async fn get_taste_grade(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<TasteGradeResponse>> {
    #[derive(sqlx::FromRow)]
    struct LibrarySummaryRow {
        total_items: i64,
        songs: i64,
        albums: i64,
        #[allow(dead_code)]
        artists: i64,
        #[allow(dead_code)]
        playlists: i64,
        unique_artists: i64,
        unique_albums: i64,
        unique_playlists: i64,
        last_synced_max: Option<DateTime<Utc>>,
    }

    let summary = sqlx::query_as::<_, LibrarySummaryRow>(
        r#"
        SELECT
          COUNT(*)::bigint AS total_items,
          COUNT(*) FILTER (
            WHERE provider_track_id NOT LIKE 'album:%'
              AND provider_track_id NOT LIKE 'artist:%'
              AND provider_track_id NOT LIKE 'subscription:%'
              AND provider_track_id NOT LIKE 'playlist:%'
          )::bigint AS songs,
          COUNT(*) FILTER (WHERE provider_track_id LIKE 'album:%')::bigint AS albums,
          COUNT(*) FILTER (
            WHERE provider_track_id LIKE 'artist:%'
               OR provider_track_id LIKE 'subscription:%'
          )::bigint AS artists,
          COUNT(*) FILTER (WHERE provider_track_id LIKE 'playlist:%')::bigint AS playlists,
          COUNT(DISTINCT LOWER(artist_name)) FILTER (
            WHERE artist_name IS NOT NULL AND TRIM(artist_name) <> '' AND provider_track_id NOT LIKE 'playlist:%'
          )::bigint AS unique_artists,
          COUNT(DISTINCT LOWER(album_name)) FILTER (
            WHERE album_name IS NOT NULL AND TRIM(album_name) <> ''
          )::bigint AS unique_albums,
          COUNT(DISTINCT LOWER(playlist_name)) FILTER (
            WHERE playlist_name IS NOT NULL AND TRIM(playlist_name) <> ''
          )::bigint AS unique_playlists,
          MAX(last_synced) AS last_synced_max
        FROM user_library_tracks
        WHERE user_id = $1
        "#,
    )
    .bind(user.id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(AppError::DatabaseQueryFailed)?;

    // Top artist concentration (songs only).
    let top_artist_counts: Vec<i64> = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::bigint AS cnt
        FROM user_library_tracks ult
        WHERE ult.user_id = $1
          AND ult.provider_track_id NOT LIKE 'album:%'
          AND ult.provider_track_id NOT LIKE 'artist:%'
          AND ult.provider_track_id NOT LIKE 'subscription:%'
          AND ult.provider_track_id NOT LIKE 'playlist:%'
          AND ult.artist_name IS NOT NULL
          AND TRIM(ult.artist_name) <> ''
        GROUP BY LOWER(ult.artist_name)
        ORDER BY cnt DESC
        LIMIT 5
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(AppError::DatabaseQueryFailed)?;

    let songs_total = summary.songs.max(0) as f64;
    let top1_share = if songs_total > 0.0 {
        (top_artist_counts.first().copied().unwrap_or(0) as f64) / songs_total
    } else {
        0.0
    };

    // "Safety" (presence of verified-offense artists) among songs.
    let flagged_songs: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::bigint
        FROM user_library_tracks ult
        WHERE ult.user_id = $1
          AND ult.provider_track_id NOT LIKE 'album:%'
          AND ult.provider_track_id NOT LIKE 'artist:%'
          AND ult.provider_track_id NOT LIKE 'subscription:%'
          AND ult.provider_track_id NOT LIKE 'playlist:%'
          AND EXISTS (
            SELECT 1
            FROM artists a
            JOIN artist_offenses ao ON ao.artist_id = a.id AND ao.status = 'verified'
            WHERE a.id = ult.artist_id
               OR (ult.artist_name IS NOT NULL AND LOWER(ult.artist_name) = LOWER(a.canonical_name))
          )
        "#,
    )
    .bind(user.id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(AppError::DatabaseQueryFailed)?;

    let flagged_ratio = if songs_total > 0.0 {
        (flagged_songs.max(0) as f64) / songs_total
    } else {
        0.0
    };

    let now = Utc::now();
    let last_synced = summary.last_synced_max.unwrap_or(now);
    let freshness_days = (now - last_synced).num_seconds().max(0) as f64 / 86_400.0;

    // Component scoring (0..100)
    let diversity_base = if songs_total > 0.0 {
        let ua = (summary.unique_artists.max(0) as f64) + 1.0;
        let st = songs_total + 1.0;
        (ua.ln() / st.ln()).clamp(0.0, 1.0)
    } else {
        0.0
    };
    // Penalize over-concentration: if top artist is >25% of library, diversity drops.
    let concentration_penalty = clamp01(1.0 - (top1_share * 1.2));
    let diversity_score = 100.0 * clamp01(diversity_base * concentration_penalty);

    // Curation: playlists + albums with logarithmic scaling (keeps big libraries from auto-acing).
    let playlists_norm =
        clamp01(((summary.unique_playlists.max(0) as f64) + 1.0).ln() / (51.0f64).ln());
    let albums_norm =
        clamp01(((summary.unique_albums.max(0) as f64) + 1.0).ln() / (2001.0f64).ln());
    let curation_score = 100.0 * clamp01(playlists_norm * 0.55 + albums_norm * 0.45);

    // Freshness: exponential decay with a 30-day half-life-ish.
    let freshness_score = 100.0 * clamp01((-freshness_days / 30.0).exp());

    // Safety: penalize verified-offense artists; stronger penalty after ~5%.
    let safety_score = 100.0 * clamp01(1.0 - (flagged_ratio * 2.0).min(1.0));

    let weights = [
        (
            "diversity",
            "Diversity",
            0.35,
            diversity_score,
            format!("{} unique artists", summary.unique_artists),
        ),
        (
            "curation",
            "Curation",
            0.25,
            curation_score,
            format!(
                "{} playlists, {} albums",
                summary.unique_playlists, summary.unique_albums
            ),
        ),
        (
            "freshness",
            "Freshness",
            0.15,
            freshness_score,
            format!("Last synced {}", last_synced.to_rfc3339()),
        ),
        (
            "safety",
            "Safety",
            0.25,
            safety_score,
            format!("{}/{} flagged songs", flagged_songs, summary.songs),
        ),
    ];

    let mut components: Vec<TasteGradeComponent> = Vec::new();
    let mut overall = 0.0f64;
    for (id, label, weight, score, summary_text) in weights {
        overall += weight * score;
        components.push(TasteGradeComponent {
            id: id.to_string(),
            label: label.to_string(),
            weight,
            score: (score * 100.0).round() / 100.0,
            grade: grade_from_score(score).to_string(),
            summary: summary_text,
        });
    }

    let overall_score = (overall * 100.0).round() / 100.0;
    let overall_grade = grade_from_score(overall_score).to_string();

    let mut signals = Vec::new();
    signals.push(format!("Library size: {} total items", summary.total_items));
    signals.push(format!(
        "Songs: {}, Albums: {}, Playlists: {}",
        summary.songs, summary.albums, summary.unique_playlists
    ));
    if summary.unique_artists > 0 {
        signals.push(format!("Unique artists: {}", summary.unique_artists));
    }
    if songs_total > 0.0 {
        signals.push(format!(
            "Top artist concentration: {:.0}%",
            top1_share * 100.0
        ));
    }
    if flagged_songs > 0 {
        signals.push(format!("Flagged songs detected: {}", flagged_songs));
    }

    let mut recommendations = Vec::new();
    if summary.unique_playlists < 5 && summary.songs > 200 {
        recommendations.push(
            "Create a few focused playlists (mood/era/genre). Your future self will thank you."
                .to_string(),
        );
    }
    if diversity_score < 75.0 && summary.songs > 200 {
        recommendations
            .push("Try a 'one-new-artist-a-week' playlist to broaden the rotation.".to_string());
    }
    if freshness_days > 60.0 {
        recommendations.push(
            "Sync your library again to refresh stats and capture recent favorites.".to_string(),
        );
    }
    if safety_score < 90.0 {
        recommendations.push(
            "Run a Library Scan to see which artists are driving the Safety score.".to_string(),
        );
    }

    Ok(Json(TasteGradeResponse {
        computed_at: now,
        overall_score,
        overall_grade,
        components,
        signals,
        recommendations,
    }))
}

/// Query parameters for getting artists by category
#[derive(Debug, Deserialize)]
pub struct CategoryArtistsQuery {
    pub category: Option<String>,
    pub artist_id: Option<Uuid>,
}

/// Get artists by category or get artist details
pub async fn get_category_artists(
    State(state): State<AppState>,
    Query(query): Query<CategoryArtistsQuery>,
) -> Result<Json<serde_json::Value>> {
    let offense_service = OffenseService::new(&state.db_pool);

    // If artist_id is provided, return artist details
    if let Some(artist_id) = query.artist_id {
        let details = offense_service.get_artist_details(artist_id).await?;
        return Ok(Json(serde_json::json!({
            "success": true,
            "data": details
        })));
    }

    // If category is provided, return artists in that category
    if let Some(category) = &query.category {
        let artists = offense_service.get_artists_by_category(category).await?;
        return Ok(Json(serde_json::json!({
            "success": true,
            "data": {
                "artists": artists
            }
        })));
    }

    // Neither provided - return all flagged artists
    let artists = offense_service.get_flagged_artists(None, 100, 0).await?;
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "artists": artists
        }
    })))
}

// ── Playlist browser ────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ListPlaylistsQuery {
    pub provider: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PlaylistSummary {
    pub provider: String,
    pub playlist_name: String,
    pub total_tracks: i64,
    pub flagged_tracks: i64,
    pub clean_ratio: f64,
    pub grade: String,
    pub unique_artists: i64,
    pub flagged_artists: Vec<String>,
    pub last_synced: Option<DateTime<Utc>>,
    pub cover_images: Vec<String>,
}

fn grade_from_ratio(clean_ratio: f64) -> String {
    if clean_ratio < 0.5 {
        "F"
    } else if clean_ratio < 0.6 {
        "D"
    } else if clean_ratio < 0.7 {
        "C"
    } else if clean_ratio < 0.8 {
        "B"
    } else if clean_ratio < 0.95 {
        "A"
    } else {
        "A+"
    }
    .to_string()
}

/// List all playlists for the authenticated user, grouped from library tracks.
pub async fn list_playlists(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<ListPlaylistsQuery>,
) -> Result<Json<serde_json::Value>> {
    #[derive(sqlx::FromRow)]
    struct PlaylistRow {
        provider: String,
        playlist_name: String,
        total_tracks: i64,
        unique_artists: i64,
        last_synced: Option<DateTime<Utc>>,
    }

    let provider = normalize_optional(query.provider);

    let playlists = if let Some(ref prov) = provider {
        sqlx::query_as::<_, PlaylistRow>(
            r#"
            SELECT
                provider,
                playlist_name,
                COUNT(*)::bigint AS total_tracks,
                COUNT(DISTINCT LOWER(artist_name)) FILTER (
                    WHERE artist_name IS NOT NULL AND TRIM(artist_name) <> ''
                )::bigint AS unique_artists,
                MAX(last_synced) AS last_synced
            FROM user_library_tracks
            WHERE user_id = $1
              AND provider = $2
              AND playlist_name IS NOT NULL
              AND TRIM(playlist_name) <> ''
            GROUP BY provider, playlist_name
            "#,
        )
        .bind(user.id)
        .bind(prov)
        .fetch_all(&state.db_pool)
        .await
        .map_err(AppError::DatabaseQueryFailed)?
    } else {
        sqlx::query_as::<_, PlaylistRow>(
            r#"
            SELECT
                provider,
                playlist_name,
                COUNT(*)::bigint AS total_tracks,
                COUNT(DISTINCT LOWER(artist_name)) FILTER (
                    WHERE artist_name IS NOT NULL AND TRIM(artist_name) <> ''
                )::bigint AS unique_artists,
                MAX(last_synced) AS last_synced
            FROM user_library_tracks
            WHERE user_id = $1
              AND playlist_name IS NOT NULL
              AND TRIM(playlist_name) <> ''
            GROUP BY provider, playlist_name
            "#,
        )
        .bind(user.id)
        .fetch_all(&state.db_pool)
        .await
        .map_err(AppError::DatabaseQueryFailed)?
    };

    // Build flagged info per playlist
    let mut summaries: Vec<PlaylistSummary> = Vec::with_capacity(playlists.len());

    for p in playlists {
        #[derive(sqlx::FromRow)]
        struct FlaggedInfo {
            flagged_count: i64,
        }

        let flagged: FlaggedInfo = sqlx::query_as(
            r#"
            SELECT COUNT(*)::bigint AS flagged_count
            FROM user_library_tracks ult
            WHERE ult.user_id = $1
              AND ult.provider = $2
              AND ult.playlist_name = $3
              AND (
                  EXISTS (
                      SELECT 1 FROM artists a
                      JOIN artist_offenses ao ON ao.artist_id = a.id AND ao.status = 'verified'
                      WHERE a.id = ult.artist_id
                         OR (ult.artist_name IS NOT NULL AND LOWER(ult.artist_name) = LOWER(a.canonical_name))
                  )
                  OR EXISTS (
                      SELECT 1 FROM user_artist_blocks uab
                      WHERE uab.user_id = ult.user_id AND uab.artist_id = ult.artist_id
                  )
              )
            "#,
        )
        .bind(user.id)
        .bind(&p.provider)
        .bind(&p.playlist_name)
        .fetch_one(&state.db_pool)
        .await
        .map_err(AppError::DatabaseQueryFailed)?;

        let flagged_artists: Vec<String> = sqlx::query_scalar(
            r#"
            SELECT DISTINCT ult.artist_name
            FROM user_library_tracks ult
            WHERE ult.user_id = $1
              AND ult.provider = $2
              AND ult.playlist_name = $3
              AND ult.artist_name IS NOT NULL
              AND TRIM(ult.artist_name) <> ''
              AND (
                  EXISTS (
                      SELECT 1 FROM artists a
                      JOIN artist_offenses ao ON ao.artist_id = a.id AND ao.status = 'verified'
                      WHERE a.id = ult.artist_id
                         OR LOWER(ult.artist_name) = LOWER(a.canonical_name)
                  )
                  OR EXISTS (
                      SELECT 1 FROM user_artist_blocks uab
                      WHERE uab.user_id = ult.user_id AND uab.artist_id = ult.artist_id
                  )
              )
            "#,
        )
        .bind(user.id)
        .bind(&p.provider)
        .bind(&p.playlist_name)
        .fetch_all(&state.db_pool)
        .await
        .map_err(AppError::DatabaseQueryFailed)?;

        // Grab up to 4 unique artist images for a mosaic cover
        let cover_images: Vec<String> = sqlx::query_scalar(
            r#"
            SELECT DISTINCT a.metadata->>'image_url'
            FROM user_library_tracks ult
            JOIN artists a ON a.id = ult.artist_id
            WHERE ult.user_id = $1
              AND ult.provider = $2
              AND ult.playlist_name = $3
              AND a.metadata->>'image_url' IS NOT NULL
              AND a.metadata->>'image_url' <> ''
            LIMIT 4
            "#,
        )
        .bind(user.id)
        .bind(&p.provider)
        .bind(&p.playlist_name)
        .fetch_all(&state.db_pool)
        .await
        .map_err(AppError::DatabaseQueryFailed)?;

        let clean_ratio = if p.total_tracks > 0 {
            (p.total_tracks - flagged.flagged_count) as f64 / p.total_tracks as f64
        } else {
            1.0
        };

        summaries.push(PlaylistSummary {
            provider: p.provider,
            playlist_name: p.playlist_name,
            total_tracks: p.total_tracks,
            flagged_tracks: flagged.flagged_count,
            clean_ratio,
            grade: grade_from_ratio(clean_ratio),
            unique_artists: p.unique_artists,
            flagged_artists,
            last_synced: p.last_synced,
            cover_images,
        });
    }

    // Sort: worst grade first, then alphabetically
    let grade_order = |g: &str| -> u8 {
        match g {
            "F" => 0,
            "D" => 1,
            "C" => 2,
            "B" => 3,
            "A" => 4,
            "A+" => 5,
            _ => 5,
        }
    };
    summaries.sort_by(|a, b| {
        grade_order(&a.grade)
            .cmp(&grade_order(&b.grade))
            .then(a.playlist_name.cmp(&b.playlist_name))
    });

    let total = summaries.len();
    Ok(Json(serde_json::json!({
        "playlists": summaries,
        "total": total
    })))
}

#[derive(Debug, Deserialize)]
pub struct PlaylistTracksQuery {
    pub provider: String,
    #[serde(alias = "playlist_name")]
    #[serde(rename = "playlistName")]
    pub playlist_name: String,
}

#[derive(Debug, Serialize)]
pub struct PlaylistTrackRow {
    pub id: Uuid,
    pub position: i64,
    pub provider_track_id: String,
    pub track_name: String,
    pub album_name: Option<String>,
    pub artist_id: Option<Uuid>,
    pub artist_name: String,
    pub artist_image_url: Option<String>,
    pub added_at: Option<DateTime<Utc>>,
    pub status: String,
}

/// Get all tracks in a specific playlist with per-track flagging status.
pub async fn get_playlist_tracks(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<PlaylistTracksQuery>,
) -> Result<Json<serde_json::Value>> {
    #[derive(sqlx::FromRow)]
    struct RawTrack {
        id: Uuid,
        provider_track_id: String,
        track_name: Option<String>,
        album_name: Option<String>,
        artist_id: Option<Uuid>,
        artist_name: Option<String>,
        artist_image_url: Option<String>,
        added_at: Option<DateTime<Utc>>,
        is_offending: bool,
        is_blocked: bool,
    }

    let rows = sqlx::query_as::<_, RawTrack>(
        r#"
        SELECT
            ult.id,
            ult.provider_track_id,
            ult.track_name,
            ult.album_name,
            ult.artist_id,
            ult.artist_name,
            a_img.metadata->>'image_url' AS artist_image_url,
            ult.added_at,
            EXISTS (
                SELECT 1 FROM artists a
                JOIN artist_offenses ao ON ao.artist_id = a.id AND ao.status = 'verified'
                WHERE a.id = ult.artist_id
                   OR (ult.artist_name IS NOT NULL AND LOWER(ult.artist_name) = LOWER(a.canonical_name))
            ) AS is_offending,
            EXISTS (
                SELECT 1 FROM user_artist_blocks uab
                WHERE uab.user_id = ult.user_id AND uab.artist_id = ult.artist_id
            ) AS is_blocked
        FROM user_library_tracks ult
        LEFT JOIN artists a_img ON a_img.id = ult.artist_id
        WHERE ult.user_id = $1
          AND ult.provider = $2
          AND ult.playlist_name = $3
        ORDER BY ult.added_at ASC NULLS LAST, ult.id
        "#,
    )
    .bind(user.id)
    .bind(&query.provider)
    .bind(&query.playlist_name)
    .fetch_all(&state.db_pool)
    .await
    .map_err(AppError::DatabaseQueryFailed)?;

    let tracks: Vec<PlaylistTrackRow> = rows
        .into_iter()
        .enumerate()
        .map(|(i, r)| {
            let status = if r.is_blocked {
                "blocked"
            } else if r.is_offending {
                "flagged"
            } else {
                "clean"
            };
            PlaylistTrackRow {
                id: r.id,
                position: (i + 1) as i64,
                provider_track_id: r.provider_track_id,
                artist_image_url: r.artist_image_url,
                track_name: r.track_name.unwrap_or_else(|| "Unknown".to_string()),
                album_name: r.album_name,
                artist_id: r.artist_id,
                artist_name: r.artist_name.unwrap_or_else(|| "Unknown".to_string()),
                added_at: r.added_at,
                status: status.to_string(),
            }
        })
        .collect();

    let total = tracks.len();
    Ok(Json(serde_json::json!({
        "tracks": tracks,
        "total": total
    })))
}
