use crate::models::{User, UserRole};
use rusqlite::Connection;

pub fn find_by_username(conn: &Connection, username: &str) -> Result<Option<User>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, username, password_hash, display_name, email_encrypted, role, store_id, active FROM users WHERE username = ?1 AND active = 1"
    )?;
    let user = stmt.query_row([username], |row| {
        Ok(User {
            id: row.get(0)?,
            username: row.get(1)?,
            password_hash: row.get(2)?,
            display_name: row.get(3)?,
            email_encrypted: row.get(4)?,
            role: UserRole::from_str(&row.get::<_, String>(5)?).unwrap_or(UserRole::Customer),
            store_id: row.get(6)?,
            active: row.get::<_, i64>(7)? != 0,
        })
    });
    match user {
        Ok(u) => Ok(Some(u)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn find_by_id(conn: &Connection, id: &str) -> Result<Option<User>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, username, password_hash, display_name, email_encrypted, role, store_id, active FROM users WHERE id = ?1"
    )?;
    let user = stmt.query_row([id], |row| {
        Ok(User {
            id: row.get(0)?,
            username: row.get(1)?,
            password_hash: row.get(2)?,
            display_name: row.get(3)?,
            email_encrypted: row.get(4)?,
            role: UserRole::from_str(&row.get::<_, String>(5)?).unwrap_or(UserRole::Customer),
            store_id: row.get(6)?,
            active: row.get::<_, i64>(7)? != 0,
        })
    });
    match user {
        Ok(u) => Ok(Some(u)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn create(conn: &Connection, id: &str, username: &str, password_hash: &str, display_name: &str, role: &str, store_id: Option<&str>) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO users (id, username, password_hash, display_name, role, store_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![id, username, password_hash, display_name, role, store_id],
    )?;
    Ok(())
}

pub fn update_password(conn: &Connection, user_id: &str, new_hash: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE users SET password_hash = ?1, updated_at = datetime('now') WHERE id = ?2",
        rusqlite::params![new_hash, user_id],
    )?;
    Ok(())
}

pub fn list_all(conn: &Connection) -> Result<Vec<User>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, username, password_hash, display_name, email_encrypted, role, store_id, active FROM users ORDER BY username"
    )?;
    let users = stmt.query_map([], |row| {
        Ok(User {
            id: row.get(0)?,
            username: row.get(1)?,
            password_hash: row.get(2)?,
            display_name: row.get(3)?,
            email_encrypted: row.get(4)?,
            role: UserRole::from_str(&row.get::<_, String>(5)?).unwrap_or(UserRole::Customer),
            store_id: row.get(6)?,
            active: row.get::<_, i64>(7)? != 0,
        })
    })?.filter_map(|r| r.ok()).collect();
    Ok(users)
}

pub fn update_role(conn: &Connection, user_id: &str, role: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE users SET role = ?1, updated_at = datetime('now') WHERE id = ?2",
        rusqlite::params![role, user_id],
    )?;
    Ok(())
}

pub fn update_active(conn: &Connection, user_id: &str, active: bool) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE users SET active = ?1, updated_at = datetime('now') WHERE id = ?2",
        rusqlite::params![active as i64, user_id],
    )?;
    Ok(())
}
