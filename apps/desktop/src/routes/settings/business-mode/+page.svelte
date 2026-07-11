<script lang="ts">
    import { invoke } from '@tauri-apps/api/core';

    let currentMode = "retail"; // "retail" or "fb"
    
    // Test states
    let testActionStatus = "";
    let isProcessing = false;

    // F&B Table mock
    let tables: any[] = [];
    
    async function loadTables() {
        if (tables.length === 0) {
            try {
                tables = await invoke('get_tables');
            } catch (err) {
                console.error(err);
            }
        }
    }

    $: if (currentMode === 'fb') {
        loadTables();
    }

    async function testRetailReturn() {
        isProcessing = true;
        testActionStatus = "Memproses retur...";
        try {
            const res: any = await invoke('process_return', { 
                orderId: "TRX-12345", 
                reason: "Barang cacat", 
                refundAmount: 50000 
            });
            testActionStatus = "SUKSES: " + res.message;
        } catch (err) {
            testActionStatus = "GAGAL: " + err;
        } finally {
            isProcessing = false;
        }
    }

    async function testKitchenPrint() {
        isProcessing = true;
        testActionStatus = "Mencetak ke Dapur...";
        try {
            const data = {
                table_no: "Meja 4",
                order_no: "KOT-999",
                time: new Date().toLocaleTimeString(),
                items: [
                    { name: "Nasi Goreng Spesial", qty: 2, notes: "Pedas, jangan pakai acar" },
                    { name: "Es Teh Manis", qty: 2, notes: null }
                ]
            };
            await invoke('print_kitchen_ticket', { data });
            testActionStatus = "SUKSES: Tiket Dapur berhasil dicetak (Lihat konsol)";
        } catch (err) {
            testActionStatus = "GAGAL: " + err;
        } finally {
            isProcessing = false;
        }
    }
</script>

<div class="p-8 max-w-4xl mx-auto">
    <h1 class="text-3xl font-bold mb-6">Mode Bisnis</h1>
    <p class="text-gray-600 mb-8">Ubah perilaku dan tata letak aplikasi kasir Anda sesuai dengan spesifikasi bisnis. Pilihan ini akan mengaktifkan modul khusus seperti Pengaturan Meja (F&B) atau Retur Penjualan (Retail).</p>

    <!-- Mode Toggle -->
    <div class="bg-white shadow rounded-lg p-6 mb-8 border border-gray-200">
        <h2 class="text-xl font-bold mb-4">Pilih Tipe Toko Anda</h2>
        
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <!-- Retail Card -->
            <!-- svelte-ignore a11y-click-events-have-key-events -->
            <!-- svelte-ignore a11y-no-static-element-interactions -->
            <div class="border-2 rounded-lg p-6 cursor-pointer transition-all {currentMode === 'retail' ? 'border-blue-500 bg-blue-50' : 'border-gray-200 hover:border-blue-300'}"
                 on:click={() => currentMode = 'retail'}>
                <div class="flex items-center justify-between mb-2">
                    <h3 class="font-bold text-lg text-blue-900">🛍️ Toko Retail</h3>
                    {#if currentMode === 'retail'}
                        <span class="bg-blue-500 text-white text-xs px-2 py-1 rounded-full">Aktif</span>
                    {/if}
                </div>
                <p class="text-sm text-gray-600">Cocok untuk toko kelontong, minimarket, butik, dan elektronik. Mendukung manajemen stok ketat dan retur barang.</p>
            </div>

            <!-- F&B Card -->
            <!-- svelte-ignore a11y-click-events-have-key-events -->
            <!-- svelte-ignore a11y-no-static-element-interactions -->
            <div class="border-2 rounded-lg p-6 cursor-pointer transition-all {currentMode === 'fb' ? 'border-orange-500 bg-orange-50' : 'border-gray-200 hover:border-orange-300'}"
                 on:click={() => currentMode = 'fb'}>
                <div class="flex items-center justify-between mb-2">
                    <h3 class="font-bold text-lg text-orange-900">🍔 Food & Beverage</h3>
                    {#if currentMode === 'fb'}
                        <span class="bg-orange-500 text-white text-xs px-2 py-1 rounded-full">Aktif</span>
                    {/if}
                </div>
                <p class="text-sm text-gray-600">Sempurna untuk kafe, restoran, dan kedai. Mendukung pesanan per meja, varian rasa, dan cetak tiket dapur (Kitchen Print).</p>
            </div>
        </div>
    </div>

    <!-- Playground Area -->
    <div class="bg-white shadow rounded-lg p-6 border border-gray-200">
        <h2 class="text-xl font-bold mb-4 border-b pb-2">Arena Simulasi ({currentMode === 'retail' ? 'Retail' : 'F&B'})</h2>
        
        {#if currentMode === 'retail'}
            <div class="mb-4">
                <p class="text-sm text-gray-600 mb-4">Dalam mode Retail, fitur <strong>Retur Transaksi</strong> tersedia untuk pelanggan yang mengembalikan barang cacat.</p>
                <button 
                    on:click={testRetailReturn}
                    disabled={isProcessing}
                    class="bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-6 rounded shadow transition">
                    Simulasikan Retur (Process Return)
                </button>
            </div>
        {:else}
            <div class="mb-4">
                <p class="text-sm text-gray-600 mb-4">Dalam mode F&B, sistem secara bawaan melacak nomor meja dan menyortir struk khusus untuk dapur.</p>
                
                <div class="flex gap-2 mb-6 flex-wrap">
                    {#each tables.slice(0, 5) as table}
                        <div class="px-4 py-2 text-sm rounded border {table.is_occupied ? 'bg-red-100 border-red-300 text-red-800' : 'bg-green-100 border-green-300 text-green-800'}">
                            {table.name}
                        </div>
                    {/each}
                    <div class="px-4 py-2 text-sm text-gray-500 italic">... dan {tables.length - 5} meja lainnya</div>
                </div>

                <button 
                    on:click={testKitchenPrint}
                    disabled={isProcessing}
                    class="bg-orange-600 hover:bg-orange-700 text-white font-bold py-2 px-6 rounded shadow transition">
                    Cetak Tiket Dapur Mock (Kitchen Print)
                </button>
            </div>
        {/if}

        {#if testActionStatus}
            <div class="mt-6 p-4 rounded text-sm font-mono {testActionStatus.startsWith('GAGAL') ? 'bg-red-100 text-red-800' : 'bg-green-100 text-green-800'}">
                {testActionStatus}
            </div>
        {/if}
    </div>
</div>
