<script lang="ts">
    import { invoke } from '@tauri-apps/api/core';
    import BackButton from '$lib/components/BackButton.svelte';

    let channel = "stable";
    let isChecking = false;
    let checkResult: any = null;
    let checkError: string | null = null;
    
    let isMigrating = false;
    let migrationStatus: string | null = null;
    let migrationError: string | null = null;

    async function checkUpdate() {
        isChecking = true;
        checkResult = null;
        checkError = null;
        
        try {
            const res: any = await invoke('check_update', { channel });
            checkResult = res;
            if (res.error) checkError = res.error;
        } catch (err: any) {
            checkError = "Failed to check update: " + err;
        } finally {
            isChecking = false;
        }
    }

    async function runSafeMigration() {
        if (!checkResult?.metadata?.signature) {
            migrationError = "Invalid update metadata.";
            return;
        }
        
        isMigrating = true;
        migrationStatus = "Validating Update Signature...";
        migrationError = null;

        try {
            // M10-002: Validate Signature first
            const isValid = await invoke('validate_update', { 
                metadata: checkResult.metadata, 
                signature: checkResult.metadata.signature 
            });
            if (!isValid) throw new Error("Signature validation failed.");
            
            // M10-003: Migration Backup Gate & Safe Migration
            migrationStatus = "Creating Pre-migration Backup and Running SQL Migrations...";
            const migRes: any = await invoke('run_safe_migration');
            
            if (migRes.success) {
                migrationStatus = "✅ Update and Migration Successful! Please restart the app.";
            } else {
                migrationError = migRes.message || "Unknown migration error.";
                migrationStatus = null;
            }
        } catch (err: any) {
            // M10-004: Failed migration recovery info
            migrationError = "MIGRATION HALTED: " + err;
            migrationStatus = null;
        } finally {
            isMigrating = false;
        }
    }
</script>

<div class="p-8 max-w-4xl mx-auto">
    <BackButton />
    <h1 class="text-3xl font-bold mb-6">Pembaruan Sistem (App Update)</h1>
    <p class="text-gray-600 mb-8">Pembaruan POSQ berjalan dengan pengamanan ketat. Setiap migrasi akan di-backup secara otomatis sebelum dieksekusi untuk mencegah kerusakan data toko Anda.</p>

    <div class="bg-white shadow rounded-lg p-6 mb-8 border border-gray-200">
        <h2 class="text-xl font-bold mb-4">Cek Pembaruan</h2>
        
        <div class="flex gap-4 items-center mb-6">
            <div>
                <label for="channel" class="block text-sm font-medium text-gray-700">Release Channel</label>
                <select id="channel" bind:value={channel} class="mt-1 block w-48 rounded-md border-gray-300 shadow-sm border p-2">
                    <option value="stable">Stable (Disarankan)</option>
                    <option value="beta">Beta (Uji Coba)</option>
                </select>
            </div>
            <button 
                on:click={checkUpdate} 
                disabled={isChecking || isMigrating}
                class="mt-5 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 text-white font-bold py-2 px-6 rounded shadow transition">
                {isChecking ? "Memeriksa..." : "Cek Pembaruan Sekarang"}
            </button>
        </div>

        {#if checkError}
            <div class="p-4 mb-4 bg-red-100 text-red-800 rounded border border-red-200">
                {checkError}
            </div>
        {/if}

        {#if checkResult && checkResult.success}
            <div class="border-t pt-4">
                {#if checkResult.update_available && checkResult.metadata}
                    <div class="bg-blue-50 p-4 rounded border border-blue-100">
                        <h3 class="font-bold text-blue-900 text-lg">Pembaruan Tersedia: Versi {checkResult.metadata.version}</h3>
                        <p class="text-blue-800 text-sm mt-1">Channel: <span class="uppercase font-semibold">{checkResult.metadata.channel}</span></p>
                        
                        <div class="mt-4 p-3 bg-white rounded border border-blue-200 text-sm">
                            <p class="font-semibold mb-1">Release Notes:</p>
                            <p class="text-gray-700">{checkResult.metadata.release_notes}</p>
                        </div>

                        <div class="mt-6">
                            {#if migrationError}
                                <div class="p-4 mb-4 bg-red-100 text-red-800 rounded border border-red-300 font-mono text-sm whitespace-pre-wrap">
                                    {migrationError}
                                </div>
                            {/if}

                            {#if migrationStatus}
                                <div class="p-4 mb-4 bg-green-100 text-green-800 rounded border border-green-300 font-bold">
                                    {migrationStatus}
                                </div>
                            {/if}

                            <button 
                                on:click={runSafeMigration}
                                disabled={isMigrating}
                                class="w-full bg-green-600 hover:bg-green-700 disabled:bg-gray-400 text-white font-bold py-3 px-6 rounded shadow transition flex justify-center items-center gap-2">
                                {#if isMigrating}
                                    <svg class="animate-spin h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path></svg>
                                    Memproses Safe Migration...
                                {:else}
                                    Install & Jalankan Safe Migration
                                {/if}
                            </button>
                            <p class="text-xs text-gray-500 mt-2 text-center">Tindakan ini akan mengunduh pembaruan, melakukan pencadangan otomatis (Auto-Backup), dan menjalankan migrasi database.</p>
                        </div>
                    </div>
                {:else}
                    <div class="bg-green-50 p-4 rounded border border-green-100 text-green-800 flex items-center gap-3">
                        <span class="text-2xl">✅</span>
                        <div>
                            <p class="font-bold">Aplikasi Anda sudah versi terbaru.</p>
                            <p class="text-sm">Tidak ada pembaruan di channel {channel} saat ini.</p>
                        </div>
                    </div>
                {/if}
            </div>
        {/if}
    </div>
</div>
