-- Migration: Revised Security and RBAC Tables and Triggers

-- 1. Alter audit_logs to support cryptographic hash chaining
ALTER TABLE audit_logs ADD COLUMN previous_hash TEXT;
ALTER TABLE audit_logs ADD COLUMN entry_hash TEXT;

-- 2. Create User Sessions table
CREATE TABLE IF NOT EXISTS user_sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    outlet_id TEXT NOT NULL REFERENCES outlets(id) ON DELETE CASCADE,
    device_id TEXT NOT NULL,
    register_id TEXT NOT NULL,
    shift_id TEXT REFERENCES shifts(id),
    login_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at DATETIME NOT NULL,
    last_activity_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    authentication_method TEXT NOT NULL,
    session_token_hash TEXT UNIQUE NOT NULL
);

-- 3. Create User Outlet Roles table (Multi-outlet RBAC assignment)
CREATE TABLE IF NOT EXISTS user_outlet_roles (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    outlet_id TEXT NOT NULL REFERENCES outlets(id) ON DELETE CASCADE,
    role_id TEXT NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    valid_from DATETIME NOT NULL,
    valid_until DATETIME NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('ACTIVE', 'REVOKED')),
    assigned_by TEXT NOT NULL REFERENCES users(id),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 4. Create Authorization Approvals table (Dual Approval support)
CREATE TABLE IF NOT EXISTS authorization_approvals (
    id TEXT PRIMARY KEY,
    authorization_request_id TEXT NOT NULL REFERENCES authorization_requests(id) ON DELETE CASCADE,
    approver_id TEXT NOT NULL REFERENCES users(id),
    approver_role_snapshot TEXT NOT NULL,
    approved_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 5. Immutability Triggers for audit_logs
CREATE TRIGGER IF NOT EXISTS prevent_audit_update
BEFORE UPDATE ON audit_logs
BEGIN
    SELECT RAISE(FAIL, 'Log audit bersifat immutable dan tidak dapat diubah.');
END;

CREATE TRIGGER IF NOT EXISTS prevent_audit_delete
BEFORE DELETE ON audit_logs
BEGIN
    SELECT RAISE(FAIL, 'Log audit bersifat immutable dan tidak dapat dihapus.');
END;

-- 6. Immutability Triggers for cash_movements
CREATE TRIGGER IF NOT EXISTS prevent_cash_movements_update
BEFORE UPDATE ON cash_movements
BEGIN
    SELECT RAISE(FAIL, 'Cash movements bersifat immutable dan tidak dapat diubah.');
END;

CREATE TRIGGER IF NOT EXISTS prevent_cash_movements_delete
BEFORE DELETE ON cash_movements
BEGIN
    SELECT RAISE(FAIL, 'Cash movements bersifat immutable dan tidak dapat dihapus.');
END;
