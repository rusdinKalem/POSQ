use sqlx::{SqlitePool, Row};
use uuid::Uuid;
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2
};
use rand_core::OsRng;
use sha2::{Sha256, Digest};
use chrono::{Utc, Duration};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserSession {
    pub id: String,
    pub user_id: String,
    pub outlet_id: String,
    pub device_id: String,
    pub register_id: String,
    pub shift_id: Option<String>,
    pub login_at: String,
    pub expires_at: String,
    pub last_activity_at: String,
    pub authentication_method: String,
    pub session_token_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserSessionDTO {
    pub session_id: String,
    pub session_token: String,
    pub user_id: String,
    pub user_name: String,
    pub outlet_id: String,
    pub roles: Vec<String>,
}

pub fn hash_pin_argon2(pin: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hashed = argon2
        .hash_password(pin.as_bytes(), &salt)
        .map_err(|e| e.to_string())?
        .to_string();
    Ok(hashed)
}

pub fn verify_pin_argon2(pin: &str, stored_hash: &str) -> Result<bool, String> {
    let parsed_hash = PasswordHash::new(stored_hash).map_err(|e| e.to_string())?;
    let argon2 = Argon2::default();
    Ok(argon2.verify_password(pin.as_bytes(), &parsed_hash).is_ok())
}

pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

pub async fn get_current_user(pool: &SqlitePool) -> Result<Uuid, String> {
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let session_row = sqlx::query(
        r#"
        SELECT user_id 
        FROM user_sessions 
        WHERE expires_at > ?
        ORDER BY last_activity_at DESC LIMIT 1
        "#
    )
    .bind(&now)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(row) = session_row {
        let user_id_str: String = row.get("user_id");
        let user_uuid = Uuid::parse_str(&user_id_str).map_err(|e| e.to_string())?;
        
        // Update last activity
        sqlx::query("UPDATE user_sessions SET last_activity_at = CURRENT_TIMESTAMP WHERE user_id = ?")
            .bind(&user_id_str)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
            
        Ok(user_uuid)
    } else {
        // Fallback for tests if no session exists yet
        #[cfg(test)]
        {
            if let Some(row) = sqlx::query("SELECT id FROM users LIMIT 1").fetch_optional(pool).await.unwrap_or(None) {
                let user_id_str: String = row.get("id");
                return Uuid::parse_str(&user_id_str).map_err(|e| e.to_string());
            }
        }
        Err("User belum login atau session telah berakhir. Silakan login kembali.".to_string())
    }
}

pub async fn get_current_outlet(pool: &SqlitePool) -> Result<Uuid, String> {
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let session_row = sqlx::query(
        r#"
        SELECT outlet_id 
        FROM user_sessions 
        WHERE expires_at > ?
        ORDER BY last_activity_at DESC LIMIT 1
        "#
    )
    .bind(&now)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(row) = session_row {
        let outlet_id_str: String = row.get("outlet_id");
        Uuid::parse_str(&outlet_id_str).map_err(|e| e.to_string())
    } else {
        #[cfg(test)]
        {
            if let Some(row) = sqlx::query("SELECT id FROM outlets LIMIT 1").fetch_optional(pool).await.unwrap_or(None) {
                let outlet_id_str: String = row.get("id");
                return Uuid::parse_str(&outlet_id_str).map_err(|e| e.to_string());
            }
        }
        Err("User belum login atau session telah berakhir.".to_string())
    }
}

pub async fn has_permission(pool: &SqlitePool, user_id: Uuid, permission_key: &str) -> Result<bool, String> {
    let outlet_id = match get_current_outlet(pool).await {
        Ok(oid) => oid.to_string(),
        Err(_) => "".to_string(),
    };

    // Check outlet-specific roles first
    let count_outlet: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)
        FROM user_outlet_roles uor
        JOIN role_permissions rp ON uor.role_id = rp.role_id
        JOIN permissions p ON rp.permission_id = p.id
        WHERE uor.user_id = ? AND uor.outlet_id = ? AND p.key = ? AND uor.status = 'ACTIVE'
        AND uor.valid_from <= datetime('now') AND uor.valid_until >= datetime('now')
        "#,
    )
    .bind(user_id.to_string())
    .bind(&outlet_id)
    .bind(permission_key)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;

    if count_outlet > 0 {
        return Ok(true);
    }

    // Fallback to legacy global roles if assigned
    let count_global: i64 = sqlx::query_scalar(
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

    Ok(count_global > 0)
}

#[tauri::command]
pub async fn login_user(
    pin: String,
    device_id: String,
    register_id: String,
    pool: tauri::State<'_, SqlitePool>,
) -> Result<UserSessionDTO, String> {
    let users = sqlx::query("SELECT id, name, pin_hash_v2, failed_login_attempts, locked_until, outlet_id FROM users WHERE status = 'active'")
        .fetch_all(pool.inner())
        .await
        .map_err(|e| e.to_string())?;

    let now = Utc::now().naive_utc();
    let mut verified_user = None;
    let mut is_locked = false;

    for u in users {
        let user_id: String = u.get("id");
        let name: String = u.get("name");
        let stored_hash: Option<String> = u.get("pin_hash_v2");
        let outlet_id: Option<String> = u.get("outlet_id");

        let Some(sh) = stored_hash else { continue; };

        if verify_pin_argon2(&pin, &sh).unwrap_or(false) {
            // Check if this matching user is locked
            if let Some(locked_until_str) = u.get::<Option<String>, _>("locked_until") {
                if let Ok(locked_time) = chrono::NaiveDateTime::parse_from_str(&locked_until_str, "%Y-%m-%d %H:%M:%S") {
                    if locked_time > now {
                        is_locked = true;
                        break;
                    }
                }
            }

            verified_user = Some((user_id.clone(), name, outlet_id.unwrap_or_default()));
            sqlx::query("UPDATE users SET failed_login_attempts = 0, locked_until = NULL WHERE id = ?")
                .bind(&user_id)
                .execute(pool.inner())
                .await
                .map_err(|e| e.to_string())?;
            break;
        }
    }

    if is_locked {
        return Err("Akun ini sedang terkunci karena terlalu banyak percobaan login salah. Silakan coba lagi nanti.".to_string());
    }

    let (user_id, name, outlet_id) = match verified_user {
        Some(val) => val,
        None => return Err("PIN yang Anda masukkan salah".to_string()),
    };

    let session_id = Uuid::new_v4().to_string();
    let session_token = Uuid::new_v4().to_string();
    let token_hash = hash_token(&session_token);
    let expires_at = (Utc::now() + Duration::hours(12)).format("%Y-%m-%d %H:%M:%S").to_string();

    sqlx::query(
        r#"
        INSERT INTO user_sessions (
            id, user_id, outlet_id, device_id, register_id, expires_at, authentication_method, session_token_hash
        ) VALUES (?, ?, ?, ?, ?, ?, 'PIN', ?)
        "#
    )
    .bind(&session_id)
    .bind(&user_id)
    .bind(&outlet_id)
    .bind(&device_id)
    .bind(&register_id)
    .bind(&expires_at)
    .bind(&token_hash)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let roles_rows = sqlx::query(
        r#"
        SELECT r.name 
        FROM user_roles ur
        JOIN roles r ON ur.role_id = r.id
        WHERE ur.user_id = ?
        "#
    )
    .bind(&user_id)
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let roles: Vec<String> = roles_rows.into_iter().map(|r| r.get("name")).collect();

    Ok(UserSessionDTO {
        session_id,
        session_token,
        user_id,
        user_name: name,
        outlet_id,
        roles,
    })
}

#[tauri::command]
pub async fn logout_user(
    session_token: String,
    pool: tauri::State<'_, SqlitePool>,
) -> Result<(), String> {
    let token_hash = hash_token(&session_token);
    sqlx::query("DELETE FROM user_sessions WHERE session_token_hash = ?")
        .bind(&token_hash)
        .execute(pool.inner())
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_active_session(
    pool: tauri::State<'_, SqlitePool>,
) -> Result<Option<UserSessionDTO>, String> {
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let session_row = sqlx::query(
        r#"
        SELECT s.id, s.user_id, s.outlet_id, u.name 
        FROM user_sessions s
        JOIN users u ON s.user_id = u.id
        WHERE s.expires_at > ?
        ORDER BY s.last_activity_at DESC LIMIT 1
        "#
    )
    .bind(&now)
    .fetch_optional(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    if let Some(row) = session_row {
        let session_id: String = row.get("id");
        let user_id: String = row.get("user_id");
        let outlet_id: String = row.get("outlet_id");
        let name: String = row.get("name");

        let roles_rows = sqlx::query(
            r#"
            SELECT r.name 
            FROM user_roles ur
            JOIN roles r ON ur.role_id = r.id
            WHERE ur.user_id = ?
            "#
        )
        .bind(&user_id)
        .fetch_all(pool.inner())
        .await
        .map_err(|e| e.to_string())?;

        let roles: Vec<String> = roles_rows.into_iter().map(|r| r.get("name")).collect();

        Ok(Some(UserSessionDTO {
            session_id,
            session_token: "".to_string(),
            user_id,
            user_name: name,
            outlet_id,
            roles,
        }))
    } else {
        Ok(None)
    }
}

