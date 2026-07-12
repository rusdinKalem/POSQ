<script lang="ts">
    import { onMount } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';
    import { showToast } from '$lib/toast.svelte';
    import SupervisorAuthModal from '$lib/components/SupervisorAuthModal.svelte';

    let shiftActive = $state(false);
    let shiftId = $state<string | null>(null);
    let startingCash = $state(0);
    let startedAt = $state('');

    // Denominations list
    let denomCounts = $state([
        { denom: 100000, label: 'Rp 100.000', qty: 0 },
        { denom: 50000,  label: 'Rp 50.000',  qty: 0 },
        { denom: 20000,  label: 'Rp 20.000',  qty: 0 },
        { denom: 10000,  label: 'Rp 10.000',  qty: 0 },
        { denom: 5000,   label: 'Rp 5.000',   qty: 0 },
        { denom: 2000,   label: 'Rp 2.000',   qty: 0 },
        { denom: 1000,   label: 'Rp 1.000',   qty: 0 },
        { denom: 500,    label: 'Rp 500',      qty: 0 },
        { denom: 200,    label: 'Rp 200',      qty: 0 },
        { denom: 100,    label: 'Rp 100',      qty: 0 },
    ]);

    // Derived total counted cash
    const totalCounted = $derived(() => {
        return denomCounts.reduce((sum, item) => sum + (item.denom * item.qty), 0);
    });

    // Verification / variance report state
    let report = $state<{
        attempt_number: number;
        expected_cash: number;
        counted_cash: number;
        variance: number;
        recount_required: boolean;
        supervisor_required: boolean;
    } | null>(null);

    // Supervisor Modal visibility
    let showAuthModal = $state(false);

    onMount(async () => {
        await loadActiveShift();
    });

    async function loadActiveShift() {
        try {
            const status: any = await invoke('check_active_shift');
            if (status.active) {
                shiftActive = true;
                shiftId = status.shift_id;
                startingCash = status.starting_cash;
                startedAt = status.started_at || '';
            } else {
                shiftActive = false;
                shiftId = null;
                startingCash = 0;
            }
        } catch (e) {
            console.error('Error checking active shift:', e);
        }
    }

    async function handleSubmitCount() {
        if (!shiftId) return;

        const denoms = denomCounts
            .filter(d => d.qty > 0)
            .map(d => ({ denomination: d.denom, quantity: d.qty }));

        try {
            const res = await invoke<any>('submit_blind_cash_count', {
                shiftId,
                countedCash: totalCounted(),
                denominations: denoms
            });
            report = res;

            if (res.recount_required) {
                showToast(`Selisih terdeteksi (Percobaan #${res.attempt_number}). Silakan hitung ulang uang fisik Anda.`, 'warning');
            } else if (res.supervisor_required) {
                showToast(`Selisih kas (Rp ${res.variance.toLocaleString('id-ID')}) melampaui toleransi. Butuh otorisasi Supervisor.`, 'error');
            } else {
                await executeClose();
            }
        } catch (e: any) {
            showToast('Gagal submit perhitungan kas: ' + e, 'error');
        }
    }

    async function executeClose() {
        if (!shiftId) return;
        try {
            await invoke('close_shift', { shiftId });
            showToast('Shift berhasil ditutup dengan sukses!', 'success');
            shiftActive = false;
            shiftId = null;
            report = null;
            denomCounts.forEach(d => d.qty = 0);
        } catch (e: any) {
            showToast('Gagal menutup shift: ' + e, 'error');
        }
    }

    async function handleSupervisorSuccess(grantId: string) {
        showAuthModal = false;
        if (!shiftId) return;

        try {
            await invoke('approve_shift_variance', {
                shiftId,
                grantId
            });
            showToast('Selisih disetujui oleh Supervisor. Menutup shift...', 'success');
            shiftActive = false;
            shiftId = null;
            report = null;
            denomCounts.forEach(d => d.qty = 0);
        } catch (e: any) {
            showToast('Gagal menyetujui selisih: ' + e, 'error');
        }
    }
</script>

<div class="min-h-screen bg-slate-950 p-4 md:p-8 flex items-start justify-center text-slate-100">
    <div class="w-full max-w-6xl bg-slate-900 border border-slate-800 rounded-3xl shadow-2xl flex flex-col gap-0 overflow-hidden">

        <!-- Header -->
        <div class="px-8 py-6 border-b border-slate-800 bg-slate-900/80 backdrop-blur">
            <div class="flex items-center gap-3">
                <div class="w-11 h-11 rounded-2xl bg-emerald-500/10 border border-emerald-500/20 flex items-center justify-center text-2xl">💵</div>
                <div>
                    <h1 class="text-2xl font-black tracking-tight text-white leading-none">Tutup Shift & Rekonsiliasi Kas</h1>
                    <p class="text-sm text-slate-400 mt-0.5">Lakukan perhitungan kas fisik akhir sebelum menutup shift Anda (Blind closing count).</p>
                </div>
            </div>
        </div>

        {#if shiftActive}
            <div class="flex flex-col md:flex-row gap-0">

                <!-- Left panel: Denominations -->
                <div class="flex-1 min-w-0 p-6 md:p-8 border-b md:border-b-0 md:border-r border-slate-800 flex flex-col gap-5">
                    <div class="flex items-center justify-between">
                        <h3 class="text-xs font-bold uppercase tracking-widest text-slate-500">
                            Denominasi Uang Kertas & Koin
                        </h3>
                        <span class="text-xs text-slate-600 font-mono">Jumlah Lembar / Keping</span>
                    </div>

                    <!-- Column headers -->
                    <div class="grid grid-cols-2 gap-3">
                        {#each denomCounts as item}
                            <div class="group flex items-center gap-3 bg-slate-800/50 border border-slate-700/60 hover:border-slate-600 rounded-xl px-4 py-3 transition-all">
                                <!-- Label -->
                                <div class="w-24 shrink-0">
                                    <span class="text-sm font-bold text-slate-200 tabular-nums">{item.label}</span>
                                    {#if item.qty > 0}
                                        <div class="text-[10px] text-emerald-400 font-medium mt-0.5 tabular-nums">
                                            = Rp {(item.denom * item.qty).toLocaleString('id-ID')}
                                        </div>
                                    {/if}
                                </div>

                                <!-- Controls -->
                                <div class="flex items-center gap-2 flex-1 justify-end">
                                    <button
                                        type="button"
                                        onclick={() => item.qty = Math.max(0, item.qty - 1)}
                                        class="w-8 h-8 shrink-0 rounded-lg bg-slate-700 hover:bg-red-500/20 hover:text-red-400 border border-slate-600 hover:border-red-500/40 font-bold text-slate-300 text-lg leading-none transition-all flex items-center justify-center cursor-pointer"
                                    >−</button>
                                    <input
                                        type="number"
                                        bind:value={item.qty}
                                        min="0"
                                        class="w-16 h-8 text-center bg-slate-950 border border-slate-700 rounded-lg font-bold text-white text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/60 focus:border-indigo-500 tabular-nums transition-all"
                                    />
                                    <button
                                        type="button"
                                        onclick={() => item.qty += 1}
                                        class="w-8 h-8 shrink-0 rounded-lg bg-slate-700 hover:bg-emerald-500/20 hover:text-emerald-400 border border-slate-600 hover:border-emerald-500/40 font-bold text-slate-300 text-lg leading-none transition-all flex items-center justify-center cursor-pointer"
                                    >+</button>
                                </div>
                            </div>
                        {/each}
                    </div>

                    <!-- Total -->
                    <div class="mt-auto pt-5 border-t border-slate-800 flex items-center justify-between bg-slate-950/40 -mx-8 md:-mx-8 px-8 py-4 -mb-8 md:-mb-8">
                        <div>
                            <div class="text-xs font-bold text-slate-500 uppercase tracking-widest">Total Kas Dihitung</div>
                            <div class="text-xs text-slate-600 mt-0.5">Jumlah semua denominasi</div>
                        </div>
                        <div class="text-right">
                            <span class="text-3xl font-black text-emerald-400 tabular-nums">
                                Rp {totalCounted().toLocaleString('id-ID')}
                            </span>
                        </div>
                    </div>
                </div>

                <!-- Right panel: Shift info + actions -->
                <div class="w-full md:w-80 shrink-0 p-6 md:p-8 flex flex-col gap-5 bg-slate-950/30">

                    <!-- Shift Info Card -->
                    <div class="bg-slate-900 border border-slate-800 rounded-2xl overflow-hidden">
                        <div class="px-5 py-3 border-b border-slate-800 bg-slate-800/40">
                            <h3 class="text-xs font-bold uppercase tracking-widest text-slate-500">Info Shift Aktif</h3>
                        </div>
                        <div class="p-5 flex flex-col gap-3">
                            <div class="flex justify-between items-center">
                                <span class="text-xs text-slate-500">ID Shift</span>
                                <span class="text-xs font-mono text-slate-300 bg-slate-800 px-2 py-0.5 rounded">{shiftId?.slice(0, 8)}…</span>
                            </div>
                            <div class="h-px bg-slate-800"></div>
                            <div class="flex justify-between items-start gap-3">
                                <span class="text-xs text-slate-500 shrink-0">Mulai Shift</span>
                                <span class="text-xs text-slate-300 text-right">{new Date(startedAt).toLocaleString('id-ID', { dateStyle: 'medium', timeStyle: 'short' })}</span>
                            </div>
                            <div class="h-px bg-slate-800"></div>
                            <div class="flex justify-between items-center">
                                <span class="text-xs text-slate-500">Modal Awal</span>
                                <span class="text-xs font-bold text-white tabular-nums">Rp {startingCash.toLocaleString('id-ID')}</span>
                            </div>
                        </div>
                    </div>

                    <!-- Variance Report -->
                    {#if report}
                        <div class="bg-slate-900 border {report.supervisor_required ? 'border-red-500/30' : 'border-amber-500/30'} rounded-2xl overflow-hidden">
                            <div class="px-5 py-3 border-b {report.supervisor_required ? 'border-red-500/20 bg-red-500/10' : 'border-amber-500/20 bg-amber-500/10'}">
                                <h3 class="text-xs font-bold uppercase tracking-widest {report.supervisor_required ? 'text-red-400' : 'text-amber-400'}">
                                    Hasil Perhitungan
                                </h3>
                            </div>
                            <div class="p-5 flex flex-col gap-3">
                                <div class="flex justify-between items-center">
                                    <span class="text-xs text-slate-500">Percobaan Ke</span>
                                    <span class="text-xs font-bold text-white">#{report.attempt_number}</span>
                                </div>
                                <div class="h-px bg-slate-800"></div>
                                <div class="flex justify-between items-center">
                                    <span class="text-xs text-slate-500">Selisih (Variance)</span>
                                    <span class="text-xs font-bold text-red-400 tabular-nums">Rp {report.variance.toLocaleString('id-ID')}</span>
                                </div>
                                <div class="h-px bg-slate-800"></div>

                                {#if report.recount_required}
                                    <div class="text-xs text-amber-300 bg-amber-500/10 border border-amber-500/20 rounded-xl p-3 leading-relaxed">
                                        ⚠️ Selisih kas terdeteksi. Harap hitung kembali uang fisik Anda dengan teliti.
                                    </div>
                                {:else}
                                    <div class="text-xs text-red-300 bg-red-500/10 border border-red-500/20 rounded-xl p-3 leading-relaxed">
                                        🛑 Selisih melampaui toleransi. Supervisor harus memverifikasi dan menyetujui.
                                    </div>
                                    <button
                                        type="button"
                                        onclick={() => showAuthModal = true}
                                        class="w-full py-3 bg-red-600 hover:bg-red-500 active:scale-95 text-white text-sm font-bold rounded-xl transition-all shadow-lg shadow-red-600/20 cursor-pointer"
                                    >
                                        🔑 Otorisasi Supervisor
                                    </button>
                                {/if}
                            </div>
                        </div>
                    {/if}

                    <!-- Submit Button -->
                    {#if !report || report.recount_required}
                        <div class="mt-auto">
                            <button
                                type="button"
                                onclick={handleSubmitCount}
                                class="w-full py-4 bg-gradient-to-r from-indigo-600 to-indigo-500 hover:from-indigo-500 hover:to-indigo-400 active:scale-95 text-white text-base font-bold rounded-2xl transition-all shadow-lg shadow-indigo-600/30 cursor-pointer"
                            >
                                📤 Kirim Perhitungan Kas
                            </button>
                            <p class="text-xs text-slate-600 text-center mt-3">Data tidak dapat diubah setelah dikirim</p>
                        </div>
                    {/if}
                </div>
            </div>

        {:else}
            <div class="p-12 flex flex-col items-center gap-5 text-center">
                <div class="w-20 h-20 rounded-3xl bg-amber-500/10 border border-amber-500/20 flex items-center justify-center text-4xl">⚠️</div>
                <div>
                    <h3 class="text-xl font-bold text-amber-400">Tidak Ada Shift Aktif</h3>
                    <p class="text-sm text-slate-400 mt-2">Saat ini Anda tidak memiliki shift kasir yang aktif.</p>
                </div>
                <a
                    href="/pos"
                    class="px-8 py-3 bg-indigo-600 hover:bg-indigo-500 active:scale-95 text-white text-sm font-bold rounded-xl transition-all"
                >
                    Kembali Ke POS
                </a>
            </div>
        {/if}
    </div>
</div>

<SupervisorAuthModal
    show={showAuthModal}
    actionType="shift.approve_shift_variance"
    amount={Math.abs(report?.variance || 0)}
    onSuccess={handleSupervisorSuccess}
    onCancel={() => showAuthModal = false}
/>
