# Recheck of `audit_report-2.md` Issues (After Claimed Claude Changes)

Static verification only (no run/test/docker). This recheck compares the prior issue list against current source.

## Summary
- **Fixed:** 4
- **Partially fixed:** 2
- **Not fixed:** 0

## Per-Issue Status

1) **H1 — Week view calendar not credibly implemented**
- **Status:** Fixed
- **Why:** Week mode now renders a dated 7-day grid and keys reservation cells by full `YYYY-MM-DDTHH:MM:SS` slot.
- **Evidence:** `repo/frontend/src/pages/calendar.rs:150`, `repo/frontend/src/pages/calendar.rs:157`, `repo/frontend/src/pages/calendar.rs:191`, `repo/frontend/src/utils/calendar.rs:3`

2) **H2 — Frontend calendar ignored configurable business hours**
- **Status:** Fixed
- **Why:** Slot generation now derives from `business_hours.start/end` via `parse_hour`, no hardcoded `7..19`.
- **Evidence:** `repo/frontend/src/pages/calendar.rs:140`, `repo/frontend/src/pages/calendar.rs:141`, `repo/frontend/src/pages/calendar.rs:142`, `repo/frontend/src/utils/time.rs:2`

3) **H3 — Logout did not invalidate bearer token for read routes**
- **Status:** Fixed
- **Why:** Logout now records revoked session key; route middleware checks revocation for auth/staff/ops/admin paths.
- **Evidence:** `repo/backend/src/handlers/auth.rs:66`, `repo/backend/src/app/state.rs:15`, `repo/backend/src/routes/mod.rs:24`, `repo/backend/src/routes/mod.rs:47`, `repo/backend/tests/api_tests/test_api_auth.rs:108`, `repo/backend/tests/api_tests/test_api_auth.rs:147`

4) **H4 — Insecure defaults / shared credentials in docs+config**
- **Status:** Partially fixed
- **Why fixed:** Docker secret fallbacks were removed and `.env.example` added; startup now requires explicit non-placeholder secrets.
- **Why partial:** README still publishes shared demo passwords, and bootstrap still sets known default admin password on first run.
- **Evidence:** `repo/docker-compose.yml:12`, `repo/docker-compose.yml:13`, `repo/.env.example:1`, `repo/backend/src/main.rs:24`, `repo/backend/src/main.rs:36`, `repo/README.md:42`, `repo/backend/src/main.rs:90`

5) **M1 — Status filter checkboxes did not trigger immediate refresh**
- **Status:** Fixed
- **Why:** Status checkbox handler now calls `load_calendar()` after updating filter state.
- **Evidence:** `repo/frontend/src/pages/calendar.rs:112`, `repo/frontend/src/pages/calendar.rs:121`

6) **M2 — Frontend tests mostly source-inspection with weak behavioral coverage**
- **Status:** Partially fixed
- **Why fixed:** Added behavioral tests for calendar utilities and business-hour slot derivation; backend calendar week/business-hour API tests expanded.
- **Why partial:** A large portion of frontend tests still rely on `include_str!` source-inspection rather than component interaction tests.
- **Evidence:** `repo/frontend/tests/frontend_behavior_spec.rs:267`, `repo/frontend/tests/frontend_utils_spec.rs:15`, `repo/backend/tests/api_tests/test_api_calendar.rs:34`, `repo/backend/tests/api_tests/test_api_calendar.rs:108`, `repo/frontend/tests/frontend_behavior_spec.rs:227`

## Final Recheck Verdict
- The previously reported **H1/H2/H3/M1 issues are now fixed** in static code.
- **H4 and M2 remain partially resolved** and still warrant follow-up hardening.
