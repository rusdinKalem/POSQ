<script lang="ts">
    export type CartItem = {
        product_id: string;
        name: string;
        sku: string;
        qty: number;
        unit_price: number;
        discount_total: number;
        line_total: number;
        notes?: string;
        variant_name?: string;
        modifiers?: string[];
        category_name?: string | null;
        category_id?: string | null;
    };

    interface Props {
        item: CartItem;
        onIncrease: (productId: string) => void;
        onDecrease: (productId: string) => void;
        onRemove: (productId: string) => void;
        onOpenCustomizer: (item: CartItem) => void;
        onOpenDiscount: (productId: string) => void;
    }

    let {
        item,
        onIncrease,
        onDecrease,
        onRemove,
        onOpenCustomizer,
        onOpenDiscount
    }: Props = $props();

    let showConfirmDelete = $state(false);
</script>

<div class="cart-item flex flex-col gap-2 p-3 bg-slate-50 border border-slate-200/60 rounded-xl hover:border-slate-300 transition">
    <div class="flex justify-between items-start gap-2">
        <div class="flex-1">
            <div class="font-bold text-slate-800 text-sm leading-snug">{item.name}</div>
            {#if item.variant_name || (item.modifiers && item.modifiers.length > 0)}
                <div class="text-[10px] text-slate-500 font-semibold mt-0.5">
                    {item.variant_name || 'Regular'} 
                    {#if item.modifiers && item.modifiers.length > 0}
                        (+ {item.modifiers.join(', ')})
                    {/if}
                </div>
            {/if}
            {#if item.notes}
                <div class="text-[10px] text-slate-400 italic mt-0.5">Note: "{item.notes}"</div>
            {/if}
            <div class="flex items-center gap-1.5 mt-1">
                <span class="text-[10px] text-slate-400 font-mono">Rp {item.unit_price.toLocaleString('id-ID')}</span>
                {#if item.discount_total > 0}
                    <span class="text-[9px] font-bold text-green-600 bg-green-50 px-1.5 py-0.5 rounded border border-green-200">
                        Diskon: -Rp {item.discount_total.toLocaleString('id-ID')}
                    </span>
                {/if}
            </div>
        </div>
        <div class="text-right shrink-0">
            <div class="font-bold text-slate-800 text-sm">Rp {item.line_total.toLocaleString('id-ID')}</div>
        </div>
    </div>

    <!-- Actions Bar inside Cart Item -->
    <div class="flex justify-between items-center mt-1 pt-2 border-t border-slate-200/40">
        <div class="flex items-center gap-1.5">
            <!-- Customize / Modifiers -->
            <button 
                type="button" 
                onclick={() => onOpenCustomizer(item)}
                class="text-[11px] font-bold text-slate-600 hover:text-blue-600 active:bg-blue-50 bg-white border border-slate-200 px-2.5 h-9 rounded-lg transition cursor-pointer flex items-center gap-1">
                ⚙️ Custom
            </button>
            <!-- Item Discount -->
            <button 
                type="button" 
                onclick={() => onOpenDiscount(item.product_id)}
                class="text-[11px] font-bold text-slate-600 hover:text-green-600 active:bg-green-50 bg-white border border-slate-200 px-2.5 h-9 rounded-lg transition cursor-pointer flex items-center gap-1">
                🏷️ Diskon
            </button>
        </div>

        <!-- Quantity Selector with touch target min 48x48px -->
        {#if showConfirmDelete}
            <div class="flex items-center gap-1.5 animate-in fade-in duration-150">
                <span class="text-[10px] font-bold text-red-600">Hapus?</span>
                <button 
                    type="button"
                    onclick={() => onRemove(item.product_id)}
                    class="h-10 px-3 bg-red-600 text-white font-bold text-xs rounded-lg hover:bg-red-700 active:bg-red-800 transition cursor-pointer flex items-center justify-center">
                    Ya
                </button>
                <button 
                    type="button"
                    onclick={() => showConfirmDelete = false}
                    class="h-10 px-2.5 bg-slate-200 text-slate-700 font-bold text-xs rounded-lg hover:bg-slate-300 transition cursor-pointer flex items-center justify-center">
                    Batal
                </button>
            </div>
        {:else}
            <div class="flex items-center gap-1.5">
                <button 
                    type="button" 
                    class="w-12 h-12 flex items-center justify-center bg-white border border-slate-200 rounded-xl text-slate-800 hover:bg-slate-100 active:bg-slate-200 transition shadow-2xs cursor-pointer font-black text-lg select-none" 
                    onclick={() => {
                        if (item.qty === 1) {
                            showConfirmDelete = true;
                        } else {
                            onDecrease(item.product_id);
                        }
                    }}>
                    -
                </button>
                <span class="w-7 text-center font-black text-slate-800 text-base">{item.qty}</span>
                <button 
                    type="button" 
                    class="w-12 h-12 flex items-center justify-center bg-white border border-slate-200 rounded-xl text-slate-800 hover:bg-slate-100 active:bg-slate-200 transition shadow-2xs cursor-pointer font-black text-lg select-none" 
                    onclick={() => onIncrease(item.product_id)}>
                    +
                </button>
            </div>
        {/if}
    </div>
</div>
