<script lang="ts">
    import { onMount } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';
    import { goto } from '$app/navigation';

    type Product = {
        id: string;
        name: string;
        sku: string;
        price: number;
        qty_on_hand: number;
        image_url?: string | null;
    };

    type CartItem = {
        product_id: string;
        name: string;
        sku: string;
        qty: number;
        unit_price: number;
        discount_total: number;
        line_total: number;
    };

    let products: Product[] = [];
    let cart: CartItem[] = [];
    let shiftActive: boolean = true;
    let shiftId: string | null = null;
    let showShiftModal: boolean = false;
    let startingCash: number = 0;
    
    let showPaymentModal: boolean = false;
    let amountPaid: number = 0;
    
    let subtotal: number = 0;
    let tax: number = 0;
    let discount: number = 0;
    let grandTotal: number = 0;

    onMount(async () => {
        await checkShift();
        if (shiftActive) {
            await fetchProducts();
        }
    });

    async function checkShift() {
        try {
            const status: any = await invoke('check_active_shift');
            if (status.active) {
                shiftActive = true;
                shiftId = status.shift_id;
            } else {
                shiftActive = false;
                showShiftModal = true;
            }
        } catch (e) {
            console.error('Error checking shift:', e);
        }
    }

    async function openShift() {
        try {
            shiftId = await invoke('open_shift', { startingCash: startingCash || 0 });
            shiftActive = true;
            showShiftModal = false;
            await fetchProducts();
        } catch (e) {
            alert('Failed to open shift: ' + e);
        }
    }

    async function fetchProducts() {
        try {
            products = await invoke('get_products');
        } catch (e) {
            console.error('Failed to load products:', e);
        }
    }

    function addToCart(product: Product) {
        let existing = cart.find(i => i.product_id === product.id);
        if (existing) {
            if (existing.qty >= product.qty_on_hand) {
                alert('Stock tidak cukup!');
                return;
            }
            existing.qty += 1;
            existing.line_total = existing.qty * existing.unit_price;
            cart = [...cart];
        } else {
            if (product.qty_on_hand < 1) {
                alert('Stok habis!');
                return;
            }
            cart = [...cart, {
                product_id: product.id,
                sku: product.sku,
                name: product.name,
                qty: 1,
                unit_price: product.price,
                discount_total: 0,
                line_total: product.price
            }];
        }
        calculateCart();
    }

    function calculateCart() {
        subtotal = cart.reduce((sum, item) => sum + item.line_total, 0);
        tax = Math.round(subtotal * 0.11); // 11% tax
        grandTotal = subtotal + tax - discount;
    }

    function openPayment() {
        if (cart.length === 0) return;
        amountPaid = grandTotal;
        showPaymentModal = true;
    }

    async function submitPayment() {
        if (amountPaid < grandTotal) {
            alert('Pembayaran kurang!');
            return;
        }
        
        let change = amountPaid - grandTotal;
        
        const payload = {
            shift_id: shiftId,
            subtotal,
            discount_total: discount,
            tax_total: tax,
            service_total: 0,
            grand_total: grandTotal,
            paid_total: amountPaid,
            change_total: change,
            payment_method: 'CASH',
            items: cart
        };

        try {
            let orderId = await invoke('checkout', { payload });
            alert('Pembayaran Berhasil!\nKembalian: Rp ' + change);
            goto(`/receipt?order_id=${orderId}`);
        } catch (e) {
            alert('Checkout failed: ' + e);
        }
    }
</script>

<div class="pos-container flex">
    <!-- PRODUCT GRID -->
    <div class="product-section w-full p-4">
        <h1 class="text-2xl font-bold mb-4">Kasir POSQ</h1>
        <div class="grid grid-products">
            {#each products as p}
                <button class="card product-card flex flex-col items-center glassmorphism p-4 hover:scale-[1.02] transition-transform duration-200" on:click={() => addToCart(p)}>
                    <div class="w-full h-28 bg-slate-100 flex items-center justify-center rounded-lg overflow-hidden mb-3 border">
                        {#if p.image_url}
                            <img src={p.image_url} class="w-full h-full object-cover" alt={p.name} />
                        {:else}
                            <div class="w-full h-full bg-slate-200 text-slate-400 font-bold flex items-center justify-center text-3xl">
                                {p.name.charAt(0).toUpperCase()}
                            </div>
                        {/if}
                    </div>
                    <div class="text-sm font-bold text-slate-800 line-clamp-1">{p.name}</div>
                    <div class="text-slate-500 text-xs mt-0.5 font-mono">{p.sku}</div>
                    <div class="text-blue-600 font-bold mt-2 text-sm">Rp {p.price.toLocaleString('id-ID')}</div>
                    <div class="text-xs mt-2 font-semibold" class:text-red-500={p.qty_on_hand < 5} class:text-slate-500={p.qty_on_hand >= 5}>{p.qty_on_hand} in stock</div>
                </button>
            {/each}
            {#if products.length === 0}
                <div class="text-gray">Tidak ada produk ditemukan.</div>
            {/if}
        </div>
    </div>

    <!-- CART SIDEBAR -->
    <div class="cart-section p-4 glassmorphism">
        <h2 class="text-xl font-bold mb-4">Keranjang</h2>
        <div class="cart-items">
            {#each cart as item}
                <div class="cart-item flex justify-between mb-2 p-2">
                    <div>
                        <div class="font-bold">{item.name}</div>
                        <div class="text-sm text-gray">{item.qty} x Rp {item.unit_price}</div>
                    </div>
                    <div class="font-bold">Rp {item.line_total}</div>
                </div>
            {/each}
        </div>

        <div class="cart-totals mt-4">
            <div class="flex justify-between mb-2"><span>Subtotal</span> <span>Rp {subtotal}</span></div>
            <div class="flex justify-between mb-2"><span>Pajak (11%)</span> <span>Rp {tax}</span></div>
            <div class="flex justify-between font-bold text-xl mt-4"><span>Total</span> <span>Rp {grandTotal}</span></div>
        </div>

        <button class="btn-primary w-full mt-4 p-4 text-lg" on:click={openPayment} disabled={cart.length === 0}>
            Bayar (Rp {grandTotal})
        </button>
    </div>
</div>

<!-- SHIFT MODAL -->
{#if showShiftModal}
<div class="modal-backdrop">
    <div class="modal-content text-center">
        <h2 class="text-2xl font-bold mb-4">Buka Shift Kasir</h2>
        <p class="mb-4 text-gray">Masukkan saldo awal uang tunai di laci untuk memulai penjualan.</p>
        <input type="number" bind:value={startingCash} class="mb-4 text-xl p-4 text-center" placeholder="Rp 0" />
        <button class="btn-success w-full p-4 text-lg" on:click={openShift}>Mulai Shift</button>
    </div>
</div>
{/if}

<!-- PAYMENT MODAL -->
{#if showPaymentModal}
<div class="modal-backdrop">
    <div class="modal-content text-center">
        <h2 class="text-2xl font-bold mb-4">Pembayaran Cash</h2>
        <div class="mb-4 text-gray">Total Belanja</div>
        <div class="text-3xl font-bold text-primary mb-4">Rp {grandTotal}</div>
        
        <div class="mb-2 text-left text-sm font-bold">Uang Diterima:</div>
        <input type="number" bind:value={amountPaid} class="mb-4 text-xl p-4 text-center" />
        
        <div class="flex justify-between mt-4">
            <button class="btn-outline p-4 w-full mr-2" on:click={() => showPaymentModal = false}>Batal</button>
            <button class="btn-success p-4 w-full ml-2 text-lg" on:click={submitPayment}>Konfirmasi</button>
        </div>
    </div>
</div>
{/if}

<style>
    .pos-container {
        height: 100vh;
        overflow: hidden;
    }
    
    .product-section {
        flex: 1;
        overflow-y: auto;
        background-color: var(--bg-color);
    }
    
    .grid-products {
        grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
        gap: 1.5rem;
    }
    
    .product-card {
        padding: 1.5rem;
        text-align: center;
        border: 2px solid transparent;
    }
    
    .product-card:hover {
        border-color: var(--primary-color);
    }
    
    .cart-section {
        width: 350px;
        background-color: white;
        border-left: 1px solid var(--border-color);
        display: flex;
        flex-direction: column;
        box-shadow: -4px 0 15px rgba(0,0,0,0.05);
    }
    
    .cart-items {
        flex: 1;
        overflow-y: auto;
    }
    
    .cart-item {
        background-color: var(--bg-color);
        border-radius: var(--radius-md);
    }
    
    .cart-totals {
        border-top: 2px dashed var(--border-color);
        padding-top: 1rem;
    }
</style>
