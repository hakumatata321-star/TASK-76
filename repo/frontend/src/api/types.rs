use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub csrf_token: String,
    pub user: MaskedUser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskedUser {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub role: String,
    pub store_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskedVehicle {
    pub id: String,
    pub vin: String,
    pub license_plate: String,
    pub make: String,
    pub model: String,
    pub trim_level: String,
    pub store_id: String,
    pub mileage_miles: i64,
    pub fuel_or_battery_pct: f64,
    pub status: String,
    pub maintenance_due: Option<String>,
    pub inspection_due: Option<String>,
    pub insurance_expiry: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reservation {
    pub id: String,
    pub asset_type: String,
    pub asset_id: String,
    pub store_id: String,
    pub user_id: String,
    pub start_time: String,
    pub end_time: String,
    pub status: String,
    pub ticket_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReservationRequest {
    pub asset_type: String,
    pub asset_id: String,
    pub store_id: String,
    pub start_time: String,
    pub end_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResponse {
    pub conflict: bool,
    pub reasons: Vec<ConflictReasonDisplay>,
    pub alternative_slots: Vec<AlternativeSlot>,
    pub alternate_assets: Vec<AlternateAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictReasonDisplay {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeSlot {
    pub start_time: String,
    pub end_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternateAsset {
    pub id: String,
    pub asset_type: String,
    pub name: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticket {
    pub id: String,
    pub ticket_number: String,
    pub reservation_id: String,
    pub qr_data: String,
    pub valid_from: String,
    pub valid_until: String,
    pub redeemed: bool,
    pub redeemed_at: Option<String>,
    pub undone: bool,
    pub undo_eligible_until: Option<String>,
    pub undo_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceBay {
    pub id: String,
    pub store_id: String,
    pub name: String,
    pub bay_type: String,
    pub capacity: i64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarResponse {
    pub store_id: String,
    pub business_hours: BusinessHours,
    pub date: String,
    pub view: String,
    pub slots: Vec<CalendarSlot>,
    pub assets: Vec<CalendarAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessHours {
    pub start: String,
    pub end: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarSlot {
    pub time: String,
    pub duration_minutes: i64,
    pub reservations: Vec<CalendarReservation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarReservation {
    pub id: String,
    pub asset_type: String,
    pub asset_id: String,
    pub asset_name: String,
    pub user_display_name: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarAsset {
    pub id: String,
    pub asset_type: String,
    pub name: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Store {
    pub id: String,
    pub name: String,
    pub location: String,
    pub business_hours_start: String,
    pub business_hours_end: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhotographerAssignment {
    pub id: String,
    pub photographer_user_id: String,
    pub store_id: String,
    pub job_description: String,
    pub vehicle_id: Option<String>,
    pub bay_id: Option<String>,
    pub start_time: String,
    pub end_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: i64,
    pub timestamp: String,
    pub actor_username: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub error: ErrorDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_response_roundtrip() {
        let v = LoginResponse {
            token: "t".into(),
            csrf_token: "c".into(),
            user: MaskedUser {
                id: "u1".into(),
                username: "a".into(),
                display_name: "Admin".into(),
                role: "Administrator".into(),
                store_id: None,
            },
        };
        let json = serde_json::to_string(&v).unwrap();
        let back: LoginResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.user.role, "Administrator");
    }

    #[test]
    fn conflict_response_roundtrip() {
        let v = ConflictResponse {
            conflict: true,
            reasons: vec![ConflictReasonDisplay {
                code: "overlap".into(),
                message: "Busy".into(),
            }],
            alternative_slots: vec![AlternativeSlot {
                start_time: "2025-01-01T10:00:00Z".into(),
                end_time: "2025-01-01T11:00:00Z".into(),
            }],
            alternate_assets: vec![],
        };
        let json = serde_json::to_string(&v).unwrap();
        let back: ConflictResponse = serde_json::from_str(&json).unwrap();
        assert!(back.conflict);
        assert_eq!(back.reasons[0].code, "overlap");
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Upload {
    pub id: String,
    pub filename: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub sha256_fingerprint: String,
}
