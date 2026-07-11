use serde::{Deserialize, Serialize};
use tauri::command;

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
