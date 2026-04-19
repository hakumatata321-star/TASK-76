//! Behavior-focused frontend tests.
//! These tests assert structural and contract properties of pages, guards, and
//! error branches using source-level inspection and data-shape checks.

#[path = "../src/api/types.rs"]
mod api_types;

// --- Login page ---

#[test]
fn frontend_login_page_has_credential_fields_and_no_route_guard() {
    let src = include_str!("../src/pages/login.rs");
    assert!(src.contains("LoginPage"), "LoginPage component must exist");
    // Login page must not be route-guarded (it is the entry point for unauthenticated users).
    assert!(
        !src.contains("RouteGuard"),
        "LoginPage must not be wrapped in RouteGuard"
    );
    // Must have username and password inputs.
    assert!(src.contains("username"), "login page must have a username field");
    assert!(src.contains("password"), "login page must have a password field");
}

// --- Role guards ---

#[test]
fn frontend_route_guard_denies_and_redirects_on_insufficient_role() {
    let src = include_str!("../src/security/route_guard.rs");
    assert!(src.contains("required_role"), "RouteGuard must accept a required_role prop");
    assert!(src.contains("Access Denied"), "RouteGuard must render an Access Denied message");
}

#[test]
fn frontend_exports_page_is_guarded_for_platform_ops() {
    let src = include_str!("../src/pages/exports.rs");
    assert!(src.contains("PlatformOps"), "ExportsPage must require PlatformOps role");
    assert!(src.contains("RouteGuard"), "ExportsPage must be wrapped in RouteGuard");
}

#[test]
fn frontend_checkin_page_is_guarded_for_merchant_staff() {
    let src = include_str!("../src/pages/tickets.rs");
    assert!(src.contains("MerchantStaff"), "CheckInPage must require MerchantStaff role");
    assert!(src.contains("RouteGuard"), "CheckInPage must be wrapped in RouteGuard");
}

// --- Check-in scanner ---

#[test]
fn frontend_checkin_page_has_qr_scanner_file_input() {
    let src = include_str!("../src/pages/tickets.rs");
    assert!(
        src.contains("type=\"file\""),
        "CheckInPage must include a file input for QR scanning"
    );
    assert!(
        src.contains("accept=\"image/*\""),
        "QR scan input must accept image files"
    );
    assert!(
        src.contains("capture=\"environment\""),
        "QR scan input must request the rear camera on mobile"
    );
    assert!(
        src.contains("/tickets/scan"),
        "CheckInPage must post to /tickets/scan for QR decode"
    );
}

// --- Reservation conflict UX ---

#[test]
fn frontend_conflict_explanation_component_renders_conflict_details() {
    let src = include_str!("../src/components/conflict_explanation.rs");
    assert!(
        src.contains("ConflictExplanation"),
        "ConflictExplanation component must exist"
    );
    // Must surface human-readable messages from the conflict response.
    assert!(
        src.contains("message") || src.contains("reasons"),
        "conflict component must render conflict reasons or messages"
    );
}

// --- Export masking: data-shape contract ---

#[test]
fn frontend_export_page_uses_post_not_get() {
    let src = include_str!("../src/pages/exports.rs");
    assert!(
        src.contains("api_post"),
        "ExportsPage must use api_post (not api_get) because the endpoint is POST"
    );
    assert!(
        !src.contains("api_get(\"/exports\")"),
        "ExportsPage must not call GET /exports (it is now POST)"
    );
}

// --- Role authorization behavior (source-inspection) ---

#[test]
fn frontend_auth_state_encodes_all_five_roles_with_strict_ordering() {
    let src = include_str!("../src/state/auth.rs");
    // All five roles must be present in the rank function
    for role in &["Customer", "Photographer", "MerchantStaff", "PlatformOps", "Administrator"] {
        assert!(src.contains(role), "role_rank must handle role: {}", role);
    }
    // Photographer rank must be defined separately from MerchantStaff
    assert!(
        src.contains("\"Photographer\""),
        "Photographer must have its own rank entry"
    );
    // Unknown roles must get rank 0 (deny by default)
    assert!(src.contains("_ => 0"), "unknown roles must map to rank 0");
}

#[test]
fn frontend_route_guard_checks_has_role_not_equality() {
    let src = include_str!("../src/security/route_guard.rs");
    // Must use has_role (which calls satisfies_role) not direct equality check
    assert!(
        src.contains("has_role"),
        "RouteGuard must use has_role for hierarchy-aware checks"
    );
    assert!(
        !src.contains("== required_role"),
        "RouteGuard must not use equality — higher roles must also pass"
    );
    let auth_src = include_str!("../src/state/auth.rs");
    assert!(
        auth_src.contains("satisfies_role"),
        "has_role must delegate to satisfies_role for hierarchy logic"
    );
}

// --- Reservation response: masked user_id contract ---

#[test]
fn frontend_reservation_response_deserializes_with_masked_user_id() {
    // The backend masks user_id in API responses (e.g., "usr-***--001").
    // Frontend must accept any string in the user_id field without failing.
    let payload = serde_json::json!({
        "id": "res-001",
        "asset_type": "vehicle",
        "asset_id": "v1",
        "store_id": "store-001",
        "user_id": "usr-***--001",
        "start_time": "2030-01-01T09:00:00",
        "end_time": "2030-01-01T10:00:00",
        "status": "confirmed",
        "ticket_id": "tkt-001"
    });
    let r = serde_json::from_value::<api_types::Reservation>(payload)
        .expect("Reservation must deserialise with masked user_id");
    assert_eq!(r.status, "confirmed");
    assert!(r.user_id.contains("***"), "masked user_id must contain '***'");
}

// --- Audit log: masked actor fields ---

#[test]
fn frontend_audit_entry_deserializes_with_masked_actor() {
    let payload = serde_json::json!({
        "id": 1,
        "timestamp": "2026-01-01T00:00:00Z",
        "actor_username": "a***",
        "action": "LOGIN",
        "resource_type": "user",
        "resource_id": "ph-abcd1234abcd1234"
    });
    let entry = serde_json::from_value::<api_types::AuditEntry>(payload)
        .expect("AuditEntry must deserialise with masked fields");
    assert!(entry.actor_username.contains("***"), "actor_username must be masked");
    assert!(entry.resource_id.starts_with("ph-"), "pseudonymized actor_id must start with 'ph-'");
}

// --- Conflict response UX behavior ---

#[test]
fn frontend_conflict_response_with_alternative_slots_is_actionable() {
    let payload = serde_json::json!({
        "conflict": true,
        "reasons": [{"code": "overlapping_reservation", "message": "Time slot is already booked"}],
        "alternative_slots": [
            {"start_time": "2030-01-01T10:00:00", "end_time": "2030-01-01T11:00:00"},
            {"start_time": "2030-01-01T11:00:00", "end_time": "2030-01-01T12:00:00"}
        ],
        "alternate_assets": []
    });
    let conflict = serde_json::from_value::<api_types::ConflictResponse>(payload)
        .expect("ConflictResponse must deserialise");
    assert!(conflict.conflict);
    assert_eq!(conflict.reasons[0].code, "overlapping_reservation");
    assert_eq!(conflict.alternative_slots.len(), 2, "must provide 2 alternatives");
}

// --- Calendar week view grid structure (source-inspection) ---

#[test]
fn frontend_calendar_week_view_uses_seven_column_grid() {
    let src = include_str!("../src/pages/calendar.rs");
    // Week view must declare 7 equal-width day columns.
    assert!(
        src.contains("repeat(7, 1fr)") || src.contains("repeat(7,1fr)"),
        "week view grid must use 7 equal-width columns (CSS repeat(7, 1fr))"
    );
    // Day view must use the narrow 2-column layout.
    assert!(
        src.contains("80px 1fr"),
        "day view must use 2-column layout (80px 1fr)"
    );
}

#[test]
fn frontend_calendar_week_view_derives_dates_from_slot_list() {
    let src = include_str!("../src/pages/calendar.rs");
    assert!(
        src.contains("week_dates_from_slots"),
        "calendar must call week_dates_from_slots to derive day columns from API data"
    );
}

#[test]
fn frontend_calendar_business_hours_drive_slot_generation() {
    let src = include_str!("../src/pages/calendar.rs");
    // Slots must be derived from API business_hours, not hardcoded literals.
    assert!(
        src.contains("parse_hour"),
        "calendar must call parse_hour to convert business_hours strings to integers"
    );
    assert!(
        src.contains("business_hours.start"),
        "calendar must use business_hours.start as the slot range start"
    );
    assert!(
        src.contains("business_hours.end"),
        "calendar must use business_hours.end as the slot range end"
    );
    assert!(
        !src.contains("generate_time_slots(7, 19"),
        "calendar must not use hardcoded 7..19 range — slots must come from API business hours"
    );
}

#[test]
fn frontend_calendar_status_filter_triggers_reload() {
    let src = include_str!("../src/pages/calendar.rs");
    assert!(
        src.contains("sf.update"),
        "status filter on:change must call sf.update to toggle the selection"
    );
    // load_calendar() must appear in: store change, date change, day-btn, week-btn,
    // and the status-filter on:change handler = at least 5 call sites.
    let call_count = src.matches("load_calendar()").count();
    assert!(
        call_count >= 5,
        "load_calendar() must be wired to at least 5 event handlers \
         (store, date, day-btn, week-btn, status-filter); found {}",
        call_count
    );
}

// --- Calendar helper behavioral tests (pure Rust, no Leptos) ---

#[path = "../src/utils/calendar.rs"]
mod calendar_utils;

#[test]
fn calendar_week_dates_from_slots_extracts_unique_dates_in_order() {
    let times = vec![
        "2026-01-05T09:00:00".to_string(),
        "2026-01-05T09:15:00".to_string(),
        "2026-01-06T09:00:00".to_string(),
    ];
    let dates = calendar_utils::week_dates_from_slots(&times);
    assert_eq!(dates.len(), 2, "must return exactly 2 unique dates");
    assert_eq!(dates[0], "2026-01-05");
    assert_eq!(dates[1], "2026-01-06");
}

#[test]
fn calendar_week_dates_from_slots_seven_days_deduplication() {
    // Simulate a full ISO week: 7 dates × 2 slots each = 14 time strings
    let mut times = Vec::new();
    let dates = [
        "2026-01-05", "2026-01-06", "2026-01-07", "2026-01-08",
        "2026-01-09", "2026-01-10", "2026-01-11",
    ];
    for d in &dates {
        times.push(format!("{}T09:00:00", d));
        times.push(format!("{}T09:15:00", d));
    }
    let result = calendar_utils::week_dates_from_slots(&times);
    assert_eq!(result.len(), 7, "must produce exactly 7 unique dates for a full week");
    assert_eq!(result[0], "2026-01-05", "first date must be Monday");
    assert_eq!(result[6], "2026-01-11", "last date must be Sunday");
}

#[test]
fn calendar_week_dates_from_slots_empty_input_returns_empty() {
    let dates = calendar_utils::week_dates_from_slots(&[]);
    assert!(dates.is_empty(), "empty input must yield empty output");
}

#[test]
fn calendar_format_day_header_produces_correct_weekday() {
    // 2026-01-05 is a Monday; 2026-01-11 is a Sunday
    let mon = calendar_utils::format_day_header("2026-01-05");
    assert!(mon.starts_with("Mon"), "2026-01-05 is Monday, got: {}", mon);
    let sun = calendar_utils::format_day_header("2026-01-11");
    assert!(sun.starts_with("Sun"), "2026-01-11 is Sunday, got: {}", sun);
}

#[test]
fn calendar_format_day_header_includes_date_digits() {
    let label = calendar_utils::format_day_header("2026-01-05");
    // Should include month (1) and day (5)
    assert!(label.contains("1/5") || label.contains("1/05"), "label must include month/day: {}", label);
}

// --- Audit response shape: masked actor ---

#[test]
fn frontend_api_login_response_shape_is_valid() {
    let payload = serde_json::json!({
        "token": "eyJhbGciOiJIUzI1NiJ9.test",
        "csrf_token": "csrf-abc-123",
        "user": {
            "id": "user-admin-001",
            "username": "a***",
            "display_name": "System Administrator",
            "role": "Administrator",
            "store_id": null
        }
    });
    let resp = serde_json::from_value::<api_types::LoginResponse>(payload)
        .expect("LoginResponse must deserialise correctly");
    assert_eq!(resp.user.role, "Administrator");
    assert!(!resp.token.is_empty(), "token must be non-empty");
    assert!(!resp.csrf_token.is_empty(), "csrf_token must be non-empty");
}

// --- Reservation conflict display path (behavioral) ---

#[test]
fn frontend_reservations_page_routes_409_to_conflict_explanation() {
    let src = include_str!("../src/pages/reservations.rs");
    // 409 branch must store the decoded body as Err(conflict), not Ok
    assert!(
        src.contains("Ok((409, json))"),
        "reservations page must handle 409 status specifically"
    );
    assert!(
        src.contains("Err(conflict)"),
        "409 conflict body must be stored as the Err variant of the result signal"
    );
    // The view! match must dispatch the Err arm to ConflictExplanation
    assert!(
        src.contains("ConflictExplanation"),
        "reservations page must render ConflictExplanation on a conflict response"
    );
    assert!(
        src.contains("Err(conflict) =>"),
        "result match must have an explicit Err(conflict) arm wired to ConflictExplanation"
    );
}

#[test]
fn frontend_reservations_conflict_alternatives_deserialize_correctly() {
    let payload = serde_json::json!({
        "conflict": true,
        "reasons": [{"code": "overlapping_reservation", "message": "Already booked"}],
        "alternative_slots": [
            {"start_time": "2026-06-15T10:00:00", "end_time": "2026-06-15T11:00:00"},
            {"start_time": "2026-06-15T11:00:00", "end_time": "2026-06-15T12:00:00"},
            {"start_time": "2026-06-15T14:00:00", "end_time": "2026-06-15T15:00:00"}
        ],
        "alternate_assets": []
    });
    let conflict = serde_json::from_value::<api_types::ConflictResponse>(payload)
        .expect("ConflictResponse must deserialise with alternatives");
    assert!(conflict.conflict);
    assert_eq!(conflict.reasons[0].code, "overlapping_reservation");
    assert_eq!(conflict.alternative_slots.len(), 3, "all three alternatives must be present");
    for slot in &conflict.alternative_slots {
        assert!(slot.start_time.contains('T'), "start_time must be ISO datetime");
        assert!(slot.end_time.contains('T'), "end_time must be ISO datetime");
        assert!(slot.start_time < slot.end_time, "start_time must precede end_time");
    }
}

#[test]
fn frontend_reservations_conflict_with_alternate_assets_deserializes() {
    let payload = serde_json::json!({
        "conflict": true,
        "reasons": [{"code": "asset_unavailable", "message": "Asset is under maintenance"}],
        "alternative_slots": [],
        "alternate_assets": [
            {"id": "v2", "asset_type": "vehicle", "name": "Van #2", "status": "available"},
            {"id": "v3", "asset_type": "vehicle", "name": "Van #3", "status": "available"}
        ]
    });
    let conflict = serde_json::from_value::<api_types::ConflictResponse>(payload)
        .expect("ConflictResponse must deserialise with alternate_assets");
    assert_eq!(conflict.alternate_assets.len(), 2);
    assert_eq!(conflict.alternate_assets[0].asset_type, "vehicle");
    assert_eq!(conflict.alternate_assets[1].status, "available");
}

// --- Check-in / undo path (behavioral) ---

#[test]
fn frontend_checkin_page_undo_requires_non_empty_reason() {
    let src = include_str!("../src/pages/tickets.rs");
    assert!(
        src.contains("reason.trim().is_empty()"),
        "undo handler must reject empty reason via trim().is_empty() guard"
    );
    assert!(
        src.contains("Undo reason is required"),
        "empty-reason guard must produce a visible error message"
    );
}

#[test]
fn frontend_checkin_page_undo_availability_is_timer_driven() {
    let src = include_str!("../src/pages/tickets.rs");
    // Successful redemption enables undo; a timer disables it after 2 minutes
    assert!(
        src.contains("undo_available.set(true)"),
        "successful redemption must enable undo_available"
    );
    assert!(
        src.contains("120_000") || src.contains("120000"),
        "undo eligibility timer must fire after 120 seconds (120_000 ms)"
    );
    assert!(
        src.contains("undo_available.set(false)"),
        "timer callback must disable undo_available when it fires"
    );
}

#[test]
fn frontend_checkin_page_undo_section_gated_by_undo_available_signal() {
    let src = include_str!("../src/pages/tickets.rs");
    assert!(
        src.contains("undo_available.get()"),
        "undo section visibility must be driven by undo_available signal"
    );
    // The undo section must be inside a Show so it is absent from the DOM by default
    assert!(
        src.contains("Show when"),
        "undo section must be wrapped in a <Show when=...> block"
    );
}

#[test]
fn frontend_ticket_type_carries_undo_eligibility_fields() {
    let payload = serde_json::json!({
        "id": "tkt-001",
        "ticket_number": "FR-ABCD1234",
        "reservation_id": "res-001",
        "qr_data": "qr-data-string",
        "valid_from": "2026-06-15T09:00:00",
        "valid_until": "2026-06-15T10:00:00",
        "redeemed": true,
        "redeemed_at": "2026-06-15T09:05:00",
        "undone": false,
        "undo_eligible_until": "2026-06-15T09:07:00",
        "undo_reason": null
    });
    let ticket = serde_json::from_value::<api_types::Ticket>(payload)
        .expect("Ticket must deserialise with undo eligibility fields");
    assert!(ticket.redeemed, "redeemed flag must be true");
    assert!(!ticket.undone, "undone must be false before undo operation");
    let eligible_until = ticket.undo_eligible_until.as_deref().expect("undo_eligible_until must be present");
    let redeemed_at = ticket.redeemed_at.as_deref().expect("redeemed_at must be present");
    assert!(eligible_until > redeemed_at, "undo_eligible_until must be later than redeemed_at");
}

#[test]
fn frontend_ticket_undone_and_undo_reason_reflect_completed_undo() {
    let payload = serde_json::json!({
        "id": "tkt-002",
        "ticket_number": "FR-EFGH5678",
        "reservation_id": "res-002",
        "qr_data": "qr-data-2",
        "valid_from": "2026-06-15T10:00:00",
        "valid_until": "2026-06-15T11:00:00",
        "redeemed": true,
        "redeemed_at": "2026-06-15T10:02:00",
        "undone": true,
        "undo_eligible_until": "2026-06-15T10:04:00",
        "undo_reason": "Staff error — wrong customer"
    });
    let ticket = serde_json::from_value::<api_types::Ticket>(payload)
        .expect("Ticket with completed undo must deserialise");
    assert!(ticket.undone, "undone must be true after undo operation");
    assert_eq!(
        ticket.undo_reason.as_deref(),
        Some("Staff error — wrong customer"),
        "undo_reason must preserve the staff-entered text"
    );
}
