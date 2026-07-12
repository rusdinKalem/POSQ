use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::SqlitePool;
use tauri::State;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkSettings {
    pub mode: String,      // "STANDALONE", "MASTER", "CLIENT"
    pub master_ip: String, // e.g. "192.168.1.10:3030"
    pub cloud_sync_enabled: bool,
    pub cloud_vps_url: String,
    pub cloud_vps_token: String,
}

// Internal helper for Rust logic
pub async fn get_network_settings_internal(pool: &SqlitePool) -> Result<NetworkSettings, String> {
    use sqlx::Row;
    
    // Default values
    let mut mode = "STANDALONE".to_string();
    let mut master_ip = "".to_string();
    let mut cloud_sync_enabled = false;
    let mut cloud_vps_url = "".to_string();
    let mut cloud_vps_token = "".to_string();

    let records = sqlx::query("SELECT key, value FROM system_settings")
        .fetch_all(pool)
        .await
        .map_err(|e| format!("DB Error: {}", e))?;

    for r in records {
        let key: String = r.try_get("key").unwrap_or_default();
        let value: String = r.try_get("value").unwrap_or_default();
        
        if key == "network_mode" {
            mode = value;
        } else if key == "master_ip" {
            master_ip = value;
        } else if key == "cloud_sync_enabled" {
            cloud_sync_enabled = value == "true";
        } else if key == "cloud_vps_url" {
            cloud_vps_url = value;
        } else if key == "cloud_vps_token" {
            cloud_vps_token = value;
        }
    }

    Ok(NetworkSettings { 
        mode, 
        master_ip, 
        cloud_sync_enabled, 
        cloud_vps_url, 
        cloud_vps_token 
    })
}

#[tauri::command]
pub async fn get_network_settings(pool: State<'_, SqlitePool>) -> Result<NetworkSettings, String> {
    get_network_settings_internal(pool.inner()).await
}

#[tauri::command]
pub async fn save_network_settings(
    pool: State<'_, SqlitePool>,
    mode: String,
    master_ip: String,
    cloud_sync_enabled: bool,
    cloud_vps_url: String,
    cloud_vps_token: String,
) -> Result<Value, String> {
    sqlx::query("INSERT OR REPLACE INTO system_settings (key, value) VALUES ('network_mode', ?)")
        .bind(&mode)
        .execute(pool.inner())
        .await
        .map_err(|e| format!("DB Error: {}", e))?;

    sqlx::query("INSERT OR REPLACE INTO system_settings (key, value) VALUES ('master_ip', ?)")
        .bind(&master_ip)
        .execute(pool.inner())
        .await
        .map_err(|e| format!("DB Error: {}", e))?;

    let sync_val = if cloud_sync_enabled { "true" } else { "false" };
    sqlx::query("INSERT OR REPLACE INTO system_settings (key, value) VALUES ('cloud_sync_enabled', ?)")
        .bind(sync_val)
        .execute(pool.inner())
        .await
        .map_err(|e| format!("DB Error: {}", e))?;

    sqlx::query("INSERT OR REPLACE INTO system_settings (key, value) VALUES ('cloud_vps_url', ?)")
        .bind(&cloud_vps_url)
        .execute(pool.inner())
        .await
        .map_err(|e| format!("DB Error: {}", e))?;

    sqlx::query("INSERT OR REPLACE INTO system_settings (key, value) VALUES ('cloud_vps_token', ?)")
        .bind(&cloud_vps_token)
        .execute(pool.inner())
        .await
        .map_err(|e| format!("DB Error: {}", e))?;

    Ok(json!({"success": true}))
}
