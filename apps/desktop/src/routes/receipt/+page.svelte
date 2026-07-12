<script lang="ts">
    import { page } from '$app/stores';
    import { onMount } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';

    let orderId = $page.url.searchParams.get('order_id');
    let receipt: any = null;

    let businessMode = "";

    onMount(async () => {
        businessMode = localStorage.getItem('businessMode') || 'retail';
        if (orderId) {
            try {
                receipt = await invoke('get_receipt', { orderId });
            } catch (e) {
                console.error("Failed to fetch receipt:", e);
            }
        }
    });

    function printReceipt() {
        window.print();
    }
</script>

<div class="receipt-wrapper flex items-center justify-center p-4">
    <div class="receipt-card card text-center w-full" style="max-width: 400px;">
        <h1 class="text-2xl font-bold mb-2">POSQ</h1>
        <p class="text-gray mb-4">Demo Merchant - Cabang Utama</p>
        
        <div class="text-left mb-4">
            <div><strong>Order ID:</strong> {orderId}</div>
            {#if receipt}
                <div><strong>No:</strong> {receipt.order_number}</div>
                <div><strong>Tgl:</strong> {receipt.created_at}</div>
                {#if businessMode === 'fb'}
                    <div><strong>Tipe:</strong> {receipt.order_type === 'dine_in' ? 'Dine In' : receipt.order_type === 'takeaway' ? 'Take Away' : 'Delivery'}</div>
                    {#if receipt.table_number}
                        <div><strong>Meja:</strong> {receipt.table_number}</div>
                    {/if}
                {:else if businessMode === 'jasa'}
                    {#if receipt.table_number}
                        <div><strong>Teknisi:</strong> {receipt.table_number}</div>
                    {/if}
                {/if}
            {/if}
        </div>

        <div class="divider mb-4"></div>

        {#if receipt}
            <div class="items-list text-left mb-4">
                {#each receipt.items as item}
                    <div class="flex justify-between mb-1">
                        <div>
                            <div>{item.name} x {item.qty}</div>
                            {#if item.notes}
                                <div class="text-[10px] text-gray-500 italic ml-2 mt-0.5">{item.notes}</div>
                            {/if}
                        </div>
                        <div>Rp {item.line_total}</div>
                    </div>
                {/each}
            </div>

            <div class="divider mb-4"></div>

            <div class="totals text-left">
                <div class="flex justify-between mb-1">
                    <span>Subtotal</span>
                    <span>Rp {receipt.subtotal}</span>
                </div>
                <div class="flex justify-between mb-1">
                    <span>Tax</span>
                    <span>Rp {receipt.tax_total}</span>
                </div>
                <div class="flex justify-between mb-1 font-bold text-lg mt-2">
                    <span>Total</span>
                    <span>Rp {receipt.grand_total}</span>
                </div>
                <div class="flex justify-between mb-1 mt-2">
                    <span>Cash</span>
                    <span>Rp {receipt.paid_total}</span>
                </div>
                <div class="flex justify-between mb-1">
                    <span>Kembali</span>
                    <span>Rp {receipt.change_total}</span>
                </div>
            </div>
        {:else}
            <div class="p-4 text-gray">Loading receipt details...</div>
        {/if}

        <div class="divider mt-4 mb-4"></div>
        <p class="text-sm font-bold">Terima Kasih!</p>

        <div class="mt-4 flex gap-2">
            <button class="btn-primary w-full p-2" on:click={printReceipt}>Print</button>
            <a href="/pos" class="btn-outline w-full p-2 text-center" style="text-decoration: none; display: inline-block;">Kembali</a>
        </div>
    </div>
</div>

<style>
    .receipt-wrapper {
        min-height: 100vh;
        background-color: var(--bg-color);
    }
    .divider {
        border-top: 1px dashed var(--border-color);
    }
</style>
