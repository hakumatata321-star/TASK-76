# FleetReserve Test Coverage & README Audit

## Project Type Detection

- Declared project type: `fullstack` in `repo/README.md:1`.
- Repository structure is consistent with that declaration: backend Rust API under `repo/backend` and frontend Leptos app under `repo/frontend`.
- Lightweight structural confirmation: backend Axum router in `repo/backend/src/routes/mod.rs:134-200`; frontend Leptos app and routes in `repo/frontend/src/app.rs:41-64` and `repo/frontend/src/routes/mod.rs:4-12`.

## Backend Endpoint Inventory

Source of truth: `repo/backend/src/routes/mod.rs:140-189`.

| # | Endpoint | Route handler evidence |
| # | Endpoint | Route evidence |
|---|---|---|
| 1 | `POST /api/auth/login` | `repo/backend/src/routes/mod.rs:142` |
| 2 | `POST /api/auth/reset-password` | `repo/backend/src/routes/mod.rs:143` |
| 3 | `POST /api/auth/logout` | `repo/backend/src/routes/mod.rs:147` |
| 4 | `GET /api/auth/me` | `repo/backend/src/routes/mod.rs:148` |
| 5 | `POST /api/reservations` | `repo/backend/src/routes/mod.rs:149` |
| 6 | `GET /api/reservations` | `repo/backend/src/routes/mod.rs:150` |
| 7 | `GET /api/tickets/:id` | `repo/backend/src/routes/mod.rs:151` |
| 8 | `GET /api/assignments` | `repo/backend/src/routes/mod.rs:152` |
| 9 | `GET /api/vehicles` | `repo/backend/src/routes/mod.rs:157` |
| 10 | `GET /api/vehicles/:id` | `repo/backend/src/routes/mod.rs:158` |
| 11 | `POST /api/vehicles` | `repo/backend/src/routes/mod.rs:159` |
| 12 | `PUT /api/vehicles/:id/status` | `repo/backend/src/routes/mod.rs:160` |
| 13 | `GET /api/bays` | `repo/backend/src/routes/mod.rs:161` |
| 14 | `POST /api/bays` | `repo/backend/src/routes/mod.rs:162` |
| 15 | `GET /api/stores` | `repo/backend/src/routes/mod.rs:163` |
| 16 | `GET /api/calendar` | `repo/backend/src/routes/mod.rs:164` |
| 17 | `POST /api/tickets/:id/redeem` | `repo/backend/src/routes/mod.rs:165` |
| 18 | `POST /api/tickets/:id/undo` | `repo/backend/src/routes/mod.rs:166` |
| 19 | `POST /api/uploads` | `repo/backend/src/routes/mod.rs:167` |
| 20 | `POST /api/assignments` | `repo/backend/src/routes/mod.rs:168` |
| 21 | `GET /api/exports` | `repo/backend/src/routes/mod.rs:173` |
| 22 | `GET /api/audit` | `repo/backend/src/routes/mod.rs:174` |
| 23 | `GET /api/admin/users` | `repo/backend/src/routes/mod.rs:179` |
| 24 | `POST /api/admin/users` | `repo/backend/src/routes/mod.rs:180` |
| 25 | `GET /api/admin/permissions` | `repo/backend/src/routes/mod.rs:181` |
| 26 | `POST /api/admin/permissions` | `repo/backend/src/routes/mod.rs:182` |
| 27 | `POST /api/admin/permissions/:id` | `repo/backend/src/routes/mod.rs:183` |
| 28 | `PUT /api/admin/users/:id/role` | `repo/backend/src/routes/mod.rs:184` |
| 29 | `PUT /api/admin/users/:id/active` | `repo/backend/src/routes/mod.rs:185` |
| 30 | `POST /api/admin/recovery-codes` | `repo/backend/src/routes/mod.rs:186` |
| 31 | `POST /api/backup` | `repo/backend/src/routes/mod.rs:187` |
| 32 | `POST /api/backup/restore` | `repo/backend/src/routes/mod.rs:188` |
| 1 | `POST /api/auth/login` | `public_routes`, `repo/backend/src/routes/mod.rs:141-143` |
| 2 | `POST /api/auth/reset-password` | `public_routes`, `repo/backend/src/routes/mod.rs:141-143` |
| 3 | `POST /api/auth/logout` | `auth_routes`, `repo/backend/src/routes/mod.rs:146-153` |
| 4 | `GET /api/auth/me` | `auth_routes`, `repo/backend/src/routes/mod.rs:146-153` |
| 5 | `POST /api/reservations` | `auth_routes`, `repo/backend/src/routes/mod.rs:149-150` |
| 6 | `GET /api/reservations` | `auth_routes`, `repo/backend/src/routes/mod.rs:149-150` |
| 7 | `GET /api/tickets/:id` | `auth_routes`, `repo/backend/src/routes/mod.rs:151` |
| 8 | `GET /api/assignments` | `auth_routes`, `repo/backend/src/routes/mod.rs:152` |
| 9 | `GET /api/vehicles` | `staff_routes`, `repo/backend/src/routes/mod.rs:156-169` |
| 10 | `GET /api/vehicles/:id` | `staff_routes`, `repo/backend/src/routes/mod.rs:157-160` |
| 11 | `POST /api/vehicles` | `staff_routes`, `repo/backend/src/routes/mod.rs:157-160` |
| 12 | `PUT /api/vehicles/:id/status` | `staff_routes`, `repo/backend/src/routes/mod.rs:160` |
| 13 | `GET /api/bays` | `staff_routes`, `repo/backend/src/routes/mod.rs:161-162` |
| 14 | `POST /api/bays` | `staff_routes`, `repo/backend/src/routes/mod.rs:161-162` |
| 15 | `GET /api/stores` | `staff_routes`, `repo/backend/src/routes/mod.rs:163` |
| 16 | `GET /api/calendar` | `staff_routes`, `repo/backend/src/routes/mod.rs:164` |
| 17 | `POST /api/tickets/:id/redeem` | `staff_routes`, `repo/backend/src/routes/mod.rs:165` |
| 18 | `POST /api/tickets/:id/undo` | `staff_routes`, `repo/backend/src/routes/mod.rs:166` |
| 19 | `POST /api/uploads` | `staff_routes`, `repo/backend/src/routes/mod.rs:167` |
| 20 | `POST /api/assignments` | `staff_routes`, `repo/backend/src/routes/mod.rs:168` |
| 21 | `GET /api/exports` | `ops_routes`, `repo/backend/src/routes/mod.rs:172-175` |
| 22 | `GET /api/audit` | `ops_routes`, `repo/backend/src/routes/mod.rs:172-175` |
| 23 | `GET /api/admin/users` | `admin_routes`, `repo/backend/src/routes/mod.rs:178-189` |
| 24 | `POST /api/admin/users` | `admin_routes`, `repo/backend/src/routes/mod.rs:179-180` |
| 25 | `GET /api/admin/permissions` | `admin_routes`, `repo/backend/src/routes/mod.rs:181-182` |
| 26 | `POST /api/admin/permissions` | `admin_routes`, `repo/backend/src/routes/mod.rs:181-182` |
| 27 | `POST /api/admin/permissions/:id` | `admin_routes`, `repo/backend/src/routes/mod.rs:183` |
| 28 | `PUT /api/admin/users/:id/role` | `admin_routes`, `repo/backend/src/routes/mod.rs:184` |
| 29 | `PUT /api/admin/users/:id/active` | `admin_routes`, `repo/backend/src/routes/mod.rs:185` |
| 30 | `POST /api/admin/recovery-codes` | `admin_routes`, `repo/backend/src/routes/mod.rs:186` |
| 31 | `POST /api/backup` | `admin_routes`, `repo/backend/src/routes/mod.rs:187` |
| 32 | `POST /api/backup/restore` | `admin_routes`, `repo/backend/src/routes/mod.rs:188` |

Total endpoints: `32`
## API Test Classification

## API Test Mapping Table
### True No-Mock HTTP

HTTP bootstrap evidence: `repo/backend/tests/api/http_helpers.rs::api_server` constructs `TestServer::new(build_router(test_app_state()))`, so requests traverse the real Axum router and handlers with seeded temp DB state. No mocking patterns were found in `repo/backend` or `repo/frontend` source/tests by direct search (`rg -n --glob '!**/target/**' "jest\.mock|vi\.mock|sinon\.stub|mockall|mockito|double::|override_provider|override\(|stub\(|mock\("` returned no matches).
- Test harness uses `axum_test::TestServer` against the real router from `build_router(test_app_state())` in `repo/backend/tests/api/http_helpers.rs:11-13`.
- Test state seeds a real SQLite database and real application state in `repo/backend/tests/api/http_support.rs:24-77`.
- No visible HTTP-layer, controller, service, or provider mocks were found in inspected API test files under `repo/backend/tests/api/*.rs`.
- Classification result: all inspected backend API tests in `repo/backend/tests/api/*.rs` are `true no-mock HTTP` under the provided static definition.

### HTTP with Mocking

- None found by static inspection in `repo/backend/tests/api/*.rs`.

### Non-HTTP (unit/integration without HTTP)

- Backend non-HTTP tests exist in `repo/backend/tests/unit/*.rs` and `repo/backend/tests/integration_tests.rs`.
- Frontend non-browser tests exist in `repo/frontend/tests/frontend_utils_spec.rs`, `repo/frontend/tests/router_spec.rs`, `repo/frontend/tests/module_direct_coverage_spec.rs`, and `repo/frontend/tests/frontend_backend_e2e_spec.rs`.

## API Test Mapping Table

| Endpoint | Covered | Test type | Test files | Evidence |
|---|---|---|---|---|
| `POST /api/auth/login` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_auth.rs` | `api_route_post_login_returns_token_and_csrf`, `api_route_post_login_invalid_password_unauthorized` |
| `POST /api/auth/reset-password` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_auth.rs` | `api_route_post_reset_password_accepts_valid_recovery_code` |
| `POST /api/auth/logout` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_auth.rs` | `api_route_post_logout_requires_csrf`, `api_route_post_logout_invalidates_session_csrf` |
| `GET /api/auth/me` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_auth.rs`, `repo/backend/tests/unit/test_routes_middleware.rs` | `api_route_get_me_returns_user_and_refreshed_token`; `unit_routes_require_auth_on_auth_routes` |
| `POST /api/reservations` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_reservations.rs`, `repo/backend/tests/api/test_api_tickets.rs`, `repo/frontend/tests/frontend_backend_e2e_spec.rs` | `api_route_post_reservation_created_with_csrf`; `api_route_get_ticket_redeem_undo_roundtrip`; `frontend_backend_e2e_login_list_vehicles_and_create_reservation` |
| `GET /api/reservations` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_reservations.rs` | `api_route_get_reservations_lists_for_admin` |
| `GET /api/tickets/:id` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_tickets.rs` | `api_route_get_ticket_redeem_undo_roundtrip` |
| `GET /api/assignments` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_assignments.rs`, `repo/backend/tests/api/test_api_authz_matrix.rs` | `api_route_get_assignments_returns_list_envelope`; `api_authz_photographer_allowed_auth_routes_forbidden_staff_routes` |
| `GET /api/vehicles` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_vehicles.rs`, `repo/backend/tests/api/test_api_authz_matrix.rs`, `repo/backend/tests/unit/test_routes_middleware.rs`, `repo/frontend/tests/frontend_backend_e2e_spec.rs` | `api_route_get_vehicles_returns_masked_row`; `api_authz_merchant_allowed_staff_forbidden_ops_admin`; `unit_routes_allow_staff_on_staff_routes`; `frontend_backend_e2e_login_list_vehicles_and_create_reservation` |
| `GET /api/vehicles/:id` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_vehicles.rs` | `api_route_get_vehicle_by_id_returns_masked_vehicle` |
| `POST /api/vehicles` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_vehicles.rs` | `api_route_post_vehicles_creates_vehicle` |
| `PUT /api/vehicles/:id/status` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_vehicles.rs` | `api_route_put_vehicle_status_updates_status` |
| `GET /api/bays` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_bays.rs` | `api_route_get_bays_returns_empty_for_store_with_no_bays`, `api_route_post_bays_then_list_shows_new_bay` |
| `POST /api/bays` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_bays.rs` | `api_route_post_bays_creates_bay` |
| `GET /api/stores` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_stores.rs` | `api_route_get_stores_lists_active_stores` |
| `GET /api/calendar` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_calendar.rs` | `api_route_get_calendar_day_view_returns_slots_and_assets`, `api_route_get_calendar_week_view_covers_seven_days`, `api_route_get_calendar_invalid_date_rejected`, `api_route_get_calendar_unknown_store_returns_not_found` |
| `POST /api/tickets/:id/redeem` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_tickets.rs` | `api_route_get_ticket_redeem_undo_roundtrip` |
| `POST /api/tickets/:id/undo` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_tickets.rs` | `api_route_get_ticket_redeem_undo_roundtrip` |
| `POST /api/uploads` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_uploads.rs` | `api_route_post_upload_creates_record`, `api_route_post_upload_rejects_non_image_over_http` |
| `POST /api/assignments` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_assignments.rs` | `api_route_post_assignments_creates_assignment`, `api_route_post_assignments_then_list_shows_assignment` |
| `GET /api/exports` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_exports.rs`, `repo/backend/tests/api/test_api_authz_matrix.rs` | `api_route_get_exports_returns_export_envelope`, `api_route_get_exports_filtered_by_store`, `api_authz_ops_allowed_ops_forbidden_admin` |
| `GET /api/audit` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_audit.rs`, `repo/backend/tests/api/test_api_authz_matrix.rs` | `api_route_get_audit_log_returns_entries_envelope`; `api_authz_ops_allowed_ops_forbidden_admin` |
| `GET /api/admin/users` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_admin.rs`, `repo/backend/tests/api/test_api_authz_matrix.rs` | `api_route_get_admin_users_returns_masked_list`, `api_route_get_admin_users_requires_auth`; role-forbidden checks in authz matrix |
| `POST /api/admin/users` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_admin.rs` | `api_route_post_admin_users_rejected_without_csrf`, `api_route_post_admin_users_creates_new_user` |
| `GET /api/admin/permissions` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_admin.rs` | `api_route_get_admin_permissions_returns_seeded_list` |
| `POST /api/admin/permissions` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_admin.rs` | `api_route_post_admin_permissions_upserts_permission`, `api_route_post_admin_permissions_rejected_without_csrf` |
| `POST /api/admin/permissions/:id` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_admin.rs` | `api_route_post_admin_permissions_id_deletes_permission` |
| `PUT /api/admin/users/:id/role` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_admin.rs` | `api_route_put_admin_user_role_updates_role`, `api_route_put_admin_user_role_rejected_without_csrf` |
| `PUT /api/admin/users/:id/active` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_admin.rs` | `api_route_put_admin_user_active_toggles_status` |
| `POST /api/admin/recovery-codes` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_admin.rs`, `repo/backend/tests/api/test_api_auth.rs` | `api_route_post_admin_recovery_code_issued_for_existing_user`, `api_route_post_admin_recovery_code_rejected_for_unknown_user`; password reset flow depends on issued code |
| `POST /api/backup` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_backup.rs` | `api_route_post_backup_rejected_without_csrf`, `api_route_post_backup_creates_encrypted_file`, `api_route_post_backup_restore_roundtrip_succeeds` |
| `POST /api/backup/restore` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_backup.rs` | `api_route_post_backup_restore_rejected_without_csrf`, `api_route_post_backup_restore_fails_for_missing_file`, `api_route_post_backup_restore_roundtrip_succeeds` |
| `POST /api/auth/login` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_auth.rs` | `api_route_post_login_returns_token_and_csrf`, `api_route_post_login_invalid_password_unauthorized`, `repo/backend/tests/api/test_api_auth.rs:8-35` |
| `POST /api/auth/reset-password` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_auth.rs` | `api_route_post_reset_password_accepts_valid_recovery_code`, `repo/backend/tests/api/test_api_auth.rs:106-145` |
| `POST /api/auth/logout` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_auth.rs` | `api_route_post_logout_requires_csrf`, `api_route_post_logout_invalidates_session_csrf`, `repo/backend/tests/api/test_api_auth.rs:55-103` |
| `GET /api/auth/me` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_auth.rs` | `api_route_get_me_returns_user_and_refreshed_token`, `repo/backend/tests/api/test_api_auth.rs:37-53` |
| `POST /api/reservations` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_reservations.rs`, `repo/backend/tests/api/test_api_tickets.rs`, `repo/frontend/tests/frontend_backend_e2e_spec.rs` | `api_route_post_reservation_rejected_without_csrf`, `api_route_post_reservation_created_with_csrf`, `frontend_backend_e2e_login_list_vehicles_and_create_reservation`, `repo/backend/tests/api/test_api_reservations.rs:7-56`, `repo/frontend/tests/frontend_backend_e2e_spec.rs:89-112` |
| `GET /api/reservations` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_reservations.rs` | `api_route_get_reservations_lists_for_admin`, `repo/backend/tests/api/test_api_reservations.rs:58-87` |
| `GET /api/tickets/:id` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_tickets.rs` | `api_route_get_ticket_redeem_undo_roundtrip`, `GET` step before redeem, `repo/backend/tests/api/test_api_tickets.rs:7-58` |
| `GET /api/assignments` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_assignments.rs`, `repo/backend/tests/api/test_api_authz_matrix.rs` | `api_route_get_assignments_returns_list_envelope`, `api_route_get_assignments_requires_auth`, `api_authz_photographer_allowed_auth_routes_forbidden_staff_routes`, `repo/backend/tests/api/test_api_assignments.rs:7-29`, `repo/backend/tests/api/test_api_authz_matrix.rs:31-45` |
| `GET /api/vehicles` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_vehicles.rs`, `repo/backend/tests/api/test_api_authz_matrix.rs`, `repo/frontend/tests/frontend_backend_e2e_spec.rs` | `api_route_get_vehicles_returns_masked_row`, authz matrix checks, `frontend_backend_e2e_login_list_vehicles_and_create_reservation`, `repo/backend/tests/api/test_api_vehicles.rs:7-30`, `repo/frontend/tests/frontend_backend_e2e_spec.rs:73-87` |
| `GET /api/vehicles/:id` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_vehicles.rs` | `api_route_get_vehicle_by_id_returns_masked_vehicle`, `repo/backend/tests/api/test_api_vehicles.rs:32-50` |
| `POST /api/vehicles` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_vehicles.rs` | `api_route_post_vehicles_creates_vehicle`, `repo/backend/tests/api/test_api_vehicles.rs:52-87` |
| `PUT /api/vehicles/:id/status` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_vehicles.rs` | `api_route_put_vehicle_status_updates_status`, `repo/backend/tests/api/test_api_vehicles.rs:89-108` |
| `GET /api/bays` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_bays.rs` | `api_route_get_bays_returns_empty_for_store_with_no_bays`, `api_route_get_bays_requires_auth`, `api_route_post_bays_then_list_shows_new_bay`, `repo/backend/tests/api/test_api_bays.rs:7-27`, `:82-105` |
| `POST /api/bays` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_bays.rs` | `api_route_post_bays_creates_bay`, `api_route_post_bays_rejected_without_csrf`, `repo/backend/tests/api/test_api_bays.rs:32-80` |
| `GET /api/stores` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_stores.rs` | `api_route_get_stores_lists_active_stores`, `repo/backend/tests/api/test_api_stores.rs:6-19` |
| `GET /api/calendar` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_calendar.rs` | `api_route_get_calendar_day_view_returns_slots_and_assets`, `api_route_get_calendar_week_view_covers_seven_days`, `api_route_get_calendar_invalid_date_rejected`, `repo/backend/tests/api/test_api_calendar.rs:6-118` |
| `POST /api/tickets/:id/redeem` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_tickets.rs` | `api_route_get_ticket_redeem_undo_roundtrip`, redeem step, `repo/backend/tests/api/test_api_tickets.rs:41-58` |
| `POST /api/tickets/:id/undo` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_tickets.rs` | `api_route_get_ticket_redeem_undo_roundtrip`, undo step, `repo/backend/tests/api/test_api_tickets.rs:60-78` |
| `POST /api/uploads` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_uploads.rs` | `api_route_post_upload_creates_record`, `api_route_post_upload_rejects_non_image_over_http`, `repo/backend/tests/api/test_api_uploads.rs:18-70` |
| `POST /api/assignments` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_assignments.rs` | `api_route_post_assignments_rejected_without_csrf`, `api_route_post_assignments_creates_assignment`, `repo/backend/tests/api/test_api_assignments.rs:31-85` |
| `GET /api/exports` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_exports.rs`, `repo/backend/tests/api/test_api_authz_matrix.rs` | `api_route_get_exports_returns_export_envelope`, `api_route_get_exports_filtered_by_store`, `api_authz_ops_allowed_ops_forbidden_admin`, `repo/backend/tests/api/test_api_exports.rs:6-72`, `repo/backend/tests/api/test_api_authz_matrix.rs:70-83` |
| `GET /api/audit` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_audit.rs`, `repo/backend/tests/api/test_api_authz_matrix.rs` | `api_route_get_audit_log_returns_entries_envelope`, `api_authz_ops_allowed_ops_forbidden_admin`, `repo/backend/tests/api/test_api_audit.rs:6-23`, `repo/backend/tests/api/test_api_authz_matrix.rs:76-79` |
| `GET /api/admin/users` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_admin.rs`, `repo/backend/tests/api/test_api_authz_matrix.rs` | `api_route_get_admin_users_returns_masked_list`, `api_route_get_admin_users_requires_auth`, authz matrix negative checks, `repo/backend/tests/api/test_api_admin.rs:7-34`, `repo/backend/tests/api/test_api_authz_matrix.rs:25-28` |
| `POST /api/admin/users` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_admin.rs` | `api_route_post_admin_users_rejected_without_csrf`, `api_route_post_admin_users_creates_new_user`, `repo/backend/tests/api/test_api_admin.rs:36-84` |
| `GET /api/admin/permissions` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_admin.rs` | `api_route_get_admin_permissions_returns_seeded_list`, `repo/backend/tests/api/test_api_admin.rs:149-165` |
| `POST /api/admin/permissions` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_admin.rs` | `api_route_post_admin_permissions_upserts_permission`, `api_route_post_admin_permissions_rejected_without_csrf`, `repo/backend/tests/api/test_api_admin.rs:167-207` |
| `POST /api/admin/permissions/:id` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_admin.rs` | `api_route_post_admin_permissions_id_deletes_permission`, `repo/backend/tests/api/test_api_admin.rs:209-250` |
| `PUT /api/admin/users/:id/role` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_admin.rs` | `api_route_put_admin_user_role_updates_role`, `api_route_put_admin_user_role_rejected_without_csrf`, `repo/backend/tests/api/test_api_admin.rs:86-122` |
| `PUT /api/admin/users/:id/active` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_admin.rs` | `api_route_put_admin_user_active_toggles_status`, `repo/backend/tests/api/test_api_admin.rs:124-147` |
| `POST /api/admin/recovery-codes` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_admin.rs`, `repo/backend/tests/api/test_api_auth.rs` | `api_route_post_admin_recovery_code_issued_for_existing_user`, recovery-code issuance step in reset-password flow, `repo/backend/tests/api/test_api_admin.rs:253-300`, `repo/backend/tests/api/test_api_auth.rs:111-128` |
| `POST /api/backup` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_backup.rs` | `api_route_post_backup_rejected_without_csrf`, `api_route_post_backup_creates_encrypted_file`, `repo/backend/tests/api/test_api_backup.rs:7-56` |
| `POST /api/backup/restore` | yes | true no-mock HTTP | `repo/backend/tests/api/test_api_backup.rs` | `api_route_post_backup_restore_rejected_without_csrf`, `api_route_post_backup_restore_fails_for_missing_file`, `api_route_post_backup_restore_roundtrip_succeeds`, `repo/backend/tests/api/test_api_backup.rs:58-122` |

## Coverage Summary

- Total endpoints: `32`
- Endpoints with HTTP tests: `32`
- Endpoints with true no-mock HTTP tests: `32`
- HTTP coverage: `100%`
- True API coverage: `100%`
- HTTP with mocking coverage: `0%`
- HTTP coverage: `32/32 = 100%`
- True API coverage: `32/32 = 100%`

## Unit Test Summary

### Backend Unit Tests

Backend unit/non-HTTP test files:
- Test files:
  - `repo/backend/tests/unit/test_auth.rs`
  - `repo/backend/tests/unit/test_authorization.rs`
  - `repo/backend/tests/unit/test_repositories.rs`
  - `repo/backend/tests/unit/test_reservation_engine.rs`
  - `repo/backend/tests/unit/test_routes_middleware.rs`
  - `repo/backend/tests/unit/test_security.rs`
  - `repo/backend/tests/unit/test_ticket_engine.rs`
  - `repo/backend/tests/unit/test_uploads.rs`
  - Additional non-HTTP integration coverage: `repo/backend/tests/integration_tests.rs`

- `repo/backend/tests/unit/test_auth.rs`
- `repo/backend/tests/unit/test_authorization.rs`
- `repo/backend/tests/unit/test_repositories.rs`
- `repo/backend/tests/unit/test_reservation_engine.rs`
- `repo/backend/tests/unit/test_routes_middleware.rs`
- `repo/backend/tests/unit/test_security.rs`
- `repo/backend/tests/unit/test_ticket_engine.rs`
- `repo/backend/tests/unit/test_uploads.rs`
- `repo/backend/tests/integration_tests.rs`
- Controllers/handlers covered indirectly via HTTP, not as direct unit tests:
  - `auth`, `reservations`, `tickets`, `vehicles`, `bays`, `stores`, `calendar`, `assignments`, `admin`, `backup`, `uploads`, `exports` via API tests in `repo/backend/tests/api/*.rs`.

Backend modules covered by direct evidence:
- Services directly covered:
  - `auth` utilities in `unit_auth_hash_and_verify` and `unit_auth_session_and_csrf`, `repo/backend/tests/unit/test_auth.rs:3-21`
  - `reservation_engine` in `unit_reservation_engine_*`, `repo/backend/tests/unit/test_reservation_engine.rs:17-82`
  - `ticket_engine` in `unit_ticket_engine_*`, `repo/backend/tests/unit/test_ticket_engine.rs:17-48`
  - `uploads::validate_upload` in `unit_uploads_*`, `repo/backend/tests/unit/test_uploads.rs:3-59`

- Auth helpers: `password`, `session`, `csrf` via `unit_auth_hash_and_verify`, `unit_auth_session_and_csrf`
- Authorization logic: `require_role`, `enforce_store_isolation` via `unit_authorization_role_hierarchy`, `unit_authorization_store_isolation`
- Repositories: `users`, `permissions`, `stores`, `vehicles`, `audit`, `recovery_codes` via `test_repositories.rs`
- Services: `reservation_engine`, `ticket_engine`, `uploads` via `test_reservation_engine.rs`, `test_ticket_engine.rs`, `test_uploads.rs`
- Middleware/route guards: `build_router` role gates via `test_routes_middleware.rs`
- Security utilities: masking, encryption, audit chain via `test_security.rs` and `integration_tests.rs`
- Repositories directly covered:
  - `users`, `permissions`, `stores`, `vehicles`, `audit`, `recovery_codes` in `repo/backend/tests/unit/test_repositories.rs:12-95`

Important backend modules not directly unit-tested or only indirectly exercised:
- Auth/guards/middleware covered:
  - `require_role` and `enforce_store_isolation` in `repo/backend/tests/unit/test_authorization.rs:14-23`
  - Route middleware behavior over HTTP in `repo/backend/tests/unit/test_routes_middleware.rs:75-116`

- Handler modules are mostly covered through HTTP tests, not handler-level direct tests: `repo/backend/src/handlers/*.rs`
- `repo/backend/src/backup/mod.rs` lacks direct non-HTTP unit coverage; backup behavior is only asserted through HTTP tests
- `repo/backend/src/services/crypto.rs` has no direct test reference
- `repo/backend/src/security/headers.rs` is not directly asserted
- `repo/backend/src/repositories/backups.rs`, `repo/backend/src/repositories/assignments.rs`, `repo/backend/src/repositories/bays.rs`, `repo/backend/src/repositories/uploads.rs`, `repo/backend/src/repositories/tickets.rs`, `repo/backend/src/repositories/reservations.rs` are exercised indirectly but do not have dedicated repository tests by name
- Important backend modules not directly unit tested:
  - `repo/backend/src/handlers/exports.rs`
  - `repo/backend/src/handlers/calendar.rs`
  - `repo/backend/src/handlers/backup.rs`
  - `repo/backend/src/services/crypto.rs`
  - `repo/backend/src/repositories/assignments.rs`
  - `repo/backend/src/repositories/reservations.rs`
  - `repo/backend/src/repositories/tickets.rs`
  - `repo/backend/src/repositories/uploads.rs`
  - `repo/backend/src/repositories/bays.rs`
  - `repo/backend/src/repositories/backups.rs`

### Frontend Unit Tests

Frontend unit tests: `PRESENT`
- Frontend unit tests: PRESENT

Frontend test files with direct evidence:
- Frameworks/tools detected:
  - Rust built-in test harness via `#[test]` and `#[tokio::test]` in `repo/frontend/tests/*.rs`
  - `axum-test` in `repo/frontend/tests/frontend_backend_e2e_spec.rs:1-3`
  - Playwright in `repo/frontend/tests/e2e/fullstack_ui_e2e.spec.ts:1-27`

- `repo/frontend/tests/frontend_utils_spec.rs`
- `repo/frontend/tests/router_spec.rs`
- `repo/frontend/tests/frontend_backend_e2e_spec.rs`
- `repo/frontend/src/api/types.rs` (`#[cfg(test)]` module)
- `repo/frontend/src/state/auth.rs` (`#[cfg(test)]` module)
- `repo/frontend/src/utils/format.rs` (`#[cfg(test)]` module)
- `repo/frontend/src/utils/time.rs` (`#[cfg(test)]` module)
- Frontend test files:
  - `repo/frontend/tests/frontend_utils_spec.rs`
  - `repo/frontend/tests/router_spec.rs`
  - `repo/frontend/tests/module_direct_coverage_spec.rs`
  - `repo/frontend/tests/frontend_backend_e2e_spec.rs`
  - `repo/frontend/tests/e2e/fullstack_ui_e2e.spec.ts`

Frameworks/tools detected:
- Components/modules covered with direct executable evidence:
  - `utils/time.rs` via `frontend_utils_time_slot_generation_is_deterministic`, `repo/frontend/tests/frontend_utils_spec.rs:6-13`
  - `utils/format.rs` via `frontend_utils_datetime_and_mileage_formatting_are_user_friendly`, `repo/frontend/tests/frontend_utils_spec.rs:15-18`
  - `routes/mod.rs` via `frontend_router_paths_are_absolute_and_unique`, `repo/frontend/tests/router_spec.rs:4-25`
  - Frontend DTO compatibility (`api/types.rs`) via `frontend_backend_e2e_login_list_vehicles_and_create_reservation`, `repo/frontend/tests/frontend_backend_e2e_spec.rs:55-112`

- Rust built-in unit test framework via `#[test]`
- Async Rust tests via `#[tokio::test]`
- `axum-test` for HTTP-layer compatibility test in `repo/frontend/tests/frontend_backend_e2e_spec.rs:1-113`
- Weak or non-behavioral frontend tests:
  - `repo/frontend/tests/module_direct_coverage_spec.rs:1-74` only checks source text with `include_str!`; it does not render pages/components or execute their logic.
  - `repo/frontend/tests/frontend_backend_e2e_spec.rs:55-112` exercises backend endpoints and DTO parsing, not the actual Leptos runtime.

Frontend modules covered by direct evidence:

- Utility formatting/time helpers via `repo/frontend/tests/frontend_utils_spec.rs:6-18`
- Frontend route constants via `repo/frontend/tests/router_spec.rs:4-25`
- Frontend API DTO compatibility with backend payloads via `repo/frontend/tests/frontend_backend_e2e_spec.rs:55-113`
- Frontend auth role hierarchy state helpers via `repo/frontend/src/state/auth.rs:83-108`
- Frontend API type serde roundtrip via `repo/frontend/src/api/types.rs:201-220`

Important frontend components/modules not tested:

- No direct render or interaction tests for Leptos components/pages: `repo/frontend/src/app.rs`, `repo/frontend/src/pages/*.rs`, `repo/frontend/src/components/*.rs`
- No direct tests for `repo/frontend/src/api/client.rs`
- No direct tests for `repo/frontend/src/security/route_guard.rs`
- No direct tests for login form behavior in `repo/frontend/src/pages/login.rs`
- No direct tests for reservation, ticket, vehicle, admin, calendar, export, assignment, dashboard page behavior

Mandatory frontend verdict:
- Important frontend components/modules not meaningfully tested:
  - `repo/frontend/src/app.rs` runtime auth refresh and route composition, `repo/frontend/src/app.rs:8-64`
  - Page behavior for `login`, `dashboard`, `calendar`, `reservations`, `vehicles`, `tickets`, `assignments`, `admin`, `exports` beyond source-string checks, route declarations at `repo/frontend/src/app.rs:49-59`
  - `repo/frontend/src/state/auth.rs`
  - `repo/frontend/src/components/nav.rs`
  - `repo/frontend/src/components/upload_form.rs`
  - `repo/frontend/src/components/calendar_grid.rs`
  - `repo/frontend/src/components/ticket_display.rs`
  - `repo/frontend/src/components/vehicle_card.rs`
  - `repo/frontend/src/security/route_guard.rs`
  - `repo/frontend/src/api/client.rs`

- `Frontend unit tests: PRESENT`
- `CRITICAL GAP`: frontend tests are present but insufficient for a `fullstack` project because they do not render or interact with real frontend components. The current frontend suite is utility/DTO/route-constant heavy, while the user-facing Leptos pages and guards are effectively untested.
- CRITICAL GAP:
  - Project type is `fullstack`, but frontend unit coverage is shallow and concentrated in utilities/routes rather than rendered components, page behavior, auth state, or route-guard behavior. Evidence: direct runtime-style coverage is limited to `repo/frontend/tests/frontend_utils_spec.rs:6-18`, `repo/frontend/tests/router_spec.rs:4-25`, and DTO parsing in `repo/frontend/tests/frontend_backend_e2e_spec.rs:55-112`; component/page tests in `repo/frontend/tests/module_direct_coverage_spec.rs:1-74` are only source-text assertions.

### Cross-Layer Observation

- Testing is materially backend-heavy.
- Backend API coverage is comprehensive, but frontend coverage is shallow and mostly non-UI.
- `repo/frontend/tests/frontend_backend_e2e_spec.rs:55-113` is not a real frontend-to-backend end-to-end test. It exercises backend HTTP endpoints and deserializes them into frontend DTOs, but it does not render frontend components, drive browser state, or validate UI behavior.
- Testing is backend-heavy.
- Backend API coverage is exhaustive and strong through real HTTP.
- Frontend presence exists, but most core UI behavior is not directly exercised. This is an imbalance for a `fullstack` project.

## Tests Check
## API Observability Check

### API Test Classification
- Strong observability examples:
  - `POST /api/auth/login` shows request body and asserts token fields, `repo/backend/tests/api/test_api_auth.rs:8-21`
  - `POST /api/reservations` shows request body and validates reservation/ticket content, `repo/backend/tests/api/test_api_reservations.rs:29-56`
  - `POST /api/uploads` shows multipart input and validates response content, `repo/backend/tests/api/test_api_uploads.rs:18-45`
  - `GET /api/calendar` shows query params and validates structured payload, `repo/backend/tests/api/test_api_calendar.rs:6-118`

1. True no-mock HTTP:
   - All `repo/backend/tests/api/*.rs` files
   - `repo/backend/tests/unit/test_routes_middleware.rs` also uses HTTP requests, but it is narrower middleware-focused coverage than the dedicated API suite
2. HTTP with mocking:
   - None detected
3. Non-HTTP:
   - `repo/backend/tests/unit/*.rs` except the middleware file
   - `repo/backend/tests/integration_tests.rs`
   - Frontend unit tests in `repo/frontend/tests/*.rs` and frontend `#[cfg(test)]` modules
- Weak observability examples:
  - `GET /api/stores` only checks non-empty list, `repo/backend/tests/api/test_api_stores.rs:6-19`
  - `GET /api/audit` only checks envelope shape and total, `repo/backend/tests/api/test_api_audit.rs:6-23`
  - Authorization matrix tests focus on status-only outcomes without response body assertions, `repo/backend/tests/api/test_api_authz_matrix.rs:9-83`

### Mock Detection
- Observability verdict: generally adequate for core CRUD/auth flows, weak for some read-only and authorization endpoints.

- No `jest.mock`, `vi.mock`, `sinon.stub`, DI override patterns, or equivalent mock libraries were found in `repo/backend` or `repo/frontend` source/tests.
- `repo/backend/tests/api/http_support.rs::test_app_state` seeds a temp SQLite database and temp upload directory. This is test scaffolding, not a service/controller mock.
## Tests Check

### API Observability Check
- `run_tests.sh` is Docker-based and therefore acceptable under the stated rule. Evidence: `docker run --rm ... cargo test ...` in `repo/run_tests.sh:16-21`.
- No local package-manager install steps were found in `run_tests.sh`.

- Strong/clear:
  - `repo/backend/tests/api/test_api_auth.rs::api_route_post_login_returns_token_and_csrf`
  - `repo/backend/tests/api/test_api_reservations.rs::api_route_post_reservation_created_with_csrf`
  - `repo/backend/tests/api/test_api_uploads.rs::api_route_post_upload_creates_record`
  - `repo/backend/tests/api/test_api_calendar.rs::api_route_get_calendar_day_view_returns_slots_and_assets`
  - These explicitly show method/path, request payload or query parameters, and response assertions.
- Weak:
  - `repo/backend/tests/api/test_api_authz_matrix.rs::*` is authorization-matrix focused and mostly asserts only status codes, not response payloads.
  - `repo/backend/tests/unit/test_routes_middleware.rs::*` is similarly status-only.
## Test Quality & Sufficiency

### Test Quality & Sufficiency
- Success paths: well represented across auth, reservations, vehicles, bays, assignments, uploads, admin, exports, audit, and backup APIs.
- Failure paths: present for invalid login, missing CSRF, invalid date, missing backup file, non-image upload, and role-based authorization.
- Edge cases: partially present.
  - Reservation overlap and invalid time window are covered in backend unit/integration tests, `repo/backend/tests/unit/test_reservation_engine.rs:43-82`, `repo/backend/tests/integration_tests.rs:113-141`
  - Ticket double-redeem and undo reason validation are covered, `repo/backend/tests/unit/test_ticket_engine.rs:31-40`, `repo/backend/tests/integration_tests.rs:42-111`
- Validation depth: good on backend business rules; weaker on frontend rendering and interaction state.
- Auth/permissions depth: good on backend via middleware and role matrix, `repo/backend/tests/unit/test_routes_middleware.rs:75-116`, `repo/backend/tests/api/test_api_authz_matrix.rs:9-83`
- Integration boundaries: backend HTTP boundary is strong; frontend browser/UI boundary has only one happy-path Playwright flow in `repo/frontend/tests/e2e/fullstack_ui_e2e.spec.ts:3-27`.

Strengths:
## End-to-End Expectations

- Success paths are broadly covered across all backend endpoints.
- Failure-path coverage exists for auth, CSRF, invalid dates, missing files, invalid uploads, overlap conflicts, unknown users, and role authorization.
- Assertions are usually meaningful, not purely status-based, in the main API files.
- `repo/run_tests.sh:16-21` is Docker-based, which satisfies the stated execution preference and does not require local language package installation during test execution.
- For a `fullstack` project, real FE↔BE tests are expected.
- Present evidence:
  - One browser-level Playwright flow exists in `repo/frontend/tests/e2e/fullstack_ui_e2e.spec.ts:3-27`.
  - One backend+frontend DTO compatibility test exists in `repo/frontend/tests/frontend_backend_e2e_spec.rs:55-112`.
- Sufficiency verdict:
  - Partial compensation only.
  - Real browser E2E depth is minimal: login plus a few page navigations, with no creation, mutation, role matrix, upload, check-in, backup/restore, or failure-path UI coverage.

Weaknesses:
## Mock Detection

- Fullstack end-to-end expectation is not met. There is no test that renders the frontend and exercises browser/UI interactions against the backend.
- Frontend component/page coverage is materially absent.
- Some API areas only have narrow happy-path validation. Example: `GET /api/stores` has a single list test in `repo/backend/tests/api/test_api_stores.rs::api_route_get_stores_lists_active_stores`.
- Authorization matrix tests improve route access confidence but are weak for response-shape observability.
- No `jest.mock`, `vi.mock`, `sinon.stub`, dependency override helpers, or visible service/controller mocks were found in inspected backend API tests under `repo/backend/tests/api/*.rs`.
- Real HTTP/router evidence:
  - `TestServer::new(build_router(test_app_state()))`, `repo/backend/tests/api/http_helpers.rs:11-13`
  - Real DB and state bootstrapping in `repo/backend/tests/api/http_support.rs:24-77`
- Non-mock but weak static shortcut detected on frontend:
  - `include_str!` source-string checks in `repo/frontend/tests/module_direct_coverage_spec.rs:3-73`

## Test Coverage Score (0-100)
## Test Coverage Score (0–100)

- Score: `90/100`

## Score Rationale

- `+40`: complete backend endpoint inventory with HTTP coverage on every endpoint
- `+20`: all endpoint coverage is true no-mock HTTP by static evidence
- `+14`: backend service/repository/security unit coverage is substantial
- `+6`: failure-path and validation coverage is present in multiple key flows
- `-10`: no real frontend component/render tests
- `-6`: no true fullstack FE↔BE end-to-end coverage
- `-4`: some endpoints have only narrow or status-only assertions
- `+40`: exact endpoint inventory fully covered by HTTP tests (`32/32`).
- `+20`: all covered endpoints are exercised through the real router and HTTP layer with no visible mocks.
- `+12`: backend unit and non-HTTP integration tests cover key business logic, auth, masking, uploads, audit, and repository behavior.
- `-10`: frontend unit coverage is materially insufficient for a fullstack app; core pages/components/state are not behaviorally tested.
- `-6`: browser E2E coverage is minimal and mostly happy-path navigation.
- `-2`: several API tests have weak observability and status/envelope-only assertions.

## Key Gaps

- Critical gap: frontend unit tests are present but insufficient for a `fullstack` project; user-facing Leptos pages/components are not directly tested.
- Missing real FE↔BE end-to-end tests through the frontend UI.
- Several backend modules are only indirectly covered and do not have dedicated direct tests (`backup`, some repositories, response headers).
- Authorization-only tests are weak for observability because they rarely assert response payload content.
- Fullstack frontend testing is not proportionate to backend coverage. Evidence: `repo/frontend/tests/module_direct_coverage_spec.rs:1-74`, `repo/frontend/tests/e2e/fullstack_ui_e2e.spec.ts:3-27`.
- Core Leptos runtime behavior in `repo/frontend/src/app.rs:8-64` has no direct behavioral test.
- Important frontend state/guard/component modules lack meaningful execution-based tests: `repo/frontend/src/state/auth.rs`, `repo/frontend/src/security/route_guard.rs`, `repo/frontend/src/components/nav.rs`, `repo/frontend/src/components/upload_form.rs`.
- Some read-only backend endpoints have shallow assertions: `repo/backend/tests/api/test_api_stores.rs:6-19`, `repo/backend/tests/api/test_api_audit.rs:6-23`.

## Confidence & Assumptions

- Confidence: `high`
- Confidence: high for backend endpoint inventory and backend HTTP coverage because routes are explicitly declared in one file and mapped directly to test files.
- Confidence: medium-high for frontend sufficiency because frontend test presence is clear, but static inspection cannot confirm runtime stability.
- Assumptions:
  - `axum-test::TestServer` counts as real HTTP-layer route execution because requests traverse the real router and handlers, even though the transport is in-process rather than over a bound external socket.
  - Coverage was determined strictly from visible files under `repo/backend`, `repo/frontend`, `repo/README.md`, `repo/run_tests.sh`, and route definitions in `repo/backend/src/routes/mod.rs`.
  - Only routes declared in `repo/backend/src/routes/mod.rs:140-189` are considered in scope.
  - Generated/build artifacts were ignored.
  - Test files referenced by path are assumed to be part of the intended suite even where runner path wiring appears unusual.

## Test Coverage Verdict

- Verdict: `PASS WITH CRITICAL FRONTEND GAP`

# README Audit

README inspected: `repo/README.md`
## README Location

- README exists at `repo/README.md`.

## Hard Gate Review

### Formatting

- Pass.
- Evidence: clear headings, fenced blocks, tables, and ordered flows throughout `repo/README.md:1-137`.

### Startup Instructions

- Pass for `fullstack`.
- Required `docker-compose up` is present at `repo/README.md:7-13`.

### Access Method

- Pass.
- Frontend URL and backend URL/port are present at `repo/README.md:15-18`.

### Verification Method

- Pass.
- Concrete verification steps via `curl` and UI login are present at `repo/README.md:45-50`.
- Additional UI/manual verification flows are present at `repo/README.md:118-137`.

### Environment Rules (STRICT)

- Pass.
- README no longer requires manual DB/SQL setup steps.
- Evidence: bootstrap section uses API/UI flow only in `repo/README.md`.

### Demo Credentials (Conditional)

- Pass.
- README states authentication is required at `repo/README.md:20-22`.
- Demo credentials for all visible roles are provided at `repo/README.md:24-30`.
- Auth existence is also consistent with protected backend routes in `repo/backend/src/routes/mod.rs:145-189`.

## Engineering Quality

- Tech stack clarity: partial.
  - Backend and frontend URLs are documented, and Docker is the execution path, but the stack is not explicitly explained as Axum + Rust + Leptos in the README. Evidence gap across `repo/README.md:1-137`.
- Architecture explanation: weak.
  - README lists features and endpoints, but does not explain system structure, data flow, persistence, or frontend/backend boundaries.
- Testing instructions: acceptable.
  - `./run_tests.sh` is documented with success/failure indicators at `repo/README.md:52-70`.
- Security/roles: acceptable.
  - Roles and credentials are documented at `repo/README.md:20-30`.
  - Role-specific manual verification is listed at `repo/README.md:118-125`.
- Workflows: acceptable.
  - Startup, verification, UI flow, and manual checks are described at `repo/README.md:7-18`, `45-50`, `118-137`.
- Presentation quality: acceptable.
  - Readable and structured, but overly operational in places and missing concise architecture context.

## High Priority Issues

- None after README remediation.

## Medium Priority Issues

- Minor improvement opportunity: reduce repetition between startup, verification, and UI-flow sections.
- Minor improvement opportunity: keep credential/bootstrap section synchronized with production deployment policy docs.

## Low Priority Issues

- The README is readable and structured, but it duplicates some operational information across quick start, first-use flow, startup indicators, and verification sections.
- “Known Manual Verifications” is useful, but it is not tied to explicit post-start commands or observable artifacts (`repo/README.md:169-173`).
- README omits explicit statement of core technologies even though they are inferable from the repo structure. Evidence gap across `repo/README.md:1-137`, with repo evidence in `repo/backend/src/routes/mod.rs:1-200` and `repo/frontend/Cargo.toml:9-36`.
- README includes a backend API inventory, but it does not indicate which routes are public vs role-protected in a concise access-control summary beyond section labels. Evidence: `repo/README.md:72-116`.

## Hard Gate Failures

- None after README remediation.

## README Verdict

`PASS`
- Verdict: `PASS`

## README Compliance Summary
## Confidence & Assumptions

Hard gates that pass:

- README exists at `repo/README.md`
- Project type declared at the top (`fullstack`)
- Access URLs are provided for frontend and backend (`repo/README.md:26-29`)
- Verification methods are provided for API and UI (`repo/README.md:82-104`)
- The README does not instruct `npm install`, `pip install`, `apt-get`, or manual DB setup

Hard gates that fail:
- Confidence: high.
- Assumptions:
  - Hard-gate evaluation is applied exactly as written in the request.
  - The bootstrap/reset block is treated as part of the README’s required operating instructions, not as optional internal notes.
- None.
## README Final Verdict

- Final verdict: `PASS`