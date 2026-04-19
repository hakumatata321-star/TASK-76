use crate::models::ServiceBay;
use rusqlite::Connection;

pub fn find_by_store(conn: &Connection, store_id: &str) -> Result<Vec<ServiceBay>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, store_id, name, bay_type, capacity, status, version FROM service_bays WHERE store_id = ?1"
    )?;
    let bays = stmt.query_map([store_id], |row| {
        Ok(ServiceBay {
            id: row.get(0)?, store_id: row.get(1)?, name: row.get(2)?,
            bay_type: row.get(3)?, capacity: row.get(4)?, status: row.get(5)?, version: row.get(6)?,
        })
    })?.filter_map(|r| r.ok()).collect();
    Ok(bays)
}

pub fn find_by_id(conn: &Connection, id: &str) -> Result<Option<ServiceBay>, rusqlite::Error> {
    match conn.query_row(
        "SELECT id, store_id, name, bay_type, capacity, status, version FROM service_bays WHERE id = ?1",
        [id],
        |row| Ok(ServiceBay {
            id: row.get(0)?, store_id: row.get(1)?, name: row.get(2)?,
            bay_type: row.get(3)?, capacity: row.get(4)?, status: row.get(5)?, version: row.get(6)?,
        }),
    ) {
        Ok(b) => Ok(Some(b)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn create(conn: &Connection, b: &ServiceBay) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO service_bays (id, store_id, name, bay_type, capacity) VALUES (?1,?2,?3,?4,?5)",
        rusqlite::params![b.id, b.store_id, b.name, b.bay_type, b.capacity],
    )?;
    Ok(())
}
