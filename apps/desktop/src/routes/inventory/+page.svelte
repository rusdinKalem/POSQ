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

    // Loading
    let loading = $state(true);

    // View Switcher
    let viewMode: 'table' | 'card' = $state('table');

    // Search & Filters
    let searchQuery = $state('');
    let filterCategory = $state(''); // '' = all
    let filterStock = $state<'all' | 'normal' | 'critical' | 'out'>('all');

    // Filtered products — client-side search by name/SKU + category + stock status
    let filteredProducts = $derived.by(() => {
        const q = searchQuery.trim().toLowerCase();
        return products.filter(p => {
            // Search: match name OR sku
            if (q) {
                const name = p.name.toLowerCase();
                const sku = (p.sku || '').toLowerCase();
                if (!name.includes(q) && !sku.includes(q)) return false;
            }
            // Category filter
            if (filterCategory && p.category_id !== filterCategory) return false;
            // Stock status filter (only meaningful when stock is tracked)
            if (filterStock !== 'all') {
                if (!p.track_stock) return false;
                const qty = p.qty_on_hand;
                const min = p.min_qty;
                if (filterStock === 'out' && qty > 0) return false;
                if (filterStock === 'critical' && (qty <= 0 || qty > min)) return false;
                if (filterStock === 'normal' && qty <= min) return false;
            }
            return true;
        });
    });

    // Inventory summary stats for KPI cards
    let stats = $derived.by(() => {
        let totalSku = products.length;
        let critical = lowStockItems.length;
        let outOfStock = products.filter(p => p.track_stock && p.qty_on_hand <= 0).length;
        let inventoryValue = 0;
        let ingredientCount = 0;
        for (const p of products) {
            if (p.is_ingredient) ingredientCount++;
            if (p.track_stock) {
                const unitCost = p.cost ?? 0;
                inventoryValue += p.qty_on_hand * unitCost;
            }
        }
        return { totalSku, critical, outOfStock, inventoryValue, ingredientCount };
    });

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
        loading = true;
        try {
            // MUST run sequentially! Both endpoints trigger write transactions to 
            // recalculate minimum stocks. Running them concurrently with Promise.all 
            // causes SQLite database locks (SQLITE_BUSY).
            lowStockItems = await invoke<LowStockItem[]>('get_low_stock');
            products = await invoke<Product[]>('get_inventory_products');
            await fetchCategories();
        } catch (e) {
            console.error(e);
            showToast('Gagal mengambil data inventaris: ' + e, 'error');
        } finally {
            loading = false;
        }
    }

    function resetFilters() {
        searchQuery = '';
        filterCategory = '';
        filterStock = 'all';
    }

    function hasActiveFilters() {
        return searchQuery.trim() !== '' || filterCategory !== '' || filterStock !== 'all';
    }

    function openModal(action: string, product: Product) {
        modalAction = action;
        selectedProduct = product;
        actionQty = 0;
        actionReason = '';
        showModal = true;
    }

    // Contextual metadata for the stock-action modal
    let modalMeta = $derived.by(() => {
        switch (modalAction) {
            case 'in':
                return { icon: '📥', title: 'Barang Masuk', sub: 'Tambah stok dari supplier/pembelian', accent: 'emerald', accentRgb: '16,185,129' };
            case 'adjust':
                return { icon: '⚖️', title: 'Penyesuaian Stok', sub: 'Catat selisih (+ atau -) karena rusak/retur', accent: 'amber', accentRgb: '245,158,11' };
            case 'opname':
                return { icon: '🔢', title: 'Stok Opname Fisik', sub: 'Input hasil hitung fisik aktual', accent: 'purple', accentRgb: '147,51,234' };
            case 'transfer':
                return { icon: '🚚', title: 'Transfer Keluar', sub: 'Pindahkan stok ke gudang pusat', accent: 'slate', accentRgb: '100,116,139' };
            default:
                return { icon: '📦', title: 'Aksi Stok', sub: '', accent: 'blue', accentRgb: '37,99,235' };
        }
    });

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
    <!-- Page Header -->
    <div class="flex flex-col xl:flex-row xl:items-center justify-between gap-4 mb-5 shrink-0">
        <div class="flex items-center gap-3">
            <div class="hidden sm:flex w-11 h-11 rounded-xl bg-blue-600 text-white items-center justify-center text-lg shadow-md shadow-blue-500/10 shrink-0">🗃️</div>
            <div>
                <div class="flex items-center gap-2">
                    <h1 class="text-xl md:text-2xl font-black text-slate-800 tracking-tight">Inventaris Backoffice</h1>
                    <span class="bg-slate-200/80 text-slate-600 px-2 py-0.5 rounded-full text-[10px] font-extrabold border border-slate-300/40">{stats.totalSku} Item</span>
                </div>
                <p class="text-xs text-slate-500 mt-0.5">Kelola produk, stok, resep, dan cetak label harga.</p>
            </div>
        </div>
        <div class="flex flex-wrap gap-2 items-center w-full xl:w-auto">
            <button class="bg-blue-600 text-white px-3.5 py-2.5 rounded-xl font-bold hover:bg-blue-700 transition text-xs flex-grow sm:flex-initial h-11 flex items-center justify-center gap-1.5 cursor-pointer shadow-sm shadow-blue-500/10" onclick={() => showAddProductModal = true}>
                <span class="text-sm leading-none">＋</span> Tambah Produk
            </button>
            <button class="bg-emerald-600 text-white px-3.5 py-2.5 rounded-xl font-bold hover:bg-emerald-700 transition text-xs flex-grow sm:flex-initial h-11 flex items-center justify-center gap-1.5 cursor-pointer shadow-sm shadow-emerald-500/10" onclick={triggerFileInput}>
                📥 Impor CSV
            </button>
            <button class="bg-white text-slate-700 px-3.5 py-2.5 rounded-xl font-bold hover:bg-slate-100 transition text-xs flex-grow sm:flex-initial h-11 flex items-center justify-center gap-1.5 cursor-pointer border border-slate-200 shadow-xs" onclick={downloadSampleCsv}>
                📄 Format CSV
            </button>
            <input type="file" bind:this={fileInput} onchange={handleCsvUpload} accept=".csv" style="display: none;" />
            <a href="/pos" class="px-3.5 py-2.5 rounded-xl font-bold border border-slate-300 text-slate-700 hover:bg-slate-100 transition text-xs flex-grow sm:flex-initial h-11 flex items-center justify-center gap-1.5 text-center no-underline cursor-pointer">
                🛒 Ke Kasir
            </a>
        </div>
    </div>

    <!-- KPI Summary Cards -->
    <div class="grid grid-cols-2 lg:grid-cols-4 gap-3 mb-5 shrink-0">
        <div class="bg-white rounded-2xl border border-slate-200 p-4 shadow-xs flex items-center gap-3">
            <div class="w-10 h-10 rounded-xl bg-blue-50 text-blue-600 flex items-center justify-center text-lg shrink-0">📦</div>
            <div class="min-w-0">
                <div class="text-[10px] uppercase font-bold tracking-wider text-slate-400">Total SKU</div>
                <div class="text-xl font-black text-slate-800 leading-tight">{stats.totalSku}</div>
            </div>
        </div>
        <div class="bg-white rounded-2xl border border-slate-200 p-4 shadow-xs flex items-center gap-3">
            <div class="w-10 h-10 rounded-xl bg-amber-50 text-amber-600 flex items-center justify-center text-lg shrink-0">⚠️</div>
            <div class="min-w-0">
                <div class="text-[10px] uppercase font-bold tracking-wider text-slate-400">Stok Kritis</div>
                <div class="text-xl font-black text-amber-600 leading-tight">{stats.critical}<span class="text-xs text-slate-400 font-bold ml-1">{stats.outOfStock > 0 ? `· ${stats.outOfStock} habis` : ''}</span></div>
            </div>
        </div>
        <div class="bg-white rounded-2xl border border-slate-200 p-4 shadow-xs flex items-center gap-3">
            <div class="w-10 h-10 rounded-xl bg-emerald-50 text-emerald-600 flex items-center justify-center text-lg shrink-0">💰</div>
            <div class="min-w-0">
                <div class="text-[10px] uppercase font-bold tracking-wider text-slate-400">Nilai Inventaris</div>
                <div class="text-base font-black text-slate-800 leading-tight truncate">Rp {stats.inventoryValue.toLocaleString('id-ID')}</div>
            </div>
        </div>
        <div class="bg-white rounded-2xl border border-slate-200 p-4 shadow-xs flex items-center gap-3">
            <div class="w-10 h-10 rounded-xl bg-indigo-50 text-indigo-600 flex items-center justify-center text-lg shrink-0">🧂</div>
            <div class="min-w-0">
                <div class="text-[10px] uppercase font-bold tracking-wider text-slate-400">Bahan Baku</div>
                <div class="text-xl font-black text-slate-800 leading-tight">{stats.ingredientCount}</div>
            </div>
        </div>
    </div>

    <!-- Search, Filters & View Switcher Toolbar -->
    <div class="bg-white rounded-2xl border border-slate-200 shadow-xs p-3 flex flex-col lg:flex-row gap-3 mb-4 shrink-0 lg:items-center justify-between">
        <!-- Left Group: Search and Filters -->
        <div class="flex flex-col lg:flex-row flex-grow items-stretch lg:items-center gap-3 min-w-0">
            <!-- Search -->
            <div class="relative w-full lg:flex-1 min-w-0">
                <input
                    type="text"
                    bind:value={searchQuery}
                    placeholder="Cari nama atau SKU produk..."
                    class="w-full h-[42px] pl-3.5 pr-9 border border-slate-200 rounded-xl text-xs font-semibold bg-slate-50 focus:bg-white focus:border-blue-400 focus:ring-2 focus:ring-blue-100 transition-all placeholder:text-slate-400 leading-normal"
                    style="font-size: 12px;"
                />
                {#if searchQuery}
                    <button type="button" class="absolute right-2.5 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-600 text-xs w-6 h-6 flex items-center justify-center rounded-full hover:bg-slate-100 transition cursor-pointer" onclick={() => searchQuery = ''} aria-label="Clear search">✕</button>
                {/if}
            </div>

            <!-- Category Filter -->
            <div class="relative w-full lg:flex-1 min-w-0">
                <select bind:value={filterCategory} class="appearance-none w-full h-[42px] pl-3.5 pr-9 border border-slate-200 rounded-xl text-xs font-semibold bg-slate-50 focus:bg-white focus:border-blue-400 focus:ring-2 focus:ring-blue-100 transition-all cursor-pointer text-slate-700 leading-normal" style="font-size: 12px;">
                    <option value="">Semua Kategori</option>
                    {#each categoryOptions as cat}
                        <option value={cat.id}>{cat.name}</option>
                    {/each}
                </select>
                <div class="absolute right-3 top-1/2 -translate-y-1/2 pointer-events-none text-slate-400">
                    <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="m6 9 6 6 6-6"/></svg>
                </div>
            </div>

            <!-- Stock Status Filter -->
            <div class="relative w-full lg:flex-1 min-w-0">
                <select bind:value={filterStock} class="appearance-none w-full h-[42px] pl-3.5 pr-9 border border-slate-200 rounded-xl text-xs font-semibold bg-slate-50 focus:bg-white focus:border-blue-400 focus:ring-2 focus:ring-blue-100 transition-all cursor-pointer text-slate-700 leading-normal" style="font-size: 12px;">
                    <option value="all">Semua Stok</option>
                    <option value="normal">Stok Normal</option>
                    <option value="critical">Stok Kritis</option>
                    <option value="out">Stok Habis</option>
                </select>
                <div class="absolute right-3 top-1/2 -translate-y-1/2 pointer-events-none text-slate-400">
                    <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="m6 9 6 6 6-6"/></svg>
                </div>
            </div>

            <!-- Reset (only when filtering) -->
            {#if hasActiveFilters()}
                <button type="button" class="w-full lg:w-auto shrink-0 h-[42px] px-4 border border-slate-200 rounded-xl text-xs font-bold text-slate-500 hover:text-rose-600 hover:bg-rose-50 transition cursor-pointer flex items-center justify-center gap-1 leading-normal bg-white" onclick={resetFilters}>
                    ✕ Reset
                </button>
            {/if}
        </div>

        <!-- Right Group: Count & View Switcher -->
        <div class="flex items-center justify-between lg:justify-end gap-3.5 shrink-0 border-t lg:border-t-0 border-slate-100 pt-3 lg:pt-0">
            <!-- Result count -->
            <div class="text-[11px] font-bold text-slate-400 bg-slate-50 px-3 py-2 rounded-xl border border-slate-200/40">
                {filteredProducts.length} / {products.length} produk
            </div>

            <!-- View Switcher -->
            <div class="bg-slate-100 p-1 rounded-xl flex gap-1 border border-slate-200/60">
                <button type="button" class="px-3.5 py-1.5 rounded-lg text-xs font-bold transition-all cursor-pointer {viewMode === 'table' ? 'bg-white shadow-xs text-blue-600' : 'text-slate-500 hover:text-slate-700'}" onclick={() => viewMode = 'table'}>
                    📊 Tabel
                </button>
                <button type="button" class="px-3.5 py-1.5 rounded-lg text-xs font-bold transition-all cursor-pointer {viewMode === 'card' ? 'bg-white shadow-xs text-blue-600' : 'text-slate-500 hover:text-slate-700'}" onclick={() => viewMode = 'card'}>
                    🃏 Kartu
                </button>
            </div>
        </div>
    </div>

    <!-- Low Stock Alert Widget -->
    {#if lowStockItems.length > 0}
        <div class="bg-gradient-to-r from-amber-50 to-red-50 border border-amber-200 rounded-2xl p-4 shadow-xs shrink-0 mb-4">
            <div class="flex items-start gap-3">
                <div class="w-9 h-9 rounded-xl bg-amber-100 text-amber-600 flex items-center justify-center text-lg shrink-0">⚠️</div>
                <div class="flex-1 min-w-0">
                    <h2 class="font-extrabold text-amber-800 text-sm">Peringatan Stok Menipis</h2>
                    <p class="text-xs text-amber-700/80 mt-0.5">Ada <span class="font-bold">{lowStockItems.length} produk</span> di bawah batas minimum. Lakukan pembelian/restock untuk menghindari kehabisan.</p>
                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-2 max-h-36 overflow-y-auto pr-1 mt-3">
                        {#each lowStockItems as item}
                            <div class="bg-white border border-amber-200/70 rounded-xl p-2.5 flex items-center justify-between text-xs shadow-xs hover:shadow-sm hover:border-amber-300 transition cursor-pointer"
                                 role="button" tabindex="0"
                                 onclick={() => {
                                     const p = products.find(x => x.id === item.product_id);
                                     if (p) openViewModal(p);
                                 }}
                                 onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); const p = products.find(x => x.id === item.product_id); if (p) openViewModal(p); } } }>
                                <div class="min-w-0">
                                    <span class="font-bold text-slate-800 line-clamp-1 block">{item.name}</span>
                                    <span class="text-slate-400 text-[10px] block font-mono">{item.sku}</span>
                                </div>
                                <div class="text-right whitespace-nowrap pl-2 shrink-0">
                                    <span class="bg-red-100 text-red-700 px-2 py-0.5 rounded-full font-extrabold text-[10px] block">
                                        {item.qty_on_hand}
                                    </span>
                                    <span class="text-[10px] text-slate-400 block mt-0.5">Min: {item.min_qty}</span>
                                </div>
                            </div>
                        {/each}
                    </div>
                </div>
            </div>
        </div>
    {/if}

    {#if viewMode === 'table'}
        <div class="bg-white border border-slate-200 rounded-2xl flex-1 overflow-hidden flex flex-col shadow-xs">
            <div class="overflow-auto flex-1">
                <table class="w-full text-left border-collapse min-w-[860px]">
                    <thead class="bg-slate-50 border-b border-slate-200">
                        <tr>
                            <th class="p-4 font-extrabold text-slate-500 text-[11px] uppercase tracking-wider">Produk</th>
                            <th class="p-4 font-extrabold text-slate-500 text-[11px] uppercase tracking-wider">SKU</th>
                            <th class="p-4 font-extrabold text-slate-500 text-[11px] uppercase tracking-wider text-right">Harga Jual</th>
                            <th class="p-4 font-extrabold text-slate-500 text-[11px] uppercase tracking-wider">Stok</th>
                            <th class="p-4 font-extrabold text-slate-500 text-[11px] uppercase tracking-wider">Min. Stok</th>
                            <th class="p-4 font-extrabold text-slate-500 text-[11px] uppercase tracking-wider text-center">Aksi</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-slate-100">
                        <!-- Loading skeleton -->
                        {#if loading}
                            {#each Array(6) as _}
                                <tr class="bg-white">
                                    <td class="p-4"><div class="flex items-center gap-3"><div class="w-10 h-10 rounded-lg bg-slate-100 animate-pulse"></div><div class="h-3 w-32 rounded bg-slate-100 animate-pulse"></div></div></td>
                                    <td class="p-4"><div class="h-3 w-20 rounded bg-slate-100 animate-pulse"></div></td>
                                    <td class="p-4"><div class="h-3 w-16 rounded bg-slate-100 animate-pulse ml-auto"></div></td>
                                    <td class="p-4"><div class="h-5 w-12 rounded-full bg-slate-100 animate-pulse"></div></td>
                                    <td class="p-4"><div class="h-3 w-10 rounded bg-slate-100 animate-pulse"></div></td>
                                    <td class="p-4"><div class="h-7 w-48 rounded-lg bg-slate-100 animate-pulse mx-auto"></div></td>
                                </tr>
                            {/each}
                        {:else}
                            {#each filteredProducts as product (product.id)}
                                <tr class="hover:bg-slate-50/70 transition-colors group">
                                    <td class="p-4">
                                        <div class="flex items-center gap-3">
                                            {#if product.image_url}
                                                <img src={product.image_url} class="w-10 h-10 object-cover rounded-lg shadow-xs border border-slate-200/60 shrink-0" alt={product.name} />
                                            {:else}
                                                <div class="w-10 h-10 bg-blue-50 text-blue-500 font-extrabold flex items-center justify-center rounded-lg shadow-xs border border-blue-100/60 shrink-0">
                                                    {product.name.charAt(0).toUpperCase()}
                                                </div>
                                            {/if}
                                            <div class="flex items-center gap-2 min-w-0">
                                                <span class="font-bold text-slate-800 text-sm line-clamp-1">{product.name}</span>
                                                {#if product.is_ingredient}
                                                    <span class="bg-indigo-50 text-indigo-700 border border-indigo-100 text-[10px] px-2 py-0.5 rounded-full font-bold whitespace-nowrap shrink-0">Bahan Baku</span>
                                                {/if}
                                                {#if !product.track_stock}
                                                    <span class="bg-slate-100 text-slate-500 text-[10px] px-2 py-0.5 rounded-full font-bold whitespace-nowrap shrink-0">Tak Lacak</span>
                                                {/if}
                                            </div>
                                        </div>
                                    </td>
                                    <td class="p-4 text-xs font-mono text-slate-500 whitespace-nowrap">{product.sku}</td>
                                    <td class="p-4 text-right whitespace-nowrap">
                                        {#if product.is_ingredient}
                                            <span class="text-xs text-slate-400 italic">—</span>
                                        {:else}
                                            <span class="text-sm font-bold text-slate-700">Rp {product.price.toLocaleString('id-ID')}</span>
                                        {/if}
                                    </td>
                                    <td class="p-4">
                                        {#if !product.track_stock}
                                            <span class="text-xs text-slate-400 italic">N/A</span>
                                        {:else}
                                            {#if product.qty_on_hand <= 0}
                                                <span class="inline-flex items-center gap-1 px-2.5 py-1 rounded-full font-bold text-xs bg-red-100 text-red-700">
                                                    <span class="w-1.5 h-1.5 rounded-full bg-red-500"></span> Habis
                                                </span>
                                            {:else if product.qty_on_hand <= product.min_qty}
                                                <span class="inline-flex items-center gap-1 px-2.5 py-1 rounded-full font-bold text-xs bg-amber-100 text-amber-700">
                                                    <span class="w-1.5 h-1.5 rounded-full bg-amber-500"></span> {product.qty_on_hand} (Kritis)
                                                </span>
                                            {:else}
                                                <span class="inline-flex items-center gap-1 px-2.5 py-1 rounded-full font-bold text-xs bg-emerald-50 text-emerald-700">
                                                    <span class="w-1.5 h-1.5 rounded-full bg-emerald-500"></span> {product.qty_on_hand}
                                                </span>
                                            {/if}
                                        {/if}
                                    </td>
                                    <td class="p-4">
                                        {#if product.is_ingredient}
                                            <div class="text-xs text-slate-700">
                                                <span class="font-bold text-indigo-700">{product.min_qty}</span>
                                                <span class="text-slate-400 text-[10px] block font-normal leading-tight mt-0.5">
                                                    Faktor: {product.min_stock_factor} · Buffer: {product.buffer_stock} · LT: {product.lead_time_days}h
                                                </span>
                                            </div>
                                        {:else if !product.track_stock}
                                            <span class="text-xs text-slate-400 italic">—</span>
                                        {:else}
                                            <span class="text-xs text-slate-700 font-bold">{product.min_qty}</span>
                                        {/if}
                                    </td>
                                    <td class="p-4 text-center">
                                        <div class="inline-flex gap-1.5 items-center justify-center">
                                            <!-- Group 1: Primary actions (clean outline style) -->
                                            <button
                                                class="w-16 h-8 flex items-center justify-center rounded-lg border border-slate-200 bg-white text-slate-600 hover:border-blue-500 hover:text-blue-600 text-[11px] font-bold tracking-wide transition shadow-sm whitespace-nowrap cursor-pointer"
                                                onclick={() => openViewModal(product)}
                                            >Detail</button>
                                            <button
                                                class="w-14 h-8 flex items-center justify-center rounded-lg border border-slate-200 bg-white text-slate-600 hover:border-amber-500 hover:text-amber-600 text-[11px] font-bold tracking-wide transition shadow-sm whitespace-nowrap cursor-pointer"
                                                onclick={() => openEditModal(product)}
                                            >Edit</button>

                                            <!-- Resep OR invisible spacer — exactly w-16 -->
                                            {#if !product.is_ingredient}
                                                <button
                                                    class="w-16 h-8 flex items-center justify-center rounded-lg border border-slate-200 bg-white text-slate-600 hover:border-indigo-500 hover:text-indigo-600 text-[11px] font-bold tracking-wide transition shadow-sm whitespace-nowrap cursor-pointer"
                                                    onclick={() => openRecipeModal(product)}
                                                >Resep</button>
                                            {:else}
                                                <span class="w-16 inline-block"></span>
                                            {/if}

                                            <!-- Divider -->
                                            <div class="w-px h-6 bg-slate-200 mx-1"></div>

                                            <!-- Group 2: Stock actions (Premium unified outline style) -->
                                            <button class="w-[52px] h-8 flex items-center justify-center rounded-lg border border-slate-200 bg-white text-emerald-600 hover:bg-emerald-50 hover:border-emerald-300 text-[11px] font-bold tracking-wide transition shadow-sm whitespace-nowrap cursor-pointer" onclick={() => openModal('in', product)}>+ In</button>
                                            <button class="w-[56px] h-8 flex items-center justify-center rounded-lg border border-slate-200 bg-white text-amber-600 hover:bg-amber-50 hover:border-amber-300 text-[11px] font-bold tracking-wide transition shadow-sm whitespace-nowrap cursor-pointer" onclick={() => openModal('adjust', product)}>Adjust</button>
                                            <button class="w-[60px] h-8 flex items-center justify-center rounded-lg border border-slate-200 bg-white text-purple-600 hover:bg-purple-50 hover:border-purple-300 text-[11px] font-bold tracking-wide transition shadow-sm whitespace-nowrap cursor-pointer" onclick={() => openModal('opname', product)}>Opname</button>
                                            <button class="w-[64px] h-8 flex items-center justify-center rounded-lg border border-slate-200 bg-white text-slate-600 hover:bg-slate-50 hover:border-slate-300 text-[11px] font-bold tracking-wide transition shadow-sm whitespace-nowrap cursor-pointer" onclick={() => openModal('transfer', product)}>Transfer</button>
                                        </div>
                                    </td>
                                </tr>
                            {/each}
                            {#if filteredProducts.length === 0}
                                <tr>
                                    <td colspan="6" class="p-10">
                                        <div class="flex flex-col items-center justify-center text-center gap-2">
                                            <span class="text-4xl opacity-60">{hasActiveFilters() ? '🔍' : '📦'}</span>
                                            <p class="text-slate-500 text-sm font-semibold">{hasActiveFilters() ? 'Tidak ada produk yang cocok dengan filter.' : 'Belum ada produk di inventaris.'}</p>
                                            {#if hasActiveFilters()}
                                                <button class="mt-2 text-xs font-bold text-blue-600 hover:text-blue-700 cursor-pointer" onclick={resetFilters}>↺ Reset Filter</button>
                                            {:else}
                                                <button class="mt-2 bg-blue-600 hover:bg-blue-700 text-white text-xs font-bold px-4 py-2 rounded-lg transition cursor-pointer" onclick={() => showAddProductModal = true}>+ Tambah Produk Pertama</button>
                                            {/if}
                                        </div>
                                    </td>
                                </tr>
                            {/if}
                        {/if}
                    </tbody>
                </table>
            </div>
        </div>
    {:else}
        <!-- Card/Grid View -->
        <div class="flex-1 overflow-y-auto pb-8">
            <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-5">
                {#if loading}
                    {#each Array(8) as _}
                        <div class="bg-white rounded-2xl border border-slate-200 overflow-hidden flex flex-col shadow-xs">
                            <div class="w-full h-40 bg-slate-100 animate-pulse"></div>
                            <div class="p-4 flex flex-col gap-2">
                                <div class="h-2.5 w-20 rounded bg-slate-100 animate-pulse"></div>
                                <div class="h-4 w-36 rounded bg-slate-100 animate-pulse"></div>
                                <div class="h-3 w-24 rounded bg-slate-100 animate-pulse"></div>
                                <div class="h-6 w-28 rounded bg-slate-100 animate-pulse mt-2"></div>
                            </div>
                        </div>
                    {/each}
                {:else}
                    {#each filteredProducts as product (product.id)}
                        <div class="bg-white rounded-2xl shadow-xs hover:shadow-md border border-slate-200 overflow-hidden flex flex-col hover:border-slate-300 transition-all duration-300">
                            <!-- Product Image -->
                            <div class="relative w-full h-40 bg-slate-100 flex items-center justify-center border-b border-slate-100">
                                {#if product.image_url}
                                    <img src={product.image_url} class="w-full h-full object-cover" alt={product.name} />
                                {:else}
                                    <div class="w-full h-full bg-gradient-to-br from-slate-100 to-slate-200 text-slate-300 font-black flex items-center justify-center text-5xl">
                                        {product.name.charAt(0).toUpperCase()}
                                    </div>
                                {/if}
                                <!-- Stock Badge -->
                                <div class="absolute top-3 right-3">
                                    {#if !product.track_stock}
                                        <span class="bg-slate-700/90 text-white px-2.5 py-1 rounded-full text-xs font-bold shadow-sm backdrop-blur-sm">Tak Lacak</span>
                                    {:else if product.qty_on_hand <= 0}
                                        <span class="bg-red-500 text-white px-2.5 py-1 rounded-full text-xs font-bold shadow-sm">Habis</span>
                                    {:else if product.qty_on_hand <= product.min_qty}
                                        <span class="bg-amber-500 text-white px-2.5 py-1 rounded-full text-xs font-bold shadow-sm">Kritis: {product.qty_on_hand}</span>
                                    {:else}
                                        <span class="bg-emerald-600 text-white px-2.5 py-1 rounded-full text-xs font-bold shadow-sm">Stok: {product.qty_on_hand}</span>
                                    {/if}
                                </div>
                                {#if product.is_ingredient}
                                    <div class="absolute top-3 left-3">
                                        <span class="bg-indigo-600/90 text-white px-2 py-0.5 rounded-full text-[10px] font-bold shadow-sm backdrop-blur-sm">Bahan Baku</span>
                                    </div>
                                {/if}
                            </div>

                            <!-- Product Info -->
                            <div class="p-4 flex-1 flex flex-col">
                                <span class="text-[10px] uppercase tracking-wider text-slate-400 font-bold">{product.category_name || 'Tanpa Kategori'}</span>
                                <h3 class="font-bold text-slate-800 text-base line-clamp-1 mt-1">{product.name}</h3>
                                <span class="text-xs text-slate-500 mt-0.5 font-mono">SKU: {product.sku}</span>
                                <span class="text-xs text-slate-500 mt-1">
                                    {#if product.is_ingredient}
                                        Min. Stok: <strong class="text-indigo-700 font-bold">{product.min_qty}</strong>
                                        <span class="text-slate-400 text-[10px] block font-normal leading-tight mt-0.5">
                                            Faktor: {product.min_stock_factor} · Buffer: {product.buffer_stock} · LT: {product.lead_time_days}h
                                        </span>
                                    {:else if !product.track_stock}
                                        <span class="text-slate-400 italic text-[11px]">Stok tidak dilacak</span>
                                    {:else}
                                        Min. Stok: <strong class="text-slate-700 font-bold">{product.min_qty}</strong>
                                    {/if}
                                </span>

                                <div class="mt-4 flex justify-between items-baseline">
                                    <div>
                                        <span class="text-[10px] text-slate-400 block font-semibold">Harga Jual</span>
                                        {#if product.is_ingredient}
                                            <span class="font-bold text-slate-400 text-base italic">—</span>
                                        {:else}
                                            <span class="font-extrabold text-blue-600 text-lg">Rp {product.price.toLocaleString('id-ID')}</span>
                                        {/if}
                                    </div>
                                    {#if product.cost}
                                        <div class="text-right">
                                            <span class="text-[10px] text-slate-400 block font-semibold">Modal</span>
                                            <span class="font-bold text-slate-500 text-sm">Rp {product.cost.toLocaleString('id-ID')}</span>
                                        </div>
                                    {/if}
                                </div>

                                <!-- Buttons Section -->
                                <div class="mt-5 pt-4 border-t border-slate-100 flex flex-col gap-2">
                                    <div class="flex gap-2">
                                        <button class="flex-1 bg-blue-600 hover:bg-blue-700 text-white py-1.5 rounded-lg text-xs font-bold transition flex justify-center items-center gap-1 shadow-xs" onclick={() => openViewModal(product)}>
                                            👁 Detail
                                        </button>
                                        <button class="flex-1 bg-amber-500 hover:bg-amber-600 text-white py-1.5 rounded-lg text-xs font-bold transition flex justify-center items-center gap-1 shadow-xs" onclick={() => openEditModal(product)}>
                                            ✏ Edit
                                        </button>
                                    </div>
                                    {#if !product.is_ingredient}
                                        <button class="w-full bg-indigo-600 hover:bg-indigo-700 text-white py-1.5 rounded-lg text-xs font-bold transition flex justify-center items-center gap-1 cursor-pointer shadow-xs" onclick={() => openRecipeModal(product)}>
                                            🍳 Kelola Resep
                                        </button>
                                    {/if}
                                    <div class="grid grid-cols-4 gap-1 mt-1 text-center">
                                        <button class="bg-emerald-50 hover:bg-emerald-100 text-emerald-700 py-1 rounded-lg text-[10px] font-bold transition cursor-pointer" onclick={() => openModal('in', product)}>+ In</button>
                                        <button class="bg-yellow-50 hover:bg-yellow-100 text-yellow-700 py-1 rounded-lg text-[10px] font-bold transition cursor-pointer" onclick={() => openModal('adjust', product)}>Adj</button>
                                        <button class="bg-purple-50 hover:bg-purple-100 text-purple-700 py-1 rounded-lg text-[10px] font-bold transition cursor-pointer" onclick={() => openModal('opname', product)}>Opn</button>
                                        <button class="bg-slate-100 hover:bg-slate-200 text-slate-700 py-1 rounded-lg text-[10px] font-bold transition cursor-pointer" onclick={() => openModal('transfer', product)}>Trf</button>
                                    </div>
                                </div>
                            </div>
                        </div>
                    {/each}
                    {#if filteredProducts.length === 0}
                        <div class="col-span-full bg-white p-12 rounded-2xl border border-dashed border-slate-300 flex flex-col items-center gap-2 text-center">
                            <span class="text-4xl opacity-60">{hasActiveFilters() ? '🔍' : '📦'}</span>
                            <p class="text-slate-500 text-sm font-semibold">{hasActiveFilters() ? 'Tidak ada produk yang cocok dengan filter.' : 'Belum ada produk di inventaris.'}</p>
                            {#if hasActiveFilters()}
                                <button class="mt-2 text-xs font-bold text-blue-600 hover:text-blue-700 cursor-pointer" onclick={resetFilters}>↺ Reset Filter</button>
                            {:else}
                                <button class="mt-2 bg-blue-600 hover:bg-blue-700 text-white text-xs font-bold px-4 py-2 rounded-lg transition cursor-pointer" onclick={() => showAddProductModal = true}>+ Tambah Produk Pertama</button>
                            {/if}
                        </div>
                    {/if}
                {/if}
            </div>
        </div>
    {/if}
</div>

<!-- ACTION MODAL -->
{#if showModal}
<div class="modal-backdrop fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4">
    <div class="bg-white rounded-2xl shadow-2xl w-full max-w-md overflow-hidden">
        <!-- Header -->
        <div class="p-5 border-b border-slate-100 flex items-start justify-between gap-3">
            <div class="flex items-center gap-3 min-w-0">
                <div class="w-11 h-11 rounded-xl flex items-center justify-center text-xl shrink-0"
                     style="background-color: rgba({modalMeta.accentRgb}, 0.1); color: rgb({modalMeta.accentRgb});">
                    {modalMeta.icon}
                </div>
                <div class="min-w-0">
                    <h2 class="text-lg font-extrabold text-slate-800 leading-tight">{modalMeta.title}</h2>
                    <p class="text-xs text-slate-500 mt-0.5">{modalMeta.sub}</p>
                </div>
            </div>
            <button class="text-slate-400 hover:text-slate-600 text-xl font-bold transition cursor-pointer shrink-0" onclick={() => showModal = false}>✕</button>
        </div>

        <!-- Body -->
        <div class="p-5 space-y-4">
            <!-- Product context card -->
            <div class="flex items-center gap-3 bg-slate-50 border border-slate-200 rounded-xl p-3">
                {#if selectedProduct?.image_url}
                    <img src={selectedProduct.image_url} class="w-11 h-11 object-cover rounded-lg border border-slate-200 shrink-0" alt={selectedProduct.name} />
                {:else}
                    <div class="w-11 h-11 bg-white text-blue-500 font-extrabold flex items-center justify-center rounded-lg border border-slate-200 shrink-0">
                        {selectedProduct?.name.charAt(0).toUpperCase()}
                    </div>
                {/if}
                <div class="min-w-0 flex-1">
                    <div class="font-bold text-slate-800 text-sm line-clamp-1">{selectedProduct?.name}</div>
                    <div class="text-[10px] text-slate-400 font-mono">{selectedProduct?.sku}</div>
                </div>
                <div class="text-right shrink-0 border-l border-slate-200 pl-3">
                    <div class="text-[10px] uppercase font-bold tracking-wider text-slate-400">Stok Sistem</div>
                    <div class="text-base font-black text-slate-800">{selectedProduct?.qty_on_hand}</div>
                </div>
            </div>

            <!-- Qty input -->
            <div>
                <label class="block text-xs font-bold text-slate-600 mb-1.5" for="actionQty">
                    {#if modalAction === 'in'}Jumlah Masuk (Stok +){/if}
                    {#if modalAction === 'adjust'}Selisih Stok (+ atau -){/if}
                    {#if modalAction === 'opname'}Stok Aktual (Fisik){/if}
                    {#if modalAction === 'transfer'}Jumlah Transfer{/if}
                </label>
                <input id="actionQty" type="number" bind:value={actionQty}
                       class="w-full p-2.5 border border-slate-300 rounded-xl text-sm font-bold focus:border-blue-400 focus:ring-2 focus:ring-blue-100 transition-all"
                       placeholder="0" />
                {#if modalAction === 'in' && selectedProduct}
                    <p class="text-[10px] text-slate-400 mt-1.5">Stok setelah aksi: <span class="font-bold text-slate-600">{selectedProduct.qty_on_hand + (actionQty || 0)}</span></p>
                {/if}
            </div>

            <!-- Reason input -->
            <div>
                <label class="block text-xs font-bold text-slate-600 mb-1.5" for="actionReason">
                    Alasan / Keterangan
                    <span class="font-normal text-slate-400">{modalAction !== 'in' ? '(Wajib)' : '(Opsional)'}</span>
                </label>
                <input id="actionReason" type="text" bind:value={actionReason}
                       class="w-full p-2.5 border border-slate-300 rounded-xl text-sm focus:border-blue-400 focus:ring-2 focus:ring-blue-100 transition-all"
                       placeholder="Contoh: Barang rusak, retur, pembelian supplier" />
            </div>
        </div>

        <!-- Footer -->
        <div class="p-5 pt-0 flex gap-2.5">
            <button class="flex-1 py-2.5 rounded-xl text-sm font-bold text-slate-700 bg-slate-100 hover:bg-slate-200 transition cursor-pointer" onclick={() => showModal = false}>Batal</button>
            <button class="flex-1 py-2.5 rounded-xl text-sm font-bold text-white transition cursor-pointer shadow-sm hover:brightness-110"
                    style="background-color: rgb({modalMeta.accentRgb});"
                    onclick={submitAction}>Simpan</button>
        </div>
    </div>
</div>
{/if}

<!-- ADD PRODUCT MODAL -->
{#if showAddProductModal}
<div class="modal-backdrop fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4">
    <div class="bg-white rounded-2xl shadow-2xl w-full max-w-lg overflow-hidden flex flex-col max-h-[92vh]">
        <!-- Header -->
        <div class="p-5 border-b border-slate-100 flex items-start justify-between gap-3 shrink-0">
            <div class="flex items-center gap-3">
                <div class="w-11 h-11 rounded-xl bg-blue-50 text-blue-600 flex items-center justify-center text-xl shrink-0">＋</div>
                <div>
                    <h2 class="text-lg font-extrabold text-slate-800 leading-tight">Tambah Produk Baru</h2>
                    <p class="text-xs text-slate-500 mt-0.5">Lengkapi informasi produk di bawah ini.</p>
                </div>
            </div>
            <button class="text-slate-400 hover:text-slate-600 text-xl font-bold transition cursor-pointer shrink-0" onclick={() => showAddProductModal = false}>✕</button>
        </div>

        <!-- Body (scrollable) -->
        <div class="p-5 overflow-y-auto space-y-4">
            <!-- Group: Identitas -->
            <div class="space-y-3">
                <div>
                    <label class="block text-xs font-bold text-slate-600 mb-1.5" for="newName">Nama Produk <span class="text-rose-500">*</span></label>
                    <input id="newName" type="text" bind:value={newName}
                           class="w-full p-2.5 border border-slate-300 rounded-xl text-sm focus:border-blue-400 focus:ring-2 focus:ring-blue-100 transition-all"
                           placeholder="Contoh: Kopi Caramel Latte" />
                </div>
                <div class="grid grid-cols-2 gap-3">
                    <div>
                        <label class="block text-xs font-bold text-slate-600 mb-1.5" for="newSku">SKU (Kode) <span class="text-rose-500">*</span></label>
                        <input id="newSku" type="text" bind:value={newSku}
                               class="w-full p-2.5 border border-slate-300 rounded-xl text-sm font-mono focus:border-blue-400 focus:ring-2 focus:ring-blue-100 transition-all"
                               placeholder="KCL-001" />
                    </div>
                    <div>
                        <label class="block text-xs font-bold text-slate-600 mb-1.5" for="newCategory">Kategori</label>
                        <select id="newCategory" bind:value={newCategoryId}
                                class="w-full p-2.5 border border-slate-300 rounded-xl text-sm bg-white focus:border-blue-400 focus:ring-2 focus:ring-blue-100 transition-all cursor-pointer">
                            <option value="">Tanpa Kategori</option>
                            {#each categoryOptions as cat}
                                <option value={cat.id}>{cat.name}</option>
                            {/each}
                        </select>
                    </div>
                </div>
            </div>

            <!-- Group: Gambar -->
            <div>
                <span class="block text-xs font-bold text-slate-600 mb-1.5">Gambar Produk</span>
                <div class="flex items-center gap-4 p-3 border border-slate-200 rounded-xl bg-slate-50">
                    {#if newImageUrl}
                        <div class="relative w-16 h-16 border border-slate-200 rounded-xl overflow-hidden bg-white shadow-xs shrink-0">
                            <img src={newImageUrl} class="w-full h-full object-cover" alt="Preview" />
                            <button type="button" class="absolute top-0 right-0 bg-rose-600 text-white rounded-bl-lg p-1 text-[10px] font-bold hover:bg-rose-700 transition cursor-pointer" onclick={() => newImageUrl = ''}>✕</button>
                        </div>
                    {:else}
                        <div class="w-16 h-16 bg-white text-slate-300 flex items-center justify-center rounded-xl border border-dashed border-slate-300 shrink-0 text-2xl">📷</div>
                    {/if}
                    <div class="min-w-0">
                        <button type="button" class="bg-white border border-slate-300 hover:bg-slate-50 text-slate-700 px-3 py-1.5 rounded-lg text-xs font-bold transition cursor-pointer" onclick={() => imageInput.click()}>Pilih File Gambar</button>
                        <input type="file" bind:this={imageInput} onchange={handleImageFileChange} accept="image/*" style="display: none;" />
                        <p class="text-slate-400 text-[10px] mt-1">Format: JPG, PNG, WEBP · Maks 1MB</p>
                    </div>
                </div>
            </div>

            <!-- Group: Harga -->
            <div>
                <span class="block text-xs font-bold text-slate-600 mb-1.5">Harga</span>
                <div class="grid grid-cols-2 gap-3">
                    <div>
                        <label class="block text-[10px] font-bold text-slate-400 mb-1" for="newPrice">Harga Jual (Rp) <span class="text-rose-500">*</span></label>
                        <input id="newPrice" type="number" bind:value={newPrice}
                               class="w-full p-2.5 border border-slate-300 rounded-xl text-sm focus:border-blue-400 focus:ring-2 focus:ring-blue-100 transition-all"
                               placeholder="28000" />
                    </div>
                    <div>
                        <label class="block text-[10px] font-bold text-slate-400 mb-1" for="newCost">Harga Modal (Rp)</label>
                        <input id="newCost" type="number" bind:value={newCost}
                               class="w-full p-2.5 border border-slate-300 rounded-xl text-sm focus:border-blue-400 focus:ring-2 focus:ring-blue-100 transition-all"
                               placeholder="10000" />
                    </div>
                </div>
            </div>

            <!-- Group: Tipe & Pelacakan -->
            <div class="bg-slate-50 border border-slate-200 rounded-xl p-3 space-y-2">
                <label class="flex items-start gap-2.5 text-sm text-slate-700 cursor-pointer select-none" for="newIsIngredient">
                    <input id="newIsIngredient" type="checkbox" bind:checked={newIsIngredient} class="w-4 h-4 mt-0.5 rounded border-slate-300 text-blue-600 focus:ring-blue-500 cursor-pointer" />
                    <span><span class="font-bold">Bahan Baku / Ingredient</span><span class="block text-[11px] text-slate-500 font-normal mt-0.5">Tidak dijual langsung, dipakai dalam resep menu.</span></span>
                </label>
                <label class="flex items-start gap-2.5 text-sm text-slate-700 cursor-pointer select-none" for="newTrackStock">
                    <input id="newTrackStock" type="checkbox" bind:checked={newTrackStock} class="w-4 h-4 mt-0.5 rounded border-slate-300 text-blue-600 focus:ring-blue-500 cursor-pointer" />
                    <span><span class="font-bold">Lacak & Kelola Stok</span><span class="block text-[11px] text-slate-500 font-normal mt-0.5">Aktifkan pencatatan masuk/keluar stok inventaris.</span></span>
                </label>
            </div>

            <!-- Group: Stok (conditional) -->
            {#if newTrackStock}
            <div class="space-y-3">
                <div class="flex items-center gap-1.5">
                    <span class="h-px flex-grow bg-slate-200"></span>
                    <span class="text-[10px] uppercase font-bold tracking-wider text-slate-400">Pengaturan Stok</span>
                    <span class="h-px flex-grow bg-slate-200"></span>
                </div>
                <div>
                    <label class="block text-xs font-bold text-slate-600 mb-1.5" for="newInitialQty">Jumlah Stok Awal</label>
                    <input id="newInitialQty" type="number" bind:value={newInitialQty}
                           class="w-full p-2.5 border border-slate-300 rounded-xl text-sm focus:border-blue-400 focus:ring-2 focus:ring-blue-100 transition-all"
                           placeholder="0" />
                </div>
                {#if newIsIngredient}
                    <div class="grid grid-cols-2 gap-3">
                        <div>
                            <label class="flex items-center gap-1 text-xs font-bold text-slate-600 mb-1.5 cursor-help" for="newMinStockFactor" title="Untuk bahan baku, stok minimal adalah total kebutuhan resep dikali faktor pengali ini. Contoh: Jika total resep butuh 6 butir telur, dan faktor diisi 5, maka minimal stok = 6 x 5 = 30 butir.">
                                Faktor Min. Stok <span class="text-slate-300">ⓘ</span>
                            </label>
                            <input id="newMinStockFactor" type="number" bind:value={newMinStockFactor}
                                   class="w-full p-2.5 border border-slate-300 rounded-xl text-sm focus:border-blue-400 focus:ring-2 focus:ring-blue-100 transition-all"
                                   placeholder="5" />
                        </div>
                        <div>
                            <label class="flex items-center gap-1 text-xs font-bold text-slate-600 mb-1.5 cursor-help" for="newBufferStock" title="Stok cadangan pengaman statis. Minimal bahan baku tidak akan pernah kurang dari angka ini.">
                                Buffer Stok <span class="text-slate-300">ⓘ</span>
                            </label>
                            <input id="newBufferStock" type="number" bind:value={newBufferStock}
                                   class="w-full p-2.5 border border-slate-300 rounded-xl text-sm focus:border-blue-400 focus:ring-2 focus:ring-blue-100 transition-all"
                                   placeholder="10" />
                        </div>
                    </div>
                    <div>
                        <label class="flex items-center gap-1 text-xs font-bold text-slate-600 mb-1.5 cursor-help" for="newLeadTimeDays" title="Waktu tunggu pengiriman dari supplier (dalam hari).">
                            Durasi Kirim Supplier (Hari) <span class="text-slate-300">ⓘ</span>
                        </label>
                        <input id="newLeadTimeDays" type="number" bind:value={newLeadTimeDays}
                               class="w-full p-2.5 border border-slate-300 rounded-xl text-sm focus:border-blue-400 focus:ring-2 focus:ring-blue-100 transition-all"
                               placeholder="2" />
                    </div>
                {:else}
                    <div>
                        <label class="block text-xs font-bold text-slate-600 mb-1.5" for="newBufferStock">Stok Minimal</label>
                        <input id="newBufferStock" type="number" bind:value={newBufferStock}
                               class="w-full p-2.5 border border-slate-300 rounded-xl text-sm focus:border-blue-400 focus:ring-2 focus:ring-blue-100 transition-all"
                               placeholder="10" />
                    </div>
                {/if}
            </div>
            {/if}
        </div>

        <!-- Footer -->
        <div class="p-5 pt-4 border-t border-slate-100 flex gap-2.5 shrink-0">
            <button class="flex-1 py-2.5 rounded-xl text-sm font-bold text-slate-700 bg-slate-100 hover:bg-slate-200 transition cursor-pointer" onclick={() => showAddProductModal = false}>Batal</button>
            <button class="flex-[1.5] py-2.5 rounded-xl text-sm font-bold text-white bg-blue-600 hover:bg-blue-700 transition cursor-pointer shadow-sm shadow-blue-500/20" onclick={addProduct}>Simpan Produk</button>
        </div>
    </div>
</div>
{/if}

<!-- EDIT PRODUCT MODAL -->
{#if showEditModal}
<div class="modal-backdrop fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4">
    <div class="bg-white rounded-2xl shadow-2xl w-full max-w-lg overflow-hidden flex flex-col max-h-[92vh]">
        <!-- Header -->
        <div class="p-5 border-b border-slate-100 flex items-start justify-between gap-3 shrink-0">
            <div class="flex items-center gap-3">
                <div class="w-11 h-11 rounded-xl bg-amber-50 text-amber-600 flex items-center justify-center text-xl shrink-0">✏️</div>
                <div>
                    <h2 class="text-lg font-extrabold text-slate-800 leading-tight">Ubah Produk</h2>
                    <p class="text-xs text-slate-500 mt-0.5 line-clamp-1">{selectedEditProduct?.name}</p>
                </div>
            </div>
            <button class="text-slate-400 hover:text-slate-600 text-xl font-bold transition cursor-pointer shrink-0" onclick={() => showEditModal = false}>✕</button>
        </div>

        <!-- Body (scrollable) -->
        <div class="p-5 overflow-y-auto space-y-4">
            <!-- Group: Identitas -->
            <div class="space-y-3">
                <div>
                    <label class="block text-xs font-bold text-slate-600 mb-1.5" for="editName">Nama Produk <span class="text-rose-500">*</span></label>
                    <input id="editName" type="text" bind:value={editName}
                           class="w-full p-2.5 border border-slate-300 rounded-xl text-sm focus:border-amber-400 focus:ring-2 focus:ring-amber-100 transition-all"
                           placeholder="Contoh: Kopi Caramel Latte" />
                </div>
                <div class="grid grid-cols-2 gap-3">
                    <div>
                        <label class="block text-xs font-bold text-slate-600 mb-1.5" for="editSku">SKU (Kode) <span class="text-rose-500">*</span></label>
                        <input id="editSku" type="text" bind:value={editSku}
                               class="w-full p-2.5 border border-slate-300 rounded-xl text-sm font-mono focus:border-amber-400 focus:ring-2 focus:ring-amber-100 transition-all"
                               placeholder="KCL-001" />
                    </div>
                    <div>
                        <label class="block text-xs font-bold text-slate-600 mb-1.5" for="editCategory">Kategori</label>
                        <select id="editCategory" bind:value={editCategoryId}
                                class="w-full p-2.5 border border-slate-300 rounded-xl text-sm bg-white focus:border-amber-400 focus:ring-2 focus:ring-amber-100 transition-all cursor-pointer">
                            <option value="">Tanpa Kategori</option>
                            {#each categoryOptions as cat}
                                <option value={cat.id}>{cat.name}</option>
                            {/each}
                        </select>
                    </div>
                </div>
            </div>

            <!-- Group: Gambar -->
            <div>
                <span class="block text-xs font-bold text-slate-600 mb-1.5">Gambar Produk</span>
                <div class="flex items-center gap-4 p-3 border border-slate-200 rounded-xl bg-slate-50">
                    {#if editImageUrl}
                        <div class="relative w-16 h-16 border border-slate-200 rounded-xl overflow-hidden bg-white shadow-xs shrink-0">
                            <img src={editImageUrl} class="w-full h-full object-cover" alt="Preview" />
                            <button type="button" class="absolute top-0 right-0 bg-rose-600 text-white rounded-bl-lg p-1 text-[10px] font-bold hover:bg-rose-700 transition cursor-pointer" onclick={() => editImageUrl = ''}>✕</button>
                        </div>
                    {:else}
                        <div class="w-16 h-16 bg-white text-slate-300 flex items-center justify-center rounded-xl border border-dashed border-slate-300 shrink-0 text-2xl">📷</div>
                    {/if}
                    <div class="min-w-0">
                        <button type="button" class="bg-white border border-slate-300 hover:bg-slate-50 text-slate-700 px-3 py-1.5 rounded-lg text-xs font-bold transition cursor-pointer" onclick={() => editImageInput.click()}>Pilih File Gambar</button>
                        <input type="file" bind:this={editImageInput} onchange={handleEditImageFileChange} accept="image/*" style="display: none;" />
                        <p class="text-slate-400 text-[10px] mt-1">Format: JPG, PNG, WEBP · Maks 1MB</p>
                    </div>
                </div>
            </div>

            <!-- Group: Harga -->
            <div>
                <span class="block text-xs font-bold text-slate-600 mb-1.5">Harga</span>
                <div class="grid grid-cols-2 gap-3">
                    <div>
                        <label class="block text-[10px] font-bold text-slate-400 mb-1" for="editPrice">Harga Jual (Rp) <span class="text-rose-500">*</span></label>
                        <input id="editPrice" type="number" bind:value={editPrice}
                               class="w-full p-2.5 border border-slate-300 rounded-xl text-sm focus:border-amber-400 focus:ring-2 focus:ring-amber-100 transition-all" />
                    </div>
                    <div>
                        <label class="block text-[10px] font-bold text-slate-400 mb-1" for="editCost">Harga Modal (Rp)</label>
                        <input id="editCost" type="number" bind:value={editCost}
                               class="w-full p-2.5 border border-slate-300 rounded-xl text-sm focus:border-amber-400 focus:ring-2 focus:ring-amber-100 transition-all" />
                    </div>
                </div>
            </div>

            <!-- Group: Tipe & Pelacakan -->
            <div class="bg-slate-50 border border-slate-200 rounded-xl p-3 space-y-2">
                <label class="flex items-start gap-2.5 text-sm text-slate-700 cursor-pointer select-none" for="editIsIngredient">
                    <input id="editIsIngredient" type="checkbox" bind:checked={editIsIngredient} class="w-4 h-4 mt-0.5 rounded border-slate-300 text-amber-500 focus:ring-amber-400 cursor-pointer" />
                    <span><span class="font-bold">Bahan Baku / Ingredient</span><span class="block text-[11px] text-slate-500 font-normal mt-0.5">Tidak dijual langsung, dipakai dalam resep menu.</span></span>
                </label>
                <label class="flex items-start gap-2.5 text-sm text-slate-700 cursor-pointer select-none" for="editTrackStock">
                    <input id="editTrackStock" type="checkbox" bind:checked={editTrackStock} class="w-4 h-4 mt-0.5 rounded border-slate-300 text-amber-500 focus:ring-amber-400 cursor-pointer" />
                    <span><span class="font-bold">Lacak & Kelola Stok</span><span class="block text-[11px] text-slate-500 font-normal mt-0.5">Aktifkan pencatatan masuk/keluar stok inventaris.</span></span>
                </label>
            </div>

            <!-- Group: Stok (conditional) -->
            {#if editTrackStock}
            <div class="space-y-3">
                <div class="flex items-center gap-1.5">
                    <span class="h-px flex-grow bg-slate-200"></span>
                    <span class="text-[10px] uppercase font-bold tracking-wider text-slate-400">Pengaturan Stok</span>
                    <span class="h-px flex-grow bg-slate-200"></span>
                </div>
                {#if editIsIngredient}
                    <div class="grid grid-cols-2 gap-3">
                        <div>
                            <label class="flex items-center gap-1 text-xs font-bold text-slate-600 mb-1.5 cursor-help" for="editMinStockFactor" title="Untuk bahan baku, stok minimal adalah total kebutuhan resep dikali faktor pengali ini. Contoh: Jika total resep butuh 6 butir telur, dan faktor diisi 5, maka minimal stok = 6 x 5 = 30 butir.">
                                Faktor Min. Stok <span class="text-slate-300">ⓘ</span>
                            </label>
                            <input id="editMinStockFactor" type="number" bind:value={editMinStockFactor}
                                   class="w-full p-2.5 border border-slate-300 rounded-xl text-sm focus:border-amber-400 focus:ring-2 focus:ring-amber-100 transition-all"
                                   placeholder="5" />
                        </div>
                        <div>
                            <label class="flex items-center gap-1 text-xs font-bold text-slate-600 mb-1.5 cursor-help" for="editBufferStock" title="Stok cadangan pengaman statis. Minimal bahan baku tidak akan pernah kurang dari angka ini.">
                                Buffer Stok <span class="text-slate-300">ⓘ</span>
                            </label>
                            <input id="editBufferStock" type="number" bind:value={editBufferStock}
                                   class="w-full p-2.5 border border-slate-300 rounded-xl text-sm focus:border-amber-400 focus:ring-2 focus:ring-amber-100 transition-all"
                                   placeholder="10" />
                        </div>
                    </div>
                    <div>
                        <label class="flex items-center gap-1 text-xs font-bold text-slate-600 mb-1.5 cursor-help" for="editLeadTimeDays" title="Waktu tunggu pengiriman dari supplier (dalam hari).">
                            Durasi Kirim Supplier (Hari) <span class="text-slate-300">ⓘ</span>
                        </label>
                        <input id="editLeadTimeDays" type="number" bind:value={editLeadTimeDays}
                               class="w-full p-2.5 border border-slate-300 rounded-xl text-sm focus:border-amber-400 focus:ring-2 focus:ring-amber-100 transition-all"
                               placeholder="2" />
                    </div>
                {:else}
                    <div>
                        <label class="block text-xs font-bold text-slate-600 mb-1.5" for="editBufferStock">Stok Minimal</label>
                        <input id="editBufferStock" type="number" bind:value={editBufferStock}
                               class="w-full p-2.5 border border-slate-300 rounded-xl text-sm focus:border-amber-400 focus:ring-2 focus:ring-amber-100 transition-all"
                               placeholder="10" />
                    </div>
                {/if}
            </div>
            {/if}
        </div>

        <!-- Footer -->
        <div class="p-5 pt-4 border-t border-slate-100 flex gap-2.5 shrink-0">
            <button class="flex-1 py-2.5 rounded-xl text-sm font-bold text-slate-700 bg-slate-100 hover:bg-slate-200 transition cursor-pointer" onclick={() => showEditModal = false}>Batal</button>
            <button class="flex-[1.5] py-2.5 rounded-xl text-sm font-bold text-white bg-amber-500 hover:bg-amber-600 transition cursor-pointer shadow-sm shadow-amber-500/20" onclick={submitEdit}>Simpan Perubahan</button>
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
            <button class="text-slate-400 hover:text-slate-600 text-xl font-bold transition-colors cursor-pointer" onclick={() => showBarcodeModal = false}>✕</button>
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
