<script lang="ts">
    import { onMount } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';
    import { showToast } from '$lib/toast.svelte';

    type AuditLog = {
        id: string;
        action: string;
        target_type: string;
        reason: string | null;
        created_at: string;
    };

    type FraudAlert = {
        id: string;
        rule_id: string;
        description: string;
        severity: 'INFO' | 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL';
        status: 'OPEN' | 'REVIEWING' | 'FALSE_POSITIVE' | 'CONFIRMED' | 'RESOLVED';
        supporting_data: any;
        cashier_name: string;
        outlet_name: string;
        created_at: string;
    };

    let activeTab = $state<'audit' | 'fraud'>('audit');
    let logs = $state<AuditLog[]>([]);
    let alerts = $state<FraudAlert[]>([]);
    let errorMsg = $state('');
    let runningChecks = $state(false);

    onMount(async () => {
        await loadAll();
    });

    async function loadAll() {
        errorMsg = '';
        try {
            logs = await invoke<AuditLog[]>('get_audit_logs');
            const fraudRes: any = await invoke('get_fraud_alerts');
            alerts = fraudRes.alerts || [];
        } catch (e: any) {
            errorMsg = e.toString();
        }
    }

    async function triggerFraudChecks() {
        runningChecks = true;
        try {
            const count = await invoke<number>('run_fraud_checks');
            showToast(`Deteksi fraud selesai. Ditemukan ${count} alert baru.`, 'success');
            await loadAll();
        } catch (e: any) {
            showToast('Gagal memicu deteksi fraud: ' + e, 'error');
        } finally {
            runningChecks = false;
        }
    }

    async function handleResolve(alertId: string, status: 'RESOLVED' | 'FALSE_POSITIVE') {
        try {
            await invoke('resolve_fraud_alert', { alertId, status });
            showToast(`Status alert berhasil diperbarui menjadi ${status}`, 'success');
            await loadAll();
        } catch (e: any) {
            showToast('Gagal memperbarui alert: ' + e, 'error');
        }
    }

    const severityColors = {
        INFO: 'bg-blue-500/10 border-blue-500/20 text-blue-400',
        LOW: 'bg-slate-500/10 border-slate-500/20 text-slate-400',
        MEDIUM: 'bg-amber-500/10 border-amber-500/20 text-amber-400',
        HIGH: 'bg-orange-500/10 border-orange-500/20 text-orange-400',
        CRITICAL: 'bg-red-500/10 border-red-500/20 text-red-400',
    };

    const statusColors = {
        OPEN: 'bg-red-500/10 text-red-400 border-red-500/20',
        REVIEWING: 'bg-amber-500/10 text-amber-400 border-amber-500/20',
        FALSE_POSITIVE: 'bg-slate-500/20 text-slate-400 border-slate-700',
        CONFIRMED: 'bg-red-600/20 text-red-300 border-red-600/30',
        RESOLVED: 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20',
    };
</script>

<div class="min-h-screen bg-slate-950 p-6 flex flex-col gap-6 text-slate-100">
    <!-- Header -->
    <div class="border-b border-slate-800 pb-4 flex justify-between items-center">
        <div>
            <h1 class="text-3xl font-black tracking-wide text-white flex items-center gap-2">
                🛡️ Pusat Keamanan & Audit Forensik
            </h1>
            <p class="text-sm text-slate-400">
                Pemantauan jejak audit immutable, mitigasi fraud, dan status otorisasi supervisor.
            </p>
        </div>
        <div class="flex gap-3">
            <button 
                type="button"
                onclick={triggerFraudChecks}
                disabled={runningChecks}
                class="px-5 py-2.5 bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white font-bold rounded-xl text-sm transition-all flex items-center gap-2"
            >
                {#if runningChecks}
                <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
                {:else}
                🔍 Run Fraud Checks
                {/if}
            </button>
            <a 
                href="/pos" 
                class="px-5 py-2.5 bg-slate-800 hover:bg-slate-700 text-white font-bold rounded-xl text-sm transition-colors border border-slate-700"
            >
                Kembali ke POS
            </a>
        </div>
    </div>

    {#if errorMsg}
        <div class="rounded-xl border border-red-500/20 bg-red-500/10 p-4 text-center text-sm font-semibold text-red-400">
            ❌ Gagal memuat data keamanan: {errorMsg}
        </div>
    {:else}
        <!-- Tabs Nav -->
        <div class="flex gap-4 border-b border-slate-800 pb-px">
            <button 
                type="button"
                onclick={() => activeTab = 'audit'}
                class="pb-3 text-sm font-bold transition-all relative
                    {activeTab === 'audit' ? 'text-indigo-400 border-b-2 border-indigo-500 font-extrabold' : 'text-slate-400 hover:text-white'}"
            >
                Jejak Audit (Audit Log)
            </button>
            <button 
                type="button"
                onclick={() => activeTab = 'fraud'}
                class="pb-3 text-sm font-bold transition-all relative flex items-center gap-2
                    {activeTab === 'fraud' ? 'text-indigo-400 border-b-2 border-indigo-500 font-extrabold' : 'text-slate-400 hover:text-white'}"
            >
                Security & Fraud Alerts 
                {#if alerts.filter(a => a.status === 'OPEN').length > 0}
                <span class="px-2 py-0.5 text-[10px] font-black bg-red-600 text-white rounded-full">
                    {alerts.filter(a => a.status === 'OPEN').length}
                </span>
                {/if}
            </button>
        </div>

        <!-- Tab Content -->
        <div class="flex-1 bg-slate-900 border border-slate-800 rounded-3xl p-6 shadow-xl flex flex-col">
            {#if activeTab === 'audit'}
                <div class="overflow-x-auto flex-1">
                    <table class="w-full border-collapse text-left">
                        <thead>
                            <tr class="border-b border-slate-800 text-xs font-bold text-slate-400 uppercase tracking-wider">
                                <th class="pb-3 px-2">Waktu</th>
                                <th class="pb-3 px-2">Tindakan/Aksi</th>
                                <th class="pb-3 px-2">Kategori/Target</th>
                                <th class="pb-3 px-2">Keterangan / Alasan Otorisasi</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-slate-800/50 text-xs">
                            {#if logs.length === 0}
                                <tr>
                                    <td colspan="4" class="py-8 text-center text-slate-500 font-medium">
                                        Belum ada data jejak audit terekam.
                                    </td>
                                </tr>
                            {:else}
                                {#each logs as log}
                                    <tr class="hover:bg-slate-950/20 transition-colors">
                                        <td class="py-3.5 px-2 text-slate-400 font-medium whitespace-nowrap">
                                            {new Date(log.created_at).toLocaleString('id-ID')}
                                        </td>
                                        <td class="py-3.5 px-2 font-bold">
                                            {#if log.action.includes('void') || log.action.includes('reverse')}
                                                <span class="text-red-400 font-semibold">{log.action}</span>
                                            {:else if log.action.includes('refund')}
                                                <span class="text-amber-400 font-semibold">{log.action}</span>
                                            {:else if log.action.includes('approve') || log.action.includes('open')}
                                                <span class="text-emerald-400 font-semibold">{log.action}</span>
                                            {:else}
                                                <span class="text-slate-300">{log.action}</span>
                                            {/if}
                                        </td>
                                        <td class="py-3.5 px-2">
                                            <span class="rounded-lg bg-slate-950 border border-slate-800 px-2 py-1 font-mono text-[10px] text-slate-400">
                                                {log.target_type}
                                            </span>
                                        </td>
                                        <td class="py-3.5 px-2 text-slate-300 font-mono">
                                            {log.reason || '-'}
                                        </td>
                                    </tr>
                                {/each}
                            {/if}
                        </tbody>
                    </table>
                </div>
            {:else if activeTab === 'fraud'}
                <div class="flex flex-col gap-4">
                    {#if alerts.length === 0}
                        <div class="py-12 text-center text-slate-500 font-medium border border-dashed border-slate-850 rounded-2xl">
                            ✅ Bersih! Tidak ada indikasi kecurangan (fraud alerts) saat ini.
                        </div>
                    {:else}
                        <div class="grid grid-cols-1 gap-4">
                            {#each alerts as alert}
                                <div class="bg-slate-950 border border-slate-800 rounded-2xl p-5 flex flex-col md:flex-row justify-between gap-4 transition-all hover:border-slate-700">
                                    <div class="flex flex-col gap-2.5">
                                        <div class="flex items-center gap-2 flex-wrap">
                                            <span class="px-2.5 py-0.5 rounded-lg border font-black text-[10px] tracking-wider
                                                {severityColors[alert.severity] || severityColors.INFO}">
                                                {alert.severity}
                                            </span>
                                            <span class="px-2.5 py-0.5 rounded-lg border font-bold text-[10px]
                                                {statusColors[alert.status] || statusColors.OPEN}">
                                                {alert.status}
                                            </span>
                                            <span class="text-[10px] text-slate-500">
                                                {new Date(alert.created_at).toLocaleString('id-ID')}
                                            </span>
                                        </div>
                                        <div>
                                            <h3 class="text-sm font-black text-white">{alert.description}</h3>
                                            <p class="text-xs text-slate-400 mt-1">
                                                Kasir: <span class="font-bold text-slate-300">{alert.cashier_name}</span> | Outlet: <span class="font-bold text-slate-300">{alert.outlet_name}</span>
                                            </p>
                                        </div>
                                        {#if alert.supporting_data && Object.keys(alert.supporting_data).length > 0}
                                            <div class="mt-1 bg-slate-900 border border-slate-800/50 rounded-xl p-3">
                                                <div class="text-[10px] font-bold uppercase tracking-wider text-slate-500 mb-1.5">Data Pendukung:</div>
                                                <div class="grid grid-cols-2 sm:grid-cols-3 gap-2 text-[10px] font-mono text-slate-300">
                                                    {#each Object.entries(alert.supporting_data) as [k, v]}
                                                        <div>
                                                            <span class="text-slate-500">{k}:</span> {JSON.stringify(v)}
                                                        </div>
                                                    {/each}
                                                </div>
                                            </div>
                                        {/if}
                                    </div>
                                    <div class="flex items-center md:items-end gap-2 md:self-end">
                                        {#if alert.status === 'OPEN'}
                                            <button 
                                                type="button"
                                                onclick={() => handleResolve(alert.id, 'RESOLVED')}
                                                class="px-3.5 py-2 bg-emerald-600 hover:bg-emerald-500 text-white font-bold rounded-xl text-xs transition-colors"
                                            >
                                                Mark Resolved
                                            </button>
                                            <button 
                                                type="button"
                                                onclick={() => handleResolve(alert.id, 'FALSE_POSITIVE')}
                                                class="px-3.5 py-2 bg-slate-800 hover:bg-slate-700 text-slate-300 font-bold rounded-xl text-xs transition-colors border border-slate-700"
                                            >
                                                False Positive
                                            </button>
                                        {/if}
                                    </div>
                                </div>
                            {/each}
                        </div>
                    {/if}
                </div>
            {/if}
        </div>
    {/if}
</div>
