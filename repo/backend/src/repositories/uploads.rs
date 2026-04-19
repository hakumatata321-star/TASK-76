use rusqlite::Connection;
use crate::models::Upload;

pub fn create(conn: &Connection, u: &Upload) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO uploads (id, filename, content_type, size_bytes, sha256_fingerprint, vehicle_id, store_id, uploader_id) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        rusqlite::params![u.id, u.filename, u.content_type, u.size_bytes, u.sha256_fingerprint, u.vehicle_id, u.store_id, u.uploader_id],
    )?;
    Ok(())
}
