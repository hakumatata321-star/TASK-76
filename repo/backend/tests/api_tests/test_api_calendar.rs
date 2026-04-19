//! HTTP API: `/api/calendar`
use axum::http::{header, HeaderValue};

use crate::http_helpers::{admin_token_and_csrf, api_server};

#[tokio::test]
async fn api_route_get_calendar_day_view_returns_slots_and_assets() {
    let s = api_server();
    let (token, _) = admin_token_and_csrf(&s).await;

    let res = s
        .get("/api/calendar")
        .add_query_param("store_id", "store-001")
        .add_query_param("date", "2026-06-15")
        .add_query_param("view", "day")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();
    assert_eq!(body["store_id"], "store-001");
    assert_eq!(body["view"], "day");
    assert!(body["slots"].is_array(), "slots must be an array");
    assert!(body["assets"].is_array(), "assets must be an array");
    assert!(body["business_hours"].is_object(), "business_hours must be present");
    // Day view with 24h range (00:00-23:59) produces many 15-min slots
    let slots = body["slots"].as_array().unwrap();
    assert!(!slots.is_empty(), "slots must not be empty for a valid business day");
}

#[tokio::test]
async fn api_route_get_calendar_week_view_covers_seven_days() {
    let s = api_server();
    let (token, _) = admin_token_and_csrf(&s).await;

    // 2026-06-15 is a Monday; week view should cover Mon-Sun
    let res = s
        .get("/api/calendar")
        .add_query_param("store_id", "store-001")
        .add_query_param("date", "2026-06-15")
        .add_query_param("view", "week")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();
    assert_eq!(body["view"], "week");
    let slots = body["slots"].as_array().unwrap();
    // Week view must include slots for exactly 7 distinct dates (Mon–Sun).
    let unique_dates: std::collections::HashSet<&str> = slots
        .iter()
        .filter_map(|s| s["time"].as_str()?.get(0..10))
        .collect();
    assert_eq!(
        unique_dates.len(),
        7,
        "week view must cover exactly 7 distinct dates; got {:?}",
        unique_dates
    );
    // Each day must have multiple slots (business hours span multiple 15-min steps).
    assert!(
        slots.len() >= 7,
        "week view must have at least one slot per day"
    );
}

#[tokio::test]
async fn api_route_get_calendar_invalid_date_rejected() {
    let s = api_server();
    let (token, _) = admin_token_and_csrf(&s).await;

    let res = s
        .get("/api/calendar")
        .add_query_param("store_id", "store-001")
        .add_query_param("date", "not-a-date")
        .add_query_param("view", "day")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .await;
    res.assert_status(axum::http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn api_route_get_calendar_unknown_store_returns_not_found() {
    let s = api_server();
    let (token, _) = admin_token_and_csrf(&s).await;

    let res = s
        .get("/api/calendar")
        .add_query_param("store_id", "store-does-not-exist")
        .add_query_param("date", "2026-06-15")
        .add_query_param("view", "day")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .await;
    res.assert_status(axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn api_route_get_calendar_slots_reflect_store_business_hours() {
    // The test DB sets store-001 hours to 00:00–23:59 (see http_support.rs).
    // Verify the calendar response echoes those hours and that all slot times
    // fall within the reported business window.
    let s = api_server();
    let (token, _) = admin_token_and_csrf(&s).await;

    let res = s
        .get("/api/calendar")
        .add_query_param("store_id", "store-001")
        .add_query_param("date", "2026-06-15")
        .add_query_param("view", "day")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();

    let bh = &body["business_hours"];
    assert!(bh.is_object(), "business_hours must be present in calendar response");
    let bh_start = bh["start"].as_str().expect("business_hours.start");
    let bh_end   = bh["end"].as_str().expect("business_hours.end");
    assert!(!bh_start.is_empty(), "business_hours.start must be non-empty");
    assert!(!bh_end.is_empty(),   "business_hours.end must be non-empty");

    // Test DB uses 00:00–23:59; verify the response echoes those values.
    assert_eq!(bh_start, "00:00", "day-view hours must match store record (00:00)");
    assert_eq!(bh_end,   "23:59", "day-view hours must match store record (23:59)");

    // All slot times must begin on the correct date and fall within the declared window.
    let slots = body["slots"].as_array().expect("slots array");
    assert!(!slots.is_empty(), "slots must not be empty for a 24-hour business day");
    for slot in slots {
        let t = slot["time"].as_str().expect("slot time");
        assert!(
            t.starts_with("2026-06-15T"),
            "slot time must be on the requested date; got {}",
            t
        );
        // Time component ("HH:MM:SS") must be within 00:00:00–23:59:59
        let time_part = &t[11..];
        assert!(
            time_part >= "00:00:00" && time_part <= "23:59:59",
            "slot time {} must be within business window 00:00–23:59",
            t
        );
    }
}

#[tokio::test]
async fn api_route_get_calendar_requires_auth() {
    let s = api_server();
    let res = s
        .get("/api/calendar")
        .add_query_param("store_id", "store-001")
        .add_query_param("date", "2026-06-15")
        .add_query_param("view", "day")
        .await;
    res.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn api_route_get_calendar_slots_have_fifteen_minute_duration() {
    let s = api_server();
    let (token, _) = admin_token_and_csrf(&s).await;

    let res = s
        .get("/api/calendar")
        .add_query_param("store_id", "store-001")
        .add_query_param("date", "2026-06-15")
        .add_query_param("view", "day")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();
    let slots = body["slots"].as_array().unwrap();
    for slot in slots {
        assert_eq!(slot["duration_minutes"], 15);
    }
}
