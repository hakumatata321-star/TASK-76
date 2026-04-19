use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use crate::app::state::AppState;
use crate::errors::AppError;
use crate::handlers::auth::*;
use crate::models::*;

pub async fn list_assignments(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, AppError> {
    let claims = extract_claims_required(&headers, &state.hmac_secret)?;

    let read_action = match UserRole::from_str(&claims.role) {
        Some(UserRole::Photographer) => "read_own",
        Some(UserRole::MerchantStaff) => "manage_store",
        Some(UserRole::PlatformOps) | Some(UserRole::Administrator) => "read_all",
        Some(UserRole::Customer) | None => {
            return Err(AppError::Forbidden("Customers do not have access to assignments".to_string()));
        }
    };
    require_permission_with_state(&state, &claims, "assignment", read_action)?;

    let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    let assignments = match UserRole::from_str(&claims.role) {
        Some(UserRole::Photographer) => {
            crate::repositories::assignments::find_by_photographer(&db, &claims.user_id)
        }
        Some(UserRole::MerchantStaff) => {
            let store_id = claims.store_id.as_deref()
                .ok_or_else(|| AppError::Forbidden("No store assigned".to_string()))?;
            crate::repositories::assignments::find_by_store(&db, store_id)
        }
        Some(UserRole::PlatformOps) | Some(UserRole::Administrator) => {
            // Return all (simplified - would normally paginate)
            let stores = crate::repositories::stores::find_all(&db)
                .map_err(|e| AppError::Internal(e.to_string()))?;
            let mut all = Vec::new();
            for s in stores {
                let mut a = crate::repositories::assignments::find_by_store(&db, &s.id)
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                all.append(&mut a);
            }
            Ok(all)
        }
        Some(UserRole::Customer) | None => {
            return Err(AppError::Forbidden("Customers do not have access to assignments".to_string()));
        }
    }.map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!({"assignments": assignments})))
}

pub async fn create_assignment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateAssignmentRequest>,
) -> Result<(axum::http::StatusCode, Json<PhotographerAssignment>), AppError> {
    let claims = extract_claims_required(&headers, &state.hmac_secret)?;
    require_role(&claims, &UserRole::MerchantStaff)?;
    require_csrf_with_state(&headers, &claims, &state)?;
    require_permission_with_state(&state, &claims, "assignment", "manage_store")?;

    // Store isolation
    if let Some(ref user_store) = claims.store_id {
        let role = UserRole::from_str(&claims.role).unwrap_or(UserRole::Customer);
        if !matches!(role, UserRole::PlatformOps | UserRole::Administrator) && req.store_id != *user_store {
            return Err(AppError::Forbidden("Cannot create assignments for another store".to_string()));
        }
    }

    let assignment = PhotographerAssignment {
        id: uuid::Uuid::new_v4().to_string(),
        photographer_user_id: req.photographer_user_id,
        store_id: req.store_id,
        job_description: req.job_description,
        vehicle_id: req.vehicle_id,
        bay_id: req.bay_id,
        start_time: req.start_time,
        end_time: req.end_time,
    };

    let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    crate::repositories::assignments::create(&db, &assignment).map_err(|e| AppError::Internal(e.to_string()))?;

    let _ = crate::audit::chain::append_audit_log_secure(
        &db, &claims.user_id, &claims.username, "CREATE", "assignment", &assignment.id,
        &serde_json::json!({"photographer": &assignment.photographer_user_id}),
        &state.hmac_secret,
    );

    Ok((axum::http::StatusCode::CREATED, Json(assignment)))
}
