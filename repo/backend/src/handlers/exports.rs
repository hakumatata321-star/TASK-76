use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use crate::app::state::AppState;
use crate::errors::AppError;
use crate::handlers::auth::*;
use crate::models::*;
use crate::security::masking;

pub async fn export_data(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(params): Json<ExportQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let claims = extract_claims_required(&headers, &state.hmac_secret)?;
    require_role(&claims, &UserRole::PlatformOps)?;
    // State-changing audit write requires CSRF.
    require_csrf_with_state(&headers, &claims, &state)?;

    let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    // Permission check: admin can revoke export access from any role via the
    // permissions table without a code deploy.
    if !crate::repositories::permissions::has_permission(&db, &claims.role, "export", "create") {
        return Err(AppError::Forbidden("Export permission not granted for your role".to_string()));
    }

    let reservations = if let Some(ref store_id) = params.store_id {
        crate::repositories::reservations::find_by_store(&db, store_id)
    } else {
        crate::repositories::reservations::find_all(&db)
    }.map_err(|e| AppError::Internal(e.to_string()))?;

    let vehicles = if let Some(ref store_id) = params.store_id {
        crate::repositories::vehicles::find_by_store(&db, store_id)
    } else {
        crate::repositories::vehicles::find_all(&db)
    }.map_err(|e| AppError::Internal(e.to_string()))?;

    // Mask vehicle sensitive fields.
    let masked_vehicles: Vec<serde_json::Value> = vehicles.iter().map(|v| {
        serde_json::json!({
            "id": v.id, "make": v.make, "model": v.model,
            "status": v.status, "store_id": v.store_id,
        })
    }).collect();

    // Mask reservation user identifiers — raw UUIDs must not appear in exports.
    let masked_reservations: Vec<serde_json::Value> = reservations.iter().map(|r| {
        serde_json::json!({
            "id": r.id,
            "asset_type": r.asset_type,
            "asset_id": r.asset_id,
            "store_id": r.store_id,
            "user_id": masking::mask_user_id(&r.user_id),
            "start_time": r.start_time,
            "end_time": r.end_time,
            "status": r.status,
        })
    }).collect();

    let _ = crate::audit::chain::append_audit_log_secure(
        &db, &claims.user_id, &claims.username, "EXPORT", "export", "",
        &serde_json::json!({"store_id": params.store_id, "type": params.export_type}),
        &state.hmac_secret,
    );

    Ok(Json(serde_json::json!({
        "export_type": params.export_type.unwrap_or("all".into()),
        "reservations": masked_reservations,
        "vehicles": masked_vehicles,
        "exported_at": chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
    })))
}
