use sqlx::{SqlitePool, Row};
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn process_refund(order_id: Uuid, reason: String, pool: State<'_, SqlitePool>) -> Result<(), String> {
    // SEC-001: Enforce license status to prevent API bypass
    crate::license::enforce_active_license().await?;

    let user_id = crate::auth::get_current_user(pool.inner()).await?;
    
    // Check permission (M4-002 Backend permission guard)
    let has_perm = crate::auth::has_permission(pool.inner(), user_id, "refund.approve").await?;
    if !has_perm {
        return Err("Akses ditolak: Anda tidak memiliki izin untuk memproses refund".to_string());
    }

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    // 1. Get Order
    let order = sqlx::query(
        "SELECT id, merchant_id, outlet_id, grand_total, status FROM orders WHERE id = ?"
    )
    .bind(order_id.to_string())
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    let status: String = order.get("status");
    if status == "refunded" {
        return Err("Order ini sudah direfund sebelumnya".to_string());
    }

    let merchant_id: String = order.get("merchant_id");
    let outlet_id: String = order.get("outlet_id");
    let grand_total: i32 = order.get("grand_total");

    // 2. Insert Refund
    let refund_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO refunds (id, order_id, amount, reason, approved_by, created_by, created_at)
        VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
        "#,
    )
    .bind(refund_id.to_string())
    .bind(order_id.to_string())
    .bind(grand_total)
    .bind(&reason)
    .bind(user_id.to_string())
    .bind(user_id.to_string())
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    // 3. Mark Order as Refunded
    sqlx::query(
        "UPDATE orders SET status = 'refunded', updated_at = CURRENT_TIMESTAMP WHERE id = ?"
    )
    .bind(order_id.to_string())
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    // 4. Return Stock
    let items = sqlx::query(
        "SELECT product_id, qty FROM order_items WHERE order_id = ?"
    )
    .bind(order_id.to_string())
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    for item in items {
        let product_id: String = item.get("product_id");
        let qty: f64 = crate::db::get_numeric_as_f64(&item, "qty");

        crate::inventory::process_stock_movement_ledger(
            &mut tx,
            crate::inventory::StockMovementPayload {
                merchant_id: merchant_id.clone(),
                outlet_id: outlet_id.clone(),
                product_id: product_id.clone(),
                movement_type: "CUSTOMER_RETURN".to_string(),
                qty_delta: qty,
                reason: Some(reason.clone()),
                reason_code: Some("CUSTOMER_RETURN".to_string()),
                reference_type: Some("refund".to_string()),
                reference_id: Some(refund_id.to_string()),
                idempotency_key: Some(format!("refund_{}_{}", refund_id, product_id)),
                created_by: user_id.to_string(),
            }
        ).await?;
    }

    // 6. Audit Log
    crate::audit::log_action(
        &mut *tx, 
        merchant_id, 
        Some(outlet_id), 
        user_id.to_string(), 
        "refund_order", 
        "order", 
        Some(order_id.to_string()), 
        Some(&reason)
    ).await?;

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(())
}
