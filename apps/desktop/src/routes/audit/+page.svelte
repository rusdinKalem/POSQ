<script lang="ts">
    import { onMount } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';

    type AuditLog = {
        id: string;
        action: string;
        target_type: string;
        reason: string | null;
        created_at: string;
    };

    let logs: AuditLog[] = [];
    let errorMsg = '';

    onMount(async () => {
        try {
            logs = await invoke('get_audit_logs');
        } catch (e: any) {
            errorMsg = e.toString();
        }
    });
</script>

<div class="p-8 h-screen bg-gray-100 flex flex-col">
    <div class="flex justify-between items-center mb-6">
        <h1 class="text-2xl font-bold">Sistem Audit Log (Security)</h1>
        <a href="/pos" class="btn-outline px-4 py-2" style="text-decoration: none;">Kembali ke POS</a>
    </div>

    {#if errorMsg}
        <div class="alert alert-danger mb-4 p-4 rounded bg-red-100 text-red-800 font-bold border border-red-300">
            {errorMsg}
        </div>
    {:else}
        <div class="card flex-1 glassmorphism overflow-hidden flex flex-col">
            <div class="overflow-y-auto flex-1 p-0">
                <table class="w-full text-left border-collapse">
                    <thead class="bg-gray-50 border-b">
                        <tr>
                            <th class="p-4 font-bold text-gray-700">Waktu</th>
                            <th class="p-4 font-bold text-gray-700">Aksi</th>
                            <th class="p-4 font-bold text-gray-700">Target</th>
                            <th class="p-4 font-bold text-gray-700">Keterangan</th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each logs as log}
                            <tr class="border-b hover:bg-gray-50 transition-colors">
                                <td class="p-4 text-sm whitespace-nowrap">
                                    {new Date(log.created_at).toLocaleString('id-ID')}
                                </td>
                                <td class="p-4 font-bold" class:text-danger={log.action === 'refund_order'}>
                                    {log.action}
                                </td>
                                <td class="p-4">
                                    <span class="bg-gray-200 px-2 py-1 rounded text-xs">{log.target_type}</span>
                                </td>
                                <td class="p-4 text-gray-600 text-sm">
                                    {log.reason || '-'}
                                </td>
                            </tr>
                        {/each}
                        {#if logs.length === 0}
                            <tr>
                                <td colspan="4" class="p-8 text-center text-gray-500">
                                    Belum ada jejak audit.
                                </td>
                            </tr>
                        {/if}
                    </tbody>
                </table>
            </div>
        </div>
    {/if}
</div>

<style>
    th {
        position: sticky;
        top: 0;
        background-color: #f8fafc;
        z-index: 10;
    }
</style>
