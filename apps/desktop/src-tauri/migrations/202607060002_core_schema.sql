-- CORE SCHEMA MIGRATION (M2 Foundation)

-- 1. USERS & ROLES
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    merchant_id TEXT NOT NULL REFERENCES merchants(id),
    outlet_id TEXT REFERENCES outlets(id),
    name TEXT NOT NULL,
    email TEXT,
    pin_hash TEXT,
    password_hash TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS roles (
    id TEXT PRIMARY KEY,
    merchant_id TEXT NOT NULL REFERENCES merchants(id),
    name TEXT NOT NULL,
    system_role BOOLEAN NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS permissions (
    id TEXT PRIMARY KEY,
    key TEXT NOT NULL UNIQUE,
    description TEXT
);

CREATE TABLE IF NOT EXISTS role_permissions (
    role_id TEXT NOT NULL REFERENCES roles(id),
    permission_id TEXT NOT NULL REFERENCES permissions(id),
    UNIQUE(role_id, permission_id)
);

CREATE TABLE IF NOT EXISTS user_roles (
    user_id TEXT NOT NULL REFERENCES users(id),
    role_id TEXT NOT NULL REFERENCES roles(id),
    UNIQUE(user_id, role_id)
);

-- 2. CATALOG
CREATE TABLE IF NOT EXISTS categories (
    id TEXT PRIMARY KEY,
    merchant_id TEXT NOT NULL REFERENCES merchants(id),
    name TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    active BOOLEAN NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS products (
    id TEXT PRIMARY KEY,
    merchant_id TEXT NOT NULL REFERENCES merchants(id),
    category_id TEXT REFERENCES categories(id),
    sku TEXT NOT NULL,
    barcode TEXT,
    name TEXT NOT NULL,
    price INTEGER NOT NULL,
    cost INTEGER,
    track_stock BOOLEAN NOT NULL DEFAULT 1,
    active BOOLEAN NOT NULL DEFAULT 1,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(merchant_id, sku)
);
CREATE INDEX IF NOT EXISTS idx_products_barcode ON products(merchant_id, barcode);
CREATE INDEX IF NOT EXISTS idx_products_name ON products(merchant_id, name);

-- 3. INVENTORY & STOCK MOVEMENTS
CREATE TABLE IF NOT EXISTS inventory_items (
    id TEXT PRIMARY KEY,
    merchant_id TEXT NOT NULL REFERENCES merchants(id),
    outlet_id TEXT NOT NULL REFERENCES outlets(id),
    product_id TEXT NOT NULL REFERENCES products(id),
    qty_on_hand NUMERIC NOT NULL DEFAULT 0,
    min_qty NUMERIC NOT NULL DEFAULT 0,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(outlet_id, product_id)
);

CREATE TABLE IF NOT EXISTS stock_movements (
    id TEXT PRIMARY KEY,
    merchant_id TEXT NOT NULL REFERENCES merchants(id),
    outlet_id TEXT NOT NULL REFERENCES outlets(id),
    product_id TEXT NOT NULL REFERENCES products(id),
    movement_type TEXT NOT NULL, -- sale, refund, stock_in, adjustment, transfer_out, transfer_in, opname
    qty_delta NUMERIC NOT NULL,
    reason TEXT,
    reference_type TEXT,
    reference_id TEXT,
    created_by TEXT REFERENCES users(id),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_stock_movements_outlet_product ON stock_movements(outlet_id, product_id, created_at);
CREATE INDEX IF NOT EXISTS idx_stock_movements_ref ON stock_movements(reference_type, reference_id);

-- 4. SHIFTS
CREATE TABLE IF NOT EXISTS shifts (
    id TEXT PRIMARY KEY,
    merchant_id TEXT NOT NULL REFERENCES merchants(id),
    outlet_id TEXT NOT NULL REFERENCES outlets(id),
    opened_by TEXT NOT NULL REFERENCES users(id),
    closed_by TEXT REFERENCES users(id),
    status TEXT NOT NULL, -- open/closed
    starting_cash INTEGER NOT NULL,
    expected_cash INTEGER,
    counted_cash INTEGER,
    opened_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    closed_at DATETIME
);

-- 5. ORDERS & PAYMENTS
-- Modifying `orders` table from initial schema
ALTER TABLE orders ADD COLUMN shift_id TEXT REFERENCES shifts(id);
ALTER TABLE orders ADD COLUMN subtotal INTEGER NOT NULL DEFAULT 0;
ALTER TABLE orders ADD COLUMN discount_total INTEGER NOT NULL DEFAULT 0;
ALTER TABLE orders ADD COLUMN tax_total INTEGER NOT NULL DEFAULT 0;
ALTER TABLE orders ADD COLUMN service_total INTEGER NOT NULL DEFAULT 0;
ALTER TABLE orders ADD COLUMN paid_total INTEGER NOT NULL DEFAULT 0;
ALTER TABLE orders ADD COLUMN change_total INTEGER NOT NULL DEFAULT 0;
ALTER TABLE orders ADD COLUMN created_by TEXT REFERENCES users(id);
ALTER TABLE orders ADD COLUMN updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP;

CREATE INDEX IF NOT EXISTS idx_orders_outlet_created ON orders(outlet_id, created_at);
CREATE INDEX IF NOT EXISTS idx_orders_shift ON orders(shift_id);
CREATE INDEX IF NOT EXISTS idx_orders_number ON orders(order_number);

CREATE TABLE IF NOT EXISTS order_items (
    id TEXT PRIMARY KEY,
    order_id TEXT NOT NULL REFERENCES orders(id),
    product_id TEXT REFERENCES products(id),
    sku TEXT,
    name TEXT NOT NULL,
    qty NUMERIC NOT NULL,
    unit_price INTEGER NOT NULL,
    discount_total INTEGER NOT NULL DEFAULT 0,
    line_total INTEGER NOT NULL,
    notes TEXT
);

CREATE TABLE IF NOT EXISTS payments (
    id TEXT PRIMARY KEY,
    merchant_id TEXT NOT NULL REFERENCES merchants(id),
    outlet_id TEXT NOT NULL REFERENCES outlets(id),
    order_id TEXT NOT NULL REFERENCES orders(id),
    method TEXT NOT NULL,
    status TEXT NOT NULL,
    amount INTEGER NOT NULL,
    reference TEXT,
    paid_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS refunds (
    id TEXT PRIMARY KEY,
    order_id TEXT NOT NULL REFERENCES orders(id),
    amount INTEGER NOT NULL,
    reason TEXT NOT NULL,
    approved_by TEXT REFERENCES users(id),
    created_by TEXT REFERENCES users(id),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 6. AUDIT LOG
CREATE TABLE IF NOT EXISTS audit_logs (
    id TEXT PRIMARY KEY,
    merchant_id TEXT NOT NULL REFERENCES merchants(id),
    outlet_id TEXT REFERENCES outlets(id),
    actor_user_id TEXT REFERENCES users(id),
    action TEXT NOT NULL,
    target_type TEXT NOT NULL,
    target_id TEXT,
    reason TEXT,
    metadata_json TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
