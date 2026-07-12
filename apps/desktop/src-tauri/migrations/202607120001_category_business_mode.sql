-- Migration: Add parent_id and business_mode to categories
ALTER TABLE categories ADD COLUMN parent_id TEXT REFERENCES categories(id);
ALTER TABLE categories ADD COLUMN business_mode TEXT;
