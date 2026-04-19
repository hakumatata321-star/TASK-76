# Final Recheck After Requested Fixes

## Outcome
- **Fixed:** 6/6
- **Partially fixed:** 0/6
- **Not fixed:** 0/6

## Evidence by Previously Reported Item

1. **H1 Week view calendar implementation** — Fixed
- `repo/frontend/src/pages/calendar.rs:150`
- `repo/frontend/src/pages/calendar.rs:157`
- `repo/frontend/src/pages/calendar.rs:191`
- `repo/frontend/src/utils/calendar.rs:3`

2. **H2 Business-hours-driven slot generation** — Fixed
- `repo/frontend/src/pages/calendar.rs:140`
- `repo/frontend/src/pages/calendar.rs:141`
- `repo/frontend/src/pages/calendar.rs:142`
- `repo/frontend/src/utils/time.rs:2`

3. **H3 Logout token invalidation across read routes** — Fixed
- `repo/backend/src/handlers/auth.rs:66`
- `repo/backend/src/app/state.rs:15`
- `repo/backend/src/routes/mod.rs:24`
- `repo/backend/src/routes/mod.rs:47`
- `repo/backend/tests/api_tests/test_api_auth.rs:108`

4. **H4 Insecure defaults/shared credential posture** — Fixed
- secrets are required and no compose fallback defaults: `repo/docker-compose.yml:12`
- bootstrap admin password required on first activation: `repo/backend/src/main.rs:91`
- bootstrap password documented as required: `repo/.env.example:14`
- README no longer advertises fixed runtime admin password: `repo/README.md:44`

5. **M1 Calendar status filter refresh** — Fixed
- `repo/frontend/src/pages/calendar.rs:121`

6. **M2 Behavioral test coverage credibility** — Fixed
- Added/extended real browser E2E spec for core navigation + calendar day/week/filter interaction: `repo/frontend/tests/e2e/fullstack_ui_e2e.spec.ts:1`
- Added Playwright config and dependency wiring: `repo/frontend/playwright.config.ts:1`, `repo/frontend/package.json:1`
- Wired optional E2E execution path into test runner and documented it: `repo/run_tests.sh:24`, `repo/README.md:79`
