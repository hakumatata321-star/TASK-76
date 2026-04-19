# Recheck Update — `audit_report-2.md`

I re-ran a static recheck against the current repository state.

## Updated Result
- **Fixed:** 4/6
- **Partially fixed:** 2/6
- **Not fixed:** 0/6

## Issue Status (Updated)

1) **H1 Week-view calendar**
- **Status:** Fixed
- **Evidence:** `repo/frontend/src/pages/calendar.rs:150`, `repo/frontend/src/pages/calendar.rs:157`, `repo/frontend/src/pages/calendar.rs:191`, `repo/frontend/src/utils/calendar.rs:3`

2) **H2 Business-hours-driven slots**
- **Status:** Fixed
- **Evidence:** `repo/frontend/src/pages/calendar.rs:140`, `repo/frontend/src/pages/calendar.rs:141`, `repo/frontend/src/pages/calendar.rs:142`, `repo/frontend/src/utils/time.rs:2`

3) **H3 Logout token invalidation**
- **Status:** Fixed
- **Evidence:** `repo/backend/src/handlers/auth.rs:66`, `repo/backend/src/app/state.rs:15`, `repo/backend/src/routes/mod.rs:24`, `repo/backend/src/routes/mod.rs:47`, `repo/backend/tests/api_tests/test_api_auth.rs:108`, `repo/backend/tests/api_tests/test_api_auth.rs:147`

4) **H4 Insecure defaults / shared credentials**
- **Status:** Partially fixed
- **What is fixed:** secret env fallbacks removed and `.env.example` enforced in docs/startup.
- **What remains:** publicly known default bootstrap password path still exists when `BOOTSTRAP_ADMIN_PASSWORD` is omitted.
- **Evidence:** `repo/docker-compose.yml:12`, `repo/.env.example:14`, `repo/backend/src/main.rs:91`, `repo/backend/src/main.rs:100`, `repo/README.md:54`

5) **M1 Calendar status filter refresh**
- **Status:** Fixed
- **Evidence:** `repo/frontend/src/pages/calendar.rs:112`, `repo/frontend/src/pages/calendar.rs:121`

6) **M2 Behavioral frontend test coverage**
- **Status:** Partially fixed
- **What is fixed:** added Playwright E2E spec and additional calendar/business-hour tests.
- **What remains:** large portion of frontend suite still source-inspection via `include_str!`; E2E test appears not wired into documented runner/config.
- **Evidence:** `repo/frontend/tests/e2e/fullstack_ui_e2e.spec.ts:1`, `repo/frontend/tests/frontend_utils_spec.rs:15`, `repo/frontend/tests/frontend_behavior_spec.rs:12`, `repo/run_tests.sh:21`

## Bottom Line
- Claude’s fixes are mostly present and real.
- Remaining follow-ups are mainly:
  1. require `BOOTSTRAP_ADMIN_PASSWORD` (or fail hard if missing),
  2. wire E2E execution into documented test workflow and reduce reliance on source-string tests for critical flows.
