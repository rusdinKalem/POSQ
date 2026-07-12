use sqlx::SqlitePool;
use tauri::State;
use uuid::Uuid;
use serde::Serialize;
use sha2::{Sha256, Digest};

#[derive(Serialize)]
pub struct AuditLog {
    pub id: Uuid,
    pub action: String,
    pub target_type: String,
    pub reason: Option<String>,
    pub created_at: String,
    pub previous_hash: Option<String>,
    pub entry_hash: Option<String>,
}

pub async fn log_action(
    conn: &mut sqlx::SqliteConnection,
    merchant_id: String,
    outlet_id: Option<String>,
    user_id: String,
    action: &str,
    target_type: &str,
    target_id: Option<String>,
    reason: Option<&str>,
) -> Result<(), String> {
    use sqlx::Row;

    // 1. Fetch previous hash
    let prev_row: Option<sqlx::sqlite::SqliteRow> = sqlx::query("SELECT entry_hash FROM audit_logs ORDER BY created_at DESC, rowid DESC LIMIT 1")
        .fetch_optional(&mut *conn)
        .await
        .map_err(|e| e.to_string())?;

    let previous_hash = match prev_row {
        Some(row) => row.get::<String, _>("entry_hash"),
        None => "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
    };

    let id = Uuid::new_v4().to_string();
    
    // Hash entry data
    let entry_data = format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{}",
        id,
        merchant_id,
        outlet_id.as_deref().unwrap_or(""),
        user_id,
        action,
        target_type,
        target_id.as_deref().unwrap_or(""),
        reason.as_deref().unwrap_or(""),
        previous_hash
    );
    
    let mut hasher = Sha256::new();
    hasher.update(entry_data.as_bytes());
    let entry_hash = hex::encode(hasher.finalize());

    sqlx::query(
        r#"
        INSERT INTO audit_logs (
            id, merchant_id, outlet_id, actor_user_id, action, target_type, target_id, reason, created_at, previous_hash, entry_hash
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(merchant_id)
    .bind(outlet_id)
    .bind(user_id)
    .bind(action)
    .bind(target_type)
    .bind(target_id)
    .bind(reason)
    .bind(&previous_hash)
    .bind(&entry_hash)
    .execute(conn)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_audit_logs(pool: State<'_, SqlitePool>) -> Result<Vec<AuditLog>, String> {
    use sqlx::Row;
    let user_id = crate::auth::get_current_user(pool.inner()).await?;
    
    let has_perm = crate::auth::has_permission(pool.inner(), user_id, "audit.view").await?;
    if !has_perm {
        return Err("Akses ditolak: Anda tidak memiliki izin untuk melihat audit log".to_string());
    }

    let records = sqlx::query(
        r#"
        SELECT id, action, target_type, reason, created_at, previous_hash, entry_hash
        FROM audit_logs
        ORDER BY created_at DESC
        LIMIT 100
        "#
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let logs = records.into_iter().map(|r| {
        let id_str: String = r.get("id");
        let created_at: String = r.get("created_at");
        AuditLog {
            id: Uuid::parse_str(&id_str).unwrap_or_default(),
            action: r.get("action"),
            target_type: r.get("target_type"),
            reason: r.get("reason"),
            created_at,
            previous_hash: r.get("previous_hash"),
            entry_hash: r.get("entry_hash"),
        }
    }).collect();

    Ok(logs)
}
