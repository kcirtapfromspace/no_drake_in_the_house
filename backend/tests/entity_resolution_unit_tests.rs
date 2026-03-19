use music_streaming_blocklist_backend::models::{Artist, ExternalIds};
use music_streaming_blocklist_backend::services::{EntityResolutionService, ExternalApiService};
use sqlx::{postgres::PgPoolOptions, PgPool};

fn lazy_test_pool() -> PgPool {
    PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy("postgres://test:test@localhost:5432/test_db")
        .expect("lazy postgres pool should be constructible for smoke tests")
}

#[test]
fn external_api_service_smoke_constructs() {
    let _service = ExternalApiService::new();
}

#[tokio::test]
async fn entity_resolution_service_smoke_constructs() {
    let service = EntityResolutionService::new(lazy_test_pool());

    let result = service
        .resolve_artist("test artist", Some("spotify"))
        .await
        .expect("stub entity resolution should not fail");

    assert!(result.is_none());
}

#[test]
fn artist_model_external_ids_builder_still_works() {
    let artist = Artist::with_external_ids(
        "Smoke Artist".to_string(),
        ExternalIds::new().with_musicbrainz("test-mbid".to_string()),
    );

    assert_eq!(artist.canonical_name, "Smoke Artist");
    assert_eq!(
        artist.external_ids.musicbrainz.as_deref(),
        Some("test-mbid")
    );
}
