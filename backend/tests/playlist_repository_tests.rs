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

    let result = repo.list_playlists_with_grades(user_id, None).await.unwrap();
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

    let summaries = repo.list_playlists_with_grades(user_id, None).await.unwrap();
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

    let all = repo.list_playlists_with_grades(user_id, None).await.unwrap();
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

    let summaries = repo.list_playlists_with_grades(user_id, None).await.unwrap();
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
