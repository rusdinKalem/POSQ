use sqlx::sqlite::{SqlitePoolOptions, SqliteConnectOptions};
use sqlx::SqlitePool;
use std::str::FromStr;

pub async fn establish_connection() -> Result<SqlitePool, String> {
    // Determine the database URL. Check environment first, otherwise fall back to local file.
    let database_url = if let Ok(url) = std::env::var("DATABASE_URL") {
        url
    } else {
        let mut path = dirs::data_dir().ok_or_else(|| "Could not find local app data directory".to_string())?;
        path.push("POSQ");
        std::fs::create_dir_all(&path).map_err(|e| format!("Failed to create POSQ data directory: {}", e))?;
        path.push("posq.db");
        format!("sqlite://{}", path.to_string_lossy())
    };

    let connection_options = SqliteConnectOptions::from_str(&database_url)
        .map_err(|e| format!("Invalid connection URL: {}", e))?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
        .busy_timeout(std::time::Duration::from_millis(5000));

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connection_options)
        .await
        .map_err(|e| format!("Failed to connect to SQLite: {}", e))?;

    Ok(pool)
}

pub fn get_numeric_as_f64(row: &sqlx::sqlite::SqliteRow, column: &str) -> f64 {
    use sqlx::Row;
    row.try_get::<f64, _>(column)
        .or_else(|_| row.try_get::<i64, _>(column).map(|i| i as f64))
        .unwrap_or(0.0)
}
