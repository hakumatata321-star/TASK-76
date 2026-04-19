use rusqlite::Connection;

pub fn create(conn: &Connection, id: &str, filename: &str, path: &str, size: i64, sha256: &str, created_by: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO backups (id, filename, path, size_bytes, sha256, created_by) VALUES (?1,?2,?3,?4,?5,?6)",
        rusqlite::params![id, filename, path, size, sha256, created_by],
    )?;
    Ok(())
}
