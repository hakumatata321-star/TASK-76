//! HTTP API: `/api/reservations`
use axum::http::{header, HeaderName, HeaderValue};
use serde_json::json;

use crate::http_helpers::{admin_token_and_csrf, api_server};

#[tokio::test]
async fn api_route_post_reservation_rejected_without_csrf() {
    let s = api_server();
    let (token, _) = admin_token_and_csrf(&s).await;

    let res = s
        .post("/api/reservations")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .json(&json!({
            "asset_type": "vehicle",
            "asset_id": "v1",
            "store_id": "store-001",
            "start_time": "2026-06-01T10:00:00",
            "end_time": "2026-06-01T11:00:00",
        }))
        .await;
    res.assert_status(axum::http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn api_route_post_reservation_created_with_csrf() {
    let s = api_server();
    let (token, csrf) = admin_token_and_csrf(&s).await;

    let res = s
        .post("/api/reservations")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .add_header(
            HeaderName::from_static("x-csrf-token"),
            HeaderValue::from_str(&csrf).unwrap(),
        )
        .json(&json!({
            "asset_type": "vehicle",
            "asset_id": "v1",
            "store_id": "store-001",
            "start_time": "2026-06-01T10:00:00",
            "end_time": "2026-06-01T11:00:00",
        }))
        .await;
    res.assert_status(axum::http::StatusCode::CREATED);
    let out = res.json::<serde_json::Value>();
    assert_eq!(out["reservation"]["status"], "confirmed");
    let uid = out["reservation"]["user_id"].as_str().unwrap_or("");
    assert!(uid.starts_with("usr-"), "reservation user_id should be masked");
    assert!(out["ticket"]["ticket_number"].as_str().unwrap().starts_with("FR-"));
}

#[tokio::test]
async fn api_route_get_reservations_lists_for_admin() {
    let s = api_server();
    let (token, csrf) = admin_token_and_csrf(&s).await;

    let create = s
        .post("/api/reservations")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .add_header(
            HeaderName::from_static("x-csrf-token"),
            HeaderValue::from_str(&csrf).unwrap(),
        )
        .json(&json!({
            "asset_type": "vehicle",
            "asset_id": "v1",
            "store_id": "store-001",
            "start_time": "2020-01-01T10:00:00",
            "end_time": "2099-12-31T11:00:00",
        }))
        .await;
    create.assert_status(axum::http::StatusCode::CREATED);

    let (token2, _) = admin_token_and_csrf(&s).await;
    let list = s
        .get("/api/reservations")
        .add_query_param("store_id", "store-001")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token2)).unwrap(),
        )
        .await;
    list.assert_status_ok();
    let body = list.json::<serde_json::Value>();
    let arr = body["reservations"].as_array().expect("reservations");
    assert!(!arr.is_empty(), "GET /api/reservations should return seeded booking");
    let uid = arr[0]["user_id"].as_str().unwrap_or("");
    assert!(uid.starts_with("usr-"), "listed reservation user_id should be masked");
}
