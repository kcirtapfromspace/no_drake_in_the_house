use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RateLimitInfo {
    pub remaining: Option<u32>,
    pub reset_epoch_seconds: Option<i64>,
    pub retry_after_seconds: Option<u64>,
    pub is_rate_limited: bool,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RateLimitState {
    pub remaining: Option<u32>,
    pub reset_epoch_seconds: Option<i64>,
    pub retry_after_seconds: Option<u64>,
    pub is_rate_limited: bool,
}

impl RateLimitState {
    pub fn update_from_headers(&mut self, headers: &HeaderMap) -> bool {
        let info = match parse_rate_limit_headers(headers) {
            Some(info) => info,
            None => return false,
        };

        self.remaining = info.remaining;
        self.reset_epoch_seconds = info.reset_epoch_seconds;
        self.retry_after_seconds = info.retry_after_seconds;
        self.is_rate_limited = info.is_rate_limited;
        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaginationInfo {
    Offset {
        total: u32,
        limit: u32,
        offset: u32,
        next: Option<String>,
        previous: Option<String>,
    },
    Cursor {
        after: Option<String>,
        before: Option<String>,
        next: Option<String>,
    },
}

pub type HeaderMap = HashMap<String, String>;

pub fn parse_rate_limit_headers(headers: &HeaderMap) -> Option<RateLimitInfo> {
    let remaining = header_value(headers, "x-ratelimit-remaining")
        .or_else(|| header_value(headers, "x-rate-limit-remaining"))
        .and_then(|value| value.parse::<u32>().ok());

    let reset_epoch_seconds = header_value(headers, "x-ratelimit-reset")
        .or_else(|| header_value(headers, "x-rate-limit-reset"))
        .and_then(|value| value.parse::<i64>().ok());

    let retry_after_seconds =
        header_value(headers, "retry-after").and_then(|value| value.parse::<u64>().ok());

    if remaining.is_none() && reset_epoch_seconds.is_none() && retry_after_seconds.is_none() {
        return None;
    }

    let is_rate_limited = remaining == Some(0) || retry_after_seconds.is_some();

    Some(RateLimitInfo {
        remaining,
        reset_epoch_seconds,
        retry_after_seconds,
        is_rate_limited,
    })
}

pub fn parse_pagination(value: &Value) -> Option<PaginationInfo> {
    parse_offset_pagination(value).or_else(|| parse_cursor_pagination(value))
}

fn parse_offset_pagination(value: &Value) -> Option<PaginationInfo> {
    let total = value.get("total")?.as_u64()? as u32;
    let limit = value.get("limit")?.as_u64()? as u32;
    let offset = value.get("offset")?.as_u64()? as u32;

    let next = value
        .get("next")
        .and_then(|next_value| next_value.as_str())
        .map(|next_value| next_value.to_string());

    let previous = value
        .get("previous")
        .and_then(|previous_value| previous_value.as_str())
        .map(|previous_value| previous_value.to_string());

    Some(PaginationInfo::Offset {
        total,
        limit,
        offset,
        next,
        previous,
    })
}

fn parse_cursor_pagination(value: &Value) -> Option<PaginationInfo> {
    let paging = value.get("paging")?.as_object()?;
    let cursors = paging
        .get("cursors")
        .and_then(|cursors| cursors.as_object());

    let after = cursors
        .and_then(|cursor| cursor.get("after"))
        .and_then(|after| after.as_str())
        .map(|after| after.to_string());

    let before = cursors
        .and_then(|cursor| cursor.get("before"))
        .and_then(|before| before.as_str())
        .map(|before| before.to_string());

    let next = paging
        .get("next")
        .and_then(|next| next.as_str())
        .map(|next| next.to_string());

    if after.is_none() && before.is_none() && next.is_none() {
        return None;
    }

    Some(PaginationInfo::Cursor {
        after,
        before,
        next,
    })
}

fn header_value<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_str())
}
