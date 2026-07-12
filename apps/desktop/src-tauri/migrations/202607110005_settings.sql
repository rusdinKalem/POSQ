CREATE TABLE IF NOT EXISTS system_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Insert default network mode
INSERT OR IGNORE INTO system_settings (key, value) VALUES ('network_mode', 'STANDALONE');
INSERT OR IGNORE INTO system_settings (key, value) VALUES ('master_ip', '');
