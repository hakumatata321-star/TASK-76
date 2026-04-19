use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::Json;
use crate::app::state::AppState;
use crate::errors::AppError;
use crate::handlers::auth::*;
use crate::models::*;

pub async fn list_bays(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let claims = extract_claims_required(&headers, &state.hmac_secret)?;
    require_role(&claims, &UserRole::MerchantStaff)?;

    let read_action = match UserRole::from_str(&claims.role) {
        Some(UserRole::PlatformOps) | Some(UserRole::Administrator) => "read_all",
        Some(UserRole::MerchantStaff) => "read_store",
        _ => return Err(AppError::Forbidden("Invalid role for bay listing".to_string())),
    };
    require_permission_with_state(&state, &claims, "bay", read_action)?;

    let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    // Determine effective store_id with isolation enforcement
    let role = UserRole::from_str(&claims.role).unwrap_or(UserRole::Customer);
    let store_id = if matches!(role, UserRole::PlatformOps | UserRole::Administrator) {
        params.get("store_id").cloned()
            .or_else(|| claims.store_id.clone())
            .ok_or_else(|| AppError::Validation("store_id required".to_string()))?
    } else {
        // MerchantStaff: always use their own store, reject if client tries another
        let user_store = claims.store_id.as_ref()
            .ok_or_else(|| AppError::Forbidden("No store assigned".to_string()))?;
        if let Some(requested) = params.get("store_id") {
            if requested != user_store {
                return Err(AppError::Forbidden("Access denied: cannot view bays from another store".to_string()));
            }
        }
        user_store.clone()
    };

    let bays = crate::repositories::bays::find_by_store(&db, &store_id)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!({"bays": bays})))
}

pub async fn create_bay(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateBayRequest>,
) -> Result<(axum::http::StatusCode, Json<ServiceBay>), AppError> {
    let claims = extract_claims_required(&headers, &state.hmac_secret)?;
    require_role(&claims, &UserRole::MerchantStaff)?;
    require_csrf_with_state(&headers, &claims, &state)?;
    require_permission_with_state(&state, &claims, "bay", "create")?;

    // Store isolation
    enforce_store_isolation(&claims, &req.store_id)?;

    let bay = ServiceBay {
        id: uuid::Uuid::new_v4().to_string(),
        store_id: req.store_id.clone(),
        name: req.name.clone(),
        bay_type: req.bay_type.clone(),
        capacity: req.capacity.unwrap_or(1),
        status: "active".to_string(),
        version: 1,
    };

    let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    crate::repositories::bays::create(&db, &bay).map_err(|e| AppError::Internal(e.to_string()))?;

    let _ = crate::audit::chain::append_audit_log_secure(
        &db, &claims.user_id, &claims.username, "CREATE", "bay", &bay.id,
        &serde_json::json!({"name": req.name, "store_id": req.store_id}),
        &state.hmac_secret,
    );

    Ok((axum::http::StatusCode::CREATED, Json(bay)))
}
