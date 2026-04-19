use axum_test::TestServer;
use fleetreserve_backend::{app::state::AppState, auth::password, routes::build_router};
use rusqlite::Connection;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tempfile::NamedTempFile;

const ADMIN_PASSWORD: &str = "FleetReserveHttpTest#2026";
const ROLE_PASSWORD: &str = "FleetReserveRoleTest#2026";

fn server() -> TestServer {
    TestServer::new(build_router(test_app_state())).unwrap()
}

fn test_app_state() -> AppState {
    let tmp = NamedTempFile::new().expect("temp db");
    let conn = Connection::open(tmp.path()).expect("open db");
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
        .expect("pragma");
    conn.execute_batch(include_str!("../../migrations/001_initial_schema.sql"))
        .expect("schema");
    conn.execute_batch(include_str!("../../migrations/002_seed_data.sql"))
        .expect("seed");
    conn.execute_batch(
        "UPDATE stores SET business_hours_start='00:00', business_hours_end='23:59' WHERE id IN ('store-001','store-002');",
    )
    .expect("hours");
    let admin_hash = password::hash_password(ADMIN_PASSWORD).expect("admin hash");
    conn.execute(
        "UPDATE users SET active = 1, password_hash = ?1 WHERE id = 'user-admin-001'",
        [&admin_hash],
    )
    .expect("admin");
    let role_hash = password::hash_password(ROLE_PASSWORD).expect("role hash");
    conn.execute(
        "INSERT INTO users (id, username, password_hash, display_name, role, store_id, active) VALUES ('user-customer-001', 'customer1', ?1, 'customer1', 'Customer', 'store-001', 1)",
        [&role_hash],
    )
    .expect("customer");
    conn.execute(
        "INSERT INTO users (id, username, password_hash, display_name, role, store_id, active) VALUES ('user-merchant-001', 'merchant1', ?1, 'merchant1', 'MerchantStaff', 'store-001', 1)",
        [&role_hash],
    )
    .expect("merchant");
    conn.execute(
        "INSERT INTO vehicles (id, vin_encrypted, vin_hash, license_plate_encrypted, license_plate_hash, make, model, store_id, status, insurance_expiry, version) VALUES ('v1', 'enc', 'h', 'enc', 'h', 'T', 'V', 'store-001', 'available', '2100-01-01T00:00:00', 1)",
        [],
    )
    .expect("vehicle");

    let upload_dir = tempfile::tempdir().expect("upload dir");
    let upload_path = upload_dir.path().to_string_lossy().into_owned();
    std::mem::forget(upload_dir);
    AppState {
        db: Arc::new(Mutex::new(conn)),
        encryption_key: "x".repeat(32),
        hmac_secret: "y".repeat(32),
        upload_dir: upload_path,
        csrf_tokens: Arc::new(Mutex::new(HashMap::new())),
        revoked_sessions: Arc::new(Mutex::new(HashSet::new())),
    }
}

async fn login_token(s: &TestServer, username: &str, password: &str) -> String {
    let res = s
        .post("/api/auth/login")
        .json(&serde_json::json!({"username": username, "password": password}))
        .await;
    res.assert_status_ok();
    res.json::<serde_json::Value>()["token"]
        .as_str()
        .unwrap()
        .to_string()
}

#[tokio::test]
async fn unit_routes_require_auth_on_auth_routes() {
    let s = server();
    s.get("/api/auth/me")
        .await
        .assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn unit_routes_require_staff_on_staff_routes() {
    let s = server();
    let customer_token = login_token(&s, "customer1", ROLE_PASSWORD).await;
    s.get("/api/vehicles")
        .add_header(
            axum::http::header::AUTHORIZATION,
            axum::http::HeaderValue::from_str(&format!("Bearer {}", customer_token)).unwrap(),
        )
        .await
        .assert_status(axum::http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn unit_routes_allow_staff_on_staff_routes() {
    let s = server();
    let merchant_token = login_token(&s, "merchant1", ROLE_PASSWORD).await;
    let res = s.get("/api/vehicles")
        .add_header(
            axum::http::header::AUTHORIZATION,
            axum::http::HeaderValue::from_str(&format!("Bearer {}", merchant_token)).unwrap(),
        )
        .await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();
    assert!(body["vehicles"].is_array());
    assert!(body["total"].as_u64().unwrap_or(0) >= 1);
}

#[tokio::test]
async fn unit_routes_public_login_route_open_without_auth() {
    let s = server();
    s
        .post("/api/auth/login")
        .json(&serde_json::json!({
            "username": "admin",
            "password": ADMIN_PASSWORD
        }))
        .await
        .assert_status_ok();
}
