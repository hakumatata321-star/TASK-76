use crate::models::Store;
use rusqlite::Connection;

pub fn find_by_id(conn: &Connection, id: &str) -> Result<Option<Store>, rusqlite::Error> {
    match conn.query_row(
        "SELECT id, name, location, business_hours_start, business_hours_end, active FROM stores WHERE id = ?1",
        [id],
        |row| Ok(Store {
            id: row.get(0)?, name: row.get(1)?, location: row.get(2)?,
            business_hours_start: row.get(3)?, business_hours_end: row.get(4)?,
            active: row.get::<_, i64>(5)? != 0,
        }),
    ) {
        Ok(s) => Ok(Some(s)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn find_all(conn: &Connection) -> Result<Vec<Store>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT id, name, location, business_hours_start, business_hours_end, active FROM stores WHERE active = 1")?;
    let rows = stmt.query_map([], |row| Ok(Store {
        id: row.get(0)?, name: row.get(1)?, location: row.get(2)?,
        business_hours_start: row.get(3)?, business_hours_end: row.get(4)?,
        active: row.get::<_, i64>(5)? != 0,
    }))?;
    let items: Vec<Store> = rows.filter_map(|r| r.ok()).collect();
    Ok(items)
}
