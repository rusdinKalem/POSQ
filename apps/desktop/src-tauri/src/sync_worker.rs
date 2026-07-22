use sqlx::{SqlitePool, Row};
use std::time::Duration;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::atomic::{AtomicBool, Ordering};

static IS_PROCESSING_SYNC: AtomicBool = AtomicBool::new(false);

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatusInfo {
    pub mode: String,
    pub cloud_url: String,
    pub cloud_sync_enabled: bool,
    pub is_online: bool,
    pub pending_count: i64,
    pub failed_count: i64,
    pub synced_count: i64,
    pub last_synced_at: Option<String>,
    pub last_error: Option<String>,
}

pub async fn start_sync_worker(pool: SqlitePool) {
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap_or_else(|_| Client::new());

    loop {
        let settings = match crate::settings::get_network_settings_internal(&pool).await {
            Ok(s) => s,
            Err(_) => {
                tokio::time::sleep(Duration::from_secs(30)).await;
                continue;
            }
        };

        if settings.cloud_sync_enabled {
            let _ = process_queue(&pool, &client, &settings.cloud_vps_url, &settings.cloud_vps_token).await;
        }

        tokio::time::sleep(Duration::from_secs(30)).await;
    }
}

pub async fn get_sync_status_internal(pool: &SqlitePool) -> Result<SyncStatusInfo, String> {
    let settings = crate::settings::get_network_settings_internal(pool).await?;

    let pending_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sync_queue WHERE status = 'PENDING'")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    let failed_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sync_queue WHERE status IN ('FAILED', 'DEAD_LETTER')")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    let synced_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sync_queue WHERE status = 'SYNCED'")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    let last_synced_at: Option<String> = sqlx::query_scalar("SELECT synced_at FROM sync_queue WHERE status = 'SYNCED' ORDER BY synced_at DESC LIMIT 1")
        .fetch_optional(pool)
        .await
        .unwrap_or(None);

    let last_error: Option<String> = sqlx::query_scalar("SELECT last_error FROM sync_queue WHERE last_error IS NOT NULL AND status != 'SYNCED' ORDER BY created_at DESC LIMIT 1")
        .fetch_optional(pool)
        .await
        .unwrap_or(None);

    // Connectivity check: if cloud VPS URL is present, perform lightweight HEAD/GET request
    let is_online = if settings.cloud_sync_enabled && !settings.cloud_vps_url.trim().is_empty() {
        let client = Client::builder().timeout(Duration::from_secs(3)).build().unwrap_or_default();
        client.get(&settings.cloud_vps_url).send().await.is_ok()
    } else {
        true // Local-first standalone mode is always healthy locally
    };

    Ok(SyncStatusInfo {
        mode: settings.mode,
        cloud_url: settings.cloud_vps_url,
        cloud_sync_enabled: settings.cloud_sync_enabled,
        is_online,
        pending_count,
        failed_count,
        synced_count,
        last_synced_at,
        last_error,
    })
}

#[tauri::command]
pub async fn get_sync_status(pool: State<'_, SqlitePool>) -> Result<SyncStatusInfo, String> {
    get_sync_status_internal(pool.inner()).await
}

#[tauri::command]
pub async fn check_connectivity(pool: State<'_, SqlitePool>) -> Result<SyncStatusInfo, String> {
    get_sync_status_internal(pool.inner()).await
}

#[tauri::command]
pub async fn trigger_sync(pool: State<'_, SqlitePool>) -> Result<SyncStatusInfo, String> {
    let settings = crate::settings::get_network_settings_internal(pool.inner()).await?;
    let client = Client::builder().timeout(Duration::from_secs(5)).build().unwrap_or_default();
    let _ = process_queue(pool.inner(), &client, &settings.cloud_vps_url, &settings.cloud_vps_token).await;
    get_sync_status_internal(pool.inner()).await
}

pub async fn process_queue(pool: &SqlitePool, client: &Client, cloud_url: &str, token: &str) -> Result<(), String> {
    if IS_PROCESSING_SYNC.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
        return Ok(()); // Another sync task is running
    }

    struct SyncGuard;
    impl Drop for SyncGuard {
        fn drop(&mut self) {
            IS_PROCESSING_SYNC.store(false, Ordering::SeqCst);
        }
    }
    let _guard = SyncGuard;

    let records = sqlx::query(
        r#"
        SELECT id, aggregate_type, aggregate_id, action_type, payload_json, idempotency_key, retry_count
        FROM sync_queue 
        WHERE status = 'PENDING' OR (status = 'FAILED' AND (next_retry_at IS NULL OR next_retry_at <= CURRENT_TIMESTAMP))
        ORDER BY created_at ASC
        LIMIT 50
        "#
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    if records.is_empty() {
        return Ok(());
    }

    for r in records {
        let id: String = r.get("id");
        let action_type: String = r.get("action_type");
        let payload: String = r.get("payload_json");
        let idempotency_key: Option<String> = r.get("idempotency_key");
        let retry_count: i32 = r.get("retry_count");

        // Mark as SYNCING
        let _ = sqlx::query("UPDATE sync_queue SET status = 'SYNCING' WHERE id = ?")
            .bind(&id)
            .execute(pool)
            .await;

        if cloud_url.trim().is_empty() {
            // Local standalone / self-contained mode: Mark event as SYNCED locally
            let _ = sqlx::query("UPDATE sync_queue SET status = 'SYNCED', synced_at = CURRENT_TIMESTAMP WHERE id = ?")
                .bind(&id)
                .execute(pool)
                .await;
            continue;
        }

        // Send request to Cloud VPS API
        let req = client.post(cloud_url)
            .header("Authorization", format!("Bearer {}", token))
            .header("X-Idempotency-Key", idempotency_key.as_deref().unwrap_or(&id))
            .json(&serde_json::json!({
                "actionType": action_type,
                "payload": payload,
            }));

        match req.send().await {
            Ok(resp) if resp.status().is_success() => {
                let _ = sqlx::query("UPDATE sync_queue SET status = 'SYNCED', synced_at = CURRENT_TIMESTAMP, last_error = NULL WHERE id = ?")
                    .bind(&id)
                    .execute(pool)
                    .await;
            }
            Ok(resp) => {
                let err_msg = format!("HTTP error: {}", resp.status());
                let new_retry = retry_count + 1;
                let new_status = if new_retry >= 5 { "DEAD_LETTER" } else { "FAILED" };
                let _ = sqlx::query(
                    "UPDATE sync_queue SET status = ?, retry_count = ?, next_retry_at = datetime('now', '+30 seconds'), last_error = ? WHERE id = ?"
                )
                .bind(new_status)
                .bind(new_retry)
                .bind(err_msg)
                .bind(&id)
                .execute(pool)
                .await;
            }
            Err(e) => {
                let err_msg = e.to_string();
                let new_retry = retry_count + 1;
                let new_status = if new_retry >= 5 { "DEAD_LETTER" } else { "FAILED" };
                let _ = sqlx::query(
                    "UPDATE sync_queue SET status = ?, retry_count = ?, next_retry_at = datetime('now', '+30 seconds'), last_error = ? WHERE id = ?"
                )
                .bind(new_status)
                .bind(new_retry)
                .bind(err_msg)
                .bind(&id)
                .execute(pool)
                .await;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migration::run_migrations;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        run_migrations(&pool).await.unwrap();
        crate::seed::run_seed(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_outbox_queue_status_transitions() {
        let pool = setup_test_db().await;

        let event_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            r#"
            INSERT INTO sync_queue (id, aggregate_type, aggregate_id, action_type, payload_version, payload_json, idempotency_key, status, created_at)
            VALUES (?, 'ORDER', 'ord-123', 'CHECKOUT', 1, '{"test":true}', 'ik_123', 'PENDING', CURRENT_TIMESTAMP)
            "#
        )
        .bind(&event_id)
        .execute(&pool)
        .await
        .unwrap();

        let client = Client::new();
        // Standalone mode processing
        process_queue(&pool, &client, "", "").await.unwrap();

        let status: String = sqlx::query_scalar("SELECT status FROM sync_queue WHERE id = ?")
            .bind(&event_id)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(status, "SYNCED");
    }

    #[tokio::test]
    async fn test_outbox_retry_limit_to_dead_letter() {
        let pool = setup_test_db().await;

        let event_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            r#"
            INSERT INTO sync_queue (id, aggregate_type, aggregate_id, action_type, payload_version, payload_json, idempotency_key, status, retry_count, created_at)
            VALUES (?, 'ORDER', 'ord-456', 'CHECKOUT', 1, '{"test":true}', 'ik_456', 'PENDING', 4, CURRENT_TIMESTAMP)
            "#
        )
        .bind(&event_id)
        .execute(&pool)
        .await
        .unwrap();

        let client = Client::new();
        // Simulate processing pointing to an unresolvable server URL to trigger failure
        process_queue(&pool, &client, "http://127.0.0.1:59999/nonexistent", "dummy_token").await.unwrap();

        let status: String = sqlx::query_scalar("SELECT status FROM sync_queue WHERE id = ?")
            .bind(&event_id)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(status, "DEAD_LETTER");
    }
}
