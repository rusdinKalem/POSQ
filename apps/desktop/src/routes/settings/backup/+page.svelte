<script>
    import { invoke } from '@tauri-apps/api/core';

    let isEncrypted = true;
    let showRecoveryKeyModal = false;
    let recoveryKey = "";
    let savedKeyConfirmation = false;
    
    let restorePath = "";
    let restoreKey = "";
    let statusMessage = "";

    async function handleGenerateKey() {
        if (!isEncrypted) {
            await executeBackup(null);
            return;
        }

        try {
            const res = await invoke('generate_recovery_key');
            recoveryKey = res.key;
            showRecoveryKeyModal = true;
        } catch (err) {
            statusMessage = "Gagal membuat key: " + err;
        }
    }

    async function confirmKeySaved() {
        if (!savedKeyConfirmation) return;
        showRecoveryKeyModal = false;
        await executeBackup(recoveryKey);
    }

    async function executeBackup(key) {
        statusMessage = "Memproses backup...";
        try {
            const result = await invoke('create_local_backup', {
                encrypt: isEncrypted,
                recoveryKey: key
            });
            if (result.success) {
                statusMessage = `Backup berhasil disimpan di: ${result.path}`;
            } else {
                statusMessage = "Gagal: " + result.message;
            }
        } catch (err) {
            statusMessage = "Error: " + err;
        }
    }

    async function handleRestore() {
        if (!restorePath) {
            statusMessage = "Masukkan path backup terlebih dahulu";
            return;
        }

        statusMessage = "Memproses restore...";
        try {
            const result = await invoke('restore_local_backup', {
                filePath: restorePath,
                recoveryKey: restoreKey ? restoreKey : null
            });
            if (result.success) {
                statusMessage = "Restore berhasil! Silakan restart aplikasi.";
            } else {
                statusMessage = "Gagal restore: " + result.message;
            }
        } catch (err) {
            statusMessage = "Error: " + err;
        }
    }
</script>

<div class="p-8 max-w-4xl mx-auto">
    <h1 class="text-3xl font-bold mb-6">Backup & Restore</h1>

    {#if statusMessage}
        <div class="p-4 mb-6 bg-blue-100 text-blue-800 rounded">
            {statusMessage}
        </div>
    {/if}

    <div class="bg-white shadow rounded-lg p-6 mb-8 border border-gray-200">
        <h2 class="text-xl font-bold mb-4">Buat Backup Lokal</h2>
        <p class="text-gray-600 mb-4">Simpan seluruh data operasional Anda. Untuk keamanan tingkat lanjut, disarankan menyalakan Enkripsi.</p>
        
        <label class="flex items-center mb-6 space-x-3 cursor-pointer">
            <input type="checkbox" bind:checked={isEncrypted} class="w-5 h-5 text-blue-600 border-gray-300 rounded" />
            <span class="font-medium text-gray-700">Aktifkan Enkripsi AES-256 (Sangat Disarankan)</span>
        </label>

        <button on:click={handleGenerateKey} class="bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-6 rounded shadow-md transition-colors">
            Mulai Backup
        </button>
    </div>

    <div class="bg-white shadow rounded-lg p-6 border border-gray-200">
        <h2 class="text-xl font-bold mb-4 text-red-600">Restore Data</h2>
        <p class="text-gray-600 mb-4">Peringatan: Proses ini akan menimpa seluruh database kasir Anda dengan data dari file backup.</p>
        
        <div class="space-y-4 max-w-lg">
            <div>
                <label class="block text-sm font-medium text-gray-700 mb-1">Path File Backup (.sql / .enc)</label>
                <input type="text" bind:value={restorePath} placeholder="C:\Users\Name\Documents\POSQ_Backups\..." class="w-full border border-gray-300 rounded px-3 py-2" />
            </div>
            
            <div>
                <label class="block text-sm font-medium text-gray-700 mb-1">Recovery Key (Kosongkan jika backup tidak dienkripsi)</label>
                <input type="text" bind:value={restoreKey} placeholder="Masukkan 64-karakter hex key" class="w-full border border-gray-300 rounded px-3 py-2 font-mono text-sm" />
            </div>

            <button on:click={handleRestore} class="bg-red-600 hover:bg-red-700 text-white font-bold py-2 px-6 rounded shadow-md transition-colors mt-2">
                Restore Sekarang
            </button>
        </div>
    </div>
</div>

{#if showRecoveryKeyModal}
<div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
    <div class="bg-white rounded-lg max-w-lg w-full p-6 m-4">
        <div class="flex items-center text-red-600 mb-4">
            <svg class="w-8 h-8 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"></path></svg>
            <h2 class="text-xl font-bold">Peringatan: Simpan Recovery Key</h2>
        </div>
        
        <p class="text-gray-700 mb-4">
            Simpan recovery key ini dengan aman. <strong>Tanpa recovery key, backup terenkripsi Anda tidak akan pernah bisa dipulihkan.</strong> Tim Support kami tidak memiliki salinan kunci ini dan tidak dapat membantu Anda jika kunci hilang.
        </p>

        <div class="bg-gray-100 p-4 rounded mb-6 font-mono text-sm break-all border border-gray-300">
            {recoveryKey}
        </div>

        <label class="flex items-start mb-6 space-x-3 cursor-pointer">
            <input type="checkbox" bind:checked={savedKeyConfirmation} class="mt-1 w-5 h-5 text-blue-600 border-gray-300 rounded" />
            <span class="font-medium text-gray-700 text-sm">Saya mengonfirmasi bahwa saya telah menyalin dan menyimpan Recovery Key ini di tempat yang aman.</span>
        </label>

        <div class="flex justify-end">
            <button on:click={() => showRecoveryKeyModal = false} class="mr-3 px-4 py-2 text-gray-600 hover:text-gray-800 font-medium">Batal</button>
            <button on:click={confirmKeySaved} disabled={!savedKeyConfirmation} class="bg-red-600 hover:bg-red-700 disabled:bg-red-300 text-white font-bold py-2 px-6 rounded shadow">
                Lanjutkan Backup
            </button>
        </div>
    </div>
</div>
{/if}
