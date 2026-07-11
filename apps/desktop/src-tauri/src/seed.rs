use sqlx::SqlitePool;
use uuid::Uuid;

pub async fn run_seed(pool: &SqlitePool) -> Result<(), String> {
    println!("Running tenant seed...");
    
    // Check if any user exists (run migrations if database is not migrated yet)
    let count: (i64,) = match sqlx::query_as("SELECT count(*) FROM users")
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

    if count.0 > 0 {
        println!("Tenant already seeded.");
        return Ok(());
    }

    println!("Seeding default tenant...");

    let merchant_id = Uuid::new_v4().to_string();
    let outlet_id = Uuid::new_v4().to_string();

    // 1. Merchant
    sqlx::query("INSERT INTO merchants (id, name) VALUES (?, 'Toko Demo POSQ')")
        .bind(&merchant_id)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to seed merchant: {}", e))?;

    // 2. Outlet
    sqlx::query("INSERT INTO outlets (id, merchant_id, name, code) VALUES (?, ?, 'Cabang Utama', 'HO01')")
        .bind(&outlet_id)
        .bind(&merchant_id)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to seed outlet: {}", e))?;

    // 3. Roles
    let roles = vec![
        "owner",
        "manager",
        "cashier",
        "inventory",
        "finance"
    ];

    for role_name in roles {
        sqlx::query("INSERT INTO roles (id, merchant_id, name, system_role) VALUES (?, ?, ?, true)")
            .bind(Uuid::new_v4().to_string())
            .bind(&merchant_id)
            .bind(role_name)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to seed role {}: {}", role_name, e))?;
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

    // 5. Default Admin User
    let admin_user_id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO users (id, merchant_id, outlet_id, name, status) VALUES (?, ?, ?, 'Owner POS', 'active')")
        .bind(&admin_user_id)
        .bind(&merchant_id)
        .bind(&outlet_id)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to seed admin user: {}", e))?;

    // Assign owner role to admin user
    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) 
         SELECT ?, id FROM roles WHERE merchant_id = ? AND name = 'owner' LIMIT 1"
    )
    .bind(&admin_user_id)
    .bind(&merchant_id)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to assign role to admin: {}", e))?;

    // Assign all permissions to 'owner' role
    sqlx::query(
        "INSERT INTO role_permissions (role_id, permission_id)
         SELECT r.id, p.id FROM roles r, permissions p WHERE r.merchant_id = ? AND r.name = 'owner'
         ON CONFLICT DO NOTHING"
    )
    .bind(&merchant_id)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to assign permissions to owner role: {}", e))?;

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

    // 6. Dummy Products & Inventory (for M3 testing)
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
        let pool = establish_connection().await.unwrap();
        crate::migration::run_migrations(&pool).await.unwrap();
        
        let res1 = run_seed(&pool).await;
        assert!(res1.is_ok(), "First seed failed");
        
        let res2 = run_seed(&pool).await;
        assert!(res2.is_ok(), "Second seed failed");
    }
}
