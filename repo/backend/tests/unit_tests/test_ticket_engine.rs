use fleetreserve_backend::services::ticket_engine::{generate_ticket, redeem_ticket};
use rusqlite::Connection;

fn setup_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    conn.execute_batch(include_str!("../../migrations/001_initial_schema.sql")).unwrap();
    conn.execute("INSERT INTO stores (id, name, location) VALUES ('s1','S','L')", []).unwrap();
    conn.execute("INSERT INTO users (id, username, password_hash, display_name, role) VALUES ('u1','u1','x','U1','Customer')", []).unwrap();
    conn.execute("INSERT INTO users (id, username, password_hash, display_name, role) VALUES ('staff-1','staff1','x','S1','MerchantStaff')", []).unwrap();
    conn.execute(
        "INSERT INTO reservations (id, asset_type, asset_id, store_id, user_id, start_time, end_time, status) VALUES ('r1','vehicle','v1','s1','u1','2026-05-01T09:00:00','2026-05-01T10:00:00','confirmed')",
        [],
    ).unwrap();
    conn
}

#[test]
fn unit_ticket_engine_redeem_works() {
    let conn = setup_db();
    let t = generate_ticket(&conn, "r1", "2000-01-01T00:00:00", "2099-01-01T00:00:00").unwrap();
    assert!(redeem_ticket(&conn, &t.id, "staff-1", "staff1", "").is_ok());
}

#[test]
fn unit_ticket_engine_ticket_number_has_fr_prefix() {
    let conn = setup_db();
    let t = generate_ticket(&conn, "r1", "2000-01-01T00:00:00", "2099-01-01T00:00:00").unwrap();
    assert!(t.ticket_number.starts_with("FR-"), "Ticket number must start with FR-, got: {}", t.ticket_number);
    assert_eq!(t.ticket_number.len(), 11, "FR- (3) + 8 chars = 11 total");
}

#[test]
fn unit_ticket_engine_double_redeem_is_rejected() {
    let conn = setup_db();
    let t = generate_ticket(&conn, "r1", "2000-01-01T00:00:00", "2099-01-01T00:00:00").unwrap();
    redeem_ticket(&conn, &t.id, "staff-1", "staff1", "").expect("first redeem must succeed");
    let err = redeem_ticket(&conn, &t.id, "staff-1", "staff1", "").expect_err("second redeem must fail");
    assert!(err.contains("already been redeemed") || err.contains("redeemed"), "Expected redeem error, got: {}", err);
}

#[test]
fn unit_ticket_engine_generate_populates_qr_data() {
    let conn = setup_db();
    let t = generate_ticket(&conn, "r1", "2026-01-01T09:00:00", "2026-01-01T10:00:00").unwrap();
    let qr: serde_json::Value = serde_json::from_str(&t.qr_data).expect("qr_data must be valid JSON");
    assert_eq!(qr["reservation_id"], "r1");
    assert_eq!(qr["valid_from"], "2026-01-01T09:00:00");
    assert_eq!(qr["valid_until"], "2026-01-01T10:00:00");
    assert!(qr["ticket_number"].as_str().unwrap().starts_with("FR-"));
}
