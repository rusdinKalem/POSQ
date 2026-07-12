-- Migration: Add min_stock_factor column to products
ALTER TABLE products ADD COLUMN min_stock_factor REAL NOT NULL DEFAULT 0.0;
