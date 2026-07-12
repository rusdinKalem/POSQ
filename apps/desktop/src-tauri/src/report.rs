use sqlx::{SqlitePool, Row};
use tauri::State;
use serde::Serialize;

#[derive(Serialize)]
pub struct SalesSummary {
    pub total_orders: i64,
    pub subtotal: i64,
    pub discount_total: i64,
    pub tax_total: i64,
    pub grand_total: i64,
}

#[derive(Serialize)]
pub struct PaymentBreakdown {
    pub payment_method: String,
    pub amount: i64,
}

#[derive(Serialize)]
pub struct ProductRanking {
    pub product_id: String,
    pub name: String,
    pub total_qty: f64,
    pub total_revenue: i64,
}

async fn check_report_permission(pool: &SqlitePool) -> Result<uuid::Uuid, String> {
    let user_id = crate::auth::get_current_user(pool).await?;
    let has_perm = crate::auth::has_permission(pool, user_id, "report.view").await?;
    if !has_perm {
        return Err("Akses ditolak: Anda tidak memiliki izin untuk melihat laporan".to_string());
    }
    Ok(user_id)
}

#[tauri::command]
pub async fn get_sales_summary(start_date: String, end_date: String, pool: State<'_, SqlitePool>) -> Result<SalesSummary, String> {
    check_report_permission(&pool).await?;

    let start_dt = chrono::DateTime::parse_from_rfc3339(&start_date)
        .map_err(|e| e.to_string())?
        .with_timezone(&chrono::Utc)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
    let end_dt = chrono::DateTime::parse_from_rfc3339(&end_date)
        .map_err(|e| e.to_string())?
        .with_timezone(&chrono::Utc)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    let record = sqlx::query(
        r#"
        SELECT 
            COUNT(id) as total_orders,
            COALESCE(SUM(subtotal), 0) as total_subtotal,
            COALESCE(SUM(discount_total), 0) as total_discount,
            COALESCE(SUM(tax_total), 0) as total_tax,
            COALESCE(SUM(grand_total), 0) as total_grand
        FROM orders
        WHERE status IN ('paid', 'completed')
          AND created_at >= ?
          AND created_at <= ?
        "#,
    )
    .bind(start_dt)
    .bind(end_dt)
    .fetch_one(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    Ok(SalesSummary {
        total_orders: record.get("total_orders"),
        subtotal: record.get("total_subtotal"),
        discount_total: record.get("total_discount"),
        tax_total: record.get("total_tax"),
        grand_total: record.get("total_grand"),
    })
}

#[tauri::command]
pub async fn get_payment_breakdown(start_date: String, end_date: String, pool: State<'_, SqlitePool>) -> Result<Vec<PaymentBreakdown>, String> {
    check_report_permission(&pool).await?;

    let start_dt = chrono::DateTime::parse_from_rfc3339(&start_date)
        .map_err(|e| e.to_string())?
        .with_timezone(&chrono::Utc)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
    let end_dt = chrono::DateTime::parse_from_rfc3339(&end_date)
        .map_err(|e| e.to_string())?
        .with_timezone(&chrono::Utc)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    let records = sqlx::query(
        r#"
        SELECT p.method as payment_method, COALESCE(SUM(p.amount), 0) as total_amount
        FROM payments p
        JOIN orders o ON p.order_id = o.id
        WHERE p.status = 'paid'
          AND o.created_at >= ?
          AND o.created_at <= ?
        GROUP BY p.method
        "#,
    )
    .bind(start_dt)
    .bind(end_dt)
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let breakdown = records.into_iter().map(|r| PaymentBreakdown {
        payment_method: r.get("payment_method"),
        amount: r.get("total_amount"),
    }).collect();

    Ok(breakdown)
}

#[tauri::command]
pub async fn get_product_ranking(start_date: String, end_date: String, pool: State<'_, SqlitePool>) -> Result<Vec<ProductRanking>, String> {
    check_report_permission(&pool).await?;

    let start_dt = chrono::DateTime::parse_from_rfc3339(&start_date)
        .map_err(|e| e.to_string())?
        .with_timezone(&chrono::Utc)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
    let end_dt = chrono::DateTime::parse_from_rfc3339(&end_date)
        .map_err(|e| e.to_string())?
        .with_timezone(&chrono::Utc)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    let records = sqlx::query(
        r#"
        SELECT 
            i.product_id, 
            i.name,
            COALESCE(SUM(i.qty), 0) as total_qty,
            COALESCE(SUM(i.line_total), 0) as total_revenue
        FROM order_items i
        JOIN orders o ON i.order_id = o.id
        WHERE o.status IN ('paid', 'completed')
          AND o.created_at >= ?
          AND o.created_at <= ?
        GROUP BY i.product_id, i.name
        ORDER BY total_qty DESC
        LIMIT 50
        "#,
    )
    .bind(start_dt)
    .bind(end_dt)
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let ranking = records.into_iter().map(|r| {
        let total_qty: f64 = r.try_get::<f64, _>("total_qty")
            .or_else(|_| r.try_get::<i64, _>("total_qty").map(|i| i as f64))
            .unwrap_or(0.0);
        ProductRanking {
            product_id: r.get::<String, _>("product_id"),
            name: r.get("name"),
            total_qty,
            total_revenue: r.get("total_revenue"),
        }
    }).collect();

    Ok(ranking)
}

#[tauri::command]
pub async fn export_sales_csv(start_date: String, end_date: String, pool: State<'_, SqlitePool>) -> Result<String, String> {
    check_report_permission(&pool).await?;
    
    let ranking = get_product_ranking(start_date, end_date, pool).await?;
    
    // Using standard standard OS downloads directory for simplicity in MVP.
    let dirs = directories::UserDirs::new().ok_or("Cannot find user directories")?;
    let download_dir = dirs.download_dir().ok_or("Cannot find downloads directory")?;
    
    let file_name = format!("sales_report_{}.csv", uuid::Uuid::new_v4().to_string().chars().take(8).collect::<String>());
    let file_path = download_dir.join(file_name);
    
    let mut wtr = csv::Writer::from_path(&file_path).map_err(|e| e.to_string())?;
    
    wtr.write_record(&["Product ID", "Name", "Total Qty", "Total Revenue"]).map_err(|e| e.to_string())?;
    
    for item in ranking {
        wtr.write_record(&[
            item.product_id,
            item.name,
            item.total_qty.to_string(),
            item.total_revenue.to_string()
        ]).map_err(|e| e.to_string())?;
    }
    
    wtr.flush().map_err(|e| e.to_string())?;
    
    Ok(file_path.to_string_lossy().to_string())
}
