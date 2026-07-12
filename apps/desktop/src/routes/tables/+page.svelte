<script lang="ts">
    import { onMount } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';
    import { goto } from '$app/navigation';
    import { showToast } from '$lib/toast.svelte';

    type BillSummary = {
        id: string;
        bill_number: string;
        status: string;
        subtotal: number;
        discount_total: number;
        tax_total: number;
        service_total: number;
        rounding_total: number;
        grand_total: number;
        paid_total: number;
        balance_amount: number;
        version: number;
        reference_id: string | null;
        reference_type: string | null;
    };

    type ActiveSession = {
        session_id: string;
        session_status: string;
        tables: string[];
        bills: BillSummary[];
    };

    type TableStatus = {
        id: string;
        name: string;
        status: string;
        version: number;
        active_session: ActiveSession | null;
    };

    let tables: TableStatus[] = $state([]);
    let loading = $state(true);
    let selectedTable: TableStatus | null = $state(null);

    // Multi-select for join table operation
    let joinMode = $state(false);
    let selectedForJoin: string[] = $state([]);

    // Move/Swap state
    let moveMode = $state(false);
    let tableToMove: TableStatus | null = $state(null);

    const statusColor: Record<string, string> = {
        available: 'bg-emerald-50 border-emerald-300 hover:border-emerald-400',
        occupied: 'bg-amber-50 border-amber-300 hover:border-amber-400',
        reserved: 'bg-blue-50 border-blue-300 hover:border-blue-400',
        dirty: 'bg-rose-50 border-rose-300 hover:border-rose-400',
        disabled: 'bg-slate-100 border-slate-200 opacity-60',
    };

    const statusDot: Record<string, string> = {
        available: 'bg-emerald-500',
        occupied: 'bg-amber-500',
        reserved: 'bg-blue-500',
        dirty: 'bg-rose-500',
        disabled: 'bg-slate-400',
    };

    const statusLabel: Record<string, string> = {
        available: 'Tersedia',
        occupied: 'Terisi',
        reserved: 'Dipesan',
        dirty: 'Perlu Dibersihkan',
        disabled: 'Tidak Aktif',
    };

    function formatRp(amount: number): string {
        return new Intl.NumberFormat('id-ID', { style: 'currency', currency: 'IDR', minimumFractionDigits: 0 }).format(amount);
    }

    async function loadTables() {
        loading = true;
        try {
            tables = await invoke<TableStatus[]>('get_all_tables_status');
        } catch (e: any) {
            showToast(e?.toString() ?? 'Gagal memuat data meja', 'error');
        } finally {
            loading = false;
        }
    }

    function handleTableClick(table: TableStatus) {
        if (table.status === 'disabled') return;

        if (joinMode) {
            if (selectedForJoin.includes(table.id)) {
                selectedForJoin = selectedForJoin.filter(id => id !== table.id);
            } else {
                selectedForJoin = [...selectedForJoin, table.id];
            }
            return;
        }

        if (moveMode && tableToMove) {
            if (table.id === tableToMove.id) {
                cancelMove();
                return;
            }
            confirmMoveOrSwap(tableToMove, table);
            return;
        }

        selectedTable = table;
    }

    function startMoveMode(table: TableStatus) {
        tableToMove = table;
        moveMode = true;
        selectedTable = null;
        showToast(`Pilih meja tujuan untuk "${table.name}"`, 'info');
    }

    function cancelMove() {
        moveMode = false;
        tableToMove = null;
    }

    async function confirmMoveOrSwap(source: TableStatus, dest: TableStatus) {
        const isSwap = dest.active_session !== null;
        const action = isSwap ? 'swap_tables' : 'move_table';
        const label = isSwap ? 'Tukar' : 'Pindah';

        if (!confirm(`${label} "${source.name}" ke "${dest.name}"?`)) {
            cancelMove();
            return;
        }

        try {
            if (isSwap) {
                await invoke('commit_swap_tables', {
                    payload: {
                        session_id_a: source.active_session!.session_id,
                        session_id_b: dest.active_session!.session_id,
                        idempotency_key: crypto.randomUUID(),
                    }
                });
            } else {
                await invoke('commit_move_table', {
                    payload: {
                        session_id: source.active_session!.session_id,
                        from_table_id: source.id,
                        to_table_id: dest.id,
                        idempotency_key: crypto.randomUUID(),
                    }
                });
            }
            showToast(`${label} meja berhasil`, 'success');
            cancelMove();
            await loadTables();
        } catch (e: any) {
            showToast(e?.toString() ?? 'Operasi gagal', 'error');
            cancelMove();
        }
    }

    async function confirmJoinTables() {
        if (selectedForJoin.length < 2) {
            showToast('Pilih minimal 2 meja untuk digabung', 'warning');
            return;
        }
        if (!confirm(`Gabung ${selectedForJoin.length} meja menjadi satu sesi?`)) return;

        // Find main session (first occupied table)
        const occupiedIds = selectedForJoin.filter(id => {
            const t = tables.find(t => t.id === id);
            return t?.active_session;
        });

        if (occupiedIds.length === 0) {
            showToast('Minimal satu meja harus sudah memiliki sesi aktif', 'warning');
            return;
        }

        const mainTable = tables.find(t => t.id === occupiedIds[0])!;

        try {
            await invoke('commit_join_tables', {
                payload: {
                    primary_session_id: mainTable.active_session!.session_id,
                    table_ids_to_add: selectedForJoin.filter(id => id !== occupiedIds[0]),
                    idempotency_key: crypto.randomUUID(),
                }
            });
            showToast('Meja berhasil digabung', 'success');
            joinMode = false;
            selectedForJoin = [];
            await loadTables();
        } catch (e: any) {
            showToast(e?.toString() ?? 'Gagal menggabung meja', 'error');
        }
    }

    function openNewOrder(table: TableStatus) {
        // Navigate to POS with pre-selected table
        goto(`/pos?table=${encodeURIComponent(table.name)}`);
    }

    function openSplitBill(session: ActiveSession, bill: BillSummary) {
        goto(`/tables/${session.session_id}/split?bill_id=${bill.id}`);
    }

    function openJoinBill(session: ActiveSession) {
        goto(`/tables/${session.session_id}/join`);
    }

    onMount(() => {
        loadTables();
    });
</script>

<svelte:head>
    <title>Floor Plan - POSQ</title>
    <meta name="description" content="Tampilan denah meja restoran untuk manajemen F&B" />
</svelte:head>

<div class="h-screen flex flex-col bg-slate-950 text-slate-100 overflow-hidden">
    <!-- Header -->
    <div class="flex items-center justify-between px-5 py-3 bg-slate-900 border-b border-slate-800 shrink-0">
        <div class="flex items-center gap-3">
            <div class="h-8 w-8 rounded-lg bg-amber-500 flex items-center justify-center text-slate-950 font-black text-sm">🍽</div>
            <div>
                <h1 class="text-sm font-bold text-slate-100">Floor Plan</h1>
                <p class="text-xs text-slate-400">Manajemen Meja F&B</p>
            </div>
        </div>

        <div class="flex items-center gap-2">
            <!-- Legend -->
            <div class="hidden sm:flex items-center gap-3 mr-4">
                {#each Object.entries(statusLabel) as [key, label]}
                    <div class="flex items-center gap-1.5 text-xs text-slate-400">
                        <span class="inline-block w-2 h-2 rounded-full {statusDot[key]}"></span>
                        {label}
                    </div>
                {/each}
            </div>

            {#if joinMode}
                <button
                    onclick={confirmJoinTables}
                    class="px-3 py-1.5 bg-blue-600 hover:bg-blue-500 text-white text-xs font-bold rounded-lg transition cursor-pointer">
                    Gabung {selectedForJoin.length} Meja
                </button>
                <button
                    onclick={() => { joinMode = false; selectedForJoin = []; }}
                    class="px-3 py-1.5 bg-slate-700 hover:bg-slate-600 text-xs font-bold rounded-lg transition cursor-pointer">
                    Batal
                </button>
            {:else if moveMode}
                <span class="text-xs text-amber-400 animate-pulse font-bold">Pilih meja tujuan…</span>
                <button onclick={cancelMove} class="px-3 py-1.5 bg-slate-700 hover:bg-slate-600 text-xs font-bold rounded-lg transition cursor-pointer">
                    Batal
                </button>
            {:else}
                <button
                    onclick={() => joinMode = true}
                    class="px-3 py-1.5 bg-slate-700 hover:bg-slate-600 text-xs font-bold rounded-lg transition cursor-pointer flex items-center gap-1.5">
                    <span>🔗</span> Gabung Meja
                </button>
                <button
                    onclick={loadTables}
                    class="px-3 py-1.5 bg-slate-700 hover:bg-slate-600 text-xs font-bold rounded-lg transition cursor-pointer">
                    ↻ Refresh
                </button>
            {/if}
        </div>
    </div>

    <div class="flex flex-1 overflow-hidden">
        <!-- Floor Grid -->
        <div class="flex-1 overflow-auto p-5">
            {#if loading}
                <div class="flex items-center justify-center h-full">
                    <div class="text-center">
                        <div class="w-10 h-10 border-2 border-amber-500 border-t-transparent rounded-full animate-spin mx-auto mb-3"></div>
                        <p class="text-slate-400 text-sm">Memuat data meja…</p>
                    </div>
                </div>
            {:else if tables.length === 0}
                <div class="flex items-center justify-center h-full">
                    <div class="text-center">
                        <div class="text-5xl mb-4">🍽️</div>
                        <p class="text-slate-400 text-sm">Belum ada meja terdaftar.</p>
                        <p class="text-slate-500 text-xs mt-1">Jalankan database migration terlebih dahulu.</p>
                    </div>
                </div>
            {:else}
                <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-3">
                    {#each tables as table (table.id)}
                        {@const isSelectedJoin = selectedForJoin.includes(table.id)}
                        {@const isSource = tableToMove?.id === table.id}
                        {@const session = table.active_session}
                        {@const totalBill = session ? session.bills.reduce((s, b) => s + b.grand_total, 0) : 0}
                        <button
                            onclick={() => handleTableClick(table)}
                            class="relative text-left rounded-xl border-2 p-3 transition-all duration-200 cursor-pointer
                                {isSource ? 'border-amber-400 bg-amber-950 ring-2 ring-amber-400/50 scale-95' :
                                 isSelectedJoin ? 'border-blue-400 bg-blue-950 ring-2 ring-blue-400/50' :
                                 table.status === 'disabled' ? 'border-slate-700 bg-slate-900 opacity-50 cursor-not-allowed' :
                                 statusColor[table.status] ?? 'bg-slate-800 border-slate-700'}
                                {moveMode && !isSource && table.status !== 'disabled' ? 'hover:scale-105' : ''}">

                            <!-- Status indicator -->
                            <div class="flex items-start justify-between mb-2">
                                <span class="inline-flex items-center gap-1 text-xs font-bold text-slate-800">
                                    <span class="w-2 h-2 rounded-full {statusDot[table.status] ?? 'bg-slate-500'}"></span>
                                    {statusLabel[table.status] ?? table.status}
                                </span>
                                {#if isSelectedJoin}
                                    <span class="text-blue-600 font-bold text-xs">✓</span>
                                {/if}
                            </div>

                            <!-- Table name -->
                            <p class="font-black text-slate-900 text-base leading-tight mb-1">{table.name}</p>

                            {#if session}
                                <!-- Session info -->
                                <div class="mt-2 pt-2 border-t border-slate-200/50">
                                    <p class="text-xs text-slate-600 font-semibold">{session.bills.length} tagihan</p>
                                    <p class="text-sm font-black text-slate-800">{formatRp(totalBill)}</p>
                                    {#if session.tables.length > 1}
                                        <p class="text-xs text-blue-600 mt-0.5">🔗 {session.tables.join(' + ')}</p>
                                    {/if}
                                </div>
                            {/if}
                        </button>
                    {/each}
                </div>
            {/if}
        </div>

        <!-- Right Panel: Table Detail -->
        {#if selectedTable && !joinMode && !moveMode}
            <div class="w-80 shrink-0 bg-slate-900 border-l border-slate-800 flex flex-col overflow-hidden">
                <!-- Panel header -->
                <div class="px-4 py-3 border-b border-slate-800 flex items-center justify-between">
                    <div>
                        <h2 class="font-bold text-sm">{selectedTable.name}</h2>
                        <p class="text-xs text-slate-400">{statusLabel[selectedTable.status] ?? selectedTable.status}</p>
                    </div>
                    <button onclick={() => selectedTable = null}
                        class="w-7 h-7 rounded-lg bg-slate-800 hover:bg-slate-700 flex items-center justify-center text-slate-400 hover:text-white transition cursor-pointer text-xs">
                        ✕
                    </button>
                </div>

                <div class="flex-1 overflow-auto p-4 space-y-4">
                    {#if !selectedTable.active_session}
                        <!-- Empty table actions -->
                        <div class="text-center py-6">
                            <div class="text-3xl mb-3">🪑</div>
                            <p class="text-slate-400 text-sm mb-4">Meja kosong & siap digunakan.</p>
                            <button
                                onclick={() => openNewOrder(selectedTable!)}
                                class="w-full py-2.5 bg-amber-500 hover:bg-amber-400 text-slate-950 font-bold rounded-xl transition text-sm cursor-pointer">
                                + Buka Pesanan Baru
                            </button>
                        </div>
                    {:else}
                        {@const session = selectedTable.active_session}
                        <!-- Session overview -->
                        <div class="bg-slate-800 rounded-xl p-3 text-xs space-y-1">
                            <p class="text-slate-400">Sesi ID</p>
                            <p class="font-mono text-slate-300 truncate">{session.session_id}</p>
                            {#if session.tables.length > 1}
                                <p class="text-blue-400">🔗 Gabungan: {session.tables.join(', ')}</p>
                            {/if}
                        </div>

                        <!-- Bills -->
                        <div>
                            <h3 class="text-xs font-bold text-slate-400 uppercase tracking-wider mb-2">Tagihan Aktif</h3>
                            <div class="space-y-2">
                                {#each session.bills as bill (bill.id)}
                                    <div class="bg-slate-800 rounded-xl p-3 border border-slate-700">
                                        <div class="flex items-start justify-between mb-2">
                                            <p class="text-xs font-bold text-slate-200">{bill.bill_number}</p>
                                            <span class="text-xs px-1.5 py-0.5 rounded-md
                                                {bill.status === 'open' ? 'bg-amber-900 text-amber-300' : 'bg-emerald-900 text-emerald-300'}">
                                                {bill.status}
                                            </span>
                                        </div>
                                        <p class="text-base font-black text-white">{formatRp(bill.grand_total)}</p>
                                        {#if bill.paid_total > 0}
                                            <p class="text-xs text-slate-400">Sisa: {formatRp(bill.balance_amount)}</p>
                                        {/if}
                                        <button
                                            onclick={() => openSplitBill(session, bill)}
                                            class="mt-2 w-full py-1.5 bg-slate-700 hover:bg-slate-600 rounded-lg text-xs font-semibold transition cursor-pointer">
                                            ✂️ Pecah Tagihan
                                        </button>
                                    </div>
                                {/each}
                            </div>
                        </div>

                        <!-- Actions -->
                        <div class="space-y-2 pb-4">
                            {#if session.bills.length >= 2}
                                <button
                                    onclick={() => openJoinBill(session)}
                                    class="w-full py-2 bg-blue-700 hover:bg-blue-600 rounded-xl text-xs font-bold transition cursor-pointer">
                                    🔗 Gabung Tagihan
                                </button>
                            {/if}
                            <button
                                onclick={() => startMoveMode(selectedTable!)}
                                class="w-full py-2 bg-slate-700 hover:bg-slate-600 rounded-xl text-xs font-bold transition cursor-pointer">
                                🚚 Pindah / Tukar Meja
                            </button>
                            <button
                                onclick={() => openNewOrder(selectedTable!)}
                                class="w-full py-2 bg-amber-500 hover:bg-amber-400 text-slate-950 rounded-xl text-xs font-bold transition cursor-pointer">
                                + Tambah Pesanan
                            </button>
                        </div>
                    {/if}
                </div>
            </div>
        {/if}
    </div>
</div>
