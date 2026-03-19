use chrono::Utc;
use music_streaming_blocklist_backend::models::{DnpListEntry, DnpListResponse, ProviderBadge};
use music_streaming_blocklist_backend::DnpListService;
use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;

fn lazy_test_pool() -> PgPool {
    PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy("postgres://test:test@localhost:5432/test_db")
        .expect("lazy postgres pool should be constructible for smoke tests")
}

#[tokio::test]
async fn dnp_service_constructs_for_router_integration_smoke() {
    let _service = DnpListService::new(lazy_test_pool());
}

#[test]
fn dnp_list_response_serializes_entries_and_tags() {
    let response = DnpListResponse {
        total: 1,
        tags: vec!["high-risk".to_string(), "manual-review".to_string()],
        entries: vec![DnpListEntry {
            artist_id: Uuid::new_v4(),
            artist_name: "Smoke Test Artist".to_string(),
            image_url: Some("https://example.com/artist.png".to_string()),
            provider_badges: vec![ProviderBadge {
                provider: "spotify".to_string(),
                verified: true,
                follower_count: Some(42),
            }],
            tags: vec!["high-risk".to_string()],
            note: Some("Imported from policy review".to_string()),
            added_at: Utc::now(),
        }],
    };

    let value = serde_json::to_value(&response).expect("response should serialize");

    assert_eq!(value["total"], 1);
    assert_eq!(value["entries"][0]["artist_name"], "Smoke Test Artist");
    assert_eq!(
        value["entries"][0]["provider_badges"][0]["provider"],
        "spotify"
    );
    assert_eq!(
        value["tags"],
        serde_json::json!(["high-risk", "manual-review"])
    );
}
