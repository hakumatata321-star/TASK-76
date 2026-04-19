//! HTTP API: cross-role authorization matrix for protected routes.
use axum::http::{header, HeaderName, HeaderValue};
use serde_json::json;

use crate::http_helpers::{
    admin_token_and_csrf, api_server, customer_token_and_csrf, merchant_token_and_csrf,
    ops_token_and_csrf, photographer_token_and_csrf,
};

#[tokio::test]
async fn api_authz_customer_forbidden_on_staff_ops_admin_routes() {
    let s = api_server();
    let (token, csrf) = customer_token_and_csrf(&s).await;
    let auth = HeaderValue::from_str(&format!("Bearer {}", token)).unwrap();
    let csrf_hdr = HeaderValue::from_str(&csrf).unwrap();

    s.get("/api/vehicles")
        .add_header(header::AUTHORIZATION, auth.clone())
        .await
        .assert_status(axum::http::StatusCode::FORBIDDEN);

    s.post("/api/exports")
        .add_header(header::AUTHORIZATION, auth.clone())
        .add_header(HeaderName::from_static("x-csrf-token"), csrf_hdr)
        .json(&json!({}))
        .await
        .assert_status(axum::http::StatusCode::FORBIDDEN);

    s.get("/api/admin/users")
        .add_header(header::AUTHORIZATION, auth)
        .await
        .assert_status(axum::http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn api_authz_photographer_allowed_auth_routes_forbidden_staff_routes() {
    let s = api_server();
    let (token, csrf) = photographer_token_and_csrf(&s).await;
    let auth = HeaderValue::from_str(&format!("Bearer {}", token)).unwrap();

    let assignments = s
        .get("/api/assignments")
        .add_header(header::AUTHORIZATION, auth.clone())
        .await;
    assignments.assert_status_ok();
    assert!(assignments.json::<serde_json::Value>()["assignments"].is_array());

    s.get("/api/vehicles")
        .add_header(header::AUTHORIZATION, auth.clone())
        .await
        .assert_status(axum::http::StatusCode::FORBIDDEN);

    // Photographer with no assignments sees an empty reservation list, not a store dump.
    let res_list = s
        .get("/api/reservations")
        .add_header(header::AUTHORIZATION, auth.clone())
        .await;
    res_list.assert_status_ok();
    let body = res_list.json::<serde_json::Value>();
    assert!(body["reservations"].is_array(), "reservations key must exist");
    assert_eq!(
        body["reservations"].as_array().unwrap().len(),
        0,
        "photographer with no assignments must see zero reservations"
    );

    s.post("/api/reservations")
        .add_header(header::AUTHORIZATION, auth)
        .add_header(
            HeaderName::from_static("x-csrf-token"),
            HeaderValue::from_str(&csrf).unwrap(),
        )
        .json(&json!({
            "asset_type": "vehicle",
            "asset_id": "v1",
            "store_id": "store-001",
            "start_time": "2030-01-01T11:00:00",
            "end_time":   "2030-01-01T12:00:00",
        }))
        .await
        .assert_status(axum::http::StatusCode::FORBIDDEN);
}

/// Photographer cannot list reservations or view tickets that are not tied to
/// one of their assignments.
#[tokio::test]
async fn api_authz_photographer_cannot_see_unrelated_reservations_or_tickets() {
    let s = api_server();

    // Admin creates a reservation (unrelated to photographer's assignments).
    let (adm_tok, adm_csrf) = admin_token_and_csrf(&s).await;
    let create = s
        .post("/api/reservations")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", adm_tok)).unwrap(),
        )
        .add_header(
            HeaderName::from_static("x-csrf-token"),
            HeaderValue::from_str(&adm_csrf).unwrap(),
        )
        .json(&json!({
            "asset_type": "vehicle",
            "asset_id": "v1",
            "store_id": "store-001",
            "start_time": "2030-01-01T09:00:00",
            "end_time":   "2030-01-01T10:00:00",
        }))
        .await;
    create.assert_status(axum::http::StatusCode::CREATED);
    let ticket_id = create.json::<serde_json::Value>()["ticket"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Photographer (photo1) has no assignments in the test DB, so they must
    // not see this reservation.
    let (ph_tok, _) = photographer_token_and_csrf(&s).await;
    let ph_auth = HeaderValue::from_str(&format!("Bearer {}", ph_tok)).unwrap();

    let res_list = s
        .get("/api/reservations")
        .add_header(header::AUTHORIZATION, ph_auth.clone())
        .await;
    res_list.assert_status_ok();
    let reservations = res_list.json::<serde_json::Value>();
    assert_eq!(
        reservations["reservations"].as_array().unwrap().len(),
        0,
        "photographer must not see unrelated reservations"
    );

    // Photographer must also be denied direct ticket access for unrelated assets.
    let ticket_res = s
        .get(&format!("/api/tickets/{}", ticket_id))
        .add_header(header::AUTHORIZATION, ph_auth)
        .await;
    ticket_res.assert_status(axum::http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn api_authz_merchant_allowed_staff_forbidden_ops_admin() {
    let s = api_server();
    let (token, csrf) = merchant_token_and_csrf(&s).await;
    let auth = HeaderValue::from_str(&format!("Bearer {}", token)).unwrap();

    let vehicles = s
        .get("/api/vehicles")
        .add_header(header::AUTHORIZATION, auth.clone())
        .await;
    vehicles.assert_status_ok();
    assert!(vehicles.json::<serde_json::Value>()["vehicles"].is_array());

    s.post("/api/exports")
        .add_header(header::AUTHORIZATION, auth.clone())
        .add_header(
            HeaderName::from_static("x-csrf-token"),
            HeaderValue::from_str(&csrf).unwrap(),
        )
        .json(&json!({}))
        .await
        .assert_status(axum::http::StatusCode::FORBIDDEN);

    s.get("/api/admin/users")
        .add_header(header::AUTHORIZATION, auth)
        .await
        .assert_status(axum::http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn api_authz_ops_allowed_ops_forbidden_admin() {
    let s = api_server();
    let (token, csrf) = ops_token_and_csrf(&s).await;
    let auth = HeaderValue::from_str(&format!("Bearer {}", token)).unwrap();

    let exports = s
        .post("/api/exports")
        .add_header(header::AUTHORIZATION, auth.clone())
        .add_header(
            HeaderName::from_static("x-csrf-token"),
            HeaderValue::from_str(&csrf).unwrap(),
        )
        .json(&json!({}))
        .await;
    exports.assert_status_ok();
    let exports_body = exports.json::<serde_json::Value>();
    assert!(exports_body["vehicles"].is_array());
    assert!(exports_body["reservations"].is_array());

    let audit = s
        .get("/api/audit")
        .add_query_param("limit", "5")
        .add_header(header::AUTHORIZATION, auth.clone())
        .await;
    audit.assert_status_ok();
    assert!(audit.json::<serde_json::Value>()["entries"].is_array());

    s.get("/api/admin/users")
        .add_header(header::AUTHORIZATION, auth)
        .await
        .assert_status(axum::http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn api_authz_photographer_cannot_create_reservations() {
    let s = api_server();
    let (token, csrf) = photographer_token_and_csrf(&s).await;
    let auth = HeaderValue::from_str(&format!("Bearer {}", token)).unwrap();
    let csrf_hdr = HeaderValue::from_str(&csrf).unwrap();

    let res = s
        .post("/api/reservations")
        .add_header(header::AUTHORIZATION, auth)
        .add_header(HeaderName::from_static("x-csrf-token"), csrf_hdr)
        .json(&json!({
            "asset_type": "vehicle",
            "asset_id": "v1",
            "store_id": "store-001",
            "start_time": "2030-06-01T09:00:00",
            "end_time": "2030-06-01T10:00:00",
        }))
        .await;
    res.assert_status(axum::http::StatusCode::FORBIDDEN);
}
