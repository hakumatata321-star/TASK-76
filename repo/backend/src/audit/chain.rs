use rusqlite::Connection;
use sha2::{Sha256, Digest};

/// Pseudonymize actor_id and mask actor_username before storing, so the raw
/// user UUID is never written to audit_log at rest. The hash chain remains
/// intact because verification uses whatever value is stored.
pub fn append_audit_log_secure(
    conn: &Connection,
    actor_id: &str,
    actor_username: &str,
    action: &str,
    resource_type: &str,
    resource_id: &str,
    details: &serde_json::Value,
    hmac_key: &str,
) -> Result<i64, rusqlite::Error> {
    let pseudo_id = crate::security::masking::pseudonymize_user_id(actor_id, hmac_key);
    let masked_username = crate::security::masking::mask_username(actor_username);
    append_audit_log(conn, &pseudo_id, &masked_username, action, resource_type, resource_id, details)
}

pub fn append_audit_log(
    conn: &Connection,
    actor_id: &str,
    actor_username: &str,
    action: &str,
    resource_type: &str,
    resource_id: &str,
    details: &serde_json::Value,
) -> Result<i64, rusqlite::Error> {
    let details_json = serde_json::to_string(details).unwrap_or_else(|_| "{}".to_string());
    let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let previous_hash: String = conn
        .query_row(
            "SELECT current_hash FROM audit_log ORDER BY id DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or_default();

    let hash_input = format!(
        "{}{}{}{}{}{}{}",
        previous_hash, timestamp, actor_id, action, resource_type, resource_id, details_json
    );
    let mut hasher = Sha256::new();
    hasher.update(hash_input.as_bytes());
    let current_hash = hex::encode(hasher.finalize());

    conn.execute(
        "INSERT INTO audit_log (timestamp, actor_id, actor_username, action, resource_type, resource_id, details_json, previous_hash, current_hash) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![timestamp, actor_id, actor_username, action, resource_type, resource_id, details_json, previous_hash, current_hash],
    )?;

    let id = conn.last_insert_rowid();

    // Check if we should create a hash anchor (every 100 entries)
    if id % 100 == 0 {
        let _ = super::anchors::create_hash_anchor(conn);
    }

    Ok(id)
}

pub fn verify_chain_integrity(conn: &Connection) -> Result<bool, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, timestamp, actor_id, action, resource_type, resource_id, details_json, previous_hash, current_hash FROM audit_log ORDER BY id ASC"
    )?;

    let mut last_hash = String::new();
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, String>(6)?,
            row.get::<_, String>(7)?,
            row.get::<_, String>(8)?,
        ))
    })?;

    for row in rows {
        let (_id, timestamp, actor_id, action, resource_type, resource_id, details_json, previous_hash, current_hash) = row?;

        if previous_hash != last_hash {
            return Ok(false);
        }

        let hash_input = format!(
            "{}{}{}{}{}{}{}",
            previous_hash, timestamp, actor_id, action, resource_type, resource_id, details_json
        );
        let mut hasher = Sha256::new();
        hasher.update(hash_input.as_bytes());
        let computed = hex::encode(hasher.finalize());

        if computed != current_hash {
            return Ok(false);
        }

        last_hash = current_hash;
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        let schema = include_str!("../../migrations/001_initial_schema.sql");
        conn.execute_batch(schema).unwrap();
        conn
    }

    #[test]
    fn test_audit_chain_append() {
        let conn = setup_test_db();
        let id = append_audit_log(
            &conn, "user-1", "admin", "LOGIN", "user", "user-1",
            &serde_json::json!({"ip": "127.0.0.1"}),
        ).unwrap();
        assert!(id > 0);
    }

    #[test]
    fn test_audit_chain_integrity() {
        let conn = setup_test_db();
        for i in 0..5 {
            append_audit_log(
                &conn, "user-1", "admin", "TEST", "test", &format!("res-{}", i),
                &serde_json::json!({"seq": i}),
            ).unwrap();
        }
        assert!(verify_chain_integrity(&conn).unwrap());
    }

    #[test]
    fn test_audit_log_update_blocked_by_trigger() {
        let conn = setup_test_db();
        for i in 0..3 {
            append_audit_log(
                &conn, "user-1", "admin", "TEST", "test", &format!("res-{}", i),
                &serde_json::json!({}),
            ).unwrap();
        }

        // Attempt to tamper: the trigger should block this
        let result = conn.execute(
            "UPDATE audit_log SET action = 'TAMPERED' WHERE id = 2",
            [],
        );
        assert!(result.is_err(), "UPDATE on audit_log should be blocked by append-only trigger");
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("append-only"), "Error should mention append-only: {}", err_msg);
    }

    #[test]
    fn test_audit_log_delete_blocked_by_trigger() {
        let conn = setup_test_db();
        append_audit_log(
            &conn, "user-1", "admin", "TEST", "test", "res-1",
            &serde_json::json!({}),
        ).unwrap();

        let result = conn.execute("DELETE FROM audit_log WHERE id = 1", []);
        assert!(result.is_err(), "DELETE on audit_log should be blocked by append-only trigger");
    }

    #[test]
    fn test_audit_chain_detects_corrupted_data() {
        // Test chain integrity with a separate DB that has no triggers (simulating external corruption)
        let conn = Connection::open_in_memory().unwrap();
        // Create table WITHOUT triggers to simulate external tampering
        conn.execute_batch("
            CREATE TABLE audit_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL DEFAULT (datetime('now')),
                actor_id TEXT NOT NULL,
                actor_username TEXT NOT NULL,
                action TEXT NOT NULL,
                resource_type TEXT NOT NULL,
                resource_id TEXT NOT NULL,
                details_json TEXT NOT NULL DEFAULT '{}',
                previous_hash TEXT NOT NULL DEFAULT '',
                current_hash TEXT NOT NULL
            );
        ").unwrap();
        // Manually insert valid chain entries
        for i in 0..3 {
            append_audit_log(
                &conn, "user-1", "admin", "TEST", "test", &format!("res-{}", i),
                &serde_json::json!({}),
            ).unwrap();
        }
        assert!(verify_chain_integrity(&conn).unwrap());

        // Now tamper (no trigger in this test DB)
        conn.execute("UPDATE audit_log SET action = 'TAMPERED' WHERE id = 2", []).unwrap();
        assert!(!verify_chain_integrity(&conn).unwrap());
    }

    #[test]
    fn test_hash_chain_links_previous() {
        let conn = setup_test_db();
        append_audit_log(&conn, "u1", "admin", "A1", "t", "r1", &serde_json::json!({})).unwrap();
        append_audit_log(&conn, "u1", "admin", "A2", "t", "r2", &serde_json::json!({})).unwrap();

        let hash1: String = conn.query_row(
            "SELECT current_hash FROM audit_log WHERE id = 1", [], |r| r.get(0),
        ).unwrap();
        let prev2: String = conn.query_row(
            "SELECT previous_hash FROM audit_log WHERE id = 2", [], |r| r.get(0),
        ).unwrap();
        assert_eq!(hash1, prev2);
    }
}
