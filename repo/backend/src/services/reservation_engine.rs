use crate::models::*;
use chrono::NaiveDateTime;
use rusqlite::Connection;

const MAX_RETRIES: u32 = 3;

#[derive(Debug)]
pub struct ReservationResult {
    pub reservation: Reservation,
    pub ticket: Ticket,
}

pub fn create_reservation(
    conn: &Connection,
    user_id: &str,
    username: &str,
    req: &CreateReservationRequest,
    hmac_key: &str,
) -> Result<ReservationResult, ConflictResponse> {
    let start = NaiveDateTime::parse_from_str(&req.start_time, "%Y-%m-%dT%H:%M:%S")
        .or_else(|_| NaiveDateTime::parse_from_str(&req.start_time, "%Y-%m-%dT%H:%M"))
        .map_err(|_| make_validation_conflict("Invalid start_time format (expected YYYY-MM-DDTHH:MM:SS or YYYY-MM-DDTHH:MM)"))?;
    let end = NaiveDateTime::parse_from_str(&req.end_time, "%Y-%m-%dT%H:%M:%S")
        .or_else(|_| NaiveDateTime::parse_from_str(&req.end_time, "%Y-%m-%dT%H:%M"))
        .map_err(|_| make_validation_conflict("Invalid end_time format (expected YYYY-MM-DDTHH:MM:SS or YYYY-MM-DDTHH:MM)"))?;

    if end <= start {
        return Err(make_validation_conflict("end_time must be after start_time"));
    }

    // Validate within business hours
    let store_start: String = conn
        .query_row(
            "SELECT business_hours_start FROM stores WHERE id = ?1",
            [&req.store_id],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "07:00".to_string());
    let store_end: String = conn
        .query_row(
            "SELECT business_hours_end FROM stores WHERE id = ?1",
            [&req.store_id],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "19:00".to_string());

    let start_time_str = start.format("%H:%M").to_string();
    let end_time_str = end.format("%H:%M").to_string();
    if start_time_str < store_start || end_time_str > store_end {
        return Err(make_validation_conflict("Reservation must be within business hours"));
    }

    for attempt in 0..MAX_RETRIES {
        match try_create_reservation(conn, user_id, username, req, attempt, hmac_key) {
            Ok(result) => return Ok(result),
            Err(ConflictType::VersionConflict) if attempt < MAX_RETRIES - 1 => {
                tracing::warn!("Reservation attempt {} failed with version conflict, retrying", attempt + 1);
                continue;
            }
            Err(ConflictType::VersionConflict) => {
                tracing::error!("Max retries exceeded for reservation");
                return Err(compute_conflict_response(conn, req));
            }
            Err(ConflictType::BusinessConflict(resp)) => return Err(resp),
        }
    }

    Err(compute_conflict_response(conn, req))
}

enum ConflictType {
    VersionConflict,
    BusinessConflict(ConflictResponse),
}

fn try_create_reservation(
    conn: &Connection,
    user_id: &str,
    username: &str,
    req: &CreateReservationRequest,
    _attempt: u32,
    hmac_key: &str,
) -> Result<ReservationResult, ConflictType> {
    conn.execute("BEGIN IMMEDIATE", []).map_err(|_| ConflictType::VersionConflict)?;

    let mut reasons: Vec<ConflictReason> = Vec::new();

    if req.asset_type == "vehicle" {
        // Check vehicle status and version
        let vehicle_row: Result<(String, String, Option<String>, i64), _> = conn.query_row(
            "SELECT status, store_id, insurance_expiry, version FROM vehicles WHERE id = ?1",
            [&req.asset_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        );

        match vehicle_row {
            Ok((status, _store_id, insurance_expiry, version)) => {
                if status == "in-repair" {
                    reasons.push(ConflictReason::InRepairHold);
                }
                if status == "decommissioned" {
                    let _ = conn.execute("ROLLBACK", []);
                    return Err(ConflictType::BusinessConflict(make_validation_conflict(
                        "Vehicle is decommissioned and cannot be reserved",
                    )));
                }

                if let Some(expiry) = insurance_expiry {
                    if expiry < req.end_time {
                        reasons.push(ConflictReason::ExpiredInsurance {
                            expiry_date: expiry,
                        });
                    }
                }

                // Optimistic concurrency: attempt version update
                let updated = conn.execute(
                    "UPDATE vehicles SET version = version + 1 WHERE id = ?1 AND version = ?2",
                    rusqlite::params![req.asset_id, version],
                ).unwrap_or(0);

                if updated == 0 {
                    let _ = conn.execute("ROLLBACK", []);
                    return Err(ConflictType::VersionConflict);
                }
            }
            Err(_) => {
                let _ = conn.execute("ROLLBACK", []);
                return Err(ConflictType::BusinessConflict(make_validation_conflict(
                    "Vehicle not found",
                )));
            }
        }
    } else if req.asset_type == "bay" {
        // Check bay capacity
        let bay_row: Result<(i64, i64), _> = conn.query_row(
            "SELECT capacity, version FROM service_bays WHERE id = ?1",
            [&req.asset_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        );

        match bay_row {
            Ok((capacity, version)) => {
                let overlapping_count: i64 = conn
                    .query_row(
                        "SELECT COUNT(*) FROM reservations WHERE asset_type = 'bay' AND asset_id = ?1 AND status = 'confirmed' AND start_time < ?3 AND end_time > ?2",
                        rusqlite::params![req.asset_id, req.start_time, req.end_time],
                        |row| row.get(0),
                    )
                    .unwrap_or(0);

                if overlapping_count >= capacity {
                    reasons.push(ConflictReason::CapacityExceeded {
                        current: overlapping_count,
                        max: capacity,
                    });
                }

                let updated = conn.execute(
                    "UPDATE service_bays SET version = version + 1 WHERE id = ?1 AND version = ?2",
                    rusqlite::params![req.asset_id, version],
                ).unwrap_or(0);

                if updated == 0 {
                    let _ = conn.execute("ROLLBACK", []);
                    return Err(ConflictType::VersionConflict);
                }
            }
            Err(_) => {
                let _ = conn.execute("ROLLBACK", []);
                return Err(ConflictType::BusinessConflict(make_validation_conflict(
                    "Service bay not found",
                )));
            }
        }
    }

    // Check overlapping reservations
    let overlapping: Vec<(String, String)> = {
        let mut stmt = conn.prepare(
            "SELECT start_time, end_time FROM reservations WHERE asset_type = ?1 AND asset_id = ?2 AND status = 'confirmed' AND start_time < ?4 AND end_time > ?3"
        ).unwrap();
        stmt.query_map(
            rusqlite::params![req.asset_type, req.asset_id, req.start_time, req.end_time],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    };

    for (existing_start, existing_end) in &overlapping {
        reasons.push(ConflictReason::OverlappingReservation {
            existing_start: existing_start.clone(),
            existing_end: existing_end.clone(),
        });
    }

    if !reasons.is_empty() {
        let _ = conn.execute("ROLLBACK", []);
        return Err(ConflictType::BusinessConflict(
            compute_conflict_response(conn, req),
        ));
    }

    // Create reservation
    let reservation_id = uuid::Uuid::new_v4().to_string();
    let user_pseudonym = crate::security::masking::pseudonymize_user_id(user_id, hmac_key);
    conn.execute(
        "INSERT INTO reservations (id, asset_type, asset_id, store_id, user_id, user_id_pseudonym, start_time, end_time, status) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'confirmed')",
        rusqlite::params![reservation_id, req.asset_type, req.asset_id, req.store_id, user_id, user_pseudonym, req.start_time, req.end_time],
    ).map_err(|_| {
        let _ = conn.execute("ROLLBACK", []);
        ConflictType::VersionConflict
    })?;

    // Generate ticket
    let ticket = crate::services::ticket_engine::generate_ticket(
        conn,
        &reservation_id,
        &req.start_time,
        &req.end_time,
    ).map_err(|_| {
        let _ = conn.execute("ROLLBACK", []);
        ConflictType::VersionConflict
    })?;

    // Update reservation with ticket_id
    conn.execute(
        "UPDATE reservations SET ticket_id = ?1 WHERE id = ?2",
        rusqlite::params![ticket.id, reservation_id],
    ).map_err(|_| {
        let _ = conn.execute("ROLLBACK", []);
        ConflictType::VersionConflict
    })?;

    conn.execute("COMMIT", []).map_err(|_| ConflictType::VersionConflict)?;

    // Audit log: pseudonymize actor so raw user UUID is not stored at rest.
    let _ = crate::audit::chain::append_audit_log_secure(
        conn, user_id, username, "CREATE", "reservation", &reservation_id,
        &serde_json::json!({
            "asset_type": req.asset_type,
            "asset_id": req.asset_id,
            "start_time": req.start_time,
            "end_time": req.end_time,
            "ticket_number": ticket.ticket_number,
        }),
        hmac_key,
    );

    let reservation = Reservation {
        id: reservation_id,
        asset_type: req.asset_type.clone(),
        asset_id: req.asset_id.clone(),
        store_id: req.store_id.clone(),
        user_id: user_id.to_string(),
        start_time: req.start_time.clone(),
        end_time: req.end_time.clone(),
        status: "confirmed".to_string(),
        ticket_id: Some(ticket.id.clone()),
        version: 1,
    };

    Ok(ReservationResult { reservation, ticket })
}

fn compute_conflict_response(conn: &Connection, req: &CreateReservationRequest) -> ConflictResponse {
    let mut reasons: Vec<ConflictReasonDisplay> = Vec::new();

    // Re-check conflicts from current committed snapshot
    if req.asset_type == "vehicle" {
        if let Ok((status, insurance_expiry)) = conn.query_row(
            "SELECT status, insurance_expiry FROM vehicles WHERE id = ?1",
            [&req.asset_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?)),
        ) {
            if status == "in-repair" {
                let r = ConflictReason::InRepairHold;
                reasons.push(ConflictReasonDisplay {
                    code: r.code().to_string(),
                    message: r.to_message(),
                });
            }
            if let Some(expiry) = insurance_expiry {
                if expiry < req.end_time {
                    let r = ConflictReason::ExpiredInsurance { expiry_date: expiry };
                    reasons.push(ConflictReasonDisplay {
                        code: r.code().to_string(),
                        message: r.to_message(),
                    });
                }
            }
        }
    }

    if req.asset_type == "bay" {
        if let Ok((capacity,)) = conn.query_row(
            "SELECT capacity FROM service_bays WHERE id = ?1",
            [&req.asset_id],
            |row| Ok((row.get::<_, i64>(0)?,)),
        ) {
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM reservations WHERE asset_type = 'bay' AND asset_id = ?1 AND status = 'confirmed' AND start_time < ?3 AND end_time > ?2",
                    rusqlite::params![req.asset_id, req.start_time, req.end_time],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            if count >= capacity {
                let r = ConflictReason::CapacityExceeded { current: count, max: capacity };
                reasons.push(ConflictReasonDisplay {
                    code: r.code().to_string(),
                    message: r.to_message(),
                });
            }
        }
    }

    // Check overlapping
    let mut stmt = conn.prepare(
        "SELECT start_time, end_time FROM reservations WHERE asset_type = ?1 AND asset_id = ?2 AND status = 'confirmed' AND start_time < ?4 AND end_time > ?3"
    ).unwrap();
    let overlapping: Vec<(String, String)> = stmt
        .query_map(
            rusqlite::params![req.asset_type, req.asset_id, req.start_time, req.end_time],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    for (s, e) in overlapping {
        let r = ConflictReason::OverlappingReservation { existing_start: s, existing_end: e };
        reasons.push(ConflictReasonDisplay {
            code: r.code().to_string(),
            message: r.to_message(),
        });
    }

    // Find alternative slots
    let alternative_slots = find_nearest_alternative_slots(conn, req);
    let alternate_assets = find_alternate_assets(conn, req);

    ConflictResponse {
        conflict: true,
        reasons,
        alternative_slots,
        alternate_assets,
    }
}

fn find_nearest_alternative_slots(
    conn: &Connection,
    req: &CreateReservationRequest,
) -> Vec<AlternativeSlot> {
    let duration_minutes = {
        let start = NaiveDateTime::parse_from_str(&req.start_time, "%Y-%m-%dT%H:%M:%S");
        let end = NaiveDateTime::parse_from_str(&req.end_time, "%Y-%m-%dT%H:%M:%S");
        match (start, end) {
            (Ok(s), Ok(e)) => (e - s).num_minutes(),
            _ => 60,
        }
    };

    let store_start: String = conn
        .query_row("SELECT business_hours_start FROM stores WHERE id = ?1", [&req.store_id], |r| r.get(0))
        .unwrap_or_else(|_| "07:00".to_string());
    let store_end: String = conn
        .query_row("SELECT business_hours_end FROM stores WHERE id = ?1", [&req.store_id], |r| r.get(0))
        .unwrap_or_else(|_| "19:00".to_string());

    let date = &req.start_time[..10]; // YYYY-MM-DD
    let day_start = format!("{}T{}:00", date, store_start);
    let day_end = format!("{}T{}:00", date, store_end);

    let existing: Vec<(String, String)> = {
        let mut stmt = conn.prepare(
            "SELECT start_time, end_time FROM reservations WHERE asset_type = ?1 AND asset_id = ?2 AND status = 'confirmed' AND start_time >= ?3 AND end_time <= ?4 ORDER BY start_time"
        ).unwrap();
        stmt.query_map(
            rusqlite::params![req.asset_type, req.asset_id, day_start, day_end],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).unwrap().filter_map(|r| r.ok()).collect()
    };

    let mut slots = Vec::new();
    let mut cursor = NaiveDateTime::parse_from_str(&day_start, "%Y-%m-%dT%H:%M:%S").unwrap();
    let end_of_day = NaiveDateTime::parse_from_str(&day_end, "%Y-%m-%dT%H:%M:%S").unwrap();
    let requested_start = NaiveDateTime::parse_from_str(&req.start_time, "%Y-%m-%dT%H:%M:%S").unwrap_or(cursor);

    while cursor + chrono::Duration::minutes(duration_minutes) <= end_of_day {
        let slot_end = cursor + chrono::Duration::minutes(duration_minutes);
        let slot_start_str = cursor.format("%Y-%m-%dT%H:%M:%S").to_string();
        let slot_end_str = slot_end.format("%Y-%m-%dT%H:%M:%S").to_string();

        let conflicts_with_existing = existing.iter().any(|(es, ee)| {
            slot_start_str < *ee && slot_end_str > *es
        });

        if !conflicts_with_existing && slot_start_str != req.start_time {
            let distance = (cursor - requested_start).num_minutes().unsigned_abs();
            slots.push((distance, cursor >= requested_start, AlternativeSlot {
                start_time: slot_start_str,
                end_time: slot_end_str,
            }));
        }

        cursor += chrono::Duration::minutes(15);
    }

    // Sort: prefer future, then by distance
    slots.sort_by(|a, b| {
        b.1.cmp(&a.1) // future first
            .then(a.0.cmp(&b.0)) // then nearest
    });

    slots.into_iter().take(2).map(|(_, _, s)| s).collect()
}

fn find_alternate_assets(conn: &Connection, req: &CreateReservationRequest) -> Vec<AlternateAsset> {
    if req.asset_type == "vehicle" {
        let mut stmt = conn.prepare(
            "SELECT v.id, v.make, v.model, v.status FROM vehicles v WHERE v.store_id = ?1 AND v.id != ?2 AND v.status = 'available' AND v.id NOT IN (SELECT asset_id FROM reservations WHERE asset_type = 'vehicle' AND status = 'confirmed' AND start_time < ?4 AND end_time > ?3) LIMIT 3"
        ).unwrap();
        stmt.query_map(
            rusqlite::params![req.store_id, req.asset_id, req.start_time, req.end_time],
            |row| {
                let make: String = row.get(1)?;
                let model: String = row.get(2)?;
                Ok(AlternateAsset {
                    id: row.get(0)?,
                    asset_type: "vehicle".to_string(),
                    name: format!("{} {}", make, model),
                    status: row.get(3)?,
                })
            },
        ).unwrap().filter_map(|r| r.ok()).collect()
    } else {
        let mut stmt = conn.prepare(
            "SELECT b.id, b.name, b.status FROM service_bays b WHERE b.store_id = ?1 AND b.id != ?2 AND b.status = 'active' LIMIT 3"
        ).unwrap();
        stmt.query_map(
            rusqlite::params![req.store_id, req.asset_id],
            |row| Ok(AlternateAsset {
                id: row.get(0)?,
                asset_type: "bay".to_string(),
                name: row.get(1)?,
                status: row.get(2)?,
            }),
        ).unwrap().filter_map(|r| r.ok()).collect()
    }
}

fn make_validation_conflict(msg: &str) -> ConflictResponse {
    ConflictResponse {
        conflict: true,
        reasons: vec![ConflictReasonDisplay {
            code: "validation".to_string(),
            message: msg.to_string(),
        }],
        alternative_slots: vec![],
        alternate_assets: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(include_str!("../../migrations/001_initial_schema.sql")).unwrap();
        conn.execute_batch(include_str!("../../migrations/002_seed_data.sql")).unwrap();

        conn.execute(
            "INSERT INTO vehicles (id, vin_encrypted, vin_hash, license_plate_encrypted, license_plate_hash, make, model, store_id, status, insurance_expiry, version) VALUES ('v1', 'enc', 'hash', 'enc', 'hash', 'Toyota', 'Camry', 'store-001', 'available', '2027-12-31T23:59:59', 1)",
            [],
        ).unwrap();

        conn.execute(
            "INSERT INTO vehicles (id, vin_encrypted, vin_hash, license_plate_encrypted, license_plate_hash, make, model, store_id, status, version) VALUES ('v2', 'enc2', 'hash2', 'enc2', 'hash2', 'Honda', 'Civic', 'store-001', 'available', 1)",
            [],
        ).unwrap();

        conn.execute(
            "INSERT INTO vehicles (id, vin_encrypted, vin_hash, license_plate_encrypted, license_plate_hash, make, model, store_id, status, version) VALUES ('v-repair', 'enc3', 'hash3', 'enc3', 'hash3', 'Ford', 'F150', 'store-001', 'in-repair', 1)",
            [],
        ).unwrap();

        conn.execute(
            "INSERT INTO service_bays (id, store_id, name, bay_type, capacity, status, version) VALUES ('bay1', 'store-001', 'Bay A', 'general', 1, 'active', 1)",
            [],
        ).unwrap();

        conn
    }

    #[test]
    fn test_reservation_happy_path() {
        let conn = setup_test_db();
        let req = CreateReservationRequest {
            asset_type: "vehicle".into(),
            asset_id: "v1".into(),
            store_id: "store-001".into(),
            start_time: "2026-04-10T09:00:00".into(),
            end_time: "2026-04-10T10:00:00".into(),
        };
        let result = create_reservation(&conn, "user-admin-001", "admin", &req, "");
        assert!(result.is_ok());
        let r = result.unwrap();
        assert_eq!(r.reservation.status, "confirmed");
        assert!(r.ticket.ticket_number.starts_with("FR-"));
    }

    #[test]
    fn test_overlapping_reservation_conflict() {
        let conn = setup_test_db();
        let req = CreateReservationRequest {
            asset_type: "vehicle".into(),
            asset_id: "v1".into(),
            store_id: "store-001".into(),
            start_time: "2026-04-10T09:00:00".into(),
            end_time: "2026-04-10T10:00:00".into(),
        };
        let _ = create_reservation(&conn, "user-admin-001", "admin", &req, "");

        // Reset version for second attempt
        conn.execute("UPDATE vehicles SET version = version WHERE id = 'v1'", []).unwrap();

        let req2 = CreateReservationRequest {
            asset_type: "vehicle".into(),
            asset_id: "v1".into(),
            store_id: "store-001".into(),
            start_time: "2026-04-10T09:30:00".into(),
            end_time: "2026-04-10T10:30:00".into(),
        };
        let result = create_reservation(&conn, "user-admin-001", "admin", &req2, "");
        assert!(result.is_err());
        let conflict = result.unwrap_err();
        assert!(conflict.conflict);
        assert!(!conflict.reasons.is_empty());
        assert!(conflict.reasons.iter().any(|r| r.code == "overlapping_reservation"));
    }

    #[test]
    fn test_in_repair_conflict() {
        let conn = setup_test_db();
        let req = CreateReservationRequest {
            asset_type: "vehicle".into(),
            asset_id: "v-repair".into(),
            store_id: "store-001".into(),
            start_time: "2026-04-10T09:00:00".into(),
            end_time: "2026-04-10T10:00:00".into(),
        };
        let result = create_reservation(&conn, "user-admin-001", "admin", &req, "");
        assert!(result.is_err());
        let conflict = result.unwrap_err();
        assert!(conflict.reasons.iter().any(|r| r.code == "in_repair_hold"));
    }

    #[test]
    fn test_alternative_slots_returned() {
        let conn = setup_test_db();
        let req = CreateReservationRequest {
            asset_type: "vehicle".into(),
            asset_id: "v1".into(),
            store_id: "store-001".into(),
            start_time: "2026-04-10T09:00:00".into(),
            end_time: "2026-04-10T10:00:00".into(),
        };
        let _ = create_reservation(&conn, "user-admin-001", "admin", &req, "");

        let req2 = CreateReservationRequest {
            asset_type: "vehicle".into(),
            asset_id: "v1".into(),
            store_id: "store-001".into(),
            start_time: "2026-04-10T09:30:00".into(),
            end_time: "2026-04-10T10:30:00".into(),
        };
        let conflict = create_reservation(&conn, "user-admin-001", "admin", &req2, "").unwrap_err();
        assert!(!conflict.alternative_slots.is_empty());
        assert!(conflict.alternative_slots.len() <= 2);
    }

    #[test]
    fn test_alternate_vehicle_suggested() {
        let conn = setup_test_db();
        let req = CreateReservationRequest {
            asset_type: "vehicle".into(),
            asset_id: "v-repair".into(),
            store_id: "store-001".into(),
            start_time: "2026-04-10T09:00:00".into(),
            end_time: "2026-04-10T10:00:00".into(),
        };
        let conflict = create_reservation(&conn, "user-admin-001", "admin", &req, "").unwrap_err();
        assert!(!conflict.alternate_assets.is_empty());
    }
}
