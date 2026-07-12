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

    const sessionId = $derived($page.params.sessionId);

    let bills: BillWithItems[] = $state([]);
    let loading = $state(true);
    let committing = $state(false);
    let selected: Set<string> = $state(new Set());

    function formatRp(amount: number) {
        return new Intl.NumberFormat('id-ID', { style: 'currency', currency: 'IDR', minimumFractionDigits: 0 }).format(amount);
    }

    async function loadBills() {
        loading = true;
        try {
            bills = await invoke<BillWithItems[]>('get_bills_by_session', { sessionId });
            if (bills.length < 2) {
                showToast('Butuh minimal 2 tagihan untuk digabung', 'warning');
                goto('/tables');
            }
        } catch (e: any) {
            showToast(e?.toString() ?? 'Gagal memuat tagihan', 'error');
        } finally {
            loading = false;
        }
    }

    function toggleSelect(id: string) {
        const s = new Set(selected);
        if (s.has(id)) s.delete(id);
        else s.add(id);
        selected = s;
    }

    const selectedBills = $derived(bills.filter(b => selected.has(b.id)));
    const totalCombined = $derived(selectedBills.reduce((s, b) => s + b.grand_total, 0));

    async function commitJoin() {
        if (selected.size < 2) {
            showToast('Pilih minimal 2 tagihan untuk digabung', 'warning');
            return;
        }
        if (!confirm(`Gabung ${selected.size} tagihan menjadi satu? Tagihan lama akan diarsipkan.`)) return;

        committing = true;
        try {
            await invoke('commit_join_bills', {
                payload: {
                    bill_ids: [...selected],
                    target_table_id: '',
                    idempotency_key: crypto.randomUUID(),
                    authorized_by: null,
                    reason: 'Join bills from floor plan',
                }
            });
            showToast('Tagihan berhasil digabung!', 'success');
            goto('/tables');
        } catch (e: any) {
            showToast(e?.toString() ?? 'Gagal menggabung tagihan', 'error');
        } finally {
            committing = false;
        }
    }

    onMount(loadBills);
</script>

<svelte:head>
    <title>Gabung Tagihan - POSQ</title>
    <meta name="description" content="Workspace untuk menggabung tagihan F&B" />
</svelte:head>

<div class="min-h-screen bg-slate-950 text-slate-100 flex flex-col">
    <!-- Topbar -->
    <div class="flex items-center gap-3 px-5 py-3 bg-slate-900 border-b border-slate-800">
        <button onclick={() => goto('/tables')}
            class="text-slate-400 hover:text-white transition cursor-pointer text-sm">← Kembali</button>
        <span class="text-slate-600">/</span>
        <h1 class="text-sm font-bold">Gabung Tagihan (Join Bill)</h1>
        {#if selected.size >= 2}
            <button
                onclick={commitJoin}
                disabled={committing}
                class="ml-auto px-4 py-1.5 bg-blue-600 hover:bg-blue-500 disabled:opacity-50 rounded-lg text-xs font-bold transition cursor-pointer">
                {committing ? 'Memproses…' : `🔗 Gabung ${selected.size} Tagihan`}
            </button>
        {/if}
    </div>

    {#if loading}
        <div class="flex items-center justify-center flex-1">
            <div class="w-8 h-8 border-2 border-blue-500 border-t-transparent rounded-full animate-spin"></div>
        </div>
    {:else}
        <div class="flex flex-1 overflow-hidden">
            <!-- Left: Bill list -->
            <div class="flex-1 overflow-auto p-5">
                <p class="text-xs text-slate-400 mb-4">Pilih tagihan yang ingin digabung. Semua item akan dikonsolidasi ke dalam satu tagihan baru.</p>
                <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                    {#each bills as bill (bill.id)}
                        {@const isSelected = selected.has(bill.id)}
                        <button
                            onclick={() => toggleSelect(bill.id)}
                            class="text-left rounded-2xl border-2 overflow-hidden transition-all cursor-pointer
                                {isSelected ? 'border-blue-500 bg-blue-950/40' : 'border-slate-700 bg-slate-900 hover:border-slate-500'}">
                            <div class="px-4 py-3 {isSelected ? 'bg-blue-900/40' : 'bg-slate-800'} flex items-center justify-between">
                                <span class="font-bold text-sm">{bill.bill_number}</span>
                                <div class="flex items-center gap-2">
                                    <span class="text-xs px-1.5 py-0.5 rounded-md
                                        {bill.status === 'open' ? 'bg-amber-900 text-amber-300' : 'bg-emerald-900 text-emerald-300'}">
                                        {bill.status}
                                    </span>
                                    {#if isSelected}
                                        <span class="w-5 h-5 rounded-full bg-blue-500 flex items-center justify-center text-xs font-bold">✓</span>
                                    {:else}
                                        <span class="w-5 h-5 rounded-full border-2 border-slate-600"></span>
                                    {/if}
                                </div>
                            </div>
                            <div class="p-4 space-y-1.5">
                                {#each bill.items as item (item.id)}
                                    <div class="flex justify-between text-xs">
                                        <span class="text-slate-300 truncate flex-1 mr-2">{item.product_name_snapshot}</span>
                                        <span class="text-slate-500">×{item.quantity}</span>
                                        <span class="text-slate-200 ml-2">{formatRp(item.net_amount)}</span>
                                    </div>
                                {/each}
                                <div class="pt-2 border-t border-slate-700 flex justify-between items-center">
                                    <span class="text-xs text-slate-400">Total</span>
                                    <span class="font-black text-white">{formatRp(bill.grand_total)}</span>
                                </div>
                            </div>
                        </button>
                    {/each}
                </div>
            </div>

            <!-- Right: Summary -->
            <div class="w-72 shrink-0 border-l border-slate-800 p-4 overflow-auto">
                <h3 class="text-xs font-bold text-slate-400 uppercase tracking-wider mb-3">Ringkasan Gabungan</h3>

                {#if selected.size === 0}
                    <p class="text-slate-500 text-sm text-center py-6">Belum ada tagihan dipilih</p>
                {:else}
                    <div class="space-y-2 mb-4">
                        {#each selectedBills as bill (bill.id)}
                            <div class="bg-slate-800 rounded-xl px-3 py-2 flex justify-between text-xs">
                                <span class="text-slate-300 truncate flex-1 mr-2">{bill.bill_number}</span>
                                <span class="font-semibold">{formatRp(bill.grand_total)}</span>
                            </div>
                        {/each}
                    </div>

                    <div class="bg-slate-800 rounded-xl p-4 border border-slate-700">
                        <p class="text-xs text-slate-400 mb-1">Total Gabungan</p>
                        <p class="text-2xl font-black text-blue-400">{formatRp(totalCombined)}</p>
                        <p class="text-xs text-slate-500 mt-1">{selectedBills.reduce((s, b) => s + b.items.length, 0)} item total</p>
                    </div>

                    <div class="mt-4 p-3 bg-amber-950 border border-amber-800 rounded-xl text-xs text-amber-300">
                        ⚠️ Tagihan lama akan diarsipkan dengan status <strong>merged</strong>. Tindakan ini tidak dapat dibatalkan.
                    </div>

                    <button
                        onclick={commitJoin}
                        disabled={selected.size < 2 || committing}
                        class="mt-4 w-full py-3 bg-blue-600 hover:bg-blue-500 disabled:opacity-50 rounded-xl font-bold text-sm transition cursor-pointer">
                        {committing ? 'Memproses…' : '🔗 Gabung Sekarang'}
                    </button>
                {/if}
            </div>
        </div>
    {/if}
</div>
