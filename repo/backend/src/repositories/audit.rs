use crate::models::AuditLogEntry;
use rusqlite::Connection;

pub fn list_by_resource(conn: &Connection, resource_type: &str, resource_id: &str) -> Result<Vec<AuditLogEntry>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, timestamp, actor_id, actor_username, action, resource_type, resource_id, details_json, previous_hash, current_hash FROM audit_log WHERE resource_type = ?1 AND resource_id = ?2 ORDER BY id DESC"
    )?;
    let rows = stmt.query_map(rusqlite::params![resource_type, resource_id], row_to_entry)?;
    let items: Vec<AuditLogEntry> = rows.filter_map(|r| r.ok()).collect();
    Ok(items)
}

pub fn list_recent(conn: &Connection, limit: i64) -> Result<Vec<AuditLogEntry>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, timestamp, actor_id, actor_username, action, resource_type, resource_id, details_json, previous_hash, current_hash FROM audit_log ORDER BY id DESC LIMIT ?1"
    )?;
    let rows = stmt.query_map([limit], row_to_entry)?;
    let items: Vec<AuditLogEntry> = rows.filter_map(|r| r.ok()).collect();
    Ok(items)
}

fn row_to_entry(row: &rusqlite::Row) -> Result<AuditLogEntry, rusqlite::Error> {
    Ok(AuditLogEntry {
        id: row.get(0)?, timestamp: row.get(1)?, actor_id: row.get(2)?,
        actor_username: row.get(3)?, action: row.get(4)?, resource_type: row.get(5)?,
        resource_id: row.get(6)?, details_json: row.get(7)?,
        previous_hash: row.get(8)?, current_hash: row.get(9)?,
    })
}
