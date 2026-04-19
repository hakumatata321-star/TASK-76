# FleetReserve Operations Suite - Architecture Design

## Architecture Overview

FleetReserve is a fully offline, facility-network web application composed of:

- **Frontend**: Leptos 0.6 compiled to WebAssembly, served as static files via nginx or trunk-serve. Runs entirely in the browser on the facility LAN.
- **Backend**: Axum 0.7 REST API server written in Rust, exposing JSON endpoints consumed by the frontend.
- **Database**: SQLite single-file database as the on-device system of record, accessed via rusqlite with WAL mode for concurrent reads.
- **Deployment**: Docker Compose orchestrating a backend container and a frontend/nginx container on the facility network.

```
Browser (Leptos WASM)
    |
    | HTTP/JSON (LAN only)
    v
Nginx (static files + reverse proxy)
    |
    | /api/* proxy
    v
Axum Server (Rust)
    |
    | rusqlite
    v
SQLite (WAL mode)
    |
    +-- data tables (users, vehicles, reservations, tickets, ...)
    +-- audit_log (append-only hash chain)
    +-- encrypted fields (AES-256-GCM)
    +-- uploads (filesystem, fingerprinted)
```

## repo/frontend Responsibilities

- Authentication UI (login, password reset)
- Role-based navigation and route guards
- Unified availability calendar (day/week views, filters, 15-min increments)
- Reservation creation with inline conflict explanation
- E-ticket display with QR code and validity window
- Check-in screen for scanning/manual entry with undo capability
- Vehicle management UI with masked sensitive fields
- Photo upload with client-side validation
- Admin panel (roles, permissions, recovery codes, backup/restore)
- Export interface for Platform Operations
- CSRF token management on state-changing requests

## repo/backend Responsibilities

- REST API for all business operations
- Authentication (Argon2id password hashing, signed session tokens)
- Authorization (route-level, function-level, object-level)
- CSRF validation on state-changing endpoints
- Reservation engine with transactional conflict detection and retry
- Ticket generation and redemption with undo enforcement
- Vehicle lifecycle management with permission-gated transitions
- Tamper-evident audit chain with periodic hash anchors
- Encryption at rest for sensitive fields
- Masking of sensitive data in API responses
- Upload validation (magic bytes, MIME, size, fingerprint dedup)
- Backup/restore with encryption
- Database migrations

## Module Decomposition

### Backend Modules
```
src/
  main.rs              -- Entry point, server startup
  app/
    mod.rs             -- App module
    state.rs           -- AppState (DB pool, keys, config)
  routes/
    mod.rs             -- Router construction with all routes and middleware
  handlers/
    mod.rs             -- Handler module aggregation
    auth.rs            -- Login, logout, me, password reset
    vehicles.rs        -- Vehicle CRUD, status transitions
    bays.rs            -- Service bay management
    reservations.rs    -- Reservation creation and queries
    tickets.rs         -- Ticket display, redemption, undo
    calendar.rs        -- Calendar data (day/week views)
    uploads.rs         -- File upload endpoint
    admin.rs           -- Role/permission/user management
    backup.rs          -- Backup creation and restore
    assignments.rs     -- Photographer assignment queries
    exports.rs         -- Data export for PlatformOps
  services/
    mod.rs             -- Service aggregation
    reservation_engine.rs  -- Core conflict detection, retry, alternatives
    ticket_engine.rs       -- Ticket generation, redemption, undo
    uploads.rs             -- Validation, fingerprinting, storage
    crypto.rs              -- AES-256-GCM encrypt/decrypt wrapper
  repositories/
    mod.rs             -- Repository aggregation
    users.rs           -- User queries
    vehicles.rs        -- Vehicle queries with version checks
    bays.rs            -- Bay queries with capacity checks
    reservations.rs    -- Reservation queries and creation
    tickets.rs         -- Ticket queries
    stores.rs          -- Store queries
    assignments.rs     -- Photographer assignment queries
    recovery_codes.rs  -- Recovery code management
    uploads.rs         -- Upload metadata queries
    backups.rs         -- Backup metadata queries
    audit.rs           -- Audit log queries
  models/
    mod.rs             -- All domain model structs and enums
  auth/
    mod.rs             -- Auth module
    password.rs        -- Argon2id hash/verify
    session.rs         -- HMAC-signed token create/validate
    csrf.rs            -- CSRF token generation and validation
  security/
    mod.rs             -- Security module
    headers.rs         -- CSP, X-Content-Type-Options, etc.
    encryption.rs      -- Field-level AES-256-GCM encryption
    masking.rs         -- VIN/plate/username masking
  audit/
    mod.rs             -- Audit module
    chain.rs           -- Append-only hash chain logic
    anchors.rs         -- Periodic hash anchor creation
  backup/
    mod.rs             -- Backup creation and restore
  uploads/
    mod.rs             -- Upload handling reexport
  errors/
    mod.rs             -- AppError enum, IntoResponse impl
  migrations/
    001_initial_schema.sql
    002_seed_data.sql
```

### Frontend Modules
```
src/
  lib.rs               -- Entry point, mount_to_body
  app.rs               -- Root App component with Router
  routes/
    mod.rs             -- Route definitions
  pages/
    mod.rs             -- Page module aggregation
    login.rs           -- Login page
    dashboard.rs       -- Role-based dashboard
    calendar.rs        -- Availability calendar
    reservations.rs    -- Reservation creation/management
    vehicles.rs        -- Vehicle management
    tickets.rs         -- Ticket display and check-in
    admin.rs           -- Admin panel
    exports.rs         -- Export interface
    assignments.rs     -- Photographer assignments
  components/
    mod.rs             -- Component aggregation
    nav.rs             -- Navigation bar
    calendar_grid.rs   -- Calendar time grid
    conflict_explanation.rs -- Conflict display
    ticket_display.rs  -- Ticket with QR
    vehicle_card.rs    -- Vehicle card
    upload_form.rs     -- File upload
    status_badge.rs    -- Status badges
    role_guard.rs      -- Role-based conditional rendering
  state/
    mod.rs             -- State management
    auth.rs            -- Auth state (token, user, CSRF)
    app_state.rs       -- Global state provider
  api/
    mod.rs             -- API module
    client.rs          -- HTTP client with auth headers
    types.rs           -- Request/response types
  security/
    mod.rs             -- Security module
    csrf.rs            -- CSRF token handling
    route_guard.rs     -- Route access guards
  utils/
    mod.rs             -- Utility aggregation
    format.rs          -- Date/time/number formatting
    time.rs            -- Time slot calculations
```

## Role Model

| Role | Scope | Capabilities |
|------|-------|-------------|
| Customer | Own data | View own reservations and tickets, request reservations |
| Photographer | Assigned jobs | View assignments tied to their jobs, access related vehicles/bays |
| Merchant/Store Staff | Own store | Manage fleet assets, reservations, bays, assignments for their store |
| Platform Operations | All stores | Cross-store calendar views, exports, manage reservations across stores |
| Administrator | System-wide | All capabilities plus user/role/permission management, recovery codes, backup/restore |

## Authentication Model

- **Mechanism**: Username + plaintext password over HTTPS (or LAN HTTP)
- **Password Storage**: Argon2id with recommended parameters (19 MiB memory, 2 iterations, 1 parallelism)
- **Offline Only**: No external identity providers, no email verification
- **Password Reset**: Admin issues one-time recovery code -> user resets password using code within 30 minutes
- **Account Lockout**: Not specified in prompt; not implemented to avoid operational lockout risk in offline facility

## Session and CSRF Model

- **Session Tokens**: HMAC-SHA256 signed JSON payload containing `{user_id, username, role, store_id, iat, exp}`
- **Idle Timeout**: 12 hours from last activity; each successful API call updates the `iat` (reissues token)
- **Token Delivery**: Returned in login response body; frontend stores in memory (not localStorage for XSS protection)
- **CSRF Protection**: Server generates a random CSRF token at login, stored server-side tied to session. Frontend sends it as `X-CSRF-Token` header on all POST/PUT/DELETE requests. Server validates before processing.
- **XSS Defense**: Output encoding handled by Leptos templating (auto-escapes), Content-Security-Policy header restricts script sources

## Authorization Model

### Route-Level
Every API endpoint declares its minimum required role. The auth middleware extracts the session token, validates it, and checks the user's role against the route requirement before the handler executes.

### Function-Level
Within handlers, business logic checks enforce additional constraints (e.g., only store staff can transition vehicle status for their store's vehicles).

### Object-Level
- **Store Isolation**: Merchant/Store Staff handlers filter all queries by `store_id = user.store_id`. Platform Operations and Admin bypass this filter.
- **Photographer Isolation**: Photographer handlers join through `photographer_assignments` to filter by `photographer_user_id = user.id`.
- **Customer Isolation**: Customer handlers filter by `user_id = current_user.id` for reservations and tickets.

## Reservation Conflict Detection Model

```
BEGIN IMMEDIATE TRANSACTION
  1. Read asset (vehicle or bay) row, note version
  2. Check: asset.status not in (in-repair, decommissioned) -- for vehicles
  3. Check: vehicle.insurance_expiry >= reservation.end_time -- for vehicles
  4. Check: no overlapping reservations for this asset in [start, end)
  5. Check: bay capacity not exceeded in [start, end) -- for bays
  6. If all pass:
     a. INSERT reservation
     b. UPDATE asset SET version = version + 1
     c. COMMIT
     d. Generate ticket
     e. Return success
  7. If any check fails:
     a. Collect all conflict reasons (deterministic order: overlapping, in-repair, expired-insurance, capacity)
     b. From SAME snapshot, compute 2 nearest alternative time slots
     c. From SAME snapshot, find alternate eligible assets
     d. ROLLBACK
     e. Return ConflictResponse
```

## Optimistic Concurrency and Retry Model

- Each asset (vehicle, bay) has a `version` integer column
- Reservation creation reads the version and includes `WHERE version = ?` in the UPDATE
- If the UPDATE affects 0 rows (version changed), the transaction is retried
- Maximum 3 retries before returning a deterministic conflict response
- The conflict response is computed from the final committed snapshot to ensure consistency

## Alternative Slot Suggestion Model

When a conflict is detected, from the same database snapshot:
1. Query all existing reservations for the same asset on the same business day
2. Identify free 15-minute-aligned windows that fit the requested duration
3. Score by proximity to the originally requested start time
4. Prefer future slots over past slots when equidistant
5. Return the 2 nearest free slots within business hours

## Alternate Vehicle/Bay Selection Model

When the primary asset has a conflict:
1. Query other assets of the same type (vehicle or bay) in the same store
2. Filter to assets that are available in the requested time window
3. For vehicles: exclude in-repair, decommissioned, expired-insurance
4. Return up to 3 alternate eligible assets with their details

## Ticket Generation and Redemption Model

- **Generation**: On reservation confirmation, generate:
  - `ticket_number`: "FR-" + 8 uppercase alphanumeric characters (cryptographically random)
  - `qr_data`: JSON-encoded `{ticket_number, valid_from, valid_until, reservation_id}`
  - `valid_from`, `valid_until`: Match the reservation time window
- **Display**: QR code rendered as SVG, human-readable number displayed prominently
- **Redemption**: POST with ticket ID or number -> mark `redeemed = 1`, record `redeemed_at` and `redeemed_by`
- **Re-entry Block**: If `redeemed = 1` and `undone = 0`, reject with "already redeemed" message

## Supervised Undo Model

- **Window**: 2 minutes from `redeemed_at` timestamp (server-enforced, not client clock)
- **Reason**: Mandatory non-empty text string explaining the undo
- **Eligibility**: Only Merchant/Store Staff or higher role can perform undo
- **Effect**: Sets `undone = 1`, records `undone_at`, `undone_by`, `undo_reason`. Ticket becomes redeemable again.
- **Audit**: Both the original redemption and the undo are recorded in the audit chain

## Upload Validation and Duplicate Fingerprint Model

1. **Size Check**: Reject files > 10 MB immediately
2. **Magic Byte Check**: Read first 8 bytes; verify JPEG (FF D8 FF) or PNG (89 50 4E 47 0D 0A 1A 0A)
3. **MIME Sniffing**: Use `infer` crate to detect actual content type from file content
4. **Content Stripping**: For JPEG, strip all non-essential APP segments and EXIF data to remove potential executable content
5. **Fingerprinting**: Compute SHA-256 of the validated file content
6. **Deduplication**: Check `uploads.sha256_fingerprint` for existing match; reject with 409 if duplicate
7. **Storage**: Save to configured upload directory with UUID filename

## Vehicle Lifecycle and Status Transition Permissions

```
available <-> reserved    (System automatic on reservation create/cancel)
available <-> on-rent     (Merchant/Store Staff+)
available <-> in-repair   (Merchant/Store Staff+)
reserved  -> on-rent      (Merchant/Store Staff+)
on-rent   -> available    (Merchant/Store Staff+)
in-repair -> available    (Merchant/Store Staff+)
any       -> decommissioned (Administrator only)
```

All transitions are validated server-side, permission-checked, and audit-logged with previous and new status.

## Audit Chain Model

Every auditable action writes an `audit_log` entry:
- `id`: Auto-increment
- `timestamp`: UTC ISO-8601
- `actor_id`, `actor_username`: Who performed the action
- `action`: CREATE, UPDATE, DELETE, LOGIN, LOGOUT, REDEEM, UNDO, EXPORT, BACKUP, RESTORE, STATUS_CHANGE, PERMISSION_CHANGE
- `resource_type`: user, vehicle, bay, reservation, ticket, upload, role, permission, recovery_code, backup
- `resource_id`: ID of the affected resource
- `details_json`: Structured JSON with action-specific details
- `previous_hash`: SHA-256 hash of the immediately preceding entry
- `current_hash`: SHA-256(previous_hash + id + timestamp + actor_id + action + resource_type + resource_id + details_json)

The chain is append-only: no UPDATE or DELETE operations on audit_log are permitted through the application.

## Periodic Hash Anchor Model

- Every 100 audit log entries, the system creates a hash anchor
- The anchor records:
  - `anchor_time`: Current UTC timestamp
  - `last_log_id`: ID of the most recent audit log entry
  - `cumulative_hash`: SHA-256 of all entry hashes since the previous anchor
- Anchors enable efficient integrity verification: check entries between anchors rather than the entire chain
- Administrators can trigger manual anchor creation

## Encryption at Rest Model

- **Algorithm**: AES-256-GCM with random 96-bit nonces
- **Scope**: VIN, license plate, and user email fields in the database
- **Key Storage**: `encryption_keys` table stores the key material (in production, the table itself should be on an encrypted filesystem)
- **Key Loading**: Key is loaded into AppState at startup and held in memory
- **Ciphertext Format**: base64(nonce || ciphertext || tag)

## Masking Policy Model

API responses to UI-facing endpoints mask sensitive fields by default:
- **VIN**: Show only last 4 characters: `"*************1234"`
- **License Plate**: Show only last 2 characters: `"*****AB"`
- **Username**: Show first character + `"***"`: `"j***"`
- **Email**: Masked entirely: `"****@****"`

Unmasked data is only available through explicit detail endpoints with appropriate authorization (Administrator or the owning user for their own data).

## Backup and Restore Model

- **Backup**: Administrator triggers backup -> system creates a copy of the SQLite database file -> encrypts it with AES-256-GCM using the system key -> writes to admin-specified local path -> records metadata in `backups` table -> audit logged
- **Restore**: Administrator provides backup file path -> system decrypts -> validates integrity -> replaces current database -> audit logged (in new database)
- **Restore Authorization**: Administrator role only, verified before restore begins

## Docker Deployment Model

```yaml
services:
  backend:   # Rust binary, port 3001, mounts data volume
  frontend:  # nginx serving WASM + proxying /api to backend
volumes:
  app-data:  # Persistent SQLite DB and upload files
```

- Backend builds from `repo/backend/Dockerfile` (multi-stage Rust build)
- Frontend builds from `repo/frontend/Dockerfile` (trunk build + nginx serve)
- Environment variables configure encryption key, HMAC secret, paths
- Single `docker-compose up` starts the complete system

## Logging and Observability Approach

- **Structured Logging**: `tracing` crate with JSON output for machine-parseable logs
- **Log Levels**: ERROR for failures, WARN for recoverable issues, INFO for business events, DEBUG for diagnostics
- **Request Logging**: `tower-http::trace` middleware logs all HTTP requests with method, path, status, and duration
- **Audit vs Logs**: Business-significant events go to both the audit chain (persistent, tamper-evident) and the application log (operational, potentially rotated)
- **No External Dependencies**: All logging is to stdout/stderr (Docker-friendly) and the SQLite audit chain
