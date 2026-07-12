# ADR-0012: Mobile vs. Desktop Hardware Integration Strategy

Status: Accepted  
Date: 2026-07-11

## Context

The POSQ application is built as a local-first application using Tauri v2. Currently targeting Desktop (Windows/Linux/macOS), the product roadmap includes expanding to Mobile (Android/iOS tablets and phones). 

Hardware integration is a major point of divergence between Desktop and Mobile platforms:
1. **Receipt Printers**:
   - *Desktop* systems typically connect to thermal POS printers via USB, Local Area Network (LAN/Ethernet), or Serial (RS-232) cables.
   - *Mobile* systems primarily use Bluetooth (Classic or BLE) portable/desktop thermal printers, since mobile devices lack standard USB/Serial ports.
2. **Barcode Scanners**:
   - *Desktop* terminals use physical USB or wireless hand-held scanners that operate as "Keyboard Wedge" devices (intercepting input as keyboard keystrokes).
   - *Mobile* terminals rely on the device's built-in camera as a barcode scanner due to the lack of dedicated USB ports.

To ensure long-term cross-platform code reuse and maintainability, a unified hardware integration and abstraction strategy is required.

## Decision

We will implement a unified **Hardware Abstraction Layer (HAL)** in both Svelte (frontend) and Rust (backend) to abstract printer and barcode scanner hardware interfaces:

1. **Receipt Printing Abstraction**:
   - The Svelte frontend will communicate only with a generic Tauri command `print_receipt(data: ReceiptData)`.
   - The Rust backend will resolve the active `printer_type` configuration and route print jobs to the corresponding driver/adapter:
     - **Desktop USB/LAN/Serial**: Handled directly in Rust using the `escpos` crate or raw TCP/Serial connections.
     - **Mobile Bluetooth (BLE)**: Handled by calling Tauri's native mobile plugins (e.g., `tauri-plugin-bluetooth` or custom Kotlin/Swift native bridges in the Tauri mobile project).
   - A fallback **Mock Printer** will be preserved for desktop and mobile testing.

2. **Barcode Scanner Abstraction**:
   - **Desktop (Keyboard Wedge)**: The Svelte application will listen for rapid keyboard input sequences at the document level (looking for specific timing thresholds and `Enter` suffixes typical of physical USB scanners) and redirect it to the checkout cart search.
   - **Mobile (Camera Scanner)**: Svelte will render an interactive camera preview modal using a lightweight web library (e.g., `html5-qrcode` or `jsqr`) or interface with a native Tauri Camera/Barcode plugin (e.g., `tauri-plugin-barcode-scanner`) to decode barcodes directly from the camera feed.
   - A manual barcode input field will remain available on all platforms.

## Alternatives Considered

| Alternative | Reason Rejected |
|---|---|
| Web Bluetooth API | Limited WebView compatibility on iOS/macOS, strict HTTPS constraints, and poor stability in embedded web views compared to native plugins. |
| Dedicated Mobile-only codebase | High maintenance overhead and violates the Tauri cross-platform promise of sharing core Svelte UI and SQLite/PostgreSQL schemas. |
| Relying only on USB scanners on Mobile | Requires OTG cables which are physically fragile, block device charging, and ruin the portable tablet form-factor. |

## Consequences

### Positive:
- **UI Consistency**: The Cashier POS view does not need to change when deploying to Mobile; it only triggers abstract scan/print actions.
- **Pluggability**: New hardware protocols (like Wi-Fi printing or specialized barcode APIs) can be added without modifying the core checkout codebase.
- **Testing Flexibility**: Developers can test the entire cashier flow using Mock Printer and simulated barcode events on standard development PCs.

### Negative:
- **Dependency Overhead**: Adding Bluetooth and Camera permissions requires platform-specific configuration in Tauri (e.g., `AndroidManifest.xml` for Android and `Info.plist` for iOS).
- **Driver Maintenance**: ESC/POS formatting differences can vary between Bluetooth thermal printers and USB desktop printers, requiring template normalization.

## Implementation Notes

- Create a shared `printer_type` config setting supporting: `mock`, `usb`, `network`, `serial`, and `bluetooth`.
- Create a shared `scanner_type` config setting supporting: `keyboard_wedge` (desktop) and `camera` (mobile).
- For Svelte 5, encapsulate the Keyboard Wedge event listener in a reusable action or store context, making it easy to disable or ignore when the Camera Scanner is active.
- For Bluetooth printing, the Rust commands will expect a MAC address or UUID identifier to initiate pairing and print payload dispatching.
