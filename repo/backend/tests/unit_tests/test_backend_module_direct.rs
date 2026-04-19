use axum::body::Body;
use axum::http::Request;
use fleetreserve_backend::models::{
    PhotographerAssignment, Reservation, ServiceBay, Ticket, Upload, Vehicle,
};
use rusqlite::Connection;
use tower::ServiceExt;

fn setup_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    conn.execute_batch(include_str!("../../migrations/001_initial_schema.sql")).unwrap();
    conn.execute_batch(include_str!("../../migrations/002_seed_data.sql")).unwrap();
    conn.execute("UPDATE users SET active = 1 WHERE id = 'user-admin-001'", []).unwrap();
    conn
}

#[test]
fn unit_backend_crypto_service_reexport_roundtrip() {
    let plaintext = "sensitive-value";
    let key = "test-32-byte-encryption-key-value!!";
    let encrypted = fleetreserve_backend::services::crypto::encrypt_field(plaintext, key).unwrap();
    let decrypted = fleetreserve_backend::services::crypto::decrypt_field(&encrypted, key).unwrap();
    assert_eq!(decrypted, plaintext);
}

#[test]
fn unit_backend_backup_repository_create_row() {
    let conn = setup_db();
    fleetreserve_backend::repositories::backups::create(
        &conn,
        "b1",
        "backup.enc",
        "/tmp/backup.enc",
        123,
        "abc123",
        "user-admin-001",
    )
    .unwrap();
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM backups WHERE id = 'b1'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn unit_backend_assignments_repository_create_and_query() {
    let conn = setup_db();
    conn.execute(
        "INSERT INTO users (id, username, password_hash, display_name, role, store_id, active) VALUES ('photo-1','photo','x','Photo','Photographer','store-001',1)",
        [],
    )
    .unwrap();
    let a = PhotographerAssignment {
        id: "as1".into(),
        photographer_user_id: "photo-1".into(),
        store_id: "store-001".into(),
        job_description: "shoot".into(),
        vehicle_id: None,
        bay_id: None,
        start_time: "2026-01-01T09:00:00".into(),
        end_time: "2026-01-01T10:00:00".into(),
    };
    fleetreserve_backend::repositories::assignments::create(&conn, &a).unwrap();
    let by_photo = fleetreserve_backend::repositories::assignments::find_by_photographer(&conn, "photo-1").unwrap();
    assert_eq!(by_photo.len(), 1);
    let by_store = fleetreserve_backend::repositories::assignments::find_by_store(&conn, "store-001").unwrap();
    assert!(!by_store.is_empty());
}

#[test]
fn unit_backend_bays_repository_create_find_by_id_and_store() {
    let conn = setup_db();
    let bay = ServiceBay {
        id: "bay-1".into(),
        store_id: "store-001".into(),
        name: "Detail Bay".into(),
        bay_type: "detail".into(),
        capacity: 2,
        status: "active".into(),
        version: 1,
    };
    fleetreserve_backend::repositories::bays::create(&conn, &bay).unwrap();
    let found = fleetreserve_backend::repositories::bays::find_by_id(&conn, "bay-1").unwrap().unwrap();
    assert_eq!(found.name, "Detail Bay");
    let list = fleetreserve_backend::repositories::bays::find_by_store(&conn, "store-001").unwrap();
    assert!(!list.is_empty());
}

#[test]
fn unit_backend_uploads_repository_create_row() {
    let conn = setup_db();
    let upload = Upload {
        id: "u1".into(),
        filename: "photo.jpg".into(),
        content_type: "image/jpeg".into(),
        size_bytes: 100,
        sha256_fingerprint: "fp1".into(),
        vehicle_id: Some("v1".into()),
        store_id: Some("store-001".into()),
        uploader_id: "user-admin-001".into(),
    };
    fleetreserve_backend::repositories::uploads::create(&conn, &upload).unwrap();
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM uploads WHERE id='u1'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn unit_backend_tickets_repository_find_by_id_and_number() {
    let conn = setup_db();
    conn.execute(
        "INSERT INTO reservations (id, asset_type, asset_id, store_id, user_id, start_time, end_time, status, version) VALUES ('r1','vehicle','v1','store-001','user-admin-001','2026-01-01T09:00:00','2026-01-01T10:00:00','confirmed',1)",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO tickets (id, ticket_number, reservation_id, qr_data, valid_from, valid_until, redeemed, undone) VALUES ('t1','FR-ABC12345','r1','{}','2026-01-01T09:00:00','2026-01-01T10:00:00',0,0)",
        [],
    )
    .unwrap();
    let t1 = fleetreserve_backend::repositories::tickets::find_by_id(&conn, "t1").unwrap().unwrap();
    assert_eq!(t1.ticket_number, "FR-ABC12345");
    let t2 = fleetreserve_backend::repositories::tickets::find_by_number(&conn, "FR-ABC12345").unwrap().unwrap();
    assert_eq!(t2.id, "t1");
}

#[test]
fn unit_backend_reservations_repository_queries() {
    let conn = setup_db();
    conn.execute(
        "INSERT INTO reservations (id, asset_type, asset_id, store_id, user_id, start_time, end_time, status, version) VALUES ('r2','vehicle','v2','store-001','user-admin-001','2026-01-02T09:00:00','2026-01-02T10:00:00','confirmed',1)",
        [],
    )
    .unwrap();
    let by_user = fleetreserve_backend::repositories::reservations::find_by_user(&conn, "user-admin-001").unwrap();
    assert!(!by_user.is_empty());
    let by_store = fleetreserve_backend::repositories::reservations::find_by_store(&conn, "store-001").unwrap();
    assert!(!by_store.is_empty());
    let all = fleetreserve_backend::repositories::reservations::find_all(&conn).unwrap();
    assert!(!all.is_empty());
}

#[tokio::test]
async fn unit_backend_security_headers_are_attached() {
    let state = {
        use fleetreserve_backend::app::state::AppState;
        use std::collections::{HashMap, HashSet};
        use std::sync::{Arc, Mutex};
        let conn = setup_db();
        AppState {
            db: Arc::new(Mutex::new(conn)),
            encryption_key: "x".repeat(32),
            hmac_secret: "y".repeat(32),
            upload_dir: "/tmp".into(),
            csrf_tokens: Arc::new(Mutex::new(HashMap::new())),
            revoked_sessions: Arc::new(Mutex::new(HashSet::new())),
        }
    };
    let app = fleetreserve_backend::routes::build_router(state);
    let response = app
        .oneshot(Request::builder().uri("/does-not-exist").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let headers = response.headers();
    assert!(headers.get("content-security-policy").is_some());
    assert_eq!(headers.get("x-frame-options").unwrap(), "DENY");
    assert_eq!(headers.get("x-content-type-options").unwrap(), "nosniff");
}

#[test]
fn unit_backend_backup_module_is_present() {
    // This test keeps a direct reference to the backup module itself.
    let module_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/backup/mod.rs");
    assert!(module_path.exists(), "backup module file should exist");
}
