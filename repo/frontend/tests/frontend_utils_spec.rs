#[path = "../src/utils/format.rs"]
mod format;
#[path = "../src/utils/time.rs"]
mod time;

#[test]
fn frontend_utils_parse_hour_from_business_hours_string() {
    assert_eq!(time::parse_hour("07:00"), 7, "standard opening time parses correctly");
    assert_eq!(time::parse_hour("19:00"), 19, "standard closing time parses correctly");
    assert_eq!(time::parse_hour("08:30"), 8, "non-zero minutes are ignored (only hour matters)");
    assert_eq!(time::parse_hour("00:00"), 0, "midnight parses to 0");
}

#[test]
fn frontend_utils_slots_derived_from_nondefault_business_hours() {
    // If business hours are 10:00–12:00 (non-default), generated slots must
    // cover exactly that range—not the old hardcoded 7–19.
    let slots = time::generate_time_slots(
        time::parse_hour("10:00"),
        time::parse_hour("12:00"),
        15,
    );
    assert!(slots.first().unwrap().starts_with("10:"), "first slot must start at business open hour");
    assert!(slots.last().unwrap().starts_with("11:"), "last slot must be within business hours");
    // 2 hours × 4 slots/hour = 8 slots
    assert_eq!(slots.len(), 8, "slot count must match (end - start) × 4");
}

#[test]
fn frontend_utils_time_slot_generation_is_deterministic() {
    let slots = time::generate_time_slots(8, 10, 15);
    assert_eq!(
        slots,
        vec!["08:00", "08:15", "08:30", "08:45", "09:00", "09:15", "09:30", "09:45"]
    );
}

#[test]
fn frontend_utils_datetime_and_mileage_formatting_are_user_friendly() {
    assert_eq!(format::format_datetime("2026-01-01T13:05:00"), "2026-01-01 1:05 PM");
    assert_eq!(format::format_mileage(123456), "123,456 mi");
}
