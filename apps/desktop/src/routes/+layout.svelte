<script lang="ts">
    import { onMount } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';
    import { page } from '$app/stores';
    import { goto } from '$app/navigation';
    import { refreshSession, logout, authState } from '$lib/auth.svelte';
    import Toast from '$lib/components/Toast.svelte';
    import '../app.css';

    let { children } = $props();

    let licenseMode = $state("Checking...");
    let mobileMenuOpen = $state(false);

    async function checkLicense() {
        try {
            const res: any = await invoke('verify_license');
            licenseMode = res.mode;
        } catch (err) {
            console.error("License check failed:", err);
            licenseMode = "Error";
        }
    }

    onMount(async () => {
        checkLicense();
        await refreshSession();
        // Guard if not logged in
        if (!authState.session && $page.url.pathname !== '/login') {
            goto('/login');
        }
    });

    // Guard on route changes and state updates
    $effect(() => {
        const path = $page.url.pathname;
        if (!authState.loading) {
            if (!authState.session && path !== '/login') {
                goto('/login');
            } else if (authState.session && path === '/login') {
                goto('/');
            }
        }
    });

    // Re-check license on route change
    $effect(() => {
        const path = $page.url.pathname;
        checkLicense();
    });
</script>

{#if authState.loading}
    <div class="min-h-screen bg-slate-900 flex items-center justify-center">
        <div class="flex flex-col items-center gap-4">
            <div class="w-12 h-12 border-4 border-blue-500 border-t-transparent rounded-full animate-spin"></div>
            <span class="text-sm font-bold text-slate-400 uppercase tracking-widest">Memuat POSQ...</span>
        </div>
    </div>
{:else}
    <div class="app-container font-sans bg-slate-50 min-h-screen flex flex-col">
        <!-- Global UI Elements -->
        <Toast />

        <!-- License Banners -->
        {#if licenseMode === "RestrictedExpired" && $page.url.pathname !== '/login'}
            <div class="bg-rose-600 text-white py-2.5 px-4 text-center font-bold text-xs shadow-md relative z-50 flex items-center justify-center gap-2">
                <span>⚠️</span>
                <span>LISENSI KADALUARSA (RESTRICTED MODE)</span>
                <span class="font-normal opacity-90 hidden sm:inline">Pembuatan transaksi ditangguhkan.</span>
                <a href="/settings/license" class="underline font-semibold hover:text-rose-100 ml-1">Perbarui Lisensi</a>
            </div>
        {/if}

        {#if licenseMode === "Unlicensed" && $page.url.pathname !== '/login'}
            <div class="bg-blue-600 text-white py-2.5 px-4 text-center font-bold text-xs shadow-md relative z-50 flex items-center justify-center gap-2">
                <span>ℹ️</span>
                <span>PERANGKAT BELUM DIAKTIVASI</span>
                <span class="font-normal opacity-90 hidden sm:inline">Aktifkan kasir ini untuk mengakses menu.</span>
                <a href="/settings/license" class="underline font-semibold hover:text-blue-100 ml-1">Pengaturan Lisensi</a>
            </div>
        {/if}

        <!-- Shared Navigation Header - only show if not on login page -->
        {#if $page.url.pathname !== '/login'}
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
                    <nav class="hidden lg:flex items-center gap-1">
                        <a href="/" class="px-3 py-1.5 rounded-lg text-xs font-bold transition-colors {$page.url.pathname === '/' ? 'bg-blue-50 text-blue-600' : 'text-slate-600 hover:text-slate-950 hover:bg-slate-50'}">
                            🖥️ PoC
                        </a>
                        <a href="/pos" class="px-3 py-1.5 rounded-lg text-xs font-bold transition-colors {$page.url.pathname.startsWith('/pos') ? 'bg-blue-50 text-blue-600' : 'text-slate-600 hover:text-slate-950 hover:bg-slate-50'}">
                            🛒 POS
                        </a>
                        <a href="/kds" class="px-3 py-1.5 rounded-lg text-xs font-bold transition-colors {$page.url.pathname.startsWith('/kds') ? 'bg-blue-50 text-blue-600' : 'text-slate-600 hover:text-slate-950 hover:bg-slate-50'}">
                            🍳 KDS
                        </a>
                        <a href="/tables" class="px-3 py-1.5 rounded-lg text-xs font-bold transition-colors {$page.url.pathname.startsWith('/tables') ? 'bg-blue-50 text-blue-600' : 'text-slate-600 hover:text-slate-950 hover:bg-slate-50'}">
                            🍽️ Meja F&B
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
                            📋 Audit
                        </a>
                        <a href="/settings" class="px-3 py-1.5 rounded-lg text-xs font-bold transition-colors {$page.url.pathname === '/settings' || $page.url.pathname.startsWith('/settings/') ? 'bg-blue-50 text-blue-600' : 'text-slate-600 hover:text-slate-950 hover:bg-slate-50'}">
                            ⚙️ Pengaturan
                        </a>
                    </nav>
                    
                    <!-- Right Actions: User Status & Logout -->
                    <div class="flex items-center gap-4">
                        {#if authState.session}
                            <div class="hidden md:flex items-center gap-3 border-l border-slate-200 pl-4">
                                <div class="flex flex-col text-right">
                                    <span class="text-xs font-bold text-slate-800">{authState.session.user_name}</span>
                                    <span class="text-[10px] text-slate-500 font-bold capitalize tracking-wider">{authState.session.roles.join(', ')}</span>
                                </div>
                                <button 
                                    type="button" 
                                    onclick={async () => {
                                        await logout();
                                        goto('/login');
                                    }}
                                    class="px-3 py-1.5 bg-slate-50 hover:bg-rose-50 text-slate-600 hover:text-rose-600 border border-slate-200 hover:border-rose-100 rounded-lg text-xs font-bold transition-all cursor-pointer"
                                >
                                    Keluar
                                </button>
                            </div>
                        {/if}

                        <!-- Mobile Menu Toggle Button (Touch target 48x48px) -->
                        <button 
                            type="button" 
                            class="lg:hidden w-12 h-12 flex items-center justify-center rounded-xl bg-slate-50 hover:bg-slate-100 text-slate-700 font-extrabold text-xl border border-slate-200 transition cursor-pointer"
                            onclick={() => mobileMenuOpen = !mobileMenuOpen}
                            aria-label="Toggle Navigation Menu">
                            {#if mobileMenuOpen}
                                ✕
                            {:else}
                                ☰
                            {/if}
                        </button>
                    </div>
                </div>

                <!-- Mobile Navigation Drawer Overlay & Content -->
                {#if mobileMenuOpen}
                    <div 
                        class="lg:hidden fixed inset-0 top-16 bg-slate-900/40 backdrop-blur-xs z-40 transition-opacity" 
                        onclick={() => mobileMenuOpen = false}
                        role="presentation">
                    </div>
                    <div class="lg:hidden absolute top-16 left-0 right-0 z-50 bg-white border-b border-slate-200 shadow-xl p-4 flex flex-col gap-1.5 animate-in slide-in-from-top duration-200">
                        <a href="/" onclick={() => mobileMenuOpen = false} class="flex items-center gap-3 px-4 py-3 rounded-xl text-sm font-bold transition-colors {$page.url.pathname === '/' ? 'bg-blue-50 text-blue-600' : 'text-slate-600 hover:text-slate-950 hover:bg-slate-50'} min-h-[48px]">
                            <span class="text-base">🖥️</span> <span>PoC Dashboard</span>
                        </a>
                        <a href="/pos" onclick={() => mobileMenuOpen = false} class="flex items-center gap-3 px-4 py-3 rounded-xl text-sm font-bold transition-colors {$page.url.pathname.startsWith('/pos') ? 'bg-blue-50 text-blue-600' : 'text-slate-600 hover:text-slate-950 hover:bg-slate-50'} min-h-[48px]">
                            <span class="text-base">🛒</span> <span>Transaksi POS</span>
                        </a>
                        <a href="/kds" onclick={() => mobileMenuOpen = false} class="flex items-center gap-3 px-4 py-3 rounded-xl text-sm font-bold transition-colors {$page.url.pathname.startsWith('/kds') ? 'bg-blue-50 text-blue-600' : 'text-slate-600 hover:text-slate-950 hover:bg-slate-50'} min-h-[48px]">
                            <span class="text-base">🍳</span> <span>KDS (Dapur)</span>
                        </a>
                        <a href="/tables" onclick={() => mobileMenuOpen = false} class="flex items-center gap-3 px-4 py-3 rounded-xl text-sm font-bold transition-colors {$page.url.pathname.startsWith('/tables') ? 'bg-blue-50 text-blue-600' : 'text-slate-600 hover:text-slate-950 hover:bg-slate-50'} min-h-[48px]">
                            <span class="text-base">🍽️</span> <span>Meja F&B</span>
                        </a>
                        <a href="/shift" onclick={() => mobileMenuOpen = false} class="flex items-center gap-3 px-4 py-3 rounded-xl text-sm font-bold transition-colors {$page.url.pathname.startsWith('/shift') ? 'bg-blue-50 text-blue-600' : 'text-slate-600 hover:text-slate-950 hover:bg-slate-50'} min-h-[48px]">
                            <span class="text-base">🔑</span> <span>Shift</span>
                        </a>
                        <a href="/inventory" onclick={() => mobileMenuOpen = false} class="flex items-center gap-3 px-4 py-3 rounded-xl text-sm font-bold transition-colors {$page.url.pathname.startsWith('/inventory') ? 'bg-blue-50 text-blue-600' : 'text-slate-600 hover:text-slate-950 hover:bg-slate-50'} min-h-[48px]">
                            <span class="text-base">🗃️</span> <span>Inventaris</span>
                        </a>
                        <a href="/reports" onclick={() => mobileMenuOpen = false} class="flex items-center gap-3 px-4 py-3 rounded-xl text-sm font-bold transition-colors {$page.url.pathname.startsWith('/reports') ? 'bg-blue-50 text-blue-600' : 'text-slate-600 hover:text-slate-950 hover:bg-slate-50'} min-h-[48px]">
                            <span class="text-base">📊</span> <span>Laporan</span>
                        </a>
                        <a href="/audit" onclick={() => mobileMenuOpen = false} class="flex items-center gap-3 px-4 py-3 rounded-xl text-sm font-bold transition-colors {$page.url.pathname.startsWith('/audit') ? 'bg-blue-50 text-blue-600' : 'text-slate-600 hover:text-slate-950 hover:bg-slate-50'} min-h-[48px]">
                            <span class="text-base">📋</span> <span>Audit Logs</span>
                        </a>
                        <a href="/settings" onclick={() => mobileMenuOpen = false} class="flex items-center gap-3 px-4 py-3 rounded-xl text-sm font-bold transition-colors {$page.url.pathname === '/settings' || $page.url.pathname.startsWith('/settings/') ? 'bg-blue-50 text-blue-600' : 'text-slate-600 hover:text-slate-950 hover:bg-slate-50'} min-h-[48px]">
                            <span class="text-base">⚙️</span> <span>Pengaturan</span>
                        </a>

                        {#if authState.session}
                            <div class="border-t border-slate-100 pt-4 mt-2 flex items-center justify-between px-4">
                                <div class="flex flex-col">
                                    <span class="text-xs font-bold text-slate-800">{authState.session.user_name}</span>
                                    <span class="text-[10px] text-slate-500 font-semibold capitalize">{authState.session.roles.join(', ')}</span>
                                </div>
                                <button 
                                    type="button" 
                                    onclick={async () => {
                                        mobileMenuOpen = false;
                                        await logout();
                                        goto('/login');
                                    }}
                                    class="px-3.5 py-2 bg-rose-50 text-rose-600 font-bold rounded-xl text-xs transition cursor-pointer"
                                >
                                    Keluar
                                </button>
                            </div>
                        {/if}
                    </div>
                {/if}
            </header>
        {/if}
        
        <div class="flex-grow flex flex-col relative">
            <!-- Route Blocker overlay -->
            {#if (licenseMode === 'RestrictedExpired' || licenseMode === 'Unlicensed') && !$page.url.pathname.startsWith('/settings') && $page.url.pathname !== '/login'}
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

            {@render children()}
        </div>
    </div>
{/if}
