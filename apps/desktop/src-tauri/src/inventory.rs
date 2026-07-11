use sqlx::{SqlitePool, Row};
use tauri::State;
use uuid::Uuid;
use serde::Serialize;

#[derive(Serialize)]
pub struct LowStockItem {
    pub product_id: Uuid,
    pub name: String,
    pub sku: String,
    pub qty_on_hand: f64,
    pub min_qty: f64,
}

#[tauri::command]
pub async fn get_low_stock(pool: State<'_, SqlitePool>) -> Result<Vec<LowStockItem>, String> {
    let user_id = crate::auth::get_current_user(pool.inner()).await?;
    let has_perm = crate::auth::has_permission(pool.inner(), user_id, "inventory.manage").await?;
    if !has_perm {
        return Err("Akses ditolak: Anda tidak memiliki izin untuk melihat laporan inventaris".to_string());
    }

    let records = sqlx::query(
        r#"
        SELECT p.id as product_id, p.name, p.sku, i.qty_on_hand, i.min_qty
        FROM products p
        JOIN inventory_items i ON p.id = i.product_id
        WHERE i.qty_on_hand <= i.min_qty AND p.track_stock = 1
        ORDER BY i.qty_on_hand ASC
        "#
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let items = records.into_iter().map(|r| {
        let qty_on_hand: f64 = crate::db::get_numeric_as_f64(&r, "qty_on_hand");
        let min_qty: f64 = crate::db::get_numeric_as_f64(&r, "min_qty");
        LowStockItem {
            product_id: Uuid::parse_str(&r.get::<String, _>("product_id")).unwrap_or_default(),
            name: r.get("name"),
            sku: r.get("sku"),
            qty_on_hand,
            min_qty,
        }
    }).collect();

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

#[tauri::command]
pub async fn create_product(
    name: String,
    sku: String,
    price: i32,
    cost: Option<i32>,
    category_name: Option<String>,
    track_stock: bool,
    initial_qty: f64,
    image_url: Option<String>,
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

    // Resolve or create category
    let mut category_id: Option<String> = None;
    if let Some(cat_name) = category_name {
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
    .bind(price)
    .bind(cost)
    .bind(track_stock)
    .bind(image_url)
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
    pub track_stock: bool,
    pub image_url: Option<String>,
    pub qty_on_hand: f64,
    pub min_qty: f64,
}

#[tauri::command]
pub async fn get_inventory_products(pool: State<'_, SqlitePool>) -> Result<Vec<DetailedProductItem>, String> {
    let records = sqlx::query(
        r#"
        SELECT p.id, p.name, p.sku, p.price, p.cost, c.name as category_name, p.track_stock, p.image_url, i.qty_on_hand, i.min_qty
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

    let products = records.into_iter().map(|r| {
        let qty_on_hand = crate::db::get_numeric_as_f64(&r, "qty_on_hand");
        let min_qty = crate::db::get_numeric_as_f64(&r, "min_qty");
        DetailedProductItem {
            id: r.get("id"),
            name: r.get("name"),
            sku: r.get("sku"),
            price: r.get("price"),
            cost: r.get("cost"),
            category_name: r.get("category_name"),
            track_stock: r.get::<i64, _>("track_stock") != 0,
            image_url: r.get("image_url"),
            qty_on_hand,
            min_qty,
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
    category_name: Option<String>,
    track_stock: bool,
    image_url: Option<String>,
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

    // Resolve or create category
    let mut category_id: Option<String> = None;
    if let Some(cat_name) = category_name {
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

    // Update Product
    sqlx::query(
        r#"
        UPDATE products 
        SET category_id = ?, sku = ?, name = ?, price = ?, cost = ?, track_stock = ?, image_url = ?, updated_at = CURRENT_TIMESTAMP
        WHERE id = ? AND merchant_id = ?
        "#
    )
    .bind(category_id)
    .bind(sku)
    .bind(name)
    .bind(price)
    .bind(cost)
    .bind(track_stock)
    .bind(image_url)
    .bind(&id)
    .bind(&merchant_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

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

