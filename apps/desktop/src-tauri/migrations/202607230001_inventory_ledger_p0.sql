-- INVENTORY LEDGER BENCHMARK MIGRATION (Phase P0)

-- 1. Add ledger columns to stock_movements if they do not exist
ALTER TABLE stock_movements ADD COLUMN stock_before NUMERIC DEFAULT 0;
ALTER TABLE stock_movements ADD COLUMN stock_after NUMERIC DEFAULT 0;
ALTER TABLE stock_movements ADD COLUMN idempotency_key TEXT;
ALTER TABLE stock_movements ADD COLUMN status TEXT NOT NULL DEFAULT 'posted';
ALTER TABLE stock_movements ADD COLUMN reason_code TEXT;

-- 2. Create unique index for idempotency enforcement per outlet
CREATE UNIQUE INDEX IF NOT EXISTS idx_stock_movements_idempotency 
ON stock_movements(outlet_id, idempotency_key) 
WHERE idempotency_key IS NOT NULL;

-- 3. Add default setting for negative stock policy if missing
INSERT OR IGNORE INTO system_settings (key, value, updated_at)
VALUES ('allow_negative_stock', '0', CURRENT_TIMESTAMP);
