<script lang="ts">
    import { onMount } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';

    type Product = {
        id: string;
        name: string;
        sku: string;
        price: number;
        cost: number | null;
        category_name: string | null;
        track_stock: boolean;
        image_url?: string | null;
        qty_on_hand: number;
        min_qty: number;
    };

    type LowStockItem = {
        product_id: string;
        name: string;
        sku: string;
        qty_on_hand: number;
        min_qty: number;
    };

    type StockMovementItem = {
        id: string;
        movement_type: string;
        qty_delta: number;
        reason: string | null;
        reference_type: string | null;
        reference_id: string | null;
        actor_name: string;
        created_at: string;
    };

    let products: Product[] = [];
    let lowStockItems: LowStockItem[] = [];
    
    // View Switcher
    let viewMode: 'table' | 'card' = 'table';

    // Modals state (Stock Actions)
    let showModal = false;
    let modalAction = ''; // 'in', 'adjust', 'opname', 'transfer'
    let selectedProduct: Product | null = null;
    let actionQty = 0;
    let actionReason = '';

    // Add Product Modal state
    let showAddProductModal = false;
    let newName = '';
    let newSku = '';
    let newCategory = '';
    let newPrice = 0;
    let newCost = 0;
    let newTrackStock = true;
    let newInitialQty = 0;
    let newImageUrl = '';

    // Edit Product Modal state
    let showEditModal = false;
    let selectedEditProduct: Product | null = null;
    let editName = '';
    let editSku = '';
    let editCategory = '';
    let editPrice = 0;
    let editCost = 0;
    let editTrackStock = true;
    let editImageUrl = '';

    // View Product Modal (Detail & Stock History) state
    let showViewModal = false;
    let selectedViewProduct: Product | null = null;
    let selectedViewProductMovements: StockMovementItem[] = [];
    let loadingMovements = false;

    let fileInput: HTMLInputElement;
    let imageInput: HTMLInputElement;
    let editImageInput: HTMLInputElement;
    
    onMount(async () => {
        await fetchData();
    });

    async function fetchData() {
        try {
            products = await invoke('get_inventory_products');
            lowStockItems = await invoke('get_low_stock');
        } catch (e) {
            console.error(e);
            alert('Gagal mengambil data inventaris: ' + e);
        }
    }

    function openModal(action: string, product: Product) {
        modalAction = action;
        selectedProduct = product;
        actionQty = 0;
        actionReason = '';
        showModal = true;
    }

    async function submitAction() {
        if (!selectedProduct) return;
        if (actionQty <= 0 && modalAction !== 'adjust' && modalAction !== 'opname') {
            alert('Kuantitas harus lebih besar dari 0');
            return;
        }

        try {
            if (modalAction === 'in') {
                await invoke('stock_in', { productId: selectedProduct.id, qty: actionQty, reason: actionReason || null });
            } else if (modalAction === 'adjust') {
                await invoke('adjust_stock', { productId: selectedProduct.id, qtyDelta: actionQty, reason: actionReason });
            } else if (modalAction === 'opname') {
                await invoke('stock_opname', { productId: selectedProduct.id, actualQty: actionQty, reason: actionReason });
            } else if (modalAction === 'transfer') {
                await invoke('transfer_stock', { productId: selectedProduct.id, qty: actionQty, destinationOutlet: 'Gudang Pusat', reason: actionReason });
            }

            alert('Berhasil memperbarui stok!');
            showModal = false;
            await fetchData();
        } catch (e) {
            alert('Gagal: ' + e);
        }
    }

    async function addProduct() {
        if (!newName.trim() || !newSku.trim() || newPrice <= 0) {
            alert('Nama, SKU, dan Harga Jual wajib diisi/lebih besar dari 0');
            return;
        }

        try {
            await invoke('create_product', {
                name: newName,
                sku: newSku,
                price: newPrice,
                cost: newCost > 0 ? newCost : null,
                categoryName: newCategory || null,
                trackStock: newTrackStock,
                initialQty: newInitialQty,
                imageUrl: newImageUrl || null,
            });
            alert('Produk baru berhasil ditambahkan!');
            showAddProductModal = false;
            // Clear inputs
            newName = '';
            newSku = '';
            newCategory = '';
            newPrice = 0;
            newCost = 0;
            newTrackStock = true;
            newInitialQty = 0;
            newImageUrl = '';
            await fetchData();
        } catch (e) {
            alert('Gagal menambahkan produk: ' + e);
        }
    }

    function openEditModal(product: Product) {
        selectedEditProduct = product;
        editName = product.name;
        editSku = product.sku;
        editCategory = product.category_name || '';
        editPrice = product.price;
        editCost = product.cost || 0;
        editTrackStock = product.track_stock;
        editImageUrl = product.image_url || '';
        showEditModal = true;
    }

    async function submitEdit() {
        if (!selectedEditProduct) return;
        if (!editName.trim() || !editSku.trim() || editPrice <= 0) {
            alert('Nama, SKU, dan Harga Jual wajib diisi/lebih besar dari 0');
            return;
        }

        try {
            await invoke('update_product', {
                id: selectedEditProduct.id,
                name: editName,
                sku: editSku,
                price: editPrice,
                cost: editCost > 0 ? editCost : null,
                categoryName: editCategory || null,
                trackStock: editTrackStock,
                imageUrl: editImageUrl || null,
            });
            alert('Produk berhasil diperbarui!');
            showEditModal = false;
            await fetchData();
        } catch (e) {
            alert('Gagal mengedit produk: ' + e);
        }
    }

    async function openViewModal(product: Product) {
        selectedViewProduct = product;
        selectedViewProductMovements = [];
        loadingMovements = true;
        showViewModal = true;
        
        try {
            selectedViewProductMovements = await invoke('get_product_movements', { productId: product.id });
        } catch (e) {
            console.error(e);
            alert('Gagal memuat riwayat pergerakan stok: ' + e);
        } finally {
            loadingMovements = false;
        }
    }

    function triggerFileInput() {
        fileInput.click();
    }

    async function handleCsvUpload(event: Event) {
        const target = event.target as HTMLInputElement;
        const file = target.files?.[0];
        if (!file) return;

        const reader = new FileReader();
        reader.onload = async (e) => {
            const csvContent = e.target?.result as string;
            try {
                const message: string = await invoke('import_products_csv', { csvContent });
                alert(message);
                await fetchData();
            } catch (err) {
                alert('Gagal mengimpor CSV: ' + err);
            }
        };
        reader.readAsText(file);
        target.value = '';
    }

    function downloadSampleCsv() {
        const header = "name,sku,category,price,cost,track_stock,initial_qty,image_url\n";
        const row1 = "Kopi Caramel Latte,KCL-001,Kopi,28000,10000,true,50,https://images.unsplash.com/photo-1541167760496-1628856ab772?q=80&w=200&auto=format&fit=crop\n";
        const row2 = "Es Teh Manis,ETM-001,Teh,8000,2000,true,100,\n";
        const csvContent = header + row1 + row2;
        
        const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
        const url = URL.createObjectURL(blob);
        const link = document.createElement("a");
        link.setAttribute("href", url);
        link.setAttribute("download", "format_import_produk_posq.csv");
        link.style.visibility = 'hidden';
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
    }

    function handleImageFileChange(event: Event) {
        const target = event.target as HTMLInputElement;
        const file = target.files?.[0];
        if (!file) return;

        // Max file size: 1MB
        if (file.size > 1024 * 1024) {
            alert('Ukuran gambar tidak boleh melebihi 1MB');
            target.value = '';
            return;
        }

        const reader = new FileReader();
        reader.onload = (e) => {
            newImageUrl = e.target?.result as string; // Base64 Data URL
        };
        reader.readAsDataURL(file);
        target.value = ''; // Reset to allow re-upload
    }

    function handleEditImageFileChange(event: Event) {
        const target = event.target as HTMLInputElement;
        const file = target.files?.[0];
        if (!file) return;

        // Max file size: 1MB
        if (file.size > 1024 * 1024) {
            alert('Ukuran gambar tidak boleh melebihi 1MB');
            target.value = '';
            return;
        }

        const reader = new FileReader();
        reader.onload = (e) => {
            editImageUrl = e.target?.result as string; // Base64 Data URL
        };
        reader.readAsDataURL(file);
        target.value = '';
    }
</script>
<div class="p-8 h-screen bg-gray-100 flex flex-col">
    <div class="flex justify-between items-center mb-6">
        <div class="flex items-center gap-4">
            <h1 class="text-2xl font-bold">Backoffice: Inventory</h1>
            <div class="bg-gray-200 p-1 rounded-lg flex gap-1">
                <button class="px-3 py-1.5 rounded-md text-xs font-bold transition-all {viewMode === 'table' ? 'bg-white shadow-sm text-blue-600' : 'text-gray-600 hover:text-gray-800'}" on:click={() => viewMode = 'table'}>
                    📊 Tabel
                </button>
                <button class="px-3 py-1.5 rounded-md text-xs font-bold transition-all {viewMode === 'card' ? 'bg-white shadow-sm text-blue-600' : 'text-gray-600 hover:text-gray-800'}" on:click={() => viewMode = 'card'}>
                    🃏 Kartu
                </button>
            </div>
        </div>
        <div class="flex gap-2">
            <button class="bg-blue-600 text-white px-4 py-2 rounded-lg font-bold hover:bg-blue-700 transition" on:click={() => showAddProductModal = true}>
                + Tambah Produk Manual
            </button>
            <button class="bg-green-600 text-white px-4 py-2 rounded-lg font-bold hover:bg-green-700 transition" on:click={triggerFileInput}>
                📥 Impor CSV
            </button>
            <button class="bg-gray-200 text-gray-700 px-4 py-2 rounded-lg font-bold hover:bg-gray-300 transition" on:click={downloadSampleCsv}>
                📄 Unduh Contoh CSV
            </button>
            <input type="file" bind:this={fileInput} on:change={handleCsvUpload} accept=".csv" style="display: none;" />
            <a href="/pos" class="btn-outline px-4 py-2" style="text-decoration: none;">Kembali ke Kasir</a>
        </div>
    </div>

    <!-- Low Stock Alert Widget -->
    {#if lowStockItems.length > 0}
        <div class="alert alert-danger mb-6 p-4 rounded bg-red-100 border border-red-300">
            <h2 class="font-bold text-red-800 text-lg mb-2">Peringatan: Stok Menipis!</h2>
            <ul class="list-disc pl-5 text-red-700">
                {#each lowStockItems as item}
                    <li>{item.name} (SKU: {item.sku}) - Sisa Stok: <strong>{item.qty_on_hand}</strong> (Min: {item.min_qty})</li>
                {/each}
            </ul>
        </div>
    {/if}

    {#if viewMode === 'table'}
        <div class="card flex-1 glassmorphism overflow-hidden flex flex-col">
            <div class="overflow-y-auto p-0">
                <table class="w-full text-left border-collapse">
                    <thead class="bg-gray-50 border-b">
                        <tr>
                            <th class="p-4 font-bold text-gray-700">Produk</th>
                            <th class="p-4 font-bold text-gray-700">SKU</th>
                            <th class="p-4 font-bold text-gray-700">Stok Saat Ini</th>
                            <th class="p-4 font-bold text-gray-700 text-center">Aksi</th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each products as product}
                            <tr class="border-b hover:bg-gray-50 transition-colors">
                                <td class="p-4 flex items-center gap-3">
                                    {#if product.image_url}
                                        <img src={product.image_url} class="w-10 h-10 object-cover rounded-lg shadow-sm" alt={product.name} />
                                    {:else}
                                        <div class="w-10 h-10 bg-slate-200 text-slate-500 font-bold flex items-center justify-center rounded-lg shadow-sm">
                                            {product.name.charAt(0).toUpperCase()}
                                        </div>
                                    {/if}
                                    <span class="font-bold">{product.name}</span>
                                </td>
                                <td class="p-4 text-gray-600">{product.sku}</td>
                                <td class="p-4">
                                    <span class="bg-blue-100 text-blue-800 px-3 py-1 rounded-full font-bold text-sm">
                                        {product.qty_on_hand}
                                    </span>
                                </td>
                                <td class="p-4 text-center">
                                    <div class="flex gap-2 justify-center items-center">
                                        <button class="bg-blue-600 hover:bg-blue-700 text-white px-2.5 py-1.5 rounded text-xs font-bold transition flex items-center gap-1" on:click={() => openViewModal(product)}>
                                            👁 Detail
                                        </button>
                                        <button class="bg-amber-500 hover:bg-amber-600 text-white px-2.5 py-1.5 rounded text-xs font-bold transition flex items-center gap-1" on:click={() => openEditModal(product)}>
                                            ✏ Edit
                                        </button>
                                        <span class="text-gray-300">|</span>
                                        <button class="bg-green-100 text-green-700 px-2 py-1 rounded text-xs hover:bg-green-200 font-bold" on:click={() => openModal('in', product)}>+ In</button>
                                        <button class="bg-yellow-100 text-yellow-700 px-2 py-1 rounded text-xs hover:bg-yellow-200 font-bold" on:click={() => openModal('adjust', product)}>Adjust</button>
                                        <button class="bg-purple-100 text-purple-700 px-2 py-1 rounded text-xs hover:bg-purple-200 font-bold" on:click={() => openModal('opname', product)}>Opname</button>
                                        <button class="bg-gray-200 text-gray-700 px-2 py-1 rounded text-xs hover:bg-gray-300 font-bold" on:click={() => openModal('transfer', product)}>Transfer</button>
                                    </div>
                                </td>
                            </tr>
                        {/each}
                        {#if products.length === 0}
                            <tr>
                                <td colspan="4" class="p-8 text-center text-gray-500">
                                    Tidak ada produk ditemukan.
                                </td>
                            </tr>
                        {/if}
                    </tbody>
                </table>
            </div>
        </div>
    {:else}
        <!-- Card/Grid View -->
        <div class="flex-1 overflow-y-auto pb-8">
            <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6">
                {#each products as product}
                    <div class="bg-white rounded-2xl shadow-md border border-slate-200 overflow-hidden flex flex-col hover:shadow-lg transition-shadow duration-300">
                        <!-- Product Image -->
                        <div class="relative w-full h-40 bg-slate-100 flex items-center justify-center border-b">
                            {#if product.image_url}
                                <img src={product.image_url} class="w-full h-full object-cover" alt={product.name} />
                            {:else}
                                <div class="w-full h-full bg-slate-200 text-slate-400 font-bold flex items-center justify-center text-4xl">
                                    {product.name.charAt(0).toUpperCase()}
                                </div>
                            {/if}
                            <!-- Stock Badge -->
                            <div class="absolute top-3 right-3">
                                {#if product.qty_on_hand <= product.min_qty}
                                    <span class="bg-red-500 text-white px-2.5 py-1 rounded-full text-xs font-bold shadow-sm">
                                        Kritis: {product.qty_on_hand}
                                    </span>
                                {:else}
                                    <span class="bg-blue-600 text-white px-2.5 py-1 rounded-full text-xs font-bold shadow-sm">
                                        Stok: {product.qty_on_hand}
                                    </span>
                                {/if}
                            </div>
                        </div>

                        <!-- Product Info -->
                        <div class="p-4 flex-1 flex flex-col">
                            <span class="text-[10px] uppercase tracking-wider text-slate-400 font-bold">{product.category_name || 'Tanpa Kategori'}</span>
                            <h3 class="font-bold text-slate-800 text-base mt-1 line-clamp-1">{product.name}</h3>
                            <span class="text-xs text-slate-500 mt-0.5">SKU: {product.sku}</span>
                            
                            <div class="mt-4 flex justify-between items-baseline">
                                <div>
                                    <span class="text-[10px] text-slate-400 block font-semibold">Harga Jual</span>
                                    <span class="font-extrabold text-blue-600 text-lg">Rp {product.price.toLocaleString('id-ID')}</span>
                                </div>
                                {#if product.cost}
                                    <div class="text-right">
                                        <span class="text-[10px] text-slate-400 block font-semibold">Harga Modal</span>
                                        <span class="font-bold text-slate-500 text-sm">Rp {product.cost.toLocaleString('id-ID')}</span>
                                    </div>
                                {/if}
                            </div>

                            <!-- Buttons Section -->
                            <div class="mt-6 pt-4 border-t border-slate-100 flex flex-col gap-2">
                                <div class="flex gap-2">
                                    <button class="flex-1 bg-blue-600 hover:bg-blue-700 text-white py-1.5 rounded-lg text-xs font-bold transition flex justify-center items-center gap-1" on:click={() => openViewModal(product)}>
                                        👁 Detail
                                    </button>
                                    <button class="flex-1 bg-amber-500 hover:bg-amber-600 text-white py-1.5 rounded-lg text-xs font-bold transition flex justify-center items-center gap-1" on:click={() => openEditModal(product)}>
                                        ✏ Edit
                                    </button>
                                </div>
                                <div class="grid grid-cols-4 gap-1 mt-1 text-center">
                                    <button class="bg-green-50 hover:bg-green-100 text-green-700 py-1 rounded text-[10px] font-bold" on:click={() => openModal('in', product)}>+ In</button>
                                    <button class="bg-yellow-50 hover:bg-yellow-100 text-yellow-700 py-1 rounded text-[10px] font-bold" on:click={() => openModal('adjust', product)}>Adj</button>
                                    <button class="bg-purple-50 hover:bg-purple-100 text-purple-700 py-1 rounded text-[10px] font-bold" on:click={() => openModal('opname', product)}>Opn</button>
                                    <button class="bg-gray-100 hover:bg-gray-200 text-gray-700 py-1 rounded text-[10px] font-bold" on:click={() => openModal('transfer', product)}>Trf</button>
                                </div>
                            </div>
                        </div>
                    </div>
                {/each}
                {#if products.length === 0}
                    <div class="col-span-full bg-white p-12 text-center text-gray-500 rounded-2xl border border-dashed border-slate-300">
                        Tidak ada produk ditemukan.
                    </div>
                {/if}
            </div>
        </div>
    {/if}
</div>

<!-- ACTION MODAL -->
{#if showModal}
<div class="modal-backdrop fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
    <div class="modal-content bg-white p-6 rounded-lg shadow-xl w-full max-w-md">
        <h2 class="text-xl font-bold mb-4 capitalize">
            {modalAction === 'in' ? 'Barang Masuk (Stock In)' : 
             modalAction === 'adjust' ? 'Penyesuaian Stok (Adjustment)' :
             modalAction === 'opname' ? 'Fisik Stok (Opname)' : 'Transfer Keluar (Ke Gudang)'}
        </h2>
        <div class="mb-4">
            <span class="text-gray-500">Produk:</span> <strong>{selectedProduct?.name}</strong> <br/>
            <span class="text-gray-500">Stok Sistem:</span> <span class="bg-gray-200 px-2 py-1 rounded text-sm">{selectedProduct?.qty_on_hand}</span>
        </div>

        <label class="block font-bold mb-2 text-sm" for="actionQty">
            {#if modalAction === 'in'}Jumlah Masuk (Stok +){/if}
            {#if modalAction === 'adjust'}Selisih Stok (+ atau -){/if}
            {#if modalAction === 'opname'}Stok Aktual (Fisik){/if}
            {#if modalAction === 'transfer'}Jumlah Transfer{/if}
        </label>
        <input id="actionQty" type="number" bind:value={actionQty} class="w-full p-2 border rounded mb-4" />

        <label class="block font-bold mb-2 text-sm" for="actionReason">
            Alasan / Keterangan {modalAction !== 'in' ? '(Wajib)' : '(Opsional)'}
        </label>
        <input id="actionReason" type="text" bind:value={actionReason} class="w-full p-2 border rounded mb-6" placeholder="Contoh: Barang rusak, retur, dll" />

        <div class="flex gap-2">
            <button class="btn-outline w-full p-3" on:click={() => showModal = false}>Batal</button>
            <button class="btn-primary w-full p-3 font-bold" on:click={submitAction}>Simpan</button>
        </div>
    </div>
</div>
{/if}

<!-- ADD PRODUCT MODAL -->
{#if showAddProductModal}
<div class="modal-backdrop fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
    <div class="modal-content bg-white p-6 rounded-lg shadow-xl w-full max-w-md overflow-y-auto max-h-[90vh]">
        <h2 class="text-xl font-bold mb-4">Tambah Produk Baru (Manual)</h2>
        
        <label class="block font-bold mb-1 text-sm" for="newName">Nama Produk *</label>
        <input id="newName" type="text" bind:value={newName} class="w-full p-2 border rounded mb-3" placeholder="Contoh: Kopi Caramel Latte" />

        <label class="block font-bold mb-1 text-sm" for="newSku">SKU (Kode Produk) *</label>
        <input id="newSku" type="text" bind:value={newSku} class="w-full p-2 border rounded mb-3" placeholder="Contoh: KCL-001" />

        <label class="block font-bold mb-1 text-sm" for="newCategory">Kategori</label>
        <input id="newCategory" type="text" bind:value={newCategory} class="w-full p-2 border rounded mb-3" placeholder="Contoh: Kopi, Makanan, dll" />

        <label class="block font-bold mb-1 text-sm">Gambar Produk</label>
        <div class="flex items-center gap-4 mb-3 p-3 border rounded-xl bg-slate-50 border-slate-200 shadow-sm">
            {#if newImageUrl}
                <div class="relative w-16 h-16 border border-slate-200 rounded-lg overflow-hidden bg-white shadow-xs">
                    <img src={newImageUrl} class="w-full h-full object-cover" alt="Preview" />
                    <button type="button" class="absolute top-0 right-0 bg-red-600 text-white rounded-bl-lg p-1 text-[10px] font-bold hover:bg-red-700 transition" on:click={() => newImageUrl = ''}>
                        ✕
                    </button>
                </div>
            {:else}
                <div class="w-16 h-16 bg-slate-200 text-slate-400 flex items-center justify-center rounded-lg border border-dashed border-slate-300">
                    📷
                </div>
            {/if}
            <div>
                <button type="button" class="bg-white border border-slate-300 hover:bg-slate-50 text-slate-700 px-3 py-1.5 rounded-lg text-xs font-bold transition" on:click={() => imageInput.click()}>
                    Pilih File Gambar
                </button>
                <input type="file" bind:this={imageInput} on:change={handleImageFileChange} accept="image/*" style="display: none;" />
                <p class="text-slate-400 text-[10px] mt-1">Format: JPG, PNG, WEBP (Maks 1MB)</p>
            </div>
        </div>

        <div class="grid grid-cols-2 gap-3 mb-3">
            <div>
                <label class="block font-bold mb-1 text-sm" for="newPrice">Harga Jual (Rp) *</label>
                <input id="newPrice" type="number" bind:value={newPrice} class="w-full p-2 border rounded" placeholder="28000" />
            </div>
            <div>
                <label class="block font-bold mb-1 text-sm" for="newCost">Harga Modal (Rp)</label>
                <input id="newCost" type="number" bind:value={newCost} class="w-full p-2 border rounded" placeholder="10000" />
            </div>
        </div>

        <div class="flex items-center gap-2 mb-4 mt-2">
            <input id="newTrackStock" type="checkbox" bind:checked={newTrackStock} class="w-4 h-4" />
            <label for="newTrackStock" class="text-sm font-bold select-none cursor-pointer">Lacak & Kelola Stok Inventaris</label>
        </div>

        {#if newTrackStock}
        <label class="block font-bold mb-1 text-sm" for="newInitialQty">Jumlah Stok Awal</label>
        <input id="newInitialQty" type="number" bind:value={newInitialQty} class="w-full p-2 border rounded mb-6" placeholder="0" />
        {/if}

        <div class="flex gap-2">
            <button class="btn-outline w-full p-3" on:click={() => showAddProductModal = false}>Batal</button>
            <button class="btn-primary w-full p-3 font-bold bg-blue-600 hover:bg-blue-700 text-white" on:click={addProduct}>Simpan Produk</button>
        </div>
    </div>
</div>
{/if}

<!-- EDIT PRODUCT MODAL -->
{#if showEditModal}
<div class="modal-backdrop fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
    <div class="modal-content bg-white p-6 rounded-lg shadow-xl w-full max-w-md overflow-y-auto max-h-[90vh]">
        <h2 class="text-xl font-bold mb-4">Ubah Informasi Produk</h2>
        
        <label class="block font-bold mb-1 text-sm" for="editName">Nama Produk *</label>
        <input id="editName" type="text" bind:value={editName} class="w-full p-2 border rounded mb-3" placeholder="Contoh: Kopi Caramel Latte" />

        <label class="block font-bold mb-1 text-sm" for="editSku">SKU (Kode Produk) *</label>
        <input id="editSku" type="text" bind:value={editSku} class="w-full p-2 border rounded mb-3" placeholder="Contoh: KCL-001" />

        <label class="block font-bold mb-1 text-sm" for="editCategory">Kategori</label>
        <input id="editCategory" type="text" bind:value={editCategory} class="w-full p-2 border rounded mb-3" placeholder="Contoh: Kopi, Makanan, dll" />

        <label class="block font-bold mb-1 text-sm">Gambar Produk</label>
        <div class="flex items-center gap-4 mb-3 p-3 border rounded-xl bg-slate-50 border-slate-200 shadow-sm">
            {#if editImageUrl}
                <div class="relative w-16 h-16 border border-slate-200 rounded-lg overflow-hidden bg-white shadow-xs">
                    <img src={editImageUrl} class="w-full h-full object-cover" alt="Preview" />
                    <button type="button" class="absolute top-0 right-0 bg-red-600 text-white rounded-bl-lg p-1 text-[10px] font-bold hover:bg-red-700 transition" on:click={() => editImageUrl = ''}>
                        ✕
                    </button>
                </div>
            {:else}
                <div class="w-16 h-16 bg-slate-200 text-slate-400 flex items-center justify-center rounded-lg border border-dashed border-slate-300">
                    📷
                </div>
            {/if}
            <div>
                <button type="button" class="bg-white border border-slate-300 hover:bg-slate-50 text-slate-700 px-3 py-1.5 rounded-lg text-xs font-bold transition" on:click={() => editImageInput.click()}>
                    Pilih File Gambar
                </button>
                <input type="file" bind:this={editImageInput} on:change={handleEditImageFileChange} accept="image/*" style="display: none;" />
                <p class="text-slate-400 text-[10px] mt-1">Format: JPG, PNG, WEBP (Maks 1MB)</p>
            </div>
        </div>

        <div class="grid grid-cols-2 gap-3 mb-3">
            <div>
                <label class="block font-bold mb-1 text-sm" for="editPrice">Harga Jual (Rp) *</label>
                <input id="editPrice" type="number" bind:value={editPrice} class="w-full p-2 border rounded" />
            </div>
            <div>
                <label class="block font-bold mb-1 text-sm" for="editCost">Harga Modal (Rp)</label>
                <input id="editCost" type="number" bind:value={editCost} class="w-full p-2 border rounded" />
            </div>
        </div>

        <div class="flex items-center gap-2 mb-6 mt-2">
            <input id="editTrackStock" type="checkbox" bind:checked={editTrackStock} class="w-4 h-4" />
            <label for="editTrackStock" class="text-sm font-bold select-none cursor-pointer">Lacak & Kelola Stok Inventaris</label>
        </div>

        <div class="flex gap-2">
            <button class="btn-outline w-full p-3" on:click={() => showEditModal = false}>Batal</button>
            <button class="btn-primary w-full p-3 font-bold bg-amber-500 hover:bg-amber-600 text-white" on:click={submitEdit}>Simpan Perubahan</button>
        </div>
    </div>
</div>
{/if}

<!-- VIEW PRODUCT MODAL -->
{#if showViewModal}
<div class="modal-backdrop fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
    <div class="modal-content bg-white p-6 rounded-2xl shadow-xl w-full max-w-4xl overflow-y-auto max-h-[90vh]">
        <div class="flex justify-between items-center mb-6 pb-3 border-b">
            <h2 class="text-xl font-bold text-slate-800">Detail & Riwayat Stok Produk</h2>
            <button class="text-slate-400 hover:text-slate-600 text-xl font-bold" on:click={() => showViewModal = false}>✕</button>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
            <!-- Column 1: Info Card -->
            <div class="md:col-span-1 border border-slate-200 rounded-2xl p-4 bg-slate-50 flex flex-col items-center">
                <div class="w-full h-48 bg-white border border-slate-200 rounded-xl overflow-hidden shadow-sm flex items-center justify-center mb-4">
                    {#if selectedViewProduct?.image_url}
                        <img src={selectedViewProduct.image_url} class="w-full h-full object-cover" alt={selectedViewProduct.name} />
                    {:else}
                        <div class="w-full h-full bg-slate-200 text-slate-400 font-bold flex items-center justify-center text-5xl">
                            {selectedViewProduct?.name.charAt(0).toUpperCase()}
                        </div>
                    {/if}
                </div>

                <div class="w-full text-left">
                    <span class="text-[10px] uppercase font-bold tracking-wider text-slate-400">{selectedViewProduct?.category_name || 'Tanpa Kategori'}</span>
                    <h3 class="font-extrabold text-slate-800 text-lg mt-0.5">{selectedViewProduct?.name}</h3>
                    <p class="text-xs text-slate-500 mt-0.5 font-mono">SKU: {selectedViewProduct?.sku}</p>

                    <div class="mt-6 space-y-3">
                        <div class="flex justify-between text-sm py-1.5 border-b border-slate-200">
                            <span class="text-slate-500 font-medium">Stok Fisik</span>
                            <span class="font-bold bg-blue-100 text-blue-800 px-2.5 py-0.5 rounded-full text-xs">
                                {selectedViewProduct?.qty_on_hand}
                            </span>
                        </div>
                        <div class="flex justify-between text-sm py-1.5 border-b border-slate-200">
                            <span class="text-slate-500 font-medium">Harga Jual</span>
                            <span class="font-bold text-slate-800">
                                Rp {selectedViewProduct?.price.toLocaleString('id-ID')}
                            </span>
                        </div>
                        <div class="flex justify-between text-sm py-1.5 border-b border-slate-200">
                            <span class="text-slate-500 font-medium">Harga Modal</span>
                            <span class="font-bold text-slate-800">
                                {selectedViewProduct?.cost ? `Rp ${selectedViewProduct.cost.toLocaleString('id-ID')}` : '-'}
                            </span>
                        </div>
                        <div class="flex justify-between text-sm py-1.5 border-b border-slate-200">
                            <span class="text-slate-500 font-medium">Lacak Stok</span>
                            <span class="font-bold {selectedViewProduct?.track_stock ? 'text-green-600' : 'text-slate-400'}">
                                {selectedViewProduct?.track_stock ? 'Aktif' : 'Non-aktif'}
                            </span>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Column 2: Stock History -->
            <div class="md:col-span-2 flex flex-col">
                <h4 class="font-bold text-slate-800 text-base mb-3 flex items-center gap-2">
                    📋 Log Pergerakan Stok
                </h4>
                
                <div class="flex-1 border border-slate-200 rounded-xl overflow-hidden bg-white shadow-xs min-h-[300px] flex flex-col">
                    {#if loadingMovements}
                        <div class="flex-1 flex flex-col items-center justify-center text-slate-400 gap-2">
                            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-slate-600"></div>
                            <span class="text-xs">Memuat riwayat stok...</span>
                        </div>
                    {:else if selectedViewProductMovements.length === 0}
                        <div class="flex-1 flex items-center justify-center text-slate-400 text-sm">
                            Tidak ada riwayat pergerakan stok untuk produk ini.
                        </div>
                    {:else}
                        <div class="overflow-y-auto max-h-[350px] p-0">
                            <table class="w-full text-left border-collapse text-xs">
                                <thead class="bg-slate-50 border-b sticky top-0">
                                    <tr>
                                        <th class="p-3 font-bold text-slate-600">Waktu</th>
                                        <th class="p-3 font-bold text-slate-600">Tipe Aksi</th>
                                        <th class="p-3 font-bold text-slate-600 text-center">Jumlah</th>
                                        <th class="p-3 font-bold text-slate-600">Keterangan / Oleh</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {#each selectedViewProductMovements as mv}
                                        <tr class="border-b hover:bg-slate-50 transition-colors">
                                            <td class="p-3 text-slate-500 font-mono whitespace-nowrap">
                                                {new Date(mv.created_at).toLocaleString('id-ID')}
                                            </td>
                                            <td class="p-3">
                                                {#if mv.movement_type === 'stock_in'}
                                                    <span class="bg-green-100 text-green-800 px-2 py-0.5 rounded-full font-bold">Stok Masuk</span>
                                                {:else if mv.movement_type === 'sale'}
                                                    <span class="bg-blue-100 text-blue-800 px-2 py-0.5 rounded-full font-bold">Penjualan</span>
                                                {:else if mv.movement_type === 'refund'}
                                                    <span class="bg-red-100 text-red-800 px-2 py-0.5 rounded-full font-bold">Refund</span>
                                                {:else if mv.movement_type === 'adjustment'}
                                                    <span class="bg-yellow-100 text-yellow-800 px-2 py-0.5 rounded-full font-bold">Penyesuaian</span>
                                                {:else if mv.movement_type === 'opname'}
                                                    <span class="bg-purple-100 text-purple-800 px-2 py-0.5 rounded-full font-bold">Opname</span>
                                                {:else}
                                                    <span class="bg-slate-100 text-slate-800 px-2 py-0.5 rounded-full font-bold">{mv.movement_type}</span>
                                                {/if}
                                            </td>
                                            <td class="p-3 text-center font-bold {mv.qty_delta > 0 ? 'text-green-600' : mv.qty_delta < 0 ? 'text-red-600' : 'text-slate-600'}">
                                                {mv.qty_delta > 0 ? `+${mv.qty_delta}` : mv.qty_delta}
                                            </td>
                                            <td class="p-3 text-slate-600">
                                                <div class="font-medium">{mv.reason || '-'}</div>
                                                <div class="text-[10px] text-slate-400 mt-0.5">Oleh: {mv.actor_name}</div>
                                            </td>
                                        </tr>
                                    {/each}
                                </tbody>
                            </table>
                        </div>
                    {/if}
                </div>
            </div>
        </div>

        <div class="mt-6 pt-3 border-t flex justify-end">
            <button class="bg-slate-800 hover:bg-slate-900 text-white font-bold px-6 py-2 rounded-xl transition text-sm" on:click={() => showViewModal = false}>
                Tutup
            </button>
        </div>
    </div>
</div>
{/if}

<style>
    th {
        position: sticky;
        top: 0;
        z-index: 10;
    }
</style>
