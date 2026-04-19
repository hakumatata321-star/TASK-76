//! HTTP API: `/api/exports`
use axum::http::{header, HeaderName, HeaderValue};
use serde_json::json;

use crate::http_helpers::{admin_token_and_csrf, api_server, ops_token_and_csrf};

#[tokio::test]
async fn api_route_post_exports_returns_export_envelope() {
    let s = api_server();
    let (token, csrf) = admin_token_and_csrf(&s).await;

    let res = s
        .post("/api/exports")
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
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();
    assert!(body["reservations"].is_array(), "export must include reservations");
    assert!(body["vehicles"].is_array(), "export must include vehicles");
    assert!(body["export_type"].as_str().is_some());
    assert!(body["exported_at"].as_str().is_some());
}

#[tokio::test]
async fn api_route_post_exports_requires_csrf() {
    let s = api_server();
    let (token, _) = admin_token_and_csrf(&s).await;

    let res = s
        .post("/api/exports")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .json(&json!({}))
        .await;
    res.assert_status(axum::http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn api_route_post_exports_filtered_by_store() {
    let s = api_server();
    let (token, csrf) = admin_token_and_csrf(&s).await;

    let res = s
        .post("/api/exports")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .add_header(
            HeaderName::from_static("x-csrf-token"),
            HeaderValue::from_str(&csrf).unwrap(),
        )
        .json(&json!({"store_id": "store-001"}))
        .await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();
    for v in body["vehicles"].as_array().unwrap() {
        assert_eq!(v["store_id"], "store-001");
    }
}

#[tokio::test]
async fn api_route_post_exports_vehicles_omit_sensitive_fields() {
    let s = api_server();
    let (token, csrf) = admin_token_and_csrf(&s).await;

    let res = s
        .post("/api/exports")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .add_header(
            HeaderName::from_static("x-csrf-token"),
            HeaderValue::from_str(&csrf).unwrap(),
        )
        .json(&json!({"store_id": "store-001"}))
        .await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();
    for v in body["vehicles"].as_array().unwrap() {
        assert!(v.get("vin").is_none(), "VIN must not appear in export");
        assert!(v.get("vin_encrypted").is_none(), "vin_encrypted must not appear in export");
        assert!(v.get("license_plate").is_none(), "license_plate must not appear in export");
    }
}

/// Reservation user_id must be masked — raw UUIDs must not appear in exports.
#[tokio::test]
async fn api_route_post_exports_reservations_mask_user_id() {
    let s = api_server();
    let (token, csrf) = admin_token_and_csrf(&s).await;

    // Create a reservation so the export has at least one row to assert on.
    s.post("/api/reservations")
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
            "start_time": "2040-06-01T09:00:00",
            "end_time":   "2040-06-01T10:00:00",
        }))
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    let (tok2, csrf2) = admin_token_and_csrf(&s).await;
    let res = s
        .post("/api/exports")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", tok2)).unwrap(),
        )
        .add_header(
            HeaderName::from_static("x-csrf-token"),
            HeaderValue::from_str(&csrf2).unwrap(),
        )
        .json(&json!({"store_id": "store-001"}))
        .await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();

    for r in body["reservations"].as_array().unwrap() {
        let uid = r["user_id"].as_str().unwrap_or("");
        assert!(
            uid.starts_with("usr-"),
            "user_id must be masked in exports, got: {}",
            uid
        );
        assert!(
            !uid.contains("user-admin-001"),
            "raw user UUID must not appear in export user_id"
        );
    }
}

#[tokio::test]
async fn api_route_post_exports_requires_auth() {
    let s = api_server();
    let res = s.post("/api/exports").json(&json!({})).await;
    res.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

/// Ops role must have the export permission seeded; if it is revoked the handler
/// denies access even though the route middleware lets ops through.
#[tokio::test]
async fn api_route_post_exports_ops_allowed_by_seeded_permission() {
    let s = api_server();
    let (token, csrf) = ops_token_and_csrf(&s).await;

    let res = s
        .post("/api/exports")
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
    res.assert_status_ok();
}

#[tokio::test]
async fn api_route_post_exports_ops_forbidden_after_permission_revoked() {
    let s = api_server();

    let (adm_tok, adm_csrf) = admin_token_and_csrf(&s).await;
    let perms = s
        .get("/api/admin/permissions")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", adm_tok)).unwrap(),
        )
        .await;
    perms.assert_status_ok();
    let id = perms
        .json::<serde_json::Value>()["permissions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| {
            p["role"] == "PlatformOps"
                && p["resource"] == "export"
                && p["action"] == "create"
        })
        .and_then(|p| p["id"].as_str())
        .expect("PlatformOps export:create permission id")
        .to_string();

    s.post(&format!("/api/admin/permissions/{}", id))
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", adm_tok)).unwrap(),
        )
        .add_header(
            HeaderName::from_static("x-csrf-token"),
            HeaderValue::from_str(&adm_csrf).unwrap(),
        )
        .await
        .assert_status_ok();

    let (ops_tok, ops_csrf) = ops_token_and_csrf(&s).await;
    s.post("/api/exports")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", ops_tok)).unwrap(),
        )
        .add_header(
            HeaderName::from_static("x-csrf-token"),
            HeaderValue::from_str(&ops_csrf).unwrap(),
        )
        .json(&json!({}))
        .await
        .assert_status(axum::http::StatusCode::FORBIDDEN);
}
