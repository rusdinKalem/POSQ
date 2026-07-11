<script lang="ts">
    import '../app.css';
    import { onMount } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';
    import { page } from '$app/stores';

    let licenseMode = $state("Checking...");

    async function checkLicense() {
        try {
            const res: any = await invoke('verify_license');
            licenseMode = res.mode;
        } catch (err) {
            console.error("License check failed:", err);
            licenseMode = "Error";
        }
    }

    onMount(() => {
        checkLicense();
    });

    // Re-check license on route change
    $effect(() => {
        const path = $page.url.pathname;
        checkLicense();
    });
</script>

<div class="app-container font-sans bg-slate-50 min-h-screen flex flex-col">
    <!-- License Banners -->
    {#if licenseMode === "RestrictedExpired"}
        <div class="bg-rose-600 text-white py-2.5 px-4 text-center font-bold text-xs shadow-md relative z-50 flex items-center justify-center gap-2">
            <span>⚠️</span>
            <span>LISENSI KADALUARSA (RESTRICTED MODE)</span>
            <span class="font-normal opacity-90 hidden sm:inline">Pembuatan transaksi ditangguhkan.</span>
            <a href="/settings/license" class="underline font-semibold hover:text-rose-100 ml-1">Perbarui Lisensi</a>
        </div>
    {/if}

    {#if licenseMode === "Unlicensed"}
        <div class="bg-blue-600 text-white py-2.5 px-4 text-center font-bold text-xs shadow-md relative z-50 flex items-center justify-center gap-2">
            <span>ℹ️</span>
            <span>PERANGKAT BELUM DIAKTIVASI</span>
            <span class="font-normal opacity-90 hidden sm:inline">Aktifkan kasir ini untuk mengakses menu.</span>
            <a href="/settings/license" class="underline font-semibold hover:text-blue-100 ml-1">Pengaturan Lisensi</a>
        </div>
    {/if}

    <!-- Shared Navigation Header -->
    <header class="bg-white border-b border-slate-200 sticky top-0 z-30 shadow-xs">
        <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 h-16 flex items-center justify-between">
            <!-- Brand Logo -->
            <a href="/" class="flex items-center gap-2.5 no-underline">
                <div class="h-9 w-9 rounded-lg bg-blue-600 flex items-center justify-center text-white font-black text-lg shadow-md shadow-blue-500/10">
                    P
                </div>
                <span class="text-xl font-bold tracking-tight text-slate-900">POSQ</span>
            </a>

            <!-- Nav Links -->
            <nav class="hidden md:flex items-center gap-1">
                <a href="/" class="px-3 py-1.5 rounded-lg text-xs font-bold transition-colors {$page.url.pathname === '/' ? 'bg-blue-50 text-blue-600' : 'text-slate-600 hover:text-slate-950 hover:bg-slate-50'}">
                    🖥️ PoC Dashboard
                </a>
                <a href="/pos" class="px-3 py-1.5 rounded-lg text-xs font-bold transition-colors {$page.url.pathname.startsWith('/pos') ? 'bg-blue-50 text-blue-600' : 'text-slate-600 hover:text-slate-950 hover:bg-slate-50'}">
                    🛒 Transaksi POS
                </a>
                <a href="/shift" class="px-3 py-1.5 rounded-lg text-xs font-bold transition-colors {$page.url.pathname.startsWith('/shift') ? 'bg-blue-50 text-blue-600' : 'text-slate-600 hover:text-slate-950 hover:bg-slate-50'}">
                    🔑 Shift
                </a>
                <a href="/inventory" class="px-3 py-1.5 rounded-lg text-xs font-bold transition-colors {$page.url.pathname.startsWith('/inventory') ? 'bg-blue-50 text-blue-600' : 'text-slate-600 hover:text-slate-950 hover:bg-slate-50'}">
                    🗃️ Inventaris
                </a>
                <a href="/reports" class="px-3 py-1.5 rounded-lg text-xs font-bold transition-colors {$page.url.pathname.startsWith('/reports') ? 'bg-blue-50 text-blue-600' : 'text-slate-600 hover:text-slate-950 hover:bg-slate-50'}">
                    📊 Laporan
                </a>
                <a href="/audit" class="px-3 py-1.5 rounded-lg text-xs font-bold transition-colors {$page.url.pathname.startsWith('/audit') ? 'bg-blue-50 text-blue-600' : 'text-slate-600 hover:text-slate-950 hover:bg-slate-50'}">
                    📋 Audit Logs
                </a>
                <a href="/settings/license" class="px-3 py-1.5 rounded-lg text-xs font-bold transition-colors {$page.url.pathname.includes('/settings/license') ? 'bg-blue-50 text-blue-600' : 'text-slate-600 hover:text-slate-950 hover:bg-slate-50'}">
                    ⚙️ Lisensi
                </a>
            </nav>
            
            <!-- Mobile indicator -->
            <div class="md:hidden text-xs bg-slate-100 text-slate-600 px-2 py-1 rounded font-bold">
                Mobile View
            </div>
        </div>
    </header>
    
    <div class="flex-grow flex flex-col relative">
        <!-- Route Blocker overlay -->
        {#if (licenseMode === 'RestrictedExpired' || licenseMode === 'Unlicensed') && !$page.url.pathname.includes('/settings/license')}
            <div class="absolute inset-0 z-40 bg-white/70 backdrop-blur-xs flex flex-col items-center justify-center p-6">
                <div class="bg-white p-8 rounded-2xl shadow-xl border border-rose-100 text-center max-w-md w-full">
                    <div class="h-12 w-12 bg-rose-50 text-rose-600 rounded-full flex items-center justify-center text-xl mx-auto mb-4 font-bold">
                        🔒
                    </div>
                    <h2 class="text-2xl font-bold text-slate-800 mb-2">Akses Diblokir</h2>
                    <p class="text-slate-500 text-sm mb-6">Operasional dibekukan karena status lisensi Anda. Silakan aktifkan lisensi untuk membuka fitur kasir.</p>
                    <a href="/settings/license" class="inline-flex items-center justify-center bg-blue-600 hover:bg-blue-700 text-white font-bold py-3 px-6 rounded-xl shadow-md transition-colors cursor-pointer w-full text-sm">
                        Buka Pengaturan Lisensi
                    </a>
                </div>
            </div>
        {/if}

        <slot />
    </div>
</div>

<style>
    .app-container {
        min-height: 100vh;
    }
</style>
