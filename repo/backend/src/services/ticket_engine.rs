use crate::models::Ticket;
use chrono::Utc;
use rand::Rng;
use rusqlite::Connection;

pub fn generate_ticket(
    conn: &Connection,
    reservation_id: &str,
    valid_from: &str,
    valid_until: &str,
) -> Result<Ticket, String> {
    let ticket_id = uuid::Uuid::new_v4().to_string();
    let ticket_number = generate_ticket_number();

    let qr_data = serde_json::json!({
        "ticket_number": ticket_number,
        "valid_from": valid_from,
        "valid_until": valid_until,
        "reservation_id": reservation_id,
    })
    .to_string();

    conn.execute(
        "INSERT INTO tickets (id, ticket_number, reservation_id, qr_data, valid_from, valid_until) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![ticket_id, ticket_number, reservation_id, qr_data, valid_from, valid_until],
    ).map_err(|e| e.to_string())?;

    Ok(Ticket {
        id: ticket_id,
        ticket_number,
        reservation_id: reservation_id.to_string(),
        qr_data,
        valid_from: valid_from.to_string(),
        valid_until: valid_until.to_string(),
        redeemed: false,
        redeemed_at: None,
        redeemed_by: None,
        undo_eligible_until: None,
        undone: false,
        undone_at: None,
        undone_by: None,
        undo_reason: None,
    })
}

fn generate_ticket_number() -> String {
    let mut rng = rand::thread_rng();
    let chars: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".chars().collect();
    let suffix: String = (0..8).map(|_| chars[rng.gen_range(0..chars.len())]).collect();
    format!("FR-{}", suffix)
}

pub fn redeem_ticket(
    conn: &Connection,
    ticket_id: &str,
    redeemed_by: &str,
    redeemed_by_username: &str,
    hmac_key: &str,
) -> Result<String, String> {
    let (redeemed, undone, valid_from, valid_until): (bool, bool, String, String) = conn
        .query_row(
            "SELECT redeemed, undone, valid_from, valid_until FROM tickets WHERE id = ?1",
            [ticket_id],
            |row| Ok((
                row.get::<_, i64>(0)? != 0,
                row.get::<_, i64>(1)? != 0,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            )),
        )
        .map_err(|_| "Ticket not found".to_string())?;

    if redeemed && !undone {
        return Err("Ticket has already been redeemed".to_string());
    }

    // Enforce validity window: redemption must occur within valid_from..valid_until
    let now = Utc::now();
    let now_str_check = now.format("%Y-%m-%dT%H:%M:%S").to_string();
    if now_str_check < valid_from {
        return Err(format!(
            "Ticket is not yet valid. Validity starts at {}.",
            valid_from
        ));
    }
    if now_str_check > valid_until {
        return Err(format!(
            "Ticket validity has expired. It was valid until {}.",
            valid_until
        ));
    }

    let now = Utc::now();
    let now_str = now.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let undo_until = (now + chrono::Duration::minutes(2))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();

    // Pseudonym for non-FK columns and audit; raw ID used where FK is enforced.
    let pseudo_id = crate::security::masking::pseudonymize_user_id(redeemed_by, hmac_key);
    let masked_username = crate::security::masking::mask_username(redeemed_by_username);

    conn.execute(
        "UPDATE tickets SET redeemed = 1, redeemed_at = ?1, redeemed_by = ?2, undo_eligible_until = ?3, undone = 0, undone_at = NULL, undone_by = NULL, undo_reason = NULL WHERE id = ?4",
        rusqlite::params![now_str, pseudo_id, undo_until, ticket_id],
    ).map_err(|e| e.to_string())?;

    // ticket_redemptions.redeemed_by has a FK REFERENCES users(id) — must store raw.
    let redemption_id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO ticket_redemptions (id, ticket_id, redeemed_by, redeemed_at) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![redemption_id, ticket_id, redeemed_by, now_str],
    ).map_err(|e| e.to_string())?;

    // Audit log: store pseudonym so raw UUID is not at rest.
    let _ = crate::audit::chain::append_audit_log(
        conn, &pseudo_id, &masked_username, "REDEEM", "ticket", ticket_id,
        &serde_json::json!({"redeemed_at": now_str}),
    );

    Ok(now_str)
}

pub fn undo_redemption(
    conn: &Connection,
    ticket_id: &str,
    undone_by: &str,
    undone_by_username: &str,
    reason: &str,
    hmac_key: &str,
) -> Result<(), String> {
    if reason.trim().is_empty() {
        return Err("Undo reason is required and must not be empty".to_string());
    }

    let (redeemed, undone, undo_eligible_until): (bool, bool, Option<String>) = conn
        .query_row(
            "SELECT redeemed, undone, undo_eligible_until FROM tickets WHERE id = ?1",
            [ticket_id],
            |row| Ok((
                row.get::<_, i64>(0)? != 0,
                row.get::<_, i64>(1)? != 0,
                row.get(2)?,
            )),
        )
        .map_err(|_| "Ticket not found".to_string())?;

    if !redeemed {
        return Err("Ticket has not been redeemed".to_string());
    }
    if undone {
        return Err("Redemption has already been undone".to_string());
    }

    // Check 2-minute window
    let now = Utc::now();
    if let Some(until_str) = undo_eligible_until {
        if let Ok(until) = chrono::DateTime::parse_from_rfc3339(&until_str) {
            if now > until {
                return Err("Undo window has expired (2-minute limit)".to_string());
            }
        } else if let Ok(until) = chrono::NaiveDateTime::parse_from_str(&until_str, "%Y-%m-%dT%H:%M:%SZ") {
            let until_utc = until.and_utc();
            if now > until_utc {
                return Err("Undo window has expired (2-minute limit)".to_string());
            }
        }
    } else {
        return Err("Undo window information not available".to_string());
    }

    let now_str = now.format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let pseudo_id = crate::security::masking::pseudonymize_user_id(undone_by, hmac_key);
    let masked_username = crate::security::masking::mask_username(undone_by_username);

    // tickets.undone_by has no FK — store pseudonym.
    conn.execute(
        "UPDATE tickets SET undone = 1, undone_at = ?1, undone_by = ?2, undo_reason = ?3 WHERE id = ?4",
        rusqlite::params![now_str, pseudo_id, reason, ticket_id],
    ).map_err(|e| e.to_string())?;

    // ticket_redemptions.undone_by also has no FK — store pseudonym.
    conn.execute(
        "UPDATE ticket_redemptions SET undone = 1, undone_at = ?1, undone_by = ?2, undo_reason = ?3 \
         WHERE id = (SELECT id FROM ticket_redemptions WHERE ticket_id = ?4 AND undone = 0 ORDER BY redeemed_at DESC LIMIT 1)",
        rusqlite::params![now_str, pseudo_id, reason, ticket_id],
    ).map_err(|e| e.to_string())?;

    // Audit log: store pseudonym.
    let _ = crate::audit::chain::append_audit_log(
        conn, &pseudo_id, &masked_username, "UNDO", "ticket", ticket_id,
        &serde_json::json!({"reason": reason, "undone_at": now_str}),
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(include_str!("../../migrations/001_initial_schema.sql")).unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        conn
            .execute(
                "INSERT INTO stores (id, name, location) VALUES ('s1', 'Test Store', 'Here')",
                [],
            )
            .unwrap();
        conn
            .execute(
                "INSERT INTO users (id, username, password_hash, display_name, role) VALUES ('u1', 'u1', 'x', 'User One', 'Customer')",
                [],
            )
            .unwrap();
        conn
            .execute(
                "INSERT INTO users (id, username, password_hash, display_name, role) VALUES ('staff-1', 'staff1', 'x', 'Staff One', 'MerchantStaff')",
                [],
            )
            .unwrap();
        conn
    }

    /// Window that always contains `Utc::now()` for redemption tests (naive UTC compared in engine).
    const VALID_FROM: &str = "2000-01-01T00:00:00";
    const VALID_UNTIL: &str = "2099-12-31T23:59:59";

    #[test]
    fn test_ticket_number_format() {
        let num = generate_ticket_number();
        assert!(num.starts_with("FR-"));
        assert_eq!(num.len(), 11); // "FR-" + 8 chars
    }

    #[test]
    fn test_generate_ticket() {
        let conn = setup_test_db();
        conn.execute(
            "INSERT INTO reservations (id, asset_type, asset_id, store_id, user_id, start_time, end_time, status) VALUES ('r1', 'vehicle', 'v1', 's1', 'u1', '2026-04-10T09:00:00', '2026-04-10T10:00:00', 'confirmed')",
            [],
        ).unwrap();

        let ticket = generate_ticket(&conn, "r1", VALID_FROM, VALID_UNTIL).unwrap();
        assert!(ticket.ticket_number.starts_with("FR-"));
        assert!(!ticket.qr_data.is_empty());
        assert_eq!(ticket.valid_from, VALID_FROM);
    }

    #[test]
    fn test_redeem_ticket() {
        let conn = setup_test_db();
        conn.execute(
            "INSERT INTO reservations (id, asset_type, asset_id, store_id, user_id, start_time, end_time, status) VALUES ('r1', 'vehicle', 'v1', 's1', 'u1', '2026-04-10T09:00:00', '2026-04-10T10:00:00', 'confirmed')",
            [],
        ).unwrap();
        let ticket = generate_ticket(&conn, "r1", VALID_FROM, VALID_UNTIL).unwrap();
        let result = redeem_ticket(&conn, &ticket.id, "staff-1", "staff", "");
        assert!(result.is_ok());
    }

    #[test]
    fn test_double_redemption_blocked() {
        let conn = setup_test_db();
        conn.execute(
            "INSERT INTO reservations (id, asset_type, asset_id, store_id, user_id, start_time, end_time, status) VALUES ('r1', 'vehicle', 'v1', 's1', 'u1', '2026-04-10T09:00:00', '2026-04-10T10:00:00', 'confirmed')",
            [],
        ).unwrap();
        let ticket = generate_ticket(&conn, "r1", VALID_FROM, VALID_UNTIL).unwrap();
        let _ = redeem_ticket(&conn, &ticket.id, "staff-1", "staff", "");
        let result = redeem_ticket(&conn, &ticket.id, "staff-1", "staff", "");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already been redeemed"));
    }

    #[test]
    fn test_undo_within_window() {
        let conn = setup_test_db();
        conn.execute(
            "INSERT INTO reservations (id, asset_type, asset_id, store_id, user_id, start_time, end_time, status) VALUES ('r1', 'vehicle', 'v1', 's1', 'u1', '2026-04-10T09:00:00', '2026-04-10T10:00:00', 'confirmed')",
            [],
        ).unwrap();
        let ticket = generate_ticket(&conn, "r1", VALID_FROM, VALID_UNTIL).unwrap();
        let _ = redeem_ticket(&conn, &ticket.id, "staff-1", "staff", "");
        let result = undo_redemption(&conn, &ticket.id, "staff-1", "staff", "Scanned wrong ticket", "");
        assert!(result.is_ok());
    }

    #[test]
    fn test_undo_without_reason_rejected() {
        let conn = setup_test_db();
        conn.execute(
            "INSERT INTO reservations (id, asset_type, asset_id, store_id, user_id, start_time, end_time, status) VALUES ('r1', 'vehicle', 'v1', 's1', 'u1', '2026-04-10T09:00:00', '2026-04-10T10:00:00', 'confirmed')",
            [],
        ).unwrap();
        let ticket = generate_ticket(&conn, "r1", VALID_FROM, VALID_UNTIL).unwrap();
        let _ = redeem_ticket(&conn, &ticket.id, "staff-1", "staff", "");
        let result = undo_redemption(&conn, &ticket.id, "staff-1", "staff", "", "");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("required"));
    }

    #[test]
    fn test_undo_after_window_rejected() {
        let conn = setup_test_db();
        conn.execute(
            "INSERT INTO reservations (id, asset_type, asset_id, store_id, user_id, start_time, end_time, status) VALUES ('r1', 'vehicle', 'v1', 's1', 'u1', '2026-04-10T09:00:00', '2026-04-10T10:00:00', 'confirmed')",
            [],
        ).unwrap();
        let ticket = generate_ticket(&conn, "r1", VALID_FROM, VALID_UNTIL).unwrap();
        let _ = redeem_ticket(&conn, &ticket.id, "staff-1", "staff", "");

        // Manually set undo_eligible_until to past
        conn.execute(
            "UPDATE tickets SET undo_eligible_until = '2020-01-01T00:00:00Z' WHERE id = ?1",
            [&ticket.id],
        ).unwrap();

        let result = undo_redemption(&conn, &ticket.id, "staff-1", "staff", "Too late", "");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("expired"));
    }
}
