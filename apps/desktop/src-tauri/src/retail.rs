use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, Row};
use tauri::{command, State};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct ReturnResult {
    pub success: bool,
    pub message: String,
    pub refund_amount: f64,
}

#[command]
pub async fn process_return(order_id: String, reason: String, refund_amount: f64, pool: State<'_, SqlitePool>) -> Result<ReturnResult, String> {
    // SEC-001: Enforce license status to prevent API bypass
    crate::license::enforce_active_license().await?;

    let parsed_order_id = Uuid::parse_str(&order_id).map_err(|_| "Invalid Order ID UUID")?;

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    // 1. Validate order exists and get context
    let order = sqlx::query(
        "SELECT merchant_id, outlet_id, status FROM orders WHERE id = ?"
    )
    .bind(parsed_order_id.to_string())
    .fetch_one(&mut *tx)
    .await
    .map_err(|_| "Pesanan tidak ditemukan.")?;

    let status: String = order.get("status");
    if status == "refunded" {
        return Err("Pesanan ini sudah pernah diretur.".into());
    }

    let merchant_id: String = order.get("merchant_id");
    let outlet_id: String = order.get("outlet_id");

    let user = sqlx::query("SELECT id FROM users LIMIT 1")
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| "User context not found.")?;
    
    let user_id: String = user.get("id");

    // 2. Fetch items to return
    let items = sqlx::query(
        "SELECT product_id, qty FROM order_items WHERE order_id = ? AND product_id IS NOT NULL"
    )
    .bind(parsed_order_id.to_string())
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    // 3. Return stock to inventory & record movement
    for item in items {
        let product_id: String = item.get("product_id");
        let qty: f64 = crate::db::get_numeric_as_f64(&item, "qty");

        // Revert inventory qty
        sqlx::query(
            "UPDATE inventory_items SET qty_on_hand = qty_on_hand + ?, updated_at = CURRENT_TIMESTAMP WHERE product_id = ? AND outlet_id = ?"
        )
        .bind(qty)
        .bind(&product_id)
        .bind(&outlet_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        // Insert stock movement as RETURN
        sqlx::query(
            r#"
            INSERT INTO stock_movements (
                id, merchant_id, outlet_id, product_id, movement_type, qty_delta, reason, reference_type, reference_id, created_by, created_at
            ) VALUES (?, ?, ?, ?, 'refund', ?, ?, 'order', ?, ?, CURRENT_TIMESTAMP)
            "#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(&merchant_id)
        .bind(&outlet_id)
        .bind(&product_id)
        .bind(qty)
        .bind(&reason)
        .bind(parsed_order_id.to_string())
        .bind(&user_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }

    // 4. Update order status
    sqlx::query(
        "UPDATE orders SET status = 'refunded', updated_at = CURRENT_TIMESTAMP WHERE id = ?"
    )
    .bind(parsed_order_id.to_string())
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    // 5. Create refund record
    sqlx::query(
        r#"
        INSERT INTO refunds (id, order_id, amount, reason, approved_by, created_by, created_at)
        VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(parsed_order_id.to_string())
    .bind(refund_amount as i32)
    .bind(&reason)
    .bind(&user_id)
    .bind(&user_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    // 6. Audit Log
    crate::audit::log_action(
        &mut *tx,
        merchant_id,
        Some(outlet_id),
        user_id,
        "process_return",
        "order",
        Some(parsed_order_id.to_string()),
        Some(&reason)
    ).await?;

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(ReturnResult {
        success: true,
        message: "Proses retur berhasil, persediaan barang dikembalikan.".into(),
        refund_amount,
    })
}
