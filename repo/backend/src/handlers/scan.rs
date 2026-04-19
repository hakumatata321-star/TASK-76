use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use crate::app::state::AppState;
use crate::errors::AppError;
use crate::handlers::auth::*;
use crate::models::UserRole;

/// Decode a QR code from an uploaded image and return the encoded string value.
/// The frontend check-in page posts camera / file-picker images here; the
/// response `ticket_value` is then fed into the same redeem flow as manual
/// entry, ensuring identical validation and error paths.
pub async fn scan_qr(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut multipart: axum::extract::Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    let claims = extract_claims_required(&headers, &state.hmac_secret)?;
    require_role(&claims, &UserRole::MerchantStaff)?;
    require_csrf_with_state(&headers, &claims, &state)?;
    require_permission_with_state(&state, &claims, "ticket", "redeem")?;

    let mut image_bytes: Vec<u8> = Vec::new();
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("Multipart error: {}", e)))?
    {
        if field.name().unwrap_or("") == "file" {
            image_bytes = field
                .bytes()
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?
                .to_vec();
            break;
        }
    }

    if image_bytes.is_empty() {
        return Err(AppError::Validation("No image file provided".to_string()));
    }

    let img = image::load_from_memory(&image_bytes)
        .map_err(|e| AppError::Validation(format!("Invalid image: {}", e)))?
        .to_luma8();

    let mut prepared = rqrr::PreparedImage::prepare(img);
    let grids = prepared.detect_grids();

    for grid in grids {
        if let Ok((_meta, content)) = grid.decode() {
            return Ok(Json(serde_json::json!({"ticket_value": content})));
        }
    }

    Err(AppError::Validation("No QR code detected in the provided image".to_string()))
}
