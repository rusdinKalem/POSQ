use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, Row};
use tauri::State;
use uuid::Uuid;

#[derive(Serialize)]
pub struct ProductItem {
    pub id: Uuid,
    pub name: String,
    pub sku: String,
    pub price: i32,
    pub qty_on_hand: f64,
    pub image_url: Option<String>,
}

#[derive(Deserialize)]
pub struct OrderItemPayload {
    pub product_id: Uuid,
    pub sku: String,
    pub name: String,
    pub qty: f64,
    pub unit_price: i32,
    pub discount_total: i32,
    pub line_total: i32,
}

#[derive(Deserialize)]
pub struct CheckoutPayload {
    pub shift_id: Option<Uuid>,
    pub subtotal: i32,
    pub discount_total: i32,
    pub tax_total: i32,
    pub service_total: i32,
    pub grand_total: i32,
    pub paid_total: i32,
    pub change_total: i32,
    pub payment_method: String, // e.g. "CASH"
    pub items: Vec<OrderItemPayload>,
}

#[tauri::command]
pub async fn get_products(pool: State<'_, SqlitePool>) -> Result<Vec<ProductItem>, String> {
    // For MVP, we fetch all active products and their inventory from the first outlet
    let records = sqlx::query(
        r#"
        SELECT p.id, p.name, p.sku, p.price, p.image_url, i.qty_on_hand 
        FROM products p
        JOIN inventory_items i ON i.product_id = p.id
        WHERE p.active = 1
        ORDER BY p.name ASC
        "#
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let products = records.into_iter().map(|r| {
        let qty_on_hand: f64 = crate::db::get_numeric_as_f64(&r, "qty_on_hand");
        ProductItem {
            id: Uuid::parse_str(&r.get::<String, _>("id")).unwrap_or_default(),
            name: r.get("name"),
            sku: r.get("sku"),
            price: r.get("price"),
            qty_on_hand,
            image_url: r.get("image_url"),
        }
    }).collect();

    Ok(products)
}

#[tauri::command]
pub async fn checkout(payload: CheckoutPayload, pool: State<'_, SqlitePool>) -> Result<Uuid, String> {
    // M13 QA Hardening: Enforce license status to prevent API bypass
    crate::license::enforce_active_license().await?;

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    // Get context
    let user = sqlx::query("SELECT id, merchant_id, outlet_id FROM users LIMIT 1")
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    
    let user_id: String = user.get("id");
    let merchant_id: String = user.get("merchant_id");
    let outlet_id: String = user.get::<Option<String>, _>("outlet_id").ok_or("User has no outlet")?;
    
    // Generate order number
    let order_number = format!("ORD-{}", chrono::Utc::now().format("%Y%m%d%H%M%S"));
    let order_id = Uuid::new_v4();

    // Insert Order
    sqlx::query(
        r#"
        INSERT INTO orders (
            id, merchant_id, outlet_id, shift_id, order_number, status, grand_total,
            subtotal, discount_total, tax_total, service_total, paid_total, change_total, created_by, created_at
        ) VALUES (?, ?, ?, ?, ?, 'completed', ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
        "#,
    )
    .bind(order_id.to_string())
    .bind(&merchant_id)
    .bind(&outlet_id)
    .bind(payload.shift_id.map(|id| id.to_string()))
    .bind(&order_number)
    .bind(payload.grand_total)
    .bind(payload.subtotal)
    .bind(payload.discount_total)
    .bind(payload.tax_total)
    .bind(payload.service_total)
    .bind(payload.paid_total)
    .bind(payload.change_total)
    .bind(&user_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    // Insert Items and Stock Movements
    for item in payload.items {
        sqlx::query(
            r#"
            INSERT INTO order_items (id, order_id, product_id, sku, name, qty, unit_price, discount_total, line_total)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(order_id.to_string())
        .bind(item.product_id.to_string())
        .bind(&item.sku)
        .bind(&item.name)
        .bind(item.qty)
        .bind(item.unit_price)
        .bind(item.discount_total)
        .bind(item.line_total)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        // Update Inventory
        let result = sqlx::query(
            r#"
            UPDATE inventory_items 
            SET qty_on_hand = qty_on_hand - ?, updated_at = CURRENT_TIMESTAMP
            WHERE product_id = ? AND outlet_id = ?
            RETURNING qty_on_hand
            "#,
        )
        .bind(item.qty)
        .bind(item.product_id.to_string())
        .bind(&outlet_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| format!("Inventory not found for {}: {}", item.name, e))?;
        
        let new_qty_on_hand: f64 = crate::db::get_numeric_as_f64(&result, "qty_on_hand");
        if new_qty_on_hand < 0.0 {
            // Negative policy: reject!
            return Err(format!("Stok tidak cukup untuk produk: {}", item.name));
        }

        // Insert stock movement
        sqlx::query(
            r#"
            INSERT INTO stock_movements (
                id, merchant_id, outlet_id, product_id, movement_type, qty_delta, reference_type, reference_id, created_by, created_at
            ) VALUES (?, ?, ?, ?, 'sale', ?, 'order', ?, ?, CURRENT_TIMESTAMP)
            "#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(&merchant_id)
        .bind(&outlet_id)
        .bind(item.product_id.to_string())
        .bind(-item.qty)
        .bind(order_id.to_string())
        .bind(&user_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }

    // Insert Payment
    sqlx::query(
        r#"
        INSERT INTO payments (id, merchant_id, outlet_id, order_id, method, status, amount, paid_at, created_at)
        VALUES (?, ?, ?, ?, ?, 'paid', ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(&merchant_id)
    .bind(&outlet_id)
    .bind(order_id.to_string())
    .bind(&payload.payment_method)
    .bind(payload.paid_total)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    // Audit Log
    crate::audit::log_action(
        &mut *tx, 
        merchant_id, 
        Some(outlet_id), 
        user_id, 
        "checkout", 
        "order", 
        Some(order_id.to_string()), 
        None
    ).await?;

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(order_id)
}

#[derive(Serialize)]
pub struct ReceiptPayload {
    pub order_number: String,
    pub created_at: String,
    pub subtotal: i32,
    pub tax_total: i32,
    pub grand_total: i32,
    pub paid_total: i32,
    pub change_total: i32,
    pub items: Vec<ReceiptItem>,
}

#[derive(Serialize)]
pub struct ReceiptItem {
    pub name: String,
    pub qty: f64,
    pub line_total: i32,
}

#[tauri::command]
pub async fn get_receipt(order_id: Uuid, pool: State<'_, SqlitePool>) -> Result<ReceiptPayload, String> {
    use sqlx::Row;
    let order = sqlx::query(
        r#"
        SELECT order_number, created_at, subtotal, tax_total, grand_total, paid_total, change_total
        FROM orders WHERE id = ?
        "#
    )
    .bind(order_id.to_string())
    .fetch_one(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let items = sqlx::query(
        r#"
        SELECT name, qty, line_total
        FROM order_items WHERE order_id = ?
        "#
    )
    .bind(order_id.to_string())
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let receipt_items = items.into_iter().map(|i| {
        let name: String = i.get("name");
        let qty: f64 = crate::db::get_numeric_as_f64(&i, "qty");
        let line_total: i32 = i.get("line_total");
        ReceiptItem {
            name,
            qty,
            line_total,
        }
    }).collect();

    let created_at: String = order.get("created_at");

    Ok(ReceiptPayload {
        order_number: order.get("order_number"),
        created_at,
        subtotal: order.get("subtotal"),
        tax_total: order.get("tax_total"),
        grand_total: order.get("grand_total"),
        paid_total: order.get("paid_total"),
        change_total: order.get("change_total"),
        items: receipt_items,
    })
}
