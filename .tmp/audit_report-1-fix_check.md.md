# FleetReserve Issue Fix Verification (Updated)

Source reviewed: `.tmp/audit_report-1.md.md`
Verification date: 2026-04-19
Method: static re-audit after fixes + targeted test invocation attempt

## High-Priority Issues

| Issue | Current status | Verification notes |
|---|---|---|
| ISS-001 (bootstrap contradiction) | **Fixed** | First-run admin bootstrap is implemented in backend startup (`repo/backend/src/main.rs:22`, `repo/backend/src/main.rs:67`) and aligned with docs (`repo/README.md:32`, `repo/README.md:46`). |
| ISS-002 (photographer overreach) | **Fixed** | Photographer reservation/ticket reads are assignment-scoped (`repo/backend/src/handlers/reservations.rs:127`, `repo/backend/src/handlers/tickets.rs:67`) and reservation creation is explicitly denied (`repo/backend/src/handlers/reservations.rs:29`). API authz test now asserts this deny path (`repo/backend/tests/api_tests/test_api_authz_matrix.rs:38`). |
| ISS-003 (sensitive identifier handling) | **Fixed** | User IDs are masked in reservation/export/audit responses (`repo/backend/src/handlers/reservations.rs:149`, `repo/backend/src/handlers/exports.rs:55`, `repo/backend/src/handlers/admin.rs:186`). Reservation writes now persist pseudonymized user identifier at rest (`repo/backend/migrations/001_initial_schema.sql:88`, `repo/backend/src/services/reservation_engine.rs:208`). |
| ISS-004 (scanner missing) | **Fixed** | QR scanner endpoint is implemented (`repo/backend/src/handlers/scan.rs:13`) and wired in routes (`repo/backend/src/routes/mod.rs:169`); frontend check-in supports camera/file scan (`repo/frontend/src/pages/tickets.rs:140`). |
| ISS-005 (permission management non-effective) | **Fixed** | Permission-table enforcement is now applied broadly via centralized helper (`repo/backend/src/handlers/auth.rs:159`) across reservations/tickets/assignments/vehicles/bays/calendar/uploads/backup/admin handlers. Permission revocation impact is covered by API test (`repo/backend/tests/api_tests/test_api_exports.rs:184`). |

## Medium / Low Issues

| Previous issue | Current status | Verification notes |
|---|---|---|
| `GET /api/exports` mutates state without CSRF | **Fixed** | Export endpoint is POST with CSRF enforcement (`repo/backend/src/routes/mod.rs:175`, `repo/backend/src/handlers/exports.rs:18`) and frontend uses POST (`repo/frontend/src/pages/exports.rs:11`). |
| Frontend tests too structural | **Fixed** | Added additional behavior-oriented frontend test coverage for auth role logic (`repo/frontend/tests/frontend_state_behavior_spec.rs:1`) alongside existing behavior specs (`repo/frontend/tests/frontend_behavior_spec.rs:1`). |
| Duplicate `API_TESTS` / `unit_tests` trees | **Fixed** | Duplicate root-level test files were removed; canonical suites remain under `repo/backend/tests/*` (root dirs now empty: `repo/API_TESTS/`, `repo/unit_tests/`). |

## Overall Recheck Verdict

**Full Pass (static re-audit): all previously reported unresolved items have been addressed in codebase structure.**

## Execution note

- Attempted to run targeted backend test command, but this environment does not have `cargo` installed (`zsh: command not found: cargo`).
- Runtime confirmation should be executed in a Rust-enabled environment with `./run_tests.sh` or backend `cargo test`.
