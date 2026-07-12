-- Add order_type and table_number columns to orders table
ALTER TABLE orders ADD COLUMN order_type TEXT NOT NULL DEFAULT 'dine_in';
ALTER TABLE orders ADD COLUMN table_number TEXT;
