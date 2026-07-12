use sqlx::{SqlitePool, Row};
use tauri::State;
use uuid::Uuid;
use serde_json::{json, Value};
use chrono::Utc;

#[tauri::command]
pub async fn run_fraud_checks(
    shift_id: String,
    pool: State<'_, SqlitePool>,
) -> Result<i32, String> {
    let mut alerts_created = 0;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    // Get shift details
    let shift_row = sqlx::query("SELECT outlet_id, opened_by FROM shifts WHERE id = ?")
        .bind(&shift_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    let (outlet_id, cashier_id) = match shift_row {
        Some(r) => (
            r.get::<String, _>("outlet_id"),
            r.get::<String, _>("opened_by")
        ),
        None => return Ok(0),
    };

    // Heuristic 1: High Void Rate (Voids count > 3 in this shift)
    let void_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*) FROM audit_logs 
        WHERE action = 'transaction.void' AND actor_user_id = ? 
        AND created_at >= (SELECT opened_at FROM shifts WHERE id = ?)
        "#
    )
    .bind(&cashier_id)
    .bind(&shift_id)
    .fetch_one(&mut *tx)
    .await
    .unwrap_or(0);

    if void_count >= 3 {
        let alert_id = Uuid::new_v4().to_string();
        let support = json!({ "void_count": void_count }).to_string();
        
        let exists: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM fraud_alerts WHERE shift_id = ? AND rule_id = 'RULE_HIGH_VOID'"
        )
        .bind(&shift_id)
        .fetch_one(&mut *tx)
        .await
        .unwrap_or(0);

        if exists == 0 {
            sqlx::query(
                r#"
                INSERT INTO fraud_alerts (id, rule_id, severity, user_id, outlet_id, shift_id, supporting_data, status)
                VALUES (?, 'RULE_HIGH_VOID', 'HIGH', ?, ?, ?, ?, 'OPEN')
                "#
            )
            .bind(&alert_id)
            .bind(&cashier_id)
            .bind(&outlet_id)
            .bind(&shift_id)
            .bind(&support)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
            alerts_created += 1;
        }
    }

    // Heuristic 2: Repetitive Cash Drawer Open No Sale (count > 5)
    let no_sale_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*) FROM audit_logs 
        WHERE action = 'cash.drawer_open_no_sale' AND actor_user_id = ? 
        AND created_at >= (SELECT opened_at FROM shifts WHERE id = ?)
        "#
    )
    .bind(&cashier_id)
    .bind(&shift_id)
    .fetch_one(&mut *tx)
    .await
    .unwrap_or(0);

    if no_sale_count >= 5 {
        let alert_id = Uuid::new_v4().to_string();
        let support = json!({ "no_sale_count": no_sale_count }).to_string();
        
        let exists: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM fraud_alerts WHERE shift_id = ? AND rule_id = 'RULE_NO_SALE_OPEN'"
        )
        .bind(&shift_id)
        .fetch_one(&mut *tx)
        .await
        .unwrap_or(0);

        if exists == 0 {
            sqlx::query(
                r#"
                INSERT INTO fraud_alerts (id, rule_id, severity, user_id, outlet_id, shift_id, supporting_data, status)
                VALUES (?, 'RULE_NO_SALE_OPEN', 'MEDIUM', ?, ?, ?, ?, 'OPEN')
                "#
            )
            .bind(&alert_id)
            .bind(&cashier_id)
            .bind(&outlet_id)
            .bind(&shift_id)
            .bind(&support)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
            alerts_created += 1;
        }
    }

    // Heuristic 3: High Total Cash Out (> 1.000.000 IDR)
    let total_cash_out: i64 = sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(amount), 0) FROM cash_movements 
        WHERE type = 'CASH_OUT' AND shift_id = ?
        "#
    )
    .bind(&shift_id)
    .fetch_one(&mut *tx)
    .await
    .unwrap_or(0);

    if total_cash_out > 1000000 {
        let alert_id = Uuid::new_v4().to_string();
        let support = json!({ "total_cash_out": total_cash_out }).to_string();

        let exists: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM fraud_alerts WHERE shift_id = ? AND rule_id = 'RULE_HIGH_CASHOUT'"
        )
        .bind(&shift_id)
        .fetch_one(&mut *tx)
        .await
        .unwrap_or(0);

        if exists == 0 {
            sqlx::query(
                r#"
                INSERT INTO fraud_alerts (id, rule_id, severity, user_id, outlet_id, shift_id, supporting_data, status)
                VALUES (?, 'RULE_HIGH_CASHOUT', 'MEDIUM', ?, ?, ?, ?, 'OPEN')
                "#
            )
            .bind(&alert_id)
            .bind(&cashier_id)
            .bind(&outlet_id)
            .bind(&shift_id)
            .bind(&support)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
            alerts_created += 1;
        }
    }

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(alerts_created)
}

#[tauri::command]
pub async fn get_fraud_alerts(
    pool: State<'_, SqlitePool>,
) -> Result<Value, String> {
    let records = sqlx::query(
        r#"
        SELECT fa.id, fa.rule_id, fa.severity, fa.status, fa.supporting_data, fa.created_at,
               u.name as cashier_name, o.name as outlet_name
        FROM fraud_alerts fa
        LEFT JOIN users u ON fa.user_id = u.id
        LEFT JOIN outlets o ON fa.outlet_id = o.id
        ORDER BY fa.created_at DESC
        LIMIT 50
        "#
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let alerts: Vec<Value> = records.into_iter().map(|r| {
        let id: String = r.get("id");
        let rule_id: String = r.get("rule_id");
        let severity: String = r.get("severity");
        let status: String = r.get("status");
        let supporting_data: Option<String> = r.get("supporting_data");
        let created_at: String = r.get("created_at");
        let cashier_name: Option<String> = r.get("cashier_name");
        let outlet_name: Option<String> = r.get("outlet_name");

        let description = match rule_id.as_str() {
            "RULE_HIGH_VOID" => "Frekuensi pembatalan transaksi (Void) sangat tinggi dalam satu shift.",
            "RULE_NO_SALE_OPEN" => "Pembukaan laci kasir tanpa adanya penjualan terdeteksi berulang kali.",
            "RULE_HIGH_CASHOUT" => "Batas pengeluaran kas kecil (Cash Out) melebihi batas kumulatif Rp 1.000.000.",
            _ => "Aktivitas mencurigakan terdeteksi.",
        };

        json!({
            "id": id,
            "rule_id": rule_id,
            "description": description,
            "severity": severity,
            "status": status,
            "supporting_data": supporting_data.map(|s| serde_json::from_str::<Value>(&s).unwrap_or(json!({}))).unwrap_or(json!({})),
            "cashier_name": cashier_name.unwrap_or("System".to_string()),
            "outlet_name": outlet_name.unwrap_or("Central".to_string()),
            "created_at": created_at
        })
    }).collect();

    Ok(json!({ "alerts": alerts }))
}

#[tauri::command]
pub async fn resolve_fraud_alert(
    alert_id: String,
    status: String, // 'RESOLVED', 'FALSE_POSITIVE'
    pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    sqlx::query("UPDATE fraud_alerts SET status = ? WHERE id = ?")
        .bind(status)
        .bind(alert_id)
        .execute(pool.inner())
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
