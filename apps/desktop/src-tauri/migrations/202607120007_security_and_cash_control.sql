-- Security and Cash Control Migrations

-- 1. Users table updates
ALTER TABLE users ADD COLUMN pin_hash_v2 TEXT;
ALTER TABLE users ADD COLUMN failed_login_attempts INTEGER NOT NULL DEFAULT 0;
ALTER TABLE users ADD COLUMN locked_until DATETIME;

-- 2. Authorization Policies
CREATE TABLE IF NOT EXISTS authorization_policies (
    id TEXT PRIMARY KEY,
    outlet_id TEXT REFERENCES outlets(id),
    action_type TEXT NOT NULL, -- e.g., 'transaction.void', 'discount.manual', 'cash.cash_out'
    min_amount INTEGER NOT NULL DEFAULT 0,
    max_amount INTEGER,
    policy_decision TEXT NOT NULL, -- 'ALLOW', 'REQUIRE_SUPERVISOR', 'REQUIRE_MANAGER', 'DENY'
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 3. Authorization Grants
CREATE TABLE IF NOT EXISTS authorization_grants (
    id TEXT PRIMARY KEY,
    action_type TEXT NOT NULL,
    resource_type TEXT,
    resource_id TEXT,
    cashier_id TEXT REFERENCES users(id),
    supervisor_id TEXT REFERENCES users(id),
    outlet_id TEXT REFERENCES outlets(id),
    shift_id TEXT REFERENCES shifts(id),
    approved_amount INTEGER,
    reason_code TEXT,
    issued_at DATETIME NOT NULL,
    expires_at DATETIME NOT NULL,
    used_at DATETIME,
    status TEXT NOT NULL -- 'PENDING', 'APPROVED', 'REJECTED', 'EXPIRED', 'USED', 'REVOKED'
);

-- 4. Cash Drawers
CREATE TABLE IF NOT EXISTS cash_drawers (
    id TEXT PRIMARY KEY,
    outlet_id TEXT NOT NULL REFERENCES outlets(id),
    name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'available', -- 'available', 'in_use', 'disabled'
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 5. Cash Drawer Sessions
CREATE TABLE IF NOT EXISTS cash_drawer_sessions (
    id TEXT PRIMARY KEY,
    cash_drawer_id TEXT NOT NULL REFERENCES cash_drawers(id),
    shift_id TEXT NOT NULL REFERENCES shifts(id),
    cashier_id TEXT NOT NULL REFERENCES users(id),
    opened_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    closed_at DATETIME,
    status TEXT NOT NULL -- 'open', 'closed'
);

-- 6. Cash Movements (Append-Only Ledger)
CREATE TABLE IF NOT EXISTS cash_movements (
    id TEXT PRIMARY KEY,
    movement_number TEXT NOT NULL UNIQUE,
    outlet_id TEXT NOT NULL REFERENCES outlets(id),
    shift_id TEXT NOT NULL REFERENCES shifts(id),
    cash_drawer_id TEXT NOT NULL REFERENCES cash_drawers(id),
    type TEXT NOT NULL, -- 'STARTING_CASH', 'CASH_SALE', 'CASH_REFUND', 'CASH_IN', 'CASH_OUT', 'SAFE_DROP', 'REVERSAL', 'CLOSING_VARIANCE'
    direction TEXT NOT NULL, -- 'IN', 'OUT'
    amount INTEGER NOT NULL,
    reason_code TEXT,
    notes TEXT,
    transaction_id TEXT,
    parent_movement_id TEXT REFERENCES cash_movements(id),
    performed_by TEXT NOT NULL REFERENCES users(id),
    approved_by TEXT REFERENCES users(id),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    version INTEGER NOT NULL DEFAULT 1
);

-- 7. Cash Count Sessions (Blind closing recount history)
CREATE TABLE IF NOT EXISTS cash_count_sessions (
    id TEXT PRIMARY KEY,
    shift_id TEXT NOT NULL REFERENCES shifts(id),
    attempt_number INTEGER NOT NULL,
    expected_cash INTEGER NOT NULL,
    actual_cash INTEGER NOT NULL,
    variance INTEGER NOT NULL,
    performed_by TEXT NOT NULL REFERENCES users(id),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 8. Cash Count Details (closing denomination count)
CREATE TABLE IF NOT EXISTS cash_count_details (
    id TEXT PRIMARY KEY,
    cash_count_session_id TEXT NOT NULL REFERENCES cash_count_sessions(id),
    denomination INTEGER NOT NULL,
    quantity INTEGER NOT NULL,
    total INTEGER NOT NULL
);

-- 9. Fraud Alerts
CREATE TABLE IF NOT EXISTS fraud_alerts (
    id TEXT PRIMARY KEY,
    rule_id TEXT NOT NULL,
    severity TEXT NOT NULL, -- 'INFO', 'LOW', 'MEDIUM', 'HIGH', 'CRITICAL'
    user_id TEXT REFERENCES users(id),
    supervisor_id TEXT REFERENCES users(id),
    outlet_id TEXT REFERENCES outlets(id),
    shift_id TEXT REFERENCES shifts(id),
    transaction_id TEXT,
    supporting_data TEXT, -- JSON
    status TEXT NOT NULL DEFAULT 'OPEN', -- 'OPEN', 'REVIEWING', 'FALSE_POSITIVE', 'CONFIRMED', 'RESOLVED'
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 10. Idempotency Records
CREATE TABLE IF NOT EXISTS idempotency_records (
    idempotency_key TEXT PRIMARY KEY,
    response_json TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 11. Seed default cash drawer using the first outlet
INSERT OR IGNORE INTO cash_drawers (id, outlet_id, name, status)
SELECT 'drw-default', id, 'Laci Utama', 'available' FROM outlets LIMIT 1;

-- 12. Seed default authorization policies
-- Manual Discount <= 10% -> ALLOW (No supervisor needed)
INSERT OR IGNORE INTO authorization_policies (id, outlet_id, action_type, min_amount, max_amount, policy_decision)
SELECT 'pol-disc-low', id, 'discount.manual', 0, 10, 'ALLOW' FROM outlets LIMIT 1;

-- Manual Discount > 10% and <= 25% -> REQUIRE_SUPERVISOR
INSERT OR IGNORE INTO authorization_policies (id, outlet_id, action_type, min_amount, max_amount, policy_decision)
SELECT 'pol-disc-mid', id, 'discount.manual', 11, 25, 'REQUIRE_SUPERVISOR' FROM outlets LIMIT 1;

-- Manual Discount > 25% -> REQUIRE_MANAGER
INSERT OR IGNORE INTO authorization_policies (id, outlet_id, action_type, min_amount, max_amount, policy_decision)
SELECT 'pol-disc-high', id, 'discount.manual', 26, 100, 'REQUIRE_MANAGER' FROM outlets LIMIT 1;

-- Void transaction -> REQUIRE_SUPERVISOR
INSERT OR IGNORE INTO authorization_policies (id, outlet_id, action_type, min_amount, max_amount, policy_decision)
SELECT 'pol-void', id, 'transaction.void', 0, NULL, 'REQUIRE_SUPERVISOR' FROM outlets LIMIT 1;

-- Cash Out (Petty Cash) > Rp 200k -> REQUIRE_SUPERVISOR
INSERT OR IGNORE INTO authorization_policies (id, outlet_id, action_type, min_amount, max_amount, policy_decision)
SELECT 'pol-cashout-mid', id, 'cash.cash_out', 200001, 1000000, 'REQUIRE_SUPERVISOR' FROM outlets LIMIT 1;

-- Cash Out (Petty Cash) > Rp 1M -> REQUIRE_MANAGER
INSERT OR IGNORE INTO authorization_policies (id, outlet_id, action_type, min_amount, max_amount, policy_decision)
SELECT 'pol-cashout-high', id, 'cash.cash_out', 1000001, NULL, 'REQUIRE_MANAGER' FROM outlets LIMIT 1;

-- Seed default permissions
INSERT OR IGNORE INTO permissions (id, key, description) VALUES ('p-sec-001', 'discount.manual', 'Permission to apply manual discount');
INSERT OR IGNORE INTO permissions (id, key, description) VALUES ('p-sec-002', 'cash.cash_in', 'Permission to perform Cash In');
INSERT OR IGNORE INTO permissions (id, key, description) VALUES ('p-sec-003', 'cash.cash_out', 'Permission to perform Cash Out / Petty Cash');
INSERT OR IGNORE INTO permissions (id, key, description) VALUES ('p-sec-004', 'cash.safe_drop', 'Permission to perform Safe Drop');
INSERT OR IGNORE INTO permissions (id, key, description) VALUES ('p-sec-005', 'cash.drawer_open_no_sale', 'Permission to open cash drawer without sales');
INSERT OR IGNORE INTO permissions (id, key, description) VALUES ('p-sec-006', 'shift.force_close', 'Permission to force close other shifts');
INSERT OR IGNORE INTO permissions (id, key, description) VALUES ('p-sec-007', 'role.manage', 'Permission to manage roles and user assignments');

-- Link default permissions to 'owner' and 'manager' roles
INSERT OR IGNORE INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.name IN ('owner', 'manager') AND p.key IN ('discount.manual', 'cash.cash_in', 'cash.cash_out', 'cash.safe_drop', 'cash.drawer_open_no_sale', 'shift.force_close', 'role.manage');
