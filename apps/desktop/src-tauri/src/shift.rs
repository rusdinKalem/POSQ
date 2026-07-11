use sqlx::{SqlitePool, Row};
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn close_shift(shift_id: Uuid, counted_cash: i32, pool: State<'_, SqlitePool>) -> Result<(), String> {
    let user_id = crate::auth::get_current_user(pool.inner()).await?;
    
    // Check permission
    let has_perm = crate::auth::has_permission(pool.inner(), user_id, "shift.manage").await?;
    if !has_perm {
        return Err("Akses ditolak: Anda tidak memiliki izin untuk mengelola shift".to_string());
    }

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let shift = sqlx::query(
        "SELECT merchant_id, outlet_id, starting_cash FROM shifts WHERE id = ? AND status = 'open'"
    )
    .bind(shift_id.to_string())
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    let shift = shift.ok_or("Shift tidak ditemukan atau sudah ditutup")?;

    // Calculate expected cash
    // expected_cash = starting_cash + sum(payments WHERE method='CASH' and order_id in (orders with this shift_id))
    // Note: for MVP we sum all 'paid_total' in orders for this shift
    let totals = sqlx::query(
        "SELECT COALESCE(SUM(paid_total), 0) as cash_sales FROM orders WHERE shift_id = ?"
    )
    .bind(shift_id.to_string())
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    let cash_sales: i64 = totals.get("cash_sales");
    let starting_cash: i32 = shift.get("starting_cash");
    let expected_cash = starting_cash + cash_sales as i32;

    let merchant_id: String = shift.get("merchant_id");
    let outlet_id: String = shift.get("outlet_id");

    sqlx::query(
        r#"
        UPDATE shifts 
        SET status = 'closed', closed_by = ?, expected_cash = ?, counted_cash = ?, closed_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
    )
    .bind(user_id.to_string())
    .bind(expected_cash)
    .bind(counted_cash)
    .bind(shift_id.to_string())
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    // Audit Log
    crate::audit::log_action(
        &mut *tx, 
        merchant_id, 
        Some(outlet_id), 
        user_id.to_string(), 
        "close_shift", 
        "shift", 
        Some(shift_id.to_string()), 
        Some(&format!("Expected: {}, Counted: {}", expected_cash, counted_cash))
    ).await?;

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(())
}

use serde::Serialize;

#[derive(Serialize)]
pub struct ShiftStatus {
    pub active: bool,
    pub shift_id: Option<Uuid>,
}

#[tauri::command]
pub async fn check_active_shift(pool: State<'_, SqlitePool>) -> Result<ShiftStatus, String> {
    let record = sqlx::query(
        r#"
        SELECT id FROM shifts 
        WHERE status = 'open' 
        ORDER BY opened_at DESC LIMIT 1
        "#
    )
    .fetch_optional(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    if let Some(row) = record {
        let id_str: String = row.get("id");
        Ok(ShiftStatus { active: true, shift_id: Uuid::parse_str(&id_str).ok() })
    } else {
        Ok(ShiftStatus { active: false, shift_id: None })
    }
}

#[tauri::command]
pub async fn open_shift(starting_cash: i32, pool: State<'_, SqlitePool>) -> Result<Uuid, String> {
    // SEC-001: Enforce license status to prevent API bypass
    crate::license::enforce_active_license().await?;

    let user_id = crate::auth::get_current_user(pool.inner()).await?;
    
    // Check permission
    let has_perm = crate::auth::has_permission(pool.inner(), user_id, "shift.manage").await?;
    if !has_perm {
        return Err("Akses ditolak: Anda tidak memiliki izin untuk mengelola shift".to_string());
    }

    let user_record = sqlx::query("SELECT merchant_id, outlet_id FROM users WHERE id = ?")
        .bind(user_id.to_string())
        .fetch_optional(pool.inner())
        .await
        .map_err(|e| e.to_string())?;
        
    let user = user_record.ok_or("User not found")?;
    let merchant_id: String = user.get("merchant_id");
    let outlet_id: String = user.get::<Option<String>, _>("outlet_id").ok_or("User has no outlet")?;

    let shift_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO shifts (id, merchant_id, outlet_id, opened_by, status, starting_cash, opened_at)
         VALUES (?, ?, ?, ?, 'open', ?, CURRENT_TIMESTAMP)"
    )
    .bind(shift_id.to_string())
    .bind(&merchant_id)
    .bind(&outlet_id)
    .bind(user_id.to_string())
    .bind(starting_cash)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    Ok(shift_id)
}
