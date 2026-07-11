<script lang="ts">
    import { onMount } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';

    type SalesSummary = {
        total_orders: number;
        subtotal: number;
        discount_total: number;
        tax_total: number;
        grand_total: number;
    };

    type PaymentBreakdown = {
        payment_method: string;
        amount: number;
    };

    type ProductRanking = {
        product_id: string;
        name: string;
        total_qty: number;
        total_revenue: number;
    };

    let summary: SalesSummary | null = null;
    let paymentData: PaymentBreakdown[] = [];
    let productData: ProductRanking[] = [];
    
    // Default: Today
    let today = new Date();
    let defaultStart = new Date(today.getFullYear(), today.getMonth(), today.getDate()).toISOString();
    let defaultEnd = new Date(today.getFullYear(), today.getMonth(), today.getDate(), 23, 59, 59).toISOString();
    
    // Bounded simple input string (YYYY-MM-DD format for input types)
    let startDateStr = today.toISOString().split('T')[0];
    let endDateStr = today.toISOString().split('T')[0];
    
    let errorMsg = '';

    onMount(async () => {
        await loadReports();
    });

    async function loadReports() {
        errorMsg = '';
        try {
            let start = new Date(startDateStr + 'T00:00:00Z').toISOString();
            let end = new Date(endDateStr + 'T23:59:59Z').toISOString();
            
            summary = await invoke('get_sales_summary', { startDate: start, endDate: end });
            paymentData = await invoke('get_payment_breakdown', { startDate: start, endDate: end });
            productData = await invoke('get_product_ranking', { startDate: start, endDate: end });
        } catch (e: any) {
            errorMsg = e.toString();
        }
    }

    async function exportCSV() {
        try {
            let start = new Date(startDateStr + 'T00:00:00Z').toISOString();
            let end = new Date(endDateStr + 'T23:59:59Z').toISOString();
            
            let path: string = await invoke('export_sales_csv', { startDate: start, endDate: end });
            alert('Berhasil mengekspor Laporan ke CSV!\nLokasi: ' + path);
        } catch (e: any) {
            alert('Gagal mengekspor laporan: ' + e);
        }
    }
</script>

<div class="p-8 min-h-screen bg-gray-100 flex flex-col">
    <div class="flex justify-between items-center mb-6">
        <h1 class="text-2xl font-bold">Laporan Omset</h1>
        <div class="flex gap-4">
            <a href="/pos" class="btn-outline px-4 py-2" style="text-decoration: none;">Kembali</a>
            <button class="btn-primary px-4 py-2 font-bold" on:click={exportCSV}>Export CSV</button>
        </div>
    </div>

    <!-- Filters -->
    <div class="card glassmorphism p-4 mb-6 flex gap-4 items-end">
        <div>
            <label class="block font-bold text-sm mb-1" for="start_date">Mulai Tanggal</label>
            <input id="start_date" type="date" bind:value={startDateStr} class="p-2 border rounded" />
        </div>
        <div>
            <label class="block font-bold text-sm mb-1" for="end_date">Sampai Tanggal</label>
            <input id="end_date" type="date" bind:value={endDateStr} class="p-2 border rounded" />
        </div>
        <button class="bg-blue-600 text-white px-6 py-2 rounded font-bold hover:bg-blue-700 transition" on:click={loadReports}>Terapkan</button>
    </div>

    {#if errorMsg}
        <div class="alert alert-danger mb-4 p-4 rounded bg-red-100 text-red-800 font-bold border border-red-300">
            {errorMsg}
        </div>
    {:else if summary}
        <!-- KPI Cards -->
        <div class="grid grid-cols-1 md:grid-cols-4 gap-6 mb-8">
            <div class="card glassmorphism p-6 text-center">
                <div class="text-gray-500 font-bold mb-2">Total Transaksi</div>
                <div class="text-3xl font-bold text-blue-600">{summary.total_orders}</div>
            </div>
            <div class="card glassmorphism p-6 text-center">
                <div class="text-gray-500 font-bold mb-2">Subtotal Penjualan</div>
                <div class="text-2xl font-bold">Rp {summary.subtotal}</div>
            </div>
            <div class="card glassmorphism p-6 text-center">
                <div class="text-gray-500 font-bold mb-2">Pajak / Diskon</div>
                <div class="text-xl font-bold text-red-500">+ {summary.tax_total} / - {summary.discount_total}</div>
            </div>
            <div class="card glassmorphism p-6 text-center border-2 border-primary">
                <div class="text-primary font-bold mb-2">Pendapatan Bersih</div>
                <div class="text-3xl font-bold text-green-600">Rp {summary.grand_total}</div>
            </div>
        </div>

        <div class="grid grid-cols-1 lg:grid-cols-3 gap-6 flex-1">
            
            <!-- Payment Breakdown -->
            <div class="card glassmorphism p-0 flex flex-col">
                <h2 class="p-4 border-b font-bold bg-gray-50">Metode Pembayaran</h2>
                <div class="overflow-y-auto flex-1 p-4">
                    {#each paymentData as pay}
                        <div class="flex justify-between items-center mb-3 p-3 bg-gray-50 rounded">
                            <span class="font-bold">{pay.payment_method}</span>
                            <span class="text-lg">Rp {pay.amount}</span>
                        </div>
                    {/each}
                    {#if paymentData.length === 0}
                        <div class="text-gray-500 text-center mt-4">Belum ada data pembayaran</div>
                    {/if}
                </div>
            </div>

            <!-- Product Ranking -->
            <div class="card glassmorphism p-0 lg:col-span-2 flex flex-col">
                <h2 class="p-4 border-b font-bold bg-gray-50">Ranking Produk (Terlaris)</h2>
                <div class="overflow-y-auto flex-1">
                    <table class="w-full text-left border-collapse">
                        <thead class="bg-gray-100">
                            <tr>
                                <th class="p-3 font-bold text-gray-700">Nama Produk</th>
                                <th class="p-3 font-bold text-gray-700 text-right">Terjual (Qty)</th>
                                <th class="p-3 font-bold text-gray-700 text-right">Omset (Rp)</th>
                            </tr>
                        </thead>
                        <tbody>
                            {#each productData as prod}
                                <tr class="border-b hover:bg-gray-50">
                                    <td class="p-3 font-bold">{prod.name}</td>
                                    <td class="p-3 text-right text-blue-600 font-bold">{prod.total_qty}</td>
                                    <td class="p-3 text-right text-green-600">Rp {prod.total_revenue}</td>
                                </tr>
                            {/each}
                            {#if productData.length === 0}
                                <tr>
                                    <td colspan="3" class="p-8 text-center text-gray-500">
                                        Tidak ada penjualan pada periode ini.
                                    </td>
                                </tr>
                            {/if}
                        </tbody>
                    </table>
                </div>
            </div>
            
        </div>
    {/if}
</div>
