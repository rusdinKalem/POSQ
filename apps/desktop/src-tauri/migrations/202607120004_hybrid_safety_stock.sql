-- Migration: Add buffer_stock and lead_time_days columns to products
ALTER TABLE products ADD COLUMN buffer_stock REAL NOT NULL DEFAULT 0.0;
ALTER TABLE products ADD COLUMN lead_time_days INTEGER NOT NULL DEFAULT 0;
