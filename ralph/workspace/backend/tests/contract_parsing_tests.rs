use ralph_backend::api_parsing::{
    parse_pagination, HeaderMap, PaginationInfo, RateLimitState,
};
use serde_json::json;

fn headers(pairs: &[(&str, &str)]) -> HeaderMap {
    pairs
        .iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

#[test]
fn rate_limit_parsing_updates_state() {
    let mut state = RateLimitState::default();
    let header_map = headers(&[
        ("X-RateLimit-Remaining", "0"),
        ("X-RateLimit-Reset", "1700000000"),
        ("Retry-After", "5"),
    ]);

    let updated = state.update_from_headers(&header_map);

    assert!(updated);
    assert_eq!(state.remaining, Some(0));
    assert_eq!(state.reset_epoch_seconds, Some(1700000000));
    assert_eq!(state.retry_after_seconds, Some(5));
    assert!(state.is_rate_limited);
}

#[test]
fn rate_limit_parsing_leaves_state_when_headers_missing() {
    let mut state = RateLimitState {
        remaining: Some(7),
        reset_epoch_seconds: Some(42),
        retry_after_seconds: None,
        is_rate_limited: false,
    };

    let updated = state.update_from_headers(&headers(&[]));

    assert!(!updated);
    assert_eq!(state.remaining, Some(7));
    assert_eq!(state.reset_epoch_seconds, Some(42));
    assert_eq!(state.retry_after_seconds, None);
    assert!(!state.is_rate_limited);
}

#[test]
fn pagination_parses_offset_format() {
    let response = json!({
        "items": [{"id": "1"}],
        "total": 100,
        "limit": 50,
        "offset": 0,
        "next": "https://api.spotify.com/v1/me/tracks?offset=50&limit=50",
        "previous": null
    });

    let pagination = parse_pagination(&response).expect("expected offset pagination");

    match pagination {
        PaginationInfo::Offset {
            total,
            limit,
            offset,
            next,
            previous,
        } => {
            assert_eq!(total, 100);
            assert_eq!(limit, 50);
            assert_eq!(offset, 0);
            assert_eq!(
                next,
                Some("https://api.spotify.com/v1/me/tracks?offset=50&limit=50".to_string())
            );
            assert_eq!(previous, None);
        }
        _ => panic!("expected offset pagination"),
    }
}

#[test]
fn pagination_parses_cursor_format() {
    let response = json!({
        "data": [{"id": "1"}],
        "paging": {
            "cursors": {"before": "cursor_before", "after": "cursor_after"},
            "next": "https://api.example.com/data?after=cursor_after"
        }
    });

    let pagination = parse_pagination(&response).expect("expected cursor pagination");

    match pagination {
        PaginationInfo::Cursor { after, before, next } => {
            assert_eq!(after, Some("cursor_after".to_string()));
            assert_eq!(before, Some("cursor_before".to_string()));
            assert_eq!(
                next,
                Some("https://api.example.com/data?after=cursor_after".to_string())
            );
        }
        _ => panic!("expected cursor pagination"),
    }
}
