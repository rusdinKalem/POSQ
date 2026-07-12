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
mod server;
mod settings;
mod fb_table;
mod proxy;
mod sync_worker;
mod ecr;
mod security_policy;
mod cash_ledger;
mod fraud_detection;
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
        .bind(merchant_id.to_string())
        .execute(pool.inner()).await.map_err(|e| e.to_string())?;

    sqlx::query("INSERT INTO outlets (id, merchant_id, name, code) VALUES (?, ?, 'Dummy Outlet', 'DUMMY1') ON CONFLICT DO NOTHING")
        .bind(outlet_id.to_string()).bind(merchant_id.to_string())
        .execute(pool.inner()).await.map_err(|e| e.to_string())?;

    sqlx::query("INSERT INTO orders (id, merchant_id, outlet_id, order_number, status, grand_total, created_at) VALUES (?, ?, ?, ?, 'paid', ?, CURRENT_TIMESTAMP)")
        .bind(order_id.to_string()).bind(merchant_id.to_string()).bind(outlet_id.to_string())
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
                
                let pool2 = pool.clone();
                app.manage(pool.clone());
                
                // Read network mode to decide if we should spawn the server
                let mut is_master = false;
                use sqlx::Row;
                if let Ok(records) = sqlx::query("SELECT value FROM system_settings WHERE key = 'network_mode'").fetch_all(&pool).await {
                    if let Some(r) = records.first() {
                        let mode: String = r.try_get("value").unwrap_or_default();
                        if mode == "MASTER" {
                            is_master = true;
                        }
                    }
                }

                if is_master {
                    // Spawn Master Server on a background thread
                    tauri::async_runtime::spawn(async move {
                        crate::server::start_server(pool.clone(), 3030).await;
                    });
                }
                
                // Spawn Cloud Sync Worker on a background thread (all modes)
                tauri::async_runtime::spawn(async move {
                    crate::sync_worker::start_sync_worker(pool2).await;
                });
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
            ecr::start_ecr_transaction,
            seed::seed_database,
            pos::get_products,
            pos::checkout,
            pos::get_receipt,
            pos::save_cart_draft,
            pos::get_cart_draft,
            pos::clear_cart_draft,
            pos::save_hold_draft,
            pos::list_hold_drafts,
            pos::delete_hold_draft,
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
            inventory::list_categories,
            inventory::create_category,
            inventory::update_category,
            inventory::delete_category,
            inventory::get_recipe_ingredients,
            inventory::save_recipe,
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
            hardware::save_hardware_settings,
            fb::print_kitchen_ticket,
            fb::get_tables,
            fb::get_active_kds_tickets,
            fb::get_completed_kds_tickets,
            fb::update_kds_ticket_status,
            fb::save_kds_ticket,
            fb::link_kds_draft_to_order,
            retail::process_return,
            settings::get_network_settings,
            settings::save_network_settings,
            fb_table::get_all_tables_status,
            fb_table::get_bills_by_session,
            fb_table::preview_split_bill,
            fb_table::commit_split_bill,
            fb_table::preview_join_bills,
            fb_table::commit_join_bills,
            fb_table::commit_move_table,
            fb_table::commit_swap_tables,
            fb_table::commit_join_tables,
            fb_table::add_new_table,
            fb_table::release_table_session,
            security_policy::evaluate_action_policy,
            security_policy::verify_supervisor_pin,
            cash_ledger::post_cash_in,
            cash_ledger::post_cash_out,
            cash_ledger::post_safe_drop,
            cash_ledger::get_cash_movements,
            cash_ledger::reverse_cash_movement,
            shift::submit_blind_cash_count,
            shift::approve_shift_variance,
            fraud_detection::run_fraud_checks,
            fraud_detection::get_fraud_alerts,
            fraud_detection::resolve_fraud_alert,
            auth::login_user,
            auth::logout_user,
            auth::get_active_session
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dummy_order_flow() {
        std::env::set_var("DATABASE_URL", "sqlite::memory:");
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

    #[tokio::test]
    async fn test_security_audit_sessions_flow() {
        std::env::set_var("DATABASE_URL", "sqlite::memory:");
        let pool = db::establish_connection().await.unwrap();
        migration::run_migrations(&pool).await.unwrap();

        let app = tauri::test::mock_app();
        app.manage(pool.clone());

        // Seed using State from mock app
        seed::seed_database(app.state::<SqlitePool>()).await.unwrap();

        // 1. Test Argon2id PIN verification with seeded owner PIN '123456'
        let login_res = auth::login_user(
            "123456".to_string(),
            "device_1".to_string(),
            "register_1".to_string(),
            app.state::<SqlitePool>()
        ).await;
        assert!(login_res.is_ok(), "Login failed for owner with PIN '123456'");
        let session = login_res.unwrap();
        assert!(!session.session_token.is_empty(), "Session token should not be empty");

        // Verify login fails with incorrect PIN
        let login_fail = auth::login_user(
            "wrong_pin".to_string(),
            "device_1".to_string(),
            "register_1".to_string(),
            app.state::<SqlitePool>()
        ).await;
        assert!(login_fail.is_err(), "Login should have failed with incorrect PIN");

        // Verify that failed_login_attempts is NOT incremented on any user since PIN was wrong and not associated with a specific user
        let attempts: i32 = sqlx::query_scalar("SELECT failed_login_attempts FROM users WHERE name = 'Owner POS'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(attempts, 0, "Failed login attempts must be 0 for Owner POS after a wrong PIN");

        let attempts_cashier: i32 = sqlx::query_scalar("SELECT failed_login_attempts FROM users WHERE name = 'Kasir POS'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(attempts_cashier, 0, "Failed login attempts must be 0 for Kasir POS after a wrong PIN");

        // Test supervisor auth failure with wrong PIN
        let supervisor_fail = security_policy::verify_supervisor_pin(
            security_policy::SupervisorAuthRequest {
                pin: "wrong_pin".to_string(),
                action_type: "refund.approve".to_string(),
                amount: 0,
                reason_code: "MISTAKE".to_string(),
                resource_id: None,
            },
            app.state::<SqlitePool>()
        ).await;
        assert!(supervisor_fail.is_err(), "Supervisor auth should have failed with incorrect PIN");

        // Verify supervisor failed attempts not incremented
        let attempts_sup: i32 = sqlx::query_scalar("SELECT failed_login_attempts FROM users WHERE name = 'Supervisor POS'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(attempts_sup, 0, "Supervisor failed attempts must be 0");

        // Test supervisor auth success with correct supervisor PIN '1234'
        let supervisor_success = security_policy::verify_supervisor_pin(
            security_policy::SupervisorAuthRequest {
                pin: "1234".to_string(),
                action_type: "refund.approve".to_string(),
                amount: 0,
                reason_code: "MISTAKE".to_string(),
                resource_id: None,
            },
            app.state::<SqlitePool>()
        ).await;
        assert!(supervisor_success.is_ok(), "Supervisor auth should succeed with PIN '1234', but failed with error: {:?}", supervisor_success.err());


        // 2. Verify cryptographic hash chain on audit logs
        let mut conn = pool.acquire().await.unwrap();
        
        let merchant_row = sqlx::query("SELECT id FROM merchants LIMIT 1")
            .fetch_one(&pool)
            .await
            .unwrap();
        let seeded_merchant_id: String = merchant_row.get(0);

        let user_row = sqlx::query("SELECT id FROM users LIMIT 1")
            .fetch_one(&pool)
            .await
            .unwrap();
        let seeded_user_id: String = user_row.get(0);

        // Log action 1
        audit::log_action(
            &mut *conn,
            seeded_merchant_id.clone(),
            None,
            seeded_user_id.clone(),
            "test_action_1",
            "test_target",
            None,
            Some("initial hash test")
        ).await.unwrap();

        // Log action 2
        audit::log_action(
            &mut *conn,
            seeded_merchant_id,
            None,
            seeded_user_id,
            "test_action_2",
            "test_target",
            None,
            Some("second hash test")
        ).await.unwrap();


        // Query the logs back to verify the chain
        let logs = sqlx::query("SELECT id, action, previous_hash, entry_hash FROM audit_logs ORDER BY created_at ASC, rowid ASC")
            .fetch_all(&pool)
            .await
            .unwrap();

        for (i, log) in logs.iter().enumerate() {
            println!("Log {}: action='{}', prev='{}', entry='{}'", i, log.get::<String, _>("action"), log.get::<String, _>("previous_hash"), log.get::<String, _>("entry_hash"));
        }

        assert_eq!(logs.len(), 3, "Expected exactly 3 audit log entries");
        
        let row1 = &logs[0];
        let row2 = &logs[1];
        let row3 = &logs[2];

        let prev_hash_1: String = row1.get("previous_hash");
        let entry_hash_1: String = row1.get("entry_hash");
        let prev_hash_2: String = row2.get("previous_hash");
        let entry_hash_2: String = row2.get("entry_hash");
        let prev_hash_3: String = row3.get("previous_hash");
        let entry_hash_3: String = row3.get("entry_hash");

        // Genesis hash should be 64 zeros
        assert_eq!(prev_hash_1, "0000000000000000000000000000000000000000000000000000000000000000");
        // Hash chain integrity: the second entry's previous_hash must match the first entry's entry_hash
        assert_eq!(prev_hash_2, entry_hash_1);
        // Hash chain integrity: the third entry's previous_hash must match the second entry's entry_hash
        assert_eq!(prev_hash_3, entry_hash_2);
        assert!(!entry_hash_3.is_empty());
    }
}

