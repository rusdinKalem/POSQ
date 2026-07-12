use serde::{Deserialize, Serialize};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

#[derive(Serialize, Deserialize, Clone)]
pub struct EcrResult {
    pub success: bool,
    pub approval_code: Option<String>,
    pub trace_number: Option<String>,
    pub message: String,
}

#[tauri::command]
pub async fn start_ecr_transaction(
    app: AppHandle,
    _amount: f64,
    port_name: String,
    baud_rate: u32,
) -> Result<(), String> {
    // We spawn this in the background so it doesn't block the UI thread.
    tauri::async_runtime::spawn(async move {
        // Emit an event that we are starting
        let _ = app.emit("ecr-status", "Membuka port serial...");

        // Try to open the port (this is a blocking call, but we are inside a spawned task,
        // although technically we should use spawn_blocking for serial port operations)
        let port_result = tokio::task::spawn_blocking(move || {
            let _port = serialport::new(&port_name, baud_rate)
                .timeout(Duration::from_secs(10))
                .open()?;
                
            // MOCK: In a real implementation, you'd send specific hex codes (e.g. STX, SALE, Amount, ETX, LRC).
            // Example:
            // let req = format!("\x02SALE{:012}\x03", amount as u64);
            // port.write_all(req.as_bytes())?;
            
            // Wait for response...
            // let mut serial_buf: Vec<u8> = vec![0; 1000];
            // let t = port.read(serial_buf.as_mut_slice())?;
            // parse the response...

            // For now, since we're mocking, we just pretend it was successful if the port opens
            // Or if we specifically want to mock a timeout, we could sleep here.
            
            Ok::<(), serialport::Error>(())
        })
        .await;

        match port_result {
            Ok(Ok(_)) => {
                // Sleep to mock the customer entering PIN
                let _ = app.emit("ecr-status", "Menunggu Pelanggan (Memasukkan PIN)...");
                tokio::time::sleep(Duration::from_secs(3)).await;

                // Mock Success
                let result = EcrResult {
                    success: true,
                    approval_code: Some("889900".to_string()),
                    trace_number: Some("000123".to_string()),
                    message: "Transaksi ECR Berhasil".to_string(),
                };
                let _ = app.emit("ecr-transaction-result", result);
            }
            Ok(Err(e)) => {
                let result = EcrResult {
                    success: false,
                    approval_code: None,
                    trace_number: None,
                    message: format!("Gagal membuka port: {}", e),
                };
                let _ = app.emit("ecr-transaction-result", result);
            }
            Err(e) => {
                let result = EcrResult {
                    success: false,
                    approval_code: None,
                    trace_number: None,
                    message: format!("Task panic: {}", e),
                };
                let _ = app.emit("ecr-transaction-result", result);
            }
        }
    });

    Ok(())
}
