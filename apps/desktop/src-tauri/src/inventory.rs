use sqlx::{SqlitePool, Row};
use tauri::State;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize)]
pub struct LowStockItem {
    pub product_id: Uuid,
    pub name: String,
    pub sku: String,
    pub qty_on_hand: f64,
    pub min_qty: f64,
}

pub async fn compute_dynamic_stocks(
    pool: &SqlitePool,
) -> Result<HashMap<String, f64>, String> {
    // Get outlet_id for current user
    let user_record = sqlx::query("SELECT outlet_id FROM users LIMIT 1")
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
    let outlet_id: String = user_record.get::<Option<String>, _>("outlet_id").ok_or("User has no outlet")?;

    // 1. Get all inventory items qty_on_hand for this outlet
    let inv_records = sqlx::query(
        "SELECT product_id, qty_on_hand FROM inventory_items WHERE outlet_id = ?"
    )
    .bind(&outlet_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut stock_map: HashMap<String, f64> = inv_records
        .into_iter()
        .map(|r| {
            let pid: String = r.get("product_id");
            let qty = crate::db::get_numeric_as_f64(&r, "qty_on_hand");
            (pid, qty)
        })
        .collect();

    // 2. Get all recipe ingredients
    let recipe_records = sqlx::query(
        "SELECT product_id, ingredient_id, qty FROM product_recipes"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    // Group recipes by product_id
    // product_id -> Vec<(ingredient_id, qty_needed)>
    let mut recipes: HashMap<String, Vec<(String, f64)>> = HashMap::new();
    for r in recipe_records {
        let pid: String = r.get("product_id");
        let ing_id: String = r.get("ingredient_id");
        let qty = crate::db::get_numeric_as_f64(&r, "qty");
        recipes.entry(pid).or_default().push((ing_id, qty));
    }

    // 3. For each product that has a recipe, compute its bottleneck stock
    let mut dynamic_stocks = HashMap::new();

    for (pid, ingredients) in &recipes {
        if ingredients.is_empty() {
            continue;
        }

        let mut min_portions = f64::MAX;
        for (ing_id, qty_needed) in ingredients {
            if *qty_needed <= 0.0 {
                continue;
            }
            let ing_stock = stock_map.get(ing_id).copied().unwrap_or(0.0);
            let portions = ing_stock / qty_needed;
            if portions < min_portions {
                min_portions = portions;
            }
        }

        if min_portions == f64::MAX {
            min_portions = 0.0;
        } else {
            min_portions = min_portions.floor();
            if min_portions < 0.0 {
                min_portions = 0.0;
            }
        }
        dynamic_stocks.insert(pid.clone(), min_portions);
    }

    // Override the stock map with dynamic stocks
    for (pid, dyn_qty) in dynamic_stocks {
        stock_map.insert(pid, dyn_qty);
    }

    Ok(stock_map)
}

#[tauri::command]
pub async fn get_low_stock(pool: State<'_, SqlitePool>) -> Result<Vec<LowStockItem>, String> {
    let user_id = crate::auth::get_current_user(pool.inner()).await?;
    let has_perm = crate::auth::has_permission(pool.inner(), user_id, "inventory.manage").await?;
    if !has_perm {
        return Err("Akses ditolak: Anda tidak memiliki izin untuk melihat laporan inventaris".to_string());
    }

    recalculate_all_min_stocks(pool.inner()).await?;

    let dynamic_stocks = compute_dynamic_stocks(pool.inner()).await?;

    let records = sqlx::query(
        r#"
        SELECT p.id as product_id, p.name, p.sku, i.min_qty
        FROM products p
        JOIN inventory_items i ON p.id = i.product_id
        WHERE p.track_stock = 1
        "#
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let mut items = Vec::new();
    for r in records {
        let product_id_str: String = r.get("product_id");
        let qty_on_hand = dynamic_stocks.get(&product_id_str).copied().unwrap_or(0.0);
        let min_qty: f64 = crate::db::get_numeric_as_f64(&r, "min_qty");
        if qty_on_hand <= min_qty {
            items.push(LowStockItem {
                product_id: Uuid::parse_str(&product_id_str).unwrap_or_default(),
                name: r.get("name"),
                sku: r.get("sku"),
                qty_on_hand,
                min_qty,
            });
        }
    }

    items.sort_by(|a, b| a.qty_on_hand.partial_cmp(&b.qty_on_hand).unwrap_or(std::cmp::Ordering::Equal));

    Ok(items)
}

pub struct StockMovementPayload {
    pub merchant_id: String,
    pub outlet_id: String,
    pub product_id: String,
    pub movement_type: String,
    pub qty_delta: f64,
    pub reason: Option<String>,
    pub reason_code: Option<String>,
    pub reference_type: Option<String>,
    pub reference_id: Option<String>,
    pub idempotency_key: Option<String>,
    pub created_by: String,
}

pub async fn process_stock_movement_ledger(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    payload: StockMovementPayload,
) -> Result<(f64, f64), String> {
    // 1. Check Idempotency Key
    if let Some(ref ik) = payload.idempotency_key {
        if !ik.trim().is_empty() {
            let existing = sqlx::query(
                "SELECT stock_before, stock_after FROM stock_movements WHERE outlet_id = ? AND idempotency_key = ? LIMIT 1"
            )
            .bind(&payload.outlet_id)
            .bind(ik)
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| e.to_string())?;

            if let Some(row) = existing {
                let sb: f64 = crate::db::get_numeric_as_f64(&row, "stock_before");
                let sa: f64 = crate::db::get_numeric_as_f64(&row, "stock_after");
                return Ok((sb, sa));
            }
        }
    }

    // 2. Fetch or initialize inventory item for product
    let current_rec = sqlx::query(
        "SELECT merchant_id, outlet_id, qty_on_hand FROM inventory_items WHERE outlet_id = ? AND product_id = ?"
    )
    .bind(&payload.outlet_id)
    .bind(&payload.product_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;

    let stock_before = if let Some(rec) = current_rec {
        crate::db::get_numeric_as_f64(&rec, "qty_on_hand")
    } else {
        // Initialize inventory item record if missing
        let inv_id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO inventory_items (id, merchant_id, outlet_id, product_id, qty_on_hand, min_qty, updated_at) VALUES (?, ?, ?, ?, 0.0, 0.0, CURRENT_TIMESTAMP)"
        )
        .bind(&inv_id)
        .bind(&payload.merchant_id)
        .bind(&payload.outlet_id)
        .bind(&payload.product_id)
        .execute(&mut **tx)
        .await
        .map_err(|e| e.to_string())?;
        0.0
    };

    let is_reservation_movement = payload.movement_type == "RESERVATION" || payload.movement_type == "RELEASE_RESERVATION";
    let stock_after = if is_reservation_movement {
        stock_before
    } else {
        stock_before + payload.qty_delta
    };

    // 3. Negative Stock Policy Check (only for physical deductions)
    if !is_reservation_movement && stock_after < 0.0 {
        let allow_neg: String = sqlx::query_scalar("SELECT value FROM system_settings WHERE key = 'allow_negative_stock'")
            .fetch_optional(&mut **tx)
            .await
            .unwrap_or(None)
            .unwrap_or_else(|| "0".to_string());

        if allow_neg != "1" && allow_neg.to_lowercase() != "true" {
            let prod_name: String = sqlx::query_scalar("SELECT name FROM products WHERE id = ?")
                .bind(&payload.product_id)
                .fetch_optional(&mut **tx)
                .await
                .unwrap_or(None)
                .unwrap_or_else(|| payload.product_id.clone());

            return Err(format!("Stok tidak mencukupi untuk '{}' (tersedia: {}, dibutuhkan: {}). Kebijakan stok negatif dinonaktifkan.", prod_name, stock_before, -payload.qty_delta));
        }
    }

    // 4. Update inventory_items qty_on_hand (only for physical movements)
    if !is_reservation_movement {
        sqlx::query(
            "UPDATE inventory_items SET qty_on_hand = ?, updated_at = CURRENT_TIMESTAMP WHERE outlet_id = ? AND product_id = ?"
        )
        .bind(stock_after)
        .bind(&payload.outlet_id)
        .bind(&payload.product_id)
        .execute(&mut **tx)
        .await
        .map_err(|e| e.to_string())?;
    }

    // 5. Insert immutable stock_movements record
    let movement_id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO stock_movements (
            id, merchant_id, outlet_id, product_id, movement_type, qty_delta, stock_before, stock_after,
            reason, reason_code, reference_type, reference_id, idempotency_key, status, created_by, created_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'posted', ?, CURRENT_TIMESTAMP)
        "#
    )
    .bind(&movement_id)
    .bind(&payload.merchant_id)
    .bind(&payload.outlet_id)
    .bind(&payload.product_id)
    .bind(&payload.movement_type)
    .bind(payload.qty_delta)
    .bind(stock_before)
    .bind(stock_after)
    .bind(&payload.reason)
    .bind(&payload.reason_code)
    .bind(payload.reference_type.as_deref().unwrap_or("manual"))
    .bind(&payload.reference_id)
    .bind(&payload.idempotency_key)
    .bind(&payload.created_by)
    .execute(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;

    // 6. Record audit log
    crate::audit::log_action(
        &mut **tx,
        payload.merchant_id.clone(),
        Some(payload.outlet_id.clone()),
        payload.created_by.clone(),
        &payload.movement_type,
        "inventory",
        Some(payload.product_id.clone()),
        payload.reason.as_deref()
    ).await?;

    Ok((stock_before, stock_after))
}

#[tauri::command]
pub async fn stock_in(product_id: Uuid, qty: f64, reason: Option<String>, pool: State<'_, SqlitePool>) -> Result<(), String> {
    if qty <= 0.0 {
        return Err("Qty harus lebih dari 0".to_string());
    }
    process_stock_movement(product_id, qty, "STOCK_ADJUSTMENT_IN", reason, pool.inner()).await
}

#[tauri::command]
pub async fn adjust_stock(product_id: Uuid, qty_delta: f64, reason: String, pool: State<'_, SqlitePool>) -> Result<(), String> {
    if reason.trim().is_empty() {
        return Err("Alasan (reason) wajib diisi untuk penyesuaian stok".to_string());
    }
    let mtype = if qty_delta >= 0.0 { "STOCK_ADJUSTMENT_IN" } else { "STOCK_ADJUSTMENT_OUT" };
    process_stock_movement(product_id, qty_delta, mtype, Some(reason), pool.inner()).await
}

#[tauri::command]
pub async fn stock_opname(product_id: Uuid, actual_qty: f64, reason: String, pool: State<'_, SqlitePool>) -> Result<(), String> {
    crate::license::enforce_active_license().await?;

    if reason.trim().is_empty() {
        return Err("Alasan (reason) wajib diisi untuk stock opname".to_string());
    }

    let user_id = crate::auth::get_current_user(pool.inner()).await?;
    let has_perm = crate::auth::has_permission(pool.inner(), user_id, "inventory.manage").await?;
    if !has_perm {
        return Err("Akses ditolak: Anda tidak memiliki izin untuk mengelola inventaris".to_string());
    }

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let user_rec = sqlx::query("SELECT merchant_id, outlet_id FROM users WHERE id = ?")
        .bind(user_id.to_string())
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    let merchant_id: String = user_rec.get("merchant_id");
    let outlet_id: String = user_rec.get::<Option<String>, _>("outlet_id").ok_or("User has no outlet")?;

    let current = sqlx::query("SELECT qty_on_hand FROM inventory_items WHERE outlet_id = ? AND product_id = ?")
        .bind(&outlet_id)
        .bind(product_id.to_string())
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    let stock_before = if let Some(rec) = current {
        crate::db::get_numeric_as_f64(&rec, "qty_on_hand")
    } else {
        0.0
    };

    let qty_delta = actual_qty - stock_before;

    process_stock_movement_ledger(
        &mut tx,
        StockMovementPayload {
            merchant_id,
            outlet_id,
            product_id: product_id.to_string(),
            movement_type: "STOCK_OPNAME".to_string(),
            qty_delta,
            reason: Some(reason),
            reason_code: Some("STOCK_OPNAME".to_string()),
            reference_type: Some("opname".to_string()),
            reference_id: None,
            idempotency_key: Some(format!("opname_{}_{}", product_id, chrono::Utc::now().timestamp_millis())),
            created_by: user_id.to_string(),
        }
    ).await?;

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn transfer_stock(product_id: Uuid, qty: f64, _destination_outlet: String, reason: String, pool: State<'_, SqlitePool>) -> Result<(), String> {
    if qty <= 0.0 {
        return Err("Qty harus lebih dari 0".to_string());
    }
    process_stock_movement(product_id, -qty, "TRANSFER_OUT", Some(reason), pool.inner()).await
}

async fn process_stock_movement(product_id: Uuid, qty_delta: f64, movement_type: &str, reason: Option<String>, pool: &SqlitePool) -> Result<(), String> {
    crate::license::enforce_active_license().await?;

    let user_id = crate::auth::get_current_user(pool).await?;
    let has_perm = crate::auth::has_permission(pool, user_id, "inventory.manage").await?;
    if !has_perm {
        return Err("Akses ditolak: Anda tidak memiliki izin untuk mengelola inventaris".to_string());
    }

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let user_rec = sqlx::query("SELECT merchant_id, outlet_id FROM users WHERE id = ?")
        .bind(user_id.to_string())
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    let merchant_id: String = user_rec.get("merchant_id");
    let outlet_id: String = user_rec.get::<Option<String>, _>("outlet_id").ok_or("User has no outlet")?;

    process_stock_movement_ledger(
        &mut tx,
        StockMovementPayload {
            merchant_id,
            outlet_id,
            product_id: product_id.to_string(),
            movement_type: movement_type.to_string(),
            qty_delta,
            reason,
            reason_code: Some(movement_type.to_string()),
            reference_type: Some("manual".to_string()),
            reference_id: None,
            idempotency_key: Some(format!("manual_{}_{}_{}", movement_type, product_id, chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0))),
            created_by: user_id.to_string(),
        }
    ).await?;

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn recalculate_min_stock(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    product_id: &str,
) -> Result<(), String> {
    let is_ingredient: i64 = sqlx::query_scalar("SELECT CAST(is_ingredient AS INTEGER) FROM products WHERE id = ?")
        .bind(product_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| e.to_string())?;

    let prod_record = sqlx::query("SELECT CAST(COALESCE(min_stock_factor, 0.0) AS REAL) as factor, CAST(COALESCE(buffer_stock, 0.0) AS REAL) as buffer, COALESCE(lead_time_days, 0) as lead_time FROM products WHERE id = ?")
        .bind(product_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| e.to_string())?;

    let min_stock_factor: f64 = prod_record.get("factor");
    let buffer_stock: f64 = prod_record.get("buffer");
    let lead_time_days: i32 = prod_record.get("lead_time");

    let min_qty = if is_ingredient != 0 {
        let sum_qty: Option<f64> = sqlx::query_scalar("SELECT CAST(SUM(qty) AS REAL) FROM product_recipes WHERE ingredient_id = ?")
            .bind(product_id)
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| e.to_string())?;

        let avg_daily_consumption: f64 = sqlx::query_scalar(
            r#"
            SELECT CAST(COALESCE(SUM(oi.qty * pr.qty), 0.0) / 7.0 AS REAL)
            FROM order_items oi
            JOIN orders o ON oi.order_id = o.id
            JOIN product_recipes pr ON oi.product_id = pr.product_id
            WHERE pr.ingredient_id = ? 
              AND o.created_at >= datetime('now', '-7 days')
            "#
        )
        .bind(product_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| e.to_string())?;

        let calculated_min = (sum_qty.unwrap_or(0.0) * min_stock_factor) + (avg_daily_consumption * (lead_time_days as f64));
        if calculated_min > buffer_stock {
            calculated_min
        } else {
            buffer_stock
        }
    } else {
        buffer_stock
    };

    sqlx::query("UPDATE inventory_items SET min_qty = ? WHERE product_id = ?")
        .bind(min_qty)
        .bind(product_id)
        .execute(&mut **tx)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn recalculate_all_min_stocks(pool: &SqlitePool) -> Result<(), String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let product_ids: Vec<String> = sqlx::query_scalar("SELECT id FROM products WHERE active = 1")
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    for pid in product_ids {
        let is_ingredient: i64 = sqlx::query_scalar("SELECT CAST(is_ingredient AS INTEGER) FROM products WHERE id = ?")
            .bind(&pid)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

        let prod_record = sqlx::query("SELECT CAST(COALESCE(min_stock_factor, 0.0) AS REAL) as factor, CAST(COALESCE(buffer_stock, 0.0) AS REAL) as buffer, COALESCE(lead_time_days, 0) as lead_time FROM products WHERE id = ?")
            .bind(&pid)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

        let min_stock_factor: f64 = prod_record.get("factor");
        let buffer_stock: f64 = prod_record.get("buffer");
        let lead_time_days: i32 = prod_record.get("lead_time");

        let min_qty = if is_ingredient != 0 {
            let sum_qty: Option<f64> = sqlx::query_scalar("SELECT CAST(SUM(qty) AS REAL) FROM product_recipes WHERE ingredient_id = ?")
                .bind(&pid)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;

            let avg_daily_consumption: f64 = sqlx::query_scalar(
                r#"
                SELECT CAST(COALESCE(SUM(oi.qty * pr.qty), 0.0) / 7.0 AS REAL)
                FROM order_items oi
                JOIN orders o ON oi.order_id = o.id
                JOIN product_recipes pr ON oi.product_id = pr.product_id
                WHERE pr.ingredient_id = ? 
                  AND o.created_at >= datetime('now', '-7 days')
                "#
            )
            .bind(&pid)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

            let calculated_min = (sum_qty.unwrap_or(0.0) * min_stock_factor) + (avg_daily_consumption * (lead_time_days as f64));
            if calculated_min > buffer_stock {
                calculated_min
            } else {
                buffer_stock
            }
        } else {
            buffer_stock
        };

        sqlx::query("UPDATE inventory_items SET min_qty = ? WHERE product_id = ?")
            .bind(min_qty)
            .bind(&pid)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn create_product(
    name: String,
    sku: String,
    price: i32,
    cost: Option<i32>,
    category_id: Option<String>,
    track_stock: bool,
    initial_qty: f64,
    image_url: Option<String>,
    is_ingredient: Option<bool>,
    min_stock_factor: Option<f64>,
    buffer_stock: Option<f64>,
    lead_time_days: Option<i32>,
    pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    // SEC-001: Enforce license status to prevent API bypass
    crate::license::enforce_active_license().await?;

    let user_id = crate::auth::get_current_user(pool.inner()).await?;
    let has_perm = crate::auth::has_permission(pool.inner(), user_id, "inventory.manage").await?;
    if !has_perm {
        return Err("Akses ditolak: Anda tidak memiliki izin untuk mengelola inventaris".to_string());
    }

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    // Get merchant and outlet
    let user_record = sqlx::query("SELECT merchant_id, outlet_id FROM users WHERE id = ?")
        .bind(user_id.to_string())
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    let merchant_id: String = user_record.get("merchant_id");
    let outlet_id: String = user_record.get::<Option<String>, _>("outlet_id").ok_or("User has no outlet")?;

    // Check if SKU already exists
    let sku_exists = sqlx::query("SELECT 1 FROM products WHERE merchant_id = ? AND sku = ?")
        .bind(&merchant_id)
        .bind(&sku)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    if sku_exists.is_some() {
        return Err(format!("SKU '{}' sudah terdaftar.", sku));
    }

    // Resolve category
    let mut resolved_cat_id: Option<String> = None;
    if let Some(cat_id) = category_id {
        let cat_id_trimmed = cat_id.trim().to_string();
        if !cat_id_trimmed.is_empty() {
            resolved_cat_id = Some(cat_id_trimmed);
        }
    }

    // Insert Product
    let product_id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO products (id, merchant_id, category_id, sku, name, price, cost, track_stock, image_url, active, is_ingredient, min_stock_factor, buffer_stock, lead_time_days, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 1, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        "#
    )
    .bind(&product_id)
    .bind(&merchant_id)
    .bind(resolved_cat_id)
    .bind(&sku)
    .bind(&name)
    .bind(price)
    .bind(cost)
    .bind(track_stock)
    .bind(image_url)
    .bind(is_ingredient.unwrap_or(false))
    .bind(min_stock_factor.unwrap_or(0.0))
    .bind(buffer_stock.unwrap_or(0.0))
    .bind(lead_time_days.unwrap_or(0))
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    // Insert Inventory Item
    let inv_id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO inventory_items (id, merchant_id, outlet_id, product_id, qty_on_hand, min_qty, updated_at)
        VALUES (?, ?, ?, ?, ?, 0, CURRENT_TIMESTAMP)
        "#
    )
    .bind(inv_id)
    .bind(&merchant_id)
    .bind(&outlet_id)
    .bind(&product_id)
    .bind(initial_qty)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    recalculate_min_stock(&mut tx, &product_id).await?;

    // Stock Movement
    if initial_qty > 0.0 {
        let movement_id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"
            INSERT INTO stock_movements (id, merchant_id, outlet_id, product_id, movement_type, qty_delta, reason, reference_type, created_by, created_at)
            VALUES (?, ?, ?, ?, 'stock_in', ?, 'Inisialisasi Produk Baru', 'manual', ?, CURRENT_TIMESTAMP)
            "#
        )
        .bind(movement_id)
        .bind(&merchant_id)
        .bind(&outlet_id)
        .bind(&product_id)
        .bind(initial_qty)
        .bind(user_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }

    // Audit Log
    crate::audit::log_action(
        &mut *tx,
        merchant_id,
        Some(outlet_id),
        user_id.to_string(),
        "create_product",
        "product",
        Some(product_id),
        Some(&format!("Nama: {}, SKU: {}, Harga: {}", name, sku, price))
    ).await?;

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn import_products_csv(
    csv_content: String,
    pool: State<'_, SqlitePool>,
) -> Result<String, String> {
    // SEC-001: Enforce license status to prevent API bypass
    crate::license::enforce_active_license().await?;

    let user_id = crate::auth::get_current_user(pool.inner()).await?;
    let has_perm = crate::auth::has_permission(pool.inner(), user_id, "inventory.manage").await?;
    if !has_perm {
        return Err("Akses ditolak: Anda tidak memiliki izin untuk mengelola inventaris".to_string());
    }

    // Get merchant and outlet
    let user_record = sqlx::query("SELECT merchant_id, outlet_id FROM users WHERE id = ?")
        .bind(user_id.to_string())
        .fetch_one(pool.inner())
        .await
        .map_err(|e| e.to_string())?;

    let merchant_id: String = user_record.get("merchant_id");
    let outlet_id: String = user_record.get::<Option<String>, _>("outlet_id").ok_or("User has no outlet")?;

    let mut rdr = csv::Reader::from_reader(csv_content.as_bytes());
    let mut success_count = 0;
    let mut skip_count = 0;

    #[derive(serde::Deserialize)]
    struct CsvRecord {
        name: String,
        sku: String,
        category: Option<String>,
        price: i32,
        cost: Option<i32>,
        track_stock: Option<bool>,
        initial_qty: Option<f64>,
        image_url: Option<String>,
    }

    for result in rdr.deserialize::<CsvRecord>() {
        let record = match result {
            Ok(r) => r,
            Err(e) => return Err(format!("Format baris CSV tidak valid: {}", e)),
        };

        let sku = record.sku.trim().to_string();
        let name = record.name.trim().to_string();
        if sku.is_empty() || name.is_empty() {
            skip_count += 1;
            continue;
        }

        let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

        // Check if SKU already exists (Skip duplicate SKU policy)
        let sku_exists = sqlx::query("SELECT 1 FROM products WHERE merchant_id = ? AND sku = ?")
            .bind(&merchant_id)
            .bind(&sku)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

        if sku_exists.is_some() {
            skip_count += 1;
            tx.rollback().await.map_err(|e| e.to_string())?;
            continue;
        }

        // Resolve or create category
        let mut category_id: Option<String> = None;
        if let Some(cat_name) = record.category {
            let cat_name_trimmed = cat_name.trim().to_string();
            if !cat_name_trimmed.is_empty() {
                let cat_record = sqlx::query("SELECT id FROM categories WHERE merchant_id = ? AND name = ?")
                    .bind(&merchant_id)
                    .bind(&cat_name_trimmed)
                    .fetch_optional(&mut *tx)
                    .await
                    .map_err(|e| e.to_string())?;

                if let Some(row) = cat_record {
                    category_id = Some(row.get("id"));
                } else {
                    let new_cat_id = Uuid::new_v4().to_string();
                    sqlx::query("INSERT INTO categories (id, merchant_id, name) VALUES (?, ?, ?)")
                        .bind(&new_cat_id)
                        .bind(&merchant_id)
                        .bind(&cat_name_trimmed)
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| e.to_string())?;
                    category_id = Some(new_cat_id);
                }
            }
        }

        let track_stock = record.track_stock.unwrap_or(true);
        let initial_qty = record.initial_qty.unwrap_or(0.0);

        // Insert Product
        let product_id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"
            INSERT INTO products (id, merchant_id, category_id, sku, name, price, cost, track_stock, image_url, active, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            "#
        )
        .bind(&product_id)
        .bind(&merchant_id)
        .bind(category_id)
        .bind(&sku)
        .bind(&name)
        .bind(record.price)
        .bind(record.cost)
        .bind(track_stock)
        .bind(record.image_url)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        // Insert Inventory Item
        let inv_id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"
            INSERT INTO inventory_items (id, merchant_id, outlet_id, product_id, qty_on_hand, min_qty, updated_at)
            VALUES (?, ?, ?, ?, ?, 0, CURRENT_TIMESTAMP)
            "#
        )
        .bind(inv_id)
        .bind(&merchant_id)
        .bind(&outlet_id)
        .bind(&product_id)
        .bind(initial_qty)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        // Stock Movement
        if initial_qty > 0.0 {
            let movement_id = Uuid::new_v4().to_string();
            sqlx::query(
                r#"
                INSERT INTO stock_movements (id, merchant_id, outlet_id, product_id, movement_type, qty_delta, reason, reference_type, created_by, created_at)
                VALUES (?, ?, ?, ?, 'stock_in', ?, 'Inisialisasi Impor Produk', 'manual', ?, CURRENT_TIMESTAMP)
                "#
            )
            .bind(movement_id)
            .bind(&merchant_id)
            .bind(&outlet_id)
            .bind(&product_id)
            .bind(initial_qty)
            .bind(user_id.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
        }

        // Audit Log
        crate::audit::log_action(
            &mut *tx,
            merchant_id.clone(),
            Some(outlet_id.clone()),
            user_id.to_string(),
            "import_product",
            "product",
            Some(product_id),
            Some(&format!("Nama: {}, SKU: {}", name, sku))
        ).await?;

        tx.commit().await.map_err(|e| e.to_string())?;
        success_count += 1;
    }

    Ok(format!(
        "Impor CSV berhasil: {} produk berhasil ditambahkan, {} produk dilewati (SKU duplikat/tidak valid).",
        success_count, skip_count
    ))
}

#[derive(Serialize)]
pub struct DetailedProductItem {
    pub id: String,
    pub name: String,
    pub sku: String,
    pub price: i32,
    pub cost: Option<i32>,
    pub category_name: Option<String>,
    pub category_id: Option<String>,
    pub track_stock: bool,
    pub image_url: Option<String>,
    pub qty_on_hand: f64,
    pub min_qty: f64,
    pub is_ingredient: bool,
    pub min_stock_factor: f64,
    pub buffer_stock: f64,
    pub lead_time_days: i32,
}

#[tauri::command]
pub async fn get_inventory_products(pool: State<'_, SqlitePool>) -> Result<Vec<DetailedProductItem>, String> {
    recalculate_all_min_stocks(pool.inner()).await?;

    let records = sqlx::query(
        r#"
        SELECT p.id, p.name, p.sku, p.price, p.cost, c.name as category_name, p.category_id, p.track_stock, p.image_url, i.min_qty, p.is_ingredient, p.min_stock_factor, p.buffer_stock, p.lead_time_days
        FROM products p
        JOIN inventory_items i ON i.product_id = p.id
        LEFT JOIN categories c ON p.category_id = c.id
        WHERE p.active = 1
        ORDER BY p.name ASC
        "#
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let dynamic_stocks = compute_dynamic_stocks(pool.inner()).await?;

    let products = records.into_iter().map(|r| {
        let id: String = r.get("id");
        let qty_on_hand = dynamic_stocks.get(&id).copied().unwrap_or(0.0);
        let min_qty = crate::db::get_numeric_as_f64(&r, "min_qty");
        let min_stock_factor = crate::db::get_numeric_as_f64(&r, "min_stock_factor");
        let buffer_stock = crate::db::get_numeric_as_f64(&r, "buffer_stock");
        let lead_time_days = r.get::<i32, _>("lead_time_days");
        DetailedProductItem {
            id,
            name: r.get("name"),
            sku: r.get("sku"),
            price: r.get("price"),
            cost: r.get("cost"),
            category_name: r.get("category_name"),
            category_id: r.get("category_id"),
            track_stock: r.get::<i64, _>("track_stock") != 0,
            image_url: r.get("image_url"),
            qty_on_hand,
            min_qty,
            is_ingredient: r.get::<i64, _>("is_ingredient") != 0,
            min_stock_factor,
            buffer_stock,
            lead_time_days,
        }
    }).collect();

    Ok(products)
}

#[tauri::command]
pub async fn update_product(
    id: String,
    name: String,
    sku: String,
    price: i32,
    cost: Option<i32>,
    category_id: Option<String>,
    track_stock: bool,
    image_url: Option<String>,
    is_ingredient: Option<bool>,
    min_stock_factor: Option<f64>,
    buffer_stock: Option<f64>,
    lead_time_days: Option<i32>,
    pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    // SEC-001: Enforce license status to prevent API bypass
    crate::license::enforce_active_license().await?;

    let user_id = crate::auth::get_current_user(pool.inner()).await?;
    let has_perm = crate::auth::has_permission(pool.inner(), user_id, "inventory.manage").await?;
    if !has_perm {
        return Err("Akses ditolak: Anda tidak memiliki izin untuk mengelola inventaris".to_string());
    }

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    // Get merchant_id
    let merchant_id: String = sqlx::query_scalar("SELECT merchant_id FROM users WHERE id = ?")
        .bind(user_id.to_string())
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    // Check SKU uniqueness (exclude this product)
    let sku_exists = sqlx::query("SELECT 1 FROM products WHERE merchant_id = ? AND sku = ? AND id != ?")
        .bind(&merchant_id)
        .bind(&sku)
        .bind(&id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    if sku_exists.is_some() {
        return Err(format!("SKU '{}' sudah terdaftar pada produk lain.", sku));
    }

    // Resolve category
    let mut resolved_cat_id: Option<String> = None;
    if let Some(cat_id) = category_id {
        let cat_id_trimmed = cat_id.trim().to_string();
        if !cat_id_trimmed.is_empty() {
            resolved_cat_id = Some(cat_id_trimmed);
        }
    }

    // Update Product
    sqlx::query(
        r#"
        UPDATE products 
        SET category_id = ?, sku = ?, name = ?, price = ?, cost = ?, track_stock = ?, image_url = ?, is_ingredient = ?, min_stock_factor = ?, buffer_stock = ?, lead_time_days = ?, updated_at = CURRENT_TIMESTAMP
        WHERE id = ? AND merchant_id = ?
        "#
    )
    .bind(resolved_cat_id)
    .bind(sku)
    .bind(name)
    .bind(price)
    .bind(cost)
    .bind(track_stock)
    .bind(image_url)
    .bind(is_ingredient.unwrap_or(false))
    .bind(min_stock_factor.unwrap_or(0.0))
    .bind(buffer_stock.unwrap_or(0.0))
    .bind(lead_time_days.unwrap_or(0))
    .bind(&id)
    .bind(&merchant_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    recalculate_min_stock(&mut tx, &id).await?;

    // Audit Log
    crate::audit::log_action(
        &mut *tx,
        merchant_id,
        None,
        user_id.to_string(),
        "update_product",
        "product",
        Some(id),
        None
    ).await?;

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(())
}

#[derive(Serialize)]
pub struct StockMovementItem {
    pub id: String,
    pub movement_type: String,
    pub qty_delta: f64,
    pub reason: Option<String>,
    pub reference_type: Option<String>,
    pub reference_id: Option<String>,
    pub actor_name: String,
    pub created_at: String,
}

#[tauri::command]
pub async fn get_product_movements(
    product_id: String,
    pool: State<'_, SqlitePool>,
) -> Result<Vec<StockMovementItem>, String> {
    // SEC-001: Enforce license status to prevent API bypass
    crate::license::enforce_active_license().await?;

    let user_id = crate::auth::get_current_user(pool.inner()).await?;
    let has_perm = crate::auth::has_permission(pool.inner(), user_id, "inventory.manage").await?;
    if !has_perm {
        return Err("Akses ditolak: Anda tidak memiliki izin untuk mengelola inventaris".to_string());
    }

    let records = sqlx::query(
        r#"
        SELECT m.id, m.movement_type, m.qty_delta, m.reason, m.reference_type, m.reference_id, u.name as actor_name, m.created_at
        FROM stock_movements m
        LEFT JOIN users u ON m.created_by = u.id
        WHERE m.product_id = ?
        ORDER BY m.created_at DESC
        "#
    )
    .bind(product_id)
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let movements = records.into_iter().map(|r| {
        let qty_delta: f64 = crate::db::get_numeric_as_f64(&r, "qty_delta");
        StockMovementItem {
            id: r.get("id"),
            movement_type: r.get("movement_type"),
            qty_delta,
            reason: r.get("reason"),
            reference_type: r.get("reference_type"),
            reference_id: r.get("reference_id"),
            actor_name: r.get::<Option<String>, _>("actor_name").unwrap_or_else(|| "Sistem".to_string()),
            created_at: r.get("created_at"),
        }
    }).collect();

    Ok(movements)
}

#[derive(Serialize)]
pub struct CategoryItem {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub parent_name: Option<String>,
    pub business_mode: Option<String>,
}

#[tauri::command]
pub async fn list_categories(pool: State<'_, SqlitePool>) -> Result<Vec<CategoryItem>, String> {
    let records = sqlx::query(
        r#"
        SELECT c.id, c.name, c.parent_id, p.name as parent_name, c.business_mode
        FROM categories c
        LEFT JOIN categories p ON c.parent_id = p.id
        ORDER BY c.name ASC
        "#
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let categories = records.into_iter().map(|r| {
        CategoryItem {
            id: r.get("id"),
            name: r.get("name"),
            parent_id: r.get("parent_id"),
            parent_name: r.get("parent_name"),
            business_mode: r.get("business_mode"),
        }
    }).collect();

    Ok(categories)
}

#[tauri::command]
pub async fn create_category(
    name: String,
    parent_id: Option<String>,
    business_mode: Option<String>,
    pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    crate::license::enforce_active_license().await?;
    let user_id = crate::auth::get_current_user(pool.inner()).await?;
    
    let merchant_id: String = sqlx::query_scalar("SELECT merchant_id FROM users WHERE id = ?")
        .bind(user_id.to_string())
        .fetch_one(pool.inner())
        .await
        .map_err(|e| e.to_string())?;

    let name_trimmed = name.trim();
    if name_trimmed.is_empty() {
        return Err("Nama kategori tidak boleh kosong.".to_string());
    }

    // Check if category/subcategory name already exists
    let cat_exists = if let Some(ref p_id) = parent_id {
        sqlx::query("SELECT 1 FROM categories WHERE merchant_id = ? AND name = ? AND parent_id = ?")
            .bind(&merchant_id)
            .bind(name_trimmed)
            .bind(p_id)
            .fetch_optional(pool.inner())
            .await
            .map_err(|e| e.to_string())?
    } else {
        sqlx::query("SELECT 1 FROM categories WHERE merchant_id = ? AND name = ? AND parent_id IS NULL")
            .bind(&merchant_id)
            .bind(name_trimmed)
            .fetch_optional(pool.inner())
            .await
            .map_err(|e| e.to_string())?
    };

    if cat_exists.is_some() {
        return Err(format!("Kategori/Sub-kategori '{}' sudah ada.", name_trimmed));
    }

    let id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO categories (id, merchant_id, name, parent_id, business_mode) VALUES (?, ?, ?, ?, ?)")
        .bind(id)
        .bind(merchant_id)
        .bind(name_trimmed)
        .bind(parent_id)
        .bind(business_mode)
        .execute(pool.inner())
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn update_category(
    id: String,
    name: String,
    parent_id: Option<String>,
    business_mode: Option<String>,
    pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    crate::license::enforce_active_license().await?;
    let name_trimmed = name.trim();
    if name_trimmed.is_empty() {
        return Err("Nama kategori tidak boleh kosong.".to_string());
    }

    // Check circular dependency
    if let Some(ref p_id) = parent_id {
        if p_id == &id {
            return Err("Kategori tidak bisa menjadi anak dari dirinya sendiri.".to_string());
        }
    }

    sqlx::query("UPDATE categories SET name = ?, parent_id = ?, business_mode = ? WHERE id = ?")
        .bind(name_trimmed)
        .bind(parent_id)
        .bind(business_mode)
        .bind(id)
        .execute(pool.inner())
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn delete_category(id: String, pool: State<'_, SqlitePool>) -> Result<(), String> {
    crate::license::enforce_active_license().await?;
    
    let is_used = sqlx::query("SELECT 1 FROM products WHERE category_id = ? LIMIT 1")
        .bind(&id)
        .fetch_optional(pool.inner())
        .await
        .map_err(|e| e.to_string())?;

    if is_used.is_some() {
        return Err("Kategori tidak dapat dihapus karena masih digunakan oleh produk.".to_string());
    }

    let has_children = sqlx::query("SELECT 1 FROM categories WHERE parent_id = ? LIMIT 1")
        .bind(&id)
        .fetch_optional(pool.inner())
        .await
        .map_err(|e| e.to_string())?;

    if has_children.is_some() {
        return Err("Kategori tidak dapat dihapus karena memiliki sub-kategori.".to_string());
    }

    sqlx::query("DELETE FROM categories WHERE id = ?")
        .bind(id)
        .execute(pool.inner())
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[derive(serde::Deserialize)]
pub struct RecipeIngredientInput {
    pub ingredient_id: String,
    pub qty: f64,
    pub unit: String,
}

#[derive(Serialize)]
pub struct RecipeIngredientItem {
    pub ingredient_id: String,
    pub name: String,
    pub sku: String,
    pub qty: f64,
    pub unit: String,
}

#[tauri::command]
pub async fn get_recipe_ingredients(product_id: String, pool: State<'_, SqlitePool>) -> Result<Vec<RecipeIngredientItem>, String> {
    let records = sqlx::query(
        r#"
        SELECT r.ingredient_id, p.name, p.sku, r.qty, r.unit
        FROM product_recipes r
        JOIN products p ON r.ingredient_id = p.id
        WHERE r.product_id = ?
        ORDER BY p.name ASC
        "#
    )
    .bind(product_id)
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let items = records.into_iter().map(|r| {
        let qty: f64 = crate::db::get_numeric_as_f64(&r, "qty");
        RecipeIngredientItem {
            ingredient_id: r.get("ingredient_id"),
            name: r.get("name"),
            sku: r.get("sku"),
            qty,
            unit: r.get("unit"),
        }
    }).collect();

    Ok(items)
}

#[tauri::command]
pub async fn save_recipe(
    product_id: String,
    ingredients: Vec<RecipeIngredientInput>,
    pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    crate::license::enforce_active_license().await?;
    
    let mut tx = pool.inner().begin().await.map_err(|e| e.to_string())?;

    // Get existing ingredient IDs before deleting to recalculate them later
    let old_ingredients: Vec<String> = sqlx::query_scalar("SELECT ingredient_id FROM product_recipes WHERE product_id = ?")
        .bind(&product_id)
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    // Delete existing recipe ingredients
    sqlx::query("DELETE FROM product_recipes WHERE product_id = ?")
        .bind(&product_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    // Insert new ones
    for ing in &ingredients {
        sqlx::query(
            r#"
            INSERT INTO product_recipes (id, product_id, ingredient_id, qty, unit)
            VALUES (?, ?, ?, ?, ?)
            "#
        )
        .bind(Uuid::new_v4().to_string())
        .bind(&product_id)
        .bind(&ing.ingredient_id)
        .bind(ing.qty)
        .bind(&ing.unit)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }

    // Gather all unique ingredient IDs (old and new)
    let mut unique_ingredients = std::collections::HashSet::new();
    for id in old_ingredients {
        unique_ingredients.insert(id);
    }
    for ing in ingredients {
        unique_ingredients.insert(ing.ingredient_id);
    }

    // Recalculate minimum stock for each unique ingredient
    for ing_id in unique_ingredients {
        recalculate_min_stock(&mut tx, &ing_id).await?;
    }

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OmnichannelStockBreakdown {
    pub product_id: String,
    pub location_id: String,
    pub qty_on_hand: f64,
    pub qty_reserved: f64,
    pub qty_available: f64,
    pub qty_in_transit: f64,
    pub qty_damaged_or_quarantine: f64,
}

pub async fn get_product_stock_breakdown(
    pool: &SqlitePool,
    location_id: &str,
    product_id: &str,
) -> Result<OmnichannelStockBreakdown, String> {
    let row = sqlx::query(
        r#"
        SELECT product_id, COALESCE(location_id, outlet_id) as location_id,
               qty_on_hand, qty_reserved, qty_in_transit, qty_damaged_or_quarantine
        FROM inventory_items
        WHERE (outlet_id = ? OR location_id = ?) AND product_id = ?
        LIMIT 1
        "#
    )
    .bind(location_id)
    .bind(location_id)
    .bind(product_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(r) = row {
        let qoh = crate::db::get_numeric_as_f64(&r, "qty_on_hand");
        let qres = crate::db::get_numeric_as_f64(&r, "qty_reserved");
        let qtransit = crate::db::get_numeric_as_f64(&r, "qty_in_transit");
        let qdamaged = crate::db::get_numeric_as_f64(&r, "qty_damaged_or_quarantine");
        let qavail = (qoh - qres - qdamaged).max(0.0);

        Ok(OmnichannelStockBreakdown {
            product_id: product_id.to_string(),
            location_id: location_id.to_string(),
            qty_on_hand: qoh,
            qty_reserved: qres,
            qty_available: qavail,
            qty_in_transit: qtransit,
            qty_damaged_or_quarantine: qdamaged,
        })
    } else {
        Ok(OmnichannelStockBreakdown {
            product_id: product_id.to_string(),
            location_id: location_id.to_string(),
            qty_on_hand: 0.0,
            qty_reserved: 0.0,
            qty_available: 0.0,
            qty_in_transit: 0.0,
            qty_damaged_or_quarantine: 0.0,
        })
    }
}

pub async fn reserve_omnichannel_stock(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    merchant_id: &str,
    location_id: &str,
    product_id: &str,
    qty: f64,
    order_id: &str,
    user_id: &str,
) -> Result<(), String> {
    if qty <= 0.0 {
        return Ok(());
    }

    let row = sqlx::query(
        "SELECT qty_on_hand, qty_reserved, qty_damaged_or_quarantine FROM inventory_items WHERE (outlet_id = ? OR location_id = ?) AND product_id = ?"
    )
    .bind(location_id)
    .bind(location_id)
    .bind(product_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;

    let (qoh, qres, qdam) = if let Some(r) = row {
        (
            crate::db::get_numeric_as_f64(&r, "qty_on_hand"),
            crate::db::get_numeric_as_f64(&r, "qty_reserved"),
            crate::db::get_numeric_as_f64(&r, "qty_damaged_or_quarantine"),
        )
    } else {
        (0.0, 0.0, 0.0)
    };

    let qavail = qoh - qres - qdam;
    if qavail < qty {
        let allow_neg: String = sqlx::query_scalar("SELECT value FROM system_settings WHERE key = 'allow_negative_stock'")
            .fetch_optional(&mut **tx)
            .await
            .unwrap_or(None)
            .unwrap_or_else(|| "0".to_string());

        if allow_neg != "1" && allow_neg.to_lowercase() != "true" {
            return Err(format!("Stok tersedia ({}) tidak mencukupi untuk reservasi (dibutuhkan: {}).", qavail, qty));
        }
    }

    sqlx::query(
        "UPDATE inventory_items SET qty_reserved = qty_reserved + ?, updated_at = CURRENT_TIMESTAMP WHERE (outlet_id = ? OR location_id = ?) AND product_id = ?"
    )
    .bind(qty)
    .bind(location_id)
    .bind(location_id)
    .bind(product_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;

    let ik = format!("reserve_{}_{}", order_id, product_id);
    let movement_payload = StockMovementPayload {
        merchant_id: merchant_id.to_string(),
        outlet_id: location_id.to_string(),
        product_id: product_id.to_string(),
        movement_type: "RESERVATION".to_string(),
        qty_delta: qty,
        reason: Some(format!("Reservasi stok untuk order {}", order_id)),
        reason_code: Some("RESERVATION".to_string()),
        reference_type: Some("order_reservation".to_string()),
        reference_id: Some(order_id.to_string()),
        idempotency_key: Some(ik),
        created_by: user_id.to_string(),
    };

    let _ = process_stock_movement_ledger(tx, movement_payload).await?;
    Ok(())
}

pub async fn release_omnichannel_reservation(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    merchant_id: &str,
    location_id: &str,
    product_id: &str,
    qty: f64,
    order_id: &str,
    user_id: &str,
) -> Result<(), String> {
    if qty <= 0.0 {
        return Ok(());
    }

    let ik = format!("release_{}_{}", order_id, product_id);
    let existing = sqlx::query_scalar::<_, String>("SELECT id FROM stock_movements WHERE idempotency_key = ?")
        .bind(&ik)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| e.to_string())?;

    if existing.is_some() {
        return Ok(());
    }

    sqlx::query(
        "UPDATE inventory_items SET qty_reserved = MAX(0.0, qty_reserved - ?), updated_at = CURRENT_TIMESTAMP WHERE (outlet_id = ? OR location_id = ?) AND product_id = ?"
    )
    .bind(qty)
    .bind(location_id)
    .bind(location_id)
    .bind(product_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;

    let movement_payload = StockMovementPayload {
        merchant_id: merchant_id.to_string(),
        outlet_id: location_id.to_string(),
        product_id: product_id.to_string(),
        movement_type: "RELEASE_RESERVATION".to_string(),
        qty_delta: -qty,
        reason: Some(format!("Pelepasan reservasi stok untuk order {}", order_id)),
        reason_code: Some("RELEASE_RESERVATION".to_string()),
        reference_type: Some("release_reservation".to_string()),
        reference_id: Some(order_id.to_string()),
        idempotency_key: Some(ik),
        created_by: user_id.to_string(),
    };

    let _ = process_stock_movement_ledger(tx, movement_payload).await?;
    Ok(())
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OmnichannelItemPayload {
    pub product_id: Option<String>,
    pub sku: Option<String>,
    pub name: String,
    pub qty: f64,
    pub price: i32,
    pub notes: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateOmnichannelOrderPayload {
    pub channel: String,
    pub external_order_id: Option<String>,
    pub items: Vec<OmnichannelItemPayload>,
    pub fulfilment_type: String,
    pub location_id: Option<String>,
}

#[tauri::command]
pub async fn create_omnichannel_order(
    payload: CreateOmnichannelOrderPayload,
    pool: State<'_, SqlitePool>,
) -> Result<String, String> {
    crate::license::enforce_active_license().await?;
    let user_id = crate::auth::get_current_user(pool.inner()).await?;

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let user_record = sqlx::query("SELECT merchant_id, outlet_id FROM users WHERE id = ?")
        .bind(user_id.to_string())
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    let merchant_id: String = user_record.get("merchant_id");
    let outlet_id: String = user_record.get::<Option<String>, _>("outlet_id").ok_or("User has no outlet")?;
    let loc_id = payload.location_id.unwrap_or_else(|| outlet_id.clone());

    if let Some(ref ext_id) = payload.external_order_id {
        let existing: Option<String> = sqlx::query_scalar("SELECT id FROM orders WHERE merchant_id = ? AND channel = ? AND external_order_id = ?")
            .bind(&merchant_id)
            .bind(&payload.channel)
            .bind(ext_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

        if let Some(ord_id) = existing {
            tx.commit().await.map_err(|e| e.to_string())?;
            return Ok(ord_id);
        }
    }

    let order_id = Uuid::new_v4().to_string();
    let order_number = format!("ORD-{}-{}", payload.channel.to_uppercase(), chrono::Utc::now().format("%Y%m%d%H%M%S"));

    let mut subtotal: i32 = 0;
    for item in &payload.items {
        subtotal += item.price * (item.qty as i32);
    }

    sqlx::query(
        r#"
        INSERT INTO orders (
            id, merchant_id, outlet_id, order_number, status, channel, external_order_id,
            payment_status, fulfilment_status, fulfilment_location_id, grand_total, subtotal,
            created_by, created_at
        ) VALUES (?, ?, ?, ?, 'completed', ?, ?, 'paid', 'allocated', ?, ?, ?, ?, CURRENT_TIMESTAMP)
        "#
    )
    .bind(&order_id)
    .bind(&merchant_id)
    .bind(&outlet_id)
    .bind(&order_number)
    .bind(&payload.channel)
    .bind(&payload.external_order_id)
    .bind(&loc_id)
    .bind(subtotal)
    .bind(subtotal)
    .bind(user_id.to_string())
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    for item in &payload.items {
        let item_id = Uuid::new_v4().to_string();
        let line_total = item.price * (item.qty as i32);
        sqlx::query(
            r#"
            INSERT INTO order_items (id, order_id, product_id, sku, name, qty, unit_price, line_total, notes)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&item_id)
        .bind(&order_id)
        .bind(&item.product_id)
        .bind(&item.sku)
        .bind(&item.name)
        .bind(item.qty)
        .bind(item.price)
        .bind(line_total)
        .bind(&item.notes)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        if let Some(ref prod_id) = item.product_id {
            reserve_omnichannel_stock(&mut tx, &merchant_id, &loc_id, prod_id, item.qty, &order_id, &user_id.to_string()).await?;
        }
    }

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(order_id)
}

#[tauri::command]
pub async fn fulfill_omnichannel_order(
    order_id: String,
    pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    crate::license::enforce_active_license().await?;
    let user_id = crate::auth::get_current_user(pool.inner()).await?;

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let order = sqlx::query("SELECT merchant_id, outlet_id, COALESCE(fulfilment_location_id, outlet_id) as location_id, fulfilment_status FROM orders WHERE id = ?")
        .bind(&order_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Order tidak ditemukan")?;

    let merchant_id: String = order.get("merchant_id");
    let loc_id: String = order.get("location_id");
    let current_status: String = order.get("fulfilment_status");

    if current_status == "completed" {
        tx.commit().await.map_err(|e| e.to_string())?;
        return Ok(());
    }

    let items = sqlx::query("SELECT product_id, qty FROM order_items WHERE order_id = ? AND product_id IS NOT NULL")
        .bind(&order_id)
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    for item in items {
        let prod_id: String = item.get("product_id");
        let qty = crate::db::get_numeric_as_f64(&item, "qty");

        // Deduct physically and release reservation
        let ik = format!("fulfill_{}_{}", order_id, prod_id);
        let _ = process_stock_movement_ledger(&mut tx, StockMovementPayload {
            merchant_id: merchant_id.clone(),
            outlet_id: loc_id.clone(),
            product_id: prod_id.clone(),
            movement_type: "SALE".to_string(),
            qty_delta: -qty,
            reason: Some(format!("Order {} penyerahan / fulfilment", order_id)),
            reason_code: Some("FULFILMENT".to_string()),
            reference_type: Some("order_fulfilment".to_string()),
            reference_id: Some(order_id.clone()),
            idempotency_key: Some(ik),
            created_by: user_id.to_string(),
        }).await?;

        // Release reservation
        sqlx::query(
            "UPDATE inventory_items SET qty_reserved = MAX(0.0, qty_reserved - ?), updated_at = CURRENT_TIMESTAMP WHERE (outlet_id = ? OR location_id = ?) AND product_id = ?"
        )
        .bind(qty)
        .bind(&loc_id)
        .bind(&loc_id)
        .bind(&prod_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }

    sqlx::query("UPDATE orders SET fulfilment_status = 'completed', updated_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(&order_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn cancel_omnichannel_order(
    order_id: String,
    pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    crate::license::enforce_active_license().await?;
    let user_id = crate::auth::get_current_user(pool.inner()).await?;

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let order = sqlx::query("SELECT merchant_id, outlet_id, COALESCE(fulfilment_location_id, outlet_id) as location_id, fulfilment_status FROM orders WHERE id = ?")
        .bind(&order_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Order tidak ditemukan")?;

    let merchant_id: String = order.get("merchant_id");
    let loc_id: String = order.get("location_id");
    let current_status: String = order.get("fulfilment_status");

    if current_status == "cancelled" {
        tx.commit().await.map_err(|e| e.to_string())?;
        return Ok(());
    }

    let items = sqlx::query("SELECT product_id, qty FROM order_items WHERE order_id = ? AND product_id IS NOT NULL")
        .bind(&order_id)
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    for item in items {
        let prod_id: String = item.get("product_id");
        let qty = crate::db::get_numeric_as_f64(&item, "qty");
        release_omnichannel_reservation(&mut tx, &merchant_id, &loc_id, &prod_id, qty, &order_id, &user_id.to_string()).await?;
    }

    sqlx::query("UPDATE orders SET fulfilment_status = 'cancelled', payment_status = 'refunded', updated_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(&order_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::establish_connection;
    use crate::migration::run_migrations;

    async fn setup_test_db() -> SqlitePool {
        use sqlx::sqlite::SqlitePoolOptions;
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        run_migrations(&pool).await.unwrap();
        crate::seed::run_seed(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_inventory_ledger_idempotency() {
        let pool = setup_test_db().await;
        let mut tx = pool.begin().await.unwrap();

        let (user_id, merchant_id, outlet_id): (String, String, String) = sqlx::query_as(
            "SELECT id, merchant_id, outlet_id FROM users LIMIT 1"
        )
        .fetch_one(&mut *tx)
        .await
        .unwrap();

        let prod_id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO products (id, merchant_id, sku, name, price, track_stock, active) VALUES (?, ?, 'TEST-SKU-1', 'Test Item 1', 10000, 1, 1)"
        )
        .bind(&prod_id)
        .bind(&merchant_id)
        .execute(&mut *tx)
        .await
        .unwrap();

        let payload = StockMovementPayload {
            merchant_id: merchant_id.clone(),
            outlet_id: outlet_id.clone(),
            product_id: prod_id.clone(),
            movement_type: "INITIAL_STOCK".to_string(),
            qty_delta: 50.0,
            reason: Some("Init Test".to_string()),
            reason_code: Some("INIT".to_string()),
            reference_type: Some("test".to_string()),
            reference_id: Some("ref-1".to_string()),
            idempotency_key: Some("unique_key_123".to_string()),
            created_by: user_id.clone(),
        };

        // First execution
        let (sb1, sa1) = process_stock_movement_ledger(&mut tx, payload).await.unwrap();
        assert_eq!(sb1, 0.0);
        assert_eq!(sa1, 50.0);

        // Retry execution with SAME idempotency_key
        let payload_retry = StockMovementPayload {
            merchant_id: merchant_id.clone(),
            outlet_id: outlet_id.clone(),
            product_id: prod_id.clone(),
            movement_type: "INITIAL_STOCK".to_string(),
            qty_delta: 50.0,
            reason: Some("Init Test Retry".to_string()),
            reason_code: Some("INIT".to_string()),
            reference_type: Some("test".to_string()),
            reference_id: Some("ref-1".to_string()),
            idempotency_key: Some("unique_key_123".to_string()),
            created_by: user_id.clone(),
        };

        let (sb2, sa2) = process_stock_movement_ledger(&mut tx, payload_retry).await.unwrap();
        assert_eq!(sb2, 0.0);
        assert_eq!(sa2, 50.0); // Stock is NOT added again! Total remains 50.0

        let row = sqlx::query("SELECT qty_on_hand FROM inventory_items WHERE product_id = ?")
            .bind(&prod_id)
            .fetch_one(&mut *tx)
            .await
            .unwrap();
        let final_qty = crate::db::get_numeric_as_f64(&row, "qty_on_hand");
        assert_eq!(final_qty, 50.0);

        tx.commit().await.unwrap();
    }

    #[tokio::test]
    async fn test_sale_and_refund_reversal() {
        let pool = setup_test_db().await;
        let mut tx = pool.begin().await.unwrap();

        let (user_id, merchant_id, outlet_id): (String, String, String) = sqlx::query_as(
            "SELECT id, merchant_id, outlet_id FROM users LIMIT 1"
        )
        .fetch_one(&mut *tx)
        .await
        .unwrap();

        let prod_id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO products (id, merchant_id, sku, name, price, track_stock, active) VALUES (?, ?, 'TEST-SKU-SALE', 'Sale Item', 10000, 1, 1)"
        )
        .bind(&prod_id)
        .bind(&merchant_id)
        .execute(&mut *tx)
        .await
        .unwrap();

        // 1. Initial stock 10
        process_stock_movement_ledger(&mut tx, StockMovementPayload {
            merchant_id: merchant_id.clone(),
            outlet_id: outlet_id.clone(),
            product_id: prod_id.clone(),
            movement_type: "INITIAL_STOCK".to_string(),
            qty_delta: 10.0,
            reason: Some("Init".to_string()),
            reason_code: None,
            reference_type: None,
            reference_id: None,
            idempotency_key: Some("init_sale_1".to_string()),
            created_by: user_id.clone(),
        }).await.unwrap();

        // 2. Sale 3 items
        let (sb_sale, sa_sale) = process_stock_movement_ledger(&mut tx, StockMovementPayload {
            merchant_id: merchant_id.clone(),
            outlet_id: outlet_id.clone(),
            product_id: prod_id.clone(),
            movement_type: "SALE".to_string(),
            qty_delta: -3.0,
            reason: Some("Sale ord-1".to_string()),
            reason_code: Some("SALE".to_string()),
            reference_type: Some("order".to_string()),
            reference_id: Some("ord-1".to_string()),
            idempotency_key: Some("checkout_ord1_prod1".to_string()),
            created_by: user_id.clone(),
        }).await.unwrap();

        assert_eq!(sb_sale, 10.0);
        assert_eq!(sa_sale, 7.0);

        // 3. Customer Return / Refund 3 items
        let (sb_ref, sa_ref) = process_stock_movement_ledger(&mut tx, StockMovementPayload {
            merchant_id: merchant_id.clone(),
            outlet_id: outlet_id.clone(),
            product_id: prod_id.clone(),
            movement_type: "CUSTOMER_RETURN".to_string(),
            qty_delta: 3.0,
            reason: Some("Refund ord-1".to_string()),
            reason_code: Some("CUSTOMER_RETURN".to_string()),
            reference_type: Some("refund".to_string()),
            reference_id: Some("ref-1".to_string()),
            idempotency_key: Some("refund_ref1_prod1".to_string()),
            created_by: user_id.clone(),
        }).await.unwrap();

        assert_eq!(sb_ref, 7.0);
        assert_eq!(sa_ref, 10.0);

        tx.commit().await.unwrap();
    }

    #[tokio::test]
    async fn test_negative_stock_policy() {
        let pool = setup_test_db().await;
        let mut tx = pool.begin().await.unwrap();

        let (user_id, merchant_id, outlet_id): (String, String, String) = sqlx::query_as(
            "SELECT id, merchant_id, outlet_id FROM users LIMIT 1"
        )
        .fetch_one(&mut *tx)
        .await
        .unwrap();

        let prod_id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO products (id, merchant_id, sku, name, price, track_stock, active) VALUES (?, ?, 'TEST-SKU-NEG', 'Test Neg Item', 10000, 1, 1)"
        )
        .bind(&prod_id)
        .bind(&merchant_id)
        .execute(&mut *tx)
        .await
        .unwrap();

        // Ensure allow_negative_stock is 0
        sqlx::query("INSERT OR REPLACE INTO system_settings (key, value) VALUES ('allow_negative_stock', '0')")
            .execute(&mut *tx)
            .await
            .unwrap();

        let payload_sale = StockMovementPayload {
            merchant_id: merchant_id.clone(),
            outlet_id: outlet_id.clone(),
            product_id: prod_id.clone(),
            movement_type: "SALE".to_string(),
            qty_delta: -5.0,
            reason: Some("Deduct overflow".to_string()),
            reason_code: None,
            reference_type: Some("order".to_string()),
            reference_id: None,
            idempotency_key: Some("neg_test_1".to_string()),
            created_by: user_id.clone(),
        };

        // Negative stock should be rejected when policy = 0
        let res = process_stock_movement_ledger(&mut tx, payload_sale).await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Kebijakan stok negatif dinonaktifkan"));

        // Enable negative stock policy = 1
        sqlx::query("INSERT OR REPLACE INTO system_settings (key, value) VALUES ('allow_negative_stock', '1')")
            .execute(&mut *tx)
            .await
            .unwrap();

        let payload_sale_allowed = StockMovementPayload {
            merchant_id: merchant_id.clone(),
            outlet_id: outlet_id.clone(),
            product_id: prod_id.clone(),
            movement_type: "SALE".to_string(),
            qty_delta: -5.0,
            reason: Some("Deduct overflow allowed".to_string()),
            reason_code: None,
            reference_type: Some("order".to_string()),
            reference_id: None,
            idempotency_key: Some("neg_test_2".to_string()),
            created_by: user_id.clone(),
        };

        let (sb, sa) = process_stock_movement_ledger(&mut tx, payload_sale_allowed).await.unwrap();
        assert_eq!(sb, 0.0);
        assert_eq!(sa, -5.0);

        tx.commit().await.unwrap();
    }

    #[tokio::test]
    async fn test_omnichannel_sku_per_location_qty_breakdown() {
        let pool = setup_test_db().await;
        let mut tx = pool.begin().await.unwrap();

        let (user_id, merchant_id, outlet_id): (String, String, String) = sqlx::query_as(
            "SELECT id, merchant_id, outlet_id FROM users LIMIT 1"
        )
        .fetch_one(&mut *tx)
        .await
        .unwrap();

        let prod_id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO products (id, merchant_id, sku, name, price, track_stock, active) VALUES (?, ?, 'SKU-OMNI-1', 'Omni Shirt', 150000, 1, 1)"
        )
        .bind(&prod_id)
        .bind(&merchant_id)
        .execute(&mut *tx)
        .await
        .unwrap();

        // 1. Initial stock 20
        process_stock_movement_ledger(&mut tx, StockMovementPayload {
            merchant_id: merchant_id.clone(),
            outlet_id: outlet_id.clone(),
            product_id: prod_id.clone(),
            movement_type: "INITIAL_STOCK".to_string(),
            qty_delta: 20.0,
            reason: Some("Init stock".to_string()),
            reason_code: None,
            reference_type: None,
            reference_id: None,
            idempotency_key: Some("omni_init_1".to_string()),
            created_by: user_id.clone(),
        }).await.unwrap();

        // 2. Reserve 5 items for web order
        reserve_omnichannel_stock(&mut tx, &merchant_id, &outlet_id, &prod_id, 5.0, "web_ord_1", &user_id).await.unwrap();

        tx.commit().await.unwrap();

        // Check breakdown
        let breakdown = get_product_stock_breakdown(&pool, &outlet_id, &prod_id).await.unwrap();
        assert_eq!(breakdown.qty_on_hand, 20.0);
        assert_eq!(breakdown.qty_reserved, 5.0);
        assert_eq!(breakdown.qty_available, 15.0);
    }

    #[tokio::test]
    async fn test_web_order_reservation_and_release() {
        let pool = setup_test_db().await;
        let mut tx = pool.begin().await.unwrap();

        let (user_id, merchant_id, outlet_id): (String, String, String) = sqlx::query_as(
            "SELECT id, merchant_id, outlet_id FROM users LIMIT 1"
        )
        .fetch_one(&mut *tx)
        .await
        .unwrap();

        let prod_id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO products (id, merchant_id, sku, name, price, track_stock, active) VALUES (?, ?, 'SKU-OMNI-2', 'Omni Shoes', 500000, 1, 1)"
        )
        .bind(&prod_id)
        .bind(&merchant_id)
        .execute(&mut *tx)
        .await
        .unwrap();

        // 1. Initial stock 10
        process_stock_movement_ledger(&mut tx, StockMovementPayload {
            merchant_id: merchant_id.clone(),
            outlet_id: outlet_id.clone(),
            product_id: prod_id.clone(),
            movement_type: "INITIAL_STOCK".to_string(),
            qty_delta: 10.0,
            reason: Some("Init stock".to_string()),
            reason_code: None,
            reference_type: None,
            reference_id: None,
            idempotency_key: Some("omni_init_2".to_string()),
            created_by: user_id.clone(),
        }).await.unwrap();

        // 2. Reserve 3 items
        reserve_omnichannel_stock(&mut tx, &merchant_id, &outlet_id, &prod_id, 3.0, "web_ord_2", &user_id).await.unwrap();

        // 3. Release reservation (e.g. order cancelled)
        release_omnichannel_reservation(&mut tx, &merchant_id, &outlet_id, &prod_id, 3.0, "web_ord_2", &user_id).await.unwrap();

        tx.commit().await.unwrap();

        let breakdown = get_product_stock_breakdown(&pool, &outlet_id, &prod_id).await.unwrap();
        assert_eq!(breakdown.qty_on_hand, 10.0);
        assert_eq!(breakdown.qty_reserved, 0.0);
        assert_eq!(breakdown.qty_available, 10.0);
    }
}


