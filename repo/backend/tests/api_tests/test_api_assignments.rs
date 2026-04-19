//! HTTP API: `/api/assignments`
use axum::http::{header, HeaderName, HeaderValue};
use serde_json::json;

use crate::http_helpers::{admin_token_and_csrf, api_server};

#[tokio::test]
async fn api_route_get_assignments_returns_list_envelope() {
    let s = api_server();
    let (token, _) = admin_token_and_csrf(&s).await;

    let res = s
        .get("/api/assignments")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();
    assert!(body["assignments"].is_array());
}

#[tokio::test]
async fn api_route_get_assignments_requires_auth() {
    let s = api_server();
    let res = s.get("/api/assignments").await;
    res.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn api_route_post_assignments_rejected_without_csrf() {
    let s = api_server();
    let (token, _) = admin_token_and_csrf(&s).await;

    let res = s
        .post("/api/assignments")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .json(&json!({
            "photographer_user_id": "user-admin-001",
            "store_id": "store-001",
            "job_description": "Shoot exterior",
            "vehicle_id": null,
            "bay_id": null,
            "start_time": "2026-07-01T09:00:00",
            "end_time": "2026-07-01T11:00:00",
        }))
        .await;
    res.assert_status(axum::http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn api_route_post_assignments_creates_assignment() {
    let s = api_server();
    let (token, csrf) = admin_token_and_csrf(&s).await;

    let res = s
        .post("/api/assignments")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .add_header(
            HeaderName::from_static("x-csrf-token"),
            HeaderValue::from_str(&csrf).unwrap(),
        )
        .json(&json!({
            "photographer_user_id": "user-admin-001",
            "store_id": "store-001",
            "job_description": "Shoot exterior photos",
            "vehicle_id": "v1",
            "bay_id": null,
            "start_time": "2026-07-01T09:00:00",
            "end_time": "2026-07-01T11:00:00",
        }))
        .await;
    res.assert_status(axum::http::StatusCode::CREATED);
    let body = res.json::<serde_json::Value>();
    assert!(body["id"].as_str().is_some());
    assert_eq!(body["store_id"], "store-001");
    assert_eq!(body["job_description"], "Shoot exterior photos");
}

#[tokio::test]
async fn api_route_post_assignments_then_list_shows_assignment() {
    let s = api_server();
    let (token, csrf) = admin_token_and_csrf(&s).await;

    s.post("/api/assignments")
        .add_header(header::AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", token)).unwrap())
        .add_header(HeaderName::from_static("x-csrf-token"), HeaderValue::from_str(&csrf).unwrap())
        .json(&json!({
            "photographer_user_id": "user-admin-001",
            "store_id": "store-001",
            "job_description": "List check job",
            "vehicle_id": null,
            "bay_id": null,
            "start_time": "2026-08-01T10:00:00",
            "end_time": "2026-08-01T12:00:00",
        }))
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    let (token2, _) = admin_token_and_csrf(&s).await;
    let list = s
        .get("/api/assignments")
        .add_header(header::AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", token2)).unwrap())
        .await;
    list.assert_status_ok();
    let body = list.json::<serde_json::Value>();
    let assignments = body["assignments"].as_array().expect("assignments array");
    assert!(!assignments.is_empty());
    assert!(assignments.iter().any(|a| a["job_description"] == "List check job"));
}
