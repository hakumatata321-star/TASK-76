/// Extract sorted unique ISO-date strings (YYYY-MM-DD) from a list of slot time strings,
/// preserving insertion order. Each string is expected to start with "YYYY-MM-DD".
pub fn week_dates_from_slots(slot_times: &[String]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut dates = Vec::new();
    for t in slot_times {
        if let Some(date) = t.get(0..10) {
            if seen.insert(date.to_string()) {
                dates.push(date.to_string());
            }
        }
    }
    dates
}

/// Format an ISO date string "YYYY-MM-DD" as a short day-column header, e.g. "Mon 1/6".
pub fn format_day_header(date: &str) -> String {
    use chrono::{Datelike, NaiveDate};
    if let Ok(d) = NaiveDate::parse_from_str(date, "%Y-%m-%d") {
        let names = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
        let idx = d.weekday().num_days_from_monday() as usize;
        format!("{} {}/{}", names[idx], d.month(), d.day())
    } else {
        date.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn week_dates_deduplicates_and_preserves_order() {
        let times = vec![
            "2026-01-05T09:00:00".to_string(),
            "2026-01-05T09:15:00".to_string(),
            "2026-01-06T09:00:00".to_string(),
        ];
        let dates = week_dates_from_slots(&times);
        assert_eq!(dates, vec!["2026-01-05", "2026-01-06"]);
    }

    #[test]
    fn week_dates_empty_input() {
        assert!(week_dates_from_slots(&[]).is_empty());
    }

    #[test]
    fn format_day_header_monday() {
        let label = format_day_header("2026-01-05");
        assert!(label.starts_with("Mon"), "got: {}", label);
    }

    #[test]
    fn format_day_header_sunday() {
        let label = format_day_header("2026-01-11");
        assert!(label.starts_with("Sun"), "got: {}", label);
    }

    #[test]
    fn format_day_header_invalid_date_returns_as_is() {
        assert_eq!(format_day_header("not-a-date"), "not-a-date");
    }
}
