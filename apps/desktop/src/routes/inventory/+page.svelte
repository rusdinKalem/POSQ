<script lang="ts">
    import { onMount, tick } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';
    import { showToast } from '$lib/toast.svelte';
    import QRCode from 'qrcode';
    import JsBarcode from 'jsbarcode';

    type Product = {
        id: string;
        name: string;
        sku: string;
        price: number;
        cost: number | null;
        category_name: string | null;
        category_id: string | null;
        track_stock: boolean;
        image_url?: string | null;
        qty_on_hand: number;
        min_qty: number;
        is_ingredient: boolean;
        min_stock_factor: number;
        buffer_stock: number;
        lead_time_days: number;
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

    let products: Product[] = $state([]);
    let lowStockItems: LowStockItem[] = $state([]);
    
    // View Switcher
    let viewMode: 'table' | 'card' = $state('table');

    // Modals state (Stock Actions)
    let showModal = $state(false);
    let modalAction = $state(''); // 'in', 'adjust', 'opname', 'transfer'
    let selectedProduct: Product | null = $state(null);
    let actionQty = $state(0);
    let actionReason = $state('');

    // Add Product Modal state
    let showAddProductModal = $state(false);
    let newName = $state('');
    let newSku = $state('');
    let newCategoryId = $state('');
    let newPrice = $state(0);
    let newCost = $state(0);
    let newTrackStock = $state(true);
    let newInitialQty = $state(0);
    let newImageUrl = $state('');
    let newIsIngredient = $state(false);
    let newMinStockFactor = $state(0);
    let newBufferStock = $state(0);
    let newLeadTimeDays = $state(0);

    // Edit Product Modal state
    let showEditModal = $state(false);
    let selectedEditProduct: Product | null = $state(null);
    let editName = $state('');
    let editSku = $state('');
    let editCategoryId = $state('');
    let editPrice = $state(0);
    let editCost = $state(0);
    let editTrackStock = $state(true);
    let editImageUrl = $state('');
    let editIsIngredient = $state(false);
    let editMinStockFactor = $state(0);
    let editBufferStock = $state(0);
    let editLeadTimeDays = $state(0);

    // Recipe Management Modal state
    let showRecipeModal = $state(false);
    let recipeTargetProduct: Product | null = $state(null);
    let recipeIngredients: any[] = $state([]);
    let selectedAddIngredientId = $state('');
    let selectedAddIngredientQty = $state(0);
    let selectedAddIngredientUnit = $state('gr');

    // View Product Modal (Detail & Stock History) state
    let showViewModal = $state(false);
    let selectedViewProduct: Product | null = $state(null);
    let selectedViewProductMovements: StockMovementItem[] = $state([]);
    let loadingMovements = $state(false);

    let fileInput: HTMLInputElement = $state()!;
    let imageInput: HTMLInputElement = $state()!;
    let editImageInput: HTMLInputElement = $state()!;

    let showBarcodeModal = $state(false);
    let barcodeCanvas = $state<HTMLCanvasElement>();
    let qrcodeCanvas = $state<HTMLCanvasElement>();
    let includeName = $state(true);
    let includePrice = $state(true);
    let qrPrintDataUrl = $state('');
    let barPrintDataUrl = $state('');

    $effect(() => {
        if (showBarcodeModal && selectedViewProduct) {
            // Track dependencies to trigger regeneration
            const _name = includeName;
            const _price = includePrice;
            generateLabel();
        }
    });

    async function openBarcodeModal() {
        showBarcodeModal = true;
        await generateLabel();
    }

    async function generateLabel() {
        if (!selectedViewProduct) return;
        await tick();
        
        const sku = selectedViewProduct.sku || selectedViewProduct.id;
        
        if (qrcodeCanvas) {
            try {
                await QRCode.toCanvas(qrcodeCanvas, sku, {
                    width: 150,
                    margin: 1,
                    color: {
                        dark: '#0f172a',
                        light: '#ffffff'
                    }
                });
                qrPrintDataUrl = qrcodeCanvas.toDataURL();
            } catch (err) {
                console.error('Failed to generate QR Code:', err);
            }
        }
        
        if (barcodeCanvas) {
            try {
                JsBarcode(barcodeCanvas, sku, {
                    format: "CODE128",
                    width: 1.8,
                    height: 50,
                    displayValue: true,
                    fontSize: 12,
                    textMargin: 4,
                    lineColor: '#0f172a'
                });
                barPrintDataUrl = barcodeCanvas.toDataURL();
            } catch (err) {
                console.error('Failed to generate Barcode:', err);
            }
        }
    }

    function downloadQrCode() {
        if (!selectedViewProduct || !qrcodeCanvas) return;
        const product = selectedViewProduct;
        qrcodeCanvas.toBlob((blob) => {
            if (!blob) return;
            const url = URL.createObjectURL(blob);
            const link = document.createElement('a');
            link.download = `QR_${product.sku || product.name}.png`;
            link.href = url;
            link.style.visibility = 'hidden';
            document.body.appendChild(link);
            link.click();
            document.body.removeChild(link);
            setTimeout(() => {
                URL.revokeObjectURL(url);
            }, 5000);
        }, 'image/png');
    }
    
    function downloadBarcode() {
        if (!selectedViewProduct || !barcodeCanvas) return;
        const product = selectedViewProduct;
        barcodeCanvas.toBlob((blob) => {
            if (!blob) return;
            const url = URL.createObjectURL(blob);
            const link = document.createElement('a');
            link.download = `BARCODE_${product.sku || product.name}.png`;
            link.href = url;
            link.style.visibility = 'hidden';
            document.body.appendChild(link);
            link.click();
            document.body.removeChild(link);
            setTimeout(() => {
                URL.revokeObjectURL(url);
            }, 5000);
        }, 'image/png');
    }

    function printLabel() {
        if (!selectedViewProduct) return;
        window.print();
    }

    interface Category {
        id: string;
        name: string;
        parent_id?: string | null;
        parent_name?: string | null;
        business_mode?: string | null;
    }

    let dbCategories: Category[] = $state([]);

    let categoryOptions = $derived.by(() => {
        return dbCategories.map(cat => {
            let label = cat.name;
            if (cat.parent_name) {
                label = `${cat.parent_name} > ${cat.name}`;
            }
            if (cat.business_mode) {
                label += ` (${cat.business_mode === 'fb' ? 'F&B' : cat.business_mode === 'retail' ? 'Retail' : 'Jasa'})`;
            }
            return {
                id: cat.id,
                name: label
            };
        }).sort((a, b) => a.name.localeCompare(b.name));
    });

    async function fetchCategories() {
        try {
            dbCategories = await invoke('list_categories');
        } catch (e) {
            console.error('Failed to load categories:', e);
        }
    }
    
    onMount(async () => {
        await fetchData();
    });

    async function fetchData() {
        try {
            products = await invoke('get_inventory_products');
            lowStockItems = await invoke('get_low_stock');
            await fetchCategories();
        } catch (e) {
            console.error(e);
            showToast('Gagal mengambil data inventaris: ' + e, 'error');
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
            showToast('Kuantitas harus lebih besar dari 0', 'error');
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

            showToast('Berhasil memperbarui stok!', 'success');
            showModal = false;
            await fetchData();
        } catch (e) {
            showToast('Gagal: ' + e, 'error');
        }
    }

    async function addProduct() {
        if (!newName.trim() || !newSku.trim() || (!newIsIngredient && newPrice <= 0)) {
            showToast('Nama, SKU, dan Harga Jual (jika bukan bahan baku) wajib diisi/lebih besar dari 0', 'error');
            return;
        }

        try {
            await invoke('create_product', {
                name: newName,
                sku: newSku,
                price: newIsIngredient ? 0 : newPrice,
                cost: newCost > 0 ? newCost : null,
                categoryId: newCategoryId || null,
                trackStock: newTrackStock,
                initialQty: newInitialQty,
                imageUrl: newImageUrl || null,
                isIngredient: newIsIngredient,
                minStockFactor: newMinStockFactor,
                bufferStock: newBufferStock,
                leadTimeDays: newLeadTimeDays,
            });
            showToast('Produk baru berhasil ditambahkan!', 'success');
            showAddProductModal = false;
            // Clear inputs
            newName = '';
            newSku = '';
            newCategoryId = '';
            newPrice = 0;
            newCost = 0;
            newTrackStock = true;
            newInitialQty = 0;
            newImageUrl = '';
            newIsIngredient = false;
            newMinStockFactor = 0;
            newBufferStock = 0;
            newLeadTimeDays = 0;
            await fetchData();
        } catch (e) {
            showToast('Gagal menambahkan produk: ' + e, 'error');
        }
    }

    function openEditModal(product: Product) {
        selectedEditProduct = product;
        editName = product.name;
        editSku = product.sku;
        editCategoryId = product.category_id || '';
        editPrice = product.price;
        editCost = product.cost || 0;
        editTrackStock = product.track_stock;
        editImageUrl = product.image_url || '';
        editIsIngredient = product.is_ingredient;
        editMinStockFactor = product.min_stock_factor || 0;
        editBufferStock = product.buffer_stock || 0;
        editLeadTimeDays = product.lead_time_days || 0;
        showEditModal = true;
    }

    async function submitEdit() {
        if (!selectedEditProduct) return;
        if (!editName.trim() || !editSku.trim() || (!editIsIngredient && editPrice <= 0)) {
            showToast('Nama, SKU, dan Harga Jual (jika bukan bahan baku) wajib diisi/lebih besar dari 0', 'error');
            return;
        }

        try {
            await invoke('update_product', {
                id: selectedEditProduct.id,
                name: editName,
                sku: editSku,
                price: editIsIngredient ? 0 : editPrice,
                cost: editCost > 0 ? editCost : null,
                categoryId: editCategoryId || null,
                trackStock: editTrackStock,
                imageUrl: editImageUrl || null,
                isIngredient: editIsIngredient,
                minStockFactor: editMinStockFactor,
                bufferStock: editBufferStock,
                leadTimeDays: editLeadTimeDays,
            });
            showToast('Produk berhasil diperbarui!', 'success');
            showEditModal = false;
            await fetchData();
        } catch (e) {
            showToast('Gagal mengedit produk: ' + e, 'error');
        }
    }

    async function openRecipeModal(product: Product) {
        recipeTargetProduct = product;
        selectedAddIngredientId = '';
        selectedAddIngredientQty = 0;
        selectedAddIngredientUnit = 'gr';
        try {
            recipeIngredients = await invoke('get_recipe_ingredients', { productId: product.id });
            showRecipeModal = true;
        } catch (e) {
            showToast('Gagal memuat resep: ' + e, 'error');
        }
    }

    function addRecipeIngredient() {
        if (!selectedAddIngredientId) {
            showToast('Pilih bahan baku terlebih dahulu', 'error');
            return;
        }
        if (selectedAddIngredientQty <= 0) {
            showToast('Jumlah bahan baku harus lebih besar dari 0', 'error');
            return;
        }

        const ingredient = products.find(p => p.id === selectedAddIngredientId);
        if (!ingredient) return;

        // Check if already in the recipe list
        const exists = recipeIngredients.some(ing => ing.ingredient_id === selectedAddIngredientId);
        if (exists) {
            showToast('Bahan baku tersebut sudah ada di resep', 'error');
            return;
        }

        recipeIngredients = [
            ...recipeIngredients,
            {
                ingredient_id: selectedAddIngredientId,
                name: ingredient.name,
                sku: ingredient.sku,
                qty: selectedAddIngredientQty,
                unit: selectedAddIngredientUnit
            }
        ];

        // Reset inputs
        selectedAddIngredientId = '';
        selectedAddIngredientQty = 0;
    }

    function removeRecipeIngredient(id: string) {
        recipeIngredients = recipeIngredients.filter(ing => ing.ingredient_id !== id);
    }

    async function saveRecipe() {
        if (!recipeTargetProduct) return;
        try {
            const payload = recipeIngredients.map(ing => ({
                ingredient_id: ing.ingredient_id,
                qty: Number(ing.qty),
                unit: ing.unit
            }));

            await invoke('save_recipe', {
                productId: recipeTargetProduct.id,
                ingredients: payload
            });

            showToast('Resep berhasil disimpan!', 'success');
            showRecipeModal = false;
        } catch (e) {
            showToast('Gagal menyimpan resep: ' + e, 'error');
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
            showToast('Gagal memuat riwayat pergerakan stok: ' + e, 'error');
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
                showToast(message);
                await fetchData();
            } catch (err) {
                showToast('Gagal mengimpor CSV: ' + err, 'error');
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
            showToast('Ukuran gambar tidak boleh melebihi 1MB', 'error');
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
            showToast('Ukuran gambar tidak boleh melebihi 1MB', 'error');
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
<div class="p-4 md:p-8 h-screen bg-slate-50 flex flex-col overflow-hidden">
    <div class="flex flex-col xl:flex-row xl:items-center justify-between gap-4 mb-6 shrink-0">
        <div class="flex flex-col sm:flex-row sm:items-center justify-between xl:justify-start gap-4">
            <h1 class="text-xl md:text-2xl font-black text-slate-800 tracking-tight">Inventaris Backoffice</h1>
            <div class="bg-slate-200/80 p-1 rounded-xl flex gap-1 self-start sm:self-auto border border-slate-300/35">
                <button type="button" class="px-3 py-1.5 rounded-lg text-xs font-bold transition-all cursor-pointer {viewMode === 'table' ? 'bg-white shadow-xs text-blue-600' : 'text-slate-600 hover:text-slate-800'}" onclick={() => viewMode = 'table'}>
                    📊 Tabel
                </button>
                <button type="button" class="px-3 py-1.5 rounded-lg text-xs font-bold transition-all cursor-pointer {viewMode === 'card' ? 'bg-white shadow-xs text-blue-600' : 'text-slate-600 hover:text-slate-800'}" onclick={() => viewMode = 'card'}>
                    🃏 Kartu
                </button>
            </div>
        </div>
        <div class="flex flex-wrap gap-2 items-center w-full xl:w-auto">
            <button class="bg-blue-600 text-white px-3.5 py-2.5 rounded-xl font-bold hover:bg-blue-700 transition text-xs flex-grow sm:flex-initial h-12 flex items-center justify-center cursor-pointer shadow-sm shadow-blue-500/10" onclick={() => showAddProductModal = true}>
                + Tambah Produk
            </button>
            <button class="bg-emerald-600 text-white px-3.5 py-2.5 rounded-xl font-bold hover:bg-emerald-700 transition text-xs flex-grow sm:flex-initial h-12 flex items-center justify-center cursor-pointer shadow-sm shadow-emerald-500/10" onclick={triggerFileInput}>
                📥 Impor CSV
            </button>
            <button class="bg-slate-200 text-slate-700 px-3.5 py-2.5 rounded-xl font-bold hover:bg-slate-300 transition text-xs flex-grow sm:flex-initial h-12 flex items-center justify-center cursor-pointer" onclick={downloadSampleCsv}>
                📄 Format CSV
            </button>
            <input type="file" bind:this={fileInput} onchange={handleCsvUpload} accept=".csv" style="display: none;" />
            <a href="/pos" class="btn-outline px-3.5 py-2.5 rounded-xl font-bold border border-slate-300 text-slate-700 hover:bg-slate-100 transition text-xs flex-grow sm:flex-initial h-12 flex items-center justify-center text-center no-underline cursor-pointer">
                Ke Kasir
            </a>
        </div>
    </div>

    <!-- Low Stock Alert Widget -->
    {#if lowStockItems.length > 0}
        <div class="bg-red-50 border-l-4 border-red-500 p-4 rounded-xl shadow-xs shrink-0 mb-6 flex flex-col sm:flex-row items-start gap-3">
            <div class="text-red-500 text-xl font-bold mt-0.5 shrink-0">
                ⚠️
            </div>
            <div class="flex-1">
                <h2 class="font-bold text-red-800 text-sm md:text-base">Peringatan: Ada {lowStockItems.length} produk di bawah batas minimum stok!</h2>
                <p class="text-xs text-red-600 mt-1 mb-2">Silakan periksa daftar bahan baku/produk di bawah ini untuk menghindari kehabisan stok penjualan:</p>
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-2.5 max-h-40 overflow-y-auto pr-2 mt-2">
                    {#each lowStockItems as item}
                        <div class="bg-white/80 border border-red-200/50 rounded-lg p-2.5 flex items-center justify-between text-xs shadow-2xs hover:shadow-xs transition duration-200">
                            <div>
                                <span class="font-bold text-slate-800 line-clamp-1">{item.name}</span>
                                <span class="text-slate-400 text-[10px] block font-mono">{item.sku}</span>
                            </div>
                            <div class="text-right whitespace-nowrap pl-2">
                                <span class="bg-red-100 text-red-800 px-2 py-0.5 rounded-full font-extrabold text-[10px] block">
                                    Stok: {item.qty_on_hand}
                                </span>
                                <span class="text-[10px] text-slate-400 block mt-0.5">Min: {item.min_qty}</span>
                            </div>
                        </div>
                    {/each}
                </div>
            </div>
        </div>
    {/if}

    {#if viewMode === 'table'}
        <div class="bg-white border border-slate-200 rounded-2xl flex-1 overflow-hidden flex flex-col shadow-xs">
            <div class="overflow-auto flex-1">
                <table class="w-full text-left border-collapse min-w-[800px]">
                    <thead class="bg-slate-50 border-b border-slate-200">
                        <tr>
                            <th class="p-4 font-extrabold text-slate-600 text-xs uppercase tracking-wider">Produk</th>
                            <th class="p-4 font-extrabold text-slate-600 text-xs uppercase tracking-wider">SKU</th>
                            <th class="p-4 font-extrabold text-slate-600 text-xs uppercase tracking-wider">Stok</th>
                            <th class="p-4 font-extrabold text-slate-600 text-xs uppercase tracking-wider">Min. Stok</th>
                            <th class="p-4 font-extrabold text-slate-600 text-xs uppercase tracking-wider text-center">Aksi</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-slate-100">
                        {#each products as product}
                            <tr class="hover:bg-slate-50/50 transition-colors">
                                <td class="p-4 flex items-center gap-3">
                                    {#if product.image_url}
                                        <img src={product.image_url} class="w-10 h-10 object-cover rounded-lg shadow-xs border border-slate-200/50" alt={product.name} />
                                    {:else}
                                        <div class="w-10 h-10 bg-blue-50 text-blue-500 font-extrabold flex items-center justify-center rounded-lg shadow-xs border border-blue-100/50">
                                            {product.name.charAt(0).toUpperCase()}
                                        </div>
                                    {/if}
                                    <div class="flex items-center gap-2">
                                        <span class="font-bold text-slate-800 text-sm">{product.name}</span>
                                        {#if product.is_ingredient}
                                            <span class="bg-indigo-50 text-indigo-700 border border-indigo-100 text-[10px] px-2 py-0.5 rounded-full font-bold">Bahan Baku</span>
                                        {/if}
                                    </div>
                                </td>
                                <td class="p-4 text-xs font-mono text-slate-500">{product.sku}</td>
                                <td class="p-4">
                                    <span class="inline-block px-3 py-1 rounded-full font-bold text-xs"
                                          class:bg-red-50={product.qty_on_hand <= product.min_qty}
                                          class:text-red-600={product.qty_on_hand <= product.min_qty}
                                          class:bg-blue-50={product.qty_on_hand > product.min_qty}
                                          class:text-blue-600={product.qty_on_hand > product.min_qty}>
                                        {product.qty_on_hand}
                                    </span>
                                </td>
                                <td class="p-4">
                                    {#if product.is_ingredient}
                                        <div class="text-xs text-slate-700">
                                            <span class="font-bold text-indigo-700">{product.min_qty}</span>
                                            <span class="text-slate-400 text-[10px] block font-normal leading-tight">
                                                Faktor: {product.min_stock_factor}<br>
                                                Buffer: {product.buffer_stock}<br>
                                                Lead Time: {product.lead_time_days} Hari
                                            </span>
                                        </div>
                                    {:else}
                                        <span class="text-xs text-slate-700 font-bold">{product.min_qty}</span>
                                    {/if}
                                </td>
                                <td class="p-4 text-center">
                                    <div class="inline-flex gap-1.5 items-center">
                                        <!-- Group 1: Primary actions (fixed widths for alignment) -->
                                        <button
                                            class="w-[72px] bg-blue-600 hover:bg-blue-700 text-white px-2 py-2 rounded-lg text-xs font-bold transition flex items-center justify-center gap-1 cursor-pointer"
                                            onclick={() => openViewModal(product)}
                                        >👁 Detail</button>
                                        <button
                                            class="w-[60px] bg-amber-500 hover:bg-amber-600 text-white px-2 py-2 rounded-lg text-xs font-bold transition flex items-center justify-center gap-1 cursor-pointer"
                                            onclick={() => openEditModal(product)}
                                        >✏ Edit</button>

                                        <!-- Resep OR invisible spacer — same width so columns stay aligned -->
                                        {#if !product.is_ingredient}
                                            <button
                                                class="w-[64px] bg-indigo-600 hover:bg-indigo-700 text-white px-2 py-2 rounded-lg text-xs font-bold transition flex items-center justify-center gap-1 cursor-pointer"
                                                onclick={() => openRecipeModal(product)}
                                            >🍳 Resep</button>
                                        {:else}
                                            <span class="w-[64px] inline-block"></span>
                                        {/if}

                                        <!-- Divider -->
                                        <span class="w-px h-5 bg-slate-200 mx-0.5"></span>

                                        <!-- Group 2: Stock actions (all same width) -->
                                        <button class="w-[46px] bg-green-50 hover:bg-green-100 text-green-700 py-2 rounded-lg text-xs font-bold transition cursor-pointer" onclick={() => openModal('in', product)}>+ In</button>
                                        <button class="w-[54px] bg-yellow-50 hover:bg-yellow-100 text-yellow-700 py-2 rounded-lg text-xs font-bold transition cursor-pointer" onclick={() => openModal('adjust', product)}>Adjust</button>
                                        <button class="w-[58px] bg-purple-50 hover:bg-purple-100 text-purple-700 py-2 rounded-lg text-xs font-bold transition cursor-pointer" onclick={() => openModal('opname', product)}>Opname</button>
                                        <button class="w-[60px] bg-slate-100 hover:bg-slate-200 text-slate-700 py-2 rounded-lg text-xs font-bold transition cursor-pointer" onclick={() => openModal('transfer', product)}>Transfer</button>
                                    </div>
                                </td>
                            </tr>
                        {/each}
                        {#if products.length === 0}
                            <tr>
                                <td colspan="4" class="p-8 text-center text-slate-450 text-sm">
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
                            <div class="flex items-center gap-1.5 mt-1">
                                <h3 class="font-bold text-slate-800 text-base line-clamp-1">{product.name}</h3>
                                {#if product.is_ingredient}
                                    <span class="bg-indigo-55 bg-opacity-10 text-indigo-700 border border-indigo-100 text-[9px] px-1.5 py-0.5 rounded-full font-bold whitespace-nowrap">Bahan Baku</span>
                                {/if}
                            </div>
                            <span class="text-xs text-slate-500 mt-0.5">SKU: {product.sku}</span>
                            <span class="text-xs text-slate-500 mt-0.5">
                                {#if product.is_ingredient}
                                    Min. Stok: <strong class="text-indigo-700 font-bold">{product.min_qty}</strong>
                                    <span class="text-slate-400 text-[10px] block font-normal leading-tight mt-0.5">
                                        Faktor: {product.min_stock_factor} | Buffer: {product.buffer_stock} | LT: {product.lead_time_days}d
                                    </span>
                                {:else}
                                    Min. Stok: <strong class="text-slate-750">{product.min_qty}</strong>
                                {/if}
                            </span>
                            
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
                                    <button class="flex-1 bg-blue-600 hover:bg-blue-700 text-white py-1.5 rounded-lg text-xs font-bold transition flex justify-center items-center gap-1" onclick={() => openViewModal(product)}>
                                        👁 Detail
                                    </button>
                                    <button class="flex-1 bg-amber-500 hover:bg-amber-600 text-white py-1.5 rounded-lg text-xs font-bold transition flex justify-center items-center gap-1" onclick={() => openEditModal(product)}>
                                        ✏ Edit
                                    </button>
                                </div>
                                {#if !product.is_ingredient}
                                    <button class="w-full bg-indigo-600 hover:bg-indigo-700 text-white py-1.5 rounded-lg text-xs font-bold transition flex justify-center items-center gap-1 cursor-pointer" onclick={() => openRecipeModal(product)}>
                                        🍳 Kelola Resep
                                    </button>
                                {/if}
                                <div class="grid grid-cols-4 gap-1 mt-1 text-center">
                                    <button class="bg-green-50 hover:bg-green-100 text-green-700 py-1 rounded text-[10px] font-bold" onclick={() => openModal('in', product)}>+ In</button>
                                    <button class="bg-yellow-50 hover:bg-yellow-100 text-yellow-700 py-1 rounded text-[10px] font-bold" onclick={() => openModal('adjust', product)}>Adj</button>
                                    <button class="bg-purple-50 hover:bg-purple-100 text-purple-700 py-1 rounded text-[10px] font-bold" onclick={() => openModal('opname', product)}>Opn</button>
                                    <button class="bg-gray-100 hover:bg-gray-200 text-gray-700 py-1 rounded text-[10px] font-bold" onclick={() => openModal('transfer', product)}>Trf</button>
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
            <button class="btn-outline w-full p-3" onclick={() => showModal = false}>Batal</button>
            <button class="btn-primary w-full p-3 font-bold" onclick={submitAction}>Simpan</button>
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
        <select id="newCategory" bind:value={newCategoryId} class="w-full p-2 border rounded mb-3 bg-white">
            <option value="">-- Tanpa Kategori (Global) --</option>
            {#each categoryOptions as cat}
                <option value={cat.id}>{cat.name}</option>
            {/each}
        </select>

        <span class="block font-bold mb-1 text-sm">Gambar Produk</span>
        <div class="flex items-center gap-4 mb-3 p-3 border rounded-xl bg-slate-50 border-slate-200 shadow-sm">
            {#if newImageUrl}
                <div class="relative w-16 h-16 border border-slate-200 rounded-lg overflow-hidden bg-white shadow-xs">
                    <img src={newImageUrl} class="w-full h-full object-cover" alt="Preview" />
                    <button type="button" class="absolute top-0 right-0 bg-red-600 text-white rounded-bl-lg p-1 text-[10px] font-bold hover:bg-red-700 transition" onclick={() => newImageUrl = ''}>
                        ✕
                    </button>
                </div>
            {:else}
                <div class="w-16 h-16 bg-slate-200 text-slate-400 flex items-center justify-center rounded-lg border border-dashed border-slate-300">
                    📷
                </div>
            {/if}
            <div>
                <button type="button" class="bg-white border border-slate-300 hover:bg-slate-50 text-slate-700 px-3 py-1.5 rounded-lg text-xs font-bold transition" onclick={() => imageInput.click()}>
                    Pilih File Gambar
                </button>
                <input type="file" bind:this={imageInput} onchange={handleImageFileChange} accept="image/*" style="display: none;" />
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

        <div class="flex flex-col gap-2 mb-4 mt-2">
            <div class="flex items-center gap-2">
                <input id="newIsIngredient" type="checkbox" bind:checked={newIsIngredient} class="w-4 h-4" />
                <label for="newIsIngredient" class="text-sm font-bold select-none cursor-pointer text-slate-700">Bahan Baku / Ingredient (Tidak Dijual Langsung)</label>
            </div>
            <div class="flex items-center gap-2">
                <input id="newTrackStock" type="checkbox" bind:checked={newTrackStock} class="w-4 h-4" />
                <label for="newTrackStock" class="text-sm font-bold select-none cursor-pointer text-slate-700">Lacak & Kelola Stok Inventaris</label>
            </div>
        </div>

        {#if newTrackStock}
        <div class="space-y-3 mb-6">
            <div>
                <label class="block font-bold mb-1 text-sm" for="newInitialQty">Jumlah Stok Awal</label>
                <input id="newInitialQty" type="number" bind:value={newInitialQty} class="w-full p-2 border rounded" placeholder="0" />
            </div>
            
            {#if newIsIngredient}
                <div class="grid grid-cols-2 gap-3">
                    <div>
                        <label class="block font-bold mb-1 text-sm flex items-center gap-1 cursor-help" for="newMinStockFactor" title="Untuk bahan baku, stok minimal adalah total kebutuhan resep dikali faktor pengali ini. Contoh: Jika total resep butuh 6 butir telur, dan faktor diisi 5, maka minimal stok = 6 x 5 = 30 butir.">
                            Faktor Min. Stok (Pengali) ℹ️
                        </label>
                        <input id="newMinStockFactor" type="number" bind:value={newMinStockFactor} class="w-full p-2 border rounded" placeholder="5" />
                    </div>
                    <div>
                        <label class="block font-bold mb-1 text-sm flex items-center gap-1 cursor-help" for="newBufferStock" title="Stok cadangan pengaman statis. Minimal bahan baku tidak akan pernah kurang dari angka ini.">
                            Buffer Stok (Statis) ℹ️
                        </label>
                        <input id="newBufferStock" type="number" bind:value={newBufferStock} class="w-full p-2 border rounded" placeholder="10" />
                    </div>
                </div>
                <div>
                    <label class="block font-bold mb-1 text-sm flex items-center gap-1 cursor-help" for="newLeadTimeDays" title="Waktu tunggu pengiriman dari supplier (dalam hari).">
                        Durasi Kirim Supplier (Hari) ℹ️
                    </label>
                    <input id="newLeadTimeDays" type="number" bind:value={newLeadTimeDays} class="w-full p-2 border rounded" placeholder="2" />
                </div>
            {:else}
                <div>
                    <label class="block font-bold mb-1 text-sm" for="newBufferStock">Stok Minimal</label>
                    <input id="newBufferStock" type="number" bind:value={newBufferStock} class="w-full p-2 border rounded" placeholder="10" />
                </div>
            {/if}
        </div>
        {/if}

        <div class="flex gap-2">
            <button class="btn-outline w-full p-3" onclick={() => showAddProductModal = false}>Batal</button>
            <button class="btn-primary w-full p-3 font-bold bg-blue-600 hover:bg-blue-700 text-white" onclick={addProduct}>Simpan Produk</button>
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
        <select id="editCategory" bind:value={editCategoryId} class="w-full p-2 border rounded mb-3 bg-white">
            <option value="">-- Tanpa Kategori (Global) --</option>
            {#each categoryOptions as cat}
                <option value={cat.id}>{cat.name}</option>
            {/each}
        </select>

        <span class="block font-bold mb-1 text-sm">Gambar Produk</span>
        <div class="flex items-center gap-4 mb-3 p-3 border rounded-xl bg-slate-50 border-slate-200 shadow-sm">
            {#if editImageUrl}
                <div class="relative w-16 h-16 border border-slate-200 rounded-lg overflow-hidden bg-white shadow-xs">
                    <img src={editImageUrl} class="w-full h-full object-cover" alt="Preview" />
                    <button type="button" class="absolute top-0 right-0 bg-red-600 text-white rounded-bl-lg p-1 text-[10px] font-bold hover:bg-red-700 transition" onclick={() => editImageUrl = ''}>
                        ✕
                    </button>
                </div>
            {:else}
                <div class="w-16 h-16 bg-slate-200 text-slate-400 flex items-center justify-center rounded-lg border border-dashed border-slate-300">
                    📷
                </div>
            {/if}
            <div>
                <button type="button" class="bg-white border border-slate-300 hover:bg-slate-50 text-slate-700 px-3 py-1.5 rounded-lg text-xs font-bold transition" onclick={() => editImageInput.click()}>
                    Pilih File Gambar
                </button>
                <input type="file" bind:this={editImageInput} onchange={handleEditImageFileChange} accept="image/*" style="display: none;" />
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

        <div class="flex flex-col gap-2 mb-6 mt-2">
            <div class="flex items-center gap-2">
                <input id="editIsIngredient" type="checkbox" bind:checked={editIsIngredient} class="w-4 h-4" />
                <label for="editIsIngredient" class="text-sm font-bold select-none cursor-pointer text-slate-700">Bahan Baku / Ingredient (Tidak Dijual Langsung)</label>
            </div>
            <div class="flex items-center gap-2">
                <input id="editTrackStock" type="checkbox" bind:checked={editTrackStock} class="w-4 h-4" />
                <label for="editTrackStock" class="text-sm font-bold select-none cursor-pointer text-slate-700">Lacak & Kelola Stok Inventaris</label>
            </div>
        </div>

        {#if editTrackStock}
        <div class="space-y-3 mb-6">
            {#if editIsIngredient}
                <div class="grid grid-cols-2 gap-3">
                    <div>
                        <label class="block font-bold mb-1 text-sm flex items-center gap-1 cursor-help" for="editMinStockFactor" title="Untuk bahan baku, stok minimal adalah total kebutuhan resep dikali faktor pengali ini. Contoh: Jika total resep butuh 6 butir telur, dan faktor diisi 5, maka minimal stok = 6 x 5 = 30 butir.">
                            Faktor Min. Stok (Pengali) ℹ️
                        </label>
                        <input id="editMinStockFactor" type="number" bind:value={editMinStockFactor} class="w-full p-2 border rounded" placeholder="5" />
                    </div>
                    <div>
                        <label class="block font-bold mb-1 text-sm flex items-center gap-1 cursor-help" for="editBufferStock" title="Stok cadangan pengaman statis. Minimal bahan baku tidak akan pernah kurang dari angka ini.">
                            Buffer Stok (Statis) ℹ️
                        </label>
                        <input id="editBufferStock" type="number" bind:value={editBufferStock} class="w-full p-2 border rounded" placeholder="10" />
                    </div>
                </div>
                <div>
                    <label class="block font-bold mb-1 text-sm flex items-center gap-1 cursor-help" for="editLeadTimeDays" title="Waktu tunggu pengiriman dari supplier (dalam hari).">
                        Durasi Kirim Supplier (Hari) ℹ️
                    </label>
                    <input id="editLeadTimeDays" type="number" bind:value={editLeadTimeDays} class="w-full p-2 border rounded" placeholder="2" />
                </div>
            {:else}
                <div>
                    <label class="block font-bold mb-1 text-sm" for="editBufferStock">Stok Minimal</label>
                    <input id="editBufferStock" type="number" bind:value={editBufferStock} class="w-full p-2 border rounded" placeholder="10" />
                </div>
            {/if}
        </div>
        {/if}

        <div class="flex gap-2">
            <button class="btn-outline w-full p-3" onclick={() => showEditModal = false}>Batal</button>
            <button class="btn-primary w-full p-3 font-bold bg-amber-500 hover:bg-amber-600 text-white" onclick={submitEdit}>Simpan Perubahan</button>
        </div>
    </div>
</div>
{/if}

<!-- VIEW PRODUCT MODAL -->
{#if showViewModal}
<div class="modal-backdrop fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
    <div class="modal-content bg-white p-6 rounded-2xl shadow-xl w-full overflow-y-auto max-h-[90vh]" style="max-width: 950px; width: 95%;">
        <div class="flex justify-between items-center mb-6 pb-3 border-b">
            <h2 class="text-xl font-bold text-slate-800">Detail & Riwayat Stok Produk</h2>
            <button class="text-slate-400 hover:text-slate-600 text-xl font-bold" onclick={() => showViewModal = false}>✕</button>
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
                        <div class="flex justify-between text-sm py-1.5 border-b border-slate-200">
                            <span class="text-slate-500 font-medium font-bold text-indigo-700">Stok Minimal (Hasil)</span>
                            <span class="font-bold text-indigo-700">
                                {selectedViewProduct?.min_qty}
                            </span>
                        </div>
                        {#if selectedViewProduct?.is_ingredient}
                            <div class="flex justify-between text-sm py-1.5 border-b border-slate-200">
                                <span class="text-slate-500 font-medium">Faktor Min. Stok</span>
                                <span class="font-bold text-slate-800">
                                    {selectedViewProduct?.min_stock_factor}
                                </span>
                            </div>
                            <div class="flex justify-between text-sm py-1.5 border-b border-slate-200">
                                <span class="text-slate-500 font-medium">Buffer Stok (Statis)</span>
                                <span class="font-bold text-slate-800">
                                    {selectedViewProduct?.buffer_stock}
                                </span>
                            </div>
                            <div class="flex justify-between text-sm py-1.5 border-b border-slate-200">
                                <span class="text-slate-500 font-medium">Durasi Kirim Supplier</span>
                                <span class="font-bold text-slate-800">
                                    {selectedViewProduct?.lead_time_days} Hari
                                </span>
                            </div>
                        {:else}
                            <div class="flex justify-between text-sm py-1.5 border-b border-slate-200">
                                <span class="text-slate-500 font-medium">Stok Minimal (Statis)</span>
                                <span class="font-bold text-slate-800">
                                    {selectedViewProduct?.buffer_stock}
                                </span>
                            </div>
                        {/if}
                    </div>

                    <div class="mt-6 pt-4 border-t border-slate-200 w-full">
                        <button class="w-full bg-indigo-600 hover:bg-indigo-700 text-white py-2.5 rounded-xl text-xs font-bold transition flex items-center justify-center gap-2 cursor-pointer shadow-sm shadow-indigo-500/10"
                                onclick={openBarcodeModal}>
                            🏷️ Buat QRCode & Barcode
                        </button>
                    </div>
                </div>
            </div>

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
                        <div class="overflow-x-auto overflow-y-auto max-h-[350px] p-0">
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
            <button class="bg-slate-800 hover:bg-slate-900 text-white font-bold px-6 py-2 rounded-xl transition text-sm" onclick={() => showViewModal = false}>
                Tutup
            </button>
        </div>
    </div>
</div>
{/if}

{#if showBarcodeModal}
<div class="modal-backdrop fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 animate-fade-in">
    <div class="modal-content bg-white p-6 rounded-2xl shadow-2xl w-full max-w-2xl overflow-y-auto max-h-[90vh] transition-all transform duration-300">
        <div class="flex justify-between items-center mb-6 pb-3 border-b border-slate-100">
            <div class="flex items-center gap-2">
                <span class="text-xl">🏷️</span>
                <h2 class="text-xl font-bold text-slate-800">Pembuat QR & Barcode</h2>
            </div>
            <button class="text-slate-400 hover:text-slate-650 text-xl font-bold transition-colors cursor-pointer" onclick={() => showBarcodeModal = false}>✕</button>
        </div>

        <div class="space-y-6">
            <!-- Label Preview Panel -->
            <div class="border border-dashed border-slate-300 rounded-2xl p-6 bg-slate-50 flex flex-col min-h-[280px] shadow-inner">
                <span class="text-[10px] text-slate-400 uppercase font-bold tracking-wider mb-4 text-center block">Pratinjau Label Cetak</span>
                
                <div class="grid grid-cols-1 md:grid-cols-2 gap-6 w-full">
                    <!-- Grid 1: QR Code Label -->
                    <div class="flex flex-col items-center">
                        <span class="text-[10px] bg-blue-50 text-blue-700 px-2.5 py-1 rounded-full font-bold mb-3 border border-blue-200/50">Label QR Code</span>
                        <div class="bg-white border border-slate-200 p-5 rounded-2xl flex flex-col items-center text-center shadow-sm w-full max-w-[240px] min-h-[220px] justify-between transition-all hover:shadow-md hover:border-blue-300 duration-300">
                            <div>
                                {#if includeName}
                                    <div class="font-extrabold text-xs text-slate-800 line-clamp-2 max-w-full leading-tight">{selectedViewProduct?.name}</div>
                                {/if}
                                {#if includePrice}
                                    <div class="font-bold text-blue-600 text-xs mt-1">Rp {selectedViewProduct?.price.toLocaleString('id-ID')}</div>
                                {/if}
                            </div>
                            <div class="flex items-center justify-center my-3 w-full">
                                <canvas bind:this={qrcodeCanvas} class="w-24 h-24 border border-slate-100 p-1.5 bg-white rounded-xl shadow-xs"></canvas>
                            </div>
                            <div class="text-[9px] font-mono text-slate-400 bg-slate-50 px-2 py-0.5 rounded border border-slate-150">SKU: {selectedViewProduct?.sku || '-'}</div>
                        </div>
                    </div>

                    <!-- Grid 2: Barcode Label -->
                    <div class="flex flex-col items-center">
                        <span class="text-[10px] bg-emerald-50 text-emerald-700 px-2.5 py-1 rounded-full font-bold mb-3 border border-emerald-200/50">Label Barcode</span>
                        <div class="bg-white border border-slate-200 p-5 rounded-2xl flex flex-col items-center text-center shadow-sm w-full max-w-[240px] min-h-[220px] justify-between transition-all hover:shadow-md hover:border-emerald-300 duration-300">
                            <div>
                                {#if includeName}
                                    <div class="font-extrabold text-xs text-slate-800 line-clamp-2 max-w-full leading-tight">{selectedViewProduct?.name}</div>
                                {/if}
                                {#if includePrice}
                                    <div class="font-bold text-emerald-600 text-xs mt-1">Rp {selectedViewProduct?.price.toLocaleString('id-ID')}</div>
                                {/if}
                            </div>
                            <div class="flex items-center justify-center my-3 w-full">
                                <canvas bind:this={barcodeCanvas} class="max-w-full h-12 object-contain"></canvas>
                            </div>
                            <div class="text-[9px] font-mono text-slate-400 bg-slate-50 px-2 py-0.5 rounded border border-slate-150">SKU: {selectedViewProduct?.sku || '-'}</div>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Customization Checkboxes -->
            <div class="bg-slate-50 p-4 rounded-xl border border-slate-200 space-y-3 shadow-xs">
                <span class="text-xs font-bold text-slate-700 block mb-1">Pengaturan Tampilan Label:</span>
                <div class="flex items-center gap-6">
                    <label class="flex items-center gap-2 text-sm text-slate-650 cursor-pointer font-bold select-none hover:text-slate-800 transition-colors">
                        <input type="checkbox" bind:checked={includeName} class="rounded border-slate-300 text-indigo-600 focus:ring-indigo-500 w-4 h-4 cursor-pointer" />
                        Tampilkan Nama Produk
                    </label>
                    <label class="flex items-center gap-2 text-sm text-slate-650 cursor-pointer font-bold select-none hover:text-slate-800 transition-colors">
                        <input type="checkbox" bind:checked={includePrice} class="rounded border-slate-300 text-indigo-600 focus:ring-indigo-500 w-4 h-4 cursor-pointer" />
                        Tampilkan Harga Jual
                    </label>
                </div>
            </div>

            <!-- Action Buttons -->
            <div class="grid grid-cols-2 gap-4 pt-2">
                <button class="bg-blue-50 hover:bg-blue-100 text-blue-700 font-extrabold py-3 px-4 rounded-xl transition text-xs flex items-center justify-center gap-2 border border-blue-200/60 shadow-xs cursor-pointer" onclick={downloadQrCode}>
                    📥 Unduh QR Code
                </button>
                <button class="bg-emerald-50 hover:bg-emerald-100 text-emerald-700 font-extrabold py-3 px-4 rounded-xl transition text-xs flex items-center justify-center gap-2 border border-emerald-200/60 shadow-xs cursor-pointer" onclick={downloadBarcode}>
                    📥 Unduh Barcode
                </button>
            </div>
        </div>

        <!-- Footer -->
        <div class="mt-8 pt-4 border-t border-slate-100 flex justify-between gap-3">
            <button class="bg-slate-150 hover:bg-slate-200 text-slate-700 font-bold px-6 py-3 rounded-xl transition text-xs cursor-pointer" onclick={() => showBarcodeModal = false}>
                Batal
            </button>
            <button class="bg-indigo-600 hover:bg-indigo-700 text-white font-extrabold px-8 py-3 rounded-xl transition text-xs flex items-center gap-2 cursor-pointer shadow-md shadow-indigo-500/20" onclick={printLabel}>
                🖨️ Cetak Label
            </button>
        </div>
    </div>
</div>
{/if}
{#if selectedViewProduct && showBarcodeModal}
<div id="print-label-section">
    <div class="print-label-card">
        {#if includeName}
            <div class="print-title">{selectedViewProduct.name}</div>
        {/if}
        {#if includePrice}
            <div class="print-price">Rp {selectedViewProduct.price.toLocaleString('id-ID')}</div>
        {/if}
        {#if qrPrintDataUrl}
            <img class="print-qr-code" src={qrPrintDataUrl} alt="QR" />
        {/if}
        <div class="print-sku">SKU: {selectedViewProduct.sku || '-'}</div>
    </div>
    <div class="print-label-card page-break">
        {#if includeName}
            <div class="print-title">{selectedViewProduct.name}</div>
        {/if}
        {#if includePrice}
            <div class="print-price">Rp {selectedViewProduct.price.toLocaleString('id-ID')}</div>
        {/if}
        {#if barPrintDataUrl}
            <img class="print-barcode" src={barPrintDataUrl} alt="Barcode" />
        {/if}
        <div class="print-sku">SKU: {selectedViewProduct.sku || '-'}</div>
    </div>
</div>
{/if}

{#if showRecipeModal && recipeTargetProduct}
<div class="modal-backdrop fixed inset-0 bg-black/60 backdrop-blur-xs flex items-center justify-center z-50 animate-fade-in">
    <div class="modal-content bg-white p-6 rounded-2xl shadow-2xl w-full max-w-3xl overflow-y-auto max-h-[90vh] transition-all transform duration-300">
        <!-- Header -->
        <div class="flex justify-between items-center mb-6 pb-4 border-b border-slate-100">
            <div class="flex items-center gap-3">
                <span class="text-2xl">🍳</span>
                <div>
                    <h2 class="text-xl font-extrabold text-slate-800">Kelola Resep & Bahan Baku</h2>
                    <p class="text-xs text-slate-500 mt-0.5">Atur resep pembentuk untuk menu <span class="font-semibold text-slate-700">{recipeTargetProduct.name}</span></p>
                </div>
            </div>
            <button class="text-slate-400 hover:text-slate-655 text-xl font-bold transition-colors cursor-pointer" onclick={() => showRecipeModal = false}>✕</button>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-5 gap-6">
            <!-- Left Side: Recipe Ingredient List (3 cols) -->
            <div class="md:col-span-3 border border-slate-200 rounded-xl p-4 bg-slate-50/50 flex flex-col justify-between min-h-[300px]">
                <div>
                    <h3 class="text-sm font-bold text-slate-700 mb-3 flex items-center gap-1.5">
                        <span>📋</span> Daftar Bahan Baku Terpilih
                    </h3>
                    
                    {#if recipeIngredients.length === 0}
                        <div class="flex flex-col items-center justify-center py-12 text-center">
                            <span class="text-4xl mb-2">🌾</span>
                            <p class="text-sm text-slate-600 font-medium">Belum ada bahan baku terhubung</p>
                            <p class="text-xs text-slate-400 mt-1 max-w-[220px]">Tambahkan bahan baku di panel sebelah kanan untuk membentuk resep menu ini.</p>
                        </div>
                    {:else}
                        <div class="space-y-2 max-h-[280px] overflow-y-auto pr-1">
                            {#each recipeIngredients as ing}
                                <div class="flex items-center justify-between bg-white border border-slate-200 p-3 rounded-xl shadow-xs transition hover:border-slate-300">
                                    <div class="flex flex-col">
                                        <span class="font-bold text-xs text-slate-800">{ing.name}</span>
                                        <span class="text-[10px] text-slate-400 font-mono">SKU: {ing.sku}</span>
                                    </div>
                                    <div class="flex items-center gap-3">
                                        <div class="bg-blue-50 text-blue-700 px-3 py-1 rounded-lg text-xs font-extrabold border border-blue-100/50">
                                            {ing.qty} {ing.unit}
                                        </div>
                                        <button class="text-red-500 hover:text-red-700 p-1.5 hover:bg-red-50 rounded-lg transition-colors cursor-pointer" onclick={() => removeRecipeIngredient(ing.ingredient_id)}>
                                            🗑️
                                        </button>
                                    </div>
                                </div>
                            {/each}
                        </div>
                    {/if}
                </div>

                <div class="mt-4 pt-3 border-t border-slate-200/80 text-[10px] text-slate-400 flex items-start gap-1">
                    <span>💡</span>
                    <span>Stok bahan baku ini akan otomatis dipotong setiap kali menu ini terjual pada saat checkout transaksi.</span>
                </div>
            </div>

            <!-- Right Side: Add Ingredient Form (2 cols) -->
            <div class="md:col-span-2 space-y-4">
                <div class="border border-slate-200 rounded-xl p-4 bg-white shadow-xs">
                    <h3 class="text-sm font-bold text-slate-700 mb-4 flex items-center gap-1.5">
                        <span>➕</span> Hubungkan Bahan Baku
                    </h3>

                    <!-- Ingredient Select -->
                    <label class="block text-xs font-bold text-slate-600 mb-1.5" for="ingredientSelect">Pilih Bahan Baku *</label>
                    <select id="ingredientSelect" bind:value={selectedAddIngredientId} class="w-full p-2.5 border border-slate-300 rounded-xl text-xs bg-white mb-4 focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-all">
                        <option value="">-- Pilih Bahan Baku --</option>
                        {#each products.filter(p => p.is_ingredient) as ing}
                            <option value={ing.id}>{ing.name} ({ing.sku})</option>
                        {/each}
                    </select>

                    <div class="grid grid-cols-2 gap-3 mb-4">
                        <!-- Qty Input -->
                        <div>
                            <label class="block text-xs font-bold text-slate-600 mb-1.5" for="ingredientQty">Jumlah *</label>
                            <input id="ingredientQty" type="number" step="any" bind:value={selectedAddIngredientQty} class="w-full p-2.5 border border-slate-300 rounded-xl text-xs focus:ring-2 focus:ring-blue-500 transition-all" placeholder="Contoh: 150" />
                        </div>
                        
                        <!-- Unit Select -->
                        <div>
                            <label class="block text-xs font-bold text-slate-600 mb-1.5" for="ingredientUnit">Satuan</label>
                            <select id="ingredientUnit" bind:value={selectedAddIngredientUnit} class="w-full p-2.5 border border-slate-300 rounded-xl text-xs bg-white focus:ring-2 focus:ring-blue-500 transition-all">
                                <option value="gr">gram (gr)</option>
                                <option value="ml">mililiter (ml)</option>
                                <option value="pcs">pieces (pcs)</option>
                                <option value="butir">butir</option>
                                <option value="sachet">sachet</option>
                                <option value="kg">kilogram (kg)</option>
                                <option value="liter">liter (l)</option>
                            </select>
                        </div>
                    </div>

                    <button class="w-full bg-blue-600 hover:bg-blue-700 text-white font-bold py-2.5 rounded-xl transition text-xs flex items-center justify-center gap-1.5 shadow-sm shadow-blue-500/10 cursor-pointer" onclick={addRecipeIngredient}>
                        <span>Add</span> Tambahkan
                    </button>
                </div>
            </div>
        </div>

        <!-- Footer Actions -->
        <div class="mt-6 pt-4 border-t border-slate-100 flex justify-end gap-3">
            <button class="bg-slate-100 hover:bg-slate-200 text-slate-700 font-bold px-5 py-2.5 rounded-xl transition text-xs cursor-pointer" onclick={() => showRecipeModal = false}>
                Batal
            </button>
            <button class="bg-blue-600 hover:bg-blue-700 text-white font-bold px-6 py-2.5 rounded-xl transition text-xs shadow-md shadow-blue-500/20 cursor-pointer" onclick={saveRecipe}>
                Simpan Resep
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

    #print-label-section {
        display: none;
    }

    @media print {
        :global(body) {
            background: white !important;
            color: black !important;
        }
        :global(body > *:not(#print-label-section)) {
            display: none !important;
        }
        #print-label-section {
            display: block !important;
            position: absolute;
            left: 0;
            top: 0;
            width: 80mm;
            margin: 0 auto;
        }
        .print-label-card {
            width: 80mm;
            min-height: 50mm;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            text-align: center;
            box-sizing: border-box;
            padding: 15px;
            margin: 0;
            background: white;
        }
        .page-break {
            page-break-before: always;
        }
        .print-title {
            font-size: 14px;
            font-weight: 850;
            margin-bottom: 4px;
            word-break: break-word;
            max-width: 100%;
            line-height: 1.2;
            color: #000;
        }
        .print-price {
            font-size: 13px;
            font-weight: 700;
            color: #000;
            margin-bottom: 10px;
        }
        .print-qr-code {
            width: 110px;
            height: 110px;
        }
        .print-barcode {
            max-width: 220px;
            height: auto;
        }
        .print-sku {
            font-size: 9px;
            font-family: monospace;
            margin-top: 8px;
            color: #64748b;
        }
    }
</style>
