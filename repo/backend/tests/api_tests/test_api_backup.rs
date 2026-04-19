//! HTTP API: `/api/backup`
use axum::http::{header, HeaderName, HeaderValue};
use serde_json::json;

use crate::http_helpers::{admin_token_and_csrf, api_server};

#[tokio::test]
async fn api_route_post_backup_rejected_without_csrf() {
    let s = api_server();
    let (token, _) = admin_token_and_csrf(&s).await;

    let tmp = tempfile::tempdir().expect("temp dir");
    let path = tmp.path().to_string_lossy().to_string();

    let res = s
        .post("/api/backup")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .json(&json!({"path": path}))
        .await;
    res.assert_status(axum::http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn api_route_post_backup_creates_encrypted_file() {
    let s = api_server();
    let (token, csrf) = admin_token_and_csrf(&s).await;

    let tmp = tempfile::tempdir().expect("temp dir");
    let path = tmp.path().to_string_lossy().to_string();

    let res = s
        .post("/api/backup")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .add_header(
            HeaderName::from_static("x-csrf-token"),
            HeaderValue::from_str(&csrf).unwrap(),
        )
        .json(&json!({"path": path}))
        .await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();
    assert!(body["id"].as_str().is_some());
    assert!(body["filename"].as_str().unwrap().ends_with(".enc"));
    assert!(body["sha256"].as_str().is_some());
    let backup_path = body["path"].as_str().unwrap();
    assert!(
        std::path::Path::new(backup_path).exists(),
        "Backup file must exist on disk"
    );
}

#[tokio::test]
async fn api_route_post_backup_restore_rejected_without_csrf() {
    let s = api_server();
    let (token, _) = admin_token_and_csrf(&s).await;

    let res = s
        .post("/api/backup/restore")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .json(&json!({"path": "/tmp/nonexistent-backup.enc"}))
        .await;
    res.assert_status(axum::http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn api_route_post_backup_restore_fails_for_missing_file() {
    let s = api_server();
    let (token, csrf) = admin_token_and_csrf(&s).await;

    let res = s
        .post("/api/backup/restore")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .add_header(
            HeaderName::from_static("x-csrf-token"),
            HeaderValue::from_str(&csrf).unwrap(),
        )
        .json(&json!({"path": "/tmp/fleetreserve-does-not-exist.enc"}))
        .await;
    res.assert_status(axum::http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn api_route_post_backup_restore_roundtrip_succeeds() {
    let s = api_server();
    let tmp = tempfile::tempdir().expect("temp dir");
    let path = tmp.path().to_string_lossy().to_string();

    // Step 1: create a backup
    let (token, csrf) = admin_token_and_csrf(&s).await;
    let backup_res = s
        .post("/api/backup")
        .add_header(header::AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", token)).unwrap())
        .add_header(HeaderName::from_static("x-csrf-token"), HeaderValue::from_str(&csrf).unwrap())
        .json(&json!({"path": path}))
        .await;
    backup_res.assert_status_ok();
    let backup_path = backup_res.json::<serde_json::Value>()["path"]
        .as_str()
        .unwrap()
        .to_string();

    // Step 2: restore from the backup
    let (token2, csrf2) = admin_token_and_csrf(&s).await;
    let restore_res = s
        .post("/api/backup/restore")
        .add_header(header::AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", token2)).unwrap())
        .add_header(HeaderName::from_static("x-csrf-token"), HeaderValue::from_str(&csrf2).unwrap())
        .json(&json!({"path": backup_path}))
        .await;
    restore_res.assert_status_ok();
    let body = restore_res.json::<serde_json::Value>();
    assert!(body["message"].as_str().unwrap().contains("success"));
}
