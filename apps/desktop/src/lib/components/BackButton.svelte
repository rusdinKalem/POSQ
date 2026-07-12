<script lang="ts">
    interface Props {
        to?: string;
        label?: string;
        class?: string;
    }
    let { to = "/settings?tab=system", label = "Kembali ke Pengaturan Sistem & Perangkat Keras", class: className = "mb-4" }: Props = $props();
</script>

<a 
    href={to} 
    class="inline-flex items-center gap-2 px-3 py-1.5 rounded-lg text-sm font-semibold text-slate-500 hover:text-slate-800 hover:bg-slate-100 transition-all w-fit cursor-pointer {className}"
>
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" class="text-slate-500">
        <path d="m15 18-6-6 6-6"/>
    </svg>
    <span>{label}</span>
</a>
