//! HTTP API: `/api/stores`
use axum::http::{header, HeaderValue};

use crate::http_helpers::{admin_token_and_csrf, api_server};

#[tokio::test]
async fn api_route_get_stores_lists_active_stores() {
    let s = api_server();
    let (token, _) = admin_token_and_csrf(&s).await;

    let res = s
        .get("/api/stores")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();
    let stores = body["stores"].as_array().expect("stores");
    assert!(!stores.is_empty());
    assert!(stores.len() >= 2, "seed should include at least two stores");
    let ids: Vec<&str> = stores
        .iter()
        .filter_map(|s| s.get("id").and_then(|v| v.as_str()))
        .collect();
    assert!(ids.contains(&"store-001"));
    assert!(ids.contains(&"store-002"));
    for s in stores {
        assert!(s["name"].as_str().unwrap_or("").len() > 2);
        assert!(s["location"].as_str().unwrap_or("").len() > 2);
        assert!(s["business_hours_start"].as_str().unwrap_or("").contains(':'));
        assert!(s["business_hours_end"].as_str().unwrap_or("").contains(':'));
        assert!(s["active"].is_boolean());
    }
}
