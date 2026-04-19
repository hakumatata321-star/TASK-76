use rusqlite::Connection;

pub fn setup_test_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;").unwrap();
    let schema = include_str!("../../migrations/001_initial_schema.sql");
    conn.execute_batch(schema).unwrap();
    let seed = include_str!("../../migrations/002_seed_data.sql");
    conn.execute_batch(seed).unwrap();
    // Relax hours so integration tests can use wide reservation/ticket windows without flaking.
    conn.execute_batch(
        "UPDATE stores SET business_hours_start = '00:00', business_hours_end = '23:59' WHERE id IN ('store-001', 'store-002');",
    )
    .unwrap();
    conn
}

pub fn create_test_vehicle(conn: &Connection, id: &str, store_id: &str, status: &str) {
    conn.execute(
        "INSERT INTO vehicles (id, vin_encrypted, vin_hash, license_plate_encrypted, license_plate_hash, make, model, store_id, status, insurance_expiry, version) VALUES (?1, 'enc', 'hash', 'enc', 'hash', 'Test', 'Vehicle', ?2, ?3, '2100-01-01T00:00:00', 1)",
        rusqlite::params![id, store_id, status],
    ).unwrap();
}

#[allow(dead_code)]
pub fn create_test_bay(conn: &Connection, id: &str, store_id: &str, capacity: i64) {
    conn.execute(
        "INSERT INTO service_bays (id, store_id, name, bay_type, capacity, status, version) VALUES (?1, ?2, 'Test Bay', 'general', ?3, 'active', 1)",
        rusqlite::params![id, store_id, capacity],
    ).unwrap();
}

#[allow(dead_code)]
pub fn create_test_user(conn: &Connection, id: &str, username: &str, role: &str, store_id: Option<&str>) {
    conn.execute(
        "INSERT INTO users (id, username, password_hash, display_name, role, store_id) VALUES (?1, ?2, '$argon2id$v=19$m=19456,t=2,p=1$dGVzdHNhbHQ$dGVzdGhhc2g', ?2, ?3, ?4)",
        rusqlite::params![id, username, role, store_id],
    ).unwrap();
}
