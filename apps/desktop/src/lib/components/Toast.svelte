<script lang="ts">
    import { toastState } from '$lib/toast.svelte';

    const colorMap: Record<string, string> = {
        success: 'bg-emerald-600 border-emerald-400 text-white',
        error: 'bg-red-600 border-red-400 text-white',
        warning: 'bg-amber-500 border-amber-300 text-slate-950',
        info: 'bg-blue-600 border-blue-400 text-white',
    };
    const iconMap: Record<string, string> = {
        success: '✅',
        error: '❌',
        warning: '⚠️',
        info: 'ℹ️',
    };
</script>

{#if toastState.show}
<div class="fixed top-6 left-1/2 -translate-x-1/2 z-[9999] animate-in fade-in slide-in-from-top-4 duration-300 shadow-2xl">
    <div class="flex items-center gap-3 px-6 py-4 rounded-2xl shadow-xl border transition-all
        {colorMap[toastState.type] ?? colorMap.success}">
        <span class="text-xl">{iconMap[toastState.type] ?? '✅'}</span>
        <div class="text-sm font-bold tracking-wide">{toastState.message}</div>
    </div>
</div>
{/if}
