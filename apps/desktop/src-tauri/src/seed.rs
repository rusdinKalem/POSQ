use sqlx::SqlitePool;
use uuid::Uuid;

pub async fn run_seed(pool: &SqlitePool) -> Result<(), String> {
    println!("Running tenant seed...");
    
    // Check if user table exists (run migrations if database is not migrated yet)
    let _count: (i64,) = match sqlx::query_as("SELECT count(*) FROM users")
        .fetch_one(pool)
        .await
    {
        Ok(c) => c,
        Err(e) => {
            let err_msg = e.to_string();
            if err_msg.contains("no such table") {
                println!("Database not migrated. Running migrations now...");
                crate::migration::run_migrations(pool).await?;
                // Try counting again after migration
                sqlx::query_as("SELECT count(*) FROM users")
                    .fetch_one(pool)
                    .await
                    .map_err(|err| format!("Failed to count users after migration: {}", err))?
            } else {
                return Err(format!("Failed to count users: {}", e));
            }
        }
    };

    println!("Seeding default tenant...");

    // 1. Merchant
    let merchant_row: Option<(String,)> = sqlx::query_as("SELECT id FROM merchants LIMIT 1")
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to check merchant: {}", e))?;

    let merchant_id = match merchant_row {
        Some((id,)) => id,
        None => {
            let mid = Uuid::new_v4().to_string();
            sqlx::query("INSERT INTO merchants (id, name) VALUES (?, 'Toko Demo POSQ')")
                .bind(&mid)
                .execute(pool)
                .await
                .map_err(|e| format!("Failed to seed merchant: {}", e))?;
            mid
        }
    };

    // 2. Outlet
    let outlet_row: Option<(String,)> = sqlx::query_as("SELECT id FROM outlets WHERE merchant_id = ? LIMIT 1")
        .bind(&merchant_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to check outlet: {}", e))?;

    let outlet_id = match outlet_row {
        Some((id,)) => id,
        None => {
            let oid = Uuid::new_v4().to_string();
            sqlx::query("INSERT INTO outlets (id, merchant_id, name, code) VALUES (?, ?, 'Cabang Utama', 'HO01')")
                .bind(&oid)
                .bind(&merchant_id)
                .execute(pool)
                .await
                .map_err(|e| format!("Failed to seed outlet: {}", e))?;
            oid
        }
    };

    // 3. Roles
    let roles = vec![
        "owner",
        "manager",
        "supervisor",
        "cashier",
        "inventory",
        "finance"
    ];

    for role_name in roles {
        let role_exists: Option<(String,)> = sqlx::query_as("SELECT id FROM roles WHERE merchant_id = ? AND name = ?")
            .bind(&merchant_id)
            .bind(role_name)
            .fetch_optional(pool)
            .await
            .map_err(|e| format!("Failed to check role: {}", e))?;

        if role_exists.is_none() {
            sqlx::query("INSERT INTO roles (id, merchant_id, name, system_role) VALUES (?, ?, ?, true)")
                .bind(Uuid::new_v4().to_string())
                .bind(&merchant_id)
                .bind(role_name)
                .execute(pool)
                .await
                .map_err(|e| format!("Failed to seed role {}: {}", role_name, e))?;
        }
    }

    // 4. Permissions
    let permissions = vec![
        "pos.sell",
        "shift.manage",
        "refund.approve",
        "audit.view",
        "inventory.manage",
        "report.view",
    ];

    for perm in &permissions {
        sqlx::query("INSERT INTO permissions (id, key, description) VALUES (?, ?, ?) ON CONFLICT DO NOTHING")
            .bind(Uuid::new_v4().to_string())
            .bind(perm)
            .bind(format!("Permission for {}", perm))
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to seed permission {}: {}", perm, e))?;
    }

    // 5. Default Users & Roles
    use sqlx::Row;
    let owner_pin_hash = crate::auth::hash_pin_argon2("123456").map_err(|e| e.to_string())?;
    let supervisor_pin_hash = crate::auth::hash_pin_argon2("1234").map_err(|e| e.to_string())?;
    let cashier_pin_hash = crate::auth::hash_pin_argon2("1111").map_err(|e| e.to_string())?;

    // Owner POS
    let owner_row: Option<sqlx::sqlite::SqliteRow> = sqlx::query("SELECT id, pin_hash_v2 FROM users WHERE name = 'Owner POS'")
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to query Owner POS: {}", e))?;
    let admin_user_id = match owner_row {
        Some(row) => {
            let id: String = row.get("id");
            let pin_hash_v2: Option<String> = row.get("pin_hash_v2");
            if pin_hash_v2.is_none() {
                sqlx::query("UPDATE users SET pin_hash_v2 = ? WHERE id = ?")
                    .bind(&owner_pin_hash)
                    .bind(&id)
                    .execute(pool)
                    .await
                    .map_err(|e| format!("Failed to update owner pin hash: {}", e))?;
            }
            id
        }
        None => {
            let id = Uuid::new_v4().to_string();
            sqlx::query("INSERT INTO users (id, merchant_id, outlet_id, name, pin_hash_v2, status) VALUES (?, ?, ?, 'Owner POS', ?, 'active')")
                .bind(&id)
                .bind(&merchant_id)
                .bind(&outlet_id)
                .bind(&owner_pin_hash)
                .execute(pool)
                .await
                .map_err(|e| format!("Failed to seed owner user: {}", e))?;
            id
        }
    };

    // Supervisor POS
    let supervisor_row: Option<sqlx::sqlite::SqliteRow> = sqlx::query("SELECT id, pin_hash_v2 FROM users WHERE name = 'Supervisor POS'")
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to query Supervisor POS: {}", e))?;
    let supervisor_user_id = match supervisor_row {
        Some(row) => {
            let id: String = row.get("id");
            let pin_hash_v2: Option<String> = row.get("pin_hash_v2");
            if pin_hash_v2.is_none() {
                sqlx::query("UPDATE users SET pin_hash_v2 = ? WHERE id = ?")
                    .bind(&supervisor_pin_hash)
                    .bind(&id)
                    .execute(pool)
                    .await
                    .map_err(|e| format!("Failed to update supervisor pin hash: {}", e))?;
            }
            id
        }
        None => {
            let id = Uuid::new_v4().to_string();
            sqlx::query("INSERT INTO users (id, merchant_id, outlet_id, name, pin_hash_v2, status) VALUES (?, ?, ?, 'Supervisor POS', ?, 'active')")
                .bind(&id)
                .bind(&merchant_id)
                .bind(&outlet_id)
                .bind(&supervisor_pin_hash)
                .execute(pool)
                .await
                .map_err(|e| format!("Failed to seed supervisor user: {}", e))?;
            id
        }
    };

    // Cashier POS
    let cashier_row: Option<sqlx::sqlite::SqliteRow> = sqlx::query("SELECT id, pin_hash_v2 FROM users WHERE name = 'Kasir POS'")
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to query Kasir POS: {}", e))?;
    let cashier_user_id = match cashier_row {
        Some(row) => {
            let id: String = row.get("id");
            let pin_hash_v2: Option<String> = row.get("pin_hash_v2");
            if pin_hash_v2.is_none() {
                sqlx::query("UPDATE users SET pin_hash_v2 = ? WHERE id = ?")
                    .bind(&cashier_pin_hash)
                    .bind(&id)
                    .execute(pool)
                    .await
                    .map_err(|e| format!("Failed to update cashier pin hash: {}", e))?;
            }
            id
        }
        None => {
            let id = Uuid::new_v4().to_string();
            sqlx::query("INSERT INTO users (id, merchant_id, outlet_id, name, pin_hash_v2, status) VALUES (?, ?, ?, 'Kasir POS', ?, 'active')")
                .bind(&id)
                .bind(&merchant_id)
                .bind(&outlet_id)
                .bind(&cashier_pin_hash)
                .execute(pool)
                .await
                .map_err(|e| format!("Failed to seed cashier user: {}", e))?;
            id
        }
    };

    // Assign roles to users (idempotently)
    let role_bindings = vec![
        (&admin_user_id, "owner"),
        (&supervisor_user_id, "supervisor"),
        (&cashier_user_id, "cashier"),
    ];

    for (uid, rname) in role_bindings {
        let binding_exists: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT ur.user_id 
            FROM user_roles ur
            JOIN roles r ON ur.role_id = r.id
            WHERE ur.user_id = ? AND r.name = ?
            "#
        )
        .bind(uid)
        .bind(rname)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to check user role binding: {}", e))?;

        if binding_exists.is_none() {
            sqlx::query(
                "INSERT INTO user_roles (user_id, role_id) 
                 SELECT ?, id FROM roles WHERE merchant_id = ? AND name = ? LIMIT 1"
            )
            .bind(uid)
            .bind(&merchant_id)
            .bind(rname)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to assign global role {} to user: {}", rname, e))?;
        }

        // Assign user_outlet_roles (for Multi-outlet RBAC)
        let outlet_role_exists: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT uor.id
            FROM user_outlet_roles uor
            JOIN roles r ON uor.role_id = r.id
            WHERE uor.user_id = ? AND uor.outlet_id = ? AND r.name = ?
            "#
        )
        .bind(uid)
        .bind(&outlet_id)
        .bind(rname)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to check user outlet role binding: {}", e))?;

        if outlet_role_exists.is_none() {
            let uor_id = Uuid::new_v4().to_string();
            sqlx::query(
                r#"
                INSERT INTO user_outlet_roles (id, user_id, outlet_id, role_id, valid_from, valid_until, status, assigned_by)
                SELECT ?, ?, ?, id, datetime('now', '-1 day'), datetime('now', '+1 year'), 'ACTIVE', ?
                FROM roles WHERE merchant_id = ? AND name = ? LIMIT 1
                "#
            )
            .bind(&uor_id)
            .bind(uid)
            .bind(&outlet_id)
            .bind(&admin_user_id)
            .bind(&merchant_id)
            .bind(rname)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to assign outlet role {} to user: {}", rname, e))?;
        }
    }

    // Assign all permissions to 'owner' and 'supervisor' roles
    sqlx::query(
        "INSERT INTO role_permissions (role_id, permission_id)
         SELECT r.id, p.id FROM roles r, permissions p WHERE r.merchant_id = ? AND r.name IN ('owner', 'supervisor')
         ON CONFLICT DO NOTHING"
    )
    .bind(&merchant_id)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to assign permissions to owner/supervisor roles: {}", e))?;

    // Assign specific permissions to 'cashier' role (no refund, no audit)
    sqlx::query(
        "INSERT INTO role_permissions (role_id, permission_id)
         SELECT r.id, p.id FROM roles r, permissions p 
         WHERE r.merchant_id = ? AND r.name = 'cashier' AND p.key IN ('pos.sell', 'shift.manage')
         ON CONFLICT DO NOTHING"
    )
    .bind(&merchant_id)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to assign permissions to cashier role: {}", e))?;

    // 6. Dummy Products & Inventory (for M3 testing) - Seed only if no products exist
    let product_count: (i64,) = sqlx::query_as("SELECT count(*) FROM products")
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Failed to check product count: {}", e))?;

    if product_count.0 == 0 {
        println!("Seeding dummy products...");
        let cat_id = Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO categories (id, merchant_id, name) VALUES (?, ?, 'Kopi')")
            .bind(&cat_id)
            .bind(&merchant_id)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to seed category: {}", e))?;

        let dummy_products = vec![
            ("Kopi Hitam", "KPH-001", 15000),
            ("Kopi Susu Gula Aren", "KPS-001", 20000),
            ("Latte", "LAT-001", 25000),
            ("Cappuccino", "CAP-001", 25000)
        ];

        for (name, sku, price) in dummy_products {
            let prod_id = Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO products (id, merchant_id, category_id, sku, name, price, cost, track_stock, active)
                 VALUES (?, ?, ?, ?, ?, ?, ?, true, true)"
            )
            .bind(&prod_id)
            .bind(&merchant_id)
            .bind(&cat_id)
            .bind(sku)
            .bind(name)
            .bind(price)
            .bind(price - 5000) // Dummy cost
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to seed product {}: {}", name, e))?;

            // Stock
            sqlx::query(
                "INSERT INTO inventory_items (id, merchant_id, outlet_id, product_id, qty_on_hand, min_qty)
                 VALUES (?, ?, ?, ?, 100, 10)"
            )
            .bind(Uuid::new_v4().to_string())
            .bind(&merchant_id)
            .bind(&outlet_id)
            .bind(&prod_id)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to seed inventory for {}: {}", name, e))?;
        }
    }

    println!("Tenant seed completed successfully.");

    Ok(())
}

#[tauri::command]
pub async fn seed_database(pool: tauri::State<'_, SqlitePool>) -> Result<(), String> {
    run_seed(&pool).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::establish_connection;

    #[tokio::test]
    async fn test_seeder_idempotency() {
        std::env::set_var("DATABASE_URL", "sqlite::memory:");
        let pool = establish_connection().await.unwrap();
        crate::migration::run_migrations(&pool).await.unwrap();
        
        let res1 = run_seed(&pool).await;
        assert!(res1.is_ok(), "First seed failed");
        
        let res2 = run_seed(&pool).await;
        assert!(res2.is_ok(), "Second seed failed");
    }

    #[tokio::test]
    async fn test_seed_real_database() {
        std::env::remove_var("DATABASE_URL");
        let pool = establish_connection().await.unwrap();
        let _ = crate::migration::run_migrations(&pool).await;
        let res = run_seed(&pool).await;
        assert!(res.is_ok(), "Real database seed failed");
    }
}
