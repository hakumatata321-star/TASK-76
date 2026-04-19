Project Type: fullstack

# FleetReserve Operations Suite

Offline fleet scheduling + e-ticket check-in with role-aware operations, auditability, and backup/restore.

## Secret Configuration (Required Before First Run)

**`ENCRYPTION_KEY` and `HMAC_SECRET` must be set to unique values before starting the containers.** The backend refuses to start with absent or placeholder secrets.

```bash
cp .env.example .env
# Edit .env and replace the placeholder values with output from:
#   openssl rand -hex 32
docker-compose --env-file .env up
```

> **Warning:** Do not copy placeholder values from `.env.example` into a real deployment. Shared or predictable key material weakens backup encryption and token security. The defaults committed in this repository are intentionally invalid (they cause a startup panic); you must supply your own.

## Quick Start (Required Command)

```bash
cp .env.example .env   # fill in ENCRYPTION_KEY and HMAC_SECRET
docker-compose --env-file .env up
```

Use `docker-compose --env-file .env up --build` for a fresh rebuild.

## Access

- Frontend UI: [http://localhost:8081](http://localhost:8081)
- Backend API (direct): [http://localhost:3001](http://localhost:3001)

## Auth and Demo Credentials

> **These credentials are for development/demo use only.** Change the `admin` password immediately after first login on any non-development deployment via the Admin page or recovery-code reset flow.

Authentication is required.

| Role | Username | Password |
|---|---|---|
| Administrator | `admin` | `FleetReserveHttpTest#2026` |
| PlatformOps | `ops1` | `FleetReserveRoleTest#2026` |
| MerchantStaff | `merchant1` | `FleetReserveRoleTest#2026` |
| Photographer | `photo1` | `FleetReserveRoleTest#2026` |
| Customer | `customer1` | `FleetReserveRoleTest#2026` |

### Bootstrap / first-run process

On a **fresh install** (against an empty database) the backend automatically activates the `admin` account with the documented default password during startup. Login works immediately after the containers are healthy. **Change this password before sharing access.**

To use a custom bootstrap password instead of the default, set `BOOTSTRAP_ADMIN_PASSWORD` in `.env` before first run (see `.env.example`).

If admin credentials were changed and are unknown, use the recovery-code reset flow:

1. Issue a recovery code for the admin via `POST /api/admin/recovery-codes` (from
   another active admin session, or by a direct DB call in an emergency).
2. Reset via `POST /api/auth/reset-password` with the issued code.
3. Confirm the new credentials at `http://localhost:8081/login`.

## Startup Verification (Concrete)

1. `docker-compose ps` must show `backend` and `frontend` as `Up`.
2. `curl -s http://localhost:3001/api/auth/login -H "content-type: application/json" -d '{"username":"admin","password":"FleetReserveHttpTest#2026"}'`
   - expected: HTTP `200` with `token` and `csrf_token` fields.
3. Open `http://localhost:8081/login` and sign in using the same admin credentials.

## Test Execution

```bash
./run_tests.sh
```

`run_tests.sh` is Docker-only and runs backend + frontend suites.

### Optional E2E (Playwright)

The repository also includes browser E2E coverage under `frontend/tests/e2e/`.

1. Start the stack: `docker-compose --env-file .env up -d`
2. Install E2E deps: `npm --prefix frontend install`
3. Install browser: `npx --prefix frontend playwright install`
4. Run E2E: `RUN_E2E=1 ./run_tests.sh`

### Successful output indicators

- Script ends with `=== All available tests complete ===`
- No `FAILED` test blocks in output
- Docker run exits with code `0`

### Failure interpretation

- `Cannot connect to the Docker daemon`: Docker service not running
- Rust compile/test failure output: test or code regression; inspect preceding `error:` blocks
- Non-zero exit from script: at least one suite failed

## Backend API Inventory

All routes are under `/api`.

### Public
- `POST /api/auth/login`
- `POST /api/auth/reset-password`

### Authenticated (`require_auth`)
- `POST /api/auth/logout`
- `GET /api/auth/me`
- `POST /api/reservations`
- `GET /api/reservations`
- `GET /api/tickets/:id`
- `GET /api/assignments`

### Staff+ (`require_staff`)
- `GET /api/vehicles`
- `GET /api/vehicles/:id`
- `POST /api/vehicles`
- `PUT /api/vehicles/:id/status`
- `GET /api/bays`
- `POST /api/bays`
- `GET /api/stores`
- `GET /api/calendar`
- `POST /api/tickets/:id/redeem`
- `POST /api/tickets/:id/undo`
- `POST /api/tickets/scan` — decode a QR code image; returns `ticket_value`
- `POST /api/uploads`
- `POST /api/assignments`

### PlatformOps+ (`require_ops`)
- `POST /api/exports` — **POST** (writes an audit entry; requires CSRF)
- `GET /api/audit`

### Administrator (`require_admin`)
- `GET /api/admin/users`
- `POST /api/admin/users`
- `GET /api/admin/permissions`
- `POST /api/admin/permissions`
- `POST /api/admin/permissions/:id`
- `PUT /api/admin/users/:id/role`
- `PUT /api/admin/users/:id/active`
- `POST /api/admin/recovery-codes`
- `POST /api/backup`
- `POST /api/backup/restore`

## Permissions

Role-based middleware gates routes at the coarse level. Within the handler, the
`permissions` table provides fine-grained control: an administrator can remove a
permission entry (e.g. `PlatformOps / export / create`) and the next request will
be denied without a code deploy. Seeds in `002_seed_data.sql` reflect the default
permission matrix.

## Authorization model

- **Customer**: own reservations and tickets only.
- **Photographer**: assignments-scoped only — can list reservations and tickets
  only for vehicles/bays explicitly assigned to them. No store-wide visibility.
- **MerchantStaff**: store-scoped vehicles, bays, reservations, tickets, and calendar.
- **PlatformOps**: read-all + exports + audit.
- **Administrator**: all of the above + user/role/permission management, recovery
  codes, backup/restore, vehicle decommission.

## Check-in Scanner

The check-in page (`/checkin`) supports both manual ticket-number entry and QR
code image scanning:

1. Enter a ticket number (`FR-XXXXXXXX`) directly, **or**
2. Use the camera/file picker ("Scan QR code image") — the selected image is sent
   to `POST /api/tickets/scan` which decodes it server-side and populates the
   ticket field automatically.

Both paths feed into the same `POST /api/tickets/:id/redeem` validation.

## UI Verification Flow (End-to-End)

1. Login as `admin` in UI.
2. Vehicles: verify list renders, create vehicle, update status.
3. Reservations: create reservation and confirm ticket output.
4. Check-in (`/checkin`): redeem then undo a ticket; also test camera/file scanner.
5. Admin: list users, issue recovery code, create and restore backup.
6. Role checks: sign in as `customer1`, `photo1`, `merchant1`, `ops1` and verify
   forbidden/allowed screens.

## Manual Verifications with Observable Output

1. Concurrency contention:
   - run two parallel reservation creates for same asset/time
   - expected: one `201`, one `409 conflict`
2. QR camera scan:
   - scan generated ticket at `/checkin` using the file/camera input
   - expected: ticket field is populated; clicking Redeem → `SUCCESS: Ticket redeemed successfully!`
3. Backup/restore:
   - create backup in Admin page
   - expected: success message + encrypted `.enc` file under mounted backup path
4. Export masking:
   - run `POST /api/exports` via the Exports page
   - expected: `user_id` in reservations shows `usr-***-XXXX` (not a raw UUID)

## Developer Notes

**Week view:** The calendar page renders two distinct grid layouts. Day view uses a 2-column CSS grid (time label + single content column). Week view uses an 8-column grid (time label + one column per day, Mon–Sun). The backend `/api/calendar?view=week` returns slots with full `YYYY-MM-DDTHH:MM:SS` time keys; the frontend extracts the seven unique date prefixes from the slot list and builds the header row and per-day slot cells from those keys.

**Business hours:** Slot generation is driven by `calendar_data.business_hours.start` and `calendar_data.business_hours.end` returned by the API (sourced from the `stores` table). The `parse_hour("HH:MM")` utility in `frontend/src/utils/time.rs` converts the string to an integer hour, which is passed directly to `generate_time_slots`. No slot bounds are hardcoded in the frontend; changing store hours in the database is immediately reflected in the rendered grid.

**Logout invalidation:** On logout the backend records a revocation key (`user_id:iat`) in an in-memory `HashSet` stored in `AppState`. Every subsequent request through `require_auth`, `require_staff`, `require_ops`, or `require_admin` checks this set before accepting the bearer token. A revoked token is rejected with `401 Unauthorized` even if it is cryptographically valid and not yet expired. The revocation set is in-memory (cleared on restart), so it is appropriate for the offline/single-node deployment model; operators requiring cross-restart revocation should replace it with a database-backed session table.
