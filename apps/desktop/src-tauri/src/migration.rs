use sqlx::SqlitePool;

pub async fn run_migrations(pool: &SqlitePool) -> Result<(), String> {
    // Backup is now handled by the safe_migration orchestrator in update.rs

    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| format!("Failed to run migrations: {}", e))?;
        
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::establish_connection;

    #[tokio::test]
    async fn test_migrations() {
        let pool = establish_connection().await.unwrap();
        let result = run_migrations(&pool).await;
        assert!(result.is_ok(), "Migrations failed");
        
        // Try running again to test idempotency
        let result2 = run_migrations(&pool).await;
        assert!(result2.is_ok(), "Idempotent migrations failed");
    }
}
