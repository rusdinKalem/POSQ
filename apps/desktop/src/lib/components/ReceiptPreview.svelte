<script lang="ts">
    export let receiptData: any;
    
    function formatCurrency(amount: number): string {
        return new Intl.NumberFormat('id-ID', {
            style: 'currency',
            currency: 'IDR',
            minimumFractionDigits: 0
        }).format(amount);
    }
</script>

<div class="receipt-container bg-white text-black p-4 font-mono text-sm max-w-[320px] mx-auto border shadow-sm">
    <div class="text-center mb-4 border-b border-dashed border-gray-400 pb-4">
        <h2 class="font-bold text-lg mb-1">{receiptData.store_name}</h2>
        <p class="text-xs whitespace-pre-wrap">{receiptData.store_address}</p>
    </div>
    
    <div class="flex justify-between text-xs mb-1">
        <span>No:</span>
        <span>{receiptData.receipt_no}</span>
    </div>
    <div class="flex justify-between text-xs mb-1">
        <span>Tgl:</span>
        <span>{receiptData.date}</span>
    </div>
    <div class="flex justify-between text-xs mb-4 border-b border-dashed border-gray-400 pb-2">
        <span>Kasir:</span>
        <span>{receiptData.cashier}</span>
    </div>
    
    <div class="mb-4 border-b border-dashed border-gray-400 pb-2">
        {#each receiptData.items as item}
            <div class="mb-2">
                <div class="font-bold">{item.name}</div>
                <div class="flex justify-between text-xs">
                    <span>{item.qty} x {formatCurrency(item.price)}</span>
                    <span>{formatCurrency(item.subtotal)}</span>
                </div>
            </div>
        {/each}
    </div>
    
    <div class="mb-4 border-b border-dashed border-gray-400 pb-2">
        <div class="flex justify-between text-xs mb-1">
            <span>Subtotal</span>
            <span>{formatCurrency(receiptData.subtotal)}</span>
        </div>
        <div class="flex justify-between text-xs mb-1">
            <span>Pajak (Tax)</span>
            <span>{formatCurrency(receiptData.tax)}</span>
        </div>
        <div class="flex justify-between font-bold text-sm mt-2">
            <span>TOTAL</span>
            <span>{formatCurrency(receiptData.total)}</span>
        </div>
    </div>
    
    <div class="mb-6">
        <div class="flex justify-between text-xs mb-1">
            <span>Metode Bayar</span>
            <span class="uppercase">{receiptData.payment_method}</span>
        </div>
        <div class="flex justify-between text-xs mb-1">
            <span>Bayar (Tunai)</span>
            <span>{formatCurrency(receiptData.amount_paid)}</span>
        </div>
        <div class="flex justify-between text-xs mb-1 font-bold">
            <span>Kembali</span>
            <span>{formatCurrency(receiptData.change)}</span>
        </div>
    </div>
    
    <div class="text-center text-xs">
        <p>*** TERIMA KASIH ***</p>
        <p class="mt-1">Powered by POSQ</p>
    </div>
</div>

<style>
    .receipt-container {
        /* Imitate thermal paper look */
        background-color: #fdfdfd;
        color: #111;
        border-radius: 2px;
    }
</style>
