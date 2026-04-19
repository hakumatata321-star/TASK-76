use crate::models::PhotographerAssignment;
use rusqlite::Connection;

pub fn find_by_photographer(conn: &Connection, photographer_id: &str) -> Result<Vec<PhotographerAssignment>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, photographer_user_id, store_id, job_description, vehicle_id, bay_id, start_time, end_time FROM photographer_assignments WHERE photographer_user_id = ?1 ORDER BY start_time"
    )?;
    let rows = stmt.query_map([photographer_id], |row| Ok(PhotographerAssignment {
        id: row.get(0)?, photographer_user_id: row.get(1)?, store_id: row.get(2)?,
        job_description: row.get(3)?, vehicle_id: row.get(4)?, bay_id: row.get(5)?,
        start_time: row.get(6)?, end_time: row.get(7)?,
    }))?;
    let items: Vec<PhotographerAssignment> = rows.filter_map(|r| r.ok()).collect();
    Ok(items)
}

pub fn find_by_store(conn: &Connection, store_id: &str) -> Result<Vec<PhotographerAssignment>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, photographer_user_id, store_id, job_description, vehicle_id, bay_id, start_time, end_time FROM photographer_assignments WHERE store_id = ?1 ORDER BY start_time"
    )?;
    let rows = stmt.query_map([store_id], |row| Ok(PhotographerAssignment {
        id: row.get(0)?, photographer_user_id: row.get(1)?, store_id: row.get(2)?,
        job_description: row.get(3)?, vehicle_id: row.get(4)?, bay_id: row.get(5)?,
        start_time: row.get(6)?, end_time: row.get(7)?,
    }))?;
    let items: Vec<PhotographerAssignment> = rows.filter_map(|r| r.ok()).collect();
    Ok(items)
}

pub fn create(conn: &Connection, a: &PhotographerAssignment) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO photographer_assignments (id, photographer_user_id, store_id, job_description, vehicle_id, bay_id, start_time, end_time) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        rusqlite::params![a.id, a.photographer_user_id, a.store_id, a.job_description, a.vehicle_id, a.bay_id, a.start_time, a.end_time],
    )?;
    Ok(())
}
