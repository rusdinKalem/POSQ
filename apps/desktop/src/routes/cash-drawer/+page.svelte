<script lang="ts">
    import { onMount } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';
    import { showToast } from '$lib/toast.svelte';
    import SupervisorAuthModal from '$lib/components/SupervisorAuthModal.svelte';

    let shiftActive = $state(false);
    let shiftId = $state<string | null>(null);

    // Ledger records
    let movements = $state<any[]>([]);
    let loading = $state(false);

    // Form inputs
    let operationType = $state<'CASH_IN' | 'CASH_OUT' | 'SAFE_DROP'>('CASH_IN');
    let amount = $state<number>(0);
    let category = $state('');
    let reason = $state('');
    let bagNumber = $state('');

    // Supervisor flow
    let showAuthModal = $state(false);
    let authAction = $state('cash.cash_out');
    let authAmount = $state(0);
    let targetMovementIdForReverse = $state<string | null>(null);

    onMount(async () => {
        await loadShiftAndLedger();
    });

    async function loadShiftAndLedger() {
        loading = true;
        try {
            const status: any = await invoke('check_active_shift');
            if (status.active) {
                shiftActive = true;
                shiftId = status.shift_id;
                await loadMovements();
            } else {
                shiftActive = false;
                shiftId = null;
                movements = [];
            }
        } catch (e) {
            console.error('Error loading shift state:', e);
        } finally {
            loading = false;
        }
    }

    async function loadMovements() {
        if (!shiftId) return;
        try {
            const list: any[] = await invoke('get_cash_movements', { shiftId });
            movements = list;
        } catch (e: any) {
            showToast('Gagal memuat mutasi kas: ' + e, 'error');
        }
    }

    async function handleSubmit() {
        if (amount <= 0) {
            showToast('Jumlah uang harus lebih besar dari Rp 0', 'error');
            return;
        }
        if (!shiftId) return;

        if (operationType === 'CASH_IN') {
            try {
                await invoke('post_cash_in', {
                    shiftId,
                    amount,
                    category,
                    reason
                });
                showToast('Sukses memposting Cash In', 'success');
                resetForm();
                await loadMovements();
            } catch (e: any) {
                showToast('Gagal posting Cash In: ' + e, 'error');
            }
        } else if (operationType === 'CASH_OUT') {
            // Require supervisor auth if amount > 200,000 IDR
            if (amount > 200000) {
                authAction = 'cash.cash_out';
                authAmount = amount;
                showAuthModal = true;
            } else {
                try {
                    await invoke('post_cash_out', {
                        shiftId,
                        amount,
                        category,
                        reason,
                        grantId: null
                    });
                    showToast('Sukses memposting Cash Out', 'success');
                    resetForm();
                    await loadMovements();
                } catch (e: any) {
                    showToast('Gagal posting Cash Out: ' + e, 'error');
                }
            }
        } else if (operationType === 'SAFE_DROP') {
            if (!bagNumber.trim()) {
                showToast('Nomor amplop/bag wajib diisi untuk Safe Drop', 'error');
                return;
            }
            try {
                await invoke('post_safe_drop', {
                    shiftId,
                    amount,
                    bagNumber
                });
                showToast('Sukses memposting Safe Drop', 'success');
                resetForm();
                await loadMovements();
            } catch (e: any) {
                showToast('Gagal posting Safe Drop: ' + e, 'error');
            }
        }
    }

    function resetForm() {
        amount = 0;
        category = '';
        reason = '';
        bagNumber = '';
    }

    async function handleAuthSuccess(grantId: string) {
        showAuthModal = false;
        if (!shiftId) return;

        if (authAction === 'cash.cash_out') {
            try {
                await invoke('post_cash_out', {
                    shiftId,
                    amount,
                    category,
                    reason,
                    grantId
                });
                showToast('Sukses memposting Cash Out dengan otorisasi supervisor', 'success');
                resetForm();
                await loadMovements();
            } catch (e: any) {
                showToast('Gagal posting Cash Out: ' + e, 'error');
            }
        } else if (authAction === 'cash.reverse') {
            if (!targetMovementIdForReverse) return;
            try {
                await invoke('reverse_cash_movement', {
                    movementId: targetMovementIdForReverse,
                    grantId
                });
                showToast('Mutasi kas berhasil di-reverse', 'success');
                targetMovementIdForReverse = null;
                await loadMovements();
            } catch (e: any) {
                showToast('Gagal melakukan reversal: ' + e, 'error');
            }
        }
    }

    function initiateReverse(mId: string, mAmount: number) {
        targetMovementIdForReverse = mId;
        authAction = 'cash.reverse';
        authAmount = mAmount;
        showAuthModal = true;
    }
</script>

<div class="min-h-screen bg-slate-950 p-6 flex flex-col gap-6 text-slate-100">
    <!-- Header -->
    <div class="border-b border-slate-800 pb-4 flex justify-between items-center">
        <div>
            <h1 class="text-3xl font-black tracking-wide text-white flex items-center gap-2">
                💰 Laci Kas & Ledger Kas Kecil
            </h1>
            <p class="text-sm text-slate-400">
                Manajemen setoran kas, petty cash keluar, dan safe drop berkala.
            </p>
        </div>
        <a 
            href="/shift" 
            class="px-5 py-2.5 bg-slate-800 hover:bg-slate-700 text-white font-bold rounded-xl text-sm transition-colors border border-slate-700"
        >
            Status Shift
        </a>
    </div>

    {#if shiftActive}
        <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
            <!-- Left panel: Cash Transaction Forms -->
            <div class="bg-slate-900 border border-slate-800 rounded-3xl p-6 shadow-xl flex flex-col gap-6">
                <h2 class="text-lg font-bold text-white tracking-wide border-b border-slate-800 pb-2">
                    Posting Mutasi Kas
                </h2>

                <!-- Tabs -->
                <div class="grid grid-cols-3 gap-2 bg-slate-950 p-1.5 rounded-2xl border border-slate-800">
                    <button 
                        type="button"
                        onclick={() => { operationType = 'CASH_IN'; resetForm(); }}
                        class="py-2 text-xs font-bold rounded-xl transition-all
                            {operationType === 'CASH_IN' ? 'bg-indigo-600 text-white shadow-md' : 'text-slate-400 hover:text-white'}"
                    >
                        Cash In
                    </button>
                    <button 
                        type="button"
                        onclick={() => { operationType = 'CASH_OUT'; resetForm(); }}
                        class="py-2 text-xs font-bold rounded-xl transition-all
                            {operationType === 'CASH_OUT' ? 'bg-indigo-600 text-white shadow-md' : 'text-slate-400 hover:text-white'}"
                    >
                        Cash Out
                    </button>
                    <button 
                        type="button"
                        onclick={() => { operationType = 'SAFE_DROP'; resetForm(); }}
                        class="py-2 text-xs font-bold rounded-xl transition-all
                            {operationType === 'SAFE_DROP' ? 'bg-indigo-600 text-white shadow-md' : 'text-slate-400 hover:text-white'}"
                    >
                        Safe Drop
                    </button>
                </div>

                <!-- Form fields -->
                <div class="flex flex-col gap-4">
                    <div class="flex flex-col gap-1.5">
                        <label class="text-xs font-bold text-slate-400 uppercase tracking-wider">Jumlah Uang (Rp)</label>
                        <input 
                            type="number" 
                            bind:value={amount} 
                            placeholder="0"
                            class="w-full text-center bg-slate-950 border border-slate-800 rounded-2xl py-3.5 text-2xl font-black text-white focus:outline-none focus:ring-2 focus:ring-indigo-500"
                        />
                    </div>

                    {#if operationType === 'CASH_IN'}
                        <div class="flex flex-col gap-1.5">
                            <label class="text-xs font-bold text-slate-400 uppercase tracking-wider">Kategori Setoran</label>
                            <select 
                                bind:value={category}
                                class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2.5 text-sm text-white focus:outline-none focus:ring-2 focus:ring-indigo-500"
                            >
                                <option value="">Pilih Kategori...</option>
                                <option value="CHANGE_FUND">Tambah Modal Laci (Change Fund)</option>
                                <option value="SAFE_RECEIPT">Terima dari Safe Utama</option>
                                <option value="PETTY_CASH_RETURN">Pengembalian Sisa Petty Cash</option>
                            </select>
                        </div>
                    {/if}

                    {#if operationType === 'CASH_OUT'}
                        <div class="flex flex-col gap-1.5">
                            <label class="text-xs font-bold text-slate-400 uppercase tracking-wider">Kategori Pengeluaran</label>
                            <select 
                                bind:value={category}
                                class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2.5 text-sm text-white focus:outline-none focus:ring-2 focus:ring-indigo-500"
                            >
                                <option value="">Pilih Kategori...</option>
                                <option value="ICE_PURCHASE">Beli Es Batu</option>
                                <option value="PARKING">Parkir Kurir/Supplier</option>
                                <option value="DELIVERY">Ongkir / Kurir</option>
                                <option value="GAS">Beli Gas LPG</option>
                                <option value="CLEANING_SUPPLIES">Alat Kebersihan</option>
                                <option value="EMERGENCY_PURCHASE">Darurat Operasional</option>
                                <option value="TRANSPORT">Transportasi / Bensin</option>
                            </select>
                        </div>
                    {/if}

                    {#if operationType === 'SAFE_DROP'}
                        <div class="flex flex-col gap-1.5">
                            <label class="text-xs font-bold text-slate-400 uppercase tracking-wider">Nomor Amplop/Bag</label>
                            <input 
                                type="text" 
                                bind:value={bagNumber} 
                                placeholder="Contoh: BAG-001"
                                class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2.5 text-sm text-white focus:outline-none focus:ring-2 focus:ring-indigo-500"
                            />
                        </div>
                    {/if}

                    <div class="flex flex-col gap-1.5">
                        <label class="text-xs font-bold text-slate-400 uppercase tracking-wider">Keterangan / Catatan</label>
                        <textarea 
                            bind:value={reason} 
                            placeholder="Alasan detail transaksi..."
                            rows="3"
                            class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-sm text-white focus:outline-none focus:ring-2 focus:ring-indigo-500 resize-none"
                        ></textarea>
                    </div>

                    <button 
                        type="button"
                        onclick={handleSubmit}
                        class="w-full py-3.5 bg-indigo-600 hover:bg-indigo-500 text-white font-bold rounded-2xl transition-colors shadow-lg shadow-indigo-600/30 mt-2"
                    >
                        Posting ke Ledger
                    </button>
                </div>
            </div>

            <!-- Right panel: Append-Only Cash Ledger View -->
            <div class="lg:col-span-2 bg-slate-900 border border-slate-800 rounded-3xl p-6 shadow-xl flex flex-col gap-4">
                <h2 class="text-lg font-bold text-white tracking-wide border-b border-slate-800 pb-2">
                    Mutasi Kas Shift Ini (Append-Only Ledger)
                </h2>

                <div class="overflow-x-auto">
                    <table class="w-full border-collapse text-left">
                        <thead>
                            <tr class="border-b border-slate-800 text-xs font-bold text-slate-400 uppercase tracking-wider">
                                <th class="pb-3 px-2">No. Mutasi</th>
                                <th class="pb-3 px-2">Jenis</th>
                                <th class="pb-3 px-2">Kategori/Info</th>
                                <th class="pb-3 px-2 text-right">Jumlah</th>
                                <th class="pb-3 px-2">Oleh</th>
                                <th class="pb-3 px-2">Waktu</th>
                                <th class="pb-3 px-2 text-center">Aksi</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-slate-800/50 text-xs">
                            {#if movements.length === 0}
                                <tr>
                                    <td colspan="7" class="py-8 text-center text-slate-500 font-medium">
                                        Belum ada mutasi kas untuk shift aktif ini.
                                    </td>
                                </tr>
                            {:else}
                                {#each movements as cm}
                                    <tr class="hover:bg-slate-950/20 transition-colors">
                                        <td class="py-3 px-2 font-mono text-slate-300">{cm.movement_number}</td>
                                        <td class="py-3 px-2 font-semibold">
                                            {#if cm.type_str === 'CASH_IN'}
                                                <span class="text-emerald-400">💵 Cash In</span>
                                            {:else if cm.type_str === 'CASH_OUT'}
                                                <span class="text-amber-400">💸 Cash Out</span>
                                            {:else if cm.type_str === 'SAFE_DROP'}
                                                <span class="text-blue-400">🔒 Safe Drop</span>
                                            {:else if cm.type_str === 'REVERSAL'}
                                                <span class="text-red-400">🔄 Reversal</span>
                                            {:else}
                                                <span class="text-slate-300">{cm.type_str}</span>
                                            {/if}
                                        </td>
                                        <td class="py-3 px-2 text-slate-300">
                                            <div>{cm.reason_code || '-'}</div>
                                            {#if cm.notes}
                                                <div class="text-[10px] text-slate-500 italic max-w-xs truncate">{cm.notes}</div>
                                            {/if}
                                        </td>
                                        <td class="py-3 px-2 text-right font-bold text-white">
                                            Rp {cm.amount.toLocaleString('id-ID')}
                                        </td>
                                        <td class="py-3 px-2 text-slate-400">{cm.performed_by_name}</td>
                                        <td class="py-3 px-2 text-slate-400">
                                            {new Date(cm.created_at).toLocaleTimeString('id-ID', { hour: '2-digit', minute: '2-digit' })}
                                        </td>
                                        <td class="py-3 px-2 text-center">
                                            {#if cm.type_str !== 'REVERSAL' && cm.type_str !== 'STARTING_CASH'}
                                                <button 
                                                    type="button"
                                                    onclick={() => initiateReverse(cm.id, cm.amount)}
                                                    class="px-2.5 py-1.5 bg-red-950 hover:bg-red-900 border border-red-500/20 text-red-400 font-bold rounded-lg transition-colors"
                                                >
                                                    Reverse
                                                </button>
                                            {:else}
                                                <span class="text-slate-600">-</span>
                                            {/if}
                                        </td>
                                    </tr>
                                {/each}
                            {/if}
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    {:else}
        <div class="border border-amber-500/20 bg-amber-500/10 rounded-2xl p-6 text-center flex flex-col items-center gap-4">
            <span class="text-4xl">⚠️</span>
            <div>
                <h3 class="text-lg font-bold text-amber-400">Tidak Ada Shift Aktif</h3>
                <p class="text-sm text-slate-400 mt-1">
                    Anda harus membuka shift sebelum dapat memposting transaksi kas masuk/keluar.
                </p>
            </div>
            <a 
                href="/pos" 
                class="px-6 py-3 bg-indigo-600 hover:bg-indigo-500 text-white text-sm font-bold rounded-xl transition-colors"
            >
                Buka Shift di POS
            </a>
        </div>
    {/if}
</div>

<SupervisorAuthModal 
    show={showAuthModal}
    actionType={authAction}
    amount={authAmount}
    onSuccess={handleAuthSuccess}
    onCancel={() => { showAuthModal = false; targetMovementIdForReverse = null; }}
/>
