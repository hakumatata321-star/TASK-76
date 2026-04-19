use fleetreserve_backend::audit::chain::{append_audit_log, verify_chain_integrity};
use fleetreserve_backend::security::masking::{mask_license_plate, mask_vin, pseudonymize_user_id};
use rusqlite::Connection;

#[test]
fn unit_security_masking() {
    assert_eq!(mask_vin("1HGCM82633A123456"), "*************3456");
    assert_eq!(mask_license_plate("ABC1234"), "*****34");
}

#[test]
fn unit_security_pseudonymize_deterministic() {
    let id = "user-admin-001";
    let key = "test-hmac-key";
    let p1 = pseudonymize_user_id(id, key);
    let p2 = pseudonymize_user_id(id, key);
    // Deterministic: same inputs produce same pseudonym
    assert_eq!(p1, p2, "pseudonymize_user_id must be deterministic");
    // Pseudonym format: "ph-" prefix followed by hex chars
    assert!(p1.starts_with("ph-"), "pseudonym must start with 'ph-'");
    assert_eq!(p1.len(), 3 + 16, "pseudonym must be 'ph-' + 16 hex chars");
    // Different IDs produce different pseudonyms
    let p3 = pseudonymize_user_id("other-user-002", key);
    assert_ne!(p1, p3, "different user IDs must produce different pseudonyms");
    // Different keys produce different pseudonyms (HMAC key binds to deployment)
    let p4 = pseudonymize_user_id(id, "different-key");
    assert_ne!(p1, p4, "different HMAC keys must produce different pseudonyms");
    // Raw UUID must not appear in pseudonym
    assert!(!p1.contains(id), "raw user ID must not appear in pseudonym");
}

#[test]
fn unit_security_audit_chain() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(include_str!("../../migrations/001_initial_schema.sql")).unwrap();
    append_audit_log(&conn, "u1", "alice", "CREATE", "x", "1", &serde_json::json!({})).unwrap();
    append_audit_log(&conn, "u1", "alice", "UPDATE", "x", "1", &serde_json::json!({"a":1})).unwrap();
    assert!(verify_chain_integrity(&conn).unwrap());
}

#[test]
fn unit_security_audit_chain_secure_stores_pseudonym_not_raw_id() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(include_str!("../../migrations/001_initial_schema.sql")).unwrap();
    let raw_id = "user-admin-001";
    let hmac_key = "test-hmac-key-for-audit";
    fleetreserve_backend::audit::chain::append_audit_log_secure(
        &conn, raw_id, "admin", "LOGIN", "user", raw_id,
        &serde_json::json!({}), hmac_key,
    ).unwrap();
    // Verify the stored actor_id is a pseudonym, not the raw UUID
    let stored_actor_id: String = conn
        .query_row("SELECT actor_id FROM audit_log WHERE action = 'LOGIN'", [], |r| r.get(0))
        .unwrap();
    assert!(stored_actor_id.starts_with("ph-"), "audit_log.actor_id must be pseudonymized");
    assert!(!stored_actor_id.contains(raw_id), "raw user ID must not appear in audit_log.actor_id");
    assert!(verify_chain_integrity(&conn).unwrap());
}
