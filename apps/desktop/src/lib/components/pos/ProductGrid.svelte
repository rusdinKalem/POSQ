<script lang="ts">
    export type Product = {
        id: string;
        name: string;
        sku: string;
        price: number;
        qty_on_hand: number;
        image_url?: string | null;
        category_name?: string | null;
        category_id?: string | null;
    };

    export type CategoryItem = {
        id: string;
        name: string;
    };

    interface Props {
        filteredProducts: Product[];
        mainCategories: CategoryItem[];
        subCategories: CategoryItem[];
        selectedMainCategory: string;
        selectedSubCategory: string;
        onAddToCart: (product: Product) => void;
    }

    let {
        filteredProducts,
        mainCategories,
        subCategories,
        selectedMainCategory = $bindable(),
        selectedSubCategory = $bindable(),
        onAddToCart
    }: Props = $props();
</script>

<div class="product-grid-container flex-1 flex flex-col overflow-hidden">
    <!-- Categories Slider -->
    <div class="space-y-2 mb-4 shrink-0">
        <!-- Main Categories Row -->
        <div class="flex gap-2 overflow-x-auto pb-1 scrollbar-none">
            {#each mainCategories as cat}
                <button 
                    type="button"
                    onclick={() => {
                        selectedMainCategory = cat.id;
                        selectedSubCategory = "Semua";
                    }}
                    class="px-4 py-2.5 rounded-xl text-xs font-bold whitespace-nowrap transition cursor-pointer shadow-xs border h-12 flex items-center justify-center min-w-[48px]"
                    class:bg-blue-600={selectedMainCategory === cat.id}
                    class:text-white={selectedMainCategory === cat.id}
                    class:border-blue-600={selectedMainCategory === cat.id}
                    class:bg-white={selectedMainCategory !== cat.id}
                    class:text-slate-600={selectedMainCategory !== cat.id}
                    class:border-slate-200={selectedMainCategory !== cat.id}
                >
                    {cat.name}
                </button>
            {/each}
        </div>

        <!-- Sub Categories Row (if available) -->
        {#if subCategories.length > 0}
            <div class="flex gap-2 overflow-x-auto pb-1 scrollbar-none bg-slate-100/60 p-1.5 rounded-xl border border-slate-200/60 animate-in fade-in slide-in-from-top-1 duration-200">
                {#each subCategories as subCat}
                    <button 
                        type="button"
                        onclick={() => selectedSubCategory = subCat.id}
                        class="px-3.5 py-2 rounded-lg text-xs font-bold whitespace-nowrap transition cursor-pointer h-10 flex items-center justify-center min-w-[48px]"
                        class:bg-slate-800={selectedSubCategory === subCat.id}
                        class:text-white={selectedSubCategory === subCat.id}
                        class:bg-white={selectedSubCategory !== subCat.id}
                        class:text-slate-600={selectedSubCategory !== subCat.id}
                        class:shadow-2xs={selectedSubCategory !== subCat.id}
                    >
                        {subCat.name}
                    </button>
                {/each}
            </div>
        {/if}
    </div>

    <!-- Products Grid -->
    <div class="flex-1 overflow-y-auto min-h-0">
        <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-4 pb-6">
            {#each filteredProducts as p (p.id)}
                <button 
                    type="button"
                    class="card flex flex-col items-start bg-white p-3.5 rounded-2xl border border-slate-200 hover:border-blue-500 hover:shadow-md active:scale-[0.98] transition-all text-left cursor-pointer relative min-h-[140px]" 
                    onclick={() => onAddToCart(p)}>
                    
                    <div class="w-full h-28 bg-slate-50 flex items-center justify-center rounded-xl overflow-hidden mb-3 border border-slate-100 shrink-0">
                        {#if p.image_url}
                            <img src={p.image_url} class="w-full h-full object-cover" alt={p.name} />
                        {:else}
                            <div class="w-full h-full bg-blue-50 text-blue-600 font-black flex items-center justify-center text-2xl uppercase">
                                {p.name.substring(0, 2)}
                            </div>
                        {/if}
                    </div>

                    <span class="text-[10px] font-bold text-slate-400 uppercase tracking-wider block mb-0.5">{p.category_name || 'Umum'}</span>
                    <div class="text-sm font-bold text-slate-800 line-clamp-2 leading-tight mb-1 h-9">{p.name}</div>
                    <div class="text-[10px] font-mono text-slate-400 mb-2">{p.sku}</div>

                    <div class="w-full flex justify-between items-center mt-auto">
                        <span class="text-blue-600 font-extrabold text-sm">Rp {p.price.toLocaleString('id-ID')}</span>
                        <span 
                            class="text-[9px] font-bold px-2 py-0.5 rounded-full"
                            class:bg-red-50={p.qty_on_hand < 5}
                            class:text-red-600={p.qty_on_hand < 5}
                            class:bg-green-50={p.qty_on_hand >= 5}
                            class:text-green-600={p.qty_on_hand >= 5}>
                            Stok: {p.qty_on_hand}
                        </span>
                    </div>
                </button>
            {/each}
            
            {#if filteredProducts.length === 0}
                <div class="col-span-full flex flex-col items-center justify-center py-20 text-slate-400 bg-white rounded-2xl border border-slate-200">
                    <span class="text-4xl mb-2">🔍</span>
                    <span class="font-bold text-sm">Tidak ada produk ditemukan.</span>
                    <span class="text-xs text-slate-400 mt-1">Coba gunakan kata kunci lain atau pilih kategori "Semua".</span>
                </div>
            {/if}
        </div>
    </div>
</div>
