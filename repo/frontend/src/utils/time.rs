/// Parse an "HH:MM" business-hours string to the integer hour component.
pub fn parse_hour(s: &str) -> u32 {
    s.split(':').next().and_then(|h| h.parse().ok()).unwrap_or(9)
}

pub fn generate_time_slots(start_hour: u32, end_hour: u32, increment_min: u32) -> Vec<String> {
    let mut slots = Vec::new();
    let mut minutes = start_hour * 60;
    let end_minutes = end_hour * 60;
    while minutes < end_minutes {
        let h = minutes / 60;
        let m = minutes % 60;
        slots.push(format!("{:02}:{:02}", h, m));
        minutes += increment_min;
    }
    slots
}

pub fn round_to_15_min(minutes: u32) -> u32 {
    ((minutes + 7) / 15) * 15
}

pub fn is_within_business_hours(time: &str, start: &str, end: &str) -> bool {
    time >= start && time <= end
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hour_extracts_hour_from_hh_mm() {
        assert_eq!(parse_hour("07:00"), 7);
        assert_eq!(parse_hour("19:00"), 19);
        assert_eq!(parse_hour("08:30"), 8);
        assert_eq!(parse_hour("00:00"), 0);
    }

    #[test]
    fn parse_hour_falls_back_on_garbage() {
        assert_eq!(parse_hour("not-a-time"), 9);
        assert_eq!(parse_hour(""), 9);
    }

    #[test]
    fn generate_time_slots_respects_bounds_and_increment() {
        let slots = generate_time_slots(9, 11, 30);
        assert_eq!(slots, vec!["09:00", "09:30", "10:00", "10:30"]);
    }

    #[test]
    fn round_to_15_min_nearest_boundary() {
        assert_eq!(super::round_to_15_min(0), 0);
        assert_eq!(super::round_to_15_min(7), 0);
        assert_eq!(super::round_to_15_min(8), 15);
        assert_eq!(super::round_to_15_min(22), 15);
        assert_eq!(super::round_to_15_min(23), 30);
        assert_eq!(super::round_to_15_min(38), 45);
    }

    #[test]
    fn is_within_business_hours_lexicographic_for_hh_mm() {
        assert!(is_within_business_hours("09:30", "09:00", "17:00"));
        assert!(!is_within_business_hours("08:59", "09:00", "17:00"));
        assert!(is_within_business_hours("17:00", "09:00", "17:00"));
    }
}
