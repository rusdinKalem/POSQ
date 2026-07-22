<script lang="ts">
    interface Props {
        show: boolean;
        grandTotal: number;
        ecrEnabled: boolean;
        ecrPortName: string;
        ecrBaudRate: number;
        ecrWaiting: boolean;
        ecrStatusMessage: string;
        onSubmitPayment: (params: {
            paymentMethod: string;
            amountPaid: number;
            cashPaid: number;
            cardPaid: number;
            cardBank: string;
            cardApprovalCode: string;
            cardTraceNumber: string;
        }) => Promise<void>;
        onSendToEcr: () => void;
        onCancelEcr: () => void;
        onClose: () => void;
    }

    let {
        show,
        grandTotal,
        ecrEnabled,
        ecrPortName,
        ecrBaudRate,
        ecrWaiting,
        ecrStatusMessage,
        onSubmitPayment,
        onSendToEcr,
        onCancelEcr,
        onClose
    }: Props = $props();

    let selectedPaymentMethod: string = $state("CASH"); // "CASH" | "CARD" | "QRIS" | "SPLIT"
    let amountPaid: number = $state(0);
    let cashPaid: number = $state(0);
    let cardPaid: number = $state(0);
    let cardBank: string = $state("BCA");
    let cardApprovalCode: string = $state("");
    let cardTraceNumber: string = $state("");
    let isSubmitting: boolean = $state(false);

    $effect(() => {
        if (show) {
            amountPaid = grandTotal;
            cashPaid = Math.round(grandTotal / 2);
            cardPaid = grandTotal - cashPaid;
            selectedPaymentMethod = "CASH";
            cardApprovalCode = "";
            cardTraceNumber = "";
            isSubmitting = false;
        }
    });

    function appendToAmount(val: string) {
        let current = amountPaid.toString();
        if (current === "0" || amountPaid === grandTotal) {
            current = "";
        }
        if (val === '000' && current === "") {
            return;
        }
        current += val;
        amountPaid = parseFloat(current) || 0;
    }

    async function handleConfirm() {
        if (isSubmitting) return;
        isSubmitting = true;
        try {
            await onSubmitPayment({
                paymentMethod: selectedPaymentMethod,
                amountPaid,
                cashPaid,
                cardPaid,
                cardBank,
                cardApprovalCode,
                cardTraceNumber
            });
        } finally {
            isSubmitting = false;
        }
    }
</script>

{#if show}
<div class="modal-backdrop fixed inset-0 bg-slate-900/60 backdrop-blur-xs flex items-center justify-center z-50 p-4">
    <div class="bg-white rounded-2xl max-w-md w-full p-6 shadow-2xl border border-slate-100 flex flex-col max-h-[90vh]">
        <h2 class="text-xl font-black text-slate-800 mb-4 text-center">Metode Pembayaran</h2>
        
        <div class="bg-slate-50 rounded-xl p-4 mb-4 border border-slate-200/50 text-center shrink-0">
            <div class="text-[10px] text-slate-400 font-bold uppercase tracking-wider mb-1">Total Tagihan</div>
            <div class="text-3xl font-extrabold text-blue-600">Rp {grandTotal.toLocaleString('id-ID')}</div>
        </div>

        <!-- Payment Method Tabs -->
        <div class="grid grid-cols-4 gap-1.5 bg-slate-100 p-1.5 rounded-xl mb-4 shrink-0">
            <button 
                type="button" 
                class="py-2.5 text-xs font-bold rounded-lg transition min-h-[44px] cursor-pointer"
                class:bg-white={selectedPaymentMethod === 'CASH'}
                class:shadow-2xs={selectedPaymentMethod === 'CASH'}
                class:text-blue-600={selectedPaymentMethod === 'CASH'}
                class:text-slate-500={selectedPaymentMethod !== 'CASH'}
                onclick={() => { selectedPaymentMethod = 'CASH'; amountPaid = grandTotal; }}>
                Tunai
            </button>
            <button 
                type="button" 
                class="py-2.5 text-xs font-bold rounded-lg transition min-h-[44px] cursor-pointer"
                class:bg-white={selectedPaymentMethod === 'CARD'}
                class:shadow-2xs={selectedPaymentMethod === 'CARD'}
                class:text-blue-600={selectedPaymentMethod === 'CARD'}
                class:text-slate-500={selectedPaymentMethod !== 'CARD'}
                onclick={() => { selectedPaymentMethod = 'CARD'; amountPaid = grandTotal; }}>
                Kartu
            </button>
            <button 
                type="button" 
                class="py-2.5 text-xs font-bold rounded-lg transition min-h-[44px] cursor-pointer"
                class:bg-white={selectedPaymentMethod === 'QRIS'}
                class:shadow-2xs={selectedPaymentMethod === 'QRIS'}
                class:text-blue-600={selectedPaymentMethod === 'QRIS'}
                class:text-slate-500={selectedPaymentMethod !== 'QRIS'}
                onclick={() => { selectedPaymentMethod = 'QRIS'; amountPaid = grandTotal; }}>
                QRIS
            </button>
            <button 
                type="button" 
                class="py-2.5 text-xs font-bold rounded-lg transition min-h-[44px] cursor-pointer"
                class:bg-white={selectedPaymentMethod === 'SPLIT'}
                class:shadow-2xs={selectedPaymentMethod === 'SPLIT'}
                class:text-blue-600={selectedPaymentMethod === 'SPLIT'}
                class:text-slate-500={selectedPaymentMethod !== 'SPLIT'}
                onclick={() => { selectedPaymentMethod = 'SPLIT'; cashPaid = Math.round(grandTotal / 2); cardPaid = grandTotal - cashPaid; }}>
                Split
            </button>
        </div>

        <div class="flex-1 overflow-y-auto pr-1">
            {#if selectedPaymentMethod === 'CASH'}
                <!-- Cash Input Panel -->
                <div class="mb-4">
                    <div class="flex justify-between items-center mb-1">
                        <label class="block text-xs font-bold text-slate-400 uppercase tracking-wider" for="payAmountInput">Uang Tunai Diterima</label>
                        {#if amountPaid > grandTotal}
                            <span class="text-xs font-bold text-green-600 bg-green-50 px-2 py-0.5 rounded border border-green-200">
                                Kembalian: Rp {(amountPaid - grandTotal).toLocaleString('id-ID')}
                            </span>
                        {:else if amountPaid === grandTotal}
                            <span class="text-xs font-bold text-blue-600 bg-blue-50 px-2 py-0.5 rounded border border-blue-200">
                                Uang Pas
                            </span>
                        {/if}
                    </div>
                    <input id="payAmountInput" type="number" bind:value={amountPaid} class="w-full text-xl p-3 text-center border border-slate-200 rounded-xl font-bold text-slate-800 focus:border-blue-500 outline-none h-12" />
                </div>

                <!-- Quick Cash Buttons with min 48px touch targets -->
                <div class="grid grid-cols-3 gap-2 mb-4">
                    <button type="button" class="h-12 px-2 bg-slate-100 hover:bg-slate-200 active:bg-slate-300 text-slate-800 font-bold rounded-xl text-xs transition cursor-pointer" onclick={() => amountPaid = grandTotal}>
                        Uang Pas
                    </button>
                    <button type="button" class="h-12 px-2 bg-slate-100 hover:bg-slate-200 active:bg-slate-300 text-slate-800 font-bold rounded-xl text-xs transition cursor-pointer" onclick={() => amountPaid = 50000}>
                        Rp 50.000
                    </button>
                    <button type="button" class="h-12 px-2 bg-slate-100 hover:bg-slate-200 active:bg-slate-300 text-slate-800 font-bold rounded-xl text-xs transition cursor-pointer" onclick={() => amountPaid = 100000}>
                        Rp 100.000
                    </button>
                </div>

                <!-- Touch Numpad with min 48x48px targets -->
                <div class="grid grid-cols-3 gap-2 mb-4">
                    {#each [7, 8, 9, 4, 5, 6, 1, 2, 3] as num}
                        <button type="button" class="h-12 flex items-center justify-center bg-slate-50 hover:bg-slate-100 active:bg-slate-200 text-slate-800 font-black text-base rounded-xl border border-slate-200/60 transition cursor-pointer" onclick={() => appendToAmount(num.toString())}>
                            {num}
                        </button>
                    {/each}
                    <button type="button" class="h-12 flex items-center justify-center bg-red-50 hover:bg-red-100 text-red-600 font-black text-base rounded-xl border border-red-100 transition cursor-pointer" onclick={() => amountPaid = 0}>
                        C
                    </button>
                    <button type="button" class="h-12 flex items-center justify-center bg-slate-50 hover:bg-slate-100 active:bg-slate-200 text-slate-800 font-black text-base rounded-xl border border-slate-200/60 transition cursor-pointer" onclick={() => appendToAmount('0')}>
                        0
                    </button>
                    <button type="button" class="h-12 flex items-center justify-center bg-slate-50 hover:bg-slate-100 active:bg-slate-200 text-slate-800 font-black text-base rounded-xl border border-slate-200/60 transition cursor-pointer" onclick={() => appendToAmount('000')}>
                        000
                    </button>
                </div>
            {:else if selectedPaymentMethod === 'CARD'}
                <!-- Card Payment Panel -->
                {#if ecrEnabled}
                    <!-- ECR Automated Mode -->
                    <div class="my-4 p-4 text-center bg-blue-50/50 border border-blue-100 rounded-2xl flex flex-col items-center gap-3">
                        {#if ecrWaiting}
                            <div class="w-14 h-14 rounded-full bg-blue-100 flex items-center justify-center animate-pulse">
                                <span class="text-3xl">💳</span>
                            </div>
                            <h3 class="text-sm font-bold text-slate-800">{ecrStatusMessage || 'Mengirim ke Mesin EDC...'}</h3>
                            <p class="text-xs text-slate-500">Silakan minta pelanggan memasukkan kartu dan PIN di mesin EDC.</p>
                            <div class="w-full bg-slate-200 rounded-full h-1.5 overflow-hidden">
                                <div class="bg-blue-600 h-full rounded-full animate-pulse" style="width: 60%"></div>
                            </div>
                            <button type="button" onclick={onCancelEcr} class="text-xs text-red-500 hover:text-red-600 font-bold mt-2 cursor-pointer">
                                ✕ Batalkan & Gunakan Mode Manual
                            </button>
                        {:else}
                            <span class="text-4xl mb-1">🔌</span>
                            <h3 class="text-sm font-bold text-slate-800">Mode ECR Otomatis Aktif</h3>
                            <p class="text-xs text-slate-500 mb-2">Port: <strong>{ecrPortName}</strong> · Baud: <strong>{ecrBaudRate}</strong></p>
                            <button type="button" onclick={onSendToEcr} class="bg-blue-600 hover:bg-blue-700 text-white font-bold py-3 px-6 rounded-xl transition shadow-md text-sm cursor-pointer w-full h-12">
                                ⚡ Kirim Rp {grandTotal.toLocaleString('id-ID')} ke Mesin EDC
                            </button>
                            <p class="text-[10px] text-slate-400 mt-1">Atau isi manual di bawah jika EDC tidak merespons.</p>
                        {/if}
                    </div>
                {/if}

                <div class="my-3 p-4 text-left bg-slate-50 border border-slate-200 rounded-2xl flex flex-col gap-3">
                    <div class="flex items-center gap-3 mb-1">
                        <span class="text-2xl">💳</span>
                        <div>
                            <h3 class="text-xs font-bold text-slate-800">Detail EDC / Kartu</h3>
                            <p class="text-[10px] text-slate-400">Masukkan data dari struk mesin EDC.</p>
                        </div>
                    </div>

                    <div>
                        <label for="modal-card-bank" class="block text-[10px] font-bold text-slate-500 uppercase mb-1">Bank EDC</label>
                        <select id="modal-card-bank" bind:value={cardBank} class="w-full p-2.5 border border-slate-200 rounded-xl text-xs text-slate-800 focus:border-blue-500 outline-none bg-white h-11">
                            <option value="BCA">BCA</option>
                            <option value="Mandiri">Mandiri</option>
                            <option value="BNI">BNI</option>
                            <option value="BRI">BRI</option>
                            <option value="Lainnya">Lainnya...</option>
                        </select>
                    </div>
                    
                    <div class="grid grid-cols-2 gap-2">
                        <div>
                            <label for="modal-card-approval" class="block text-[10px] font-bold text-slate-500 uppercase mb-1">Approval Code <span class="text-red-500">*</span></label>
                            <input id="modal-card-approval" type="text" bind:value={cardApprovalCode} placeholder="Mis: 123456" class="w-full p-2.5 border border-slate-200 rounded-xl text-xs font-bold text-slate-800 focus:border-blue-500 outline-none h-11" />
                        </div>
                        <div>
                            <label for="modal-card-trace" class="block text-[10px] font-bold text-slate-500 uppercase mb-1">Trace Number</label>
                            <input id="modal-card-trace" type="text" bind:value={cardTraceNumber} placeholder="Opsional" class="w-full p-2.5 border border-slate-200 rounded-xl text-xs font-bold text-slate-800 focus:border-blue-500 outline-none h-11" />
                        </div>
                    </div>
                </div>
            {:else if selectedPaymentMethod === 'QRIS'}
                <div class="my-6 p-6 text-center bg-blue-50/50 border border-blue-100 rounded-2xl flex flex-col items-center">
                    <span class="text-5xl mb-3">📱</span>
                    <h3 class="text-sm font-bold text-slate-800 mb-1">Menunggu Pembayaran QRIS</h3>
                    <p class="text-xs text-slate-500 max-w-xs">Pastikan pembayaran QRIS pada EDC/aplikasi merchant telah dikonfirmasi berhasil oleh pelanggan.</p>
                </div>
            {:else if selectedPaymentMethod === 'SPLIT'}
                <!-- Split Payment Panel -->
                <div class="space-y-4 my-3">
                    <div>
                        <label for="split-cash-paid" class="block text-xs font-bold text-slate-500 mb-1">Nominal Tunai (Cash)</label>
                        <input id="split-cash-paid" type="number" bind:value={cashPaid} class="w-full p-3 border border-slate-200 rounded-xl text-sm font-bold text-slate-800 focus:border-blue-500 outline-none h-12" />
                    </div>
                    <div>
                        <label for="split-card-paid" class="block text-xs font-bold text-slate-500 mb-1">Nominal Non-Tunai (Kartu/QRIS)</label>
                        <input id="split-card-paid" type="number" bind:value={cardPaid} class="w-full p-3 border border-slate-200 rounded-xl text-sm font-bold text-slate-800 focus:border-blue-500 outline-none h-12" />
                    </div>
                    <div class="text-center p-3 rounded-xl text-xs font-bold" 
                         class:bg-red-50={cashPaid + cardPaid < grandTotal} 
                         class:text-red-600={cashPaid + cardPaid < grandTotal}
                         class:bg-green-50={cashPaid + cardPaid >= grandTotal}
                         class:text-green-600={cashPaid + cardPaid >= grandTotal}>
                         Total Dimasukkan: Rp {(cashPaid + cardPaid).toLocaleString('id-ID')}
                         {#if cashPaid + cardPaid < grandTotal}
                             (Kurang: Rp {(grandTotal - (cashPaid + cardPaid)).toLocaleString('id-ID')})
                         {/if}
                    </div>
                </div>
            {/if}
        </div>
        
        <div class="flex gap-3 mt-4 pt-3 border-t border-slate-100 shrink-0">
            <button 
                type="button"
                disabled={isSubmitting}
                class="bg-white border border-slate-200 text-slate-600 hover:bg-slate-50 font-bold py-2.5 px-6 rounded-xl transition w-1/2 text-xs h-12 flex items-center justify-center cursor-pointer min-w-[48px]" 
                onclick={onClose}>
                Batal
            </button>
            <button 
                type="button"
                class="bg-blue-600 hover:bg-blue-700 active:bg-blue-800 disabled:bg-slate-200 disabled:text-slate-400 text-white font-bold py-2.5 px-6 rounded-xl transition w-1/2 text-xs h-12 flex items-center justify-center cursor-pointer min-w-[48px] shadow-sm" 
                onclick={handleConfirm} 
                disabled={isSubmitting || (selectedPaymentMethod === 'SPLIT' ? (cashPaid + cardPaid < grandTotal) : (amountPaid < grandTotal))}>
                {#if isSubmitting}
                    <span class="inline-flex items-center gap-1.5">
                        <svg class="animate-spin h-4 w-4 text-white" fill="none" viewBox="0 0 24 24">
                            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                        </svg>
                        Memproses...
                    </span>
                {:else}
                    Konfirmasi Pembayaran
                {/if}
            </button>
        </div>
    </div>
</div>
{/if}
