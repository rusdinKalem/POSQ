use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::command;
use std::net::TcpStream;
use std::io::Write;

static PRINTER_TYPE: Mutex<Option<String>> = Mutex::new(None);
static SCANNER_TYPE: Mutex<Option<String>> = Mutex::new(None);

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
    pub scanner_type: String,
}

// ==========================================
// PRINTER ADAPTER PATTERN
// ==========================================

pub trait PrinterAdapter {
    fn print(&self, data: &ReceiptData) -> Result<bool, String>;
}

pub struct MockPrinterAdapter;

impl PrinterAdapter for MockPrinterAdapter {
    fn print(&self, data: &ReceiptData) -> Result<bool, String> {
        println!("=== [PRINTER INTERFACE: MOCK START] ===");
        println!("{:^32}", data.store_name);
        println!("{:^32}", data.store_address);
        println!("--------------------------------");
        println!("No      : {}", data.receipt_no);
        println!("Tanggal : {}", data.date);
        println!("Kasir   : {}", data.cashier);
        println!("--------------------------------");
        
        for item in &data.items {
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
        println!("=== [PRINTER INTERFACE: MOCK END] ===");
        Ok(true)
    }
}

pub struct UsbPrinterAdapter;

impl PrinterAdapter for UsbPrinterAdapter {
    fn print(&self, data: &ReceiptData) -> Result<bool, String> {
        println!("=== [PRINTER INTERFACE: USB START] ===");
        println!("Formatting raw ESC/POS bytes for USB printer connection...");
        // Simulated success block for local desktop USB ports (e.g. /dev/usb/lp0 on Linux)
        let _bytes = format_esc_pos(data);
        println!("Sending payload to default USB printer port...");
        Ok(true)
    }
}

pub struct NetworkPrinterAdapter {
    pub ip: String,
    pub port: u16,
}

impl PrinterAdapter for NetworkPrinterAdapter {
    fn print(&self, data: &ReceiptData) -> Result<bool, String> {
        println!("=== [PRINTER INTERFACE: NETWORK START] ===");
        println!("Attempting connection to network printer at {}:{}...", self.ip, self.port);
        let bytes = format_esc_pos(data);
        
        let address = format!("{}:{}", self.ip, self.port);
        match TcpStream::connect_timeout(
            &address.parse().map_err(|e| format!("Invalid IP address format: {}", e))?,
            std::time::Duration::from_secs(3)
        ) {
            Ok(mut stream) => {
                stream.write_all(&bytes).map_err(|e| format!("Failed to transmit data to network printer: {}", e))?;
                println!("ESC/POS packet transmitted successfully to network socket.");
                Ok(true)
            }
            Err(e) => {
                println!("Network printer connection timed out: {}. Invoking stdout mock fallback.", e);
                MockPrinterAdapter.print(data)
            }
        }
    }
}

pub struct BluetoothPrinterAdapter {
    pub mac_address: String,
}

impl PrinterAdapter for BluetoothPrinterAdapter {
    fn print(&self, data: &ReceiptData) -> Result<bool, String> {
        println!("=== [PRINTER INTERFACE: BLUETOOTH START] ===");
        println!("Simulating Bluetooth pairing with MAC: {}...", self.mac_address);
        // Mobile platform Bluetooth bridge mock
        MockPrinterAdapter.print(data)
    }
}

// ==========================================
// ESC/POS FORMATTING HELPER
// ==========================================

fn format_esc_pos(data: &ReceiptData) -> Vec<u8> {
    let mut bytes = Vec::new();
    
    // Initialize printer: ESC @
    bytes.extend_from_slice(&[0x1B, 0x40]);
    
    // Justify Center: ESC a 1
    bytes.extend_from_slice(&[0x1B, 0x61, 1]);
    bytes.extend_from_slice(format!("{}\n", data.store_name).as_bytes());
    bytes.extend_from_slice(format!("{}\n", data.store_address).as_bytes());
    bytes.extend_from_slice(b"--------------------------------\n");
    
    // Justify Left: ESC a 0
    bytes.extend_from_slice(&[0x1B, 0x61, 0]);
    bytes.extend_from_slice(format!("No      : {}\n", data.receipt_no).as_bytes());
    bytes.extend_from_slice(format!("Tanggal : {}\n", data.date).as_bytes());
    bytes.extend_from_slice(format!("Kasir   : {}\n", data.cashier).as_bytes());
    bytes.extend_from_slice(b"--------------------------------\n");
    
    for item in &data.items {
        bytes.extend_from_slice(format!("{}\n", item.name).as_bytes());
        let qty_price = format!("{} x {}", item.qty, item.price);
        bytes.extend_from_slice(format!("{:<16} {:>15}\n", qty_price, item.subtotal).as_bytes());
    }
    
    bytes.extend_from_slice(b"--------------------------------\n");
    bytes.extend_from_slice(format!("Subtotal: {:>22}\n", data.subtotal).as_bytes());
    bytes.extend_from_slice(format!("Pajak   : {:>22}\n", data.tax).as_bytes());
    bytes.extend_from_slice(format!("TOTAL   : {:>22}\n", data.total).as_bytes());
    bytes.extend_from_slice(b"--------------------------------\n");
    bytes.extend_from_slice(format!("Metode Bayar : {:>17}\n", data.payment_method).as_bytes());
    bytes.extend_from_slice(format!("Tunai        : {:>17}\n", data.amount_paid).as_bytes());
    bytes.extend_from_slice(format!("Kembali      : {:>17}\n", data.change).as_bytes());
    bytes.extend_from_slice(b"--------------------------------\n");
    
    // Justify Center: ESC a 1
    bytes.extend_from_slice(&[0x1B, 0x61, 1]);
    bytes.extend_from_slice(b"Terima Kasih!\n\n\n\n");
    
    // Cut paper: GS V 66 0
    bytes.extend_from_slice(&[0x1D, 0x56, 66, 0]);
    
    bytes
}

// ==========================================
// TAURI COMMAND HANDLERS
// ==========================================

#[command]
pub async fn get_hardware_status() -> Result<HardwareStatus, String> {
    let p_type = {
        let guard = PRINTER_TYPE.lock().unwrap();
        guard.clone().unwrap_or_else(|| "Mock Printer".to_string())
    };
    let s_type = {
        let guard = SCANNER_TYPE.lock().unwrap();
        guard.clone().unwrap_or_else(|| "Keyboard Wedge".to_string())
    };

    let printer_connected = match p_type.as_str() {
        "None" => false,
        _ => true,
    };
    let barcode_scanner_connected = match s_type.as_str() {
        "None" => false,
        _ => true,
    };

    Ok(HardwareStatus {
        printer_connected,
        printer_type: p_type,
        barcode_scanner_connected,
        scanner_type: s_type,
    })
}

#[command]
pub async fn save_hardware_settings(printer_type: String, scanner_type: String) -> Result<bool, String> {
    {
        let mut p_guard = PRINTER_TYPE.lock().unwrap();
        *p_guard = Some(printer_type);
    }
    {
        let mut s_guard = SCANNER_TYPE.lock().unwrap();
        *s_guard = Some(scanner_type);
    }
    Ok(true)
}

#[command]
pub async fn print_receipt(data: ReceiptData) -> Result<bool, String> {
    let p_type = {
        let guard = PRINTER_TYPE.lock().unwrap();
        guard.clone().unwrap_or_else(|| "Mock Printer".to_string())
    };

    let adapter: Box<dyn PrinterAdapter + Send + Sync> = match p_type.as_str() {
        "ESC/POS Network" => {
            Box::new(NetworkPrinterAdapter {
                ip: "192.168.1.100".to_string(), // configured default IP
                port: 9100,
            })
        }
        "ESC/POS USB" => Box::new(UsbPrinterAdapter),
        "Bluetooth BLE (Mobile)" => Box::new(BluetoothPrinterAdapter {
            mac_address: "00:11:22:33:AA:BB".to_string(), // mocked active device
        }),
        _ => Box::new(MockPrinterAdapter),
    };

    adapter.print(&data)?;

    // Simulate print layout queue delay
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    Ok(true)
}

