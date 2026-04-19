use axum::extract::{Multipart, State};
use axum::http::HeaderMap;
use axum::Json;
use crate::app::state::AppState;
use crate::errors::AppError;
use crate::handlers::auth::*;
use crate::models::*;

pub async fn upload_file(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<(axum::http::StatusCode, Json<Upload>), AppError> {
    let claims = extract_claims_required(&headers, &state.hmac_secret)?;
    require_role(&claims, &UserRole::MerchantStaff)?;
    require_csrf_with_state(&headers, &claims, &state)?;
    require_permission_with_state(&state, &claims, "upload", "create")?;

    let mut file_data: Option<Vec<u8>> = None;
    let mut filename = String::new();
    let mut vehicle_id: Option<String> = None;
    let mut store_id: Option<String> = claims.store_id.clone();

    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::Upload(e.to_string()))? {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                filename = field.file_name().unwrap_or("upload").to_string();
                file_data = Some(field.bytes().await.map_err(|e| AppError::Upload(e.to_string()))?.to_vec());
            }
            "vehicle_id" => {
                vehicle_id = Some(field.text().await.map_err(|e| AppError::Upload(e.to_string()))?);
            }
            "store_id" => {
                store_id = Some(field.text().await.map_err(|e| AppError::Upload(e.to_string()))?);
            }
            _ => {}
        }
    }

    let data = file_data.ok_or_else(|| AppError::Upload("No file provided".to_string()))?;

    // Enforce store isolation: ignore client-supplied store_id for non-elevated roles
    let role = UserRole::from_str(&claims.role).unwrap_or(UserRole::Customer);
    if !matches!(role, UserRole::PlatformOps | UserRole::Administrator) {
        store_id = claims.store_id.clone();
    }
    if let Some(ref sid) = store_id {
        enforce_store_isolation(&claims, sid)?;
    }

    let validated = crate::services::uploads::validate_upload(&data, &filename)
        .map_err(|e| AppError::Upload(e))?;

    let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    if crate::services::uploads::check_duplicate(&db, &validated.fingerprint)
        .map_err(|e| AppError::Internal(e.to_string()))? {
        return Err(AppError::Conflict("Duplicate file: a file with the same content already exists".to_string()));
    }

    let upload_id = uuid::Uuid::new_v4().to_string();
    let stored_filename = format!("{}.{}", upload_id, if validated.content_type == "image/jpeg" { "jpg" } else { "png" });
    let file_path = format!("{}/{}", state.upload_dir, stored_filename);
    std::fs::write(&file_path, &validated.data).map_err(|e| AppError::Internal(e.to_string()))?;

    let upload = Upload {
        id: upload_id.clone(), filename: filename.clone(), content_type: validated.content_type,
        size_bytes: validated.data.len() as i64, sha256_fingerprint: validated.fingerprint,
        vehicle_id: vehicle_id.clone(), store_id: store_id.clone(), uploader_id: claims.user_id.clone(),
    };

    crate::repositories::uploads::create(&db, &upload).map_err(|e| AppError::Internal(e.to_string()))?;

    let _ = crate::audit::chain::append_audit_log_secure(
        &db, &claims.user_id, &claims.username, "CREATE", "upload", &upload_id,
        &serde_json::json!({"filename": filename, "vehicle_id": vehicle_id}),
        &state.hmac_secret,
    );

    Ok((axum::http::StatusCode::CREATED, Json(upload)))
}
