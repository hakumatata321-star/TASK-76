//! Shared `TestServer` helpers for HTTP API route tests (`tests/api/`).
use axum_test::TestServer;
use fleetreserve_backend::routes::build_router;
use serde_json::json;

use crate::http_support::{
    test_app_state, TEST_ADMIN_PASSWORD, TEST_CUSTOMER_USERNAME, TEST_MERCHANT_USERNAME,
    TEST_OPS_USERNAME, TEST_PHOTOGRAPHER_USERNAME, TEST_ROLE_PASSWORD,
};

pub fn api_server() -> TestServer {
    TestServer::new(build_router(test_app_state())).unwrap()
}

pub async fn admin_token_and_csrf(s: &TestServer) -> (String, String) {
    login_token_and_csrf(s, "admin", TEST_ADMIN_PASSWORD).await
}

pub async fn customer_token_and_csrf(s: &TestServer) -> (String, String) {
    login_token_and_csrf(s, TEST_CUSTOMER_USERNAME, TEST_ROLE_PASSWORD).await
}

pub async fn photographer_token_and_csrf(s: &TestServer) -> (String, String) {
    login_token_and_csrf(s, TEST_PHOTOGRAPHER_USERNAME, TEST_ROLE_PASSWORD).await
}

pub async fn merchant_token_and_csrf(s: &TestServer) -> (String, String) {
    login_token_and_csrf(s, TEST_MERCHANT_USERNAME, TEST_ROLE_PASSWORD).await
}

pub async fn ops_token_and_csrf(s: &TestServer) -> (String, String) {
    login_token_and_csrf(s, TEST_OPS_USERNAME, TEST_ROLE_PASSWORD).await
}

pub async fn login_token_and_csrf(s: &TestServer, username: &str, password: &str) -> (String, String) {
    let res = s
        .post("/api/auth/login")
        .json(&json!({
            "username": username,
            "password": password,
        }))
        .await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();
    (
        body["token"].as_str().unwrap().to_string(),
        body["csrf_token"].as_str().unwrap().to_string(),
    )
}
