use sqlx::SqlitePool;
use std::time::Duration;
use reqwest::Client;

pub async fn start_sync_worker(pool: SqlitePool) {
    let client = Client::new();
    
    loop {
        // 1. Fetch settings from DB
        let settings = match crate::settings::get_network_settings_internal(&pool).await {
            Ok(s) => s,
            Err(_) => {
                tokio::time::sleep(Duration::from_secs(30)).await;
                continue;
            }
        };

        // 2. Only run sync if enabled
        if settings.cloud_sync_enabled && !settings.cloud_vps_url.is_empty() {
            let _ = process_queue(&pool, &client, &settings.cloud_vps_url, &settings.cloud_vps_token).await;
        }

        // Sleep for 30 seconds before next check
        tokio::time::sleep(Duration::from_secs(30)).await;
    }
}

async fn process_queue(pool: &SqlitePool, _client: &Client, _cloud_url: &str, _token: &str) -> Result<(), String> {
    use sqlx::Row;

    // Fetch up to 50 pending actions
    let records = sqlx::query("SELECT id, action_type, payload_json FROM sync_queue WHERE status = 'PENDING' LIMIT 50")
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    if records.is_empty() {
        return Ok(());
    }

    println!("Found {} items in sync queue, processing...", records.len());

    for r in records {
        let id: String = r.try_get("id").unwrap_or_default();
        let _action_type: String = r.try_get("action_type").unwrap_or_default();
        let _payload: String = r.try_get("payload_json").unwrap_or_default();

        // TODO: In a real implementation, send payload to the Cloud VPS API here.
        // let response = client.post(cloud_url).json(&payload).send().await;

        // Mark as completed in local DB
        let _ = sqlx::query("UPDATE sync_queue SET status = 'COMPLETED' WHERE id = ?")
            .bind(&id)
            .execute(pool)
            .await;
    }

    Ok(())
}
