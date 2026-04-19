use crate::models::PermissionRule;
use rusqlite::Connection;

/// Return true if the given role has an explicit entry for (resource, action)
/// OR a wildcard (resource='all', action='all') in the permissions table.
/// Used to gate handlers so admin-managed permission removals take effect.
pub fn has_permission(conn: &Connection, role: &str, resource: &str, action: &str) -> bool {
    conn.query_row(
        "SELECT COUNT(*) FROM permissions \
         WHERE role = ?1 \
           AND (resource = ?2 OR resource = 'all') \
           AND (action = ?3 OR action = 'all')",
        rusqlite::params![role, resource, action],
        |row| row.get::<_, i64>(0),
    )
    .map(|c| c > 0)
    .unwrap_or(false)
}

pub fn list_all(conn: &Connection) -> Result<Vec<PermissionRule>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, role, resource, action FROM permissions ORDER BY role, resource, action",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(PermissionRule {
            id: row.get(0)?,
            role: row.get(1)?,
            resource: row.get(2)?,
            action: row.get(3)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn upsert(conn: &Connection, role: &str, resource: &str, action: &str) -> Result<String, rusqlite::Error> {
    if let Ok(existing_id) = conn.query_row(
        "SELECT id FROM permissions WHERE role = ?1 AND resource = ?2 AND action = ?3",
        rusqlite::params![role, resource, action],
        |row| row.get::<_, String>(0),
    ) {
        return Ok(existing_id);
    }
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO permissions (id, role, resource, action) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![id, role, resource, action],
    )?;
    Ok(id)
}

pub fn delete_by_id(conn: &Connection, id: &str) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM permissions WHERE id = ?1", [id])?;
    Ok(())
}
