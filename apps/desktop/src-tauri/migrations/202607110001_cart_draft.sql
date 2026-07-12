-- Create cart_drafts table for auto-save support
CREATE TABLE IF NOT EXISTS cart_drafts (
    id TEXT PRIMARY KEY,
    cart_json TEXT NOT NULL,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
