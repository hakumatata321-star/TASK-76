//! HTTP API: `/api/vehicles`
use axum::http::{header, HeaderValue};
use serde_json::json;

use crate::http_helpers::{admin_token_and_csrf, api_server, merchant_token_and_csrf};

#[tokio::test]
async fn api_route_get_vehicles_returns_masked_row() {
    let s = api_server();
    let (token, _) = admin_token_and_csrf(&s).await;

    let res = s
        .get("/api/vehicles")
        .add_query_param("store_id", "store-001")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();
    let vehicles = body["vehicles"].as_array().expect("vehicles");
    assert!(!vehicles.is_empty());
    let v0 = &vehicles[0];
    assert_eq!(v0["id"], "v1");
    assert_eq!(body["total"], vehicles.len() as u64);
    let vin = v0["vin"].as_str().unwrap();
    assert!(vin.contains('*'), "VIN must be masked in API JSON: {}", vin);
    assert_eq!(v0["store_id"], "store-001");
}

#[tokio::test]
async fn api_route_get_vehicle_by_id_returns_masked_vehicle() {
    let s = api_server();
    let (token, _) = admin_token_and_csrf(&s).await;

    let res = s
        .get("/api/vehicles/v1")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();
    assert_eq!(body["id"], "v1");
    assert_eq!(body["store_id"], "store-001");
    assert!(body["vin"].as_str().unwrap().contains('*'));
    assert!(body["license_plate"].as_str().unwrap().contains('*'));
}

#[tokio::test]
async fn api_route_post_vehicles_creates_vehicle() {
    let s = api_server();
    let (token, csrf) = merchant_token_and_csrf(&s).await;

    let res = s
        .post("/api/vehicles")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .add_header(
            header::HeaderName::from_static("x-csrf-token"),
            HeaderValue::from_str(&csrf).unwrap(),
        )
        .json(&json!({
            "vin": "5YJSA1E26HF000337",
            "license_plate": "ABC1234",
            "make": "Tesla",
            "model": "Model S",
            "trim_level": "Plaid",
            "store_id": "store-001",
            "mileage_miles": 42123,
            "fuel_or_battery_pct": 75.0,
            "maintenance_due": null,
            "inspection_due": null,
            "insurance_expiry": "2099-01-01T00:00:00",
        }))
        .await;
    res.assert_status(axum::http::StatusCode::CREATED);
    let body = res.json::<serde_json::Value>();
    assert_eq!(body["make"], "Tesla");
    assert_eq!(body["model"], "Model S");
    assert_eq!(body["store_id"], "store-001");
    assert_eq!(body["status"], "available");
}

#[tokio::test]
async fn api_route_put_vehicle_status_updates_status() {
    let s = api_server();
    let (token, csrf) = merchant_token_and_csrf(&s).await;

    let res = s
        .put("/api/vehicles/v1/status")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .add_header(
            header::HeaderName::from_static("x-csrf-token"),
            HeaderValue::from_str(&csrf).unwrap(),
        )
        .json(&json!({"status": "reserved"}))
        .await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();
    assert_eq!(body["message"], "Status updated");
    assert_eq!(body["status"], "reserved");
}
