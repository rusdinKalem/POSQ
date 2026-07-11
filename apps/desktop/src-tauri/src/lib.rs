use serde_json::{json, Value};
use sqlx::{SqlitePool, Row};
use tauri::{State, Manager};
use uuid::Uuid;

mod db;
mod migration;
mod backup;
mod seed;
mod pos;
mod auth;
mod audit;
mod shift;
mod refund;
mod inventory;
mod report;
mod license;
mod update;
mod hardware;
mod fb;
mod retail;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn health_check() -> Result<Value, String> {
    Ok(json!({
        "status": "ok",
        "app_version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

#[tauri::command]
async fn db_health_check(pool: State<'_, SqlitePool>) -> Result<Value, String> {
    match sqlx::query("SELECT 1").execute(pool.inner()).await {
        Ok(_) => Ok(json!({"db": "connected"})),
        Err(e) => Ok(json!({"db": "unavailable", "error": e.to_string()})),
    }
}

#[tauri::command]
async fn create_dummy_order(pool: State<'_, SqlitePool>) -> Result<Value, String> {
    let merchant_id = Uuid::new_v4();
    let outlet_id = Uuid::new_v4();
    let order_id = Uuid::new_v4();

    // In a real app these would be existing merchant/outlets, but for PoC we insert dummies first to satisfy FKs
    sqlx::query("INSERT INTO merchants (id, name) VALUES (?, 'Dummy Merchant') ON CONFLICT DO NOTHING")
        .bind(merchant_id)
        .execute(pool.inner()).await.map_err(|e| e.to_string())?;

    sqlx::query("INSERT INTO outlets (id, merchant_id, name, code) VALUES (?, ?, 'Dummy Outlet', 'DUMMY1') ON CONFLICT DO NOTHING")
        .bind(outlet_id).bind(merchant_id)
        .execute(pool.inner()).await.map_err(|e| e.to_string())?;

    sqlx::query("INSERT INTO orders (id, merchant_id, outlet_id, order_number, status, grand_total, created_at) VALUES (?, ?, ?, ?, 'paid', ?, CURRENT_TIMESTAMP)")
        .bind(order_id).bind(merchant_id).bind(outlet_id)
        .bind(format!("ORD-{}", &order_id.to_string()[..8].to_uppercase()))
        .bind(50000_i32)
        .execute(pool.inner()).await
        .map_err(|e| e.to_string())?;

    Ok(json!({"order_id": order_id}))
}

#[tauri::command]
async fn list_orders(pool: State<'_, SqlitePool>) -> Result<Value, String> {
    let records = sqlx::query("SELECT id, order_number, grand_total FROM orders ORDER BY created_at DESC LIMIT 5")
        .fetch_all(pool.inner())
        .await
        .map_err(|e| e.to_string())?;

    let orders: Vec<Value> = records.into_iter().map(|rec| {
        let id: String = rec.get("id");
        let order_number: String = rec.get("order_number");
        let grand_total: i32 = rec.get("grand_total");
        json!({
            "id": id,
            "order_number": order_number,
            "grand_total": grand_total
        })
    }).collect();

    Ok(json!({"orders": orders}))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            tauri::async_runtime::block_on(async {
                let pool = db::establish_connection().await.expect("Failed to connect to DB on startup");
                // Run migrations on startup to ensure SQLite schema is up-to-date
                let _ = migration::run_migrations(&pool).await;
                let _ = seed::run_seed(&pool).await;
                app.manage(pool);
            });
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            health_check,
            db_health_check,
            create_dummy_order,
            list_orders,
            seed::seed_database,
            pos::get_products,
            pos::checkout,
            pos::get_receipt,
            shift::check_active_shift,
            shift::open_shift,
            shift::close_shift,
            audit::get_audit_logs,
            refund::process_refund,
            inventory::get_low_stock,
            inventory::stock_in,
            inventory::adjust_stock,
            inventory::stock_opname,
            inventory::transfer_stock,
            inventory::create_product,
            inventory::import_products_csv,
            inventory::get_inventory_products,
            inventory::update_product,
            inventory::get_product_movements,
            report::get_sales_summary,
            report::get_payment_breakdown,
            report::get_product_ranking,
            report::export_sales_csv,
            backup::generate_recovery_key,
            backup::create_local_backup,
            backup::restore_local_backup,
            license::activate_device,
            license::verify_license,
            license::refresh_license,
            update::check_update,
            update::validate_update,
            update::run_safe_migration,
            hardware::print_receipt,
            hardware::get_hardware_status,
            fb::print_kitchen_ticket,
            fb::get_tables,
            retail::process_return
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dummy_order_flow() {
        let pool = db::establish_connection().await.unwrap();
        migration::run_migrations(&pool).await.unwrap();

        let app = tauri::test::mock_app();
        app.manage(pool);
        
        let order_res = create_dummy_order(app.state::<SqlitePool>()).await;
        assert!(order_res.is_ok(), "Failed to create dummy order");

        let list_res = list_orders(app.state::<SqlitePool>()).await;
        assert!(list_res.is_ok(), "Failed to list orders");
        
        let val = list_res.unwrap();
        assert!(val.get("orders").unwrap().as_array().unwrap().len() > 0, "No orders found");
    }
}
