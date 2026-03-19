use music_streaming_blocklist_backend::models::{AddToDnpRequest, DnpEntry, UpdateDnpEntryRequest};
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
async fn dnp_service_smoke_constructs() {
    let _service = DnpListService::new(lazy_test_pool());
}

#[test]
fn add_to_dnp_request_keeps_optional_fields() {
    let artist_id = Uuid::new_v4();
    let request = AddToDnpRequest {
        artist_id,
        tags: Some(vec!["test".to_string(), "example".to_string()]),
        note: Some("Needs manual verification".to_string()),
    };

    assert_eq!(request.artist_id, artist_id);
    assert_eq!(
        request.tags.as_deref(),
        Some(["test".to_string(), "example".to_string()].as_slice())
    );
    assert_eq!(request.note.as_deref(), Some("Needs manual verification"));
}

#[test]
fn update_request_allows_tag_only_updates() {
    let request = UpdateDnpEntryRequest {
        tags: Some(vec!["updated".to_string()]),
        note: None,
    };

    assert_eq!(
        request.tags.as_deref(),
        Some(["updated".to_string()].as_slice())
    );
    assert!(request.note.is_none());
}

#[test]
fn dnp_entry_model_stores_basic_metadata() {
    let user_id = Uuid::new_v4();
    let artist_id = Uuid::new_v4();
    let entry = DnpEntry {
        user_id,
        artist_id,
        tags: Some(vec!["archived".to_string()]),
        note: Some("Legacy blocklist entry".to_string()),
        created_at: chrono::Utc::now(),
    };

    assert_eq!(entry.user_id, user_id);
    assert_eq!(entry.artist_id, artist_id);
    assert_eq!(
        entry.tags.as_deref(),
        Some(["archived".to_string()].as_slice())
    );
    assert_eq!(entry.note.as_deref(), Some("Legacy blocklist entry"));
}
