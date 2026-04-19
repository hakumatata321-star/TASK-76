use fleetreserve_backend::models::{RecoveryCode, Vehicle};
use rusqlite::Connection;

fn setup_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    conn.execute_batch(include_str!("../../migrations/001_initial_schema.sql")).unwrap();
    conn.execute_batch(include_str!("../../migrations/002_seed_data.sql")).unwrap();
    conn.execute("UPDATE users SET active = 1 WHERE id = 'user-admin-001'", []).unwrap();
    conn
}

#[test]
fn unit_repositories_users_create_and_find() {
    let conn = setup_db();
    fleetreserve_backend::repositories::users::create(
        &conn,
        "u-repo-1",
        "repo_user",
        "hash",
        "Repo User",
        "Customer",
        Some("store-001"),
    )
    .unwrap();
    let user = fleetreserve_backend::repositories::users::find_by_username(&conn, "repo_user")
        .unwrap()
        .expect("user");
    assert_eq!(user.id, "u-repo-1");
    assert_eq!(user.role.as_str(), "Customer");
}

#[test]
fn unit_repositories_permissions_upsert_and_delete() {
    let conn = setup_db();
    let id = fleetreserve_backend::repositories::permissions::upsert(
        &conn,
        "Customer",
        "vehicle",
        "repo-test-action",
    )
    .unwrap();
    let all = fleetreserve_backend::repositories::permissions::list_all(&conn).unwrap();
    assert!(all.iter().any(|p| p.id == id));
    fleetreserve_backend::repositories::permissions::delete_by_id(&conn, &id).unwrap();
    let after = fleetreserve_backend::repositories::permissions::list_all(&conn).unwrap();
    assert!(!after.iter().any(|p| p.id == id));
}

#[test]
fn unit_repositories_stores_and_vehicles_queries() {
    let conn = setup_db();
    let stores = fleetreserve_backend::repositories::stores::find_all(&conn).unwrap();
    assert!(stores.len() >= 2);

    let vehicle = Vehicle {
        id: "repo-v1".into(),
        vin_encrypted: "enc".into(),
        vin_hash: "h1".into(),
        license_plate_encrypted: "enc".into(),
        license_plate_hash: "h2".into(),
        make: "Make".into(),
        model: "Model".into(),
        trim_level: "".into(),
        store_id: "store-001".into(),
        mileage_miles: 1000,
        fuel_or_battery_pct: 80.0,
        status: "available".into(),
        maintenance_due: None,
        inspection_due: None,
        insurance_expiry: Some("2099-01-01T00:00:00".into()),
        version: 1,
    };
    fleetreserve_backend::repositories::vehicles::create(&conn, &vehicle).unwrap();
    let found = fleetreserve_backend::repositories::vehicles::find_by_id(&conn, "repo-v1")
        .unwrap()
        .expect("vehicle");
    assert_eq!(found.store_id, "store-001");
}

#[test]
fn unit_repositories_audit_and_recovery_code_roundtrip() {
    let conn = setup_db();
    fleetreserve_backend::audit::chain::append_audit_log(
        &conn,
        "user-admin-001",
        "admin",
        "TEST",
        "repo",
        "1",
        &serde_json::json!({"k":"v"}),
    )
    .unwrap();
    let logs = fleetreserve_backend::repositories::audit::list_recent(&conn, 10).unwrap();
    assert!(!logs.is_empty());

    let rc = RecoveryCode {
        id: "rc-test-1".into(),
        user_id: "user-admin-001".into(),
        code_hash: "codehash".into(),
        issued_by: "user-admin-001".into(),
        issued_at: "2026-01-01T00:00:00Z".into(),
        expires_at: "2099-01-01T00:00:00Z".into(),
        used: false,
    };
    fleetreserve_backend::repositories::recovery_codes::create(&conn, &rc).unwrap();
    let found = fleetreserve_backend::repositories::recovery_codes::find_valid(
        &conn,
        "user-admin-001",
        "codehash",
    )
    .unwrap();
    assert!(found.is_some());
}
