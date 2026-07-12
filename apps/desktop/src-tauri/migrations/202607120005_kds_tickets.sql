-- Create kds_tickets table for Kitchen Display System (KDS)
CREATE TABLE IF NOT EXISTS kds_tickets (
    id TEXT PRIMARY KEY,
    reference_id TEXT NOT NULL UNIQUE,
    reference_type TEXT NOT NULL,         -- 'order' or 'draft'
    order_number TEXT NOT NULL,
    table_number TEXT,
    order_type TEXT NOT NULL,             -- 'dine_in' or 'takeaway'
    status TEXT NOT NULL DEFAULT 'pending', -- 'pending', 'cooking', 'done'
    items_json TEXT NOT NULL,             -- JSON string array of items
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_kds_tickets_status ON kds_tickets(status);
