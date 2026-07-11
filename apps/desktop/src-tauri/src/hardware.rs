use serde::{Deserialize, Serialize};
use tauri::command;

#[derive(Serialize, Deserialize, Debug)]
pub struct ReceiptItem {
    pub name: String,
    pub qty: i32,
    pub price: f64,
    pub subtotal: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReceiptData {
    pub store_name: String,
    pub store_address: String,
    pub receipt_no: String,
    pub date: String,
    pub cashier: String,
    pub items: Vec<ReceiptItem>,
    pub subtotal: f64,
    pub tax: f64,
    pub total: f64,
    pub payment_method: String,
    pub amount_paid: f64,
    pub change: f64,
}

#[derive(Serialize, Deserialize)]
pub struct HardwareStatus {
    pub printer_connected: bool,
    pub printer_type: String,
    pub barcode_scanner_connected: bool,
}

#[command]
pub async fn get_hardware_status() -> Result<HardwareStatus, String> {
    // Dummy status for MVP
    Ok(HardwareStatus {
        printer_connected: true,
        printer_type: "Mock Printer".into(),
        barcode_scanner_connected: false,
    })
}

#[command]
pub async fn print_receipt(data: ReceiptData) -> Result<bool, String> {
    // M11-001 Printer mock & M11-003 Generic print command
    println!("=== [MOCK PRINTER START] ===");
    println!("{:^32}", data.store_name);
    println!("{:^32}", data.store_address);
    println!("--------------------------------");
    println!("No      : {}", data.receipt_no);
    println!("Tanggal : {}", data.date);
    println!("Kasir   : {}", data.cashier);
    println!("--------------------------------");
    
    for item in data.items {
        println!("{:<32}", item.name);
        let qty_price = format!("{} x {}", item.qty, item.price);
        println!("{:<16} {:>15}", qty_price, item.subtotal);
    }
    
    println!("--------------------------------");
    println!("Subtotal: {:>22}", data.subtotal);
    println!("Pajak   : {:>22}", data.tax);
    println!("TOTAL   : {:>22}", data.total);
    println!("--------------------------------");
    println!("Metode Bayar : {:>17}", data.payment_method);
    println!("Tunai        : {:>17}", data.amount_paid);
    println!("Kembali      : {:>17}", data.change);
    println!("--------------------------------");
    println!("{:^32}", "Terima Kasih!");
    println!("=== [MOCK PRINTER END] ===");
    
    // Simulate printing delay
    tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;

    Ok(true)
}
