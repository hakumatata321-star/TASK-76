use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use crate::app::state::AppState;
use crate::auth::{csrf, password, session};
use crate::errors::AppError;
use crate::models::*;
use crate::security::masking;

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    let user = crate::repositories::users::find_by_username(&db, &req.username)
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::Auth("Invalid credentials".to_string()))?;

    if !user.active {
        return Err(AppError::Auth("Account is disabled".to_string()));
    }

    if !password::verify_password(&req.password, &user.password_hash) {
        return Err(AppError::Auth("Invalid credentials".to_string()));
    }

    let token = session::create_token(
        &user.id, &user.username, user.role.as_str(),
        user.store_id.as_deref(), &state.hmac_secret,
    );
    let csrf_token = csrf::generate_csrf_token();

    // Store CSRF token server-side bound to this user's session
    {
        let mut csrf_map = state.csrf_tokens.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        csrf_map.insert(user.id.clone(), csrf_token.clone());
    }

    let _ = crate::audit::chain::append_audit_log_secure(
        &db, &user.id, &user.username, "LOGIN", "user", &user.id,
        &serde_json::json!({"method": "password"}),
        &state.hmac_secret,
    );

    Ok(Json(LoginResponse {
        token,
        csrf_token,
        user: MaskedUser {
            id: user.id,
            username: masking::mask_username(&user.username),
            display_name: user.display_name,
            role: user.role.as_str().to_string(),
            store_id: user.store_id,
        },
    }))
}

pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, AppError> {
    let claims = extract_claims_required(&headers, &state.hmac_secret)?;
    require_csrf_with_state(&headers, &claims, &state)?;
    // Revoke this session so the bearer token is rejected on all future requests
    if let Ok(mut revoked) = state.revoked_sessions.lock() {
        revoked.insert(format!("{}:{}", claims.user_id, claims.iat));
    }
    // Remove CSRF token for this session
    if let Ok(mut csrf_map) = state.csrf_tokens.lock() {
        csrf_map.remove(&claims.user_id);
    }
    let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    let _ = crate::audit::chain::append_audit_log_secure(
        &db, &claims.user_id, &claims.username, "LOGOUT", "user", &claims.user_id,
        &serde_json::json!({}),
        &state.hmac_secret,
    );
    Ok(Json(serde_json::json!({"message": "Logged out"})))
}

pub async fn me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<MeResponse>, AppError> {
    let claims = extract_claims_required(&headers, &state.hmac_secret)?;
    let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    let user = crate::repositories::users::find_by_id(&db, &claims.user_id)
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
    if !user.active {
        return Err(AppError::Auth("Account is disabled".to_string()));
    }

    // Reissue token with fresh iat for idle timeout refresh
    let refreshed_token = session::create_token(
        &user.id, &user.username, user.role.as_str(),
        user.store_id.as_deref(), &state.hmac_secret,
    );

    Ok(Json(MeResponse {
        user: MaskedUser {
            id: user.id,
            username: masking::mask_username(&user.username),
            display_name: user.display_name,
            role: user.role.as_str().to_string(),
            store_id: user.store_id,
        },
        refreshed_token,
    }))
}

pub async fn reset_password(
    State(state): State<AppState>,
    Json(req): Json<ResetPasswordRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    let user = crate::repositories::users::find_by_username(&db, &req.username)
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::Validation("Invalid username or recovery code".to_string()))?;

    let code_hash = hash_recovery_code(&req.recovery_code);
    let rc = crate::repositories::recovery_codes::find_valid(&db, &user.id, &code_hash)
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::Validation("Invalid or expired recovery code".to_string()))?;

    let new_hash = password::hash_password(&req.new_password)
        .map_err(|e| AppError::Internal(e))?;
    crate::repositories::users::update_password(&db, &user.id, &new_hash)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    crate::repositories::recovery_codes::mark_used(&db, &rc.id)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let _ = crate::audit::chain::append_audit_log_secure(
        &db, &user.id, &user.username, "PASSWORD_RESET", "user", &user.id,
        &serde_json::json!({"method": "recovery_code"}),
        &state.hmac_secret,
    );

    Ok(Json(serde_json::json!({"message": "Password reset successful"})))
}

pub fn extract_claims(headers: &HeaderMap, secret: &str) -> Option<Claims> {
    let auth_header = headers.get("authorization")?.to_str().ok()?;
    let token = auth_header.strip_prefix("Bearer ")?;
    session::validate_token(token, secret)
}

pub fn extract_claims_required(headers: &HeaderMap, secret: &str) -> Result<Claims, AppError> {
    extract_claims(headers, secret).ok_or_else(|| AppError::Auth("Authentication required".to_string()))
}

pub fn require_role(claims: &Claims, min_role: &UserRole) -> Result<(), AppError> {
    let user_role = UserRole::from_str(&claims.role)
        .ok_or_else(|| AppError::Forbidden("Invalid role".to_string()))?;
    if !user_role.has_at_least(min_role) {
        return Err(AppError::Forbidden(format!("Requires {} or higher role", min_role.as_str())));
    }
    Ok(())
}

pub fn require_permission_with_state(
    state: &AppState,
    claims: &Claims,
    resource: &str,
    action: &str,
) -> Result<(), AppError> {
    let db = state
        .db
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    if crate::repositories::permissions::has_permission(&db, &claims.role, resource, action) {
        Ok(())
    } else {
        Err(AppError::Forbidden(format!(
            "Permission denied for {}:{}",
            resource, action
        )))
    }
}

/// Validate CSRF token against the server-side store, bound to the user's session.
pub fn require_csrf(headers: &HeaderMap, claims: &Claims) -> Result<(), AppError> {
    let _ = headers;
    let _ = claims;
    Err(AppError::Forbidden(
        "Deprecated CSRF validator in use; handlers must call require_csrf_with_state".to_string(),
    ))
}

/// Validate CSRF token against the server-side session-bound store.
pub fn require_csrf_with_state(headers: &HeaderMap, claims: &Claims, state: &AppState) -> Result<(), AppError> {
    let provided = headers
        .get("x-csrf-token")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if provided.is_empty() {
        return Err(AppError::Forbidden("CSRF token required for state-changing requests".to_string()));
    }

    let csrf_map = state.csrf_tokens.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    let expected = csrf_map.get(&claims.user_id)
        .ok_or_else(|| AppError::Forbidden("No CSRF session found; please re-authenticate".to_string()))?;

    if !crate::auth::csrf::validate_csrf_token(provided, expected) {
        return Err(AppError::Forbidden("Invalid CSRF token".to_string()));
    }

    Ok(())
}

pub fn hash_recovery_code(code: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(code.as_bytes());
    hex::encode(hasher.finalize())
}

/// Enforce store isolation: for MerchantStaff/Photographer, the target store_id must match
/// the user's assigned store. PlatformOps and Administrator bypass this check.
pub fn enforce_store_isolation(claims: &Claims, target_store_id: &str) -> Result<(), AppError> {
    let role = UserRole::from_str(&claims.role).unwrap_or(UserRole::Customer);
    if matches!(role, UserRole::PlatformOps | UserRole::Administrator) {
        return Ok(());
    }
    match &claims.store_id {
        Some(user_store) if user_store == target_store_id => Ok(()),
        Some(_) => Err(AppError::Forbidden("Access denied: resource belongs to another store".to_string())),
        None => Err(AppError::Forbidden("No store assigned to your account".to_string())),
    }
}
