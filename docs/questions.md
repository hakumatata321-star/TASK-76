# FleetReserve Operations Suite - Business Ambiguities

## Q1: Photographer Assignment Creation

Question:
Who creates photographer assignments? The prompt defines Photographers as having access "only to assignments tied to their jobs" but does not specify which role creates or manages those assignments.

My Understanding:
Assignments are created by Merchant/Store Staff or Platform Operations for photographers working at specific stores. Photographers themselves cannot create assignments.

Solution:
Merchant/Store Staff can create and manage photographer assignments for their own store. Platform Operations and Administrators can manage assignments across stores. Photographers have read-only access to their own assignments.

---

## Q2: Undo Supervision Model

Question:
The prompt says supervised "undo" within 2 minutes. Does "supervised" mean a separate supervisor must authenticate, or does any operator with sufficient role perform the undo?

My Understanding:
"Supervised" means the undo action requires an authenticated user with at least Merchant/Store Staff role (not a Customer), and the mandatory reason entry provides the supervision audit trail.

Solution:
Any authenticated user with Merchant/Store Staff or higher role can perform the undo within the 2-minute window. The mandatory reason entry plus audit logging provides the supervision record. A separate supervisor authentication is not required.

---

## Q3: Backup Encryption Key Handling

Question:
The prompt specifies encrypted backup files and encryption at rest using a "locally managed key." How is the key generated, stored, rotated, and recovered?

My Understanding:
A single AES-256 key is generated on first startup if absent, stored in the encryption_keys table (itself protected by filesystem permissions), and used for both at-rest field encryption and backup encryption.

Solution:
On first startup, the system generates an AES-256-GCM key and stores it in the encryption_keys table. The key is loaded into memory at startup. Key rotation is supported by adding a new key and re-encrypting, but is not automated. Key backup/escrow is the Administrator's responsibility and documented as a manual verification item.

---

## Q4: Periodic Hash Anchor Timing

Question:
The prompt requires "periodic hash anchors" for the audit chain but does not define the period (time-based, count-based, or manual trigger).

My Understanding:
Count-based anchoring is more predictable and easier to verify statically. A new anchor is created every 100 audit log entries.

Solution:
A hash anchor is created every 100 audit log entries. Additionally, Administrators can trigger an anchor manually. The anchor stores the cumulative SHA-256 hash of all entries since the previous anchor.

---

## Q5: Conflict Suggestion Tie-Breaking

Question:
When computing the "nearest two alternative time slots," how should ties be broken? For example, if slots at -15min and +15min from the requested time are both free.

My Understanding:
Time-forward slots are preferred since users typically want a later time rather than earlier. Among equidistant slots, earlier times are preferred within the same direction.

Solution:
Alternative slots are computed bidirectionally from the requested time. The two nearest free slots are returned, preferring future slots over past slots when equidistant. Slots must be within the same business day's operating hours.

---

## Q6: Store Isolation for Platform Operations

Question:
Platform Operations can "manage cross-store calendars and exports." Does this mean they can create reservations for any store, or only view and export?

My Understanding:
Platform Operations has read access across all stores for calendars and exports, and can create reservations in any store, but cannot manage roles, permissions, or recovery actions (which are Administrator-only).

Solution:
Platform Operations has full read/write access to reservations, vehicles, bays, and calendars across all stores. They cannot manage users, roles, permissions, or perform recovery/restore operations.

---

## Q7: Bay Capacity Definition

Question:
The prompt mentions "capacity exceeded" as a conflict reason for service bays. What defines capacity - the number of concurrent reservations or a physical limit?

My Understanding:
Each service bay has a numeric capacity field representing the maximum number of concurrent reservations in any given time window.

Solution:
Each service bay has a `capacity` integer (default 1). A conflict is raised when the number of overlapping reservations in the requested time window would exceed this capacity. Most bays have capacity 1 (single vehicle at a time).

---

## Q8: Recovery Code Issuance Limits

Question:
Can an Administrator issue multiple concurrent recovery codes for the same user? Should old codes be invalidated when a new one is issued?

My Understanding:
Only one valid recovery code should exist per user at a time. Issuing a new code invalidates any existing unused codes.

Solution:
Issuing a new recovery code for a user marks all prior unused codes for that user as expired. Only the most recent unused, unexpired code is valid.

---

## Q9: Export Format and Scope

Question:
The prompt mentions exports for Platform Operations but does not specify format (CSV, JSON, PDF) or scope (what data is exported).

My Understanding:
JSON export is the most natural format for a REST API system. Exports cover reservations, vehicles, and calendar data filtered by date range and optionally by store.

Solution:
Exports produce JSON files containing reservations, vehicles, and bay data. Filters include date range, store, and asset type. Export actions are logged in the audit trail.

---

## Q10: Configurable Business Hours Scope

Question:
Business hours are "configurable" with a default of 7:00 AM-7:00 PM. Are they configurable per store or system-wide?

My Understanding:
Per-store configuration is more practical for multi-location scenarios where stores may have different operating hours.

Solution:
Business hours are stored per store with defaults of 07:00-19:00. Each store can have its own business hours. The calendar and reservation engine respect the specific store's configured hours.
