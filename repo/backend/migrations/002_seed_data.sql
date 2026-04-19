-- Seed data for FleetReserve Operations Suite

-- Default store
INSERT OR IGNORE INTO stores (id, name, location, business_hours_start, business_hours_end)
VALUES ('store-001', 'Main Yard', '100 Fleet Drive', '07:00', '19:00');

INSERT OR IGNORE INTO stores (id, name, location, business_hours_start, business_hours_end)
VALUES ('store-002', 'East Branch', '200 Service Lane', '08:00', '18:00');

-- Bootstrap admin is seeded inactive so the startup bootstrap_admin() function
-- can activate it with the documented default credentials (admin / FleetReserveHttpTest#2026).
-- On first boot the backend hashes the password at runtime and sets active=1.
-- Change the password via the admin UI or recovery-code flow after first login.
INSERT OR IGNORE INTO users (id, username, password_hash, display_name, role, store_id, active)
VALUES (
    'user-admin-001',
    'admin',
    '$argon2id$v=19$m=19456,t=2,p=1$c2FsdHNhbHRzYWx0c2FsdA$YzFkMGU1ZjBhNTJkNGJhZDk5ZTEyYjVhNmQzYzRhMjE',
    'System Administrator',
    'Administrator',
    NULL,
    0
);

-- Seed permissions
-- Customer permissions
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-c-01', 'Customer', 'reservation', 'create');
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-c-02', 'Customer', 'reservation', 'read_own');
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-c-03', 'Customer', 'ticket', 'read_own');
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-c-04', 'Customer', 'calendar', 'read');

-- Photographer permissions
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-p-01', 'Photographer', 'assignment', 'read_own');
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-p-02', 'Photographer', 'vehicle', 'read_assigned');
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-p-03', 'Photographer', 'bay', 'read_assigned');
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-p-04', 'Photographer', 'reservation', 'read_assigned');
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-p-05', 'Photographer', 'ticket', 'read_assigned');

-- MerchantStaff permissions
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-m-01', 'MerchantStaff', 'vehicle', 'read_store');
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-m-02', 'MerchantStaff', 'vehicle', 'create');
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-m-03', 'MerchantStaff', 'vehicle', 'update_store');
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-m-04', 'MerchantStaff', 'vehicle', 'status_transition');
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-m-05', 'MerchantStaff', 'bay', 'read_store');
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-m-06', 'MerchantStaff', 'bay', 'create');
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-m-07', 'MerchantStaff', 'reservation', 'read_store');
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-m-08', 'MerchantStaff', 'reservation', 'create');
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-m-09', 'MerchantStaff', 'ticket', 'read_store');
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-m-10', 'MerchantStaff', 'ticket', 'redeem');
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-m-11', 'MerchantStaff', 'ticket', 'undo');
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-m-12', 'MerchantStaff', 'calendar', 'read_store');
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-m-13', 'MerchantStaff', 'upload', 'create');
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-m-14', 'MerchantStaff', 'assignment', 'manage_store');

-- PlatformOps permissions
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-o-01', 'PlatformOps', 'vehicle', 'read_all');
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-o-02', 'PlatformOps', 'bay', 'read_all');
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-o-03', 'PlatformOps', 'reservation', 'read_all');
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-o-04', 'PlatformOps', 'reservation', 'create');
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-o-05', 'PlatformOps', 'calendar', 'read_all');
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-o-06', 'PlatformOps', 'export', 'create');
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-o-07', 'PlatformOps', 'ticket', 'read_all');
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-o-08', 'PlatformOps', 'audit', 'read');
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-o-09', 'PlatformOps', 'assignment', 'read_all');

-- Administrator permissions (inherits all + admin-specific)
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-a-01', 'Administrator', 'user', 'manage');
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-a-02', 'Administrator', 'role', 'manage');
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-a-03', 'Administrator', 'permission', 'manage');
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-a-04', 'Administrator', 'recovery_code', 'issue');
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-a-05', 'Administrator', 'backup', 'create');
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-a-06', 'Administrator', 'backup', 'restore');
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-a-07', 'Administrator', 'vehicle', 'decommission');
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-a-08', 'Administrator', 'audit', 'read');
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-a-09', 'Administrator', 'audit', 'anchor');
INSERT OR IGNORE INTO permissions (id, role, resource, action) VALUES ('perm-a-10', 'Administrator', 'all', 'all');
