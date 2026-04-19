use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::Json;
use crate::app::state::AppState;
use crate::errors::AppError;
use crate::handlers::auth::*;
use crate::models::*;
use crate::security::masking;

pub async fn list_users(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, AppError> {
    let claims = extract_claims_required(&headers, &state.hmac_secret)?;
    require_role(&claims, &UserRole::Administrator)?;
    require_permission_with_state(&state, &claims, "user", "manage")?;

    let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    let users = crate::repositories::users::list_all(&db).map_err(|e| AppError::Internal(e.to_string()))?;

    let masked: Vec<MaskedUser> = users.iter().map(|u| MaskedUser {
        id: u.id.clone(), username: masking::mask_username(&u.username),
        display_name: u.display_name.clone(), role: u.role.as_str().to_string(),
        store_id: u.store_id.clone(),
    }).collect();

    Ok(Json(serde_json::json!({"users": masked})))
}

pub async fn create_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateUserRequest>,
) -> Result<(axum::http::StatusCode, Json<serde_json::Value>), AppError> {
    let claims = extract_claims_required(&headers, &state.hmac_secret)?;
    require_role(&claims, &UserRole::Administrator)?;
    require_csrf_with_state(&headers, &claims, &state)?;
    require_permission_with_state(&state, &claims, "user", "manage")?;

    let password_hash = crate::auth::password::hash_password(&req.password)
        .map_err(|e| AppError::Internal(e))?;
    let user_id = uuid::Uuid::new_v4().to_string();

    let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    crate::repositories::users::create(&db, &user_id, &req.username, &password_hash, &req.display_name, &req.role, req.store_id.as_deref())
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let _ = crate::audit::chain::append_audit_log_secure(
        &db, &claims.user_id, &claims.username, "CREATE", "user", &user_id,
        &serde_json::json!({"username": req.username, "role": req.role}),
        &state.hmac_secret,
    );

    Ok((axum::http::StatusCode::CREATED, Json(serde_json::json!({"id": user_id, "message": "User created"}))))
}

pub async fn update_user_role(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(req): Json<UpdateRoleRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let claims = extract_claims_required(&headers, &state.hmac_secret)?;
    require_role(&claims, &UserRole::Administrator)?;
    require_csrf_with_state(&headers, &claims, &state)?;
    require_permission_with_state(&state, &claims, "role", "manage")?;

    let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    crate::repositories::users::update_role(&db, &id, &req.role).map_err(|e| AppError::Internal(e.to_string()))?;

    let _ = crate::audit::chain::append_audit_log_secure(
        &db, &claims.user_id, &claims.username, "PERMISSION_CHANGE", "user", &id,
        &serde_json::json!({"new_role": req.role}),
        &state.hmac_secret,
    );

    Ok(Json(serde_json::json!({"message": "Role updated"})))
}

pub async fn update_user_active(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(req): Json<UpdateActiveRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let claims = extract_claims_required(&headers, &state.hmac_secret)?;
    require_role(&claims, &UserRole::Administrator)?;
    require_csrf_with_state(&headers, &claims, &state)?;
    require_permission_with_state(&state, &claims, "user", "manage")?;

    let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    crate::repositories::users::update_active(&db, &id, req.active).map_err(|e| AppError::Internal(e.to_string()))?;

    let _ = crate::audit::chain::append_audit_log_secure(
        &db, &claims.user_id, &claims.username, "UPDATE", "user", &id,
        &serde_json::json!({"active": req.active}),
        &state.hmac_secret,
    );

    Ok(Json(serde_json::json!({"message": "User status updated"})))
}

pub async fn issue_recovery_code(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<IssueRecoveryCodeRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let claims = extract_claims_required(&headers, &state.hmac_secret)?;
    require_role(&claims, &UserRole::Administrator)?;
    require_csrf_with_state(&headers, &claims, &state)?;
    require_permission_with_state(&state, &claims, "recovery_code", "issue")?;

    let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    // Verify target user exists
    let _user = crate::repositories::users::find_by_id(&db, &req.user_id)
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    // Generate recovery code
    let code: String = {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        (0..12).map(|_| {
            let idx = rng.gen_range(0..36);
            if idx < 10 { (b'0' + idx) as char } else { (b'A' + idx - 10) as char }
        }).collect()
    };

    let code_hash = hash_recovery_code(&code);
    let expires_at = (chrono::Utc::now() + chrono::Duration::minutes(30))
        .format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let rc = RecoveryCode {
        id: uuid::Uuid::new_v4().to_string(),
        user_id: req.user_id.clone(),
        code_hash,
        issued_by: claims.user_id.clone(),
        issued_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        expires_at: expires_at.clone(),
        used: false,
    };

    crate::repositories::recovery_codes::create(&db, &rc).map_err(|e| AppError::Internal(e.to_string()))?;

    let _ = crate::audit::chain::append_audit_log_secure(
        &db, &claims.user_id, &claims.username, "CREATE", "recovery_code", &rc.id,
        &serde_json::json!({"target_user_id": req.user_id}),
        &state.hmac_secret,
    );

    Ok(Json(serde_json::json!({
        "code": code,
        "expires_at": expires_at,
    })))
}

pub async fn get_audit_log(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let claims = extract_claims_required(&headers, &state.hmac_secret)?;
    require_role(&claims, &UserRole::PlatformOps)?;

    let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    // Honour the permissions table so admin can restrict audit access per role.
    if !crate::repositories::permissions::has_permission(&db, &claims.role, "audit", "read") {
        return Err(AppError::Forbidden("Audit read permission not granted for your role".to_string()));
    }

    let entries = if let (Some(rt), Some(ri)) = (params.get("resource_type"), params.get("resource_id")) {
        crate::repositories::audit::list_by_resource(&db, rt, ri)
    } else {
        let limit = params.get("limit").and_then(|l| l.parse().ok()).unwrap_or(100);
        crate::repositories::audit::list_recent(&db, limit)
    }.map_err(|e| AppError::Internal(e.to_string()))?;

    // Redact actor identifiers: the hash chain already provides integrity, so
    // displaying raw UUIDs in the API response is unnecessary exposure.
    let masked: Vec<serde_json::Value> = entries.iter().map(|e| {
        serde_json::json!({
            "id": e.id,
            "timestamp": e.timestamp,
            "actor_id": masking::mask_user_id(&e.actor_id),
            "actor_username": masking::mask_username(&e.actor_username),
            "action": e.action,
            "resource_type": e.resource_type,
            "resource_id": e.resource_id,
            "details_json": e.details_json,
            "previous_hash": e.previous_hash,
            "current_hash": e.current_hash,
        })
    }).collect();
    Ok(Json(serde_json::json!({"entries": masked, "total": masked.len()})))
}

pub async fn list_permissions(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, AppError> {
    let claims = extract_claims_required(&headers, &state.hmac_secret)?;
    require_role(&claims, &UserRole::Administrator)?;
    require_permission_with_state(&state, &claims, "permission", "manage")?;
    let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    let permissions = crate::repositories::permissions::list_all(&db)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(serde_json::json!({ "permissions": permissions })))
}

pub async fn upsert_permission(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<UpsertPermissionRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let claims = extract_claims_required(&headers, &state.hmac_secret)?;
    require_role(&claims, &UserRole::Administrator)?;
    require_csrf_with_state(&headers, &claims, &state)?;
    require_permission_with_state(&state, &claims, "permission", "manage")?;
    let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    let id = crate::repositories::permissions::upsert(&db, &req.role, &req.resource, &req.action)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let _ = crate::audit::chain::append_audit_log_secure(
        &db,
        &claims.user_id,
        &claims.username,
        "PERMISSION_CHANGE",
        "permission",
        &id,
        &serde_json::json!({"role": req.role, "resource": req.resource, "action": req.action}),
        &state.hmac_secret,
    );
    Ok(Json(serde_json::json!({"id": id, "message": "Permission upserted"})))
}

pub async fn delete_permission(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let claims = extract_claims_required(&headers, &state.hmac_secret)?;
    require_role(&claims, &UserRole::Administrator)?;
    require_csrf_with_state(&headers, &claims, &state)?;
    require_permission_with_state(&state, &claims, "permission", "manage")?;
    let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    crate::repositories::permissions::delete_by_id(&db, &id)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let _ = crate::audit::chain::append_audit_log_secure(
        &db,
        &claims.user_id,
        &claims.username,
        "PERMISSION_CHANGE",
        "permission",
        &id,
        &serde_json::json!({"deleted": true}),
        &state.hmac_secret,
    );
    Ok(Json(serde_json::json!({"message": "Permission deleted"})))
}
