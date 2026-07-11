use sqlx::{SqlitePool, Row};
use uuid::Uuid;

pub async fn get_current_user(pool: &SqlitePool) -> Result<Uuid, String> {
    // For MVP: Get the first admin user
    let user_record = sqlx::query("SELECT id FROM users LIMIT 1")
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;
        
    let user = user_record.ok_or("No user found")?;
    let id_str: String = user.get("id");
    let user_id = Uuid::parse_str(&id_str).map_err(|e| e.to_string())?;
    Ok(user_id)
}

pub async fn has_permission(pool: &SqlitePool, user_id: Uuid, permission_key: &str) -> Result<bool, String> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)
        FROM user_roles ur
        JOIN role_permissions rp ON ur.role_id = rp.role_id
        JOIN permissions p ON rp.permission_id = p.id
        WHERE ur.user_id = ? AND p.key = ?
        "#,
    )
    .bind(user_id.to_string())
    .bind(permission_key)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(count > 0)
}
