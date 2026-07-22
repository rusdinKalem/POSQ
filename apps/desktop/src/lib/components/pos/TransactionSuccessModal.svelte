<script lang="ts">
    interface Props {
        show: boolean;
        orderId: string | null;
        grandTotal: number;
        paidTotal: number;
        changeTotal: number;
        paymentMethod: string;
        onPrintReceipt: () => void;
        onNewTransaction: () => void;
    }

    let {
        show,
        orderId,
        grandTotal,
        paidTotal,
        changeTotal,
        paymentMethod,
        onPrintReceipt,
        onNewTransaction
    }: Props = $props();
</script>

{#if show}
<div class="modal-backdrop fixed inset-0 bg-slate-900/60 backdrop-blur-xs flex items-center justify-center z-50 p-4 animate-in fade-in duration-200">
    <div class="bg-white rounded-2xl max-w-sm w-full p-6 shadow-2xl border border-slate-100 text-center flex flex-col items-center">
        <!-- Success Icon -->
        <div class="w-16 h-16 bg-green-100 text-green-600 rounded-full flex items-center justify-center mb-3 text-3xl shadow-xs">
            ✓
        </div>

        <h2 class="text-xl font-black text-slate-800 mb-1">Pembayaran Berhasil!</h2>
        <div class="inline-flex items-center gap-1.5 px-3 py-1 bg-slate-100 border border-slate-200 rounded-full text-[11px] font-bold text-slate-600 mb-4">
            <span>💾</span>
            <span>Tersimpan di Perangkat (Offline-First)</span>
        </div>

        <!-- Transaction Details Box -->
        <div class="w-full bg-slate-50 border border-slate-200/60 rounded-xl p-4 space-y-2 mb-6 text-left">
            <div class="flex justify-between items-center text-xs">
                <span class="text-slate-400 font-bold">Metode Bayar</span>
                <span class="font-bold text-slate-800 uppercase">{paymentMethod}</span>
            </div>
            <div class="flex justify-between items-center text-xs">
                <span class="text-slate-400 font-bold">Total Tagihan</span>
                <span class="font-bold text-slate-800">Rp {grandTotal.toLocaleString('id-ID')}</span>
            </div>
            <div class="flex justify-between items-center text-xs">
                <span class="text-slate-400 font-bold">Total Diterima</span>
                <span class="font-bold text-slate-800">Rp {paidTotal.toLocaleString('id-ID')}</span>
            </div>
            <div class="flex justify-between items-center text-xs pt-2 border-t border-slate-200/60 font-black">
                <span class="text-slate-600">Kembalian</span>
                <span class="text-green-600 text-sm">Rp {changeTotal.toLocaleString('id-ID')}</span>
            </div>
        </div>

        <!-- Actions -->
        <div class="w-full space-y-2.5">
            <button 
                type="button" 
                onclick={onPrintReceipt}
                class="w-full bg-slate-100 hover:bg-slate-200 active:bg-slate-300 text-slate-800 font-bold py-3 rounded-xl transition text-xs flex items-center justify-center gap-2 cursor-pointer h-12 min-w-[48px] border border-slate-200/60">
                🖨️ Cetak / Preview Struk
            </button>
            <button 
                type="button" 
                onclick={onNewTransaction}
                class="w-full bg-blue-600 hover:bg-blue-700 active:bg-blue-800 text-white font-extrabold py-3 rounded-xl transition text-xs flex items-center justify-center gap-2 shadow-sm cursor-pointer h-12 min-w-[48px]">
                ➕ Transaksi Baru
            </button>
        </div>
    </div>
</div>
{/if}
