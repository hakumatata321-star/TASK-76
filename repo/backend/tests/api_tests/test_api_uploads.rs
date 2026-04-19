//! HTTP API: `/api/uploads`
use axum::http::{header, HeaderName, HeaderValue};
use axum_test::multipart::{MultipartForm, Part};

use crate::http_helpers::{admin_token_and_csrf, api_server};

/// 1×1 RGB PNG (known-good for `infer` + our magic-byte checks).
fn one_pixel_png() -> Vec<u8> {
    vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
        0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00,
        0x00, 0x03, 0x01, 0x01, 0x00, 0x18, 0xDD, 0x8D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E,
        0x44, 0xAE, 0x42, 0x60, 0x82,
    ]
}

#[tokio::test]
async fn api_route_post_upload_creates_record() {
    let s = api_server();
    let (token, csrf) = admin_token_and_csrf(&s).await;

    let file_part = Part::bytes(one_pixel_png()).file_name("probe.png");
    let form = MultipartForm::new()
        .add_part("file", file_part)
        .add_text("vehicle_id", "v1")
        .add_text("store_id", "store-001");

    let res = s
        .post("/api/uploads")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .add_header(
            HeaderName::from_static("x-csrf-token"),
            HeaderValue::from_str(&csrf).unwrap(),
        )
        .multipart(form)
        .await;
    res.assert_status(axum::http::StatusCode::CREATED);
    let body = res.json::<serde_json::Value>();
    assert!(body["id"].as_str().unwrap().len() > 4);
    assert_eq!(body["vehicle_id"], "v1");
}

#[tokio::test]
async fn api_route_post_upload_rejects_non_image_over_http() {
    let s = api_server();
    let (token, csrf) = admin_token_and_csrf(&s).await;

    let file_part = Part::bytes(b"hello not a jpeg".to_vec()).file_name("x.jpg");
    let form = MultipartForm::new().add_part("file", file_part);

    let res = s
        .post("/api/uploads")
        .add_header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .add_header(
            HeaderName::from_static("x-csrf-token"),
            HeaderValue::from_str(&csrf).unwrap(),
        )
        .multipart(form)
        .await;
    res.assert_status(axum::http::StatusCode::BAD_REQUEST);
}
