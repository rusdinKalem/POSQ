use axum::{
    routing::get,
    Router, Json, extract::State, http::StatusCode,
};
use tower_http::cors::{CorsLayer, Any};
use sqlx::SqlitePool;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[derive(Serialize, Deserialize)]
pub struct ServerProduct {
    pub id: String,
    pub sku: String,
    pub name: String,
    pub description: Option<String>,
    pub unit_price: f64,
    pub cost_price: f64,
    pub stock_quantity: f64,
    pub category: String,
    pub track_stock: bool,
    pub has_variants: bool,
    pub image_path: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

// Basic product fetching for clients
async fn list_products(State(state): State<Arc<AppState>>) -> Result<Json<Vec<ServerProduct>>, (StatusCode, String)> {
    let pool = &state.db;
    
    // Using the same logic from inventory::list_products but accessed via HTTP
    // This is a simplified version just to demonstrate the concept.
    let records = sqlx::query(
        "SELECT id, sku, name, description, unit_price, cost_price, stock_quantity, category, track_stock, has_variants, image_path, created_at, updated_at 
         FROM products ORDER BY name ASC"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    use sqlx::Row;
    let mut products = Vec::new();
    for r in records {
        products.push(ServerProduct {
            id: r.try_get("id").unwrap_or_default(),
            sku: r.try_get("sku").unwrap_or_default(),
            name: r.try_get("name").unwrap_or_default(),
            description: r.try_get("description").ok(),
            unit_price: crate::db::get_numeric_as_f64(&r, "unit_price"),
            cost_price: crate::db::get_numeric_as_f64(&r, "cost_price"),
            stock_quantity: r.try_get::<f64, _>("stock_quantity").unwrap_or(0.0),
            category: r.try_get("category").unwrap_or_default(),
            track_stock: r.try_get("track_stock").unwrap_or(false),
            has_variants: r.try_get("has_variants").unwrap_or(false),
            image_path: r.try_get("image_path").ok(),
            created_at: r.try_get("created_at").unwrap_or_default(),
            updated_at: r.try_get("updated_at").unwrap_or_default(),
        });
    }

    Ok(Json(products))
}

pub async fn start_server(pool: SqlitePool, port: u16) {
    let state = Arc::new(AppState { db: pool });

    // Enable CORS for all origins (useful for local network)
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/products", get(list_products))
        .layer(cors)
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    println!("Starting local master server on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
