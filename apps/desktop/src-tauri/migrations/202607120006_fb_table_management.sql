-- F&B Table & Bill Management Tables

-- 1. Physical Tables
CREATE TABLE IF NOT EXISTS tables (
    id TEXT PRIMARY KEY,
    outlet_id TEXT NOT NULL REFERENCES outlets(id),
    name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'available', -- 'available', 'reserved', 'occupied', 'dirty', 'disabled'
    version INTEGER NOT NULL DEFAULT 1,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(outlet_id, name)
);

-- 2. Dining Sessions
CREATE TABLE IF NOT EXISTS dining_sessions (
    id TEXT PRIMARY KEY,
    outlet_id TEXT NOT NULL REFERENCES outlets(id),
    status TEXT NOT NULL DEFAULT 'active', -- 'active', 'merged', 'closed'
    merged_into_session_id TEXT REFERENCES dining_sessions(id),
    version INTEGER NOT NULL DEFAULT 1,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 3. Session Tables (M-to-M for joint tables)
CREATE TABLE IF NOT EXISTS session_tables (
    id TEXT PRIMARY KEY,
    dining_session_id TEXT NOT NULL REFERENCES dining_sessions(id),
    table_id TEXT NOT NULL REFERENCES tables(id),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(dining_session_id, table_id)
);

-- 4. Bills
CREATE TABLE IF NOT EXISTS bills (
    id TEXT PRIMARY KEY,
    dining_session_id TEXT NOT NULL REFERENCES dining_sessions(id),
    order_id TEXT REFERENCES orders(id),
    reference_id TEXT, -- cart_drafts.id or orders.id
    reference_type TEXT, -- 'draft' or 'order'
    status TEXT NOT NULL DEFAULT 'open', -- 'open', 'merged', 'paid', 'closed', 'void', 'refunded'
    merged_into_bill_id TEXT REFERENCES bills(id),
    bill_number TEXT NOT NULL,
    subtotal INTEGER NOT NULL DEFAULT 0,
    discount_total INTEGER NOT NULL DEFAULT 0,
    tax_total INTEGER NOT NULL DEFAULT 0,
    service_total INTEGER NOT NULL DEFAULT 0,
    rounding_total INTEGER NOT NULL DEFAULT 0,
    grand_total INTEGER NOT NULL DEFAULT 0,
    paid_total INTEGER NOT NULL DEFAULT 0,
    balance_amount INTEGER NOT NULL DEFAULT 0,
    currency TEXT NOT NULL DEFAULT 'IDR',
    tax_profile_id TEXT,
    price_list_id TEXT,
    version INTEGER NOT NULL DEFAULT 1,
    created_by TEXT REFERENCES users(id),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 5. Bill Items
CREATE TABLE IF NOT EXISTS bill_items (
    id TEXT PRIMARY KEY,
    bill_id TEXT NOT NULL REFERENCES bills(id),
    product_id TEXT REFERENCES products(id),
    product_name_snapshot TEXT NOT NULL,
    order_item_id TEXT REFERENCES order_items(id),
    parent_bill_item_id TEXT REFERENCES bill_items(id),
    quantity NUMERIC NOT NULL,
    unit_price INTEGER NOT NULL,
    gross_amount INTEGER NOT NULL,
    item_discount_amount INTEGER NOT NULL DEFAULT 0,
    bill_discount_allocation INTEGER NOT NULL DEFAULT 0,
    taxable_amount INTEGER NOT NULL DEFAULT 0,
    tax_amount INTEGER NOT NULL DEFAULT 0,
    service_charge_amount INTEGER NOT NULL DEFAULT 0,
    rounding_amount INTEGER NOT NULL DEFAULT 0,
    net_amount INTEGER NOT NULL DEFAULT 0,
    seat_number INTEGER,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 6. Payment Allocations (for partially paid split bills)
CREATE TABLE IF NOT EXISTS payment_allocations (
    id TEXT PRIMARY KEY,
    bill_id TEXT NOT NULL REFERENCES bills(id),
    payment_id TEXT NOT NULL REFERENCES payments(id),
    amount INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 7. Table Transfer History
CREATE TABLE IF NOT EXISTS table_transfer_history (
    id TEXT PRIMARY KEY,
    dining_session_id TEXT NOT NULL REFERENCES dining_sessions(id),
    source_table_id TEXT NOT NULL REFERENCES tables(id),
    destination_table_id TEXT NOT NULL REFERENCES tables(id),
    transfer_type TEXT NOT NULL, -- 'move', 'swap'
    performed_by TEXT REFERENCES users(id),
    authorized_by TEXT REFERENCES users(id),
    reason TEXT,
    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 8. Operation Audit Logs
CREATE TABLE IF NOT EXISTS operation_audit_logs (
    id TEXT PRIMARY KEY,
    operation_type TEXT NOT NULL, -- 'split_bill', 'join_bill', 'move_table', 'swap_table', 'join_table'
    idempotency_key TEXT NOT NULL UNIQUE,
    source_entity_ids TEXT, -- JSON array of source IDs
    target_entity_ids TEXT, -- JSON array of target IDs
    before_snapshot TEXT, -- JSON string
    after_snapshot TEXT, -- JSON string
    performed_by TEXT REFERENCES users(id),
    authorized_by TEXT REFERENCES users(id),
    reason TEXT,
    device_id TEXT,
    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 9. Transactional Outbox Events
CREATE TABLE IF NOT EXISTS outbox_events (
    id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL, -- 'BillSplitCompleted', 'BillJoinedCompleted', 'TableMoved', 'TableSwapped', 'TableJoined', 'DiningSessionMerged'
    payload_json TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending', -- 'pending', 'processed', 'failed'
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    processed_at DATETIME
);

-- Seed default tables using first outlet (idempotent)
INSERT OR IGNORE INTO tables (id, outlet_id, name, status, version)
SELECT 'tbl-001', id, 'Meja 01', 'available', 1 FROM outlets LIMIT 1;
INSERT OR IGNORE INTO tables (id, outlet_id, name, status, version)
SELECT 'tbl-002', id, 'Meja 02', 'available', 1 FROM outlets LIMIT 1;
INSERT OR IGNORE INTO tables (id, outlet_id, name, status, version)
SELECT 'tbl-003', id, 'Meja 03', 'available', 1 FROM outlets LIMIT 1;
INSERT OR IGNORE INTO tables (id, outlet_id, name, status, version)
SELECT 'tbl-004', id, 'Meja 04', 'available', 1 FROM outlets LIMIT 1;
INSERT OR IGNORE INTO tables (id, outlet_id, name, status, version)
SELECT 'tbl-005', id, 'Meja 05', 'available', 1 FROM outlets LIMIT 1;
INSERT OR IGNORE INTO tables (id, outlet_id, name, status, version)
SELECT 'tbl-006', id, 'Meja 06', 'available', 1 FROM outlets LIMIT 1;
INSERT OR IGNORE INTO tables (id, outlet_id, name, status, version)
SELECT 'tbl-007', id, 'Meja 07', 'available', 1 FROM outlets LIMIT 1;
INSERT OR IGNORE INTO tables (id, outlet_id, name, status, version)
SELECT 'tbl-008', id, 'Meja 08', 'available', 1 FROM outlets LIMIT 1;

-- Seed permissions
INSERT OR IGNORE INTO permissions (id, key, description) VALUES ('p-fb-001', 'table.move', 'Permission to move dining session between tables');
INSERT OR IGNORE INTO permissions (id, key, description) VALUES ('p-fb-002', 'table.swap', 'Permission to swap dining session tables');
INSERT OR IGNORE INTO permissions (id, key, description) VALUES ('p-fb-003', 'table.join', 'Permission to join tables under one dining session');
INSERT OR IGNORE INTO permissions (id, key, description) VALUES ('p-fb-004', 'bill.split', 'Permission to split a bill');
INSERT OR IGNORE INTO permissions (id, key, description) VALUES ('p-fb-005', 'bill.join', 'Permission to join bills');
INSERT OR IGNORE INTO permissions (id, key, description) VALUES ('p-fb-006', 'bill.split_partially_paid', 'Permission to split partially paid bills');
INSERT OR IGNORE INTO permissions (id, key, description) VALUES ('p-fb-007', 'bill.join_partially_paid', 'Permission to join partially paid bills');
INSERT OR IGNORE INTO permissions (id, key, description) VALUES ('p-fb-008', 'bill.recalculate_promotion', 'Permission to recalculate promotion on bill operations');
INSERT OR IGNORE INTO permissions (id, key, description) VALUES ('p-fb-009', 'bill.override_tax_profile', 'Permission to override tax profile on bill operations');
INSERT OR IGNORE INTO permissions (id, key, description) VALUES ('p-fb-010', 'bill.reopen', 'Permission to reopen closed/paid bills');

-- Link new permissions to 'owner' and 'manager' roles
INSERT OR IGNORE INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.name IN ('owner', 'manager') AND p.key LIKE '%.%';
