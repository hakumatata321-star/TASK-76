//! HTTP API: `/api/admin/*`
use axum::http::{header, HeaderName, HeaderValue};
use serde_json::json;

use crate::http_helpers::{admin_token_and_csrf, api_server};

#[tokio::test]
async fn api_route_get_admin_users_returns_masked_list() {
    let s = api_server();
    let (token, _) = admin_token_and_csrf(&s).await;

    let res = s
        .get("/api/admin/users")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();
    let users = body["users"].as_array().expect("users array");
    assert!(!users.is_empty());
    for u in users {
        let username = u["username"].as_str().unwrap();
        assert!(username.contains('*'), "Username must be masked: {}", username);
    }
}

#[tokio::test]
async fn api_route_get_admin_users_requires_auth() {
    let s = api_server();
    let res = s.get("/api/admin/users").await;
    res.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn api_route_post_admin_users_rejected_without_csrf() {
    let s = api_server();
    let (token, _) = admin_token_and_csrf(&s).await;

    let res = s
        .post("/api/admin/users")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .json(&json!({
            "username": "newuser",
            "password": "TestPassword#123",
            "display_name": "New User",
            "role": "Customer",
            "store_id": null,
        }))
        .await;
    res.assert_status(axum::http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn api_route_post_admin_users_creates_new_user() {
    let s = api_server();
    let (token, csrf) = admin_token_and_csrf(&s).await;

    let res = s
        .post("/api/admin/users")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .add_header(
            HeaderName::from_static("x-csrf-token"),
            HeaderValue::from_str(&csrf).unwrap(),
        )
        .json(&json!({
            "username": "newstaff",
            "password": "TestPassword#123",
            "display_name": "New Staff",
            "role": "MerchantStaff",
            "store_id": "store-001",
        }))
        .await;
    res.assert_status(axum::http::StatusCode::CREATED);
    let body = res.json::<serde_json::Value>();
    assert!(body["id"].as_str().is_some());
}

#[tokio::test]
async fn api_route_put_admin_user_role_updates_role() {
    let s = api_server();
    let (token, csrf) = admin_token_and_csrf(&s).await;

    // Create a user to update
    let create_res = s
        .post("/api/admin/users")
        .add_header(header::AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", token)).unwrap())
        .add_header(HeaderName::from_static("x-csrf-token"), HeaderValue::from_str(&csrf).unwrap())
        .json(&json!({"username": "roletest", "password": "TestPwd#123", "display_name": "Role Test", "role": "Customer", "store_id": null}))
        .await;
    create_res.assert_status(axum::http::StatusCode::CREATED);
    let user_id = create_res.json::<serde_json::Value>()["id"].as_str().unwrap().to_string();

    let (token2, csrf2) = admin_token_and_csrf(&s).await;
    let res = s
        .put(&format!("/api/admin/users/{}/role", user_id))
        .add_header(header::AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", token2)).unwrap())
        .add_header(HeaderName::from_static("x-csrf-token"), HeaderValue::from_str(&csrf2).unwrap())
        .json(&json!({"role": "Photographer"}))
        .await;
    res.assert_status_ok();
}

#[tokio::test]
async fn api_route_put_admin_user_role_rejected_without_csrf() {
    let s = api_server();
    let (token, _) = admin_token_and_csrf(&s).await;

    let res = s
        .put("/api/admin/users/user-admin-001/role")
        .add_header(header::AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", token)).unwrap())
        .json(&json!({"role": "Customer"}))
        .await;
    res.assert_status(axum::http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn api_route_put_admin_user_active_toggles_status() {
    let s = api_server();
    let (token, csrf) = admin_token_and_csrf(&s).await;

    // Create a user then deactivate them
    let create_res = s
        .post("/api/admin/users")
        .add_header(header::AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", token)).unwrap())
        .add_header(HeaderName::from_static("x-csrf-token"), HeaderValue::from_str(&csrf).unwrap())
        .json(&json!({"username": "activetest", "password": "TestPwd#123", "display_name": "Active Test", "role": "Customer", "store_id": null}))
        .await;
    create_res.assert_status(axum::http::StatusCode::CREATED);
    let user_id = create_res.json::<serde_json::Value>()["id"].as_str().unwrap().to_string();

    let (token2, csrf2) = admin_token_and_csrf(&s).await;
    let res = s
        .put(&format!("/api/admin/users/{}/active", user_id))
        .add_header(header::AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", token2)).unwrap())
        .add_header(HeaderName::from_static("x-csrf-token"), HeaderValue::from_str(&csrf2).unwrap())
        .json(&json!({"active": false}))
        .await;
    res.assert_status_ok();
}

#[tokio::test]
async fn api_route_get_admin_permissions_returns_seeded_list() {
    let s = api_server();
    let (token, _) = admin_token_and_csrf(&s).await;

    let res = s
        .get("/api/admin/permissions")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();
    let perms = body["permissions"].as_array().expect("permissions array");
    assert!(!perms.is_empty(), "seed data must have permissions");
}

#[tokio::test]
async fn api_route_post_admin_permissions_upserts_permission() {
    let s = api_server();
    let (token, csrf) = admin_token_and_csrf(&s).await;

    let res = s
        .post("/api/admin/permissions")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .add_header(
            HeaderName::from_static("x-csrf-token"),
            HeaderValue::from_str(&csrf).unwrap(),
        )
        .json(&json!({
            "role": "Customer",
            "resource": "vehicle",
            "action": "view_test",
        }))
        .await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();
    assert!(body["id"].as_str().is_some());
}

#[tokio::test]
async fn api_route_post_admin_permissions_rejected_without_csrf() {
    let s = api_server();
    let (token, _) = admin_token_and_csrf(&s).await;

    let res = s
        .post("/api/admin/permissions")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .json(&json!({"role": "Customer", "resource": "vehicle", "action": "view_test"}))
        .await;
    res.assert_status(axum::http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn api_route_post_admin_permissions_id_deletes_permission() {
    let s = api_server();
    let (token, csrf) = admin_token_and_csrf(&s).await;

    // Create a synthetic permission row first.
    let create = s
        .post("/api/admin/permissions")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .add_header(
            HeaderName::from_static("x-csrf-token"),
            HeaderValue::from_str(&csrf).unwrap(),
        )
        .json(&json!({
            "role": "Customer",
            "resource": "vehicle",
            "action": "delete_test_permission",
        }))
        .await;
    create.assert_status_ok();
    let id = create.json::<serde_json::Value>()["id"]
        .as_str()
        .expect("permission id")
        .to_string();

    let (token2, csrf2) = admin_token_and_csrf(&s).await;
    let del = s
        .post(&format!("/api/admin/permissions/{}", id))
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token2)).unwrap(),
        )
        .add_header(
            HeaderName::from_static("x-csrf-token"),
            HeaderValue::from_str(&csrf2).unwrap(),
        )
        .await;
    del.assert_status_ok();
    assert_eq!(del.json::<serde_json::Value>()["message"], "Permission deleted");
}

#[tokio::test]
async fn api_route_post_admin_recovery_code_issued_for_existing_user() {
    let s = api_server();
    let (token, csrf) = admin_token_and_csrf(&s).await;

    let res = s
        .post("/api/admin/recovery-codes")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .add_header(
            HeaderName::from_static("x-csrf-token"),
            HeaderValue::from_str(&csrf).unwrap(),
        )
        .json(&json!({"user_id": "user-admin-001"}))
        .await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();
    let code = body["code"].as_str().expect("code present");
    assert_eq!(code.len(), 12, "Recovery code must be 12 characters");
    assert!(body["expires_at"].as_str().is_some());
}

#[tokio::test]
async fn api_route_post_admin_recovery_code_rejected_for_unknown_user() {
    let s = api_server();
    let (token, csrf) = admin_token_and_csrf(&s).await;

    let res = s
        .post("/api/admin/recovery-codes")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .add_header(
            HeaderName::from_static("x-csrf-token"),
            HeaderValue::from_str(&csrf).unwrap(),
        )
        .json(&json!({"user_id": "nonexistent-user-id"}))
        .await;
    res.assert_status(axum::http::StatusCode::NOT_FOUND);
}
