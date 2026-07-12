use sqlx::{SqlitePool, Row};
use tauri::State;
use uuid::Uuid;
use serde::Serialize;
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

#[tauri::command]
pub async fn stock_in(product_id: Uuid, qty: f64, reason: Option<String>, pool: State<'_, SqlitePool>) -> Result<(), String> {
    if qty <= 0.0 {
        return Err("Qty harus lebih dari 0".to_string());
    }
    process_stock_movement(product_id, qty, "stock_in", reason, pool.inner()).await
}

#[tauri::command]
pub async fn adjust_stock(product_id: Uuid, qty_delta: f64, reason: String, pool: State<'_, SqlitePool>) -> Result<(), String> {
    if reason.trim().is_empty() {
        return Err("Alasan (reason) wajib diisi untuk penyesuaian stok".to_string());
    }
    process_stock_movement(product_id, qty_delta, "adjustment", Some(reason), pool.inner()).await
}

#[tauri::command]
pub async fn stock_opname(product_id: Uuid, actual_qty: f64, reason: String, pool: State<'_, SqlitePool>) -> Result<(), String> {
    // SEC-001: Enforce license status to prevent API bypass
    crate::license::enforce_active_license().await?;

    let user_id = crate::auth::get_current_user(pool.inner()).await?;
    let has_perm = crate::auth::has_permission(pool.inner(), user_id, "inventory.manage").await?;
    if !has_perm {
        return Err("Akses ditolak: Anda tidak memiliki izin untuk mengelola inventaris".to_string());
    }

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let current = sqlx::query(
        "SELECT merchant_id, outlet_id, qty_on_hand FROM inventory_items WHERE product_id = ?"
    )
    .bind(product_id.to_string())
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    let current = current.ok_or("Produk tidak ditemukan di inventaris")?;
    let merchant_id: String = current.get("merchant_id");
    let outlet_id: String = current.get("outlet_id");
    
    let current_qty: f64 = crate::db::get_numeric_as_f64(&current, "qty_on_hand");
    let qty_delta = actual_qty - current_qty;

    sqlx::query(
        "UPDATE inventory_items SET qty_on_hand = ?, updated_at = CURRENT_TIMESTAMP WHERE product_id = ?"
    )
    .bind(actual_qty)
    .bind(product_id.to_string())
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    let movement_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO stock_movements (id, merchant_id, outlet_id, product_id, movement_type, qty_delta, reason, reference_type, created_by, created_at)
        VALUES (?, ?, ?, ?, 'opname', ?, ?, 'manual', ?, CURRENT_TIMESTAMP)
        "#
    )
    .bind(movement_id.to_string())
    .bind(&merchant_id)
    .bind(&outlet_id)
    .bind(product_id.to_string())
    .bind(qty_delta)
    .bind(&reason)
    .bind(user_id.to_string())
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    crate::audit::log_action(
        &mut *tx, 
        merchant_id, 
        Some(outlet_id), 
        user_id.to_string(), 
        "stock_opname", 
        "inventory", 
        Some(product_id.to_string()), 
        Some(&reason)
    ).await?;

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn transfer_stock(product_id: Uuid, qty: f64, _destination_outlet: String, reason: String, pool: State<'_, SqlitePool>) -> Result<(), String> {
    if qty <= 0.0 {
        return Err("Qty harus lebih dari 0".to_string());
    }
    // Untuk MVP M5, transfer disimulasikan sebagai transfer_out dari toko ini ke gudang.
    process_stock_movement(product_id, -qty, "transfer_out", Some(reason), pool.inner()).await
}

async fn process_stock_movement(product_id: Uuid, qty_delta: f64, movement_type: &str, reason: Option<String>, pool: &SqlitePool) -> Result<(), String> {
    // SEC-001: Enforce license status to prevent API bypass
    crate::license::enforce_active_license().await?;

    let user_id = crate::auth::get_current_user(pool).await?;
    let has_perm = crate::auth::has_permission(pool, user_id, "inventory.manage").await?;
    if !has_perm {
        return Err("Akses ditolak: Anda tidak memiliki izin untuk mengelola inventaris".to_string());
    }

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let current = sqlx::query(
        "SELECT merchant_id, outlet_id, qty_on_hand FROM inventory_items WHERE product_id = ?"
    )
    .bind(product_id.to_string())
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    let current = current.ok_or("Produk tidak ditemukan di inventaris")?;
    let merchant_id: String = current.get("merchant_id");
    let outlet_id: String = current.get("outlet_id");
    let current_qty: f64 = crate::db::get_numeric_as_f64(&current, "qty_on_hand");
    
    if current_qty + qty_delta < 0.0 {
        return Err("Stok tidak boleh negatif".to_string());
    }

    sqlx::query(
        "UPDATE inventory_items SET qty_on_hand = qty_on_hand + ?, updated_at = CURRENT_TIMESTAMP WHERE product_id = ?"
    )
    .bind(qty_delta)
    .bind(product_id.to_string())
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    let movement_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO stock_movements (id, merchant_id, outlet_id, product_id, movement_type, qty_delta, reason, reference_type, created_by, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, 'manual', ?, CURRENT_TIMESTAMP)
        "#,
    )
    .bind(movement_id.to_string())
    .bind(&merchant_id)
    .bind(&outlet_id)
    .bind(product_id.to_string())
    .bind(movement_type)
    .bind(qty_delta)
    .bind(&reason)
    .bind(user_id.to_string())
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    crate::audit::log_action(
        &mut *tx, 
        merchant_id, 
        Some(outlet_id), 
        user_id.to_string(), 
        movement_type, 
        "inventory", 
        Some(product_id.to_string()), 
        reason.as_deref()
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


