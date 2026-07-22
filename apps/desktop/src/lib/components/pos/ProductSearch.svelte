<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import SyncStatusModal from "./SyncStatusModal.svelte";

    interface Props {
        searchQuery: string;
        savedBillsCount: number;
        onOpenCustomItem: () => void;
        onOpenSavedBills: () => void;
    }

    let { searchQuery = $bindable(), savedBillsCount, onOpenCustomItem, onOpenSavedBills }: Props = $props();

    let showSyncModal = $state(false);
    let pendingCount = $state(0);
    let isOnline = $state(true);

    async function checkStatus() {
        try {
            const status: any = await invoke("get_sync_status");
            if (status) {
                pendingCount = status.pendingCount || 0;
                isOnline = status.isOnline ?? true;
            }
        } catch (_) {
            // Ignore error when running in dev/preview
        }
    }

    onMount(() => {
        checkStatus();
        const interval = setInterval(checkStatus, 10000);
        return () => clearInterval(interval);
    });
</script>

<div class="mb-4 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between shrink-0">
    <div class="hidden sm:block">
        <div class="flex items-center gap-2">
            <h1 class="text-2xl font-black text-slate-800 tracking-tight">Katalog Menu</h1>
            <button
                type="button"
                onclick={() => showSyncModal = true}
                class="px-2.5 py-1 rounded-full text-[11px] font-bold border flex items-center gap-1.5 transition cursor-pointer hover:shadow-sm {isOnline ? 'bg-emerald-50 text-emerald-700 border-emerald-200 hover:bg-emerald-100' : 'bg-amber-50 text-amber-800 border-amber-300 hover:bg-amber-100'}"
            >
                <span class="w-2 h-2 rounded-full {isOnline ? 'bg-emerald-500 animate-pulse' : 'bg-amber-500'}"></span>
                <span>{isOnline ? 'Local-First Online' : 'Mode Offline'}</span>
                {#if pendingCount > 0}
                    <span class="bg-amber-500 text-white text-[10px] px-1.5 py-0.2 rounded-full font-black">{pendingCount} pending</span>
                {/if}
            </button>
        </div>
        <p class="text-xs text-slate-400">Pilih menu atau scan barcode untuk menambahkan ke keranjang.</p>
    </div>
    
    <div class="flex items-center gap-2 w-full sm:w-auto">
        <div class="relative flex-1 sm:w-72">
            <div class="absolute inset-y-0 left-0 pl-3.5 flex items-center pointer-events-none">
                <svg class="w-5 h-5 text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path>
                </svg>
            </div>
            <input 
                type="text" 
                bind:value={searchQuery} 
                placeholder="Cari menu atau scan SKU/Barcode..." 
                class="w-full h-12 border border-slate-200 rounded-xl text-sm focus:border-blue-500 focus:ring-2 focus:ring-blue-100 outline-none bg-slate-50 hover:bg-white focus:bg-white transition"
                style="padding-left: 2.8rem; padding-right: 2.8rem;"
            />
            <div class="absolute inset-y-0 right-0 pr-3.5 flex items-center pointer-events-none text-slate-400" title="Barcode scanner aktif">
                <svg class="w-5 h-5 text-slate-400 animate-pulse" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 5h2M7 5h1M10 5h3M15 5h1M18 5h3M3 9h2M7 9h3M12 9h2M16 9h1M19 9h2M3 13h1M6 13h2M10 13h1M13 13h3M18 13h3M3 17h3M8 17h1M11 17h2M15 17h2M19 17h2"></path>
                </svg>
            </div>
        </div>
        
        <button 
            type="button" 
            onclick={onOpenCustomItem}
            class="bg-blue-600 hover:bg-blue-700 active:bg-blue-800 text-white text-xs font-bold px-4 rounded-xl transition flex items-center gap-1.5 shadow-sm shrink-0 cursor-pointer h-12 min-w-[48px] justify-center">
            ➕ <span class="hidden sm:inline">Item Kustom</span>
        </button>

        <button 
            type="button" 
            onclick={onOpenSavedBills}
            class="bg-amber-500 hover:bg-amber-600 active:bg-amber-700 text-white text-xs font-bold px-4 rounded-xl transition flex items-center gap-1.5 shadow-sm shrink-0 cursor-pointer h-12 min-w-[48px] justify-center relative">
            📂 <span class="hidden sm:inline">Tagihan</span>
            {#if savedBillsCount > 0}
                <span class="absolute -top-1.5 -right-1.5 bg-red-600 text-white text-[10px] w-5 h-5 flex items-center justify-center rounded-full shadow-sm border-2 border-white animate-in zoom-in duration-300 font-black">
                    {savedBillsCount}
                </span>
            {/if}
        </button>
    </div>
</div>

<SyncStatusModal show={showSyncModal} onClose={() => showSyncModal = false} />
