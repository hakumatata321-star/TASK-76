-- FleetReserve Operations Suite - Initial Schema

CREATE TABLE IF NOT EXISTS stores (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    location TEXT NOT NULL,
    business_hours_start TEXT NOT NULL DEFAULT '07:00',
    business_hours_end TEXT NOT NULL DEFAULT '19:00',
    active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    display_name TEXT NOT NULL,
    email_encrypted TEXT,
    email_hash TEXT,
    role TEXT NOT NULL CHECK(role IN ('Customer','Photographer','MerchantStaff','PlatformOps','Administrator')),
    store_id TEXT REFERENCES stores(id),
    active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS permissions (
    id TEXT PRIMARY KEY,
    role TEXT NOT NULL,
    resource TEXT NOT NULL,
    action TEXT NOT NULL,
    UNIQUE(role, resource, action)
);

CREATE TABLE IF NOT EXISTS photographer_assignments (
    id TEXT PRIMARY KEY,
    photographer_user_id TEXT NOT NULL REFERENCES users(id),
    store_id TEXT NOT NULL REFERENCES stores(id),
    job_description TEXT NOT NULL,
    vehicle_id TEXT,
    bay_id TEXT,
    start_time TEXT NOT NULL,
    end_time TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS vehicles (
    id TEXT PRIMARY KEY,
    vin_encrypted TEXT NOT NULL,
    vin_hash TEXT NOT NULL,
    license_plate_encrypted TEXT NOT NULL,
    license_plate_hash TEXT NOT NULL,
    make TEXT NOT NULL,
    model TEXT NOT NULL,
    trim_level TEXT NOT NULL DEFAULT '',
    store_id TEXT NOT NULL REFERENCES stores(id),
    mileage_miles INTEGER NOT NULL DEFAULT 0,
    fuel_or_battery_pct REAL NOT NULL DEFAULT 100.0,
    status TEXT NOT NULL DEFAULT 'available' CHECK(status IN ('available','reserved','on-rent','in-repair','decommissioned')),
    maintenance_due TEXT,
    inspection_due TEXT,
    insurance_expiry TEXT,
    version INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS service_bays (
    id TEXT PRIMARY KEY,
    store_id TEXT NOT NULL REFERENCES stores(id),
    name TEXT NOT NULL,
    bay_type TEXT NOT NULL DEFAULT 'general',
    capacity INTEGER NOT NULL DEFAULT 1,
    status TEXT NOT NULL DEFAULT 'active',
    version INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS reservations (
    id TEXT PRIMARY KEY,
    asset_type TEXT NOT NULL CHECK(asset_type IN ('vehicle','bay')),
    asset_id TEXT NOT NULL,
    store_id TEXT NOT NULL REFERENCES stores(id),
    user_id TEXT NOT NULL REFERENCES users(id),
    user_id_pseudonym TEXT NOT NULL DEFAULT '',
    start_time TEXT NOT NULL,
    end_time TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'confirmed' CHECK(status IN ('confirmed','cancelled','completed')),
    ticket_id TEXT,
    version INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS tickets (
    id TEXT PRIMARY KEY,
    ticket_number TEXT NOT NULL UNIQUE,
    reservation_id TEXT NOT NULL REFERENCES reservations(id),
    qr_data TEXT NOT NULL,
    valid_from TEXT NOT NULL,
    valid_until TEXT NOT NULL,
    redeemed INTEGER NOT NULL DEFAULT 0,
    redeemed_at TEXT,
    redeemed_by TEXT,
    undo_eligible_until TEXT,
    undone INTEGER NOT NULL DEFAULT 0,
    undone_at TEXT,
    undone_by TEXT,
    undo_reason TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS ticket_redemptions (
    id TEXT PRIMARY KEY,
    ticket_id TEXT NOT NULL REFERENCES tickets(id),
    redeemed_by TEXT NOT NULL REFERENCES users(id),
    redeemed_at TEXT NOT NULL,
    undone INTEGER NOT NULL DEFAULT 0,
    undone_at TEXT,
    undone_by TEXT,
    undo_reason TEXT
);

CREATE TABLE IF NOT EXISTS recovery_codes (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id),
    code_hash TEXT NOT NULL,
    issued_by TEXT NOT NULL REFERENCES users(id),
    issued_at TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at TEXT NOT NULL,
    used INTEGER NOT NULL DEFAULT 0,
    used_at TEXT
);

CREATE TABLE IF NOT EXISTS uploads (
    id TEXT PRIMARY KEY,
    filename TEXT NOT NULL,
    content_type TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    sha256_fingerprint TEXT NOT NULL UNIQUE,
    vehicle_id TEXT,
    store_id TEXT,
    uploader_id TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    actor_id TEXT NOT NULL,
    actor_username TEXT NOT NULL,
    action TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id TEXT NOT NULL,
    details_json TEXT NOT NULL DEFAULT '{}',
    previous_hash TEXT NOT NULL DEFAULT '',
    current_hash TEXT NOT NULL
);

-- Append-only enforcement: prevent UPDATE and DELETE on audit_log
CREATE TRIGGER IF NOT EXISTS audit_log_no_update
    BEFORE UPDATE ON audit_log
BEGIN
    SELECT RAISE(ABORT, 'audit_log is append-only: UPDATE is prohibited');
END;

CREATE TRIGGER IF NOT EXISTS audit_log_no_delete
    BEFORE DELETE ON audit_log
BEGIN
    SELECT RAISE(ABORT, 'audit_log is append-only: DELETE is prohibited');
END;

CREATE TABLE IF NOT EXISTS audit_hash_anchors (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    anchor_time TEXT NOT NULL DEFAULT (datetime('now')),
    last_log_id INTEGER NOT NULL,
    cumulative_hash TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS encryption_keys (
    id TEXT PRIMARY KEY,
    key_name TEXT NOT NULL UNIQUE,
    key_material_encrypted TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS backups (
    id TEXT PRIMARY KEY,
    filename TEXT NOT NULL,
    path TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    sha256 TEXT NOT NULL,
    created_by TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_store ON users(store_id);
CREATE INDEX IF NOT EXISTS idx_vehicles_store ON vehicles(store_id);
CREATE INDEX IF NOT EXISTS idx_vehicles_status ON vehicles(status);
CREATE INDEX IF NOT EXISTS idx_vehicles_vin_hash ON vehicles(vin_hash);
CREATE INDEX IF NOT EXISTS idx_reservations_asset ON reservations(asset_type, asset_id);
CREATE INDEX IF NOT EXISTS idx_reservations_time ON reservations(start_time, end_time);
CREATE INDEX IF NOT EXISTS idx_reservations_store ON reservations(store_id);
CREATE INDEX IF NOT EXISTS idx_reservations_user ON reservations(user_id);
CREATE INDEX IF NOT EXISTS idx_reservations_user_pseudo ON reservations(user_id_pseudonym);
CREATE INDEX IF NOT EXISTS idx_tickets_number ON tickets(ticket_number);
CREATE INDEX IF NOT EXISTS idx_tickets_reservation ON tickets(reservation_id);
CREATE INDEX IF NOT EXISTS idx_audit_resource ON audit_log(resource_type, resource_id);
CREATE INDEX IF NOT EXISTS idx_audit_actor ON audit_log(actor_id);
CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_log(timestamp);
CREATE INDEX IF NOT EXISTS idx_assignments_photographer ON photographer_assignments(photographer_user_id);
CREATE INDEX IF NOT EXISTS idx_assignments_store ON photographer_assignments(store_id);
CREATE INDEX IF NOT EXISTS idx_recovery_codes_user ON recovery_codes(user_id);
CREATE INDEX IF NOT EXISTS idx_uploads_fingerprint ON uploads(sha256_fingerprint);
