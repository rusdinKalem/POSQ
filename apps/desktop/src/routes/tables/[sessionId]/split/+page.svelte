<script lang="ts">
    import { onMount } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';
    import { page } from '$app/stores';
    import { goto } from '$app/navigation';
    import { showToast } from '$lib/toast.svelte';

    type BillItemInfo = {
        id: string;
        product_id: string | null;
        product_name_snapshot: string;
        quantity: number;
        unit_price: number;
        gross_amount: number;
        item_discount_amount: number;
        taxable_amount: number;
        tax_amount: number;
        service_charge_amount: number;
        net_amount: number;
    };

    type BillWithItems = {
        id: string;
        dining_session_id: string;
        status: string;
        bill_number: string;
        subtotal: number;
        discount_total: number;
        tax_total: number;
        service_total: number;
        rounding_total: number;
        grand_total: number;
        paid_total: number;
        balance_amount: number;
        version: number;
        items: BillItemInfo[];
    };

    type SplitBillPreview = {
        original_bill_id: string;
        operation_token: string;
        bills: BillWithItems[];
    };

    const sessionId = $derived($page.params.sessionId);
    const billId = $derived($page.url.searchParams.get('bill_id') ?? '');

    let originalBill: BillWithItems | null = $state(null);
    let previewBills: BillWithItems[] = $state([]);
    let loading = $state(true);
    let previewing = $state(false);
    let committing = $state(false);

    let strategy: 'equally' | 'item' = $state('equally');
    let numSplits = $state(2);

    // Item-based allocation: map from item_id -> per-bill quantity
    type Allocation = Record<string, number[]>; // item_id -> [qty_for_bill_0, qty_for_bill_1, ...]
    let allocation: Allocation = $state({});

    function formatRp(amount: number) {
        return new Intl.NumberFormat('id-ID', { style: 'currency', currency: 'IDR', minimumFractionDigits: 0 }).format(amount);
    }

    async function loadOriginalBill() {
        loading = true;
        try {
            const bills = await invoke<BillWithItems[]>('get_bills_by_session', { sessionId });
            originalBill = bills.find(b => b.id === billId) ?? bills[0] ?? null;
            if (!originalBill) {
                showToast('Tagihan tidak ditemukan', 'error');
                goto(`/tables`);
                return;
            }
            // Initialize allocation
            const a: Allocation = {};
            for (const item of originalBill.items) {
                a[item.id] = Array.from({ length: numSplits }, (_, i) => i === 0 ? item.quantity : 0);
            }
            allocation = a;
        } catch (e: any) {
            showToast(e?.toString() ?? 'Gagal memuat tagihan', 'error');
        } finally {
            loading = false;
        }
    }

    function updateNumSplits(n: number) {
        numSplits = Math.max(2, Math.min(8, n));
        if (strategy === 'item' && originalBill) {
            const a: Allocation = {};
            for (const item of originalBill.items) {
                const old = allocation[item.id] ?? [];
                const arr = Array.from({ length: numSplits }, (_, i) => old[i] ?? 0);
                a[item.id] = arr;
            }
            allocation = a;
        }
        previewBills = [];
    }

    function setItemQty(itemId: string, billIdx: number, value: number) {
        const item = originalBill!.items.find(i => i.id === itemId)!;
        const arr = [...(allocation[itemId] ?? Array(numSplits).fill(0))];
        arr[billIdx] = Math.max(0, Math.min(value, item.quantity));
        allocation = { ...allocation, [itemId]: arr };
    }

    async function previewSplit() {
        if (!originalBill) return;
        previewing = true;
        try {
            const targets = strategy === 'item' ? Array.from({ length: numSplits }, (_, billIdx) => ({
                target_bill_index: billIdx,
                items: Object.entries(allocation)
                    .filter(([_, qtys]) => qtys[billIdx] > 0)
                    .map(([id, qtys]) => ({ id, quantity_to_move: qtys[billIdx] }))
            })) : [];

            const result = await invoke<SplitBillPreview>('preview_split_bill', {
                payload: {
                    bill_id: originalBill.id,
                    num_splits: numSplits,
                    strategy,
                    targets,
                    idempotency_key: crypto.randomUUID(),
                    authorized_by: null,
                    reason: null,
                }
            });
            previewBills = result.bills;
        } catch (e: any) {
            showToast(e?.toString() ?? 'Gagal preview split', 'error');
        } finally {
            previewing = false;
        }
    }

    async function commitSplit() {
        if (!originalBill || previewBills.length === 0) return;
        if (!confirm(`Konfirmasi pecah tagihan menjadi ${numSplits} bagian?`)) return;

        committing = true;
        try {
            const targets = strategy === 'item' ? Array.from({ length: numSplits }, (_, billIdx) => ({
                target_bill_index: billIdx,
                items: Object.entries(allocation)
                    .filter(([_, qtys]) => qtys[billIdx] > 0)
                    .map(([id, qtys]) => ({ id, quantity_to_move: qtys[billIdx] }))
            })) : [];

            await invoke('commit_split_bill', {
                payload: {
                    bill_id: originalBill.id,
                    num_splits: numSplits,
                    strategy,
                    targets,
                    idempotency_key: crypto.randomUUID(),
                    authorized_by: null,
                    reason: 'Split bill from floor plan',
                }
            });
            showToast('Tagihan berhasil dipecah!', 'success');
            goto('/tables');
        } catch (e: any) {
            showToast(e?.toString() ?? 'Gagal commit split', 'error');
        } finally {
            committing = false;
        }
    }

    onMount(loadOriginalBill);
</script>

<svelte:head>
    <title>Pecah Tagihan - POSQ</title>
    <meta name="description" content="Workspace untuk memecah tagihan F&B" />
</svelte:head>

<div class="min-h-screen bg-slate-950 text-slate-100 flex flex-col">
    <!-- Topbar -->
    <div class="flex items-center gap-3 px-5 py-3 bg-slate-900 border-b border-slate-800">
        <button onclick={() => goto('/tables')}
            class="text-slate-400 hover:text-white transition cursor-pointer text-sm">← Kembali</button>
        <span class="text-slate-600">/</span>
        <h1 class="text-sm font-bold">Pecah Tagihan (Split Bill)</h1>
        {#if originalBill}
            <span class="ml-auto text-xs text-slate-400 font-mono">{originalBill.bill_number}</span>
        {/if}
    </div>

    {#if loading}
        <div class="flex items-center justify-center flex-1">
            <div class="w-8 h-8 border-2 border-amber-500 border-t-transparent rounded-full animate-spin"></div>
        </div>
    {:else if originalBill}
        <div class="flex flex-1 overflow-hidden">
            <!-- Left: Config -->
            <div class="w-80 shrink-0 border-r border-slate-800 overflow-auto p-4 space-y-5">
                <!-- Original bill summary -->
                <div class="bg-slate-800 rounded-xl p-4">
                    <p class="text-xs text-slate-400 mb-1">Tagihan Asli</p>
                    <p class="font-bold text-white">{originalBill.bill_number}</p>
                    <p class="text-2xl font-black text-amber-400">{formatRp(originalBill.grand_total)}</p>
                    <p class="text-xs text-slate-400 mt-1">{originalBill.items.length} item</p>
                </div>

                <!-- Strategy -->
                <div>
                    <label class="text-xs font-bold text-slate-400 uppercase tracking-wider block mb-2">Metode Pemecahan</label>
                    <div class="grid grid-cols-2 gap-2">
                        <button onclick={() => { strategy = 'equally'; previewBills = []; }}
                            class="py-2 rounded-lg text-xs font-bold border-2 transition cursor-pointer
                                {strategy === 'equally' ? 'bg-amber-500 border-amber-400 text-slate-950' : 'border-slate-700 text-slate-400 hover:border-slate-500'}">
                            Rata
                        </button>
                        <button onclick={() => { strategy = 'item'; previewBills = []; }}
                            class="py-2 rounded-lg text-xs font-bold border-2 transition cursor-pointer
                                {strategy === 'item' ? 'bg-amber-500 border-amber-400 text-slate-950' : 'border-slate-700 text-slate-400 hover:border-slate-500'}">
                            Per Item
                        </button>
                    </div>
                </div>

                <!-- Num splits -->
                <div>
                    <label class="text-xs font-bold text-slate-400 uppercase tracking-wider block mb-2">Jumlah Bagian</label>
                    <div class="flex items-center gap-2">
                        <button onclick={() => updateNumSplits(numSplits - 1)}
                            class="w-8 h-8 rounded-lg bg-slate-700 hover:bg-slate-600 flex items-center justify-center font-bold cursor-pointer">−</button>
                        <span class="flex-1 text-center font-black text-xl">{numSplits}</span>
                        <button onclick={() => updateNumSplits(numSplits + 1)}
                            class="w-8 h-8 rounded-lg bg-slate-700 hover:bg-slate-600 flex items-center justify-center font-bold cursor-pointer">+</button>
                    </div>
                </div>

                <!-- Item allocation for 'item' strategy -->
                {#if strategy === 'item'}
                    <div>
                        <label class="text-xs font-bold text-slate-400 uppercase tracking-wider block mb-2">Alokasi Item</label>
                        <div class="space-y-3">
                            {#each originalBill.items as item (item.id)}
                                <div class="bg-slate-800 rounded-xl p-3">
                                    <p class="text-xs font-semibold text-slate-200 mb-2 truncate">{item.product_name_snapshot}</p>
                                    <p class="text-xs text-slate-500 mb-2">Total: {item.quantity} pcs</p>
                                    <div class="grid grid-cols-2 gap-1.5">
                                        {#each Array.from({ length: numSplits }, (_, i) => i) as billIdx}
                                            <div class="flex items-center gap-1">
                                                <span class="text-xs text-slate-500 w-6">B{billIdx + 1}</span>
                                                <input
                                                    type="number"
                                                    min="0"
                                                    max={item.quantity}
                                                    value={allocation[item.id]?.[billIdx] ?? 0}
                                                    oninput={(e) => setItemQty(item.id, billIdx, parseFloat((e.target as HTMLInputElement).value) || 0)}
                                                    class="flex-1 bg-slate-900 border border-slate-700 rounded-lg px-2 py-1 text-xs text-center focus:outline-none focus:border-amber-500" />
                                            </div>
                                        {/each}
                                    </div>
                                </div>
                            {/each}
                        </div>
                    </div>
                {/if}

                <!-- Actions -->
                <div class="space-y-2 pt-2">
                    <button
                        onclick={previewSplit}
                        disabled={previewing}
                        class="w-full py-2.5 bg-slate-700 hover:bg-slate-600 disabled:opacity-50 rounded-xl text-sm font-bold transition cursor-pointer">
                        {previewing ? 'Menghitung…' : '👁 Preview'}
                    </button>
                    <button
                        onclick={commitSplit}
                        disabled={committing || previewBills.length === 0}
                        class="w-full py-2.5 bg-amber-500 hover:bg-amber-400 text-slate-950 disabled:opacity-50 rounded-xl text-sm font-bold transition cursor-pointer">
                        {committing ? 'Memproses…' : '✂️ Konfirmasi Pecah'}
                    </button>
                </div>
            </div>

            <!-- Right: Preview -->
            <div class="flex-1 overflow-auto p-5">
                {#if previewBills.length === 0}
                    <div class="flex items-center justify-center h-full text-center">
                        <div>
                            <div class="text-4xl mb-3">✂️</div>
                            <p class="text-slate-400 text-sm">Klik <strong>Preview</strong> untuk melihat simulasi pemecahan tagihan.</p>
                        </div>
                    </div>
                {:else}
                    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                        {#each previewBills as bill, idx (bill.id)}
                            <div class="bg-slate-900 border border-slate-700 rounded-2xl overflow-hidden">
                                <div class="bg-slate-800 px-4 py-3 flex items-center justify-between">
                                    <span class="font-bold text-sm">Bagian {idx + 1}</span>
                                    <span class="text-xs text-slate-400">{bill.items.length} item</span>
                                </div>
                                <div class="p-4 space-y-2">
                                    {#each bill.items as item (item.id)}
                                        <div class="flex items-center justify-between text-xs">
                                            <span class="text-slate-300 truncate flex-1 mr-2">{item.product_name_snapshot}</span>
                                            <span class="text-slate-500 mr-2">×{item.quantity}</span>
                                            <span class="text-slate-200 font-semibold">{formatRp(item.net_amount)}</span>
                                        </div>
                                    {/each}
                                    <div class="pt-2 mt-2 border-t border-slate-700 space-y-1">
                                        <div class="flex justify-between text-xs text-slate-400">
                                            <span>Subtotal</span><span>{formatRp(bill.subtotal)}</span>
                                        </div>
                                        {#if bill.discount_total > 0}
                                            <div class="flex justify-between text-xs text-rose-400">
                                                <span>Diskon</span><span>−{formatRp(bill.discount_total)}</span>
                                            </div>
                                        {/if}
                                        {#if bill.tax_total > 0}
                                            <div class="flex justify-between text-xs text-slate-400">
                                                <span>Pajak</span><span>{formatRp(bill.tax_total)}</span>
                                            </div>
                                        {/if}
                                        <div class="flex justify-between text-sm font-black text-white pt-1">
                                            <span>TOTAL</span><span class="text-amber-400">{formatRp(bill.grand_total)}</span>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        {/each}
                    </div>

                    <!-- Invariant check -->
                    {@const totalPreview = previewBills.reduce((s, b) => s + b.grand_total, 0)}
                    <div class="mt-4 p-4 rounded-xl {totalPreview === originalBill.grand_total ? 'bg-emerald-950 border border-emerald-800' : 'bg-rose-950 border border-rose-800'}">
                        <div class="flex items-center gap-2 text-sm font-bold {totalPreview === originalBill.grand_total ? 'text-emerald-400' : 'text-rose-400'}">
                            <span>{totalPreview === originalBill.grand_total ? '✓' : '⚠'}</span>
                            <span>
                                Total Preview: {formatRp(totalPreview)}
                                {#if totalPreview !== originalBill.grand_total}
                                    — Selisih: {formatRp(Math.abs(totalPreview - originalBill.grand_total))}
                                {:else}
                                    — Sesuai tagihan asli
                                {/if}
                            </span>
                        </div>
                    </div>
                {/if}
            </div>
        </div>
    {/if}
</div>
