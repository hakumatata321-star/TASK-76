use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::Json;
use chrono::Datelike;
use crate::app::state::AppState;
use crate::errors::AppError;
use crate::handlers::auth::*;
use crate::models::*;

pub async fn get_calendar(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<CalendarQuery>,
) -> Result<Json<CalendarResponse>, AppError> {
    let claims = extract_claims_required(&headers, &state.hmac_secret)?;

    let read_action = match UserRole::from_str(&claims.role) {
        Some(UserRole::Customer) => "read",
        Some(UserRole::MerchantStaff) => "read_store",
        Some(UserRole::PlatformOps) | Some(UserRole::Administrator) => "read_all",
        Some(UserRole::Photographer) => "read_assigned",
        None => return Err(AppError::Forbidden("Invalid role".to_string())),
    };
    require_permission_with_state(&state, &claims, "calendar", read_action)?;

    // Store isolation for non-elevated roles
    enforce_store_isolation(&claims, &params.store_id)?;

    let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    let store = crate::repositories::stores::find_by_id(&db, &params.store_id)
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Store not found".to_string()))?;

    let business_hours = BusinessHours {
        start: store.business_hours_start.clone(),
        end: store.business_hours_end.clone(),
    };

    // Parse asset_status filter from comma-separated query param
    let status_filter: Option<Vec<String>> = params.asset_status.as_ref().map(|s| {
        s.split(',').map(|v| v.trim().to_string()).filter(|v| !v.is_empty()).collect()
    });

    // Determine date range based on view (day vs week)
    let base_date = chrono::NaiveDate::parse_from_str(&params.date, "%Y-%m-%d")
        .map_err(|_| AppError::Validation("Invalid date format, expected YYYY-MM-DD".to_string()))?;

    let (range_start, range_end) = if params.view == "week" {
        // ISO week: Monday through Sunday
        let weekday = base_date.weekday().num_days_from_monday();
        let monday = base_date - chrono::Duration::days(weekday as i64);
        let sunday = monday + chrono::Duration::days(6);
        (monday, sunday)
    } else {
        (base_date, base_date)
    };

    // Build slots for each day in the range
    let mut all_slots = Vec::new();
    let mut current_day = range_start;
    while current_day <= range_end {
        let day_str = current_day.format("%Y-%m-%d").to_string();
        let day_slots = build_calendar_slots(&db, &day_str, &params.store_id, &store)?;
        all_slots.extend(day_slots);
        current_day += chrono::Duration::days(1);
    }

    // Get assets for the store, filtered by status
    let vehicles = crate::repositories::vehicles::find_by_store(&db, &params.store_id)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let bays = crate::repositories::bays::find_by_store(&db, &params.store_id)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut assets: Vec<CalendarAsset> = vehicles.iter()
        .filter(|v| {
            status_filter.as_ref().map_or(true, |f| f.contains(&v.status))
        })
        .map(|v| CalendarAsset {
            id: v.id.clone(),
            asset_type: "vehicle".into(),
            name: format!("{} {} {}", v.make, v.model, v.trim_level).trim().to_string(),
            status: v.status.clone(),
        })
        .collect();
    assets.extend(
        bays.iter()
            .filter(|b| {
                status_filter.as_ref().map_or(true, |f| f.contains(&b.status) || f.contains(&"active".to_string()))
            })
            .map(|b| CalendarAsset {
                id: b.id.clone(),
                asset_type: "bay".into(),
                name: b.name.clone(),
                status: b.status.clone(),
            }),
    );

    Ok(Json(CalendarResponse {
        store_id: params.store_id,
        business_hours,
        date: params.date,
        view: params.view,
        slots: all_slots,
        assets,
    }))
}

fn build_calendar_slots(
    db: &rusqlite::Connection,
    date: &str,
    store_id: &str,
    store: &Store,
) -> Result<Vec<CalendarSlot>, AppError> {
    let day_start = format!("{}T{}:00", date, store.business_hours_start);
    let day_end = format!("{}T{}:00", date, store.business_hours_end);

    let mut stmt = db.prepare(
        "SELECT r.id, r.asset_type, r.asset_id, r.status, r.start_time, r.end_time, u.display_name FROM reservations r LEFT JOIN users u ON r.user_id = u.id WHERE r.store_id = ?1 AND r.start_time < ?3 AND r.end_time > ?2 AND r.status = 'confirmed' ORDER BY r.start_time"
    ).map_err(|e| AppError::Internal(e.to_string()))?;

    let reservations: Vec<(String, String, String, String, String, String, String)> = stmt.query_map(
        rusqlite::params![store_id, day_start, day_end],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get::<_, String>(6).unwrap_or_default())),
    ).map_err(|e| AppError::Internal(e.to_string()))?.filter_map(|r| r.ok()).collect();

    let start = chrono::NaiveDateTime::parse_from_str(&day_start, "%Y-%m-%dT%H:%M:%S")
        .map_err(|_| AppError::Internal("Invalid time".into()))?;
    let end = chrono::NaiveDateTime::parse_from_str(&day_end, "%Y-%m-%dT%H:%M:%S")
        .map_err(|_| AppError::Internal("Invalid time".into()))?;

    let mut slots = Vec::new();
    let mut cursor = start;
    while cursor < end {
        let slot_time = cursor.format("%Y-%m-%dT%H:%M:%S").to_string();
        let slot_end = (cursor + chrono::Duration::minutes(15)).format("%Y-%m-%dT%H:%M:%S").to_string();

        let slot_reservations: Vec<CalendarReservation> = reservations.iter()
            .filter(|(_, _, _, _, rs, re, _)| *rs < slot_end && *re > slot_time)
            .map(|(id, at, aid, status, _, _, uname)| CalendarReservation {
                id: id.clone(), asset_type: at.clone(), asset_id: aid.clone(),
                asset_name: aid.clone(), user_display_name: uname.clone(), status: status.clone(),
            }).collect();

        slots.push(CalendarSlot {
            time: slot_time, duration_minutes: 15, reservations: slot_reservations,
        });
        cursor += chrono::Duration::minutes(15);
    }

    Ok(slots)
}
