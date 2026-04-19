# FleetReserve Operations Suite - API Specification

Base URL: `http://localhost:3001/api`

All responses are JSON. All state-changing requests require `X-CSRF-Token` header.
Authentication via `Authorization: Bearer <token>` header.

---

## Authentication

### POST /api/auth/login
**Auth**: None
**Request**:
```json
{"username": "string", "password": "string"}
```
**Response 200**:
```json
{
  "token": "string (HMAC-signed session token)",
  "csrf_token": "string",
  "user": {
    "id": "string (UUID)",
    "username": "string",
    "display_name": "string",
    "role": "Customer|Photographer|MerchantStaff|PlatformOps|Administrator",
    "store_id": "string|null"
  }
}
```
**Errors**: 401 Invalid credentials, 400 Missing fields

### POST /api/auth/logout
**Auth**: Any authenticated user
**Request**: Empty body
**Response 200**: `{"message": "Logged out"}`

### GET /api/auth/me
**Auth**: Any authenticated user
**Response 200**:
```json
{
  "id": "string",
  "username": "string (masked)",
  "display_name": "string",
  "role": "string",
  "store_id": "string|null"
}
```
**Errors**: 401 Not authenticated

---

## Password Recovery

### POST /api/admin/recovery-codes
**Auth**: Administrator only
**Request**:
```json
{"user_id": "string"}
```
**Response 200**:
```json
{
  "code": "string (plaintext, one-time display)",
  "expires_at": "string (ISO-8601, 30 min from now)"
}
```
**Errors**: 403 Not admin, 404 User not found

### POST /api/auth/reset-password
**Auth**: None (uses recovery code)
**Request**:
```json
{
  "username": "string",
  "recovery_code": "string",
  "new_password": "string"
}
```
**Response 200**: `{"message": "Password reset successful"}`
**Errors**: 400 Invalid/expired code, 400 Weak password

---

## Vehicles

### GET /api/vehicles
**Auth**: MerchantStaff+ (filtered by store), PlatformOps/Admin (all stores)
**Query params**: `store_id`, `status`, `page`, `per_page`
**Response 200**:
```json
{
  "vehicles": [{
    "id": "string",
    "vin": "string (masked: *************1234)",
    "license_plate": "string (masked: *****AB)",
    "make": "string",
    "model": "string",
    "trim_level": "string",
    "store_id": "string",
    "mileage_miles": 12345,
    "fuel_or_battery_pct": 85.5,
    "status": "available|reserved|on-rent|in-repair|decommissioned",
    "maintenance_due": "string|null (ISO date)",
    "inspection_due": "string|null (ISO date)",
    "insurance_expiry": "string|null (ISO date)",
    "photos": ["string (upload IDs)"]
  }],
  "total": 42,
  "page": 1,
  "per_page": 20
}
```

### GET /api/vehicles/:id
**Auth**: MerchantStaff+ (own store), PlatformOps/Admin
**Response 200**: Single vehicle object (same schema as list item)
**Errors**: 404, 403 Wrong store

### POST /api/vehicles
**Auth**: MerchantStaff+ (own store), Admin
**Request**:
```json
{
  "vin": "string (plaintext, encrypted at rest)",
  "license_plate": "string (plaintext, encrypted at rest)",
  "make": "string",
  "model": "string",
  "trim_level": "string",
  "store_id": "string",
  "mileage_miles": 0,
  "fuel_or_battery_pct": 100.0,
  "maintenance_due": "string|null",
  "inspection_due": "string|null",
  "insurance_expiry": "string|null"
}
```
**Response 201**: Created vehicle (masked)
**Errors**: 400 Validation, 403 Permission, 409 Duplicate VIN hash

### PUT /api/vehicles/:id/status
**Auth**: Role-dependent (see design.md vehicle lifecycle)
**Request**:
```json
{"status": "available|reserved|on-rent|in-repair|decommissioned"}
```
**Response 200**: Updated vehicle
**Errors**: 400 Invalid transition, 403 Permission, 404, 409 Version conflict

---

## Service Bays

### GET /api/bays
**Auth**: MerchantStaff+ (own store), PlatformOps/Admin
**Query params**: `store_id`, `status`
**Response 200**:
```json
{
  "bays": [{
    "id": "string",
    "store_id": "string",
    "name": "string",
    "bay_type": "string",
    "capacity": 1,
    "status": "string"
  }]
}
```

### POST /api/bays
**Auth**: MerchantStaff+ (own store), Admin
**Request**:
```json
{"name": "string", "store_id": "string", "bay_type": "string", "capacity": 1}
```
**Response 201**: Created bay

---

## Reservations

### POST /api/reservations
**Auth**: Any authenticated user (Customer+)
**Request**:
```json
{
  "asset_type": "vehicle|bay",
  "asset_id": "string",
  "store_id": "string",
  "start_time": "string (ISO-8601)",
  "end_time": "string (ISO-8601)"
}
```
**Response 201** (success):
```json
{
  "reservation": {
    "id": "string",
    "asset_type": "vehicle",
    "asset_id": "string",
    "store_id": "string",
    "user_id": "string",
    "start_time": "string",
    "end_time": "string",
    "status": "confirmed"
  },
  "ticket": {
    "id": "string",
    "ticket_number": "FR-A1B2C3D4",
    "qr_data": "string (JSON)",
    "valid_from": "string",
    "valid_until": "string"
  }
}
```
**Response 409** (conflict):
```json
{
  "conflict": true,
  "reasons": [
    {"code": "overlapping_reservation", "message": "This vehicle has an overlapping reservation from 9:00 AM to 10:30 AM on 03/27/2026."},
    {"code": "expired_insurance", "message": "This vehicle's insurance expired on 03/15/2026."}
  ],
  "alternative_slots": [
    {"start_time": "2026-03-27T10:30:00", "end_time": "2026-03-27T12:00:00"},
    {"start_time": "2026-03-27T14:00:00", "end_time": "2026-03-27T15:30:00"}
  ],
  "alternate_assets": [
    {"id": "string", "asset_type": "vehicle", "make": "Toyota", "model": "Camry", "status": "available"}
  ]
}
```
**Errors**: 400 Invalid times/outside business hours, 403 Permission

### GET /api/reservations
**Auth**: Customer (own), MerchantStaff (own store), PlatformOps/Admin (all)
**Query params**: `user_id`, `store_id`, `asset_type`, `date_from`, `date_to`, `status`
**Response 200**: Array of reservation objects

---

## Calendar

### GET /api/calendar
**Auth**: MerchantStaff+ (own store), PlatformOps/Admin (any store)
**Query params**: `store_id` (required), `date` (ISO date), `view` (day|week), `asset_status` (comma-separated filter)
**Response 200**:
```json
{
  "store_id": "string",
  "business_hours": {"start": "07:00", "end": "19:00"},
  "date": "2026-03-27",
  "view": "day",
  "slots": [{
    "time": "2026-03-27T09:00:00",
    "duration_minutes": 15,
    "reservations": [{
      "id": "string",
      "asset_type": "vehicle",
      "asset_id": "string",
      "asset_name": "2024 Toyota Camry",
      "user_display_name": "string",
      "status": "confirmed"
    }]
  }],
  "assets": [{
    "id": "string",
    "type": "vehicle",
    "name": "2024 Toyota Camry LE",
    "status": "available"
  }]
}
```

---

## Tickets

### GET /api/tickets/:id
**Auth**: Ticket owner (Customer), MerchantStaff+, Admin
**Response 200**:
```json
{
  "id": "string",
  "ticket_number": "FR-A1B2C3D4",
  "reservation_id": "string",
  "qr_data": "string",
  "valid_from": "2026-03-27T09:00:00",
  "valid_until": "2026-03-27T12:00:00",
  "redeemed": false,
  "redeemed_at": null,
  "undone": false
}
```
**Errors**: 404, 403

### POST /api/tickets/:id/redeem
**Auth**: MerchantStaff+
**Request**: Empty body (ticket ID in path)
**Response 200**: `{"message": "Ticket redeemed", "redeemed_at": "string"}`
**Errors**: 400 Already redeemed, 400 Outside validity window, 404

### POST /api/tickets/:id/undo
**Auth**: MerchantStaff+
**Request**:
```json
{"reason": "string (non-empty, mandatory)"}
```
**Response 200**: `{"message": "Redemption undone"}`
**Errors**: 400 Not redeemed, 400 Undo window expired (>2 min), 400 Missing/empty reason, 403, 404

---

## Uploads

### POST /api/uploads
**Auth**: MerchantStaff+
**Content-Type**: multipart/form-data
**Fields**: `file` (binary), `vehicle_id` (optional), `store_id`
**Response 201**:
```json
{
  "id": "string",
  "filename": "string",
  "content_type": "image/jpeg|image/png",
  "size_bytes": 1234567,
  "sha256_fingerprint": "string"
}
```
**Errors**: 400 Invalid type/size/magic bytes, 409 Duplicate fingerprint, 403

---

## Exports

### GET /api/exports
**Auth**: PlatformOps, Administrator
**Query params**: `store_id` (optional), `date_from`, `date_to`, `type` (reservations|vehicles|bays)
**Response 200**: JSON export of requested data
**Errors**: 403

---

## Admin - Roles and Permissions

### GET /api/admin/users
**Auth**: Administrator
**Response 200**: Array of user objects (masked)

### POST /api/admin/users
**Auth**: Administrator
**Request**:
```json
{"username": "string", "password": "string", "display_name": "string", "role": "string", "store_id": "string|null"}
```
**Response 201**: Created user

### PUT /api/admin/users/:id/role
**Auth**: Administrator
**Request**: `{"role": "string"}`
**Response 200**: Updated user

### PUT /api/admin/users/:id/active
**Auth**: Administrator
**Request**: `{"active": true|false}`
**Response 200**: Updated user

---

## Backup and Restore

### POST /api/backup
**Auth**: Administrator only
**Request**: `{"path": "string (local filesystem path)"}`
**Response 200**:
```json
{"id": "string", "filename": "string", "path": "string", "size_bytes": 1234, "sha256": "string"}
```
**Errors**: 403, 500 Backup failed

### POST /api/backup/restore
**Auth**: Administrator only
**Request**: `{"path": "string (path to encrypted backup file)"}`
**Response 200**: `{"message": "Restore successful"}`
**Errors**: 403, 400 Invalid backup, 500 Restore failed

---

## Audit Log

### GET /api/audit
**Auth**: Administrator, PlatformOps (read-only)
**Query params**: `resource_type`, `resource_id`, `actor_id`, `action`, `date_from`, `date_to`, `page`, `per_page`
**Response 200**:
```json
{
  "entries": [{
    "id": 1,
    "timestamp": "string",
    "actor_username": "string (masked)",
    "action": "string",
    "resource_type": "string",
    "resource_id": "string",
    "details": {},
    "chain_valid": true
  }],
  "total": 100
}
```

---

## Photographer Assignments

### GET /api/assignments
**Auth**: Photographer (own only), MerchantStaff+ (own store)
**Response 200**: Array of assignment objects with related vehicle/bay info

### POST /api/assignments
**Auth**: MerchantStaff+ (own store)
**Request**:
```json
{
  "photographer_user_id": "string",
  "store_id": "string",
  "job_description": "string",
  "vehicle_id": "string|null",
  "bay_id": "string|null",
  "start_time": "string",
  "end_time": "string"
}
```
**Response 201**: Created assignment

---

## Common Error Response Format

```json
{
  "error": {
    "code": "string",
    "message": "string (human-readable)"
  }
}
```

HTTP Status Codes:
- 400: Validation error
- 401: Not authenticated
- 403: Forbidden (insufficient role or object access)
- 404: Resource not found
- 409: Conflict (reservation conflict, duplicate upload)
- 500: Internal server error
