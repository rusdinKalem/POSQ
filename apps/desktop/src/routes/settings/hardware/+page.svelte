<script lang="ts">
    import { invoke } from '@tauri-apps/api/core';
    import { onMount } from 'svelte';
    import ReceiptPreview from '$lib/components/ReceiptPreview.svelte';

    let printerType = "Mock Printer";
    let isConnected = false;
    let barcodeConnected = false;
    
    let isPrinting = false;
    let printMessage = "";
    
    // Dummy receipt for testing
    const testReceipt = {
        store_name: "Toko Kelontong POSQ",
        store_address: "Jl. Merdeka No. 123\nJakarta Selatan",
        receipt_no: "TRX-TEST-001",
        date: new Date().toLocaleString(),
        cashier: "Admin/Kasir",
        items: [
            { name: "Kopi Susu Gula Aren", qty: 2, price: 15000, subtotal: 30000 },
            { name: "Roti Coklat", qty: 1, price: 12000, subtotal: 12000 }
        ],
        subtotal: 42000,
        tax: 0,
        total: 42000,
        payment_method: "tunai",
        amount_paid: 50000,
        change: 8000
    };

    async function loadStatus() {
        try {
            const res: any = await invoke('get_hardware_status');
            printerType = res.printer_type;
            isConnected = res.printer_connected;
            barcodeConnected = res.barcode_scanner_connected;
        } catch (err) {
            console.error(err);
        }
    }

    async function testPrint() {
        isPrinting = true;
        printMessage = "Mencetak struk percobaan...";
        try {
            // invoke hardware API
            await invoke('print_receipt', { data: testReceipt });
            printMessage = "Struk percobaan sukses dicetak (Periksa konsol terminal)!";
        } catch (err: any) {
            printMessage = "Gagal mencetak: " + err;
        } finally {
            isPrinting = false;
        }
    }

    onMount(() => {
        loadStatus();
    });
</script>

<div class="p-8 max-w-5xl mx-auto">
    <h1 class="text-3xl font-bold mb-6">Pengaturan Perangkat Keras</h1>
    <p class="text-gray-600 mb-8">Konfigurasikan koneksi Printer Struk (Termal) dan Pemindai Barcode untuk operasional kasir Anda.</p>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-8">
        <!-- Pengaturan Printer -->
        <div class="bg-white shadow rounded-lg p-6 border border-gray-200">
            <h2 class="text-xl font-bold mb-4 flex items-center gap-2">
                🖨️ Printer Struk
                {#if isConnected}
                    <span class="bg-green-100 text-green-800 text-xs px-2 py-1 rounded-full border border-green-300">Tersambung</span>
                {:else}
                    <span class="bg-red-100 text-red-800 text-xs px-2 py-1 rounded-full border border-red-300">Terputus</span>
                {/if}
            </h2>
            
            <div class="mb-4">
                <label for="printerType" class="block text-sm font-medium text-gray-700">Jenis Printer Aktif</label>
                <select id="printerType" bind:value={printerType} class="mt-1 block w-full rounded-md border-gray-300 shadow-sm border p-2">
                    <option value="Mock Printer">Mock Printer (Pengujian/Console)</option>
                    <option value="ESC/POS Network" disabled>ESC/POS Network (Segera Hadir)</option>
                    <option value="ESC/POS USB" disabled>ESC/POS USB (Segera Hadir)</option>
                </select>
            </div>

            <div class="mt-6 border-t pt-4">
                <button 
                    on:click={testPrint} 
                    disabled={isPrinting}
                    class="bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 text-white font-bold py-2 px-4 rounded shadow transition w-full">
                    {isPrinting ? "Sedang Mencetak..." : "Lakukan Uji Cetak (Test Print)"}
                </button>
                
                {#if printMessage}
                    <p class="mt-3 text-sm text-center font-medium {printMessage.includes('Gagal') ? 'text-red-600' : 'text-green-600'}">
                        {printMessage}
                    </p>
                {/if}
            </div>
        </div>
        
        <!-- Preview Layout -->
        <div class="bg-gray-100 rounded-lg p-6 flex flex-col items-center justify-center border border-gray-200 shadow-inner">
            <h3 class="text-sm font-bold text-gray-500 mb-4 uppercase tracking-wider">Pratinjau Struk Kasir</h3>
            <div class="transform scale-90 origin-top shadow-xl">
                <ReceiptPreview receiptData={testReceipt} />
            </div>
        </div>
    </div>
    
    <!-- Pengaturan Barcode Scanner -->
    <div class="bg-white shadow rounded-lg p-6 border border-gray-200 mt-8 opacity-60">
        <h2 class="text-xl font-bold mb-4 flex items-center gap-2">
            📷 Pemindai Barcode (Scanner)
        </h2>
        <p class="text-sm text-gray-600 mb-2">Sistem POSQ sudah dirancang mendukung <strong>Keyboard Wedge Scanner</strong>. Cukup colokkan scanner USB Anda; sistem secara otomatis menangkap pindaian barcode di halaman Kasir tanpa konfigurasi tambahan.</p>
        <div class="bg-yellow-50 p-3 rounded text-yellow-800 text-sm border border-yellow-200 inline-block mt-2">
            Status Native Scanner API: Segera Hadir
        </div>
    </div>
</div>
