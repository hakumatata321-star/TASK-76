use crate::models::Reservation;
use rusqlite::Connection;

pub fn find_by_user(conn: &Connection, user_id: &str) -> Result<Vec<Reservation>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, asset_type, asset_id, store_id, user_id, start_time, end_time, status, ticket_id, version FROM reservations WHERE user_id = ?1 ORDER BY start_time"
    )?;
    let rows = stmt.query_map([user_id], |row| row_to_reservation(row))?;
    let items: Vec<Reservation> = rows.filter_map(|r| r.ok()).collect();
    Ok(items)
}

pub fn find_by_store(conn: &Connection, store_id: &str) -> Result<Vec<Reservation>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, asset_type, asset_id, store_id, user_id, start_time, end_time, status, ticket_id, version FROM reservations WHERE store_id = ?1 ORDER BY start_time"
    )?;
    let rows = stmt.query_map([store_id], |row| row_to_reservation(row))?;
    let items: Vec<Reservation> = rows.filter_map(|r| r.ok()).collect();
    Ok(items)
}

pub fn find_all(conn: &Connection) -> Result<Vec<Reservation>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, asset_type, asset_id, store_id, user_id, start_time, end_time, status, ticket_id, version FROM reservations ORDER BY start_time"
    )?;
    let rows = stmt.query_map([], |row| row_to_reservation(row))?;
    let items: Vec<Reservation> = rows.filter_map(|r| r.ok()).collect();
    Ok(items)
}

/// Return reservations whose asset is assigned to the given photographer.
/// Empty asset lists produce an empty result rather than a full-table scan.
pub fn find_for_photographer(conn: &Connection, photographer_id: &str) -> Result<Vec<Reservation>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT r.id, r.asset_type, r.asset_id, r.store_id, r.user_id,
                r.start_time, r.end_time, r.status, r.ticket_id, r.version
         FROM reservations r
         JOIN photographer_assignments pa ON (
             (r.asset_type = 'vehicle' AND r.asset_id = pa.vehicle_id)
             OR (r.asset_type = 'bay'     AND r.asset_id = pa.bay_id)
         )
         WHERE pa.photographer_user_id = ?1
         ORDER BY r.start_time",
    )?;
    let rows = stmt.query_map([photographer_id], |row| row_to_reservation(row))?;
    let items: Vec<Reservation> = rows.filter_map(|r| r.ok()).collect();
    Ok(items)
}

fn row_to_reservation(row: &rusqlite::Row) -> Result<Reservation, rusqlite::Error> {
    Ok(Reservation {
        id: row.get(0)?, asset_type: row.get(1)?, asset_id: row.get(2)?,
        store_id: row.get(3)?, user_id: row.get(4)?, start_time: row.get(5)?,
        end_time: row.get(6)?, status: row.get(7)?, ticket_id: row.get(8)?, version: row.get(9)?,
    })
}
