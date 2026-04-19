use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use crate::app::state::AppState;
use crate::errors::AppError;
use crate::handlers::auth::*;
use crate::models::*;
use sha2::{Sha256, Digest};

pub async fn create_backup(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<BackupRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let claims = extract_claims_required(&headers, &state.hmac_secret)?;
    require_role(&claims, &UserRole::Administrator)?;
    require_csrf_with_state(&headers, &claims, &state)?;
    require_permission_with_state(&state, &claims, "backup", "create")?;

    // Validate destination path directory exists
    let dest_dir = std::path::Path::new(&req.path);
    if !dest_dir.exists() {
        std::fs::create_dir_all(dest_dir).map_err(|e| {
            AppError::Validation(format!("Cannot create backup directory: {}", e))
        })?;
    }

    let db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    let backup_id = uuid::Uuid::new_v4().to_string();
    let filename = format!("fleetreserve-backup-{}.enc", chrono::Utc::now().format("%Y%m%d%H%M%S"));
    let backup_path = format!("{}/{}", req.path.trim_end_matches('/'), filename);

    // Step 1: SQLite backup API -> temp file
    let temp_path = format!("/tmp/fleetreserve-backup-{}.db", backup_id);
    {
        let mut dest_conn = rusqlite::Connection::open(&temp_path)
            .map_err(|e| AppError::Internal(format!("Failed to open temp backup: {}", e)))?;
        let backup = rusqlite::backup::Backup::new(&db, &mut dest_conn)
            .map_err(|e| AppError::Internal(format!("Failed to init backup: {}", e)))?;
        backup
            .run_to_completion(100, std::time::Duration::from_millis(50), None)
            .map_err(|e| AppError::Internal(format!("Backup failed: {}", e)))?;
    }

    // Step 2: Read temp file, encrypt with AES-256-GCM, write to destination
    let plaintext_bytes = std::fs::read(&temp_path)
        .map_err(|e| AppError::Internal(format!("Failed to read temp backup: {}", e)))?;

    // Clean up temp file immediately
    let _ = std::fs::remove_file(&temp_path);

    let encrypted_bytes = crate::security::encryption::encrypt_bytes(&plaintext_bytes, &state.encryption_key)
        .map_err(|e| AppError::Internal(format!("Encryption failed: {}", e)))?;

    std::fs::write(&backup_path, &encrypted_bytes)
        .map_err(|e| AppError::Internal(format!("Failed to write backup: {}", e)))?;

    // Step 3: Compute SHA-256 of encrypted file for integrity
    let mut hasher = Sha256::new();
    hasher.update(&encrypted_bytes);
    let sha256 = hex::encode(hasher.finalize());
    let size_bytes = encrypted_bytes.len() as i64;

    // Step 4: Record metadata and audit
    crate::repositories::backups::create(
        &db, &backup_id, &filename, &backup_path, size_bytes, &sha256, &claims.user_id,
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let _ = crate::audit::chain::append_audit_log_secure(
        &db,
        &claims.user_id,
        &claims.username,
        "BACKUP",
        "backup",
        &backup_id,
        &serde_json::json!({
            "filename": filename,
            "size_bytes": size_bytes,
            "sha256": sha256,
        }),
        &state.hmac_secret,
    );

    Ok(Json(serde_json::json!({
        "id": backup_id,
        "filename": filename,
        "path": backup_path,
        "size_bytes": size_bytes,
        "sha256": sha256,
        "message": "Backup created successfully"
    })))
}

pub async fn restore_backup(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<BackupRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let claims = extract_claims_required(&headers, &state.hmac_secret)?;
    require_role(&claims, &UserRole::Administrator)?;
    require_csrf_with_state(&headers, &claims, &state)?;
    require_permission_with_state(&state, &claims, "backup", "restore")?;

    // Verify backup file exists
    let backup_file = std::path::Path::new(&req.path);
    if !backup_file.exists() || !backup_file.is_file() {
        return Err(AppError::Validation("Backup file not found".to_string()));
    }

    // Step 1: Read encrypted backup
    let encrypted_bytes = std::fs::read(&req.path)
        .map_err(|e| AppError::Internal(format!("Failed to read backup file: {}", e)))?;

    // Step 2: Decrypt
    let plaintext_bytes =
        crate::security::encryption::decrypt_bytes(&encrypted_bytes, &state.encryption_key)
            .map_err(|e| AppError::Validation(format!("Decryption failed (wrong key or corrupt backup): {}", e)))?;

    // Step 3: Validate it is a valid SQLite database (magic bytes)
    if plaintext_bytes.len() < 16 || &plaintext_bytes[..16] != b"SQLite format 3\0" {
        return Err(AppError::Validation(
            "Decrypted content is not a valid SQLite database".to_string(),
        ));
    }

    // Step 4: Write to temp file, open as SQLite to validate
    let temp_path = format!("/tmp/fleetreserve-restore-{}.db", uuid::Uuid::new_v4());
    std::fs::write(&temp_path, &plaintext_bytes)
        .map_err(|e| AppError::Internal(format!("Failed to write temp restore: {}", e)))?;

    let source_conn = rusqlite::Connection::open(&temp_path)
        .map_err(|e| {
            let _ = std::fs::remove_file(&temp_path);
            AppError::Validation(format!("Backup contains invalid database: {}", e))
        })?;

    // Verify the restored DB has expected tables
    let table_count: i64 = source_conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('users','vehicles','reservations','tickets','audit_log')",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if table_count < 5 {
        let _ = std::fs::remove_file(&temp_path);
        return Err(AppError::Validation(
            "Backup database is missing required tables".to_string(),
        ));
    }

    // Step 5: Use SQLite backup API to restore into the live database
    let mut db = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    {
        let restore = rusqlite::backup::Backup::new(&source_conn, &mut db)
            .map_err(|e| AppError::Internal(format!("Failed to init restore: {}", e)))?;
        restore
            .run_to_completion(100, std::time::Duration::from_millis(50), None)
            .map_err(|e| AppError::Internal(format!("Restore failed: {}", e)))?;
    }

    // Clean up
    let _ = std::fs::remove_file(&temp_path);

    // Step 6: Audit log the restore in the newly restored database
    let _ = crate::audit::chain::append_audit_log_secure(
        &db,
        &claims.user_id,
        &claims.username,
        "RESTORE",
        "backup",
        "",
        &serde_json::json!({
            "source_path": req.path,
            "restored_by": claims.user_id,
        }),
        &state.hmac_secret,
    );

    Ok(Json(serde_json::json!({"message": "Restore completed successfully"})))
}
