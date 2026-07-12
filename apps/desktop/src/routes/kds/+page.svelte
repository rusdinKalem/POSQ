<script lang="ts">
    import { invoke } from '@tauri-apps/api/core';
    import { onMount, onDestroy } from 'svelte';
    import { showToast } from '$lib/toast.svelte';

    interface KdsItem {
        name: string;
        qty: number;
        notes?: string;
    }

    interface KdsTicket {
        id: string;
        reference_id: string;
        reference_type: 'draft' | 'order';
        order_number: string;
        table_number: string | null;
        order_type: string;
        status: 'pending' | 'cooking' | 'done';
        items_json: string;
        created_at: string;
        updated_at: string;
        // Client-side additions
        parsedItems?: KdsItem[];
        elapsedTime?: string;
        severity?: 'normal' | 'warning' | 'danger';
    }

    let activeTickets = $state<KdsTicket[]>([]);
    let completedTickets = $state<KdsTicket[]>([]);
    let currentTab = $state<'active' | 'completed'>('active');
    let orderTypeFilter = $state<string>('all');
    let now = $state<number>(Date.now());
    let filteredTickets = $derived((currentTab === 'active' ? activeTickets : completedTickets)
        .filter(t => orderTypeFilter === 'all' || t.order_type === orderTypeFilter));

    let pollInterval: ReturnType<typeof setInterval>;
    let clockInterval: ReturnType<typeof setInterval>;
    let lastTicketCount = 0;

    function playKitchenAlert() {
        try {
            const ctx = new (window.AudioContext || (window as any).webkitAudioContext)();
            
            // Double beep
            const playBeep = (time: number, freq: number) => {
                const osc = ctx.createOscillator();
                const gain = ctx.createGain();
                osc.type = 'sine';
                osc.frequency.setValueAtTime(freq, time);
                gain.gain.setValueAtTime(0.15, time);
                gain.gain.exponentialRampToValueAtTime(0.01, time + 0.15);
                osc.connect(gain);
                gain.connect(ctx.destination);
                osc.start(time);
                osc.stop(time + 0.15);
            };

            playBeep(ctx.currentTime, 880);
            playBeep(ctx.currentTime + 0.2, 1200);
        } catch (e) {
            console.error("Audio Context alert failed:", e);
        }
    }

    async function fetchTickets() {
        try {
            if (currentTab === 'active') {
                const tickets: KdsTicket[] = await invoke('get_active_kds_tickets');
                
                // Parse items and handle alerts
                const parsed = tickets.map(t => {
                    let items: KdsItem[] = [];
                    try {
                        items = JSON.parse(t.items_json);
                    } catch (e) {
                        console.error("Error parsing items_json:", e);
                    }
                    return {
                        ...t,
                        parsedItems: items
                    };
                });

                // Check if new tickets arrived to alert kitchen staff
                if (parsed.length > lastTicketCount && lastTicketCount > 0) {
                    playKitchenAlert();
                    showToast('Ada pesanan baru masuk!', 'success');
                }
                lastTicketCount = parsed.length;
                activeTickets = parsed;
            } else {
                const tickets: KdsTicket[] = await invoke('get_completed_kds_tickets');
                completedTickets = tickets.map(t => {
                    let items: KdsItem[] = [];
                    try {
                        items = JSON.parse(t.items_json);
                    } catch (e) {
                        console.error("Error parsing items_json:", e);
                    }
                    return {
                        ...t,
                        parsedItems: items
                    };
                });
            }
        } catch (err) {
            console.error("Failed to fetch KDS tickets:", err);
        }
    }

    async function updateStatus(id: string, newStatus: 'pending' | 'cooking' | 'done') {
        try {
            await invoke('update_kds_ticket_status', { id, status: newStatus });
            showToast(`Status pesanan diperbarui`, 'success');
            await fetchTickets();
        } catch (err) {
            showToast(`Gagal memperbarui status: ${err}`, 'error');
        }
    }

    // Format elapsed time in mm:ss and determine warning severity color
    function updateTicketTimers() {
        now = Date.now();
        activeTickets = activeTickets.map(ticket => {
            // SQLite CURRENT_TIMESTAMP is UTC. Convert SQLite timestamp to Local Time
            // Format is usually YYYY-MM-DD HH:MM:SS
            const cleanTimestamp = ticket.created_at.replace(' ', 'T') + 'Z';
            const createdTime = new Date(cleanTimestamp).getTime();
            const diffMs = now - createdTime;
            const diffSeconds = Math.max(0, Math.floor(diffMs / 1000));
            const mins = Math.floor(diffSeconds / 60);
            const secs = diffSeconds % 60;
            const elapsedTime = `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
            
            let severity: 'normal' | 'warning' | 'danger' = 'normal';
            if (mins >= 10) {
                severity = 'danger';
            } else if (mins >= 5) {
                severity = 'warning';
            }

            return {
                ...ticket,
                elapsedTime,
                severity
            };
        });
    }

    onMount(() => {
        fetchTickets();
        // Poll database every 5 seconds
        pollInterval = setInterval(fetchTickets, 5000);
        // Refresh timer rendering every second
        clockInterval = setInterval(() => {
            if (currentTab === 'active') {
                updateTicketTimers();
            }
        }, 1000);
    });

    onDestroy(() => {
        clearInterval(pollInterval);
        clearInterval(clockInterval);
    });

    // Reactive handling of tab changes
    $effect(() => {
        fetchTickets();
    });

    // Helper for styling order type badge
    function getOrderTypeLabel(type: string): string {
        switch (type) {
            case 'dine_in': return 'DINE IN';
            case 'takeaway': return 'TAKE AWAY';
            case 'delivery': return 'DELIVERY';
            default: return type.toUpperCase();
        }
    }
</script>

<div class="flex-grow flex flex-col min-h-0 bg-slate-900 text-slate-100 font-sans">
    <!-- Top Control Bar -->
    <header class="bg-slate-800 border-b border-slate-700 py-4 px-6 flex flex-col md:flex-row md:items-center md:justify-between gap-4 shrink-0 shadow-lg">
        <div class="flex items-center gap-4">
            <div class="h-10 w-10 bg-emerald-500 rounded-xl flex items-center justify-center text-white font-bold text-lg shadow-md shadow-emerald-500/20">
                🍳
            </div>
            <div>
                <h1 class="text-xl font-bold tracking-tight text-white">Kitchen Display System (KDS)</h1>
                <p class="text-xs text-slate-400">Pantauan pesanan dapur real-time</p>
            </div>
        </div>

        <div class="flex items-center gap-3">
            <!-- Tabs -->
            <div class="bg-slate-950 p-1.5 rounded-xl border border-slate-700/50 flex">
                <button 
                    type="button" 
                    class="px-4 py-2 text-xs font-bold rounded-lg transition-all cursor-pointer flex items-center gap-2 {currentTab === 'active' ? 'bg-slate-800 text-white shadow-xs' : 'text-slate-400 hover:text-slate-200'}"
                    onclick={() => currentTab = 'active'}>
                    🟢 Pesanan Aktif
                    {#if activeTickets.length > 0}
                        <span class="bg-red-500 text-white font-extrabold text-[10px] px-2 py-0.5 rounded-full min-w-5 text-center animate-pulse">
                            {activeTickets.length}
                        </span>
                    {/if}
                </button>
                <button 
                    type="button" 
                    class="px-4 py-2 text-xs font-bold rounded-lg transition-all cursor-pointer {currentTab === 'completed' ? 'bg-slate-800 text-white shadow-xs' : 'text-slate-400 hover:text-slate-200'}"
                    onclick={() => currentTab = 'completed'}>
                    ✓ Riwayat Selesai
                </button>
            </div>

            <!-- Filters -->
            <select 
                bind:value={orderTypeFilter}
                class="bg-slate-950 border border-slate-700 text-slate-200 text-xs font-semibold rounded-xl px-3 py-2 focus:ring-2 focus:ring-emerald-500/40 focus:border-emerald-500 transition outline-hidden cursor-pointer">
                <option value="all">Semua Tipe</option>
                <option value="dine_in">Dine In</option>
                <option value="takeaway">Take Away</option>
                <option value="delivery">Delivery</option>
            </select>
        </div>
    </header>

    <!-- Main Board Grid -->
    <main class="flex-grow overflow-y-auto p-6">
        {#if filteredTickets.length === 0}
            <div class="h-full flex flex-col items-center justify-center text-slate-500 py-20">
                <div class="text-5xl mb-4">🍽️</div>
                <h3 class="text-lg font-bold text-slate-400">Tidak ada tiket pesanan</h3>
                <p class="text-sm text-slate-500 mt-1">Pesanan baru akan muncul secara otomatis di sini</p>
            </div>
        {:else}
            <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6">
                {#each filteredTickets as ticket (ticket.id)}
                    <!-- Ticket Card -->
                    <div class="flex flex-col bg-slate-800 border border-slate-700 rounded-2xl shadow-xl hover:shadow-2xl transition duration-200 overflow-hidden relative">
                        <!-- Card Border/Accent Color based on timer severity -->
                        {#if currentTab === 'active'}
                            <div class="h-1.5 w-full {ticket.severity === 'danger' ? 'bg-red-500 animate-pulse' : ticket.severity === 'warning' ? 'bg-amber-500' : 'bg-emerald-500'}"></div>
                        {/if}

                        <!-- Header -->
                        <div class="p-4 bg-slate-800/80 border-b border-slate-700/60 flex items-start justify-between gap-2">
                            <div>
                                <div class="flex items-center gap-2">
                                    <span class="text-xs font-black px-2 py-0.5 rounded-md tracking-wider bg-slate-700/80 text-slate-300">
                                        {getOrderTypeLabel(ticket.order_type)}
                                    </span>
                                    {#if ticket.reference_type === 'draft'}
                                        <span class="text-[10px] font-bold px-1.5 py-0.5 rounded-sm bg-yellow-500/10 text-yellow-400 border border-yellow-500/20">
                                            DRAFT
                                        </span>
                                    {:else}
                                        <span class="text-[10px] font-bold px-1.5 py-0.5 rounded-sm bg-blue-500/10 text-blue-400 border border-blue-500/20">
                                            PAID
                                        </span>
                                    {/if}
                                </div>
                                <h3 class="text-base font-bold text-white mt-1.5 truncate max-w-[150px]">
                                    {ticket.table_number ? `Meja ${ticket.table_number}` : ticket.order_number}
                                </h3>
                                {#if ticket.table_number}
                                    <p class="text-[10px] text-slate-500 mt-0.5 truncate max-w-[130px]">{ticket.order_number}</p>
                                {/if}
                            </div>

                            {#if currentTab === 'active'}
                                <div class="text-right flex flex-col items-end">
                                    <span class="font-mono text-sm font-bold {ticket.severity === 'danger' ? 'text-red-400 animate-pulse' : ticket.severity === 'warning' ? 'text-amber-400' : 'text-slate-300'}">
                                        ⏱️ {ticket.elapsedTime || '00:00'}
                                    </span>
                                    <span class="text-[9px] text-slate-500 mt-1 uppercase font-semibold">
                                        {ticket.status === 'cooking' ? '🔥 Memasak' : '⏳ Menunggu'}
                                    </span>
                                </div>
                            {/if}
                        </div>

                        <!-- Items List -->
                        <div class="flex-grow p-4 space-y-3 overflow-y-auto max-h-60">
                            {#if ticket.parsedItems && ticket.parsedItems.length > 0}
                                {#each ticket.parsedItems as item}
                                    <div class="flex items-start justify-between gap-3 text-sm">
                                        <div class="flex-grow">
                                            <span class="text-white font-bold text-base mr-2">{item.qty}x</span>
                                            <span class="text-slate-200 font-semibold">{item.name}</span>
                                            {#if item.notes}
                                                <div class="mt-1 bg-amber-950/40 border border-amber-900/30 text-amber-300 text-xs px-2.5 py-1.5 rounded-lg font-medium italic flex items-center gap-1.5">
                                                    <span>💡</span>
                                                    <span>{item.notes}</span>
                                                </div>
                                            {/if}
                                        </div>
                                    </div>
                                {/each}
                            {:else}
                                <p class="text-xs text-slate-500 italic">Tidak ada detail menu</p>
                            {/if}
                        </div>

                        <!-- Action Footer -->
                        <div class="p-4 bg-slate-850/50 border-t border-slate-700/40 flex items-center justify-between gap-2">
                            {#if currentTab === 'active'}
                                {#if ticket.status === 'pending'}
                                    <button 
                                        type="button" 
                                        class="w-full bg-blue-600 hover:bg-blue-700 active:scale-95 text-white text-xs font-bold py-2.5 px-4 rounded-xl transition duration-150 cursor-pointer flex items-center justify-center gap-2"
                                        onclick={() => updateStatus(ticket.id, 'cooking')}>
                                        🍳 Mulai Masak
                                    </button>
                                {:else if ticket.status === 'cooking'}
                                    <button 
                                        type="button" 
                                        class="w-full bg-emerald-600 hover:bg-emerald-700 active:scale-95 text-white text-xs font-bold py-2.5 px-4 rounded-xl transition duration-150 cursor-pointer flex items-center justify-center gap-2"
                                        onclick={() => updateStatus(ticket.id, 'done')}>
                                        ✅ Selesai Saji
                                    </button>
                                {/if}
                            {:else}
                                <!-- Recall completed ticket back to cooking status -->
                                <button 
                                    type="button" 
                                    class="w-full bg-slate-700 hover:bg-slate-600 text-slate-200 text-xs font-semibold py-2 px-3 rounded-xl transition cursor-pointer flex items-center justify-center gap-1.5"
                                    onclick={() => updateStatus(ticket.id, 'cooking')}>
                                    ↩️ Panggil Kembali
                                </button>
                            {/if}
                        </div>
                    </div>
                {/each}
            </div>
        {/if}
    </main>
</div>
