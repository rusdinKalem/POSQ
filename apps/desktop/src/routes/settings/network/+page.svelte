<script lang="ts">
    import { onMount } from 'svelte';
    import { goto } from '$app/navigation';
    import { invoke } from '@tauri-apps/api/core';
    import BackButton from '$lib/components/BackButton.svelte';

    type NetworkMode = 'Standalone' | 'Master' | 'Client';

    let selectedMode: NetworkMode = $state('Standalone');
    let masterIp: string = $state('');
    let cloudSyncEnabled: boolean = $state(false);
    let cloudVpsUrl: string = $state('');
    let cloudVpsToken: string = $state('');
    let saveMessage: string = $state('');

    onMount(async () => {
        try {
            const settings: any = await invoke('get_network_settings');
            if (settings.mode) {
                const modeStr = settings.mode.charAt(0) + settings.mode.slice(1).toLowerCase();
                selectedMode = modeStr as NetworkMode;
            }
            if (settings.master_ip) {
                masterIp = settings.master_ip;
            }
            if (settings.cloudSyncEnabled !== undefined) {
                cloudSyncEnabled = settings.cloudSyncEnabled;
            }
            if (settings.cloudVpsUrl) {
                cloudVpsUrl = settings.cloudVpsUrl;
            }
            if (settings.cloudVpsToken) {
                cloudVpsToken = settings.cloudVpsToken;
            }
        } catch (e) {
            console.error("Failed to load network settings from backend", e);
        }
    });

    async function saveNetworkSettings() {
        if (selectedMode === 'Client' && !masterIp) {
            saveMessage = "Error: IP Address Master wajib diisi jika mode Client dipilih.";
            setTimeout(() => saveMessage = "", 3000);
            return;
        }

        try {
            await invoke('save_network_settings', {
                mode: selectedMode.toUpperCase(),
                masterIp: masterIp,
                cloudSyncEnabled: cloudSyncEnabled,
                cloudVpsUrl: cloudVpsUrl,
                cloudVpsToken: cloudVpsToken
            });
            saveMessage = "Pengaturan Jaringan & Cloud berhasil disimpan!";
        } catch (e) {
            saveMessage = "Error: Gagal menyimpan pengaturan jaringan ke database.";
        }
        
        setTimeout(() => saveMessage = "", 5000);
    }
</script>
 
<div class="p-8 max-w-4xl mx-auto font-sans">
    <BackButton />
    <div class="mb-8">
        <h1 class="text-3xl font-extrabold text-slate-800">Pengaturan Jaringan & Topologi</h1>
        <p class="text-slate-500 mt-1">Konfigurasi sinkronisasi dan mode multi-terminal</p>
    </div>

    <div class="space-y-6">
        
        {#if saveMessage}
            <div class="p-4 rounded-xl text-sm font-bold {saveMessage.includes('Error') ? 'bg-rose-50 text-rose-600' : 'bg-emerald-50 text-emerald-600'}">
                {saveMessage}
            </div>
        {/if}

        <div class="bg-white rounded-2xl shadow-sm border border-slate-200 overflow-hidden">
            <div class="p-5 border-b border-slate-100 bg-slate-50/50">
                <h2 class="text-sm font-bold text-slate-800 flex items-center gap-2">
                    <span class="text-lg">🌐</span> Pilihan Mode Operasional
                </h2>
            </div>
            
            <div class="p-6 space-y-4">
                <!-- Standalone -->
                <label class="flex items-start gap-4 p-4 rounded-xl border transition-all cursor-pointer {selectedMode === 'Standalone' ? 'border-blue-500 bg-blue-50/50' : 'border-slate-200 hover:border-blue-300'}">
                    <div class="pt-1">
                        <input type="radio" name="networkMode" value="Standalone" bind:group={selectedMode} class="w-4 h-4 text-blue-600 accent-blue-600">
                    </div>
                    <div>
                        <div class="font-bold text-slate-800">Standalone (Kasir Tunggal)</div>
                        <div class="text-xs text-slate-500 mt-1">Sistem beroperasi mandiri tanpa perlu menghubungkan kasir lain. Cocok untuk toko dengan 1 perangkat kasir. (Default)</div>
                    </div>
                </label>

                <!-- Master -->
                <label class="flex items-start gap-4 p-4 rounded-xl border transition-all cursor-pointer {selectedMode === 'Master' ? 'border-blue-500 bg-blue-50/50' : 'border-slate-200 hover:border-blue-300'}">
                    <div class="pt-1">
                        <input type="radio" name="networkMode" value="Master" bind:group={selectedMode} class="w-4 h-4 text-blue-600 accent-blue-600">
                    </div>
                    <div>
                        <div class="font-bold text-slate-800">Server Master (Lokal)</div>
                        <div class="text-xs text-slate-500 mt-1">Perangkat ini akan memegang database utama dan menyalakan Web Server Latar Belakang (Port 3030) agar tablet/kasir lain bisa terhubung.</div>
                        
                        {#if selectedMode === 'Master'}
                            <div class="mt-4 p-3 bg-indigo-50 border border-indigo-100 rounded-lg text-xs text-indigo-700">
                                <span class="font-bold">Info:</span> Pastikan IP perangkat ini Statis. Tablet lain harus memasukkan IP dari komputer ini untuk terhubung.
                            </div>
                        {/if}
                    </div>
                </label>

                <!-- Client -->
                <label class="flex items-start gap-4 p-4 rounded-xl border transition-all cursor-pointer {selectedMode === 'Client' ? 'border-blue-500 bg-blue-50/50' : 'border-slate-200 hover:border-blue-300'}">
                    <div class="pt-1">
                        <input type="radio" name="networkMode" value="Client" bind:group={selectedMode} class="w-4 h-4 text-blue-600 accent-blue-600">
                    </div>
                    <div>
                        <div class="font-bold text-slate-800">Terminal Client (Kasir Tambahan / Tablet)</div>
                        <div class="text-xs text-slate-500 mt-1">Perangkat ini TIDAK akan menyimpan data secara lokal. Seluruh transaksi akan ditembakkan langsung ke IP Komputer Master.</div>
                        
                        {#if selectedMode === 'Client'}
                            <div class="mt-4 space-y-2">
                                <label for="master-ip" class="block text-xs font-bold text-slate-700">IP Address Server Master</label>
                                <input 
                                    id="master-ip"
                                    type="text" 
                                    bind:value={masterIp} 
                                    placeholder="Contoh: 192.168.1.10"
                                    class="w-full p-3 border border-slate-200 rounded-xl text-sm outline-none focus:border-blue-500" 
                                />
                                <p class="text-xs text-slate-500">Pastikan Anda terhubung di jaringan WiFi yang sama dengan Master.</p>
                            </div>
                        {/if}
                    </div>
                </label>
            </div>
        </div>

        {#if selectedMode !== 'Client'}
            <!-- Cloud Sync Section -->
            <div class="bg-white rounded-2xl shadow-sm border border-slate-200 overflow-hidden">
                <div class="p-5 border-b border-slate-100 bg-slate-50/50 flex items-center justify-between">
                    <h2 class="text-sm font-bold text-slate-800 flex items-center gap-2">
                        <span class="text-lg">☁️</span> Sinkronisasi Cloud (VPS)
                    </h2>
                    
                    <label class="relative inline-flex items-center cursor-pointer">
                        <input type="checkbox" bind:checked={cloudSyncEnabled} class="sr-only peer">
                        <div class="w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
                    </label>
                </div>
                
                {#if cloudSyncEnabled}
                    <div class="p-6 space-y-5">
                        <p class="text-xs text-slate-500">Fitur ini akan secara otomatis mengirimkan data transaksi (offline-first) ke VPS Anda setiap 30 detik secara berurutan.</p>
                        
                        <div class="space-y-4">
                            <div>
                                <label for="cloud-url" class="block text-xs font-bold text-slate-700 mb-1">URL / API Endpoint VPS</label>
                                <input 
                                    id="cloud-url"
                                    type="url" 
                                    bind:value={cloudVpsUrl} 
                                    placeholder="https://api.namatoko.com/sync"
                                    class="w-full p-3 border border-slate-200 rounded-xl text-sm outline-none focus:border-blue-500" 
                                />
                            </div>
                            
                            <div>
                                <label for="cloud-token" class="block text-xs font-bold text-slate-700 mb-1">Token API (Kunci Autentikasi)</label>
                                <input 
                                    id="cloud-token"
                                    type="password" 
                                    bind:value={cloudVpsToken} 
                                    placeholder="Masukkan Bearer Token VPS"
                                    class="w-full p-3 border border-slate-200 rounded-xl text-sm outline-none focus:border-blue-500" 
                                />
                            </div>
                        </div>
                    </div>
                {/if}
            </div>
        {/if}

        <div class="flex justify-end pt-4 pb-12">
            <button 
                onclick={saveNetworkSettings}
                class="bg-blue-600 hover:bg-blue-700 text-white font-bold py-3.5 px-8 rounded-xl transition shadow-lg shadow-blue-500/30 text-sm cursor-pointer w-full sm:w-auto">
                Simpan Semua Pengaturan
            </button>
        </div>

    </div>
</div>
