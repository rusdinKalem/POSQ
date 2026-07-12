use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, Row};
use tauri::State;
use uuid::Uuid;
use chrono::{Utc, Duration};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PolicyDecision {
    ALLOW,
    REQUIRE_SUPERVISOR,
    REQUIRE_MANAGER,
    DENY,
}

impl PolicyDecision {
    pub fn to_string(&self) -> String {
        match self {
            PolicyDecision::ALLOW => "ALLOW".to_string(),
            PolicyDecision::REQUIRE_SUPERVISOR => "REQUIRE_SUPERVISOR".to_string(),
            PolicyDecision::REQUIRE_MANAGER => "REQUIRE_MANAGER".to_string(),
            PolicyDecision::DENY => "DENY".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PolicyEvaluationResult {
    pub decision: PolicyDecision,
    pub message: String,
    pub policy_id: Option<String>,
}

#[tauri::command]
pub async fn evaluate_action_policy(
    action_type: String,
    amount: i32,
    pool: State<'_, SqlitePool>,
) -> Result<PolicyEvaluationResult, String> {
    let user_id = crate::auth::get_current_user(pool.inner()).await?;

    // Check user outlet first
    let user_row = sqlx::query("SELECT outlet_id FROM users WHERE id = ?")
        .bind(user_id.to_string())
        .fetch_optional(pool.inner())
        .await
        .map_err(|e| e.to_string())?;
    
    let outlet_id = match user_row {
        Some(row) => row.get::<Option<String>, _>("outlet_id"),
        None => return Err("User tidak ditemukan".to_string()),
    };

    let query_policy = match &outlet_id {
        Some(oid) => {
            sqlx::query(
                r#"
                SELECT id, policy_decision 
                FROM authorization_policies 
                WHERE action_type = ? AND (outlet_id = ? OR outlet_id IS NULL)
                AND min_amount <= ? AND (max_amount >= ? OR max_amount IS NULL)
                ORDER BY outlet_id DESC, min_amount DESC LIMIT 1
                "#
            )
            .bind(&action_type)
            .bind(oid)
            .bind(amount)
            .bind(amount)
        },
        None => {
            sqlx::query(
                r#"
                SELECT id, policy_decision 
                FROM authorization_policies 
                WHERE action_type = ? AND outlet_id IS NULL
                AND min_amount <= ? AND (max_amount >= ? OR max_amount IS NULL)
                ORDER BY min_amount DESC LIMIT 1
                "#
            )
            .bind(&action_type)
            .bind(amount)
            .bind(amount)
        }
    };

    let policy_row = query_policy
        .fetch_optional(pool.inner())
        .await
        .map_err(|e| e.to_string())?;

    if let Some(row) = policy_row {
        let decision_str: String = row.get("policy_decision");
        let decision = match decision_str.as_str() {
            "ALLOW" => PolicyDecision::ALLOW,
            "REQUIRE_SUPERVISOR" => PolicyDecision::REQUIRE_SUPERVISOR,
            "REQUIRE_MANAGER" => PolicyDecision::REQUIRE_MANAGER,
            _ => PolicyDecision::DENY,
        };
        Ok(PolicyEvaluationResult {
            decision,
            message: format!("Policy {} matching action {}", row.get::<String, _>("id"), action_type),
            policy_id: Some(row.get("id")),
        })
    } else {
        // Default fallbacks if no policy configured
        let decision = if action_type == "transaction.void" || action_type == "cash.cash_out" {
            PolicyDecision::REQUIRE_SUPERVISOR
        } else {
            PolicyDecision::ALLOW
        };
        Ok(PolicyEvaluationResult {
            decision,
            message: "Default policy applied".to_string(),
            policy_id: None,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct SupervisorAuthRequest {
    pub pin: String,
    pub action_type: String,
    pub amount: i32,
    pub reason_code: String,
    pub resource_id: Option<String>,
}

#[derive(Serialize)]
pub struct SupervisorAuthResponse {
    pub grant_id: Uuid,
    pub supervisor_id: Uuid,
    pub supervisor_name: String,
}

#[tauri::command]
pub async fn verify_supervisor_pin(
    req: SupervisorAuthRequest,
    pool: State<'_, SqlitePool>,
) -> Result<SupervisorAuthResponse, String> {
    let cashier_id = crate::auth::get_current_user(pool.inner()).await?;

    // Find all users who are owners, managers, or supervisors
    let eligible_users = sqlx::query(
        r#"
        SELECT u.id, u.name, u.pin_hash_v2, u.pin_hash, u.failed_login_attempts, u.locked_until
        FROM users u
        JOIN user_roles ur ON u.id = ur.user_id
        JOIN roles r ON ur.role_id = r.id
        WHERE r.name IN ('owner', 'manager', 'supervisor') AND u.status = 'active'
        "#
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let now = Utc::now().naive_utc();
    let mut verified_user = None;
    let mut is_locked = false;
    let mut matched_but_no_perm = false;

    for u in eligible_users {
        let u_id: String = u.get("id");
        let u_name: String = u.get("name");
        let stored_hash: Option<String> = u.get("pin_hash_v2");

        let Some(sh) = stored_hash else { continue; };

        if crate::auth::verify_pin_argon2(&req.pin, &sh).unwrap_or(false) {
            // Check lock status
            if let Some(locked_until_str) = u.get::<Option<String>, _>("locked_until") {
                if let Ok(locked_time) = chrono::NaiveDateTime::parse_from_str(&locked_until_str, "%Y-%m-%d %H:%M:%S") {
                    if locked_time > now {
                        is_locked = true;
                        break;
                    }
                }
            }

            // Verify permission of this supervisor for this action
            let has_perm = crate::auth::has_permission(
                pool.inner(), 
                Uuid::parse_str(&u_id).unwrap_or_default(), 
                &req.action_type
            ).await?;

            if has_perm {
                verified_user = Some((u_id.clone(), u_name));
                // Reset failed attempts
                sqlx::query("UPDATE users SET failed_login_attempts = 0, locked_until = NULL WHERE id = ?")
                    .bind(&u_id)
                    .execute(pool.inner())
                    .await
                    .map_err(|e| e.to_string())?;
            } else {
                matched_but_no_perm = true;
            }
            break;
        }
    }

    if is_locked {
        return Err("Akun Supervisor ini sedang terkunci karena terlalu banyak percobaan salah. Silakan coba lagi nanti.".to_string());
    }

    if matched_but_no_perm {
        return Err("Supervisor tidak memiliki izin untuk tindakan ini".to_string());
    }

    let (supervisor_id_str, supervisor_name) = match verified_user {
        Some(val) => val,
        None => return Err("PIN Otorisasi salah".to_string()),
    };

    let supervisor_uuid = Uuid::parse_str(&supervisor_id_str).map_err(|e| e.to_string())?;

    // Prevent self-approval for cashiers unless allowed (default: false)
    if supervisor_uuid == cashier_id {
        return Err("Self-approval tidak diizinkan untuk tindakan sensitif ini".to_string());
    }

    // Get current active shift
    let shift_row = sqlx::query("SELECT id, outlet_id FROM shifts WHERE status = 'open' LIMIT 1")
        .fetch_optional(pool.inner())
        .await
        .map_err(|e| e.to_string())?;

    let (shift_id, outlet_id) = match shift_row {
        Some(r) => (Some(r.get::<String, _>("id")), Some(r.get::<String, _>("outlet_id"))),
        None => (None, None),
    };

    // Create Grant
    let grant_id = Uuid::new_v4();
    let expires_at = Utc::now() + Duration::seconds(120); // 2 minutes expiration

    sqlx::query(
        r#"
        INSERT INTO authorization_grants (
            id, action_type, resource_id, cashier_id, supervisor_id, outlet_id, shift_id,
            approved_amount, reason_code, issued_at, expires_at, status
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'APPROVED')
        "#
    )
    .bind(grant_id.to_string())
    .bind(&req.action_type)
    .bind(req.resource_id)
    .bind(cashier_id.to_string())
    .bind(&supervisor_id_str)
    .bind(outlet_id)
    .bind(shift_id)
    .bind(req.amount)
    .bind(&req.reason_code)
    .bind(Utc::now().format("%Y-%m-%d %H:%M:%S").to_string())
    .bind(expires_at.format("%Y-%m-%d %H:%M:%S").to_string())
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    // Fetch merchant details for logging
    let merchant_row = sqlx::query("SELECT merchant_id, outlet_id FROM users WHERE id = ?")
        .bind(cashier_id.to_string())
        .fetch_optional(pool.inner())
        .await
        .unwrap_or(None);

    let (m_id, o_id) = match merchant_row {
        Some(r) => (r.get::<String, _>("merchant_id"), r.get::<Option<String>, _>("outlet_id")),
        None => ("".to_string(), None),
    };

    // Log the successful authorization with full cryptographic hash chain integrity
    let mut tx = pool.inner().begin().await.map_err(|e| e.to_string())?;
    crate::audit::log_action(
        &mut *tx,
        m_id,
        o_id,
        cashier_id.to_string(),
        &format!("auth_grant_{}", req.action_type),
        "authorization_grant",
        Some(grant_id.to_string()),
        Some(req.reason_code.as_str()),
    )
    .await?;
    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(SupervisorAuthResponse {
        grant_id,
        supervisor_id: supervisor_uuid,
        supervisor_name,
    })
}

pub async fn validate_and_consume_grant(
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    grant_id: Uuid,
    action_type: &str,
) -> Result<String, String> {
    let now = Utc::now().naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();

    let grant_row = sqlx::query(
        r#"
        SELECT supervisor_id, expires_at, status 
        FROM authorization_grants 
        WHERE id = ? FOR UPDATE
        "#
    )
    .bind(grant_id.to_string())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|e| e.to_string())?;

    let grant = grant_row.ok_or("Token Otorisasi tidak ditemukan")?;
    let status: String = grant.get("status");
    let expires_at_str: String = grant.get("expires_at");
    let supervisor_id: String = grant.get("supervisor_id");

    if status != "APPROVED" {
        return Err("Token Otorisasi tidak valid atau sudah digunakan".to_string());
    }

    if expires_at_str < now {
        sqlx::query("UPDATE authorization_grants SET status = 'EXPIRED' WHERE id = ?")
            .bind(grant_id.to_string())
            .execute(&mut **transaction)
            .await
            .map_err(|e| e.to_string())?;
        return Err("Token Otorisasi sudah kedaluwarsa".to_string());
    }

    // Mark as USED
    sqlx::query(
        r#"
        UPDATE authorization_grants 
        SET status = 'USED', used_at = CURRENT_TIMESTAMP 
        WHERE id = ?
        "#
    )
    .bind(grant_id.to_string())
    .execute(&mut **transaction)
    .await
    .map_err(|e| e.to_string())?;

    Ok(supervisor_id)
}
