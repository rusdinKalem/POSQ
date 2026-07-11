<script lang="ts">
    import { onMount } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';

    let shiftActive = false;
    let shiftId: string | null = null;
    let countedCash: number = 0;
    
    onMount(async () => {
        try {
            const status: any = await invoke('check_active_shift');
            if (status.active) {
                shiftActive = true;
                shiftId = status.shift_id;
            }
        } catch (e) {
            console.error('Error checking shift:', e);
        }
    });

    async function closeShift() {
        if (!shiftId) return;
        
        try {
            await invoke('close_shift', { 
                shiftId: shiftId,
                countedCash: countedCash || 0
            });
            alert('Shift berhasil ditutup!');
            shiftActive = false;
            shiftId = null;
        } catch (e) {
            alert('Gagal menutup shift: ' + e);
        }
    }
</script>

<div class="p-8 h-screen bg-gray-100 flex flex-col items-center">
    <div class="card w-full max-w-md p-6 glassmorphism">
        <h1 class="text-2xl font-bold mb-4">Manajemen Shift</h1>
        
        {#if shiftActive}
            <div class="alert alert-success mb-4 p-4 rounded text-white bg-green-500 font-bold">
                Shift Sedang Aktif
            </div>
            
            <p class="text-gray-600 mb-4">
                Masukkan jumlah uang tunai aktual (Counted Cash) yang ada di laci kasir saat ini untuk mengakhiri shift.
            </p>
            
            <label class="block font-bold mb-2" for="countedCash">Uang Fisik Dihitung (Rp):</label>
            <input id="countedCash" type="number" bind:value={countedCash} class="w-full p-3 border rounded mb-4 text-xl text-center" placeholder="0" />
            
            <button class="btn-danger w-full p-4 font-bold text-lg" on:click={closeShift}>
                Tutup Shift (End of Day)
            </button>
        {:else}
            <div class="alert alert-warning mb-4 p-4 rounded bg-yellow-100 text-yellow-800 font-bold border border-yellow-300">
                Tidak ada shift yang aktif
            </div>
            <p class="text-gray-600">
                Silakan buka shift melalui halaman kasir (POS) untuk memulai transaksi hari ini.
            </p>
            <a href="/pos" class="btn-primary block w-full p-4 text-center font-bold text-lg mt-4" style="text-decoration: none;">
                Ke Halaman Kasir
            </a>
        {/if}
    </div>
</div>

<style>
    .btn-danger {
        background-color: var(--danger-color);
        color: white;
        border: none;
        border-radius: var(--radius-md);
        cursor: pointer;
        transition: transform 0.2s;
    }
    .btn-danger:hover {
        transform: translateY(-2px);
    }
</style>
