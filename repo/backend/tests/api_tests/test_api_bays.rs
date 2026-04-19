//! HTTP API: `/api/bays`
use axum::http::{header, HeaderName, HeaderValue};
use serde_json::json;

use crate::http_helpers::{admin_token_and_csrf, api_server};

#[tokio::test]
async fn api_route_get_bays_returns_empty_for_store_with_no_bays() {
    let s = api_server();
    let (token, _) = admin_token_and_csrf(&s).await;

    let res = s
        .get("/api/bays")
        .add_query_param("store_id", "store-001")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();
    assert!(body["bays"].is_array());
}

#[tokio::test]
async fn api_route_get_bays_requires_auth() {
    let s = api_server();
    let res = s.get("/api/bays").add_query_param("store_id", "store-001").await;
    res.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn api_route_post_bays_creates_bay() {
    let s = api_server();
    let (token, csrf) = admin_token_and_csrf(&s).await;

    let res = s
        .post("/api/bays")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .add_header(
            HeaderName::from_static("x-csrf-token"),
            HeaderValue::from_str(&csrf).unwrap(),
        )
        .json(&json!({
            "name": "Bay A",
            "store_id": "store-001",
            "bay_type": "general",
            "capacity": 2,
        }))
        .await;
    res.assert_status(axum::http::StatusCode::CREATED);
    let body = res.json::<serde_json::Value>();
    assert_eq!(body["name"], "Bay A");
    assert_eq!(body["capacity"], 2);
    assert!(body["id"].as_str().is_some());
}

#[tokio::test]
async fn api_route_post_bays_rejected_without_csrf() {
    let s = api_server();
    let (token, _) = admin_token_and_csrf(&s).await;

    let res = s
        .post("/api/bays")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .json(&json!({
            "name": "Bay B",
            "store_id": "store-001",
            "bay_type": "general",
            "capacity": 1,
        }))
        .await;
    res.assert_status(axum::http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn api_route_post_bays_then_list_shows_new_bay() {
    let s = api_server();
    let (token, csrf) = admin_token_and_csrf(&s).await;

    s.post("/api/bays")
        .add_header(header::AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", token)).unwrap())
        .add_header(HeaderName::from_static("x-csrf-token"), HeaderValue::from_str(&csrf).unwrap())
        .json(&json!({"name": "Bay List Test", "store_id": "store-001", "bay_type": "wash", "capacity": 1}))
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    let (token2, _) = admin_token_and_csrf(&s).await;
    let list = s
        .get("/api/bays")
        .add_query_param("store_id", "store-001")
        .add_header(header::AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", token2)).unwrap())
        .await;
    list.assert_status_ok();
    let body = list.json::<serde_json::Value>();
    let bays = body["bays"].as_array().expect("bays array");
    assert!(!bays.is_empty());
    assert!(bays.iter().any(|b| b["name"] == "Bay List Test"));
}
