<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";

    interface Props {
        show: boolean;
        onClose: () => void;
    }

    let { show = false, onClose }: Props = $props();

    let loading = $state(false);
    let syncing = $state(false);
    let status = $state<{
        mode: string;
        cloudUrl: string;
        cloudSyncEnabled: boolean;
        isOnline: boolean;
        pendingCount: number;
        failedCount: number;
        syncedCount: number;
        lastSyncedAt: string | null;
        lastError: string | null;
    } | null>(null);

    async function loadStatus() {
        loading = true;
        try {
            status = await invoke("get_sync_status");
        } catch (e) {
            console.error("Failed to load sync status", e);
        } finally {
            loading = false;
        }
    }

    async function handleTriggerSync() {
        syncing = true;
        try {
            status = await invoke("trigger_sync");
        } catch (e) {
            console.error("Failed to trigger sync", e);
        } finally {
            syncing = false;
        }
    }

    $effect(() => {
        if (show) {
            loadStatus();
        }
    });
</script>

{#if show}
    <div class="fixed inset-0 bg-slate-950/70 backdrop-blur-sm z-50 flex items-center justify-center p-4">
        <div class="bg-slate-900 border border-slate-800 rounded-2xl max-w-lg w-full overflow-hidden shadow-2xl">
            <!-- Header -->
            <div class="p-5 border-b border-slate-800 flex items-center justify-between">
                <div class="flex items-center gap-3">
                    <div class="w-10 h-10 rounded-xl bg-indigo-500/10 text-indigo-400 flex items-center justify-center">
                        <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                        </svg>
                    </div>
                    <div>
                        <h3 class="text-base font-bold text-white">Status Sinkronisasi & Offline</h3>
                        <p class="text-xs text-slate-400">Pengawasan Outbox Queue & Status Jaringan</p>
                    </div>
                </div>
                <button 
                    onclick={onClose}
                    class="text-slate-400 hover:text-white p-2 rounded-lg hover:bg-slate-800 transition"
                >
                    ✕
                </button>
            </div>

            <!-- Body -->
            <div class="p-5 space-y-4">
                {#if loading}
                    <div class="flex items-center justify-center py-8">
                        <div class="w-6 h-6 border-2 border-indigo-500 border-t-transparent rounded-full animate-spin"></div>
                    </div>
                {:else if status}
                    <!-- Connection Status Banner -->
                    <div class="p-4 rounded-xl border flex items-center justify-between {status.isOnline ? 'bg-emerald-500/10 border-emerald-500/20 text-emerald-400' : 'bg-amber-500/10 border-amber-500/20 text-amber-400'}">
                        <div class="flex items-center gap-3">
                            <span class="relative flex h-3 w-3">
                                {#if status.isOnline}
                                    <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                                    <span class="relative inline-flex rounded-full h-3 w-3 bg-emerald-500"></span>
                                {:else}
                                    <span class="relative inline-flex rounded-full h-3 w-3 bg-amber-500"></span>
                                {/if}
                            </span>
                            <div>
                                <p class="text-sm font-bold">
                                    {status.isOnline ? 'Koneksi Terhubung' : 'Mode Offline — Standalone Local First'}
                                </p>
                                <p class="text-xs opacity-80">
                                    {status.isOnline ? 'Perangkat terhubung ke server sync' : 'Transaksi tersimpan aman di SQLite lokal'}
                                </p>
                            </div>
                        </div>
                        <span class="text-xs font-mono font-semibold px-2.5 py-1 rounded bg-slate-800 text-slate-300">
                            {status.mode}
                        </span>
                    </div>

                    <!-- Counter Stats Grid -->
                    <div class="grid grid-cols-3 gap-3">
                        <div class="bg-slate-950/60 p-3 rounded-xl border border-slate-800 text-center">
                            <p class="text-xs text-slate-400">Pending Outbox</p>
                            <p class="text-xl font-bold text-amber-400 mt-1">{status.pendingCount}</p>
                        </div>
                        <div class="bg-slate-950/60 p-3 rounded-xl border border-slate-800 text-center">
                            <p class="text-xs text-slate-400">Tersinkron</p>
                            <p class="text-xl font-bold text-emerald-400 mt-1">{status.syncedCount}</p>
                        </div>
                        <div class="bg-slate-950/60 p-3 rounded-xl border border-slate-800 text-center">
                            <p class="text-xs text-slate-400">Gagal / Exceeded</p>
                            <p class="text-xl font-bold text-rose-400 mt-1">{status.failedCount}</p>
                        </div>
                    </div>

                    <!-- Extra Info -->
                    <div class="space-y-2 text-xs">
                        {#if status.lastSyncedAt}
                            <div class="flex justify-between text-slate-400">
                                <span>Terakhir Berhasil Sync:</span>
                                <span class="font-mono text-slate-200">{new Date(status.lastSyncedAt).toLocaleString('id-ID')}</span>
                            </div>
                        {/if}
                        {#if status.lastError}
                            <div class="p-3 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-400 space-y-1">
                                <p class="font-bold">Catatan Error Terakhir:</p>
                                <p class="font-mono text-[11px] break-all">{status.lastError}</p>
                            </div>
                        {/if}
                    </div>
                {/if}
            </div>

            <!-- Footer Actions -->
            <div class="p-4 border-t border-slate-800 bg-slate-950/40 flex items-center justify-between">
                <button
                    onclick={loadStatus}
                    disabled={loading}
                    class="px-4 py-2 rounded-xl text-xs font-semibold bg-slate-800 hover:bg-slate-700 text-slate-300 transition"
                >
                    Refresh Status
                </button>
                <button
                    onclick={handleTriggerSync}
                    disabled={syncing || loading}
                    class="px-5 py-2 rounded-xl text-xs font-semibold bg-indigo-600 hover:bg-indigo-500 text-white flex items-center gap-2 transition disabled:opacity-50"
                >
                    {#if syncing}
                        <div class="w-3.5 h-3.5 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
                        <span>Memproses Sync...</span>
                    {:else}
                        <span>Retry Sync Sekarang</span>
                    {/if}
                </button>
            </div>
        </div>
    </div>
{/if}
