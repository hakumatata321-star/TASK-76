use crate::models::Ticket;
use rusqlite::Connection;

pub fn find_by_id(conn: &Connection, id: &str) -> Result<Option<Ticket>, rusqlite::Error> {
    match conn.query_row(
        "SELECT id, ticket_number, reservation_id, qr_data, valid_from, valid_until, redeemed, redeemed_at, redeemed_by, undo_eligible_until, undone, undone_at, undone_by, undo_reason FROM tickets WHERE id = ?1",
        [id],
        |row| Ok(Ticket {
            id: row.get(0)?, ticket_number: row.get(1)?, reservation_id: row.get(2)?,
            qr_data: row.get(3)?, valid_from: row.get(4)?, valid_until: row.get(5)?,
            redeemed: row.get::<_, i64>(6)? != 0, redeemed_at: row.get(7)?, redeemed_by: row.get(8)?,
            undo_eligible_until: row.get(9)?, undone: row.get::<_, i64>(10)? != 0,
            undone_at: row.get(11)?, undone_by: row.get(12)?, undo_reason: row.get(13)?,
        }),
    ) {
        Ok(t) => Ok(Some(t)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn find_by_number(conn: &Connection, number: &str) -> Result<Option<Ticket>, rusqlite::Error> {
    match conn.query_row(
        "SELECT id, ticket_number, reservation_id, qr_data, valid_from, valid_until, redeemed, redeemed_at, redeemed_by, undo_eligible_until, undone, undone_at, undone_by, undo_reason FROM tickets WHERE ticket_number = ?1",
        [number],
        |row| Ok(Ticket {
            id: row.get(0)?, ticket_number: row.get(1)?, reservation_id: row.get(2)?,
            qr_data: row.get(3)?, valid_from: row.get(4)?, valid_until: row.get(5)?,
            redeemed: row.get::<_, i64>(6)? != 0, redeemed_at: row.get(7)?, redeemed_by: row.get(8)?,
            undo_eligible_until: row.get(9)?, undone: row.get::<_, i64>(10)? != 0,
            undone_at: row.get(11)?, undone_by: row.get(12)?, undo_reason: row.get(13)?,
        }),
    ) {
        Ok(t) => Ok(Some(t)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}
