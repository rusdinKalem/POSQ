use sqlx::{SqlitePool, Row};
use tauri::State;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::Utc;

#[derive(Serialize)]
pub struct ShiftStatus {
    pub active: bool,
    pub shift_id: Option<Uuid>,
    pub started_at: Option<String>,
    pub starting_cash: i32,
}

#[derive(Serialize, Deserialize)]
pub struct DenominationItem {
    pub denomination: i32,
    pub quantity: i32,
}

#[derive(Serialize)]
pub struct VarianceReport {
    pub attempt_number: i32,
    pub expected_cash: i32,
    pub counted_cash: i32,
    pub variance: i32,
    pub recount_required: bool,
    pub supervisor_required: bool,
}

#[tauri::command]
pub async fn check_active_shift(pool: State<'_, SqlitePool>) -> Result<ShiftStatus, String> {
    let record = sqlx::query(
        r#"
        SELECT id, opened_at, starting_cash FROM shifts 
        WHERE status = 'open' 
        ORDER BY opened_at DESC LIMIT 1
        "#
    )
    .fetch_optional(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    if let Some(row) = record {
        let id_str: String = row.get("id");
        let opened_at: String = row.get("opened_at");
        let starting_cash: i32 = row.get("starting_cash");
        Ok(ShiftStatus { 
            active: true, 
            shift_id: Uuid::parse_str(&id_str).ok(),
            started_at: Some(opened_at),
            starting_cash,
        })
    } else {
        Ok(ShiftStatus { 
            active: false, 
            shift_id: None,
            started_at: None,
            starting_cash: 0,
        })
    }
}

#[tauri::command]
pub async fn open_shift(
    starting_cash: i32, 
    denominations: Vec<DenominationItem>,
    pool: State<'_, SqlitePool>
) -> Result<Uuid, String> {
    crate::license::enforce_active_license().await?;

    let user_id = crate::auth::get_current_user(pool.inner()).await?;
    
    // Check permission
    let has_perm = crate::auth::has_permission(pool.inner(), user_id, "shift.manage").await?;
    if !has_perm {
        return Err("Akses ditolak: Anda tidak memiliki izin untuk mengelola shift".to_string());
    }

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    // Check if there is already an open shift
    let active: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM shifts WHERE status = 'open'"
    )
    .fetch_one(&mut *tx)
    .await
    .unwrap_or(0);

    if active > 0 {
        return Err("Gagal: Sudah ada shift yang aktif saat ini".to_string());
    }

    let user_row = sqlx::query("SELECT merchant_id, outlet_id FROM users WHERE id = ?")
        .bind(user_id.to_string())
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
        
    let merchant_id: String = user_row.get("merchant_id");
    let outlet_id: String = user_row.get::<Option<String>, _>("outlet_id").ok_or("User has no outlet")?;

    // Get first available drawer
    let drawer_row = sqlx::query("SELECT id FROM cash_drawers WHERE outlet_id = ? AND status = 'available' LIMIT 1")
        .bind(&outlet_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    let drawer_id = match drawer_row {
        Some(r) => r.get::<String, _>("id"),
        None => "drw-default".to_string(), // fallback
    };

    let shift_id = Uuid::new_v4();
    
    // Create shift
    sqlx::query(
        "INSERT INTO shifts (id, merchant_id, outlet_id, opened_by, status, starting_cash, opened_at)
         VALUES (?, ?, ?, ?, 'open', ?, CURRENT_TIMESTAMP)"
    )
    .bind(shift_id.to_string())
    .bind(&merchant_id)
    .bind(&outlet_id)
    .bind(user_id.to_string())
    .bind(starting_cash)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    // Create cash drawer session
    let session_id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO cash_drawer_sessions (id, cash_drawer_id, shift_id, cashier_id, status) VALUES (?, ?, ?, ?, 'open')"
    )
    .bind(&session_id)
    .bind(&drawer_id)
    .bind(shift_id.to_string())
    .bind(user_id.to_string())
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    // Mark drawer as in_use
    sqlx::query("UPDATE cash_drawers SET status = 'in_use' WHERE id = ?")
        .bind(&drawer_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    // Record cash movement for starting cash
    let movement_number = format!("START-{}", Utc::now().format("%Y%m%d%H%M%S"));
    sqlx::query(
        r#"
        INSERT INTO cash_movements (
            id, movement_number, outlet_id, shift_id, cash_drawer_id, type, direction, amount,
            reason_code, notes, performed_by
        ) VALUES (?, ?, ?, ?, ?, 'STARTING_CASH', 'IN', ?, 'STARTING_CASH', 'Modal Awal', ?)
        "#
    )
    .bind(Uuid::new_v4().to_string())
    .bind(&movement_number)
    .bind(&outlet_id)
    .bind(shift_id.to_string())
    .bind(&drawer_id)
    .bind(starting_cash)
    .bind(user_id.to_string())
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(shift_id)
}

#[tauri::command]
pub async fn submit_blind_cash_count(
    shift_id: Uuid,
    counted_cash: i32,
    denominations: Vec<DenominationItem>,
    pool: State<'_, SqlitePool>,
) -> Result<VarianceReport, String> {
    let user_id = crate::auth::get_current_user(pool.inner()).await?;

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    // Fetch shift starting cash
    let shift_row = sqlx::query("SELECT starting_cash, status FROM shifts WHERE id = ?")
        .bind(shift_id.to_string())
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    
    let shift = shift_row.ok_or("Shift tidak ditemukan")?;
    let starting_cash: i32 = shift.get("starting_cash");
    let status: String = shift.get("status");

    if status != "open" {
        return Err("Shift sudah ditutup".to_string());
    }

    // Expected Cash calculation
    let sales: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(paid_total), 0) FROM orders WHERE shift_id = ?"
    )
    .bind(shift_id.to_string())
    .fetch_one(&mut *tx)
    .await
    .unwrap_or(0);

    let cash_in: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(amount), 0) FROM cash_movements WHERE shift_id = ? AND direction = 'IN' AND type != 'STARTING_CASH'"
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

    let expected_cash = starting_cash + sales as i32 + cash_in as i32 - cash_out as i32;
    let variance = counted_cash - expected_cash;

    // Get previous attempt count
    let prev_attempts: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM cash_count_sessions WHERE shift_id = ?"
    )
    .bind(shift_id.to_string())
    .fetch_one(&mut *tx)
    .await
    .unwrap_or(0);

    let attempt_number = (prev_attempts as i32) + 1;
    let session_id = Uuid::new_v4().to_string();

    // Save count session
    sqlx::query(
        r#"
        INSERT INTO cash_count_sessions (id, shift_id, attempt_number, expected_cash, actual_cash, variance, performed_by)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(&session_id)
    .bind(shift_id.to_string())
    .bind(attempt_number)
    .bind(expected_cash)
    .bind(counted_cash)
    .bind(variance)
    .bind(user_id.to_string())
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    // Insert count details
    for item in denominations {
        let total = item.denomination * item.quantity;
        sqlx::query(
            r#"
            INSERT INTO cash_count_details (id, cash_count_session_id, denomination, quantity, total)
            VALUES (?, ?, ?, ?, ?)
            "#
        )
        .bind(Uuid::new_v4().to_string())
        .bind(&session_id)
        .bind(item.denomination)
        .bind(item.quantity)
        .bind(total)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }

    let tolerance = 5000; // Rp 5.000 threshold
    let has_variance = variance.abs() > tolerance;
    
    // Require recount only if variance exists and attempts < 3
    let recount_required = has_variance && attempt_number < 3;
    let supervisor_required = has_variance;

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(VarianceReport {
        attempt_number,
        expected_cash,
        counted_cash,
        variance,
        recount_required,
        supervisor_required,
    })
}

#[tauri::command]
pub async fn approve_shift_variance(
    shift_id: Uuid,
    grant_id: Uuid,
    pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    // Validate and consume supervisor grant for shift variance
    let supervisor_id = crate::security_policy::validate_and_consume_grant(&mut tx, grant_id, "shift.approve_variance").await?;

    // Get last variance
    let last_session = sqlx::query("SELECT expected_cash, actual_cash, variance FROM cash_count_sessions WHERE shift_id = ? ORDER BY attempt_number DESC LIMIT 1")
        .bind(shift_id.to_string())
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    let s = last_session.ok_or("Belum ada pengisian cash count untuk shift ini")?;
    let expected_cash: i32 = s.get("expected_cash");
    let counted_cash: i32 = s.get("actual_cash");
    let variance: i32 = s.get("variance");

    // Close shift
    sqlx::query(
        r#"
        UPDATE shifts 
        SET status = 'closed', closed_by = ?, expected_cash = ?, counted_cash = ?, closed_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#
    )
    .bind(&supervisor_id)
    .bind(expected_cash)
    .bind(counted_cash)
    .bind(shift_id.to_string())
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    // Close drawer session
    sqlx::query("UPDATE cash_drawer_sessions SET status = 'closed', closed_at = CURRENT_TIMESTAMP WHERE shift_id = ? AND status = 'open'")
        .bind(shift_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    // Set drawer status back to available
    sqlx::query(
        r#"
        UPDATE cash_drawers 
        SET status = 'available' 
        WHERE id IN (SELECT cash_drawer_id FROM cash_drawer_sessions WHERE shift_id = ?)
        "#
    )
    .bind(shift_id.to_string())
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    // Add closing variance movement
    let movement_number = format!("VAR-{}", Utc::now().format("%Y%m%d%H%M%S"));
    let direction = if variance >= 0 { "IN" } else { "OUT" };

    sqlx::query(
        r#"
        INSERT INTO cash_movements (
            id, movement_number, outlet_id, shift_id, cash_drawer_id, type, direction, amount,
            reason_code, notes, performed_by, approved_by
        )
        SELECT ?, ?, outlet_id, ?, (SELECT cash_drawer_id FROM cash_drawer_sessions WHERE shift_id = ? LIMIT 1),
               'CLOSING_VARIANCE', ?, ?, 'CLOSING_VARIANCE', 'Selisih Kas Akhir', opened_by, ?
        FROM shifts WHERE id = ?
        "#
    )
    .bind(Uuid::new_v4().to_string())
    .bind(&movement_number)
    .bind(shift_id.to_string())
    .bind(shift_id.to_string())
    .bind(direction)
    .bind(variance.abs())
    .bind(&supervisor_id)
    .bind(shift_id.to_string())
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn close_shift(
    shift_id: Uuid,
    pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    let user_id = crate::auth::get_current_user(pool.inner()).await?;

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    // Get last session count
    let last_session = sqlx::query("SELECT expected_cash, actual_cash, variance FROM cash_count_sessions WHERE shift_id = ? ORDER BY attempt_number DESC LIMIT 1")
        .bind(shift_id.to_string())
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    let s = last_session.ok_or("Silakan isi perhitungan kas fisik (Cash Count) terlebih dahulu")?;
    let expected_cash: i32 = s.get("expected_cash");
    let counted_cash: i32 = s.get("actual_cash");
    let variance: i32 = s.get("variance");

    let tolerance = 5000;
    if variance.abs() > tolerance {
        return Err("Variance melebihi batas toleransi Rp 5.000. Memerlukan otorisasi Supervisor.".to_string());
    }

    // Close shift
    sqlx::query(
        r#"
        UPDATE shifts 
        SET status = 'closed', closed_by = ?, expected_cash = ?, counted_cash = ?, closed_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#
    )
    .bind(user_id.to_string())
    .bind(expected_cash)
    .bind(counted_cash)
    .bind(shift_id.to_string())
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    // Close drawer session
    sqlx::query("UPDATE cash_drawer_sessions SET status = 'closed', closed_at = CURRENT_TIMESTAMP WHERE shift_id = ? AND status = 'open'")
        .bind(shift_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    // Set drawer status back to available
    sqlx::query(
        r#"
        UPDATE cash_drawers 
        SET status = 'available' 
        WHERE id IN (SELECT cash_drawer_id FROM cash_drawer_sessions WHERE shift_id = ?)
        "#
    )
    .bind(shift_id.to_string())
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(())
}
