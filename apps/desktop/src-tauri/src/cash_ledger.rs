use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, Row};
use tauri::State;
use uuid::Uuid;
use chrono::Utc;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CashMovement {
    pub id: Uuid,
    pub movement_number: String,
    pub type_str: String,
    pub direction: String,
    pub amount: i32,
    pub reason_code: Option<String>,
    pub notes: Option<String>,
    pub performed_by_name: String,
    pub created_at: String,
}

#[tauri::command]
pub async fn post_cash_in(
    shift_id: Uuid,
    amount: i32,
    category: String,
    reason: String,
    pool: State<'_, SqlitePool>,
) -> Result<String, String> {
    if amount <= 0 {
        return Err("Jumlah uang harus lebih besar dari nol".to_string());
    }

    let user_id = crate::auth::get_current_user(pool.inner()).await?;

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    // Check shift status
    let shift = sqlx::query("SELECT status, outlet_id FROM shifts WHERE id = ?")
        .bind(shift_id.to_string())
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    
    let shift_row = shift.ok_or("Shift tidak ditemukan")?;
    let status: String = shift_row.get("status");
    let outlet_id: String = shift_row.get("outlet_id");

    if status != "open" {
        return Err("Shift tidak aktif, transaksi ditolak".to_string());
    }

    // Get default cash drawer
    let drawer = sqlx::query("SELECT id FROM cash_drawers WHERE outlet_id = ? LIMIT 1")
        .bind(&outlet_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| "Laci kasir tidak ditemukan untuk outlet ini".to_string())?;
    
    let drawer_id: String = drawer.get("id");

    let movement_number = format!("CSH-IN-{}", Utc::now().format("%Y%m%d%H%M%S"));
    let movement_id = Uuid::new_v4().to_string();

    sqlx::query(
        r#"
        INSERT INTO cash_movements (
            id, movement_number, outlet_id, shift_id, cash_drawer_id, type, direction, amount,
            reason_code, notes, performed_by
        ) VALUES (?, ?, ?, ?, ?, 'CASH_IN', 'IN', ?, ?, ?, ?)
        "#
    )
    .bind(&movement_id)
    .bind(&movement_number)
    .bind(&outlet_id)
    .bind(shift_id.to_string())
    .bind(&drawer_id)
    .bind(amount)
    .bind(&category)
    .bind(&reason)
    .bind(user_id.to_string())
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(movement_number)
}

#[tauri::command]
pub async fn post_cash_out(
    shift_id: Uuid,
    amount: i32,
    category: String,
    reason: String,
    grant_id: Option<Uuid>,
    pool: State<'_, SqlitePool>,
) -> Result<String, String> {
    if amount <= 0 {
        return Err("Jumlah uang harus lebih besar dari nol".to_string());
    }

    let user_id = crate::auth::get_current_user(pool.inner()).await?;

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    // Check shift status
    let shift = sqlx::query("SELECT status, outlet_id FROM shifts WHERE id = ?")
        .bind(shift_id.to_string())
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    
    let shift_row = shift.ok_or("Shift tidak ditemukan")?;
    let status: String = shift_row.get("status");
    let outlet_id: String = shift_row.get("outlet_id");

    if status != "open" {
        return Err("Shift tidak aktif, transaksi ditolak".to_string());
    }

    // High risk limit: Rp 200.000 requires supervisor approval
    let mut supervisor_id = None;
    if amount > 200000 {
        let gid = grant_id.ok_or("Nilai pengeluaran melebihi limit Rp 200.000. Memerlukan otorisasi Supervisor.")?;
        let sup_id = crate::security_policy::validate_and_consume_grant(&mut tx, gid, "cash.cash_out").await?;
        supervisor_id = Some(sup_id);
    }

    // Get default cash drawer
    let drawer = sqlx::query("SELECT id FROM cash_drawers WHERE outlet_id = ? LIMIT 1")
        .bind(&outlet_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| "Laci kasir tidak ditemukan untuk outlet ini".to_string())?;
    
    let drawer_id: String = drawer.get("id");

    // Enforce cash availability (cannot cash out more than expected cash)
    // Calc current cash in drawer
    let expected: i64 = sqlx::query_scalar(
        r#"
        SELECT COALESCE(starting_cash, 0) FROM shifts WHERE id = ?
        "#
    )
    .bind(shift_id.to_string())
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    let sales: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(paid_total), 0) FROM orders WHERE shift_id = ?"
    )
    .bind(shift_id.to_string())
    .fetch_one(&mut *tx)
    .await
    .unwrap_or(0);

    let cash_in: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(amount), 0) FROM cash_movements WHERE shift_id = ? AND direction = 'IN'"
    )
    .bind(shift_id.to_string())
    .fetch_one(&mut *tx)
    .await
    .unwrap_or(0);

    let cash_out: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(amount), 0) FROM cash_movements WHERE shift_id = ? AND direction = 'OUT'"
    )
    .bind(shift_id.to_string())
    .fetch_one(&mut *tx)
    .await
    .unwrap_or(0);

    let available_cash = expected + sales + cash_in - cash_out;
    if (amount as i64) > available_cash {
        return Err(format!("Uang di laci tidak mencukupi (Tersedia: Rp {})", available_cash));
    }

    let movement_number = format!("CSH-OUT-{}", Utc::now().format("%Y%m%d%H%M%S"));
    let movement_id = Uuid::new_v4().to_string();

    sqlx::query(
        r#"
        INSERT INTO cash_movements (
            id, movement_number, outlet_id, shift_id, cash_drawer_id, type, direction, amount,
            reason_code, notes, performed_by, approved_by
        ) VALUES (?, ?, ?, ?, ?, 'CASH_OUT', 'OUT', ?, ?, ?, ?, ?)
        "#
    )
    .bind(&movement_id)
    .bind(&movement_number)
    .bind(&outlet_id)
    .bind(shift_id.to_string())
    .bind(&drawer_id)
    .bind(amount)
    .bind(&category)
    .bind(&reason)
    .bind(user_id.to_string())
    .bind(supervisor_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(movement_number)
}

#[tauri::command]
pub async fn post_safe_drop(
    shift_id: Uuid,
    amount: i32,
    bag_number: String,
    pool: State<'_, SqlitePool>,
) -> Result<String, String> {
    if amount <= 0 {
        return Err("Jumlah safe drop harus lebih besar dari nol".to_string());
    }

    let user_id = crate::auth::get_current_user(pool.inner()).await?;

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    // Check shift status
    let shift = sqlx::query("SELECT status, outlet_id FROM shifts WHERE id = ?")
        .bind(shift_id.to_string())
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    
    let shift_row = shift.ok_or("Shift tidak ditemukan")?;
    let status: String = shift_row.get("status");
    let outlet_id: String = shift_row.get("outlet_id");

    if status != "open" {
        return Err("Shift tidak aktif, transaksi ditolak".to_string());
    }

    // Get default cash drawer
    let drawer = sqlx::query("SELECT id FROM cash_drawers WHERE outlet_id = ? LIMIT 1")
        .bind(&outlet_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| "Laci kasir tidak ditemukan untuk outlet ini".to_string())?;
    
    let drawer_id: String = drawer.get("id");

    let movement_number = format!("SAFE-DROP-{}", Utc::now().format("%Y%m%d%H%M%S"));
    let movement_id = Uuid::new_v4().to_string();

    sqlx::query(
        r#"
        INSERT INTO cash_movements (
            id, movement_number, outlet_id, shift_id, cash_drawer_id, type, direction, amount,
            reason_code, notes, performed_by
        ) VALUES (?, ?, ?, ?, ?, 'SAFE_DROP', 'OUT', ?, 'SAFE_DROP', ?, ?)
        "#
    )
    .bind(&movement_id)
    .bind(&movement_number)
    .bind(&outlet_id)
    .bind(shift_id.to_string())
    .bind(&drawer_id)
    .bind(amount)
    .bind(format!("Bag No: {}", bag_number))
    .bind(user_id.to_string())
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(movement_number)
}

#[tauri::command]
pub async fn get_cash_movements(
    shift_id: Uuid,
    pool: State<'_, SqlitePool>,
) -> Result<Vec<CashMovement>, String> {
    let records = sqlx::query(
        r#"
        SELECT cm.id, cm.movement_number, cm.type, cm.direction, cm.amount, cm.reason_code, cm.notes, u.name as performed_by_name, cm.created_at
        FROM cash_movements cm
        JOIN users u ON cm.performed_by = u.id
        WHERE cm.shift_id = ?
        ORDER BY cm.created_at DESC
        "#
    )
    .bind(shift_id.to_string())
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let movements = records.into_iter().map(|r| {
        let id_str: String = r.get("id");
        CashMovement {
            id: Uuid::parse_str(&id_str).unwrap_or_default(),
            movement_number: r.get("movement_number"),
            type_str: r.get("type"),
            direction: r.get("direction"),
            amount: r.get("amount"),
            reason_code: r.get("reason_code"),
            notes: r.get("notes"),
            performed_by_name: r.get("performed_by_name"),
            created_at: r.get("created_at"),
        }
    }).collect();

    Ok(movements)
}

#[tauri::command]
pub async fn reverse_cash_movement(
    movement_id: Uuid,
    grant_id: Uuid,
    pool: State<'_, SqlitePool>,
) -> Result<String, String> {
    let user_id = crate::auth::get_current_user(pool.inner()).await?;

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    // Consume supervisor authorization for reverse action
    let supervisor_id = crate::security_policy::validate_and_consume_grant(&mut tx, grant_id, "cash.reverse").await?;

    // Get original movement
    let orig = sqlx::query(
        "SELECT outlet_id, shift_id, cash_drawer_id, type, direction, amount, movement_number FROM cash_movements WHERE id = ?"
    )
    .bind(movement_id.to_string())
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    let orig_row = orig.ok_or("Data cash movement asli tidak ditemukan")?;
    let o_type: String = orig_row.get("type");
    if o_type == "REVERSAL" {
        return Err("Tidak dapat me-reverse transaksi reversal".to_string());
    }

    // Check if already reversed
    let is_reversed = sqlx::query("SELECT count(*) FROM cash_movements WHERE parent_movement_id = ?")
        .bind(movement_id.to_string())
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    
    let count: i64 = is_reversed.get(0);
    if count > 0 {
        return Err("Transaksi ini sudah pernah di-reverse sebelumnya".to_string());
    }

    let outlet_id: String = orig_row.get("outlet_id");
    let shift_id: String = orig_row.get("shift_id");
    let cash_drawer_id: String = orig_row.get("cash_drawer_id");
    let direction: String = orig_row.get("direction");
    let amount: i32 = orig_row.get("amount");
    let orig_number: String = orig_row.get("movement_number");

    let rev_direction = if direction == "IN" { "OUT" } else { "IN" };
    let rev_number = format!("REV-{}", orig_number);
    let new_id = Uuid::new_v4().to_string();

    sqlx::query(
        r#"
        INSERT INTO cash_movements (
            id, movement_number, outlet_id, shift_id, cash_drawer_id, type, direction, amount,
            reason_code, notes, parent_movement_id, performed_by, approved_by
        ) VALUES (?, ?, ?, ?, ?, 'REVERSAL', ?, ?, 'REVERSAL', ?, ?, ?, ?)
        "#
    )
    .bind(&new_id)
    .bind(&rev_number)
    .bind(&outlet_id)
    .bind(&shift_id)
    .bind(&cash_drawer_id)
    .bind(rev_direction)
    .bind(amount)
    .bind(format!("Reversal of {}", orig_number))
    .bind(movement_id.to_string())
    .bind(user_id.to_string())
    .bind(&supervisor_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(rev_number)
}
