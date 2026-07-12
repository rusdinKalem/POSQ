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
    pub category_name: Option<String>,
    pub category_id: Option<String>,
}

#[derive(Deserialize)]
pub struct OrderItemPayload {
    pub product_id: Option<Uuid>,
    pub sku: String,
    pub name: String,
    pub qty: f64,
    pub unit_price: i32,
    pub discount_total: i32,
    pub line_total: i32,
    pub notes: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CardDetails {
    pub bank: String,
    pub approval_code: String,
    pub trace_number: String,
    pub edc_mode: String,
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
    pub order_type: Option<String>,
    pub table_number: Option<String>,
    pub items: Vec<OrderItemPayload>,
    pub card_details: Option<CardDetails>,
}

#[tauri::command]
pub async fn get_products(pool: State<'_, SqlitePool>) -> Result<Vec<ProductItem>, String> {
    // 1. Check network mode
    let net_settings = crate::settings::get_network_settings_internal(pool.inner())
        .await
        .unwrap_or(crate::settings::NetworkSettings { 
            mode: "STANDALONE".to_string(), 
            master_ip: "".to_string(),
            cloud_sync_enabled: false,
            cloud_vps_url: "".to_string(),
            cloud_vps_token: "".to_string()
        });

    if net_settings.mode == "CLIENT" && !net_settings.master_ip.is_empty() {
        // Fetch from Master Server using HTTP proxy
        // Expecting Master Server to return Vec<crate::server::ServerProduct>
        // We will map it to ProductItem which the POS frontend expects.
        let server_products: Vec<crate::server::ServerProduct> = 
            crate::proxy::forward_request(&net_settings.master_ip, "products").await?;
            
        let mut mapped_products = Vec::new();
        for sp in server_products {
            use std::str::FromStr;
            let uuid = Uuid::from_str(&sp.id).unwrap_or_else(|_| Uuid::new_v4());
            mapped_products.push(ProductItem {
                id: uuid,
                name: sp.name,
                sku: sp.sku,
                price: sp.unit_price as i32,
                qty_on_hand: sp.stock_quantity,
                image_url: sp.image_path,
                category_name: Some(sp.category),
                category_id: None,
            });
        }
        return Ok(mapped_products);
    }

    // For MVP, we fetch all active products and their inventory from the first outlet
    let records = sqlx::query(
        r#"
        SELECT p.id, p.name, p.sku, p.price, p.image_url, c.name as category_name, p.category_id
        FROM products p
        JOIN inventory_items i ON i.product_id = p.id
        LEFT JOIN categories c ON p.category_id = c.id
        WHERE p.active = 1 AND p.is_ingredient = 0
        ORDER BY p.name ASC
        "#
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let dynamic_stocks = crate::inventory::compute_dynamic_stocks(pool.inner()).await?;

    let products = records.into_iter().map(|r| {
        let id_str: String = r.get("id");
        let qty_on_hand = dynamic_stocks.get(&id_str).copied().unwrap_or(0.0);
        ProductItem {
            id: Uuid::parse_str(&id_str).unwrap_or_default(),
            name: r.get("name"),
            sku: r.get("sku"),
            price: r.get("price"),
            qty_on_hand,
            image_url: r.get("image_url"),
            category_name: r.get("category_name"),
            category_id: r.get("category_id"),
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
            subtotal, discount_total, tax_total, service_total, paid_total, change_total, created_by,
            order_type, table_number, created_at, card_bank, card_approval_code, card_trace_number, edc_mode
        ) VALUES (?, ?, ?, ?, ?, 'completed', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, ?, ?, ?, ?)
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
    .bind(payload.order_type.clone().unwrap_or_else(|| "dine_in".to_string()))
    .bind(payload.table_number.clone())
    .bind(payload.card_details.as_ref().map(|c| c.bank.clone()))
    .bind(payload.card_details.as_ref().map(|c| c.approval_code.clone()))
    .bind(payload.card_details.as_ref().map(|c| c.trace_number.clone()))
    .bind(payload.card_details.as_ref().map(|c| c.edc_mode.clone()))
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    // Insert Items and Stock Movements
    for item in &payload.items {
        sqlx::query(
            r#"
            INSERT INTO order_items (id, order_id, product_id, sku, name, qty, unit_price, discount_total, line_total, notes)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(order_id.to_string())
        .bind(item.product_id.map(|id| id.to_string()))
        .bind(&item.sku)
        .bind(&item.name)
        .bind(item.qty)
        .bind(item.unit_price)
        .bind(item.discount_total)
        .bind(item.line_total)
        .bind(item.notes.clone())
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        // Update Inventory only for non-custom items
        if let Some(product_id) = item.product_id {
            if item.sku != "CUSTOM" {
                // 1. Process Recipe Ingredients if any
                let recipe_ingredients = sqlx::query(
                    r#"
                    SELECT r.ingredient_id, r.qty, r.unit, p.name, p.track_stock
                    FROM product_recipes r
                    JOIN products p ON r.ingredient_id = p.id
                    WHERE r.product_id = ?
                    "#
                )
                .bind(product_id.to_string())
                .fetch_all(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;

                for ing in recipe_ingredients {
                    let ingredient_id: String = ing.get("ingredient_id");
                    let ing_qty_per_item: f64 = crate::db::get_numeric_as_f64(&ing, "qty");
                    let ing_name: String = ing.get("name");
                    let ing_track_stock: bool = ing.get::<i64, _>("track_stock") != 0;
                    let total_deduct = item.qty * ing_qty_per_item;

                    if ing_track_stock {
                        let result = sqlx::query(
                            r#"
                            UPDATE inventory_items 
                            SET qty_on_hand = qty_on_hand - ?, updated_at = CURRENT_TIMESTAMP
                            WHERE product_id = ? AND outlet_id = ?
                            RETURNING qty_on_hand
                            "#,
                        )
                        .bind(total_deduct)
                        .bind(&ingredient_id)
                        .bind(&outlet_id)
                        .fetch_one(&mut *tx)
                        .await;

                        match result {
                            Ok(row) => {
                                let new_qty_on_hand: f64 = crate::db::get_numeric_as_f64(&row, "qty_on_hand");
                                if new_qty_on_hand < 0.0 {
                                    return Err(format!("Stok bahan baku '{}' tidak mencukupi untuk menu '{}'", ing_name, item.name));
                                }
                            }
                            Err(e) => {
                                return Err(format!("Stok bahan baku '{}' tidak ditemukan untuk outlet ini: {}", ing_name, e));
                            }
                        }

                        // Insert stock movement for ingredient
                        sqlx::query(
                            r#"
                            INSERT INTO stock_movements (
                                id, merchant_id, outlet_id, product_id, movement_type, qty_delta, reference_type, reference_id, created_by, created_at
                            ) VALUES (?, ?, ?, ?, 'sale', ?, 'order_ingredient', ?, ?, CURRENT_TIMESTAMP)
                            "#
                        )
                        .bind(Uuid::new_v4().to_string())
                        .bind(&merchant_id)
                        .bind(&outlet_id)
                        .bind(&ingredient_id)
                        .bind(-total_deduct)
                        .bind(order_id.to_string())
                        .bind(&user_id)
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| e.to_string())?;
                    }
                }

                // 2. Process product's own stock if track_stock is enabled
                let track_stock: bool = sqlx::query_scalar("SELECT track_stock FROM products WHERE id = ?")
                    .bind(product_id.to_string())
                    .fetch_one(&mut *tx)
                    .await
                    .unwrap_or(true);

                if track_stock {
                    let result = sqlx::query(
                        r#"
                        UPDATE inventory_items 
                        SET qty_on_hand = qty_on_hand - ?, updated_at = CURRENT_TIMESTAMP
                        WHERE product_id = ? AND outlet_id = ?
                        RETURNING qty_on_hand
                        "#,
                    )
                    .bind(item.qty)
                    .bind(product_id.to_string())
                    .bind(&outlet_id)
                    .fetch_one(&mut *tx)
                    .await;

                    match result {
                        Ok(row) => {
                            let new_qty_on_hand: f64 = crate::db::get_numeric_as_f64(&row, "qty_on_hand");
                            if new_qty_on_hand < 0.0 {
                                return Err(format!("Stok tidak cukup untuk produk: {}", item.name));
                            }
                        }
                        Err(e) => {
                            return Err(format!("Inventory not found for {}: {}", item.name, e));
                        }
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
                    .bind(product_id.to_string())
                    .bind(-item.qty)
                    .bind(order_id.to_string())
                    .bind(&user_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| e.to_string())?;
                }
            }
        }
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
        Some(outlet_id.clone()), 
        user_id.clone(), 
        "checkout", 
        "order", 
        Some(order_id.to_string()), 
        None
    ).await?;

    // Create or Link KDS Ticket
    let mut ticket_linked = false;
    if payload.order_type.as_deref().unwrap_or("dine_in") == "dine_in" {
        if let Some(ref table_no) = payload.table_number {
            if !table_no.is_empty() {
                // Check if there is an active KDS ticket for this table
                let active_draft_ticket: Option<(String,)> = sqlx::query_as(
                    "SELECT id FROM kds_tickets WHERE table_number = ? AND status IN ('pending', 'cooking') AND reference_type = 'draft' LIMIT 1"
                )
                .bind(table_no)
                .fetch_optional(&mut *tx)
                .await
                .unwrap_or(None);

                if let Some((ticket_id,)) = active_draft_ticket {
                    // Serialize final checkout items
                    #[derive(Serialize)]
                    struct KdsItemJson {
                        name: String,
                        qty: f64,
                        notes: Option<String>,
                    }

                    let mut kds_items = Vec::new();
                    for item in &payload.items {
                        kds_items.push(KdsItemJson {
                            name: item.name.clone(),
                            qty: item.qty,
                            notes: item.notes.clone(),
                        });
                    }
                    let items_json = serde_json::to_string(&kds_items).unwrap_or_else(|_| "[]".to_string());

                    // Update existing ticket
                    sqlx::query(
                        "UPDATE kds_tickets SET reference_id = ?, reference_type = 'order', order_number = ?, items_json = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"
                    )
                    .bind(order_id.to_string())
                    .bind(&order_number)
                    .bind(items_json)
                    .bind(ticket_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| e.to_string())?;
                    
                    ticket_linked = true;
                }
            }
        }
    }

    if !ticket_linked {
        // Create new KDS ticket
        #[derive(Serialize)]
        struct KdsItemJson {
            name: String,
            qty: f64,
            notes: Option<String>,
        }

        let mut kds_items = Vec::new();
        for item in &payload.items {
            kds_items.push(KdsItemJson {
                name: item.name.clone(),
                qty: item.qty,
                notes: item.notes.clone(),
            });
        }
        let items_json = serde_json::to_string(&kds_items).unwrap_or_else(|_| "[]".to_string());

        sqlx::query(
            "INSERT INTO kds_tickets (id, reference_id, reference_type, order_number, table_number, order_type, status, items_json, created_at, updated_at) \
             VALUES (?, ?, 'order', ?, ?, ?, 'pending', ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"
        )
        .bind(Uuid::new_v4().to_string())
        .bind(order_id.to_string())
        .bind(&order_number)
        .bind(&payload.table_number)
        .bind(payload.order_type.clone().unwrap_or_else(|| "dine_in".to_string()))
        .bind(items_json)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }

    // F&B Table & Dining Session checkout synchronization
    if payload.order_type.as_deref().unwrap_or("dine_in") == "dine_in" {
        if let Some(ref table_no) = payload.table_number {
            if !table_no.trim().is_empty() {
                // Find table details
                let table_record = sqlx::query("SELECT id FROM tables WHERE name = ? LIMIT 1")
                    .bind(table_no)
                    .fetch_optional(&mut *tx)
                    .await
                    .map_err(|e| e.to_string())?;

                if let Some(tr) = table_record {
                    let table_id: String = tr.get("id");

                    // Check if table has an active dining session
                    let active_session = sqlx::query(
                        "SELECT ds.id FROM dining_sessions ds \
                         JOIN session_tables st ON ds.id = st.dining_session_id \
                         WHERE st.table_id = ? AND ds.status = 'active' LIMIT 1"
                    )
                    .bind(&table_id)
                    .fetch_optional(&mut *tx)
                    .await
                    .map_err(|e| e.to_string())?;

                    let session_id = if let Some(sr) = active_session {
                        sr.get::<String, _>("id")
                    } else {
                        // Create a new temporary active session if one didn't exist
                        let new_session_id = Uuid::new_v4().to_string();
                        sqlx::query("INSERT INTO dining_sessions (id, outlet_id, status) VALUES (?, ?, 'active')")
                            .bind(&new_session_id)
                            .bind(&outlet_id)
                            .execute(&mut *tx)
                            .await
                            .map_err(|e| e.to_string())?;

                        // Create session-table mapping
                        sqlx::query("INSERT INTO session_tables (id, dining_session_id, table_id) VALUES (?, ?, ?)")
                            .bind(Uuid::new_v4().to_string())
                            .bind(&new_session_id)
                            .bind(&table_id)
                            .execute(&mut *tx)
                            .await
                            .map_err(|e| e.to_string())?;

                        new_session_id
                    };

                    // Insert paid bill linked to this order
                    sqlx::query(
                        "INSERT INTO bills (id, dining_session_id, order_id, reference_id, reference_type, status, bill_number, subtotal, discount_total, tax_total, grand_total, paid_total, balance_amount, created_by) \
                         VALUES (?, ?, ?, ?, 'order', 'paid', ?, ?, ?, ?, ?, ?, 0, ?)"
                    )
                    .bind(Uuid::new_v4().to_string())
                    .bind(&session_id)
                    .bind(order_id.to_string())
                    .bind(order_id.to_string())
                    .bind(&order_number)
                    .bind(payload.subtotal)
                    .bind(payload.discount_total)
                    .bind(payload.tax_total)
                    .bind(payload.grand_total)
                    .bind(payload.grand_total)
                    .bind(&user_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| e.to_string())?;

                    // Close the active dining session
                    sqlx::query("UPDATE dining_sessions SET status = 'closed', updated_at = CURRENT_TIMESTAMP WHERE id = ?")
                        .bind(&session_id)
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| e.to_string())?;

                    // Set table to available
                    sqlx::query("UPDATE tables SET status = 'available', version = version + 1, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
                        .bind(&table_id)
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| e.to_string())?;
                }
            }
        }
    }

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
    pub order_type: String,
    pub table_number: Option<String>,
    pub items: Vec<ReceiptItem>,
}

#[derive(Serialize)]
pub struct ReceiptItem {
    pub name: String,
    pub qty: f64,
    pub line_total: i32,
    pub notes: Option<String>,
}

#[tauri::command]
pub async fn get_receipt(order_id: Uuid, pool: State<'_, SqlitePool>) -> Result<ReceiptPayload, String> {
    use sqlx::Row;
    let order = sqlx::query(
        r#"
        SELECT order_number, created_at, subtotal, tax_total, grand_total, paid_total, change_total, order_type, table_number
        FROM orders WHERE id = ?
        "#
    )
    .bind(order_id.to_string())
    .fetch_one(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let items = sqlx::query(
        r#"
        SELECT name, qty, line_total, notes
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
        let notes: Option<String> = i.get("notes");
        ReceiptItem {
            name,
            qty,
            line_total,
            notes,
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
        order_type: order.get("order_type"),
        table_number: order.get("table_number"),
        items: receipt_items,
    })
}

#[tauri::command]
pub async fn save_cart_draft(cart_json: String, pool: State<'_, SqlitePool>) -> Result<(), String> {
    sqlx::query(
        r#"
        INSERT INTO cart_drafts (id, cart_json, updated_at)
        VALUES ('active_cart', ?, CURRENT_TIMESTAMP)
        ON CONFLICT(id) DO UPDATE SET cart_json = excluded.cart_json, updated_at = CURRENT_TIMESTAMP
        "#
    )
    .bind(cart_json)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_cart_draft(pool: State<'_, SqlitePool>) -> Result<Option<String>, String> {
    let record = sqlx::query(
        r#"
        SELECT cart_json FROM cart_drafts WHERE id = 'active_cart'
        "#
    )
    .fetch_optional(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    if let Some(row) = record {
        let cart_json: String = row.get("cart_json");
        Ok(Some(cart_json))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn clear_cart_draft(pool: State<'_, SqlitePool>) -> Result<(), String> {
    sqlx::query("DELETE FROM cart_drafts WHERE id = 'active_cart'")
        .execute(pool.inner())
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Serialize)]
pub struct DraftItem {
    pub id: String,
    pub name: Option<String>,
    pub cart_json: String,
    pub updated_at: String,
}

#[tauri::command]
pub async fn save_hold_draft(id: String, name: String, cart_json: String, pool: State<'_, SqlitePool>) -> Result<(), String> {
    sqlx::query(
        r#"
        INSERT INTO cart_drafts (id, name, cart_json, updated_at)
        VALUES (?, ?, ?, CURRENT_TIMESTAMP)
        ON CONFLICT(id) DO UPDATE SET name = excluded.name, cart_json = excluded.cart_json, updated_at = CURRENT_TIMESTAMP
        "#
    )
    .bind(&id)
    .bind(&name)
    .bind(&cart_json)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    // Create or update KDS ticket for the draft
    #[derive(Deserialize)]
    struct DraftCartItem {
        name: String,
        qty: f64,
        notes: Option<String>,
    }

    #[derive(Deserialize)]
    struct DraftCartPayload {
        cart: Vec<DraftCartItem>,
        #[serde(rename = "orderType")]
        order_type: Option<String>,
        #[serde(rename = "tableNumber")]
        table_number: Option<String>,
    }

    #[derive(Serialize)]
    struct KdsItemJson {
        name: String,
        qty: f64,
        notes: Option<String>,
    }

    if let Ok(draft_payload) = serde_json::from_str::<DraftCartPayload>(&cart_json) {
        let mut kds_items = Vec::new();
        for item in draft_payload.cart {
            kds_items.push(KdsItemJson {
                name: item.name,
                qty: item.qty,
                notes: item.notes,
            });
        }
        let items_json = serde_json::to_string(&kds_items).unwrap_or_else(|_| "[]".to_string());

        sqlx::query(
            "INSERT INTO kds_tickets (id, reference_id, reference_type, order_number, table_number, order_type, status, items_json, created_at, updated_at) \
             VALUES (?, ?, 'draft', ?, ?, ?, 'pending', ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP) \
             ON CONFLICT(reference_id) DO UPDATE SET \
                order_number = excluded.order_number, \
                table_number = excluded.table_number, \
                order_type = excluded.order_type, \
                items_json = excluded.items_json, \
                updated_at = CURRENT_TIMESTAMP"
        )
        .bind(Uuid::new_v4().to_string())
        .bind(&id)
        .bind(&name)
        .bind(draft_payload.table_number.clone())
        .bind(draft_payload.order_type.clone().unwrap_or_else(|| "dine_in".to_string()))
        .bind(items_json)
        .execute(pool.inner())
        .await
        .map_err(|e| e.to_string())?;

        // F&B Table & Dining Session draft synchronization
        if let Some(ref tbl_name) = draft_payload.table_number {
            if !tbl_name.trim().is_empty() {
                let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

                // Get table details
                let table_record = sqlx::query("SELECT id, outlet_id FROM tables WHERE name = ? LIMIT 1")
                    .bind(tbl_name)
                    .fetch_optional(&mut *tx)
                    .await
                    .map_err(|e| e.to_string())?;

                if let Some(tr) = table_record {
                    let table_id: String = tr.get("id");
                    let outlet_id: String = tr.get("outlet_id");

                    // Check if table already has an active dining session
                    let active_session = sqlx::query(
                        "SELECT ds.id FROM dining_sessions ds \
                         JOIN session_tables st ON ds.id = st.dining_session_id \
                         WHERE st.table_id = ? AND ds.status = 'active' LIMIT 1"
                    )
                    .bind(&table_id)
                    .fetch_optional(&mut *tx)
                    .await
                    .map_err(|e| e.to_string())?;

                    let session_id = if let Some(sr) = active_session {
                        sr.get::<String, _>("id")
                    } else {
                        // Create a new dining session
                        let new_session_id = Uuid::new_v4().to_string();
                        sqlx::query("INSERT INTO dining_sessions (id, outlet_id, status) VALUES (?, ?, 'active')")
                            .bind(&new_session_id)
                            .bind(&outlet_id)
                            .execute(&mut *tx)
                            .await
                            .map_err(|e| e.to_string())?;

                        // Create session-table mapping
                        sqlx::query("INSERT INTO session_tables (id, dining_session_id, table_id) VALUES (?, ?, ?)")
                            .bind(Uuid::new_v4().to_string())
                            .bind(&new_session_id)
                            .bind(&table_id)
                            .execute(&mut *tx)
                            .await
                            .map_err(|e| e.to_string())?;

                        // Set table status to occupied
                        sqlx::query("UPDATE tables SET status = 'occupied', version = version + 1, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
                            .bind(&table_id)
                            .execute(&mut *tx)
                            .await
                            .map_err(|e| e.to_string())?;

                        new_session_id
                    };

                    // Compute financial summary from the draft payload for the bill record
                    #[derive(Deserialize)]
                    struct CartItem {
                        unit_price: i32,
                        qty: f64,
                        discount_total: i32,
                    }
                    #[derive(Deserialize)]
                    struct FullDraftPayload {
                        cart: Vec<CartItem>,
                        #[serde(rename = "cartDiscountType")]
                        cart_discount_type: Option<String>,
                        #[serde(rename = "cartDiscountValue")]
                        cart_discount_value: Option<i32>,
                    }

                    let mut subtotal = 0;
                    let mut discount_total = 0;
                    let mut tax_total = 0;
                    let mut grand_total = 0;

                    if let Ok(full_payload) = serde_json::from_str::<FullDraftPayload>(&cart_json) {
                        for item in &full_payload.cart {
                            subtotal += (item.unit_price as f64 * item.qty) as i32 - item.discount_total;
                        }
                        // Apply cart discount if any
                        let disc_val = full_payload.cart_discount_value.unwrap_or(0);
                        if let Some(ref disc_type) = full_payload.cart_discount_type {
                            if disc_type == "percent" {
                                discount_total = (subtotal as f64 * (disc_val as f64 / 100.0)) as i32;
                            } else {
                                discount_total = disc_val;
                            }
                        }
                        if discount_total > subtotal {
                            discount_total = subtotal;
                        }
                        tax_total = ((subtotal - discount_total) as f64 * 0.11) as i32;
                        grand_total = subtotal - discount_total + tax_total;
                    }

                    // Get created_by user
                    let user_rec = sqlx::query("SELECT id FROM users LIMIT 1")
                        .fetch_one(&mut *tx)
                        .await
                        .map_err(|e| e.to_string())?;
                    let user_id: String = user_rec.get("id");

                    // Check if bill exists
                    let existing_bill = sqlx::query("SELECT id FROM bills WHERE reference_id = ? AND reference_type = 'draft' LIMIT 1")
                        .bind(&id)
                        .fetch_optional(&mut *tx)
                        .await
                        .map_err(|e| e.to_string())?;

                    if let Some(eb) = existing_bill {
                        let bill_id: String = eb.get("id");
                        sqlx::query(
                            "UPDATE bills SET dining_session_id = ?, subtotal = ?, discount_total = ?, tax_total = ?, grand_total = ?, balance_amount = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"
                        )
                        .bind(&session_id)
                        .bind(subtotal)
                        .bind(discount_total)
                        .bind(tax_total)
                        .bind(grand_total)
                        .bind(grand_total)
                        .bind(bill_id)
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| e.to_string())?;
                    } else {
                        sqlx::query(
                            "INSERT INTO bills (id, dining_session_id, reference_id, reference_type, status, bill_number, subtotal, discount_total, tax_total, grand_total, balance_amount, created_by) \
                             VALUES (?, ?, ?, 'draft', 'open', ?, ?, ?, ?, ?, ?, ?)"
                        )
                        .bind(Uuid::new_v4().to_string())
                        .bind(&session_id)
                        .bind(&id)
                        .bind(&name)
                        .bind(subtotal)
                        .bind(discount_total)
                        .bind(tax_total)
                        .bind(grand_total)
                        .bind(grand_total)
                        .bind(user_id)
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| e.to_string())?;
                    }
                }

                tx.commit().await.map_err(|e| e.to_string())?;
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn list_hold_drafts(pool: State<'_, SqlitePool>) -> Result<Vec<DraftItem>, String> {
    let records = sqlx::query(
        r#"
        SELECT id, name, cart_json, updated_at FROM cart_drafts
        WHERE id != 'active_cart'
        ORDER BY updated_at DESC
        "#
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let drafts = records.into_iter().map(|r| {
        DraftItem {
            id: r.get("id"),
            name: r.get("name"),
            cart_json: r.get("cart_json"),
            updated_at: r.get("updated_at"),
        }
    }).collect();

    Ok(drafts)
}

#[tauri::command]
pub async fn delete_hold_draft(id: String, pool: State<'_, SqlitePool>) -> Result<(), String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    // 1. Find the bill for this draft to know its dining_session_id
    let bill_rec = sqlx::query("SELECT dining_session_id FROM bills WHERE reference_id = ? AND reference_type = 'draft' LIMIT 1")
        .bind(&id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    // 2. Delete from cart_drafts
    sqlx::query("DELETE FROM cart_drafts WHERE id = ?")
        .bind(&id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    // 3. Delete from kds_tickets
    sqlx::query("DELETE FROM kds_tickets WHERE reference_id = ? AND reference_type = 'draft'")
        .bind(&id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    // 4. Delete the bill
    sqlx::query("DELETE FROM bills WHERE reference_id = ? AND reference_type = 'draft'")
        .bind(&id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    // 5. If the session has no more active bills, close the dining session and set table(s) to available!
    if let Some(br) = bill_rec {
        let session_id: String = br.get("dining_session_id");

        let active_bills_count = sqlx::query(
            "SELECT COUNT(*) as count FROM bills WHERE dining_session_id = ? AND status = 'open'"
        )
        .bind(&session_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        let count: i64 = active_bills_count.get("count");

        if count == 0 {
            // Close the dining session
            sqlx::query("UPDATE dining_sessions SET status = 'closed', updated_at = CURRENT_TIMESTAMP WHERE id = ?")
                .bind(&session_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;

            // Set all tables in this session to available
            sqlx::query(
                "UPDATE tables SET status = 'available', version = version + 1, updated_at = CURRENT_TIMESTAMP \
                 WHERE id IN (SELECT table_id FROM session_tables WHERE dining_session_id = ?)"
            )
            .bind(&session_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
        }
    }

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}
