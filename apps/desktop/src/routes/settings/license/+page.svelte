<script lang="ts">
    import { onMount } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';

    let mode = $state("Checking...");
    let token = $state<any>(null);
    let errorMsg = $state<string | null>(null);
    let activationName = $state("Kasir-Utama");

    async function loadLicense() {
        errorMsg = null;
        try {
            const res: any = await invoke('verify_license');
            mode = res.mode;
            token = res.token;
            if (res.error) errorMsg = res.error;
        } catch (err: any) {
            errorMsg = "System Error: " + err;
        }
    }

    async function activateDevice() {
        try {
            const res: any = await invoke('activate_device', { deviceName: activationName });
            mode = res.mode;
            token = res.token;
            if (res.error) errorMsg = res.error;
        } catch (err: any) {
            errorMsg = "Activation Failed: " + err;
        }
    }

    async function refreshLicense() {
        try {
            const res: any = await invoke('refresh_license');
            mode = res.mode;
            token = res.token;
            if (res.error) errorMsg = res.error;
        } catch (err: any) {
            errorMsg = "Refresh Failed: " + err;
        }
    }

    onMount(() => {
        loadLicense();
    });

    const statusBadge = $derived(
        mode === "Active" ? "bg-emerald-50 text-emerald-700 border-emerald-200" 
        : mode === "Grace" ? "bg-amber-50 text-amber-700 border-amber-200" 
        : "bg-rose-50 text-rose-700 border-rose-200"
    );
</script>

<div class="min-h-screen bg-slate-50/50 py-12 px-4 sm:px-6 lg:px-8 font-sans">
    <div class="max-w-3xl mx-auto">
        <div class="mb-8 flex flex-col gap-3">
            <a href="/" class="inline-flex items-center text-sm font-semibold text-slate-500 hover:text-blue-600 transition-colors w-fit">
                <span class="mr-1">←</span> Kembali ke Dashboard
            </a>
            <h1 class="text-3xl font-extrabold text-slate-900 tracking-tight">Manajemen Lisensi</h1>
            <p class="text-slate-500 text-sm">Kelola status aktivasi dan lisensi kasir lokal POSQ Anda.</p>
        </div>

        {#if errorMsg}
            <div class="mb-6 p-4 bg-rose-50 border border-rose-200 text-rose-900 rounded-xl flex gap-3 items-start text-sm">
                <span class="text-rose-500 font-bold text-base leading-none">⚠️</span>
                <div>
                    <h4 class="font-bold mb-1">Terjadi Kesalahan</h4>
                    <p class="opacity-90">{errorMsg}</p>
                </div>
            </div>
        {/if}

        <div class="bg-white border border-slate-200/80 rounded-2xl shadow-xs p-6 sm:p-8">
            <h2 class="text-xl font-bold text-slate-800 mb-6 border-b border-slate-100 pb-4">Status Lisensi Saat Ini</h2>
            
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 mb-8">
                <div class="bg-slate-50/50 border border-slate-100 p-5 rounded-2xl flex flex-col gap-1.5">
                    <span class="text-[10px] uppercase tracking-wider text-slate-400 font-bold">Mode Operasional</span>
                    <span class="text-xs font-bold px-3 py-1 border rounded-full w-fit uppercase text-center mt-1 {statusBadge}">
                        {mode}
                    </span>
                </div>
                
                {#if token}
                    <div class="bg-slate-50/50 border border-slate-100 p-5 rounded-2xl flex flex-col gap-1">
                        <span class="text-[10px] uppercase tracking-wider text-slate-400 font-bold">ID Perangkat</span>
                        <span class="text-sm font-mono font-medium text-slate-700 select-all truncate">{token.device_id}</span>
                    </div>
                    <div class="bg-slate-50/50 border border-slate-100 p-5 rounded-2xl flex flex-col gap-1">
                        <span class="text-[10px] uppercase tracking-wider text-slate-400 font-bold">Paket Langganan</span>
                        <span class="text-base font-bold text-slate-800 uppercase">{token.plan}</span>
                    </div>
                    <div class="bg-slate-50/50 border border-slate-100 p-5 rounded-2xl flex flex-col gap-1">
                        <span class="text-[10px] uppercase tracking-wider text-slate-400 font-bold">Berlaku Sampai</span>
                        <span class="text-sm font-medium text-slate-700">{new Date(token.valid_until).toLocaleString('id-ID', { dateStyle: 'long', timeStyle: 'short' })}</span>
                    </div>
                {/if}
            </div>

            {#if mode === "Unlicensed"}
                <div class="border-t border-slate-100 pt-6 mt-2">
                    <h3 class="text-lg font-bold text-slate-800 mb-2">Aktivasi Perangkat Baru</h3>
                    <p class="text-sm text-slate-500 mb-4">Daftarkan kasir ini dengan nama perangkat unik untuk menghubungkannya ke control plane.</p>
                    
                    <div class="flex flex-col sm:flex-row gap-3">
                        <input 
                            type="text" 
                            bind:value={activationName} 
                            placeholder="Nama Perangkat (ex: Kasir-1)" 
                            class="flex-1 bg-white border border-slate-300 rounded-xl px-4 py-3 text-sm focus:outline-none focus:border-blue-500 focus:ring-4 focus:ring-blue-500/10 transition-all font-medium text-slate-800"
                        />
                        <button 
                            onclick={activateDevice} 
                            class="bg-blue-600 hover:bg-blue-700 text-white font-bold text-sm py-3 px-6 rounded-xl shadow-md hover:shadow-lg active:scale-98 transition-all cursor-pointer"
                        >
                            Aktivasi via Server
                        </button>
                    </div>
                </div>
            {:else}
                <div class="flex flex-col gap-4 border-t border-slate-100 pt-6">
                    <button 
                        onclick={refreshLicense} 
                        class="w-full sm:w-fit bg-emerald-600 hover:bg-emerald-700 text-white font-bold text-sm py-3 px-6 rounded-xl shadow-md hover:shadow-lg active:scale-98 transition-all cursor-pointer"
                    >
                        Refresh Lisensi (Sync)
                    </button>
                    <p class="text-xs text-slate-400">Gunakan ini setelah melakukan pembayaran langganan untuk memperbarui status lokal.</p>
                </div>
            {/if}
        </div>
    </div>
</div>
