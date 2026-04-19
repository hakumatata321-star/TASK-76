//! HTTP API: `/api/tickets/*`
use axum::http::{header, HeaderName, HeaderValue};
use serde_json::json;

use crate::http_helpers::{admin_token_and_csrf, api_server};

#[tokio::test]
async fn api_route_get_ticket_redeem_undo_roundtrip() {
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
            "start_time": "2020-01-01T12:00:00",
            "end_time": "2099-12-31T13:00:00",
        }))
        .await;
    create.assert_status(axum::http::StatusCode::CREATED);
    let created = create.json::<serde_json::Value>();
    let ticket_id = created["ticket"]["id"].as_str().unwrap().to_string();

    let (tok2, _) = admin_token_and_csrf(&s).await;
    let get = s
        .get(&format!("/api/tickets/{}", ticket_id))
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", tok2)).unwrap(),
        )
        .await;
    get.assert_status_ok();
    let ticket = get.json::<serde_json::Value>();
    assert_eq!(ticket["id"], ticket_id);
    assert!(!ticket["redeemed"].as_bool().unwrap());

    let (tok3, csrf3) = admin_token_and_csrf(&s).await;
    let redeem = s
        .post(&format!("/api/tickets/{}/redeem", ticket_id))
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", tok3)).unwrap(),
        )
        .add_header(
            HeaderName::from_static("x-csrf-token"),
            HeaderValue::from_str(&csrf3).unwrap(),
        )
        .await;
    redeem.assert_status_ok();

    let (tok4, csrf4) = admin_token_and_csrf(&s).await;
    let undo = s
        .post(&format!("/api/tickets/{}/undo", ticket_id))
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", tok4)).unwrap(),
        )
        .add_header(
            HeaderName::from_static("x-csrf-token"),
            HeaderValue::from_str(&csrf4).unwrap(),
        )
        .json(&json!({ "reason": "operator correction within window" }))
        .await;
    undo.assert_status_ok();
}

#[tokio::test]
async fn api_route_post_scan_requires_auth() {
    let s = api_server();
    // Unauthenticated request to the QR scan endpoint must be rejected.
    let res = s.post("/api/tickets/scan").await;
    res.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn api_route_post_scan_requires_csrf() {
    let s = api_server();
    let (token, _) = admin_token_and_csrf(&s).await;

    // Send a minimal valid multipart body (empty, just a closing delimiter) so
    // axum's Multipart extractor succeeds and the handler body is reached where
    // the CSRF check runs. Without a multipart Content-Type axum would return
    // 400 before any handler code executes.
    let boundary = "FleetReserveScanBoundary";
    let body = format!("--{}--\r\n", boundary);
    let content_type = format!("multipart/form-data; boundary={}", boundary);

    let res = s
        .post("/api/tickets/scan")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .add_header(
            header::CONTENT_TYPE,
            HeaderValue::from_str(&content_type).unwrap(),
        )
        .bytes(body.into_bytes().into())
        .await;
    res.assert_status(axum::http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn api_route_post_scan_rejects_missing_image() {
    let s = api_server();
    let (token, csrf) = admin_token_and_csrf(&s).await;
    // Send a valid CSRF + auth but no multipart body → validation error.
    let res = s
        .post("/api/tickets/scan")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .add_header(
            HeaderName::from_static("x-csrf-token"),
            HeaderValue::from_str(&csrf).unwrap(),
        )
        .json(&json!({}))
        .await;
    // Sending JSON instead of multipart causes a 400 (validation) or 415 (unsupported media type).
    assert!(
        res.status_code().as_u16() == 400 || res.status_code().as_u16() == 415
            || res.status_code().as_u16() == 422,
        "expected a client-error status, got {}",
        res.status_code()
    );
}
