use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::Json;
use crate::app::state::AppState;
use crate::errors::AppError;
use crate::handlers::auth::*;
use crate::models::*;
use crate::security::masking;

/// Normalize datetime-local input ("2026-04-10T09:00") to full seconds format ("2026-04-10T09:00:00")
fn normalize_datetime(input: &str) -> String {
    if input.len() == 16 {
        // "YYYY-MM-DDTHH:MM" -> "YYYY-MM-DDTHH:MM:00"
        format!("{}:00", input)
    } else {
        input.to_string()
    }
}

pub async fn create_reservation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(mut req): Json<CreateReservationRequest>,
) -> Result<(axum::http::StatusCode, Json<serde_json::Value>), AppError> {
    let claims = extract_claims_required(&headers, &state.hmac_secret)?;

    require_csrf_with_state(&headers, &claims, &state)?;

    if matches!(UserRole::from_str(&claims.role), Some(UserRole::Photographer)) {
        return Err(AppError::Forbidden("Photographer role cannot create reservations".to_string()));
    }

    require_permission_with_state(&state, &claims, "reservation", "create")?;

    if req.asset_type != "vehicle" && req.asset_type != "bay" {
        return Err(AppError::Validation("asset_type must be 'vehicle' or 'bay'".to_string()));
    }

    // Normalize datetime inputs (frontend datetime-local may omit seconds)
    req.start_time = normalize_datetime(&req.start_time);
    req.end_time = normalize_datetime(&req.end_time);

    let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    // Verify the asset exists and belongs to the claimed store_id
    if req.asset_type == "vehicle" {
        let vehicle_store: String = db
            .query_row(
                "SELECT store_id FROM vehicles WHERE id = ?1",
                [&req.asset_id],
                |row| row.get(0),
            )
            .map_err(|_| AppError::NotFound("Vehicle not found".to_string()))?;
        if vehicle_store != req.store_id {
            return Err(AppError::Validation("Vehicle does not belong to the specified store".to_string()));
        }
    } else {
        let bay_store: String = db
            .query_row(
                "SELECT store_id FROM service_bays WHERE id = ?1",
                [&req.asset_id],
                |row| row.get(0),
            )
            .map_err(|_| AppError::NotFound("Service bay not found".to_string()))?;
        if bay_store != req.store_id {
            return Err(AppError::Validation("Service bay does not belong to the specified store".to_string()));
        }
    }

    // Store isolation: non-elevated roles can only create reservations in their own store
    enforce_store_isolation(&claims, &req.store_id)?;

    match crate::services::reservation_engine::create_reservation(&db, &claims.user_id, &claims.username, &req, &state.hmac_secret) {
        Ok(result) => {
            let masked_reservation = serde_json::json!({
                "id": result.reservation.id,
                "asset_type": result.reservation.asset_type,
                "asset_id": result.reservation.asset_id,
                "store_id": result.reservation.store_id,
                "user_id": masking::mask_user_id(&result.reservation.user_id),
                "start_time": result.reservation.start_time,
                "end_time": result.reservation.end_time,
                "status": result.reservation.status,
                "ticket_id": result.reservation.ticket_id,
                "version": result.reservation.version,
            });
            Ok((axum::http::StatusCode::CREATED, Json(serde_json::json!({
                "reservation": masked_reservation,
                "ticket": result.ticket,
            }))))
        }
        Err(conflict) => {
            let is_validation = conflict.reasons.iter().any(|r| r.code == "validation");
            let status = if is_validation {
                axum::http::StatusCode::BAD_REQUEST
            } else {
                axum::http::StatusCode::CONFLICT
            };
            Ok((status, Json(serde_json::to_value(conflict).unwrap())))
        }
    }
}

pub async fn list_reservations(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let claims = extract_claims_required(&headers, &state.hmac_secret)?;

    let read_action = match UserRole::from_str(&claims.role) {
        Some(UserRole::Customer) => "read_own",
        Some(UserRole::Photographer) => "read_assigned",
        Some(UserRole::MerchantStaff) => "read_store",
        Some(UserRole::PlatformOps) | Some(UserRole::Administrator) => "read_all",
        None => return Err(AppError::Forbidden("Invalid role".to_string())),
    };
    require_permission_with_state(&state, &claims, "reservation", read_action)?;

    let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    let reservations = match UserRole::from_str(&claims.role) {
        Some(UserRole::Customer) => {
            crate::repositories::reservations::find_by_user(&db, &claims.user_id)
        }
        // Photographers see only reservations for their directly assigned assets.
        Some(UserRole::Photographer) => {
            crate::repositories::reservations::find_for_photographer(&db, &claims.user_id)
        }
        Some(UserRole::PlatformOps) | Some(UserRole::Administrator) => {
            if let Some(store_id) = params.get("store_id") {
                crate::repositories::reservations::find_by_store(&db, store_id)
            } else {
                crate::repositories::reservations::find_all(&db)
            }
        }
        _ => {
            let store_id = claims.store_id.as_deref()
                .ok_or_else(|| AppError::Forbidden("No store assigned".to_string()))?;
            crate::repositories::reservations::find_by_store(&db, store_id)
        }
    }.map_err(|e| AppError::Internal(e.to_string()))?;

    let masked: Vec<serde_json::Value> = reservations.iter().map(|r| serde_json::json!({
        "id": r.id,
        "asset_type": r.asset_type,
        "asset_id": r.asset_id,
        "store_id": r.store_id,
        "user_id": masking::mask_user_id(&r.user_id),
        "start_time": r.start_time,
        "end_time": r.end_time,
        "status": r.status,
        "ticket_id": r.ticket_id,
        "version": r.version,
    })).collect();
    Ok(Json(serde_json::json!({"reservations": masked})))
}
