use fleetreserve_backend::models::CreateReservationRequest;
use fleetreserve_backend::services::reservation_engine::create_reservation;
use rusqlite::Connection;

fn setup_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    conn.execute_batch(include_str!("../../migrations/001_initial_schema.sql")).unwrap();
    conn.execute_batch(include_str!("../../migrations/002_seed_data.sql")).unwrap();
    conn.execute("UPDATE users SET active = 1 WHERE id = 'user-admin-001'", []).unwrap();
    conn.execute(
        "INSERT INTO vehicles (id, vin_encrypted, vin_hash, license_plate_encrypted, license_plate_hash, make, model, store_id, status, insurance_expiry, version) VALUES ('v1', 'enc', 'h', 'enc', 'h', 'T', 'V', 'store-001', 'available', '2100-01-01T00:00:00', 1)",
        [],
    ).unwrap();
    conn
}

#[test]
fn unit_reservation_engine_happy_path() {
    let conn = setup_db();
    let req = CreateReservationRequest {
        asset_type: "vehicle".into(),
        asset_id: "v1".into(),
        store_id: "store-001".into(),
        start_time: "2026-05-01T09:00:00".into(),
        end_time: "2026-05-01T10:00:00".into(),
    };
    let out = create_reservation(&conn, "user-admin-001", "admin", &req, "").unwrap();
    assert_eq!(out.reservation.status, "confirmed");
}

#[test]
fn unit_reservation_engine_produces_ticket_with_fr_prefix() {
    let conn = setup_db();
    let req = CreateReservationRequest {
        asset_type: "vehicle".into(),
        asset_id: "v1".into(),
        store_id: "store-001".into(),
        start_time: "2026-05-02T09:00:00".into(),
        end_time: "2026-05-02T10:00:00".into(),
    };
    let out = create_reservation(&conn, "user-admin-001", "admin", &req, "").unwrap();
    assert!(out.ticket.ticket_number.starts_with("FR-"), "Ticket number must start with FR-");
}

#[test]
fn unit_reservation_engine_end_before_start_fails() {
    let conn = setup_db();
    let req = CreateReservationRequest {
        asset_type: "vehicle".into(),
        asset_id: "v1".into(),
        store_id: "store-001".into(),
        start_time: "2026-05-03T10:00:00".into(),
        end_time: "2026-05-03T09:00:00".into(),
    };
    let err = create_reservation(&conn, "user-admin-001", "admin", &req, "").unwrap_err();
    assert!(err.conflict);
    let messages: Vec<&str> = err.reasons.iter().map(|r| r.message.as_str()).collect();
    assert!(
        messages.iter().any(|m| m.contains("end_time") || m.contains("after")),
        "Expected validation message about end_time, got: {:?}",
        messages
    );
}

#[test]
fn unit_reservation_engine_overlapping_reservation_is_rejected() {
    let conn = setup_db();
    let req = CreateReservationRequest {
        asset_type: "vehicle".into(),
        asset_id: "v1".into(),
        store_id: "store-001".into(),
        start_time: "2026-05-10T09:00:00".into(),
        end_time: "2026-05-10T11:00:00".into(),
    };
    create_reservation(&conn, "user-admin-001", "admin", &req, "").unwrap();
    let err = create_reservation(&conn, "user-admin-001", "admin", &req, "").unwrap_err();
    assert!(err.conflict);
}

#[test]
fn unit_reservation_engine_vehicle_in_repair_is_rejected() {
    let conn = setup_db();
    conn.execute("UPDATE vehicles SET status = 'in-repair' WHERE id = 'v1'", []).unwrap();
    let req = CreateReservationRequest {
        asset_type: "vehicle".into(),
        asset_id: "v1".into(),
        store_id: "store-001".into(),
        start_time: "2026-05-11T09:00:00".into(),
        end_time: "2026-05-11T10:00:00".into(),
    };
    let err = create_reservation(&conn, "user-admin-001", "admin", &req, "").unwrap_err();
    assert!(err.conflict);
    let codes: Vec<&str> = err.reasons.iter().map(|r| r.code.as_str()).collect();
    assert!(codes.contains(&"in_repair_hold"), "Expected in_repair_hold reason, got: {:?}", codes);
}
