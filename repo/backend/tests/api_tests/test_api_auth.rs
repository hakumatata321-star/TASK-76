//! HTTP API: `/api/auth/*`
use axum::http::{header, HeaderValue};
use serde_json::json;

use crate::http_helpers::{admin_token_and_csrf, api_server, login_token_and_csrf};
use crate::http_support::TEST_ADMIN_PASSWORD;

#[tokio::test]
async fn api_route_post_login_returns_token_and_csrf() {
    let s = api_server();
    let res = s
        .post("/api/auth/login")
        .json(&json!({
            "username": "admin",
            "password": TEST_ADMIN_PASSWORD,
        }))
        .await;
    res.assert_status_ok();
    let body: serde_json::Value = res.json();
    assert!(body["token"].as_str().unwrap_or("").len() > 10);
    assert!(body["csrf_token"].as_str().unwrap_or("").len() > 4);
}

#[tokio::test]
async fn api_route_post_login_invalid_password_unauthorized() {
    let s = api_server();
    let res = s
        .post("/api/auth/login")
        .json(&json!({
            "username": "admin",
            "password": "wrong-password-xxxxxxxx",
        }))
        .await;
    res.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn api_route_get_me_returns_user_and_refreshed_token() {
    let s = api_server();
    let (token, _) = admin_token_and_csrf(&s).await;

    let res = s
        .get("/api/auth/me")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();
    assert_eq!(body["user"]["role"], "Administrator");
    assert!(body["refreshed_token"].as_str().unwrap_or("").len() > 10);
}

#[tokio::test]
async fn api_route_post_logout_requires_csrf() {
    let s = api_server();
    let (token, _) = admin_token_and_csrf(&s).await;

    let res = s
        .post("/api/auth/logout")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .await;
    res.assert_status(axum::http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn api_route_post_logout_invalidates_session_csrf() {
    let s = api_server();
    let (token, csrf) = admin_token_and_csrf(&s).await;

    let res = s
        .post("/api/auth/logout")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token.clone())).unwrap(),
        )
        .add_header(
            header::HeaderName::from_static("x-csrf-token"),
            HeaderValue::from_str(&csrf).unwrap(),
        )
        .await;
    res.assert_status_ok();

    let body: serde_json::Value = res.json();
    assert_eq!(body["message"], "Logged out");

    // After logout the session is revoked; the bearer token is rejected (401) before
    // any CSRF check is reached.
    let post_logout = s
        .post("/api/auth/logout")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .add_header(
            header::HeaderName::from_static("x-csrf-token"),
            HeaderValue::from_str(&csrf).unwrap(),
        )
        .await;
    post_logout.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn api_route_get_me_after_logout_is_rejected() {
    let s = api_server();
    let (token, csrf) = admin_token_and_csrf(&s).await;

    // Token works before logout.
    let before = s
        .get("/api/auth/me")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .await;
    before.assert_status_ok();

    // Perform logout.
    s.post("/api/auth/logout")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .add_header(
            header::HeaderName::from_static("x-csrf-token"),
            HeaderValue::from_str(&csrf).unwrap(),
        )
        .await
        .assert_status_ok();

    // Same token must now be rejected on an authenticated GET route (not just logout).
    let after = s
        .get("/api/auth/me")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .await;
    after.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn api_route_get_calendar_after_logout_is_rejected() {
    let s = api_server();
    let (token, csrf) = admin_token_and_csrf(&s).await;

    // Token works before logout.
    let before = s
        .get("/api/calendar")
        .add_query_param("store_id", "store-001")
        .add_query_param("date", "2026-06-15")
        .add_query_param("view", "day")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .await;
    before.assert_status_ok();

    // Logout
    s.post("/api/auth/logout")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .add_header(
            header::HeaderName::from_static("x-csrf-token"),
            HeaderValue::from_str(&csrf).unwrap(),
        )
        .await
        .assert_status_ok();

    // Old token must be rejected on the staff-level GET calendar route.
    let after = s
        .get("/api/calendar")
        .add_query_param("store_id", "store-001")
        .add_query_param("date", "2026-06-15")
        .add_query_param("view", "day")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .await;
    after.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn api_route_post_reset_password_accepts_valid_recovery_code() {
    let s = api_server();
    let (token, csrf) = admin_token_and_csrf(&s).await;

    // Issue a recovery code for admin.
    let issue = s
        .post("/api/admin/recovery-codes")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .add_header(
            header::HeaderName::from_static("x-csrf-token"),
            HeaderValue::from_str(&csrf).unwrap(),
        )
        .json(&json!({"user_id": "user-admin-001"}))
        .await;
    issue.assert_status_ok();
    let code = issue.json::<serde_json::Value>()["code"]
        .as_str()
        .expect("recovery code")
        .to_string();

    let new_password = "FleetReserveNewAdmin#2026";
    let reset = s
        .post("/api/auth/reset-password")
        .json(&json!({
            "username": "admin",
            "recovery_code": code,
            "new_password": new_password,
        }))
        .await;
    reset.assert_status_ok();
    assert_eq!(reset.json::<serde_json::Value>()["message"], "Password reset successful");

    // Verify login now works with the new password.
    let (new_token, new_csrf) = login_token_and_csrf(&s, "admin", new_password).await;
    assert!(!new_token.is_empty());
    assert!(!new_csrf.is_empty());
}

