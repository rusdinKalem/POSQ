<script lang="ts">
    import { invoke } from '@tauri-apps/api/core';
    import { onMount } from 'svelte';
    import ReceiptPreview from '$lib/components/ReceiptPreview.svelte';
    import BackButton from '$lib/components/BackButton.svelte';

    let printerType = "Mock Printer";
    let scannerType = "Keyboard Wedge";
    
    let isConnected = false;
    let barcodeConnected = false;
    
    let isPrinting = false;
    let printMessage = "";
    let saveMessage = "";
    
    // EDC ECR Integration state
    let ecrEnabled = false;
    let ecrPortName = "COM3";
    let ecrBaudRate = 115200;
    
    // Mobile BLE Simulation state
    let bleDevices = [
        { name: "RPP02N Mini Printer", mac: "00:11:22:33:AA:BB", status: "Terputus" },
        { name: "Panda PRJ-58D BLE", mac: "AA:BB:CC:DD:EE:FF", status: "Terputus" },
        { name: "Eppos EP5802AI", mac: "55:66:77:88:99:AA", status: "Terputus" }
    ];
    let isScanningBle = false;
    let pairedBleMac = "";

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
            scannerType = res.scanner_type || "Keyboard Wedge";
        } catch (err) {
            console.error(err);
        }
    }

    async function saveSettings() {
        saveMessage = "Menyimpan pengaturan...";
        try {
            await invoke('save_hardware_settings', { printerType, scannerType });
            
            // Save EDC settings to localStorage
            localStorage.setItem("posq_ecr_enabled", ecrEnabled ? "true" : "false");
            localStorage.setItem("posq_ecr_port", ecrPortName);
            localStorage.setItem("posq_ecr_baud", ecrBaudRate.toString());
            
            saveMessage = "Pengaturan berhasil disimpan!";
            await loadStatus();
            setTimeout(() => {
                saveMessage = "";
            }, 3000);
        } catch (err: any) {
            saveMessage = "Gagal menyimpan: " + err;
        }
    }

    async function testPrint() {
        isPrinting = true;
        printMessage = "Mencetak struk percobaan...";
        try {
            await invoke('print_receipt', { data: testReceipt });
            printMessage = `Struk percobaan sukses dicetak (Periksa konsol / interface)!`;
        } catch (err: any) {
            printMessage = "Gagal mencetak: " + err;
        } finally {
            isPrinting = false;
        }
    }

    function scanBle() {
        isScanningBle = true;
        setTimeout(() => {
            isScanningBle = false;
        }, 1500);
    }

    function pairBleDevice(mac: string) {
        pairedBleMac = mac;
        bleDevices = bleDevices.map(d => ({
            ...d,
            status: d.mac === mac ? "Tersambung" : "Terputus"
        }));
        isConnected = true;
    }

    onMount(() => {
        loadStatus();
        
        // Load EDC Settings
        ecrEnabled = localStorage.getItem("posq_ecr_enabled") === "true";
        ecrPortName = localStorage.getItem("posq_ecr_port") || "COM3";
        ecrBaudRate = parseInt(localStorage.getItem("posq_ecr_baud") || "115200");
    });
</script>

<div class="p-8 max-w-5xl mx-auto">
    <BackButton />
    <div class="flex justify-between items-center mb-6">
        <div>
            <h1 class="text-3xl font-extrabold text-slate-800">Pengaturan Perangkat Keras</h1>
            <p class="text-slate-500 mt-1">Konfigurasikan printer struk dan scanner barcode untuk operasional kasir Desktop maupun Mobile.</p>
        </div>
        <div class="flex items-center gap-4">
            {#if saveMessage}
                <span class="text-sm font-semibold text-green-600 bg-green-50 px-3 py-1.5 rounded-full border border-green-200 animate-pulse">{saveMessage}</span>
            {/if}
            <button onclick={saveSettings} class="bg-blue-600 hover:bg-blue-700 text-white font-bold py-2.5 px-6 rounded-lg shadow-md transition-all">
                Simpan Konfigurasi
            </button>
        </div>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
        <!-- Pengaturan Printer & Scanner (lg:col-span-2) -->
        <div class="lg:col-span-2 space-y-8">
            
            <!-- Printer Panel -->
            <div class="bg-white shadow-md rounded-2xl p-6 border border-slate-200">
                <div class="flex justify-between items-center mb-6">
                    <h2 class="text-xl font-bold text-slate-800 flex items-center gap-2">
                        🖨️ Printer Struk Kasir
                    </h2>
                    {#if isConnected}
                        <span class="bg-green-100 text-green-800 text-xs font-bold px-3 py-1 rounded-full border border-green-200">Tersambung</span>
                    {:else}
                        <span class="bg-slate-100 text-slate-600 text-xs font-bold px-3 py-1 rounded-full border border-slate-200">Terputus</span>
                    {/if}
                </div>
                
                <div class="space-y-4">
                    <div>
                        <label for="printerType" class="block text-xs font-bold text-slate-400 uppercase tracking-wider mb-2">Jenis Koneksi Printer</label>
                        <select id="printerType" bind:value={printerType} class="block w-full rounded-xl border-slate-200 shadow-sm border p-3 text-slate-700 bg-slate-50 focus:bg-white focus:border-blue-500 transition-colors">
                            <option value="Mock Printer">Mock Printer (Pengujian/Console)</option>
                            <option value="ESC/POS Network">ESC/POS Network (LAN/Wi-Fi)</option>
                            <option value="ESC/POS USB">ESC/POS USB (Kabel Desktop)</option>
                            <option value="Bluetooth BLE (Mobile)">Bluetooth BLE (Mobile Android/iOS)</option>
                        </select>
                    </div>

                    <!-- Config based on Printer Type -->
                    {#if printerType === 'ESC/POS Network'}
                        <div class="p-4 bg-blue-50 rounded-xl border border-blue-200 space-y-3">
                            <h4 class="font-bold text-blue-800 text-sm">Konfigurasi Printer Jaringan (LAN/Wi-Fi)</h4>
                            <div class="grid grid-cols-2 gap-3">
                                <div>
                                    <label class="block text-[10px] font-bold text-blue-600 mb-1" for="printerIp">IP Address Printer</label>
                                    <input id="printerIp" type="text" class="w-full p-2 border rounded-lg bg-white" placeholder="192.168.1.100" />
                                </div>
                                <div>
                                    <label class="block text-[10px] font-bold text-blue-600 mb-1" for="printerPort">Port</label>
                                    <input id="printerPort" type="number" class="w-full p-2 border rounded-lg bg-white" placeholder="9100" />
                                </div>
                            </div>
                        </div>
                    {:else if printerType === 'ESC/POS USB'}
                        <div class="p-4 bg-slate-50 rounded-xl border border-slate-200">
                            <h4 class="font-bold text-slate-800 text-sm mb-2">Pilih Port USB Printer (Desktop)</h4>
                            <select class="w-full p-2 border rounded-lg bg-white">
                                <option>USB001 (Thermal Printer 58mm)</option>
                                <option>USB002 (Generic POS Printer 80mm)</option>
                            </select>
                        </div>
                    {:else if printerType === 'Bluetooth BLE (Mobile)'}
                        <div class="p-4 bg-purple-50 rounded-xl border border-purple-200 space-y-4">
                            <div class="flex justify-between items-center">
                                <h4 class="font-bold text-purple-800 text-sm">Pindai Printer Bluetooth BLE (Mobile/Tablet)</h4>
                                <button onclick={scanBle} disabled={isScanningBle} class="bg-purple-600 hover:bg-purple-700 disabled:bg-purple-300 text-white text-xs font-bold px-3 py-1.5 rounded-lg shadow-sm transition">
                                    {isScanningBle ? "Memindai..." : "Mulai Pindai"}
                                </button>
                            </div>

                            {#if isScanningBle}
                                <div class="flex items-center justify-center py-4 gap-2 text-purple-600">
                                    <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-purple-600"></div>
                                    <span class="text-xs font-bold">Mencari printer Bluetooth sekitar...</span>
                                </div>
                            {:else}
                                <div class="space-y-2">
                                    {#each bleDevices as dev}
                                        <div class="flex justify-between items-center bg-white p-3 rounded-lg border border-purple-100 shadow-xs">
                                            <div>
                                                <div class="font-bold text-slate-800 text-sm">{dev.name}</div>
                                                <div class="text-[10px] text-slate-400 font-mono">{dev.mac}</div>
                                            </div>
                                            <div>
                                                {#if dev.status === 'Tersambung'}
                                                    <span class="text-green-600 font-bold text-xs mr-2">Tersambung</span>
                                                {:else}
                                                    <button onclick={() => pairBleDevice(dev.mac)} class="bg-purple-100 hover:bg-purple-200 text-purple-700 font-bold text-xs px-2.5 py-1 rounded">Hubungkan</button>
                                                {/if}
                                            </div>
                                        </div>
                                    {/each}
                                </div>
                            {/if}
                        </div>
                    {/if}

                    <div class="pt-4 border-t border-slate-100">
                        <button 
                            onclick={testPrint} 
                            disabled={isPrinting}
                            class="bg-slate-800 hover:bg-slate-900 disabled:bg-slate-300 text-white font-bold py-2.5 px-4 rounded-xl shadow-md transition w-full text-sm">
                            {isPrinting ? "Mencetak Struk Percobaan..." : "Lakukan Uji Cetak (Test Print)"}
                        </button>
                        
                        {#if printMessage}
                            <p class="mt-3 text-xs text-center font-bold {printMessage.includes('Gagal') ? 'text-red-600' : 'text-green-600'}">
                                {printMessage}
                            </p>
                        {/if}
                    </div>
                </div>
            </div>

            <!-- Barcode Scanner Panel -->
            <div class="bg-white shadow-md rounded-2xl p-6 border border-slate-200">
                <div class="flex justify-between items-center mb-6">
                    <h2 class="text-xl font-bold text-slate-800 flex items-center gap-2">
                        📷 Pemindai Barcode (Scanner)
                    </h2>
                    {#if barcodeConnected}
                        <span class="bg-green-100 text-green-800 text-xs font-bold px-3 py-1 rounded-full border border-green-200">Aktif</span>
                    {:else}
                        <span class="bg-slate-100 text-slate-600 text-xs font-bold px-3 py-1 rounded-full border border-slate-200">Aktif (Simulasi)</span>
                    {/if}
                </div>

                <div class="space-y-4">
                    <div>
                        <label for="scannerType" class="block text-xs font-bold text-slate-400 uppercase tracking-wider mb-2">Mode Pemindaian Barcode</label>
                        <select id="scannerType" bind:value={scannerType} class="block w-full rounded-xl border-slate-200 shadow-sm border p-3 text-slate-700 bg-slate-50 focus:bg-white focus:border-blue-500 transition-colors">
                            <option value="Keyboard Wedge">Keyboard Wedge (Fisik USB / Desktop)</option>
                            <option value="Kamera Internal (Mobile)">Kamera Internal (Mobile Tablet/HP)</option>
                        </select>
                    </div>

                    {#if scannerType === 'Keyboard Wedge'}
                        <div class="p-4 bg-slate-50 rounded-xl border border-slate-200">
                            <h4 class="font-bold text-slate-800 text-sm mb-1">Keyboard Wedge Mode</h4>
                            <p class="text-xs text-slate-500">Sistem POSQ akan mendeteksi scanner USB fisik Anda secara otomatis. Scanner bertindak seperti keyboard yang mengetik kode SKU produk dengan cepat diikuti dengan tombol Enter.</p>
                        </div>
                    {:else if scannerType === 'Kamera Internal (Mobile)'}
                        <div class="p-4 bg-orange-50 rounded-xl border border-orange-200 space-y-2">
                            <h4 class="font-bold text-orange-800 text-sm">Mode Kamera Tablet / Handphone</h4>
                            <p class="text-xs text-orange-600">POSQ akan menampilkan popup viewfinder kamera web/tablet saat tombol scan ditekan di halaman kasir. Menggunakan kamera bawaan untuk memindai barcode produk secara instan.</p>
                            <div class="bg-white p-3 rounded-lg border border-orange-100 text-[10px] text-slate-500">
                                💡 <em>Memerlukan izin akses kamera pada browser atau perangkat Android/iOS tablet Anda.</em>
                            </div>
                        </div>
                    {/if}
                </div>
            </div>

            <!-- Integrasi Mesin EDC (ECR) -->
            <div class="bg-white rounded-2xl shadow-sm border border-slate-200 p-6">
                <div class="flex justify-between items-center mb-6">
                    <div>
                        <h2 class="text-xl font-bold text-slate-800">💳 Mesin EDC (ECR Protocol)</h2>
                        <p class="text-sm text-slate-500 mt-1">Integrasi pembayaran kartu otomatis ke mesin fisik EDC bank via Serial Port</p>
                    </div>
                    <label class="relative inline-flex items-center cursor-pointer">
                        <input type="checkbox" bind:checked={ecrEnabled} class="sr-only peer">
                        <div class="w-14 h-7 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-6 after:w-6 after:transition-all peer-checked:bg-blue-600"></div>
                    </label>
                </div>
                
                {#if ecrEnabled}
                <div class="space-y-4">
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div>
                            <label for="ecrPort" class="block text-xs font-bold text-slate-400 uppercase tracking-wider mb-2">Nama Port Serial (COM/USB)</label>
                            <input id="ecrPort" type="text" bind:value={ecrPortName} placeholder="Misal: COM3 atau /dev/ttyUSB0" class="block w-full rounded-xl border-slate-200 shadow-sm border p-3 text-slate-700 bg-slate-50 focus:bg-white focus:border-blue-500 transition-colors">
                        </div>
                        <div>
                            <label for="ecrBaud" class="block text-xs font-bold text-slate-400 uppercase tracking-wider mb-2">Baud Rate</label>
                            <select id="ecrBaud" bind:value={ecrBaudRate} class="block w-full rounded-xl border-slate-200 shadow-sm border p-3 text-slate-700 bg-slate-50 focus:bg-white focus:border-blue-500 transition-colors">
                                <option value={9600}>9600 (BCA Standard)</option>
                                <option value={19200}>19200</option>
                                <option value={38400}>38400</option>
                                <option value={115200}>115200 (Mandiri / Modern)</option>
                            </select>
                        </div>
                    </div>
                    <div class="p-4 bg-amber-50 rounded-xl border border-amber-200">
                        <h4 class="font-bold text-amber-800 text-sm mb-1">⚠️ Catatan ECR</h4>
                        <p class="text-xs text-amber-700">Pastikan mesin EDC bank sudah terhubung via kabel USB-to-Serial ke komputer kasir dan driver Serial Port sudah terinstal. Jika EDC belum tersambung, sistem akan kembali ke mode Manual (input Approval Code secara manual).</p>
                    </div>
                </div>
                {:else}
                <div class="p-4 bg-slate-50 rounded-xl border border-slate-200">
                    <p class="text-xs text-slate-500">Integrasi EDC ECR dinonaktifkan. Kasir akan menggunakan <strong>Mode Manual</strong> — memasukkan Approval Code dari struk EDC fisik secara manual saat memilih metode pembayaran Kartu.</p>
                </div>
                {/if}
            </div>

        </div>

        <!-- Preview Column -->
        <div class="lg:col-span-1">
            <div class="sticky top-8 bg-slate-50 rounded-2xl p-6 flex flex-col items-center justify-center border border-slate-200 shadow-inner">
                <h3 class="text-xs font-bold text-slate-400 mb-4 uppercase tracking-wider">Pratinjau Struk Kasir</h3>
                <div class="transform scale-95 origin-top shadow-xl">
                    <ReceiptPreview receiptData={testReceipt} />
                </div>
            </div>
        </div>
    </div>
</div>


<div class="p-8 max-w-5xl mx-auto">
    <div class="flex justify-between items-center mb-6">
        <div>
            <h1 class="text-3xl font-extrabold text-slate-800">Pengaturan Perangkat Keras</h1>
            <p class="text-slate-500 mt-1">Konfigurasikan printer struk dan scanner barcode untuk operasional kasir Desktop maupun Mobile.</p>
        </div>
        <div class="flex items-center gap-4">
            {#if saveMessage}
                <span class="text-sm font-semibold text-green-600 bg-green-50 px-3 py-1.5 rounded-full border border-green-200 animate-pulse">{saveMessage}</span>
            {/if}
            <button onclick={saveSettings} class="bg-blue-600 hover:bg-blue-700 text-white font-bold py-2.5 px-6 rounded-lg shadow-md transition-all">
                Simpan Konfigurasi
            </button>
        </div>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
        <!-- Pengaturan Printer & Scanner -->
        <div class="lg:col-span-2 space-y-8">
            
            <!-- Printer Panel -->
            <div class="bg-white shadow-md rounded-2xl p-6 border border-slate-200">
                <div class="flex justify-between items-center mb-6">
                    <h2 class="text-xl font-bold text-slate-800 flex items-center gap-2">
                        🖨️ Printer Struk Kasir
                    </h2>
                    {#if isConnected}
                        <span class="bg-green-100 text-green-800 text-xs font-bold px-3 py-1 rounded-full border border-green-200">Tersambung</span>
                    {:else}
                        <span class="bg-slate-100 text-slate-600 text-xs font-bold px-3 py-1 rounded-full border border-slate-200">Terputus</span>
                    {/if}
                </div>
                
                <div class="space-y-4">
                    <div>
                        <label for="printerType" class="block text-xs font-bold text-slate-400 uppercase tracking-wider mb-2">Jenis Koneksi Printer</label>
                        <select id="printerType" bind:value={printerType} class="block w-full rounded-xl border-slate-200 shadow-sm border p-3 text-slate-700 bg-slate-50 focus:bg-white focus:border-blue-500 transition-colors">
                            <option value="Mock Printer">Mock Printer (Pengujian/Console)</option>
                            <option value="ESC/POS Network">ESC/POS Network (LAN/Wi-Fi)</option>
                            <option value="ESC/POS USB">ESC/POS USB (Kabel Desktop)</option>
                            <option value="Bluetooth BLE (Mobile)">Bluetooth BLE (Mobile Android/iOS)</option>
                        </select>
                    </div>

                    <!-- Config based on Printer Type -->
                    {#if printerType === 'ESC/POS Network'}
                        <div class="p-4 bg-blue-50 rounded-xl border border-blue-200 space-y-3">
                            <h4 class="font-bold text-blue-800 text-sm">Konfigurasi Printer Jaringan (LAN/Wi-Fi)</h4>
                            <div class="grid grid-cols-2 gap-3">
                                <div>
                                    <label class="block text-[10px] font-bold text-blue-600 mb-1" for="printerIp">IP Address Printer</label>
                                    <input id="printerIp" type="text" class="w-full p-2 border rounded-lg bg-white" placeholder="192.168.1.100" />
                                </div>
                                <div>
                                    <label class="block text-[10px] font-bold text-blue-600 mb-1" for="printerPort">Port</label>
                                    <input id="printerPort" type="number" class="w-full p-2 border rounded-lg bg-white" placeholder="9100" />
                                </div>
                            </div>
                        </div>
                    {:else if printerType === 'ESC/POS USB'}
                        <div class="p-4 bg-slate-50 rounded-xl border border-slate-200">
                            <h4 class="font-bold text-slate-800 text-sm mb-2">Pilih Port USB Printer (Desktop)</h4>
                            <select class="w-full p-2 border rounded-lg bg-white">
                                <option>USB001 (Thermal Printer 58mm)</option>
                                <option>USB002 (Generic POS Printer 80mm)</option>
                            </select>
                        </div>
                    {:else if printerType === 'Bluetooth BLE (Mobile)'}
                        <div class="p-4 bg-purple-50 rounded-xl border border-purple-200 space-y-4">
                            <div class="flex justify-between items-center">
                                <h4 class="font-bold text-purple-800 text-sm">Pindai Printer Bluetooth BLE (Mobile/Tablet)</h4>
                                <button onclick={scanBle} disabled={isScanningBle} class="bg-purple-600 hover:bg-purple-700 disabled:bg-purple-300 text-white text-xs font-bold px-3 py-1.5 rounded-lg shadow-sm transition">
                                    {isScanningBle ? "Memindai..." : "Mulai Pindai"}
                                </button>
                            </div>

                            {#if isScanningBle}
                                <div class="flex items-center justify-center py-4 gap-2 text-purple-600">
                                    <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-purple-600"></div>
                                    <span class="text-xs font-bold">Mencari printer Bluetooth sekitar...</span>
                                </div>
                            {:else}
                                <div class="space-y-2">
                                    {#each bleDevices as dev}
                                        <div class="flex justify-between items-center bg-white p-3 rounded-lg border border-purple-100 shadow-xs">
                                            <div>
                                                <div class="font-bold text-slate-800 text-sm">{dev.name}</div>
                                                <div class="text-[10px] text-slate-400 font-mono">{dev.mac}</div>
                                            </div>
                                            <div>
                                                {#if dev.status === 'Tersambung'}
                                                    <span class="text-green-600 font-bold text-xs mr-2">Tersambung</span>
                                                {:else}
                                                    <button onclick={() => pairBleDevice(dev.mac)} class="bg-purple-100 hover:bg-purple-200 text-purple-700 font-bold text-xs px-2.5 py-1 rounded">Hubungkan</button>
                                                {/if}
                                            </div>
                                        </div>
                                    {/each}
                                </div>
                            {/if}
                        </div>
                    {/if}

                    <div class="pt-4 border-t border-slate-100">
                        <button 
                            onclick={testPrint} 
                            disabled={isPrinting}
                            class="bg-slate-800 hover:bg-slate-900 disabled:bg-slate-300 text-white font-bold py-2.5 px-4 rounded-xl shadow-md transition w-full text-sm">
                            {isPrinting ? "Mencetak Struk Percobaan..." : "Lakukan Uji Cetak (Test Print)"}
                        </button>
                        
                        {#if printMessage}
                            <p class="mt-3 text-xs text-center font-bold {printMessage.includes('Gagal') ? 'text-red-600' : 'text-green-600'}">
                                {printMessage}
                            </p>
                        {/if}
                    </div>
                </div>
            </div>

            <!-- Barcode Scanner Panel -->
            <div class="bg-white shadow-md rounded-2xl p-6 border border-slate-200">
                <div class="flex justify-between items-center mb-6">
                    <h2 class="text-xl font-bold text-slate-800 flex items-center gap-2">
                        📷 Pemindai Barcode (Scanner)
                    </h2>
                    {#if barcodeConnected}
                        <span class="bg-green-100 text-green-800 text-xs font-bold px-3 py-1 rounded-full border border-green-200">Aktif</span>
                    {:else}
                        <span class="bg-slate-100 text-slate-600 text-xs font-bold px-3 py-1 rounded-full border border-slate-200">Aktif (Simulasi)</span>
                    {/if}
                </div>

                <div class="space-y-4">
                    <div>
                        <label for="scannerType" class="block text-xs font-bold text-slate-400 uppercase tracking-wider mb-2">Mode Pemindaian Barcode</label>
                        <select id="scannerType" bind:value={scannerType} class="block w-full rounded-xl border-slate-200 shadow-sm border p-3 text-slate-700 bg-slate-50 focus:bg-white focus:border-blue-500 transition-colors">
                            <option value="Keyboard Wedge">Keyboard Wedge (Fisik USB / Desktop)</option>
                            <option value="Kamera Internal (Mobile)">Kamera Internal (Mobile Tablet/HP)</option>
                        </select>
                    </div>

                    {#if scannerType === 'Keyboard Wedge'}
                        <div class="p-4 bg-slate-50 rounded-xl border border-slate-200">
                            <h4 class="font-bold text-slate-800 text-sm mb-1">Keyboard Wedge Mode</h4>
                            <p class="text-xs text-slate-500">Sistem POSQ akan mendeteksi scanner USB fisik Anda secara otomatis. Scanner bertindak seperti keyboard yang mengetik kode SKU produk dengan cepat diikuti dengan tombol Enter.</p>
                        </div>
                    {:else if scannerType === 'Kamera Internal (Mobile)'}
                        <div class="p-4 bg-orange-50 rounded-xl border border-orange-200 space-y-2">
                            <h4 class="font-bold text-orange-800 text-sm">Mode Kamera Tablet / Handphone</h4>
                            <p class="text-xs text-orange-600">POSQ akan menampilkan popup viewfinder kamera web/tablet saat tombol scan ditekan di halaman kasir. Menggunakan kamera bawaan untuk memindai barcode produk secara instan.</p>
                            <div class="bg-white p-3 rounded-lg border border-orange-100 text-[10px] text-slate-500">
                                💡 <em>Memerlukan izin akses kamera pada browser atau perangkat Android/iOS tablet Anda.</em>
                            </div>
                        </div>
                    {/if}
                </div>
            </div>

            <!-- Integrasi Mesin EDC (ECR) -->
            <div class="bg-white rounded-2xl shadow-sm border border-slate-200 p-6">
                <div class="flex justify-between items-center mb-6">
                    <div>
                        <h2 class="text-xl font-bold text-slate-800 flex items-center gap-2">
                            💳 Mesin EDC (ECR Protocol)
                        </h2>
                        <p class="text-xs text-slate-500 mt-1">Integrasi pembayaran kartu otomatis ke mesin fisik EDC bank via Serial Port</p>
                    </div>
                    <label class="relative inline-flex items-center cursor-pointer">
                        <input type="checkbox" bind:checked={ecrEnabled} class="sr-only peer">
                        <div class="w-14 h-7 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-6 after:w-6 after:transition-all peer-checked:bg-blue-600"></div>
                    </label>
                </div>
                
                {#if ecrEnabled}
                <div class="grid grid-cols-1 md:grid-cols-2 gap-6 animate-in slide-in-from-top-2 duration-300">
                    <div>
                        <label for="ecr-port-name" class="block text-sm font-bold text-slate-700 mb-2">Nama Port Serial (COM/USB)</label>
                        <input id="ecr-port-name" type="text" bind:value={ecrPortName} placeholder="Misal: COM3 atau /dev/ttyUSB0" class="w-full border border-slate-200 rounded-xl px-4 py-3 text-slate-800 focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500 bg-slate-50 focus:bg-white">
                    </div>
                    <div>
                        <label for="ecr-baud-rate" class="block text-sm font-bold text-slate-700 mb-2">Baud Rate</label>
                        <select id="ecr-baud-rate" bind:value={ecrBaudRate} class="w-full border border-slate-200 rounded-xl px-4 py-3 text-slate-800 focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500 bg-slate-50 focus:bg-white">
                            <option value={9600}>9600 (BCA Standard)</option>
                            <option value={19200}>19200</option>
                            <option value={38400}>38400</option>
                            <option value={115200}>115200 (Mandiri / Modern)</option>
                        </select>
                    </div>
                </div>
                <div class="mt-4 p-4 bg-amber-50 rounded-xl border border-amber-200">
                    <h4 class="font-bold text-amber-800 text-sm mb-1">⚠️ Catatan ECR</h4>
                    <p class="text-xs text-amber-700">Pastikan mesin EDC bank sudah terhubung via kabel USB-to-Serial ke komputer kasir dan driver Serial Port sudah terinstal. Jika EDC belum tersambung, sistem akan kembali ke mode Manual (input Approval Code secara manual).</p>
                </div>
                {:else}
                <div class="p-4 bg-slate-50 rounded-xl border border-slate-200">
                    <p class="text-xs text-slate-500">Integrasi EDC ECR dinonaktifkan. Kasir akan menggunakan <strong>Mode Manual</strong> — memasukkan Approval Code dari struk EDC fisik secara manual saat memilih metode pembayaran Kartu.</p>
                </div>
                {/if}
            </div>
        </div>

        <!-- Preview Column -->
        <div class="lg:col-span-1">
            <div class="sticky top-8 bg-slate-50 rounded-2xl p-6 flex flex-col items-center justify-center border border-slate-200 shadow-inner">
                <h3 class="text-xs font-bold text-slate-400 mb-4 uppercase tracking-wider">Pratinjau Struk Kasir</h3>
                <div class="transform scale-95 origin-top shadow-xl">
                    <ReceiptPreview receiptData={testReceipt} />
                </div>
            </div>
        </div>
    </div>
</div>
