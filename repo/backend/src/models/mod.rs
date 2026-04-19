use serde::{Deserialize, Serialize};

// --- Roles ---

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserRole {
    Customer,
    Photographer,
    MerchantStaff,
    PlatformOps,
    Administrator,
}

impl UserRole {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Customer" => Some(Self::Customer),
            "Photographer" => Some(Self::Photographer),
            "MerchantStaff" => Some(Self::MerchantStaff),
            "PlatformOps" => Some(Self::PlatformOps),
            "Administrator" => Some(Self::Administrator),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Customer => "Customer",
            Self::Photographer => "Photographer",
            Self::MerchantStaff => "MerchantStaff",
            Self::PlatformOps => "PlatformOps",
            Self::Administrator => "Administrator",
        }
    }

    pub fn has_at_least(&self, required: &UserRole) -> bool {
        self.level() >= required.level()
    }

    fn level(&self) -> u8 {
        match self {
            Self::Customer => 1,
            Self::Photographer => 2,
            Self::MerchantStaff => 3,
            Self::PlatformOps => 4,
            Self::Administrator => 5,
        }
    }
}

// --- User ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub display_name: String,
    pub email_encrypted: Option<String>,
    pub role: UserRole,
    pub store_id: Option<String>,
    pub active: bool,
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
pub struct MeResponse {
    pub user: MaskedUser,
    pub refreshed_token: String,
}

// --- Vehicle ---

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VehicleStatus {
    Available,
    Reserved,
    OnRent,
    InRepair,
    Decommissioned,
}

impl VehicleStatus {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "available" => Some(Self::Available),
            "reserved" => Some(Self::Reserved),
            "on-rent" => Some(Self::OnRent),
            "in-repair" => Some(Self::InRepair),
            "decommissioned" => Some(Self::Decommissioned),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Reserved => "reserved",
            Self::OnRent => "on-rent",
            Self::InRepair => "in-repair",
            Self::Decommissioned => "decommissioned",
        }
    }

    pub fn can_transition_to(&self, target: &VehicleStatus) -> bool {
        matches!(
            (self, target),
            (Self::Available, Self::Reserved)
                | (Self::Available, Self::OnRent)
                | (Self::Available, Self::InRepair)
                | (Self::Available, Self::Decommissioned)
                | (Self::Reserved, Self::Available)
                | (Self::Reserved, Self::OnRent)
                | (Self::Reserved, Self::Decommissioned)
                | (Self::OnRent, Self::Available)
                | (Self::OnRent, Self::Decommissioned)
                | (Self::InRepair, Self::Available)
                | (Self::InRepair, Self::Decommissioned)
        )
    }

    pub fn requires_admin(&self, target: &VehicleStatus) -> bool {
        matches!(target, VehicleStatus::Decommissioned)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vehicle {
    pub id: String,
    pub vin_encrypted: String,
    pub vin_hash: String,
    pub license_plate_encrypted: String,
    pub license_plate_hash: String,
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
    pub version: i64,
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

// --- Service Bay ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceBay {
    pub id: String,
    pub store_id: String,
    pub name: String,
    pub bay_type: String,
    pub capacity: i64,
    pub status: String,
    pub version: i64,
}

// --- Reservation ---

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
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReservationRequest {
    pub asset_type: String,
    pub asset_id: String,
    pub store_id: String,
    pub start_time: String,
    pub end_time: String,
}

// --- Conflict ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictReason {
    OverlappingReservation { existing_start: String, existing_end: String },
    InRepairHold,
    ExpiredInsurance { expiry_date: String },
    CapacityExceeded { current: i64, max: i64 },
}

impl ConflictReason {
    pub fn to_message(&self) -> String {
        match self {
            Self::OverlappingReservation { existing_start, existing_end } => {
                format!("This asset has an overlapping reservation from {} to {}.", existing_start, existing_end)
            }
            Self::InRepairHold => {
                "This vehicle is currently in repair and cannot be reserved.".to_string()
            }
            Self::ExpiredInsurance { expiry_date } => {
                format!("This vehicle's insurance expired on {}.", expiry_date)
            }
            Self::CapacityExceeded { current, max } => {
                format!("This service bay is at capacity ({}/{}) during the requested time.", current, max)
            }
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::OverlappingReservation { .. } => "overlapping_reservation",
            Self::InRepairHold => "in_repair_hold",
            Self::ExpiredInsurance { .. } => "expired_insurance",
            Self::CapacityExceeded { .. } => "capacity_exceeded",
        }
    }
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

// --- Ticket ---

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
    pub redeemed_by: Option<String>,
    pub undo_eligible_until: Option<String>,
    pub undone: bool,
    pub undone_at: Option<String>,
    pub undone_by: Option<String>,
    pub undo_reason: Option<String>,
}

// --- Recovery Code ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryCode {
    pub id: String,
    pub user_id: String,
    pub code_hash: String,
    pub issued_by: String,
    pub issued_at: String,
    pub expires_at: String,
    pub used: bool,
}

// --- Upload ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Upload {
    pub id: String,
    pub filename: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub sha256_fingerprint: String,
    pub vehicle_id: Option<String>,
    pub store_id: Option<String>,
    pub uploader_id: String,
}

// --- Audit ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: i64,
    pub timestamp: String,
    pub actor_id: String,
    pub actor_username: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub details_json: String,
    pub previous_hash: String,
    pub current_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditHashAnchor {
    pub id: i64,
    pub anchor_time: String,
    pub last_log_id: i64,
    pub cumulative_hash: String,
}

// --- Backup ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backup {
    pub id: String,
    pub filename: String,
    pub path: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub created_by: String,
}

// --- Store ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Store {
    pub id: String,
    pub name: String,
    pub location: String,
    pub business_hours_start: String,
    pub business_hours_end: String,
    pub active: bool,
}

// --- Assignment ---

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

// --- Auth types ---

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
pub struct Claims {
    pub user_id: String,
    pub username: String,
    pub role: String,
    pub store_id: Option<String>,
    pub iat: i64,
    pub exp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetPasswordRequest {
    pub username: String,
    pub recovery_code: String,
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueRecoveryCodeRequest {
    pub user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusTransitionRequest {
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoRedemptionRequest {
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub display_name: String,
    pub role: String,
    pub store_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRequest {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarQuery {
    pub store_id: String,
    pub date: String,
    pub view: String,
    pub asset_status: Option<String>,
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
pub struct CreateBayRequest {
    pub name: String,
    pub store_id: String,
    pub bay_type: String,
    pub capacity: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVehicleRequest {
    pub vin: String,
    pub license_plate: String,
    pub make: String,
    pub model: String,
    pub trim_level: Option<String>,
    pub store_id: String,
    pub mileage_miles: Option<i64>,
    pub fuel_or_battery_pct: Option<f64>,
    pub maintenance_due: Option<String>,
    pub inspection_due: Option<String>,
    pub insurance_expiry: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAssignmentRequest {
    pub photographer_user_id: String,
    pub store_id: String,
    pub job_description: String,
    pub vehicle_id: Option<String>,
    pub bay_id: Option<String>,
    pub start_time: String,
    pub end_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportQuery {
    pub store_id: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub export_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRoleRequest {
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateActiveRequest {
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRule {
    pub id: String,
    pub role: String,
    pub resource: String,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertPermissionRequest {
    pub role: String,
    pub resource: String,
    pub action: String,
}
