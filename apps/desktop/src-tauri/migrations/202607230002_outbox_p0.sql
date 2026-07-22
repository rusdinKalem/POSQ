-- OUTBOX SYNC QUEUE BENCHMARK MIGRATION (Phase P0)

-- 1. Add outbox columns to sync_queue if missing
ALTER TABLE sync_queue ADD COLUMN aggregate_type TEXT NOT NULL DEFAULT 'ORDER';
ALTER TABLE sync_queue ADD COLUMN aggregate_id TEXT NOT NULL DEFAULT '';
ALTER TABLE sync_queue ADD COLUMN payload_version INTEGER NOT NULL DEFAULT 1;
ALTER TABLE sync_queue ADD COLUMN idempotency_key TEXT;
ALTER TABLE sync_queue ADD COLUMN retry_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE sync_queue ADD COLUMN next_retry_at DATETIME;
ALTER TABLE sync_queue ADD COLUMN last_error TEXT;
ALTER TABLE sync_queue ADD COLUMN synced_at DATETIME;

-- 2. Create unique index for outbox idempotency
CREATE UNIQUE INDEX IF NOT EXISTS idx_sync_queue_idempotency 
ON sync_queue(idempotency_key) 
WHERE idempotency_key IS NOT NULL;

-- 3. Create index for background worker status query
CREATE INDEX IF NOT EXISTS idx_sync_queue_status_retry 
ON sync_queue(status, next_retry_at);
