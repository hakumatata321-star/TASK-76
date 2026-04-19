use crate::models::RecoveryCode;
use rusqlite::Connection;

pub fn create(conn: &Connection, rc: &RecoveryCode) -> Result<(), rusqlite::Error> {
    // Invalidate existing unused codes for this user
    conn.execute(
        "UPDATE recovery_codes SET used = 1, used_at = datetime('now') WHERE user_id = ?1 AND used = 0",
        [&rc.user_id],
    )?;
    conn.execute(
        "INSERT INTO recovery_codes (id, user_id, code_hash, issued_by, expires_at) VALUES (?1,?2,?3,?4,?5)",
        rusqlite::params![rc.id, rc.user_id, rc.code_hash, rc.issued_by, rc.expires_at],
    )?;
    Ok(())
}

pub fn find_valid(conn: &Connection, user_id: &str, code_hash: &str) -> Result<Option<RecoveryCode>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, user_id, code_hash, issued_by, issued_at, expires_at, used
         FROM recovery_codes
         WHERE user_id = ?1 AND code_hash = ?2 AND used = 0
         ORDER BY issued_at DESC
         LIMIT 1",
    )?;
    let mut rows = stmt.query(rusqlite::params![user_id, code_hash])?;
    if let Some(row) = rows.next()? {
        let rc = RecoveryCode {
            id: row.get(0)?,
            user_id: row.get(1)?,
            code_hash: row.get(2)?,
            issued_by: row.get(3)?,
            issued_at: row.get(4)?,
            expires_at: row.get(5)?,
            used: row.get::<_, i64>(6)? != 0,
        };
        let expiry = chrono::DateTime::parse_from_rfc3339(&rc.expires_at)
            .map(|d| d.with_timezone(&chrono::Utc));
        if let Ok(expiry_utc) = expiry {
            if expiry_utc > chrono::Utc::now() {
                return Ok(Some(rc));
            }
        }
        Ok(None)
    } else {
        Ok(None)
    }
}

pub fn mark_used(conn: &Connection, id: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE recovery_codes SET used = 1, used_at = datetime('now') WHERE id = ?1",
        [id],
    )?;
    Ok(())
}
