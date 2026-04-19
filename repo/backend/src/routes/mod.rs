use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::{get, post, put},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::app::state::AppState;
use crate::handlers;
use crate::security::headers::security_headers;

fn user_matches_claims(state: &AppState, user_id: &str, role_claim: &str) -> Result<bool, StatusCode> {
    let db = state.db.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user = crate::repositories::users::find_by_id(&db, user_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(user.map_or(false, |u| u.active && u.role.as_str() == role_claim))
}

/// Returns true if the session identified by user_id:iat was explicitly revoked via logout.
fn is_session_revoked(state: &AppState, user_id: &str, iat: i64) -> bool {
    let key = format!("{}:{}", user_id, iat);
    state
        .revoked_sessions
        .lock()
        .map(|r| r.contains(&key))
        .unwrap_or(false)
}

/// Auth middleware that rejects unauthenticated requests
async fn require_auth(
    State(state): State<AppState>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if let Some(token) = auth_header.strip_prefix("Bearer ") {
        if let Some(claims) = crate::auth::session::validate_token(token, &state.hmac_secret) {
            if is_session_revoked(&state, &claims.user_id, claims.iat) {
                return Err(StatusCode::UNAUTHORIZED);
            }
            let authorized = user_matches_claims(&state, &claims.user_id, &claims.role)?;
            if authorized {
                return Ok(next.run(request).await);
            }
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}

/// Auth middleware that requires MerchantStaff or higher role
async fn require_staff(
    State(state): State<AppState>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let claims = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .and_then(|t| crate::auth::session::validate_token(t, &state.hmac_secret));

    match claims {
        Some(c) => {
            if is_session_revoked(&state, &c.user_id, c.iat) {
                return Err(StatusCode::UNAUTHORIZED);
            }
            let role = crate::models::UserRole::from_str(&c.role);
            let authorized = user_matches_claims(&state, &c.user_id, &c.role)?;
            if authorized
                && role.map_or(false, |r| r.has_at_least(&crate::models::UserRole::MerchantStaff))
            {
                Ok(next.run(request).await)
            } else {
                Err(StatusCode::FORBIDDEN)
            }
        }
        None => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Auth middleware that requires PlatformOps or higher role
async fn require_ops(
    State(state): State<AppState>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let claims = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .and_then(|t| crate::auth::session::validate_token(t, &state.hmac_secret));

    match claims {
        Some(c) => {
            if is_session_revoked(&state, &c.user_id, c.iat) {
                return Err(StatusCode::UNAUTHORIZED);
            }
            let role = crate::models::UserRole::from_str(&c.role);
            let authorized = user_matches_claims(&state, &c.user_id, &c.role)?;
            if authorized
                && role.map_or(false, |r| r.has_at_least(&crate::models::UserRole::PlatformOps))
            {
                Ok(next.run(request).await)
            } else {
                Err(StatusCode::FORBIDDEN)
            }
        }
        None => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Auth middleware that requires Administrator role
async fn require_admin(
    State(state): State<AppState>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let claims = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .and_then(|t| crate::auth::session::validate_token(t, &state.hmac_secret));

    match claims {
        Some(c) => {
            if is_session_revoked(&state, &c.user_id, c.iat) {
                return Err(StatusCode::UNAUTHORIZED);
            }
            let role = crate::models::UserRole::from_str(&c.role);
            let authorized = user_matches_claims(&state, &c.user_id, &c.role)?;
            if authorized
                && role.map_or(false, |r| r.has_at_least(&crate::models::UserRole::Administrator))
            {
                Ok(next.run(request).await)
            } else {
                Err(StatusCode::FORBIDDEN)
            }
        }
        None => Err(StatusCode::UNAUTHORIZED),
    }
}

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/api/auth/login", post(handlers::auth::login))
        .route("/api/auth/reset-password", post(handlers::auth::reset_password));

    // Authenticated routes (any role) - protected by require_auth middleware
    let auth_routes = Router::new()
        .route("/api/auth/logout", post(handlers::auth::logout))
        .route("/api/auth/me", get(handlers::auth::me))
        .route("/api/reservations", post(handlers::reservations::create_reservation))
        .route("/api/reservations", get(handlers::reservations::list_reservations))
        .route("/api/tickets/:id", get(handlers::tickets::get_ticket))
        .route("/api/assignments", get(handlers::assignments::list_assignments))
        .route_layer(middleware::from_fn_with_state(state.clone(), require_auth));

    // Staff routes (MerchantStaff+) - protected by require_staff middleware
    let staff_routes = Router::new()
        .route("/api/vehicles", get(handlers::vehicles::list_vehicles))
        .route("/api/vehicles/:id", get(handlers::vehicles::get_vehicle))
        .route("/api/vehicles", post(handlers::vehicles::create_vehicle))
        .route("/api/vehicles/:id/status", put(handlers::vehicles::update_status))
        .route("/api/bays", get(handlers::bays::list_bays))
        .route("/api/bays", post(handlers::bays::create_bay))
        .route("/api/stores", get(handlers::stores::list_stores))
        .route("/api/calendar", get(handlers::calendar::get_calendar))
        .route("/api/tickets/:id/redeem", post(handlers::tickets::redeem_ticket))
        .route("/api/tickets/:id/undo", post(handlers::tickets::undo_redemption))
        .route("/api/uploads", post(handlers::uploads::upload_file))
        .route("/api/assignments", post(handlers::assignments::create_assignment))
        .route("/api/tickets/scan", post(handlers::scan::scan_qr))
        .route_layer(middleware::from_fn_with_state(state.clone(), require_staff));

    // Operations routes (PlatformOps+) - protected by require_ops middleware
    let ops_routes = Router::new()
        // POST because export writes an audit entry (state change requires CSRF).
        .route("/api/exports", post(handlers::exports::export_data))
        .route("/api/audit", get(handlers::admin::get_audit_log))
        .route_layer(middleware::from_fn_with_state(state.clone(), require_ops));

    // Admin routes (Administrator only) - protected by require_admin middleware
    let admin_routes = Router::new()
        .route("/api/admin/users", get(handlers::admin::list_users))
        .route("/api/admin/users", post(handlers::admin::create_user))
        .route("/api/admin/permissions", get(handlers::admin::list_permissions))
        .route("/api/admin/permissions", post(handlers::admin::upsert_permission))
        .route("/api/admin/permissions/:id", post(handlers::admin::delete_permission))
        .route("/api/admin/users/:id/role", put(handlers::admin::update_user_role))
        .route("/api/admin/users/:id/active", put(handlers::admin::update_user_active))
        .route("/api/admin/recovery-codes", post(handlers::admin::issue_recovery_code))
        .route("/api/backup", post(handlers::backup::create_backup))
        .route("/api/backup/restore", post(handlers::backup::restore_backup))
        .route_layer(middleware::from_fn_with_state(state.clone(), require_admin));

    Router::new()
        .merge(public_routes)
        .merge(auth_routes)
        .merge(staff_routes)
        .merge(ops_routes)
        .merge(admin_routes)
        .layer(middleware::from_fn(security_headers))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
