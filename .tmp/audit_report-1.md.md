# FleetReserve Static Audit Report

## 1. Verdict
- **Overall conclusion:** **Partial Pass**

## 2. Scope and Static Verification Boundary
- **Reviewed:** repository docs/config, backend Axum routes/handlers/services/repositories/migrations, frontend Leptos routes/pages/components, and test sources under `backend/tests`, `API_TESTS`, `unit_tests`, and `frontend/tests`.
- **Excluded:** `./.tmp/` and all subpaths as required.
- **Not executed intentionally:** project startup, Docker, tests, browser flows, API calls, concurrency race reproduction.
- **Cannot confirm statistically:** real runtime behavior (e.g., true concurrent contention outcomes, camera scanning UX/perf, browser rendering/accessibility details, backup/restore behavior on actual operator paths).
- **Manual verification required for:** camera/scan experience, end-to-end deployment bootstrap on fresh install, production-like restore/rollback safety, and real-world role workflow validation.

## 3. Repository / Requirement Mapping Summary
- **Prompt core goal mapped:** offline fleet scheduling + e-ticket admissions + auditable role-based operations.
- **Core flows mapped:** auth/session/CSRF, reservation creation + conflict reasons/alternatives, ticket redeem/undo, role-scoped listing/access, uploads, audit log chain/anchors, backup/restore.
- **Main implementation areas reviewed:** `backend/src/routes/mod.rs`, `backend/src/handlers/*.rs`, `backend/src/services/*.rs`, `backend/migrations/*.sql`, `frontend/src/pages/*.rs`, `frontend/src/components/*.rs`.
- **Primary risks assessed:** bootstrap viability, authorization boundaries (route/object/function scope), sensitive data handling/masking/encryption claims, and static test support for high-risk paths.

## 4. Section-by-section Review

### 1. Hard Gates

#### 1.1 Documentation and static verifiability
- **Conclusion:** **Fail**
- **Rationale:** Documentation says default admin login should work immediately, but seeded admin is disabled and no public bootstrap path exists to activate first admin on fresh deployment.
- **Evidence:** `README.md:46`, `README.md:26`, `backend/migrations/002_seed_data.sql:10`, `backend/migrations/002_seed_data.sql:20`, `backend/src/routes/mod.rs:142`, `backend/src/routes/mod.rs:186`, `backend/tests/api/http_support.rs:35`.
- **Manual verification note:** Fresh-install bootstrap must be manually validated after code/doc fix.

#### 1.2 Material deviation from Prompt
- **Conclusion:** **Partial Pass**
- **Rationale:** Most business modules exist, but role semantics deviate materially (Photographer access broader than “assignments tied to jobs”), and check-in scanner requirement is only represented as manual text-entry workflow.
- **Evidence:** `backend/src/handlers/assignments.rs:17`, `backend/src/handlers/reservations.rs:103`, `backend/src/handlers/tickets.rs:59`, `frontend/src/pages/tickets.rs:111`.

### 2. Delivery Completeness

#### 2.1 Core requirement coverage
- **Conclusion:** **Partial Pass**
- **Rationale:** Calendar day/week + filters, reservation conflicts/alternatives, ticket QR display, redeem/undo window/reason, backup/restore, and audit chain are present; however, scanner-based check-in support is not statically implemented.
- **Evidence:** `frontend/src/pages/calendar.rs:74`, `frontend/src/pages/calendar.rs:82`, `backend/src/services/reservation_engine.rs:339`, `frontend/src/components/conflict_explanation.rs:25`, `frontend/src/components/ticket_display.rs:68`, `backend/src/services/ticket_engine.rs:149`, `frontend/src/pages/tickets.rs:111`.

#### 2.2 End-to-end deliverable shape
- **Conclusion:** **Pass (with caveat)**
- **Rationale:** Coherent full-stack structure with backend/frontend/tests/docs exists; caveat is bootstrap inconsistency blocks credible first-run verification.
- **Evidence:** `README.md:3`, `backend/src/main.rs:45`, `frontend/src/app.rs:42`, `backend/tests/api_tests_runner.rs:1`, `frontend/tests/router_spec.rs:1`.

### 3. Engineering and Architecture Quality

#### 3.1 Structure and module decomposition
- **Conclusion:** **Pass**
- **Rationale:** Clear separation across routes, handlers, services, repositories, models, security, and audit modules; frontend split by pages/components/api/state.
- **Evidence:** `backend/src/lib.rs:1`, `backend/src/handlers/mod.rs:1`, `backend/src/services/mod.rs:1`, `frontend/src/pages/mod.rs:1`, `frontend/src/components/mod.rs:1`.

#### 3.2 Maintainability and extensibility
- **Conclusion:** **Partial Pass**
- **Rationale:** Structure is maintainable overall, but permission rules are stored/admin-managed yet not used in authorization decisions (hard-coded role hierarchy), reducing extensibility and creating misleading configurability.
- **Evidence:** `backend/src/routes/mod.rs:65`, `backend/src/handlers/auth.rs:147`, `backend/src/handlers/admin.rs:181`, `backend/src/repositories/permissions.rs:19`.

### 4. Engineering Details and Professionalism

#### 4.1 Error handling/logging/validation/API design
- **Conclusion:** **Partial Pass**
- **Rationale:** Central error envelope and key validations exist; however, state-changing side effect occurs on `GET /api/exports` (audit write) without CSRF boundary, and scanner flow is not implemented.
- **Evidence:** `backend/src/errors/mod.rs:23`, `backend/src/services/reservation_engine.rs:27`, `backend/src/services/uploads.rs:17`, `backend/src/handlers/exports.rs:39`, `backend/src/routes/mod.rs:173`.

#### 4.2 Product vs demo shape
- **Conclusion:** **Partial Pass**
- **Rationale:** Broad real-product surface area and role/API coverage exists; however, frontend tests are largely structural/string checks and do not strongly evidence critical UI behavior.
- **Evidence:** `frontend/tests/module_direct_coverage_spec.rs:3`, `frontend/tests/router_spec.rs:5`, `frontend/tests/frontend_backend_e2e_spec.rs:5`.

### 5. Prompt Understanding and Requirement Fit

#### 5.1 Business understanding and constraint fit
- **Conclusion:** **Partial Pass**
- **Rationale:** Core business intent is mostly implemented, but key requirement semantics are weakened in two places: photographer scope and default masking/encryption expectations for user identifiers.
- **Evidence:** `backend/src/handlers/reservations.rs:103`, `backend/src/handlers/tickets.rs:59`, `backend/migrations/001_initial_schema.sql:16`, `backend/src/handlers/exports.rs:46`, `frontend/src/pages/exports.rs:29`.

### 6. Aesthetics (frontend/full-stack)

#### 6.1 Visual/interaction quality
- **Conclusion:** **Cannot Confirm Statistically**
- **Rationale:** Static code supports a coherent layout/theme and interaction states, but final rendering quality, responsiveness, and interactive polish cannot be proven without execution.
- **Evidence:** `frontend/index.html:10`, `frontend/index.html:48`, `frontend/src/pages/login.rs:60`, `frontend/src/pages/calendar.rs:102`.
- **Manual verification note:** Browser-based review on desktop/mobile required.

## 5. Issues / Suggestions (Severity-Rated)

### Blocker / High

**ISS-001**
- **Severity:** **Blocker**
- **Title:** Fresh-install authentication bootstrap is internally contradictory and likely non-functional
- **Conclusion:** **Fail**
- **Evidence:** `README.md:46`, `README.md:26`, `backend/migrations/002_seed_data.sql:10`, `backend/migrations/002_seed_data.sql:20`, `backend/src/routes/mod.rs:186`, `backend/tests/api/http_support.rs:35`
- **Impact:** Delivery cannot be credibly verified from clean state using documented steps; first admin access path appears blocked.
- **Minimum actionable fix:** Provide a deterministic first-run bootstrap path (or seed active admin with forced rotate), and align README startup verification with actual seeded state.

**ISS-002**
- **Severity:** **High**
- **Title:** Photographer authorization exceeds Prompt scope (access beyond own assignments)
- **Conclusion:** **Fail**
- **Evidence:** `backend/src/handlers/assignments.rs:17`, `backend/src/handlers/reservations.rs:103`, `backend/src/handlers/tickets.rs:59`, `frontend/src/pages/reservations.rs:66`
- **Impact:** Photographers can access store-level reservations/tickets, violating least privilege and Prompt role semantics.
- **Minimum actionable fix:** Restrict photographer routes to assignment-scoped resources only; deny reservations/tickets unless explicitly assignment-linked.

**ISS-003**
- **Severity:** **High**
- **Title:** Sensitive user identifiers are exposed by default in exported/visible UI flows and not encrypted at rest
- **Conclusion:** **Fail**
- **Evidence:** `backend/migrations/001_initial_schema.sql:16`, `backend/src/handlers/exports.rs:46`, `backend/src/models/mod.rs:193`, `frontend/src/pages/exports.rs:29`, `backend/src/repositories/audit.rs:24`
- **Impact:** Violates Prompt requirement for default masking/encryption expectations on sensitive identifiers; increases data disclosure risk.
- **Minimum actionable fix:** Encrypt sensitive user identifiers at rest where required; mask/redact identifiers by default in export/audit payloads rendered in UI.

**ISS-004**
- **Severity:** **High**
- **Title:** Check-in scanner capability is missing; only manual entry is implemented
- **Conclusion:** **Fail**
- **Evidence:** `frontend/src/pages/tickets.rs:111`, `frontend/src/pages/tickets.rs:57`, `frontend/src/components/ticket_display.rs:68`
- **Impact:** Prompt-required “fast scanning or manual entry” is only partially delivered.
- **Minimum actionable fix:** Implement a scanning path (camera/file decode) and integrate it into check-in flow with equivalent validation/feedback.

**ISS-005**
- **Severity:** **High**
- **Title:** Permission management appears non-effective (stored permissions are not used in authorization decisions)
- **Conclusion:** **Fail**
- **Evidence:** `backend/src/routes/mod.rs:65`, `backend/src/handlers/auth.rs:147`, `backend/src/handlers/admin.rs:181`, `backend/src/repositories/permissions.rs:19`
- **Impact:** Admin permission changes may not affect access control, weakening “manage permissions” credibility and increasing policy drift risk.
- **Minimum actionable fix:** Enforce authorization via permission rules (resource/action checks) or clearly scope/remove non-effective permission surfaces.

### Medium / Low

- **Severity:** Medium; **Conclusion:** Partial Pass; **Evidence:** `backend/src/handlers/exports.rs:39`, `backend/src/routes/mod.rs:173`; **Issue:** `GET /api/exports` mutates audit state without CSRF guard; **Minimum fix:** make export a state-changing POST with CSRF or avoid write-on-read side effect.
- **Severity:** Medium; **Conclusion:** Partial Pass; **Evidence:** `frontend/tests/module_direct_coverage_spec.rs:3`, `frontend/tests/router_spec.rs:5`; **Issue:** frontend tests are mostly structural/static string assertions; **Minimum fix:** add behavior-driven component/page tests for login, reservation conflict UX, role gate outcomes, check-in failure branches.
- **Severity:** Low; **Conclusion:** Partial Pass; **Evidence:** `unit_tests/test_auth.rs:1`, `backend/tests/unit_tests_runner.rs:1`; **Issue:** duplicated test trees (`unit_tests`, `API_TESTS`) increase maintenance overhead; **Minimum fix:** consolidate into single canonical test source per suite.

## 6. Security Review Summary

- **authentication entry points:** **Partial Pass**; login/reset implemented with Argon2 verify and recovery-code expiry checks (`backend/src/handlers/auth.rs:10`, `backend/src/auth/password.rs:9`, `backend/src/repositories/recovery_codes.rs:36`), but bootstrap contradiction is unresolved (`backend/migrations/002_seed_data.sql:20`).
- **route-level authorization:** **Pass**; middleware layers enforce auth/staff/ops/admin at route registration (`backend/src/routes/mod.rs:153`, `backend/src/routes/mod.rs:169`, `backend/src/routes/mod.rs:175`, `backend/src/routes/mod.rs:189`).
- **object-level authorization:** **Partial Pass**; customer ticket ownership and store isolation exist (`backend/src/handlers/tickets.rs:46`, `backend/src/handlers/auth.rs:195`), but photographer overreach remains for reservations/tickets (`backend/src/handlers/reservations.rs:103`, `backend/src/handlers/tickets.rs:59`).
- **function-level authorization:** **Partial Pass**; critical mutators check role + CSRF (`backend/src/handlers/vehicles.rs:156`, `backend/src/handlers/admin.rs:188`), but permissions table is not functionally enforced (`backend/src/repositories/permissions.rs:19`).
- **tenant / user data isolation:** **Partial Pass**; store isolation helper is broadly used (`backend/src/handlers/auth.rs:195`, `backend/src/handlers/calendar.rs:18`), but role semantics for photographer violate intended scope.
- **admin / internal / debug protection:** **Pass**; admin routes are middleware-protected and require admin role (`backend/src/routes/mod.rs:178`, `backend/src/routes/mod.rs:189`).

## 7. Tests and Logging Review

- **Unit tests:** **Partial Pass**; numerous backend unit/integration-like tests exist (`backend/tests/integration_tests.rs:21`, `backend/tests/unit_tests_runner.rs:1`), but some frontend tests are non-behavioral/static.
- **API / integration tests:** **Pass (static evidence)**; broad API route coverage with auth/csrf/authz and major modules (`backend/tests/api_tests_runner.rs:7`, `API_TESTS/test_api_authz_matrix.rs:10`, `API_TESTS/test_api_backup.rs:95`).
- **Logging categories / observability:** **Partial Pass**; tracing and structured error logging plus audit logs exist (`backend/src/main.rs:14`, `backend/src/errors/mod.rs:33`, `backend/src/audit/chain.rs:4`).
- **Sensitive-data leakage risk in logs/responses:** **Fail**; exports and audit-related responses can expose raw identifiers in UI flows (`backend/src/handlers/exports.rs:46`, `frontend/src/pages/exports.rs:29`, `backend/src/repositories/audit.rs:24`).

## 8. Test Coverage Assessment (Static Audit)

### 8.1 Test Overview
- Unit tests exist for backend and frontend modules: `backend/tests/unit_tests_runner.rs:1`, `frontend/tests/frontend_utils_spec.rs:1`.
- API/integration-style tests exist via `axum-test`: `backend/Cargo.toml:37`, `backend/tests/api_tests_runner.rs:1`.
- Test entry points: `backend/tests/api_tests_runner.rs:1`, `backend/tests/unit_tests_runner.rs:1`, `backend/tests/integration_tests.rs:1`, `frontend/tests/*.rs`.
- Documentation test command provided: `README.md:52`, `run_tests.sh:21`.

### 8.2 Coverage Mapping Table

| Requirement / Risk Point | Mapped Test Case(s) | Key Assertion / Fixture / Mock | Coverage Assessment | Gap | Minimum Test Addition |
|---|---|---|---|---|---|
| Auth login + token/csrf issuance | `API_TESTS/test_api_auth.rs:9` | token/csrf present (`API_TESTS/test_api_auth.rs:20`) | sufficient | none major | add malformed payload cases |
| 401/403 boundaries for protected routes | `API_TESTS/test_api_authz_matrix.rs:10` | customer forbidden on staff/ops/admin routes (`API_TESTS/test_api_authz_matrix.rs:15`) | basically covered | object-level role-scope gaps not fully tested | add photographer negative tests for reservations/tickets |
| CSRF enforcement on mutating endpoints | `API_TESTS/test_api_reservations.rs:8`, `API_TESTS/test_api_backup.rs:8` | missing CSRF -> 403 | basically covered | not exhaustive across all mutators | add matrix for every POST/PUT route |
| Reservation happy path + conflict reasons/alternatives | `backend/tests/integration_tests.rs:21`, `backend/src/services/reservation_engine.rs:514` | overlap conflict code + alternative slots (`backend/src/services/reservation_engine.rs:540`) | sufficient | none major | add deterministic ordering assertion for alternatives |
| Ticket redeem + double-redeem block + undo reason/window | `backend/src/services/ticket_engine.rs:256`, `backend/src/services/ticket_engine.rs:283`, `API_TESTS/test_api_tickets.rs:8` | duplicate blocked / empty reason rejected | sufficient | scanner-specific flow untested | add UI/API scanner-input path test once implemented |
| Upload validation + duplicate fingerprint | `API_TESTS/test_api_uploads.rs:18`, `backend/src/services/uploads.rs:195` | non-image rejected + duplicate lookup | basically covered | executable-content stripping claims weakly tested | add malicious polyglot payload tests |
| Backup/restore access + roundtrip | `API_TESTS/test_api_backup.rs:95` | create then restore success | basically covered | corruption/wrong-key negative cases incomplete | add wrong-key/corrupt-file tests |
| Audit append-only chain integrity | `backend/src/audit/chain.rs:126`, `backend/tests/integration_tests.rs:145` | update/delete blocked, integrity verification | sufficient | anchor cadence not deeply validated | add explicit 100-entry anchor creation assertion |
| Photographer least-privilege semantics | none meaningful | N/A | missing | Prompt-specific role constraint untested | add tests asserting photographer cannot list reservations or access unrelated tickets |
| Sensitive identifier masking in exports/UI | partial (`API_TESTS/test_api_exports.rs:48`) | checks VIN omission only (`API_TESTS/test_api_exports.rs:63`) | insufficient | user identifiers not asserted masked/redacted | add assertions for `user_id`/actor identifiers redaction |

### 8.3 Security Coverage Audit
- **authentication:** **covered** (login, invalid password, me/logout/reset paths).
- **route authorization:** **covered** at broad role level via authz matrix.
- **object-level authorization:** **partially covered** (customer ticket ownership enforced in code; insufficient tests for photographer overreach).
- **tenant / data isolation:** **partially covered** (store isolation helper tested, but not comprehensively across all endpoints/roles).
- **admin / internal protection:** **covered** for core admin routes and CSRF checks.

### 8.4 Final Coverage Judgment
- **Final coverage judgment:** **Partial Pass**
- Major auth/csrf/conflict/ticket/audit paths have static test evidence, but uncovered high-risk gaps (photographer least privilege and sensitive identifier masking/redaction) mean severe defects could still pass current suites.

## 9. Final Notes
- This report is static-only and evidence-based; runtime claims were intentionally avoided.
- Highest unblock priorities are bootstrap correctness, photographer authorization narrowing, and sensitive identifier handling.
