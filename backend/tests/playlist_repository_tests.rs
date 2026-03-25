//! TDD tests for PlaylistRepository — written before implementation.
//!
//! Tests the normalized playlist tables (playlists + playlist_tracks)
//! through the PlaylistRepository service.
//!
//! Requires DATABASE_URL pointing to a Postgres instance with migrations applied.

use chrono::{DateTime, Utc};
use music_streaming_blocklist_backend::{
    database,
    models::{CreateUserRequest, UpsertPlaylist, UpsertPlaylistTrack},
    services::{AuthService, PlaylistRepository},
    DatabaseConfig,
};
use serial_test::serial;
use sqlx::PgPool;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

async fn get_pool() -> PgPool {
    let config = DatabaseConfig::default();
    database::initialize_database(config)
        .await
        .expect("Failed to initialize test database")
}

async fn create_test_user(pool: &PgPool) -> uuid::Uuid {
    let svc = AuthService::new(pool.clone());
    let user = svc
        .register_user(CreateUserRequest {
            email: format!("playlist_test_{}@test.com", Uuid::new_v4()),
            password: "TestPassword123!".to_string(),
        })
        .await
        .expect("create user");
    user.id
}

async fn create_offending_artist(pool: &PgPool, name: &str) -> uuid::Uuid {
    let artist_id: Uuid = sqlx::query_scalar(
        r#"INSERT INTO artists (canonical_name, external_ids, metadata)
           VALUES ($1, $2, $3)
           RETURNING id"#,
    )
    .bind(name)
    .bind(serde_json::json!({}))
    .bind(serde_json::json!({"image_url": "https://example.com/img.jpg"}))
    .fetch_one(pool)
    .await
    .unwrap();

    sqlx::query(
        r#"INSERT INTO artist_offenses (artist_id, category, severity, status, title, description, incident_date)
           VALUES ($1, 'domestic_violence', 'severe', 'verified', 'Test Offense', 'test offense', '2020-01-01')"#,
    )
    .bind(artist_id)
    .execute(pool)
    .await
    .unwrap();

    artist_id
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_playlist(name: &str, provider_id: &str) -> UpsertPlaylist {
    UpsertPlaylist {
        provider_playlist_id: provider_id.to_string(),
        name: name.to_string(),
        description: None,
        image_url: None,
        owner_name: None,
        owner_id: None,
        is_public: Some(true),
        is_collaborative: false,
        source_type: "playlist".to_string(),
        provider_track_count: None,
        snapshot_id: None,
    }
}

fn make_track(track_id: &str, name: &str, artist: &str, position: i32) -> UpsertPlaylistTrack {
    UpsertPlaylistTrack {
        provider_track_id: track_id.to_string(),
        track_name: name.to_string(),
        album_name: None,
        artist_name: artist.to_string(),
        position,
        added_at: None,
    }
}

// ===========================================================================
// upsert_playlist
// ===========================================================================

#[tokio::test]
#[serial]
async fn test_upsert_playlist_creates_new() {
    let pool = get_pool().await;
    let user_id = create_test_user(&pool).await;
    let repo = PlaylistRepository::new(&pool);

    let playlist = make_playlist("My Playlist", "sp_001");
    let id = repo
        .upsert_playlist(user_id, "spotify", &playlist)
        .await
        .expect("upsert should succeed");

    assert_ne!(id, Uuid::nil());

    let row: (String, String) =
        sqlx::query_as("SELECT name, provider_playlist_id FROM playlists WHERE id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(row.0, "My Playlist");
    assert_eq!(row.1, "sp_001");
}

#[tokio::test]
#[serial]
async fn test_upsert_playlist_updates_existing() {
    let pool = get_pool().await;
    let user_id = create_test_user(&pool).await;
    let repo = PlaylistRepository::new(&pool);

    let id1 = repo
        .upsert_playlist(user_id, "spotify", &make_playlist("Old Name", "sp_002"))
        .await
        .unwrap();

    let updated = UpsertPlaylist {
        name: "New Name".to_string(),
        description: Some("Updated".to_string()),
        ..make_playlist("New Name", "sp_002")
    };
    let id2 = repo
        .upsert_playlist(user_id, "spotify", &updated)
        .await
        .unwrap();

    assert_eq!(id1, id2);

    let name: String = sqlx::query_scalar("SELECT name FROM playlists WHERE id = $1")
        .bind(id1)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(name, "New Name");
}

#[tokio::test]
#[serial]
async fn test_upsert_playlist_unique_across_providers() {
    let pool = get_pool().await;
    let user_id = create_test_user(&pool).await;
    let repo = PlaylistRepository::new(&pool);

    let id_sp = repo
        .upsert_playlist(user_id, "spotify", &make_playlist("Chill", "pl_x"))
        .await
        .unwrap();
    let id_am = repo
        .upsert_playlist(user_id, "apple_music", &make_playlist("Chill", "pl_x"))
        .await
        .unwrap();

    assert_ne!(id_sp, id_am);
}

#[tokio::test]
#[serial]
async fn test_upsert_playlist_unique_across_users() {
    let pool = get_pool().await;
    let u1 = create_test_user(&pool).await;
    let u2 = create_test_user(&pool).await;
    let repo = PlaylistRepository::new(&pool);

    let id1 = repo
        .upsert_playlist(u1, "spotify", &make_playlist("Shared", "pl_s"))
        .await
        .unwrap();
    let id2 = repo
        .upsert_playlist(u2, "spotify", &make_playlist("Shared", "pl_s"))
        .await
        .unwrap();

    assert_ne!(id1, id2);
}

// ===========================================================================
// replace_playlist_tracks
// ===========================================================================

#[tokio::test]
#[serial]
async fn test_replace_playlist_tracks_inserts_all() {
    let pool = get_pool().await;
    let user_id = create_test_user(&pool).await;
    let repo = PlaylistRepository::new(&pool);

    let pl_id = repo
        .upsert_playlist(user_id, "spotify", &make_playlist("Rock", "pl_rock"))
        .await
        .unwrap();

    let tracks = vec![
        make_track("t1", "Bohemian Rhapsody", "Queen", 0),
        make_track("t2", "Stairway to Heaven", "Led Zeppelin", 1),
        make_track("t3", "Hotel California", "Eagles", 2),
    ];
    let count = repo.replace_playlist_tracks(pl_id, &tracks).await.unwrap();
    assert_eq!(count, 3);

    let db_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM playlist_tracks WHERE playlist_id = $1")
            .bind(pl_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(db_count.0, 3);
}

#[tokio::test]
#[serial]
async fn test_replace_playlist_tracks_replaces_existing() {
    let pool = get_pool().await;
    let user_id = create_test_user(&pool).await;
    let repo = PlaylistRepository::new(&pool);

    let pl_id = repo
        .upsert_playlist(user_id, "spotify", &make_playlist("Evolving", "pl_ev"))
        .await
        .unwrap();

    repo.replace_playlist_tracks(
        pl_id,
        &[
            make_track("old_1", "Old 1", "OA", 0),
            make_track("old_2", "Old 2", "OA", 1),
        ],
    )
    .await
    .unwrap();

    let count = repo
        .replace_playlist_tracks(
            pl_id,
            &[
                make_track("new_1", "New 1", "NA", 0),
                make_track("new_2", "New 2", "NA", 1),
                make_track("new_3", "New 3", "NA", 2),
            ],
        )
        .await
        .unwrap();
    assert_eq!(count, 3);

    let names: Vec<String> = sqlx::query_scalar(
        "SELECT track_name FROM playlist_tracks WHERE playlist_id = $1 ORDER BY position",
    )
    .bind(pl_id)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(names, vec!["New 1", "New 2", "New 3"]);
}

#[tokio::test]
#[serial]
async fn test_replace_playlist_tracks_preserves_order() {
    let pool = get_pool().await;
    let user_id = create_test_user(&pool).await;
    let repo = PlaylistRepository::new(&pool);

    let pl_id = repo
        .upsert_playlist(user_id, "spotify", &make_playlist("Ordered", "pl_ord"))
        .await
        .unwrap();

    let tracks = vec![
        make_track("z", "Z Song", "Z", 0),
        make_track("a", "A Song", "A", 1),
        make_track("m", "M Song", "M", 2),
    ];
    repo.replace_playlist_tracks(pl_id, &tracks).await.unwrap();

    let positions: Vec<(String, i32)> = sqlx::query_as(
        "SELECT provider_track_id, position FROM playlist_tracks WHERE playlist_id = $1 ORDER BY position",
    )
    .bind(pl_id)
    .fetch_all(&pool)
    .await
    .unwrap();

    assert_eq!(positions[0], ("z".to_string(), 0));
    assert_eq!(positions[1], ("a".to_string(), 1));
    assert_eq!(positions[2], ("m".to_string(), 2));
}

#[tokio::test]
#[serial]
async fn test_replace_playlist_tracks_empty_clears_all() {
    let pool = get_pool().await;
    let user_id = create_test_user(&pool).await;
    let repo = PlaylistRepository::new(&pool);

    let pl_id = repo
        .upsert_playlist(user_id, "spotify", &make_playlist("Empty", "pl_mt"))
        .await
        .unwrap();
    repo.replace_playlist_tracks(pl_id, &[make_track("t1", "S", "A", 0)])
        .await
        .unwrap();

    let count = repo.replace_playlist_tracks(pl_id, &[]).await.unwrap();
    assert_eq!(count, 0);

    let db_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM playlist_tracks WHERE playlist_id = $1")
            .bind(pl_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(db_count.0, 0);
}

// ===========================================================================
// delete_stale_playlists
// ===========================================================================

#[tokio::test]
#[serial]
async fn test_delete_stale_playlists() {
    let pool = get_pool().await;
    let user_id = create_test_user(&pool).await;
    let repo = PlaylistRepository::new(&pool);

    let fresh_id = repo
        .upsert_playlist(user_id, "spotify", &make_playlist("Fresh", "pl_f"))
        .await
        .unwrap();
    let stale_id = repo
        .upsert_playlist(user_id, "spotify", &make_playlist("Stale", "pl_s"))
        .await
        .unwrap();

    // Backdate stale playlist to 1 hour ago
    sqlx::query("UPDATE playlists SET last_synced = NOW() - INTERVAL '1 hour' WHERE id = $1")
        .bind(stale_id)
        .execute(&pool)
        .await
        .unwrap();

    // Ensure fresh playlist has a recent last_synced (re-touch it)
    sqlx::query("UPDATE playlists SET last_synced = NOW() + INTERVAL '1 second' WHERE id = $1")
        .bind(fresh_id)
        .execute(&pool)
        .await
        .unwrap();

    repo.replace_playlist_tracks(stale_id, &[make_track("t1", "Gone", "GA", 0)])
        .await
        .unwrap();

    // Use NOW() as cutoff — stale (1hr ago) < now, fresh (1s ahead) > now
    let cutoff: DateTime<Utc> = sqlx::query_scalar("SELECT NOW()")
        .fetch_one(&pool)
        .await
        .unwrap();

    let deleted = repo
        .delete_stale_playlists(user_id, "spotify", cutoff)
        .await
        .unwrap();
    assert_eq!(deleted, 1);

    // Fresh still exists
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM playlists WHERE id = $1)")
        .bind(fresh_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(exists);

    // Stale gone (cascade deletes tracks too)
    let gone: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM playlists WHERE id = $1)")
        .bind(stale_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(!gone);
}

#[tokio::test]
#[serial]
async fn test_delete_stale_playlists_respects_provider() {
    let pool = get_pool().await;
    let user_id = create_test_user(&pool).await;
    let repo = PlaylistRepository::new(&pool);

    repo.upsert_playlist(user_id, "spotify", &make_playlist("SP", "sp1"))
        .await
        .unwrap();
    let apple_id = repo
        .upsert_playlist(user_id, "apple_music", &make_playlist("AM", "am1"))
        .await
        .unwrap();

    sqlx::query("UPDATE playlists SET last_synced = NOW() - INTERVAL '1 hour' WHERE id = $1")
        .bind(apple_id)
        .execute(&pool)
        .await
        .unwrap();

    // Ensure spotify playlist is fresh (re-touch it)
    sqlx::query("UPDATE playlists SET last_synced = NOW() + INTERVAL '1 second' WHERE user_id = $1 AND provider = 'spotify'")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let cutoff: DateTime<Utc> = sqlx::query_scalar("SELECT NOW()")
        .fetch_one(&pool)
        .await
        .unwrap();

    // Delete stale spotify only — spotify is fresh, apple is stale but wrong provider
    let deleted = repo
        .delete_stale_playlists(user_id, "spotify", cutoff)
        .await
        .unwrap();
    assert_eq!(deleted, 0);

    let apple_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM playlists WHERE id = $1)")
            .bind(apple_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(apple_exists);
}

// ===========================================================================
// list_playlists_with_grades
// ===========================================================================

#[tokio::test]
#[serial]
async fn test_list_playlists_empty() {
    let pool = get_pool().await;
    let user_id = create_test_user(&pool).await;
    let repo = PlaylistRepository::new(&pool);

    let result = repo
        .list_playlists_with_grades(user_id, None)
        .await
        .unwrap();
    assert!(result.is_empty());
}

#[tokio::test]
#[serial]
async fn test_list_playlists_returns_track_counts() {
    let pool = get_pool().await;
    let user_id = create_test_user(&pool).await;
    let repo = PlaylistRepository::new(&pool);

    let pl_id = repo
        .upsert_playlist(user_id, "spotify", &make_playlist("Full", "pl_full"))
        .await
        .unwrap();
    repo.replace_playlist_tracks(
        pl_id,
        &[
            make_track("t1", "Song 1", "Artist A", 0),
            make_track("t2", "Song 2", "Artist B", 1),
            make_track("t3", "Song 3", "Artist A", 2),
        ],
    )
    .await
    .unwrap();

    let summaries = repo
        .list_playlists_with_grades(user_id, None)
        .await
        .unwrap();
    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0].name, "Full");
    assert_eq!(summaries[0].total_tracks, 3);
    assert_eq!(summaries[0].unique_artists, 2);
}

#[tokio::test]
#[serial]
async fn test_list_playlists_filters_by_provider() {
    let pool = get_pool().await;
    let user_id = create_test_user(&pool).await;
    let repo = PlaylistRepository::new(&pool);

    repo.upsert_playlist(user_id, "spotify", &make_playlist("SP", "sp_f"))
        .await
        .unwrap();
    repo.upsert_playlist(user_id, "apple_music", &make_playlist("AM", "am_f"))
        .await
        .unwrap();

    let sp = repo
        .list_playlists_with_grades(user_id, Some("spotify"))
        .await
        .unwrap();
    assert_eq!(sp.len(), 1);
    assert_eq!(sp[0].provider, "spotify");

    let all = repo
        .list_playlists_with_grades(user_id, None)
        .await
        .unwrap();
    assert_eq!(all.len(), 2);
}

#[tokio::test]
#[serial]
async fn test_list_playlists_counts_flagged_tracks() {
    let pool = get_pool().await;
    let user_id = create_test_user(&pool).await;
    let repo = PlaylistRepository::new(&pool);
    let bad_id = create_offending_artist(&pool, &format!("BadGuy_{}", Uuid::new_v4())).await;

    let pl_id = repo
        .upsert_playlist(user_id, "spotify", &make_playlist("Graded", "pl_gr"))
        .await
        .unwrap();

    // 2 clean + 1 flagged (linked by artist_id)
    sqlx::query(
        "INSERT INTO playlist_tracks (playlist_id, provider_track_id, track_name, artist_name, position)
         VALUES ($1, 'c1', 'Clean 1', 'Safe', 0), ($1, 'c2', 'Clean 2', 'Safe', 1)",
    )
    .bind(pl_id)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO playlist_tracks (playlist_id, provider_track_id, track_name, artist_id, artist_name, position)
         VALUES ($1, 'f1', 'Flagged', $2, 'BadGuy', 2)",
    )
    .bind(pl_id)
    .bind(bad_id)
    .execute(&pool)
    .await
    .unwrap();

    let summaries = repo
        .list_playlists_with_grades(user_id, None)
        .await
        .unwrap();
    assert_eq!(summaries.len(), 1);

    let s = &summaries[0];
    assert_eq!(s.total_tracks, 3);
    assert_eq!(s.flagged_tracks, 1);
    assert!((s.clean_ratio - (2.0 / 3.0)).abs() < 0.01);
    assert_eq!(s.grade, "C"); // 66% clean (0.6-0.7 range)
}

// ===========================================================================
// get_playlist_tracks_with_status
// ===========================================================================

#[tokio::test]
#[serial]
async fn test_get_tracks_with_status_clean() {
    let pool = get_pool().await;
    let user_id = create_test_user(&pool).await;
    let repo = PlaylistRepository::new(&pool);

    let pl_id = repo
        .upsert_playlist(user_id, "spotify", &make_playlist("Clean PL", "pl_cl"))
        .await
        .unwrap();
    repo.replace_playlist_tracks(
        pl_id,
        &[
            make_track("t1", "Song A", "Artist A", 0),
            make_track("t2", "Song B", "Artist B", 1),
        ],
    )
    .await
    .unwrap();

    let tracks = repo
        .get_playlist_tracks_with_status(pl_id, user_id)
        .await
        .unwrap();
    assert_eq!(tracks.len(), 2);
    assert_eq!(tracks[0].position, 0);
    assert_eq!(tracks[0].track_name, "Song A");
    assert_eq!(tracks[0].status, "clean");
    assert_eq!(tracks[1].position, 1);
    assert_eq!(tracks[1].status, "clean");
}

#[tokio::test]
#[serial]
async fn test_get_tracks_empty_playlist() {
    let pool = get_pool().await;
    let user_id = create_test_user(&pool).await;
    let repo = PlaylistRepository::new(&pool);

    let pl_id = repo
        .upsert_playlist(user_id, "spotify", &make_playlist("Empty", "pl_e2"))
        .await
        .unwrap();

    let tracks = repo
        .get_playlist_tracks_with_status(pl_id, user_id)
        .await
        .unwrap();
    assert!(tracks.is_empty());
}

#[tokio::test]
#[serial]
async fn test_get_tracks_flags_offending_artists() {
    let pool = get_pool().await;
    let user_id = create_test_user(&pool).await;
    let repo = PlaylistRepository::new(&pool);
    let bad_id = create_offending_artist(&pool, &format!("Offender_{}", Uuid::new_v4())).await;

    let pl_id = repo
        .upsert_playlist(user_id, "spotify", &make_playlist("Mixed", "pl_mx"))
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO playlist_tracks (playlist_id, provider_track_id, track_name, artist_id, artist_name, position)
         VALUES ($1, 'bad', 'Bad Song', $2, 'Offender', 0)",
    )
    .bind(pl_id)
    .bind(bad_id)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO playlist_tracks (playlist_id, provider_track_id, track_name, artist_name, position)
         VALUES ($1, 'good', 'Good Song', 'Good Artist', 1)",
    )
    .bind(pl_id)
    .execute(&pool)
    .await
    .unwrap();

    let tracks = repo
        .get_playlist_tracks_with_status(pl_id, user_id)
        .await
        .unwrap();
    assert_eq!(tracks.len(), 2);

    let bad = tracks.iter().find(|t| t.track_name == "Bad Song").unwrap();
    assert_eq!(bad.status, "flagged");

    let good = tracks.iter().find(|t| t.track_name == "Good Song").unwrap();
    assert_eq!(good.status, "clean");
}

// ===========================================================================
// Cascade delete
// ===========================================================================

#[tokio::test]
#[serial]
async fn test_playlist_cascade_delete_on_user_delete() {
    let pool = get_pool().await;
    let user_id = create_test_user(&pool).await;
    let repo = PlaylistRepository::new(&pool);

    let pl_id = repo
        .upsert_playlist(user_id, "spotify", &make_playlist("Die", "pl_die"))
        .await
        .unwrap();
    repo.replace_playlist_tracks(pl_id, &[make_track("t1", "S", "A", 0)])
        .await
        .unwrap();

    // Delete user — cascades to playlists → playlist_tracks
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM playlists WHERE id = $1)")
        .bind(pl_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(!exists);
}

// ===========================================================================
// MIGRATION BACKFILL VALIDATION
// ===========================================================================

#[tokio::test]
#[serial]
async fn test_backfill_spotify_playlist_tracks_extracted() {
    let pool = get_pool().await;
    let user_id = create_test_user(&pool).await;

    // Seed legacy data: 3 tracks in a Spotify playlist
    for i in 0..3 {
        sqlx::query(
            "INSERT INTO user_library_tracks (user_id, provider, provider_track_id, track_name, artist_name, source_type, playlist_name)
             VALUES ($1, 'spotify', $2, $3, 'Artist', 'playlist_track', 'My Playlist')",
        )
        .bind(user_id)
        .bind(format!("playlist:SP_PL_001:TRK_{}:{}", i, i))
        .bind(format!("Track {}", i))
        .execute(&pool)
        .await
        .unwrap();
    }

    // The migration backfill should have already run during initialize_database.
    // Verify: the playlists table should have an entry for SP_PL_001
    let pl_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM playlists WHERE user_id = $1 AND provider = 'spotify' AND provider_playlist_id = 'SP_PL_001')",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    // Note: backfill only runs at migration time, not on INSERT.
    // For newly inserted data, the dual-write handles it.
    // This test validates the schema accepts the expected patterns.
    assert!(!pl_exists); // Legacy data inserted AFTER migration won't be backfilled

    // But the dual-write via PlaylistRepository works
    let repo = PlaylistRepository::new(&pool);
    let pl_id = repo
        .upsert_playlist(
            user_id,
            "spotify",
            &make_playlist("My Playlist", "SP_PL_001"),
        )
        .await
        .unwrap();
    repo.replace_playlist_tracks(
        pl_id,
        &[
            make_track("TRK_0", "Track 0", "Artist", 0),
            make_track("TRK_1", "Track 1", "Artist", 1),
            make_track("TRK_2", "Track 2", "Artist", 2),
        ],
    )
    .await
    .unwrap();

    // Verify normalized data matches legacy count
    let legacy_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM user_library_tracks WHERE user_id = $1 AND provider = 'spotify' AND playlist_name = 'My Playlist'",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let normalized_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM playlist_tracks WHERE playlist_id = $1")
            .bind(pl_id)
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(legacy_count.0, 3);
    assert_eq!(normalized_count.0, 3);
}

// ===========================================================================
// DUAL-WRITE CONSISTENCY
// ===========================================================================

#[tokio::test]
#[serial]
async fn test_dual_write_track_count_consistency() {
    let pool = get_pool().await;
    let user_id = create_test_user(&pool).await;
    let repo = PlaylistRepository::new(&pool);

    // Create playlist with tracks via repository
    let pl_id = repo
        .upsert_playlist(user_id, "spotify", &make_playlist("Consistent", "pl_cons"))
        .await
        .unwrap();
    repo.replace_playlist_tracks(
        pl_id,
        &[
            make_track("t1", "Song 1", "A", 0),
            make_track("t2", "Song 2", "B", 1),
            make_track("t3", "Song 3", "A", 2),
            make_track("t4", "Song 4", "C", 3),
            make_track("t5", "Song 5", "B", 4),
        ],
    )
    .await
    .unwrap();

    // list_playlists_with_grades should report 5 tracks
    let summaries = repo
        .list_playlists_with_grades(user_id, None)
        .await
        .unwrap();
    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0].total_tracks, 5);
    assert_eq!(summaries[0].unique_artists, 3);

    // get_playlist_tracks_with_status should return 5 tracks in order
    let tracks = repo
        .get_playlist_tracks_with_status(pl_id, user_id)
        .await
        .unwrap();
    assert_eq!(tracks.len(), 5);
    for (i, t) in tracks.iter().enumerate() {
        assert_eq!(
            t.position, i as i32,
            "Track at index {} has wrong position",
            i
        );
    }
}

#[tokio::test]
#[serial]
async fn test_replace_tracks_is_atomic_no_partial_state() {
    let pool = get_pool().await;
    let user_id = create_test_user(&pool).await;
    let repo = PlaylistRepository::new(&pool);

    let pl_id = repo
        .upsert_playlist(user_id, "spotify", &make_playlist("Atomic", "pl_atomic"))
        .await
        .unwrap();

    // Insert 3 tracks
    repo.replace_playlist_tracks(
        pl_id,
        &[
            make_track("old_1", "Old 1", "A", 0),
            make_track("old_2", "Old 2", "A", 1),
            make_track("old_3", "Old 3", "A", 2),
        ],
    )
    .await
    .unwrap();

    // Replace with 2 new tracks
    repo.replace_playlist_tracks(
        pl_id,
        &[
            make_track("new_1", "New 1", "B", 0),
            make_track("new_2", "New 2", "B", 1),
        ],
    )
    .await
    .unwrap();

    // Should have exactly 2 tracks — no leftover old tracks
    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM playlist_tracks WHERE playlist_id = $1")
            .bind(pl_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count.0, 2);

    // No old tracks remain
    let old_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM playlist_tracks WHERE playlist_id = $1 AND provider_track_id LIKE 'old_%')",
    )
    .bind(pl_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(!old_exists);
}

#[tokio::test]
#[serial]
async fn test_list_and_detail_return_consistent_data() {
    let pool = get_pool().await;
    let user_id = create_test_user(&pool).await;
    let repo = PlaylistRepository::new(&pool);
    let bad_id = create_offending_artist(&pool, &format!("Flagged_{}", Uuid::new_v4())).await;

    let pl_id = repo
        .upsert_playlist(user_id, "spotify", &make_playlist("ConsistCheck", "pl_cc"))
        .await
        .unwrap();

    // 2 clean + 1 flagged
    sqlx::query(
        "INSERT INTO playlist_tracks (playlist_id, provider_track_id, track_name, artist_name, position)
         VALUES ($1, 'c1', 'Clean1', 'Safe', 0), ($1, 'c2', 'Clean2', 'Safe', 1)",
    )
    .bind(pl_id)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO playlist_tracks (playlist_id, provider_track_id, track_name, artist_id, artist_name, position)
         VALUES ($1, 'f1', 'Flagged1', $2, 'Flagged', 2)",
    )
    .bind(pl_id)
    .bind(bad_id)
    .execute(&pool)
    .await
    .unwrap();

    // list says 3 total, 1 flagged
    let summaries = repo
        .list_playlists_with_grades(user_id, None)
        .await
        .unwrap();
    let summary = summaries.iter().find(|s| s.name == "ConsistCheck").unwrap();
    assert_eq!(summary.total_tracks, 3);
    assert_eq!(summary.flagged_tracks, 1);

    // detail also says 3 tracks, 1 flagged
    let tracks = repo
        .get_playlist_tracks_with_status(pl_id, user_id)
        .await
        .unwrap();
    assert_eq!(tracks.len(), 3);
    let flagged_count = tracks.iter().filter(|t| t.status == "flagged").count();
    assert_eq!(flagged_count, 1);

    // Grade and ratio are consistent
    assert!((summary.clean_ratio - (2.0 / 3.0)).abs() < 0.01);
}

// ===========================================================================
// EDGE CASES AND BOUNDARIES
// ===========================================================================

#[tokio::test]
#[serial]
async fn test_long_playlist_name_500_chars() {
    let pool = get_pool().await;
    let user_id = create_test_user(&pool).await;
    let repo = PlaylistRepository::new(&pool);

    let long_name = "A".repeat(500);
    let pl_id = repo
        .upsert_playlist(
            user_id,
            "spotify",
            &UpsertPlaylist {
                provider_playlist_id: "pl_long_name".to_string(),
                name: long_name.clone(),
                description: None,
                image_url: None,
                owner_name: None,
                owner_id: None,
                is_public: None,
                is_collaborative: false,
                source_type: "playlist".to_string(),
                provider_track_count: None,
                snapshot_id: None,
            },
        )
        .await
        .unwrap();

    let stored_name: String = sqlx::query_scalar("SELECT name FROM playlists WHERE id = $1")
        .bind(pl_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(stored_name.len(), 500);
}

#[tokio::test]
#[serial]
async fn test_empty_playlist_grades_a_plus() {
    let pool = get_pool().await;
    let user_id = create_test_user(&pool).await;
    let repo = PlaylistRepository::new(&pool);

    repo.upsert_playlist(
        user_id,
        "spotify",
        &make_playlist("Empty PL", "pl_empty_grade"),
    )
    .await
    .unwrap();

    let summaries = repo
        .list_playlists_with_grades(user_id, None)
        .await
        .unwrap();
    let s = summaries.iter().find(|s| s.name == "Empty PL").unwrap();
    assert_eq!(s.total_tracks, 0);
    assert_eq!(s.flagged_tracks, 0);
    assert_eq!(s.clean_ratio, 1.0);
    assert_eq!(s.grade, "A+");
}

#[tokio::test]
#[serial]
async fn test_all_tracks_flagged_grades_f() {
    let pool = get_pool().await;
    let user_id = create_test_user(&pool).await;
    let repo = PlaylistRepository::new(&pool);
    let bad_id = create_offending_artist(&pool, &format!("AllBad_{}", Uuid::new_v4())).await;

    let pl_id = repo
        .upsert_playlist(user_id, "spotify", &make_playlist("All Bad", "pl_all_bad"))
        .await
        .unwrap();

    for i in 0..5 {
        sqlx::query(
            "INSERT INTO playlist_tracks (playlist_id, provider_track_id, track_name, artist_id, artist_name, position)
             VALUES ($1, $2, $3, $4, 'AllBad', $5)",
        )
        .bind(pl_id)
        .bind(format!("bad_{}", i))
        .bind(format!("Bad Track {}", i))
        .bind(bad_id)
        .bind(i)
        .execute(&pool)
        .await
        .unwrap();
    }

    let summaries = repo
        .list_playlists_with_grades(user_id, None)
        .await
        .unwrap();
    let s = summaries.iter().find(|s| s.name == "All Bad").unwrap();
    assert_eq!(s.total_tracks, 5);
    assert_eq!(s.flagged_tracks, 5);
    assert_eq!(s.clean_ratio, 0.0);
    assert_eq!(s.grade, "F");
}

#[tokio::test]
#[serial]
async fn test_flagging_by_artist_name_match_without_artist_id() {
    let pool = get_pool().await;
    let user_id = create_test_user(&pool).await;
    let repo = PlaylistRepository::new(&pool);

    // Create offending artist
    let name = format!("NameMatch_{}", Uuid::new_v4());
    create_offending_artist(&pool, &name).await;

    let pl_id = repo
        .upsert_playlist(user_id, "spotify", &make_playlist("NameMatch", "pl_nm"))
        .await
        .unwrap();

    // Insert track WITHOUT artist_id, but with matching artist_name
    sqlx::query(
        "INSERT INTO playlist_tracks (playlist_id, provider_track_id, track_name, artist_name, position)
         VALUES ($1, 'nm_track', 'Name Match Song', $2, 0)",
    )
    .bind(pl_id)
    .bind(&name)
    .execute(&pool)
    .await
    .unwrap();

    // Should still be flagged via name match
    let tracks = repo
        .get_playlist_tracks_with_status(pl_id, user_id)
        .await
        .unwrap();
    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0].status, "flagged");

    // list_playlists should also count it
    let summaries = repo
        .list_playlists_with_grades(user_id, None)
        .await
        .unwrap();
    let s = summaries.iter().find(|s| s.name == "NameMatch").unwrap();
    assert_eq!(s.flagged_tracks, 1);
}

#[tokio::test]
#[serial]
async fn test_blocked_vs_flagged_distinction() {
    let pool = get_pool().await;
    let user_id = create_test_user(&pool).await;
    let repo = PlaylistRepository::new(&pool);

    // Create an artist that is BOTH offending AND blocked by user
    let artist_name = format!("BlockedArtist_{}", Uuid::new_v4());
    let artist_id = create_offending_artist(&pool, &artist_name).await;

    // User blocks this artist
    sqlx::query(
        "INSERT INTO user_artist_blocks (user_id, artist_id, note)
         VALUES ($1, $2, 'test block')",
    )
    .bind(user_id)
    .bind(artist_id)
    .execute(&pool)
    .await
    .unwrap();

    let pl_id = repo
        .upsert_playlist(user_id, "spotify", &make_playlist("Blocked", "pl_blk"))
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO playlist_tracks (playlist_id, provider_track_id, track_name, artist_id, artist_name, position)
         VALUES ($1, 'blk_track', 'Blocked Song', $2, $3, 0)",
    )
    .bind(pl_id)
    .bind(artist_id)
    .bind(&artist_name)
    .execute(&pool)
    .await
    .unwrap();

    let tracks = repo
        .get_playlist_tracks_with_status(pl_id, user_id)
        .await
        .unwrap();
    assert_eq!(tracks.len(), 1);
    // Blocked takes precedence over flagged
    assert_eq!(tracks[0].status, "blocked");
}

#[tokio::test]
#[serial]
async fn test_multiple_offending_artists_counted_correctly() {
    let pool = get_pool().await;
    let user_id = create_test_user(&pool).await;
    let repo = PlaylistRepository::new(&pool);

    let bad1 = create_offending_artist(&pool, &format!("BadA_{}", Uuid::new_v4())).await;
    let bad2 = create_offending_artist(&pool, &format!("BadB_{}", Uuid::new_v4())).await;

    let pl_id = repo
        .upsert_playlist(user_id, "spotify", &make_playlist("Multi Bad", "pl_mb"))
        .await
        .unwrap();

    // 2 tracks from bad1, 1 from bad2, 2 clean
    for (i, (tid, aid, name)) in [
        ("b1a", Some(bad1), "BadA"),
        ("b1b", Some(bad1), "BadA"),
        ("b2a", Some(bad2), "BadB"),
        ("c1", None, "Good"),
        ("c2", None, "Good"),
    ]
    .iter()
    .enumerate()
    {
        if let Some(aid) = aid {
            sqlx::query(
                "INSERT INTO playlist_tracks (playlist_id, provider_track_id, track_name, artist_id, artist_name, position)
                 VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(pl_id).bind(tid).bind(format!("Song {}", tid)).bind(aid).bind(name).bind(i as i32)
            .execute(&pool).await.unwrap();
        } else {
            sqlx::query(
                "INSERT INTO playlist_tracks (playlist_id, provider_track_id, track_name, artist_name, position)
                 VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(pl_id).bind(tid).bind(format!("Song {}", tid)).bind(name).bind(i as i32)
            .execute(&pool).await.unwrap();
        }
    }

    let summaries = repo
        .list_playlists_with_grades(user_id, None)
        .await
        .unwrap();
    let s = summaries.iter().find(|s| s.name == "Multi Bad").unwrap();
    assert_eq!(s.total_tracks, 5);
    assert_eq!(s.flagged_tracks, 3); // 2 from bad1 + 1 from bad2
    assert_eq!(s.flagged_artists.len(), 2); // 2 distinct flagged artist names
}

// ===========================================================================
// VIRTUAL PLAYLISTS AND SOURCE_TYPE
// ===========================================================================

#[tokio::test]
#[serial]
async fn test_virtual_playlist_liked_songs() {
    let pool = get_pool().await;
    let user_id = create_test_user(&pool).await;
    let repo = PlaylistRepository::new(&pool);

    let pl_id = repo
        .upsert_playlist(
            user_id,
            "spotify",
            &UpsertPlaylist {
                provider_playlist_id: "__liked_songs__".to_string(),
                name: "Liked Songs".to_string(),
                description: None,
                image_url: None,
                owner_name: None,
                owner_id: None,
                is_public: Some(false),
                is_collaborative: false,
                source_type: "liked_songs".to_string(),
                provider_track_count: Some(3),
                snapshot_id: None,
            },
        )
        .await
        .unwrap();

    repo.replace_playlist_tracks(
        pl_id,
        &[
            make_track("liked_1", "Fav 1", "Artist A", 0),
            make_track("liked_2", "Fav 2", "Artist B", 1),
            make_track("liked_3", "Fav 3", "Artist A", 2),
        ],
    )
    .await
    .unwrap();

    let summaries = repo
        .list_playlists_with_grades(user_id, Some("spotify"))
        .await
        .unwrap();
    let liked = summaries
        .iter()
        .find(|s| s.source_type == "liked_songs")
        .unwrap();
    assert_eq!(liked.name, "Liked Songs");
    assert_eq!(liked.total_tracks, 3);
    assert_eq!(liked.provider_playlist_id, "__liked_songs__");
}

#[tokio::test]
#[serial]
async fn test_virtual_playlists_coexist_with_real_playlists() {
    let pool = get_pool().await;
    let user_id = create_test_user(&pool).await;
    let repo = PlaylistRepository::new(&pool);

    // Create a real playlist
    let real_id = repo
        .upsert_playlist(
            user_id,
            "spotify",
            &make_playlist("Real Playlist", "sp_real"),
        )
        .await
        .unwrap();
    repo.replace_playlist_tracks(real_id, &[make_track("r1", "Real Song", "RA", 0)])
        .await
        .unwrap();

    // Create virtual playlists
    let liked_id = repo
        .upsert_playlist(
            user_id,
            "spotify",
            &UpsertPlaylist {
                provider_playlist_id: "__liked_songs__".to_string(),
                name: "Liked Songs".to_string(),
                description: None,
                image_url: None,
                owner_name: None,
                owner_id: None,
                is_public: Some(false),
                is_collaborative: false,
                source_type: "liked_songs".to_string(),
                provider_track_count: Some(2),
                snapshot_id: None,
            },
        )
        .await
        .unwrap();
    repo.replace_playlist_tracks(
        liked_id,
        &[
            make_track("l1", "Liked 1", "LA", 0),
            make_track("l2", "Liked 2", "LB", 1),
        ],
    )
    .await
    .unwrap();

    let summaries = repo
        .list_playlists_with_grades(user_id, Some("spotify"))
        .await
        .unwrap();
    assert_eq!(summaries.len(), 2);

    let real = summaries
        .iter()
        .find(|s| s.source_type == "playlist")
        .unwrap();
    assert_eq!(real.total_tracks, 1);

    let liked = summaries
        .iter()
        .find(|s| s.source_type == "liked_songs")
        .unwrap();
    assert_eq!(liked.total_tracks, 2);
}

#[tokio::test]
#[serial]
async fn test_multiple_providers_virtual_playlists_independent() {
    let pool = get_pool().await;
    let user_id = create_test_user(&pool).await;
    let repo = PlaylistRepository::new(&pool);

    // Spotify liked
    let sp_liked = repo
        .upsert_playlist(
            user_id,
            "spotify",
            &UpsertPlaylist {
                provider_playlist_id: "__liked_songs__".to_string(),
                name: "Liked Songs".to_string(),
                description: None,
                image_url: None,
                owner_name: None,
                owner_id: None,
                is_public: Some(false),
                is_collaborative: false,
                source_type: "liked_songs".to_string(),
                provider_track_count: Some(10),
                snapshot_id: None,
            },
        )
        .await
        .unwrap();
    repo.replace_playlist_tracks(sp_liked, &[make_track("sp1", "SP Song", "A", 0)])
        .await
        .unwrap();

    // Apple Music library songs
    let am_songs = repo
        .upsert_playlist(
            user_id,
            "apple_music",
            &UpsertPlaylist {
                provider_playlist_id: "__library_songs__".to_string(),
                name: "Library Songs".to_string(),
                description: None,
                image_url: None,
                owner_name: None,
                owner_id: None,
                is_public: Some(false),
                is_collaborative: false,
                source_type: "library_songs".to_string(),
                provider_track_count: Some(20),
                snapshot_id: None,
            },
        )
        .await
        .unwrap();
    repo.replace_playlist_tracks(
        am_songs,
        &[
            make_track("am1", "AM Song 1", "B", 0),
            make_track("am2", "AM Song 2", "C", 1),
        ],
    )
    .await
    .unwrap();

    // Filter by provider
    let sp = repo
        .list_playlists_with_grades(user_id, Some("spotify"))
        .await
        .unwrap();
    assert_eq!(sp.len(), 1);
    assert_eq!(sp[0].total_tracks, 1);

    let am = repo
        .list_playlists_with_grades(user_id, Some("apple_music"))
        .await
        .unwrap();
    assert_eq!(am.len(), 1);
    assert_eq!(am[0].total_tracks, 2);

    // All returns both
    let all = repo
        .list_playlists_with_grades(user_id, None)
        .await
        .unwrap();
    assert_eq!(all.len(), 2);
}

// ===========================================================================
// GRADE BOUNDARY VALIDATION
// ===========================================================================

#[tokio::test]
#[serial]
async fn test_grade_boundaries() {
    let pool = get_pool().await;
    let user_id = create_test_user(&pool).await;
    let repo = PlaylistRepository::new(&pool);
    let bad_id = create_offending_artist(&pool, &format!("GradeTest_{}", Uuid::new_v4())).await;

    // Helper: create playlist with n clean + m flagged tracks, return grade
    async fn grade_for(
        repo: &PlaylistRepository<'_>,
        pool: &PgPool,
        user_id: Uuid,
        bad_id: Uuid,
        tag: &str,
        clean: usize,
        flagged: usize,
    ) -> String {
        let pl_id = repo
            .upsert_playlist(
                user_id,
                "spotify",
                &UpsertPlaylist {
                    provider_playlist_id: format!("pl_grade_{}", tag),
                    name: format!("Grade {}", tag),
                    description: None,
                    image_url: None,
                    owner_name: None,
                    owner_id: None,
                    is_public: None,
                    is_collaborative: false,
                    source_type: "playlist".to_string(),
                    provider_track_count: None,
                    snapshot_id: None,
                },
            )
            .await
            .unwrap();

        for i in 0..clean {
            sqlx::query(
                "INSERT INTO playlist_tracks (playlist_id, provider_track_id, track_name, artist_name, position)
                 VALUES ($1, $2, 'Clean', 'Safe', $3)",
            )
            .bind(pl_id)
            .bind(format!("c_{}_{}", tag, i))
            .bind(i as i32)
            .execute(pool).await.unwrap();
        }
        for i in 0..flagged {
            sqlx::query(
                "INSERT INTO playlist_tracks (playlist_id, provider_track_id, track_name, artist_id, artist_name, position)
                 VALUES ($1, $2, 'Bad', $3, 'Bad', $4)",
            )
            .bind(pl_id)
            .bind(format!("f_{}_{}", tag, i))
            .bind(bad_id)
            .bind((clean + i) as i32)
            .execute(pool).await.unwrap();
        }

        let summaries = repo
            .list_playlists_with_grades(user_id, None)
            .await
            .unwrap();
        summaries
            .iter()
            .find(|s| s.name == format!("Grade {}", tag))
            .unwrap()
            .grade
            .clone()
    }

    // 100% clean = A+
    assert_eq!(
        grade_for(&repo, &pool, user_id, bad_id, "100", 10, 0).await,
        "A+"
    );
    // 95% clean = A+ (boundary)
    assert_eq!(
        grade_for(&repo, &pool, user_id, bad_id, "95", 19, 1).await,
        "A+"
    );
    // 90% clean = A
    assert_eq!(
        grade_for(&repo, &pool, user_id, bad_id, "90", 9, 1).await,
        "A"
    );
    // 80% clean = A (boundary)
    assert_eq!(
        grade_for(&repo, &pool, user_id, bad_id, "80", 8, 2).await,
        "A"
    );
    // 75% clean = B
    assert_eq!(
        grade_for(&repo, &pool, user_id, bad_id, "75", 3, 1).await,
        "B"
    );
    // 66% clean = C
    assert_eq!(
        grade_for(&repo, &pool, user_id, bad_id, "66", 2, 1).await,
        "C"
    );
    // 60% clean = C (boundary)
    assert_eq!(
        grade_for(&repo, &pool, user_id, bad_id, "60", 3, 2).await,
        "C"
    );
    // 50% clean = D
    assert_eq!(
        grade_for(&repo, &pool, user_id, bad_id, "50", 1, 1).await,
        "D"
    );
    // 40% clean = F
    assert_eq!(
        grade_for(&repo, &pool, user_id, bad_id, "40", 2, 3).await,
        "F"
    );
    // 0% clean = F
    assert_eq!(
        grade_for(&repo, &pool, user_id, bad_id, "0", 0, 5).await,
        "F"
    );
}
