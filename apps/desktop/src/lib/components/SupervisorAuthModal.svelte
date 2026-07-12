<script lang="ts">
    import { invoke } from '@tauri-apps/api/core';
    import { showToast } from '$lib/toast.svelte';

    let { 
        show = false,
        actionType = 'transaction.void',
        amount = 0,
        resourceId = null,
        onSuccess,
        onCancel
    } = $props<{
        show: boolean;
        actionType: string;
        amount?: number;
        resourceId?: string | null;
        onSuccess: (grantId: string, supervisorName: string) => void;
        onCancel: () => void;
    }>();

    let pin = $state('');
    let reasonCode = $state('');
    let notes = $state('');
    let error = $state('');
    let loading = $state(false);

    // Reason Codes options mapping
    const reasonOptions = $derived(() => {
        if (actionType.startsWith('transaction')) {
            return [
                { value: 'WRONG_ITEM', label: 'Salah Input Menu/Item' },
                { value: 'WRONG_QUANTITY', label: 'Salah Jumlah/Qty' },
                { value: 'DUPLICATE_ENTRY', label: 'Input Ganda/Double' },
                { value: 'CUSTOMER_CANCELLED', label: 'Pelanggan Batal Memesan' },
                { value: 'PRODUCT_UNAVAILABLE', label: 'Bahan/Stok Habis' },
                { value: 'KITCHEN_ERROR', label: 'Kesalahan Dapur' },
                { value: 'CASHIER_ERROR', label: 'Kesalahan Kasir' },
                { value: 'OTHER', label: 'Lainnya (Tulis di Catatan)' }
            ];
        } else if (actionType.startsWith('cash')) {
            return [
                { value: 'ICE_PURCHASE', label: 'Pembelian Es Batu' },
                { value: 'PARKING', label: 'Parkir Kurir/Suplier' },
                { value: 'DELIVERY', label: 'Ongkos Kirim/Kurir' },
                { value: 'GAS', label: 'Pembelian Gas LPG' },
                { value: 'CLEANING_SUPPLIES', label: 'Alat/Bahan Kebersihan' },
                { value: 'EMERGENCY_PURCHASE', label: 'Kebutuhan Darurat/Operasional' },
                { value: 'TRANSPORT', label: 'Transportasi Karyawan/Driver' },
                { value: 'OTHER', label: 'Lainnya (Tulis di Catatan)' }
            ];
        } else {
            return [
                { value: 'CUSTOMER_REQUEST', label: 'Permintaan Pelanggan' },
                { value: 'PRICE_DISPUTE', label: 'Perselisihan Harga' },
                { value: 'OTHER', label: 'Lainnya (Tulis di Catatan)' }
            ];
        }
    });

    // Default the reasonCode to first item
    $effect(() => {
        const opts = reasonOptions();
        if (opts.length > 0 && !reasonCode) {
            reasonCode = opts[0].value;
        }
    });

    function appendDigit(digit: string) {
        if (pin.length < 6) {
            pin += digit;
            error = '';
        }
    }

    function backspace() {
        pin = pin.slice(0, -1);
    }

    function clearPin() {
        pin = '';
    }

    async function handleVerify() {
        if (pin.length < 6) {
            error = 'PIN harus 6 digit angka';
            return;
        }
        if (reasonCode === 'OTHER' && !notes.trim()) {
            error = 'Catatan wajib diisi jika alasan adalah Lainnya';
            return;
        }

        loading = true;
        error = '';

        try {
            const res = await invoke<{ grant_id: string; supervisor_name: string }>('verify_supervisor_pin', {
                req: {
                    pin,
                    action_type: actionType,
                    amount,
                    reason_code: reasonCode + (notes.trim() ? `: ${notes.trim()}` : ''),
                    resource_id: resourceId
                }
            });
            showToast('Otorisasi berhasil diberikan oleh ' + res.supervisor_name, 'success');
            onSuccess(res.grant_id, res.supervisor_name);
            // Reset
            pin = '';
            notes = '';
        } catch (e: any) {
            error = e.toString();
            pin = ''; // Reset pin on failure for security
        } finally {
            loading = false;
        }
    }

    // Keyboard handlers
    function handleKeyDown(event: KeyboardEvent) {
        if (!show) return;
        if (event.key >= '0' && event.key <= '9') {
            appendDigit(event.key);
        } else if (event.key === 'Backspace') {
            backspace();
        } else if (event.key === 'Enter') {
            handleVerify();
        } else if (event.key === 'Escape') {
            onCancel();
        }
    }
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if show}
<div class="fixed inset-0 z-[8888] flex items-center justify-center bg-slate-950/75 backdrop-blur-sm animate-in fade-in duration-200">
    <div class="w-full max-w-md overflow-hidden rounded-3xl border border-slate-800 bg-slate-900 shadow-2xl p-6 flex flex-col gap-6 scale-in-95 duration-200">
        <!-- Title & Action info -->
        <div class="flex flex-col gap-1 border-b border-slate-800 pb-4">
            <h2 class="text-xl font-black text-white tracking-wide flex items-center gap-2">
                🔒 Otorisasi Supervisor
            </h2>
            <p class="text-xs text-slate-400">
                Tindakan sensitif terdeteksi. Silakan minta supervisor memasukkan PIN otorisasi.
            </p>
            <div class="mt-3 flex flex-wrap gap-2 text-xs">
                <span class="rounded-lg bg-red-500/10 border border-red-500/20 px-2 py-1 text-red-400 font-medium">
                    Tindakan: {actionType}
                </span>
                {#if amount > 0}
                <span class="rounded-lg bg-amber-500/10 border border-amber-500/20 px-2 py-1 text-amber-400 font-medium">
                    Nominal: Rp {amount.toLocaleString('id-ID')}
                </span>
                {/if}
            </div>
        </div>

        <!-- Inputs Form -->
        <div class="flex flex-col gap-4">
            <!-- Reason dropdown -->
            <div class="flex flex-col gap-1.5">
                <label class="text-xs font-bold text-slate-400 uppercase tracking-wider">Alasan Otorisasi</label>
                <select 
                    bind:value={reasonCode}
                    class="w-full rounded-xl border border-slate-800 bg-slate-950 px-3 py-2.5 text-sm text-white focus:outline-none focus:ring-2 focus:ring-indigo-500"
                >
                    {#each reasonOptions() as opt}
                    <option value={opt.value}>{opt.label}</option>
                    {/each}
                </select>
            </div>

            <!-- Notes field -->
            <div class="flex flex-col gap-1.5">
                <label class="text-xs font-bold text-slate-400 uppercase tracking-wider">Catatan Tambahan (Wajib untuk Lainnya)</label>
                <textarea 
                    bind:value={notes}
                    placeholder="Masukkan alasan detail..."
                    rows="2"
                    class="w-full rounded-xl border border-slate-800 bg-slate-950 px-3 py-2 text-sm text-white focus:outline-none focus:ring-2 focus:ring-indigo-500 resize-none"
                ></textarea>
            </div>
        </div>

        <!-- PIN Dots indicator -->
        <div class="flex flex-col items-center gap-3">
            <label class="text-xs font-bold text-slate-400 uppercase tracking-wider">PIN Supervisor</label>
            <div class="flex gap-4">
                {#each Array(6) as _, i}
                <div class="w-4 h-4 rounded-full border-2 transition-all duration-150
                    {i < pin.length ? 'bg-indigo-500 border-indigo-400 scale-110 shadow-lg shadow-indigo-500/50' : 'border-slate-700 bg-slate-950'}">
                </div>
                {/each}
            </div>
        </div>

        <!-- On-Screen Numeric Keypad (Crucial for POS touchscreen) -->
        <div class="grid grid-cols-3 gap-2.5 max-w-[280px] mx-auto w-full">
            {#each ['1', '2', '3', '4', '5', '6', '7', '8', '9'] as num}
            <button 
                type="button" 
                onclick={() => appendDigit(num)}
                class="h-12 text-lg font-bold text-white rounded-xl bg-slate-800 border border-slate-700 hover:bg-slate-700 active:bg-slate-600 transition-colors"
            >
                {num}
            </button>
            {/each}
            <button 
                type="button" 
                onclick={clearPin}
                class="h-12 text-xs font-bold text-red-400 rounded-xl bg-slate-800/50 border border-slate-700/50 hover:bg-red-500/10 transition-colors"
            >
                CLEAR
            </button>
            <button 
                type="button" 
                onclick={() => appendDigit('0')}
                class="h-12 text-lg font-bold text-white rounded-xl bg-slate-800 border border-slate-700 hover:bg-slate-700 active:bg-slate-600 transition-colors"
            >
                0
            </button>
            <button 
                type="button" 
                onclick={backspace}
                class="h-12 text-lg font-bold text-slate-400 rounded-xl bg-slate-800/50 border border-slate-700/50 hover:bg-slate-700 transition-colors"
            >
                ⌫
            </button>
        </div>

        <!-- Error message -->
        {#if error}
        <div class="rounded-xl border border-red-500/20 bg-red-500/10 p-3 text-center text-xs font-semibold text-red-400">
            ❌ {error}
        </div>
        {/if}

        <!-- Actions -->
        <div class="flex gap-3 border-t border-slate-800 pt-4 mt-2">
            <button 
                type="button" 
                onclick={onCancel}
                class="flex-1 rounded-xl border border-slate-700 bg-transparent py-2.5 text-sm font-bold text-slate-300 hover:bg-slate-800 hover:text-white transition-colors"
            >
                Batal
            </button>
            <button 
                type="button" 
                onclick={handleVerify}
                disabled={loading}
                class="flex-1 rounded-xl bg-indigo-600 hover:bg-indigo-500 py-2.5 text-sm font-bold text-white disabled:opacity-50 transition-colors flex items-center justify-center gap-2"
            >
                {#if loading}
                <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
                {:else}
                Verifikasi
                {/if}
            </button>
        </div>
    </div>
</div>
{/if}
