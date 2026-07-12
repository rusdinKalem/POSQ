use serde::{Deserialize, Serialize};
use tauri::{command, State};
use sqlx::{SqlitePool, Row};

#[derive(Serialize, Deserialize, Debug)]
pub struct KitchenItem {
    pub name: String,
    pub qty: i32,
    pub notes: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KitchenOrderData {
    pub table_no: String,
    pub order_no: String,
    pub time: String,
    pub items: Vec<KitchenItem>,
}

#[derive(Serialize, Deserialize)]
pub struct TableStatus {
    pub id: String,
    pub name: String,
    pub is_occupied: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KdsTicket {
    pub id: String,
    pub reference_id: String,
    pub reference_type: String,
    pub order_number: String,
    pub table_number: Option<String>,
    pub order_type: String,
    pub status: String,
    pub items_json: String,
    pub created_at: String,
    pub updated_at: String,
}

#[command]
pub async fn get_tables() -> Result<Vec<TableStatus>, String> {
    // Mock 20 tables
    let mut tables = Vec::new();
    for i in 1..=20 {
        tables.push(TableStatus {
            id: format!("table_{}", i),
            name: format!("Meja {}", i),
            is_occupied: i % 5 == 0, // Mock: Every 5th table is occupied
        });
    }
    Ok(tables)
}

#[command]
pub async fn print_kitchen_ticket(data: KitchenOrderData) -> Result<bool, String> {
    // SEC-001: Enforce license status to prevent API bypass
    crate::license::enforce_active_license().await?;

    // M12-001 F&B basic mode: Kitchen Print
    println!("======================================");
    println!("{:^38}", "[ KITCHEN TICKET ]");
    println!("======================================");
    println!("Order No : {}", data.order_no);
    println!("Table No : {}", data.table_no);
    println!("Time     : {}", data.time);
    println!("--------------------------------------");
    
    for item in data.items {
        println!("{}x {}", item.qty, item.name);
        if let Some(notes) = item.notes {
            if !notes.is_empty() {
                println!("   * Notes: {}", notes);
            }
        }
        println!();
    }
    println!("======================================");
    
    // Simulate printer delay
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    Ok(true)
}

#[command]
pub async fn get_active_kds_tickets(pool: State<'_, SqlitePool>) -> Result<Vec<KdsTicket>, String> {
    crate::license::enforce_active_license().await?;

    let records = sqlx::query(
        "SELECT id, reference_id, reference_type, order_number, table_number, order_type, status, items_json, created_at, updated_at \
         FROM kds_tickets WHERE status IN ('pending', 'cooking') ORDER BY created_at ASC"
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let tickets = records.into_iter().map(|r| KdsTicket {
        id: r.get("id"),
        reference_id: r.get("reference_id"),
        reference_type: r.get("reference_type"),
        order_number: r.get("order_number"),
        table_number: r.get("table_number"),
        order_type: r.get("order_type"),
        status: r.get("status"),
        items_json: r.get("items_json"),
        created_at: r.get("created_at"),
        updated_at: r.get("updated_at"),
    }).collect();

    Ok(tickets)
}

#[command]
pub async fn get_completed_kds_tickets(pool: State<'_, SqlitePool>) -> Result<Vec<KdsTicket>, String> {
    crate::license::enforce_active_license().await?;

    let records = sqlx::query(
        "SELECT id, reference_id, reference_type, order_number, table_number, order_type, status, items_json, created_at, updated_at \
         FROM kds_tickets WHERE status = 'done' ORDER BY updated_at DESC LIMIT 50"
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let tickets = records.into_iter().map(|r| KdsTicket {
        id: r.get("id"),
        reference_id: r.get("reference_id"),
        reference_type: r.get("reference_type"),
        order_number: r.get("order_number"),
        table_number: r.get("table_number"),
        order_type: r.get("order_type"),
        status: r.get("status"),
        items_json: r.get("items_json"),
        created_at: r.get("created_at"),
        updated_at: r.get("updated_at"),
    }).collect();

    Ok(tickets)
}

#[command]
pub async fn update_kds_ticket_status(id: String, status: String, pool: State<'_, SqlitePool>) -> Result<(), String> {
    crate::license::enforce_active_license().await?;

    sqlx::query(
        "UPDATE kds_tickets SET status = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"
    )
    .bind(status)
    .bind(id)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[command]
pub async fn save_kds_ticket(
    reference_id: String,
    reference_type: String,
    order_number: String,
    table_number: Option<String>,
    order_type: String,
    items_json: String,
    pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    crate::license::enforce_active_license().await?;

    sqlx::query(
        "INSERT INTO kds_tickets (id, reference_id, reference_type, order_number, table_number, order_type, status, items_json, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, 'pending', ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP) \
         ON CONFLICT(reference_id) DO UPDATE SET \
            order_number = excluded.order_number, \
            table_number = excluded.table_number, \
            order_type = excluded.order_type, \
            items_json = excluded.items_json, \
            updated_at = CURRENT_TIMESTAMP"
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(reference_id)
    .bind(reference_type)
    .bind(order_number)
    .bind(table_number)
    .bind(order_type)
    .bind(items_json)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[command]
pub async fn link_kds_draft_to_order(
    draft_id: String,
    order_id: String,
    order_number: String,
    pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    crate::license::enforce_active_license().await?;

    sqlx::query(
        "UPDATE kds_tickets SET reference_id = ?, reference_type = 'order', order_number = ?, updated_at = CURRENT_TIMESTAMP WHERE reference_id = ?"
    )
    .bind(order_id)
    .bind(order_number)
    .bind(draft_id)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

