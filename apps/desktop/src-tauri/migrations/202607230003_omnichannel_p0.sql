-- OMNICHANNEL UNIFIED COMMERCE BENCHMARK MIGRATION (Phase P0)

-- 1. Inventory Items quantity breakdown columns
ALTER TABLE inventory_items ADD COLUMN location_id TEXT;
ALTER TABLE inventory_items ADD COLUMN qty_reserved NUMERIC NOT NULL DEFAULT 0;
ALTER TABLE inventory_items ADD COLUMN qty_in_transit NUMERIC NOT NULL DEFAULT 0;
ALTER TABLE inventory_items ADD COLUMN qty_damaged_or_quarantine NUMERIC NOT NULL DEFAULT 0;

-- Set default location_id to outlet_id for existing rows
UPDATE inventory_items SET location_id = outlet_id WHERE location_id IS NULL;

-- 2. Orders omnichannel channels and separate status fields
ALTER TABLE orders ADD COLUMN channel TEXT NOT NULL DEFAULT 'POS';
ALTER TABLE orders ADD COLUMN external_order_id TEXT;
ALTER TABLE orders ADD COLUMN payment_status TEXT NOT NULL DEFAULT 'paid';
ALTER TABLE orders ADD COLUMN fulfilment_status TEXT NOT NULL DEFAULT 'completed';
ALTER TABLE orders ADD COLUMN fulfilment_location_id TEXT;
ALTER TABLE orders ADD COLUMN reservation_expires_at DATETIME;

-- Set default fulfilment_location_id to outlet_id for existing rows
UPDATE orders SET fulfilment_location_id = outlet_id WHERE fulfilment_location_id IS NULL;

-- 3. Unique index for deduplication of external channel orders
CREATE UNIQUE INDEX IF NOT EXISTS idx_orders_channel_ext_id 
ON orders(merchant_id, channel, external_order_id) 
WHERE external_order_id IS NOT NULL;
