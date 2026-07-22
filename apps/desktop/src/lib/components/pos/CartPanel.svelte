<script lang="ts">
    import CartItemRow, { type CartItem } from './CartItemRow.svelte';

    export type Customer = {
        id: string;
        name: string;
        phone: string;
    };

    interface Props {
        cart: CartItem[];
        businessMode: string;
        orderType: string;
        tableNumber: string;
        selectedCustomer: Customer;
        subtotal: number;
        tax: number;
        grandTotal: number;
        activeCartDiscount: number;
        activeTaxRate: number;
        isMobile: boolean;
        activeTab: string;
        onIncreaseQty: (productId: string) => void;
        onDecreaseQty: (productId: string) => void;
        onRemoveFromCart: (productId: string) => void;
        onOpenItemCustomizer: (item: CartItem) => void;
        onOpenItemDiscount: (productId: string) => void;
        onOpenCustomerModal: () => void;
        onOpenResourceModal: () => void;
        onOpenCartDiscountModal: () => void;
        onClearCart: () => void;
        onHoldBill: () => void;
        onOpenPayment: () => void;
    }

    let {
        cart,
        businessMode,
        orderType = $bindable(),
        tableNumber = $bindable(),
        selectedCustomer,
        subtotal,
        tax,
        grandTotal,
        activeCartDiscount,
        activeTaxRate,
        isMobile,
        activeTab,
        onIncreaseQty,
        onDecreaseQty,
        onRemoveFromCart,
        onOpenItemCustomizer,
        onOpenItemDiscount,
        onOpenCustomerModal,
        onOpenResourceModal,
        onOpenCartDiscountModal,
        onClearCart,
        onHoldBill,
        onOpenPayment
    }: Props = $props();

    let showClearConfirm = $state(false);
</script>

<div class="cart-section bg-white border-l border-slate-200 flex flex-col shadow-xl shrink-0" 
     class:w-full={isMobile}
     class:md:w-[400px]={!isMobile}
     class:!hidden={isMobile && activeTab !== 'cart'}>
    
    <!-- Cart Header -->
    <div class="p-4 border-b border-slate-100 flex justify-between items-center bg-slate-50/50 shrink-0">
        <div>
            <h2 class="text-base font-black text-slate-800 flex items-center gap-1.5">
                🛒 Detail Keranjang
            </h2>
            <div class="text-xs font-bold text-slate-500">
                Pelanggan: <button type="button" class="text-blue-600 hover:underline cursor-pointer" onclick={onOpenCustomerModal}>{selectedCustomer.name}</button>
            </div>
        </div>
        {#if cart.length > 0}
            {#if showClearConfirm}
                <div class="flex items-center gap-1">
                    <span class="text-[10px] text-red-600 font-bold">Kosongkan?</span>
                    <button 
                        type="button" 
                        class="text-[10px] font-bold text-white bg-red-600 hover:bg-red-700 px-2 py-1 rounded-lg transition cursor-pointer"
                        onclick={() => {
                            showClearConfirm = false;
                            onClearCart();
                        }}>
                        Ya
                    </button>
                    <button 
                        type="button" 
                        class="text-[10px] font-bold text-slate-700 bg-slate-200 hover:bg-slate-300 px-2 py-1 rounded-lg transition cursor-pointer"
                        onclick={() => showClearConfirm = false}>
                        Batal
                    </button>
                </div>
            {:else}
                <button 
                    type="button" 
                    class="text-[11px] font-bold text-red-600 hover:text-red-700 bg-red-50 hover:bg-red-100 px-2.5 h-9 rounded-lg transition cursor-pointer flex items-center gap-1"
                    onclick={() => showClearConfirm = true}>
                    Kosongkan
                </button>
            {/if}
        {/if}
    </div>

    <!-- Order Options Segment & Table Selection (F&B Only) -->
    {#if businessMode === 'fb'}
        <div class="px-4 py-3 border-b border-slate-100 bg-slate-50/50 space-y-2 shrink-0">
            <div class="grid grid-cols-3 gap-1 bg-slate-100 p-1 rounded-xl">
                <button 
                    type="button" 
                    class="py-2 text-xs font-bold rounded-lg transition h-10 flex items-center justify-center cursor-pointer min-w-[48px]"
                    class:bg-white={orderType === 'dine_in'}
                    class:shadow-2xs={orderType === 'dine_in'}
                    class:text-blue-600={orderType === 'dine_in'}
                    class:text-slate-500={orderType !== 'dine_in'}
                    onclick={() => orderType = 'dine_in'}>
                    Dine In
                </button>
                <button 
                    type="button" 
                    class="py-2 text-xs font-bold rounded-lg transition h-10 flex items-center justify-center cursor-pointer min-w-[48px]"
                    class:bg-white={orderType === 'takeaway'}
                    class:shadow-2xs={orderType === 'takeaway'}
                    class:text-blue-600={orderType === 'takeaway'}
                    class:text-slate-500={orderType !== 'takeaway'}
                    onclick={() => orderType = 'takeaway'}>
                    Take Away
                </button>
                <button 
                    type="button" 
                    class="py-2 text-xs font-bold rounded-lg transition h-10 flex items-center justify-center cursor-pointer min-w-[48px]"
                    class:bg-white={orderType === 'delivery'}
                    class:shadow-2xs={orderType === 'delivery'}
                    class:text-blue-600={orderType === 'delivery'}
                    class:text-slate-500={orderType !== 'delivery'}
                    onclick={() => orderType = 'delivery'}>
                    Delivery
                </button>
            </div>

            {#if orderType === 'dine_in'}
                <div class="flex items-center justify-between py-2 bg-white px-3.5 rounded-xl border border-slate-200/60 shadow-2xs">
                    <span class="text-xs text-slate-500 font-bold">Meja:</span>
                    <div class="flex items-center gap-1.5">
                        <button type="button" class="text-xs text-blue-600 font-extrabold hover:underline cursor-pointer min-h-[36px] flex items-center" onclick={onOpenResourceModal}>
                            {tableNumber || 'Pilih Meja...'}
                        </button>
                        {#if tableNumber}
                            <button type="button" class="text-lg text-red-500 hover:text-red-700 font-bold font-mono cursor-pointer leading-none px-1.5 py-1" onclick={() => tableNumber = ""}>×</button>
                        {/if}
                    </div>
                </div>
            {/if}
        </div>
    {:else if businessMode === 'jasa'}
        <div class="px-4 py-3 border-b border-slate-100 bg-slate-50/50 space-y-2 shrink-0">
            <div class="flex items-center justify-between py-2 bg-white px-3.5 rounded-xl border border-slate-200/60 shadow-2xs">
                <span class="text-xs text-slate-500 font-bold">Teknisi / Terapis:</span>
                <div class="flex items-center gap-1.5">
                    <button type="button" class="text-xs text-blue-600 font-extrabold hover:underline cursor-pointer min-h-[36px] flex items-center" onclick={onOpenResourceModal}>
                        {tableNumber || 'Pilih Teknisi...'}
                    </button>
                    {#if tableNumber}
                        <button type="button" class="text-lg text-red-500 hover:text-red-700 font-bold font-mono cursor-pointer leading-none px-1.5 py-1" onclick={() => tableNumber = ""}>×</button>
                    {/if}
                </div>
            </div>
        </div>
    {/if}

    <!-- Cart List -->
    <div class="flex-1 overflow-y-auto p-4 space-y-3">
        {#each cart as item (item.product_id)}
            <CartItemRow 
                {item}
                onIncrease={onIncreaseQty}
                onDecrease={onDecreaseQty}
                onRemove={onRemoveFromCart}
                onOpenCustomizer={onOpenItemCustomizer}
                onOpenDiscount={onOpenItemDiscount}
            />
        {/each}

        {#if cart.length === 0}
            <div class="flex flex-col items-center justify-center h-full min-h-[250px] text-slate-400 py-12">
                <span class="text-4xl mb-2">🛒</span>
                <span class="text-xs font-bold text-slate-400">Keranjang masih kosong</span>
                <span class="text-[11px] text-slate-400 mt-1 text-center">Pilih produk dari katalog di sebelah kiri untuk memulai pesanan.</span>
            </div>
        {/if}
    </div>

    <!-- Totals & Hold Buttons -->
    <div class="p-4 bg-slate-50 border-t border-slate-200 shrink-0">
        <div class="space-y-2 text-xs text-slate-600 mb-4">
            <div class="flex justify-between">
                <span>Subtotal</span> 
                <span class="font-semibold text-slate-800">Rp {subtotal.toLocaleString('id-ID')}</span>
            </div>
            
            <div class="flex justify-between items-center">
                <span class="flex items-center gap-1">
                    Diskon Transaksi 
                    <button type="button" class="text-blue-600 hover:underline cursor-pointer" onclick={onOpenCartDiscountModal}>[Ubah]</button>
                </span>
                {#if activeCartDiscount > 0}
                    <span class="font-semibold text-green-600">-Rp {activeCartDiscount.toLocaleString('id-ID')}</span>
                {:else}
                    <span class="font-semibold text-slate-400">Rp 0</span>
                {/if}
            </div>

            <div class="flex justify-between">
                <span>Pajak ({activeTaxRate}%)</span> 
                <span class="font-semibold text-slate-800">Rp {tax.toLocaleString('id-ID')}</span>
            </div>

            <div class="flex justify-between font-black text-base text-slate-800 pt-2 border-t border-slate-200/60">
                <span>Total Tagihan</span> 
                <span class="text-blue-600">Rp {grandTotal.toLocaleString('id-ID')}</span>
            </div>
        </div>

        <!-- Hold order and Checkout buttons -->
        <div class="grid grid-cols-2 gap-2">
            <button 
                type="button"
                onclick={onHoldBill}
                disabled={cart.length === 0}
                class="bg-slate-200 hover:bg-slate-300 active:bg-slate-400 disabled:bg-slate-100 disabled:text-slate-400 text-slate-800 font-bold rounded-xl transition text-xs flex items-center justify-center gap-1 cursor-pointer h-12 min-w-[48px]">
                💾 {businessMode === 'fb' ? 'Simpan Pesanan' : businessMode === 'jasa' ? 'Tunda Layanan' : 'Simpan Antrean'}
            </button>
            <button 
                type="button"
                onclick={onOpenPayment} 
                disabled={cart.length === 0}
                class="bg-blue-600 hover:bg-blue-700 active:bg-blue-800 disabled:bg-slate-200 disabled:text-slate-400 text-white font-extrabold rounded-xl transition text-xs flex items-center justify-center gap-1 shadow-md cursor-pointer h-12 min-w-[48px]">
                💳 Bayar (Rp {grandTotal.toLocaleString('id-ID')})
            </button>
        </div>
    </div>
</div>
