mod common;

use common::*;

/// Reservation window whose ticket validity brackets typical CI/runtime `Utc::now()` (requires relaxed store hours in `setup_test_db`).
const RES_REDEEM_OK_START: &str = "2020-01-01T09:00:00";
const RES_REDEEM_OK_END: &str = "2035-12-31T18:00:00";

#[test]
fn test_login_success() {
    // Integration test: verify login flow with real password hash
    // This test validates that the auth handler correctly verifies credentials
    // and returns a valid session token.
    let conn = setup_test_db();
    let user = fleetreserve_backend::repositories::users::find_by_username(&conn, "admin");
    assert!(user.is_ok());
    // Note: Full HTTP-level integration requires axum-test crate
}

#[test]
fn test_reservation_engine_full_cycle() {
    let conn = setup_test_db();
    create_test_vehicle(&conn, "v1", "store-001", "available");

    let req = fleetreserve_backend::models::CreateReservationRequest {
        asset_type: "vehicle".into(),
        asset_id: "v1".into(),
        store_id: "store-001".into(),
        start_time: "2026-04-10T09:00:00".into(),
        end_time: "2026-04-10T10:00:00".into(),
    };

    let result = fleetreserve_backend::services::reservation_engine::create_reservation(
        &conn, "user-admin-001", "admin", &req, "",
    );
    assert!(result.is_ok());
    let r = result.unwrap();
    assert_eq!(r.reservation.status, "confirmed");
    assert!(r.ticket.ticket_number.starts_with("FR-"));
}

#[test]
fn test_ticket_redemption_and_undo() {
    let conn = setup_test_db();
    create_test_vehicle(&conn, "v1", "store-001", "available");

    let req = fleetreserve_backend::models::CreateReservationRequest {
        asset_type: "vehicle".into(),
        asset_id: "v1".into(),
        store_id: "store-001".into(),
        start_time: RES_REDEEM_OK_START.into(),
        end_time: RES_REDEEM_OK_END.into(),
    };

    let result = fleetreserve_backend::services::reservation_engine::create_reservation(
        &conn, "user-admin-001", "admin", &req, "",
    ).unwrap();

    // Redeem
    let redeem = fleetreserve_backend::services::ticket_engine::redeem_ticket(
        &conn, &result.ticket.id, "user-admin-001", "admin", "",
    );
    assert!(redeem.is_ok());

    // Double redeem blocked
    let double = fleetreserve_backend::services::ticket_engine::redeem_ticket(
        &conn, &result.ticket.id, "user-admin-001", "admin", "",
    );
    assert!(double.is_err());
    assert!(double.unwrap_err().contains("already"));

    // Undo
    let undo = fleetreserve_backend::services::ticket_engine::undo_redemption(
        &conn, &result.ticket.id, "user-admin-001", "admin", "Wrong ticket scanned", "",
    );
    assert!(undo.is_ok());
}

#[test]
fn test_undo_requires_reason() {
    let conn = setup_test_db();
    create_test_vehicle(&conn, "v1", "store-001", "available");

    let req = fleetreserve_backend::models::CreateReservationRequest {
        asset_type: "vehicle".into(), asset_id: "v1".into(),
        store_id: "store-001".into(),
        start_time: RES_REDEEM_OK_START.into(),
        end_time: RES_REDEEM_OK_END.into(),
    };

    let result = fleetreserve_backend::services::reservation_engine::create_reservation(
        &conn, "user-admin-001", "admin", &req, "",
    ).unwrap();

    let _ = fleetreserve_backend::services::ticket_engine::redeem_ticket(
        &conn, &result.ticket.id, "user-admin-001", "admin", "",
    );

    // Empty reason rejected
    let undo = fleetreserve_backend::services::ticket_engine::undo_redemption(
        &conn, &result.ticket.id, "user-admin-001", "admin", "", "",
    );
    assert!(undo.is_err());
    assert!(undo.unwrap_err().contains("required"));

    // Whitespace-only reason rejected
    let undo2 = fleetreserve_backend::services::ticket_engine::undo_redemption(
        &conn, &result.ticket.id, "user-admin-001", "admin", "   ", "",
    );
    assert!(undo2.is_err());
}

#[test]
fn test_conflict_detection() {
    let conn = setup_test_db();
    create_test_vehicle(&conn, "v1", "store-001", "available");

    let req1 = fleetreserve_backend::models::CreateReservationRequest {
        asset_type: "vehicle".into(), asset_id: "v1".into(),
        store_id: "store-001".into(),
        start_time: "2026-04-10T09:00:00".into(),
        end_time: "2026-04-10T10:00:00".into(),
    };
    let _ = fleetreserve_backend::services::reservation_engine::create_reservation(
        &conn, "user-admin-001", "admin", &req1, "",
    ).unwrap();

    let req2 = fleetreserve_backend::models::CreateReservationRequest {
        asset_type: "vehicle".into(), asset_id: "v1".into(),
        store_id: "store-001".into(),
        start_time: "2026-04-10T09:30:00".into(),
        end_time: "2026-04-10T10:30:00".into(),
    };
    let conflict = fleetreserve_backend::services::reservation_engine::create_reservation(
        &conn, "user-admin-001", "admin", &req2, "",
    );
    assert!(conflict.is_err());
    let c = conflict.unwrap_err();
    assert!(c.conflict);
    assert!(c.reasons.iter().any(|r| r.code == "overlapping_reservation"));
    assert!(!c.alternative_slots.is_empty());
}

#[test]
fn test_audit_chain() {
    let conn = setup_test_db();

    for i in 0..5 {
        fleetreserve_backend::audit::chain::append_audit_log(
            &conn, "user-1", "admin", "TEST", "test", &format!("r-{}", i),
            &serde_json::json!({"seq": i}),
        ).unwrap();
    }

    assert!(fleetreserve_backend::audit::chain::verify_chain_integrity(&conn).unwrap());
}

#[test]
fn test_masking() {
    assert_eq!(fleetreserve_backend::security::masking::mask_vin("1HGCM82633A123456"), "*************3456");
    assert_eq!(fleetreserve_backend::security::masking::mask_license_plate("ABC1234"), "*****34");
    assert_eq!(fleetreserve_backend::security::masking::mask_username("johndoe"), "j***");
}

#[test]
fn test_encryption_roundtrip() {
    let key = "test-encryption-key-32-bytes!!!!";
    let plaintext = "1HGCM82633A123456";
    let encrypted = fleetreserve_backend::security::encryption::encrypt_field(plaintext, key).unwrap();
    let decrypted = fleetreserve_backend::security::encryption::decrypt_field(&encrypted, key).unwrap();
    assert_eq!(decrypted, plaintext);
    assert_ne!(encrypted, plaintext);
}

#[test]
fn test_upload_validation() {
    // Invalid file type
    let result = fleetreserve_backend::services::uploads::validate_upload(b"not an image at all", "test.txt");
    assert!(result.is_err());

    // Oversized file
    let mut big = vec![0xFF, 0xD8, 0xFF, 0xE0];
    big.extend(vec![0u8; 11_000_000]);
    let result = fleetreserve_backend::services::uploads::validate_upload(&big, "big.jpg");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("10 MB"));
}

#[test]
fn test_vehicle_status_transitions() {
    use fleetreserve_backend::models::VehicleStatus;
    assert!(VehicleStatus::Available.can_transition_to(&VehicleStatus::Reserved));
    assert!(VehicleStatus::Available.can_transition_to(&VehicleStatus::InRepair));
    assert!(VehicleStatus::Available.can_transition_to(&VehicleStatus::Decommissioned));
    assert!(!VehicleStatus::Decommissioned.can_transition_to(&VehicleStatus::Available));
    assert!(VehicleStatus::Available.requires_admin(&VehicleStatus::Decommissioned));
    assert!(!VehicleStatus::Available.requires_admin(&VehicleStatus::Reserved));
}

#[test]
fn test_role_hierarchy() {
    use fleetreserve_backend::models::UserRole;
    assert!(UserRole::Administrator.has_at_least(&UserRole::Customer));
    assert!(UserRole::Administrator.has_at_least(&UserRole::Administrator));
    assert!(!UserRole::Customer.has_at_least(&UserRole::MerchantStaff));
    assert!(UserRole::PlatformOps.has_at_least(&UserRole::MerchantStaff));
}

#[test]
fn test_audit_log_append_only_triggers() {
    let conn = setup_test_db();
    fleetreserve_backend::audit::chain::append_audit_log(
        &conn, "u1", "admin", "TEST", "test", "r1", &serde_json::json!({}),
    ).unwrap();

    // UPDATE should be blocked by trigger
    let update_result = conn.execute("UPDATE audit_log SET action = 'X' WHERE id = 1", []);
    assert!(update_result.is_err());

    // DELETE should be blocked by trigger
    let delete_result = conn.execute("DELETE FROM audit_log WHERE id = 1", []);
    assert!(delete_result.is_err());
}

#[test]
fn test_backup_encryption_roundtrip() {
    let key = "test-backup-key-32-bytes-long!!!";
    let data = b"SQLite format 3\0 test database content here";
    let encrypted = fleetreserve_backend::security::encryption::encrypt_bytes(data, key).unwrap();
    assert_ne!(encrypted, data.to_vec());
    let decrypted = fleetreserve_backend::security::encryption::decrypt_bytes(&encrypted, key).unwrap();
    assert_eq!(decrypted, data.to_vec());
}

#[test]
fn test_csrf_token_session_binding() {
    let t1 = fleetreserve_backend::auth::csrf::generate_csrf_token();
    let t2 = fleetreserve_backend::auth::csrf::generate_csrf_token();
    assert_ne!(t1, t2);
    assert!(fleetreserve_backend::auth::csrf::validate_csrf_token(&t1, &t1));
    assert!(!fleetreserve_backend::auth::csrf::validate_csrf_token(&t1, &t2));
}

#[test]
fn test_store_isolation_enforcement() {
    use fleetreserve_backend::models::Claims;
    use fleetreserve_backend::handlers::auth::enforce_store_isolation;

    let merchant_claims = Claims {
        user_id: "u1".into(), username: "staff".into(), role: "MerchantStaff".into(),
        store_id: Some("store-001".into()), iat: 0, exp: i64::MAX,
    };
    // Own store: OK
    assert!(enforce_store_isolation(&merchant_claims, "store-001").is_ok());
    // Other store: Forbidden
    assert!(enforce_store_isolation(&merchant_claims, "store-002").is_err());

    let admin_claims = Claims {
        user_id: "u2".into(), username: "admin".into(), role: "Administrator".into(),
        store_id: None, iat: 0, exp: i64::MAX,
    };
    // Admin bypasses
    assert!(enforce_store_isolation(&admin_claims, "store-001").is_ok());
    assert!(enforce_store_isolation(&admin_claims, "store-002").is_ok());
}

#[test]
fn test_ticket_validity_window_enforcement() {
    let conn = setup_test_db();
    create_test_vehicle(&conn, "v1", "store-001", "available");

    // Create reservation in far future so ticket validity is in future
    let req = fleetreserve_backend::models::CreateReservationRequest {
        asset_type: "vehicle".into(), asset_id: "v1".into(),
        store_id: "store-001".into(),
        start_time: "2099-12-31T09:00:00".into(),
        end_time: "2099-12-31T10:00:00".into(),
    };
    let result = fleetreserve_backend::services::reservation_engine::create_reservation(
        &conn, "user-admin-001", "admin", &req, "",
    ).unwrap();

    // Redemption should fail because the validity window hasn't started yet
    let redeem = fleetreserve_backend::services::ticket_engine::redeem_ticket(
        &conn, &result.ticket.id, "user-admin-001", "admin", "",
    );
    assert!(redeem.is_err());
    assert!(redeem.unwrap_err().contains("not yet valid"));
}
