use music_streaming_blocklist_backend::models::{
    AddToDnpRequest, BulkImportRequest, ImportFormat, UpdateDnpEntryRequest,
};
use serde_json::json;
use uuid::Uuid;

#[test]
fn add_to_dnp_request_round_trips_as_json() {
    let artist_id = Uuid::new_v4();
    let request = AddToDnpRequest {
        artist_id,
        tags: Some(vec!["safety".to_string(), "policy".to_string()]),
        note: Some("Block collaborations too".to_string()),
    };

    let value = serde_json::to_value(&request).expect("request should serialize");

    assert_eq!(value["artist_id"], artist_id.to_string());
    assert_eq!(value["tags"], json!(["safety", "policy"]));
    assert_eq!(value["note"], "Block collaborations too");
}

#[test]
fn update_dnp_entry_request_deserializes_partial_payloads() {
    let request: UpdateDnpEntryRequest = serde_json::from_value(json!({
        "note": "Escalated after review"
    }))
    .expect("partial update payload should deserialize");

    assert!(request.tags.is_none());
    assert_eq!(request.note.as_deref(), Some("Escalated after review"));
}

#[test]
fn bulk_import_request_deserializes_json_imports() {
    let request: BulkImportRequest = serde_json::from_value(json!({
        "format": "json",
        "data": "[{\"artist_name\":\"Example Artist\"}]",
        "overwrite_existing": true
    }))
    .expect("bulk import payload should deserialize");

    assert!(matches!(request.format, ImportFormat::Json));
    assert!(request.data.contains("Example Artist"));
    assert_eq!(request.overwrite_existing, Some(true));
}
