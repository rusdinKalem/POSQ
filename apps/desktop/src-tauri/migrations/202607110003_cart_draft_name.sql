-- Add name column to cart_drafts to support named hold orders
ALTER TABLE cart_drafts ADD COLUMN name TEXT;
