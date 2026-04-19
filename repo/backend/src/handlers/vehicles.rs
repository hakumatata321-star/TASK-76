use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::Json;
use crate::app::state::AppState;
use crate::errors::AppError;
use crate::handlers::auth::*;
use crate::models::*;
use crate::security::{encryption, masking};
use crate::repositories::permissions;

pub async fn list_vehicles(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let claims = extract_claims_required(&headers, &state.hmac_secret)?;
    require_role(&claims, &UserRole::MerchantStaff)?;

    let read_action = match UserRole::from_str(&claims.role) {
        Some(UserRole::PlatformOps) | Some(UserRole::Administrator) => "read_all",
        Some(UserRole::MerchantStaff) => "read_store",
        _ => return Err(AppError::Forbidden("Invalid role for vehicle listing".to_string())),
    };
    require_permission_with_state(&state, &claims, "vehicle", read_action)?;

    let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    let vehicles = match UserRole::from_str(&claims.role) {
        Some(UserRole::PlatformOps) | Some(UserRole::Administrator) => {
            if let Some(store_id) = params.get("store_id") {
                crate::repositories::vehicles::find_by_store(&db, store_id)
            } else {
                crate::repositories::vehicles::find_all(&db)
            }
        }
        _ => {
            let store_id = claims.store_id.as_deref()
                .ok_or_else(|| AppError::Forbidden("No store assigned".to_string()))?;
            crate::repositories::vehicles::find_by_store(&db, store_id)
        }
    }.map_err(|e| AppError::Internal(e.to_string()))?;

    let masked: Vec<MaskedVehicle> = vehicles.iter().map(|v| {
        let vin = encryption::decrypt_field(&v.vin_encrypted, &state.encryption_key)
            .unwrap_or_else(|_| "DECRYPTION_ERROR".to_string());
        let plate = encryption::decrypt_field(&v.license_plate_encrypted, &state.encryption_key)
            .unwrap_or_else(|_| "DECRYPTION_ERROR".to_string());
        MaskedVehicle {
            id: v.id.clone(),
            vin: masking::mask_vin(&vin),
            license_plate: masking::mask_license_plate(&plate),
            make: v.make.clone(), model: v.model.clone(), trim_level: v.trim_level.clone(),
            store_id: v.store_id.clone(), mileage_miles: v.mileage_miles,
            fuel_or_battery_pct: v.fuel_or_battery_pct, status: v.status.clone(),
            maintenance_due: v.maintenance_due.clone(), inspection_due: v.inspection_due.clone(),
            insurance_expiry: v.insurance_expiry.clone(),
        }
    }).collect();

    Ok(Json(serde_json::json!({"vehicles": masked, "total": masked.len()})))
}

pub async fn get_vehicle(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<MaskedVehicle>, AppError> {
    let claims = extract_claims_required(&headers, &state.hmac_secret)?;
    require_role(&claims, &UserRole::MerchantStaff)?;

    let read_action = match UserRole::from_str(&claims.role) {
        Some(UserRole::PlatformOps) | Some(UserRole::Administrator) => "read_all",
        Some(UserRole::MerchantStaff) => "read_store",
        _ => return Err(AppError::Forbidden("Invalid role for vehicle read".to_string())),
    };
    require_permission_with_state(&state, &claims, "vehicle", read_action)?;

    let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    let v = crate::repositories::vehicles::find_by_id(&db, &id)
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Vehicle not found".to_string()))?;

    // Store isolation check
    if let Some(ref user_store) = claims.store_id {
        let role = UserRole::from_str(&claims.role).unwrap_or(UserRole::Customer);
        if !matches!(role, UserRole::PlatformOps | UserRole::Administrator) && v.store_id != *user_store {
            return Err(AppError::Forbidden("Access denied: vehicle belongs to another store".to_string()));
        }
    }

    let vin = encryption::decrypt_field(&v.vin_encrypted, &state.encryption_key)
        .unwrap_or_else(|_| "DECRYPTION_ERROR".to_string());
    let plate = encryption::decrypt_field(&v.license_plate_encrypted, &state.encryption_key)
        .unwrap_or_else(|_| "DECRYPTION_ERROR".to_string());

    Ok(Json(MaskedVehicle {
        id: v.id, vin: masking::mask_vin(&vin), license_plate: masking::mask_license_plate(&plate),
        make: v.make, model: v.model, trim_level: v.trim_level, store_id: v.store_id,
        mileage_miles: v.mileage_miles, fuel_or_battery_pct: v.fuel_or_battery_pct,
        status: v.status, maintenance_due: v.maintenance_due,
        inspection_due: v.inspection_due, insurance_expiry: v.insurance_expiry,
    }))
}

pub async fn create_vehicle(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateVehicleRequest>,
) -> Result<(axum::http::StatusCode, Json<MaskedVehicle>), AppError> {
    let claims = extract_claims_required(&headers, &state.hmac_secret)?;
    require_role(&claims, &UserRole::MerchantStaff)?;
    require_csrf_with_state(&headers, &claims, &state)?;

    {
        let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        if !permissions::has_permission(&db, &claims.role, "vehicle", "create") {
            return Err(AppError::Forbidden("Vehicle creation permission not granted for your role".to_string()));
        }
    }

    // Store isolation
    if let Some(ref user_store) = claims.store_id {
        let role = UserRole::from_str(&claims.role).unwrap_or(UserRole::Customer);
        if !matches!(role, UserRole::Administrator) && req.store_id != *user_store {
            return Err(AppError::Forbidden("Cannot create vehicles for another store".to_string()));
        }
    }

    let vin_encrypted = encryption::encrypt_field(&req.vin, &state.encryption_key)
        .map_err(|e| AppError::Internal(e))?;
    let vin_hash = hash_field(&req.vin);
    let plate_encrypted = encryption::encrypt_field(&req.license_plate, &state.encryption_key)
        .map_err(|e| AppError::Internal(e))?;
    let plate_hash = hash_field(&req.license_plate);

    let vehicle = Vehicle {
        id: uuid::Uuid::new_v4().to_string(),
        vin_encrypted, vin_hash, license_plate_encrypted: plate_encrypted, license_plate_hash: plate_hash,
        make: req.make.clone(), model: req.model.clone(),
        trim_level: req.trim_level.clone().unwrap_or_default(),
        store_id: req.store_id.clone(),
        mileage_miles: req.mileage_miles.unwrap_or(0),
        fuel_or_battery_pct: req.fuel_or_battery_pct.unwrap_or(100.0),
        status: "available".to_string(),
        maintenance_due: req.maintenance_due.clone(),
        inspection_due: req.inspection_due.clone(),
        insurance_expiry: req.insurance_expiry.clone(),
        version: 1,
    };

    let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    crate::repositories::vehicles::create(&db, &vehicle)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let _ = crate::audit::chain::append_audit_log_secure(
        &db, &claims.user_id, &claims.username, "CREATE", "vehicle", &vehicle.id,
        &serde_json::json!({"make": req.make, "model": req.model, "store_id": req.store_id}),
        &state.hmac_secret,
    );

    Ok((axum::http::StatusCode::CREATED, Json(MaskedVehicle {
        id: vehicle.id, vin: masking::mask_vin(&req.vin),
        license_plate: masking::mask_license_plate(&req.license_plate),
        make: req.make, model: req.model, trim_level: req.trim_level.unwrap_or_default(),
        store_id: req.store_id, mileage_miles: req.mileage_miles.unwrap_or(0),
        fuel_or_battery_pct: req.fuel_or_battery_pct.unwrap_or(100.0),
        status: "available".to_string(), maintenance_due: req.maintenance_due,
        inspection_due: req.inspection_due, insurance_expiry: req.insurance_expiry,
    })))
}

pub async fn update_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(req): Json<StatusTransitionRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let claims = extract_claims_required(&headers, &state.hmac_secret)?;
    require_role(&claims, &UserRole::MerchantStaff)?;
    require_csrf_with_state(&headers, &claims, &state)?;

    let new_status = VehicleStatus::from_str(&req.status)
        .ok_or_else(|| AppError::Validation(format!("Invalid status: {}", req.status)))?;

    let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    if !permissions::has_permission(&db, &claims.role, "vehicle", "status_transition") {
        return Err(AppError::Forbidden("Vehicle status transition permission not granted for your role".to_string()));
    }
    let vehicle = crate::repositories::vehicles::find_by_id(&db, &id)
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Vehicle not found".to_string()))?;

    // Store isolation
    if let Some(ref user_store) = claims.store_id {
        let role = UserRole::from_str(&claims.role).unwrap_or(UserRole::Customer);
        if !matches!(role, UserRole::PlatformOps | UserRole::Administrator) && vehicle.store_id != *user_store {
            return Err(AppError::Forbidden("Access denied".to_string()));
        }
    }

    let current = VehicleStatus::from_str(&vehicle.status)
        .ok_or_else(|| AppError::Internal("Invalid current status".to_string()))?;

    if !current.can_transition_to(&new_status) {
        return Err(AppError::Validation(format!(
            "Cannot transition from {} to {}", current.as_str(), new_status.as_str()
        )));
    }

    if current.requires_admin(&new_status) {
        require_role(&claims, &UserRole::Administrator)?;
    }

    let updated = crate::repositories::vehicles::update_status(&db, &id, new_status.as_str(), vehicle.version)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if !updated {
        return Err(AppError::Conflict("Version conflict, please retry".to_string()));
    }

    let _ = crate::audit::chain::append_audit_log_secure(
        &db, &claims.user_id, &claims.username, "STATUS_CHANGE", "vehicle", &id,
        &serde_json::json!({"from": current.as_str(), "to": new_status.as_str()}),
        &state.hmac_secret,
    );

    Ok(Json(serde_json::json!({"message": "Status updated", "status": new_status.as_str()})))
}

fn hash_field(value: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hex::encode(hasher.finalize())
}
