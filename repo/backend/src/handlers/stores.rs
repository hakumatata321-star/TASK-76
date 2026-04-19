use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;

use crate::app::state::AppState;
use crate::errors::AppError;
use crate::handlers::auth::*;
use crate::models::UserRole;

pub async fn list_stores(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, AppError> {
    let claims = extract_claims_required(&headers, &state.hmac_secret)?;
    require_role(&claims, &UserRole::MerchantStaff)?;
    let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    let stores = crate::repositories::stores::find_all(&db).map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(serde_json::json!({ "stores": stores })))
}
