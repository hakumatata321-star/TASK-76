use crate::models::Vehicle;
use rusqlite::Connection;

pub fn find_by_id(conn: &Connection, id: &str) -> Result<Option<Vehicle>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, vin_encrypted, vin_hash, license_plate_encrypted, license_plate_hash, make, model, trim_level, store_id, mileage_miles, fuel_or_battery_pct, status, maintenance_due, inspection_due, insurance_expiry, version FROM vehicles WHERE id = ?1"
    )?;
    match stmt.query_row([id], |row| Ok(row_to_vehicle(row)?)) {
        Ok(v) => Ok(Some(v)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn find_by_store(conn: &Connection, store_id: &str) -> Result<Vec<Vehicle>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, vin_encrypted, vin_hash, license_plate_encrypted, license_plate_hash, make, model, trim_level, store_id, mileage_miles, fuel_or_battery_pct, status, maintenance_due, inspection_due, insurance_expiry, version FROM vehicles WHERE store_id = ?1 ORDER BY make, model"
    )?;
    let vehicles = stmt.query_map([store_id], |row| row_to_vehicle(row))?
        .filter_map(|r| r.ok()).collect();
    Ok(vehicles)
}

pub fn find_all(conn: &Connection) -> Result<Vec<Vehicle>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, vin_encrypted, vin_hash, license_plate_encrypted, license_plate_hash, make, model, trim_level, store_id, mileage_miles, fuel_or_battery_pct, status, maintenance_due, inspection_due, insurance_expiry, version FROM vehicles ORDER BY make, model"
    )?;
    let vehicles = stmt.query_map([], |row| row_to_vehicle(row))?
        .filter_map(|r| r.ok()).collect();
    Ok(vehicles)
}

pub fn create(conn: &Connection, v: &Vehicle) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO vehicles (id, vin_encrypted, vin_hash, license_plate_encrypted, license_plate_hash, make, model, trim_level, store_id, mileage_miles, fuel_or_battery_pct, status, maintenance_due, inspection_due, insurance_expiry) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)",
        rusqlite::params![v.id, v.vin_encrypted, v.vin_hash, v.license_plate_encrypted, v.license_plate_hash, v.make, v.model, v.trim_level, v.store_id, v.mileage_miles, v.fuel_or_battery_pct, v.status, v.maintenance_due, v.inspection_due, v.insurance_expiry],
    )?;
    Ok(())
}

pub fn update_status(conn: &Connection, id: &str, status: &str, expected_version: i64) -> Result<bool, rusqlite::Error> {
    let updated = conn.execute(
        "UPDATE vehicles SET status = ?1, version = version + 1, updated_at = datetime('now') WHERE id = ?2 AND version = ?3",
        rusqlite::params![status, id, expected_version],
    )?;
    Ok(updated > 0)
}

fn row_to_vehicle(row: &rusqlite::Row) -> Result<Vehicle, rusqlite::Error> {
    Ok(Vehicle {
        id: row.get(0)?,
        vin_encrypted: row.get(1)?,
        vin_hash: row.get(2)?,
        license_plate_encrypted: row.get(3)?,
        license_plate_hash: row.get(4)?,
        make: row.get(5)?,
        model: row.get(6)?,
        trim_level: row.get(7)?,
        store_id: row.get(8)?,
        mileage_miles: row.get(9)?,
        fuel_or_battery_pct: row.get(10)?,
        status: row.get(11)?,
        maintenance_due: row.get(12)?,
        inspection_due: row.get(13)?,
        insurance_expiry: row.get(14)?,
        version: row.get(15)?,
    })
}
