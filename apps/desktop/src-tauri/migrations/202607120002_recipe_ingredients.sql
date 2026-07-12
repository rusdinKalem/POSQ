-- Migration: Add recipe ingredients and ingredient flag to products
ALTER TABLE products ADD COLUMN is_ingredient BOOLEAN NOT NULL DEFAULT 0;

CREATE TABLE IF NOT EXISTS product_recipes (
    id TEXT PRIMARY KEY,
    product_id TEXT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    ingredient_id TEXT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    qty NUMERIC NOT NULL DEFAULT 0,
    unit TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(product_id, ingredient_id)
);
