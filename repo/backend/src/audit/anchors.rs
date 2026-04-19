use rusqlite::Connection;
use sha2::{Sha256, Digest};

pub fn create_hash_anchor(conn: &Connection) -> Result<i64, rusqlite::Error> {
    let last_anchor_log_id: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(last_log_id), 0) FROM audit_hash_anchors",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let mut stmt = conn.prepare(
        "SELECT current_hash FROM audit_log WHERE id > ?1 ORDER BY id ASC"
    )?;

    let hashes: Vec<String> = stmt
        .query_map([last_anchor_log_id], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    if hashes.is_empty() {
        return Ok(0);
    }

    let mut hasher = Sha256::new();
    for h in &hashes {
        hasher.update(h.as_bytes());
    }
    let cumulative_hash = hex::encode(hasher.finalize());

    let last_log_id: i64 = conn.query_row(
        "SELECT MAX(id) FROM audit_log",
        [],
        |row| row.get(0),
    )?;

    conn.execute(
        "INSERT INTO audit_hash_anchors (anchor_time, last_log_id, cumulative_hash) VALUES (datetime('now'), ?1, ?2)",
        rusqlite::params![last_log_id, cumulative_hash],
    )?;

    Ok(conn.last_insert_rowid())
}

pub fn should_create_anchor(conn: &Connection) -> bool {
    let last_anchor_log_id: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(last_log_id), 0) FROM audit_hash_anchors",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let current_max: i64 = conn
        .query_row("SELECT COALESCE(MAX(id), 0) FROM audit_log", [], |row| row.get(0))
        .unwrap_or(0);

    (current_max - last_anchor_log_id) >= 100
}
