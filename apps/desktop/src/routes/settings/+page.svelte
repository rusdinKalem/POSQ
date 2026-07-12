<script lang="ts">
    import { onMount } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';

    // State runes
    let activeTab = $state("shop"); // "shop", "categories", "tax_discount", "system"
    
    // Store profile states
    let storeName = $state("Toko POSQ");
    let storeAddress = $state("Jl. Merdeka No. 123, Jakarta");
    let storePhone = $state("0812-3456-7890");

    // Tax & Discount states
    let taxRate = $state(11);
    let serviceCharge = $state(0);
    let quickDiscountsInput = $state("5000,10000,5%,10%");

    // Categories states
    interface Category {
        id: string;
        name: string;
        parent_id?: string | null;
        parent_name?: string | null;
        business_mode?: string | null;
    }
    let categories = $state<Category[]>([]);
    let newCategoryName = $state("");
    let newCategoryParentId = $state<string | null>(null);
    let newCategoryBusinessMode = $state<string | null>(null);
    let isSavingCategory = $state(false);

    // Edit Category state
    let editingCategory = $state<Category | null>(null);
    let editCategoryName = $state("");
    let editCategoryParentId = $state<string | null>(null);
    let editCategoryBusinessMode = $state<string | null>(null);
    let isUpdatingCategory = $state(false);

    // Feedback states
    let toastMessage = $state("");
    let toastType = $state("success"); // "success" or "error"
    let isSavingSettings = $state(false);

    // Load configurations from localStorage
    function loadSettings() {
        if (typeof window !== 'undefined') {
            storeName = localStorage.getItem("posq_store_name") || "Toko POSQ";
            storeAddress = localStorage.getItem("posq_store_address") || "Jl. Merdeka No. 123, Jakarta";
            storePhone = localStorage.getItem("posq_store_phone") || "0812-3456-7890";
            
            const savedTax = localStorage.getItem("posq_tax_rate");
            taxRate = savedTax !== null ? parseFloat(savedTax) : 11;

            const savedService = localStorage.getItem("posq_service_charge");
            serviceCharge = savedService !== null ? parseFloat(savedService) : 0;

            quickDiscountsInput = localStorage.getItem("posq_quick_discounts") || "5000,10000,5%,10%";
        }
    }

    // Save configurations to localStorage
    function saveGeneralSettings() {
        isSavingSettings = true;
        try {
            localStorage.setItem("posq_store_name", storeName);
            localStorage.setItem("posq_store_address", storeAddress);
            localStorage.setItem("posq_store_phone", storePhone);
            localStorage.setItem("posq_tax_rate", taxRate.toString());
            localStorage.setItem("posq_service_charge", serviceCharge.toString());
            localStorage.setItem("posq_quick_discounts", quickDiscountsInput);
            
            showToast("Pengaturan berhasil disimpan!", "success");
        } catch (err: any) {
            showToast("Gagal menyimpan pengaturan: " + err.message, "error");
        } finally {
            isSavingSettings = false;
        }
    }

    // Category CRUD
    async function loadCategories() {
        try {
            categories = await invoke("list_categories");
        } catch (err: any) {
            console.error("Gagal memuat kategori:", err);
            showToast("Gagal memuat kategori: " + err, "error");
        }
    }

    async function addCategory() {
        const name = newCategoryName.trim();
        if (!name) return;
        isSavingCategory = true;
        try {
            await invoke("create_category", { 
                name, 
                parentId: newCategoryParentId || null, 
                businessMode: newCategoryBusinessMode || null 
            });
            newCategoryName = "";
            newCategoryParentId = null;
            newCategoryBusinessMode = null;
            showToast("Kategori baru berhasil ditambahkan!", "success");
            await loadCategories();
        } catch (err: any) {
            showToast(err.toString(), "error");
        } finally {
            isSavingCategory = false;
        }
    }

    function startEditCategory(cat: Category) {
        editingCategory = cat;
        editCategoryName = cat.name;
        editCategoryParentId = cat.parent_id || null;
        editCategoryBusinessMode = cat.business_mode || null;
    }

    function cancelEditCategory() {
        editingCategory = null;
    }

    async function saveCategoryEdit() {
        if (!editingCategory) return;
        const name = editCategoryName.trim();
        if (!name) return;
        isUpdatingCategory = true;
        try {
            await invoke("update_category", {
                id: editingCategory.id,
                name,
                parentId: editCategoryParentId || null,
                businessMode: editCategoryBusinessMode || null
            });
            editingCategory = null;
            showToast("Kategori berhasil diperbarui!", "success");
            await loadCategories();
        } catch (err: any) {
            showToast(err.toString(), "error");
        } finally {
            isUpdatingCategory = false;
        }
    }

    async function removeCategory(id: string) {
        if (!confirm("Apakah Anda yakin ingin menghapus kategori ini?")) return;
        try {
            await invoke("delete_category", { id });
            showToast("Kategori berhasil dihapus!", "success");
            await loadCategories();
        } catch (err: any) {
            showToast(err.toString(), "error");
        }
    }

    // Helper functions
    function showToast(message: string, type = "success") {
        toastMessage = message;
        toastType = type;
        setTimeout(() => {
            toastMessage = "";
        }, 3000);
    }

    onMount(() => {
        loadSettings();
        loadCategories();
        
        if (typeof window !== 'undefined') {
            const params = new URLSearchParams(window.location.search);
            const tab = params.get('tab');
            if (tab && ["shop", "categories", "tax_discount", "system"].includes(tab)) {
                activeTab = tab;
            }
        }
    });
</script>

<div class="min-h-screen bg-slate-50/50 py-8 px-4 sm:px-6 lg:px-8 font-sans">
    <div class="max-w-7xl mx-auto">
        <!-- Toast Notification -->
        {#if toastMessage}
            <div class="fixed top-6 right-6 z-50 animate-in fade-in slide-in-from-top duration-300 max-w-sm">
                <div class="p-4 rounded-xl shadow-lg border backdrop-blur-md flex items-center gap-3 {toastType === 'success' ? 'bg-emerald-50/90 text-emerald-800 border-emerald-200' : 'bg-rose-50/90 text-rose-800 border-rose-200'}">
                    <span>{toastType === 'success' ? '✅' : '⚠️'}</span>
                    <span class="text-sm font-semibold">{toastMessage}</span>
                </div>
            </div>
        {/if}

        <!-- Header -->
        <div class="mb-8 flex flex-col gap-2">
            <a href="/" class="inline-flex items-center text-sm font-semibold text-slate-500 hover:text-blue-600 transition-colors w-fit">
                <span class="mr-1">←</span> Kembali ke Dashboard
            </a>
            <h1 class="text-3xl font-extrabold text-slate-900 tracking-tight">Pengaturan Sistem</h1>
            <p class="text-slate-500 text-sm">Konfigurasi outlet, kategori produk, diskon, pajak, dan perangkat keras.</p>
        </div>

        <div class="flex flex-col lg:flex-row gap-8">
            <!-- Sidebar Navigation Tabs -->
            <div class="w-full lg:w-64 shrink-0 flex flex-row lg:flex-col gap-2 overflow-x-auto pb-3 lg:pb-0 scrollbar-none">
                <button 
                    onclick={() => activeTab = "shop"}
                    class="flex items-center gap-3 px-4 py-3 rounded-xl text-sm font-bold transition-all text-left whitespace-nowrap lg:w-full {activeTab === 'shop' ? 'bg-blue-600 text-white shadow-md shadow-blue-500/10' : 'bg-white text-slate-600 hover:bg-slate-100/80 border border-slate-200/60'}"
                >
                    <span class="text-base">📋</span>
                    <span>Profil Toko</span>
                </button>

                <button 
                    onclick={() => activeTab = "categories"}
                    class="flex items-center gap-3 px-4 py-3 rounded-xl text-sm font-bold transition-all text-left whitespace-nowrap lg:w-full {activeTab === 'categories' ? 'bg-blue-600 text-white shadow-md shadow-blue-500/10' : 'bg-white text-slate-600 hover:bg-slate-100/80 border border-slate-200/60'}"
                >
                    <span class="text-base">🏷️</span>
                    <span>Kategori Produk</span>
                </button>

                <button 
                    onclick={() => activeTab = "tax_discount"}
                    class="flex items-center gap-3 px-4 py-3 rounded-xl text-sm font-bold transition-all text-left whitespace-nowrap lg:w-full {activeTab === 'tax_discount' ? 'bg-blue-600 text-white shadow-md shadow-blue-500/10' : 'bg-white text-slate-600 hover:bg-slate-100/80 border border-slate-200/60'}"
                >
                    <span class="text-base">💰</span>
                    <span>Diskon & Pajak</span>
                </button>

                <button 
                    onclick={() => activeTab = "system"}
                    class="flex items-center gap-3 px-4 py-3 rounded-xl text-sm font-bold transition-all text-left whitespace-nowrap lg:w-full {activeTab === 'system' ? 'bg-blue-600 text-white shadow-md shadow-blue-500/10' : 'bg-white text-slate-600 hover:bg-slate-100/80 border border-slate-200/60'}"
                >
                    <span class="text-base">⚙️</span>
                    <span>Perangkat & Sistem</span>
                </button>
            </div>

            <!-- Content Area -->
            <div class="flex-1 bg-white border border-slate-200/80 rounded-2xl shadow-xs p-6 sm:p-8">
                <!-- TAB 1: Shop Profile -->
                {#if activeTab === "shop"}
                    <div class="space-y-6">
                        <div>
                            <h2 class="text-xl font-bold text-slate-800">Profil Bisnis / Toko</h2>
                            <p class="text-xs text-slate-400 mt-1">Informasi ini akan tercantum pada header struk kasir.</p>
                        </div>
                        <div class="border-t border-slate-100 pt-4 space-y-4">
                            <div>
                                <label for="store-name" class="block text-xs font-bold text-slate-500 uppercase tracking-wider mb-2">Nama Toko / Outlet</label>
                                <input 
                                    id="store-name"
                                    type="text" 
                                    bind:value={storeName} 
                                    class="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-3 text-sm focus:outline-none focus:border-blue-500 focus:bg-white transition-all font-medium text-slate-800"
                                />
                            </div>

                            <div>
                                <label for="store-address" class="block text-xs font-bold text-slate-500 uppercase tracking-wider mb-2">Alamat Toko</label>
                                <textarea 
                                    id="store-address"
                                    bind:value={storeAddress} 
                                    rows="3"
                                    class="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-3 text-sm focus:outline-none focus:border-blue-500 focus:bg-white transition-all font-medium text-slate-800 resize-none"
                                ></textarea>
                            </div>

                            <div>
                                <label for="store-phone" class="block text-xs font-bold text-slate-500 uppercase tracking-wider mb-2">Telepon / Kontak</label>
                                <input 
                                    id="store-phone"
                                    type="text" 
                                    bind:value={storePhone} 
                                    class="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-3 text-sm focus:outline-none focus:border-blue-500 focus:bg-white transition-all font-medium text-slate-800"
                                />
                            </div>
                        </div>

                        <div class="border-t border-slate-100 pt-6 flex justify-end">
                            <button 
                                onclick={saveGeneralSettings}
                                disabled={isSavingSettings}
                                class="bg-blue-600 hover:bg-blue-700 text-white font-bold text-sm py-3 px-6 rounded-xl shadow-md hover:shadow-lg active:scale-98 transition-all cursor-pointer"
                            >
                                {isSavingSettings ? 'Menyimpan...' : 'Simpan Perubahan'}
                            </button>
                        </div>
                    </div>
                {/if}

                <!-- TAB 2: Categories -->
                {#if activeTab === "categories"}
                    <div class="space-y-6">
                        <div>
                            <h2 class="text-xl font-bold text-slate-800">Manajemen Kategori Produk</h2>
                            <p class="text-xs text-slate-400 mt-1">Tambahkan atau hapus kategori produk yang tersedia di kasir.</p>
                        </div>

                        <div class="border-t border-slate-100 pt-4">
                            <!-- New Category Input Form -->
                            <div class="bg-slate-50 p-5 rounded-2xl border border-slate-200/50 mb-6">
                                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 items-end">
                                    <!-- Nama Kategori -->
                                    <div class="flex flex-col">
                                        <label for="new-category-name" class="block text-[10px] font-bold text-slate-500 uppercase tracking-wider mb-2">Nama Kategori / Sub-Kategori</label>
                                        <input 
                                            id="new-category-name"
                                            type="text" 
                                            bind:value={newCategoryName} 
                                            placeholder="Tulis nama kategori baru..." 
                                            class="custom-category-input h-11 w-full bg-white border border-slate-200 rounded-xl px-4 py-2 text-sm focus:outline-none focus:border-blue-500 focus:ring-4 focus:ring-blue-500/5 transition-all font-medium text-slate-800"
                                        />
                                    </div>
                                    
                                    <!-- Kategori Induk -->
                                    <div class="flex flex-col">
                                        <label for="new-category-parent" class="block text-[10px] font-bold text-slate-500 uppercase tracking-wider mb-2">Kategori Induk (Sub-Kategori)</label>
                                        <select 
                                            id="new-category-parent"
                                            bind:value={newCategoryParentId} 
                                            class="custom-category-input h-11 w-full bg-white border border-slate-200 rounded-xl px-4 py-2 text-sm focus:outline-none focus:border-blue-500 focus:ring-4 focus:ring-blue-500/5 transition-all font-medium text-slate-800"
                                        >
                                            <option value={null}>-- Tanpa Induk (Kategori Utama) --</option>
                                            {#each categories.filter(c => !c.parent_id) as mainCat}
                                                <option value={mainCat.id}>{mainCat.name}</option>
                                            {/each}
                                        </select>
                                    </div>

                                    <!-- Mode Bisnis -->
                                    <div class="flex flex-col">
                                        <label for="new-category-business-mode" class="block text-[10px] font-bold text-slate-500 uppercase tracking-wider mb-2">Mode Bisnis</label>
                                        <select 
                                            id="new-category-business-mode"
                                            bind:value={newCategoryBusinessMode} 
                                            class="custom-category-input h-11 w-full bg-white border border-slate-200 rounded-xl px-4 py-2 text-sm focus:outline-none focus:border-blue-500 focus:ring-4 focus:ring-blue-500/5 transition-all font-medium text-slate-800"
                                        >
                                            <option value={null}>Semua Mode Bisnis</option>
                                            <option value="fb">Food & Beverage (F&B)</option>
                                            <option value="retail">Retail (Toko/Eceran)</option>
                                            <option value="jasa">Jasa (Layanan/Teknisi)</option>
                                        </select>
                                    </div>

                                    <!-- Tombol Tambah -->
                                    <div class="flex flex-col">
                                        <button 
                                            onclick={addCategory}
                                            disabled={isSavingCategory}
                                            class="h-11 w-full bg-blue-600 hover:bg-blue-700 disabled:bg-blue-400 text-white font-bold text-sm rounded-xl shadow-md hover:shadow-lg active:scale-98 transition-all cursor-pointer whitespace-nowrap flex items-center justify-center gap-2"
                                        >
                                            <span>{isSavingCategory ? 'Menyimpan...' : 'Tambah Kategori'}</span>
                                        </button>
                                    </div>
                                </div>
                            </div>

                            <!-- Edit Category Modal Backdrop -->
                            {#if editingCategory}
                                <div class="fixed inset-0 bg-slate-900/40 backdrop-blur-sm z-50 flex items-center justify-center p-4">
                                    <div class="bg-white rounded-2xl shadow-2xl border border-slate-100 max-w-md w-full p-6 space-y-4">
                                        <div class="flex justify-between items-center pb-2 border-b border-slate-100">
                                            <h3 class="text-lg font-bold text-slate-800">Edit Kategori</h3>
                                            <button onclick={cancelEditCategory} class="text-slate-400 hover:text-slate-600 font-bold text-xl">×</button>
                                        </div>
                                        
                                        <div class="space-y-4">
                                            <div>
                                                <label for="edit-category-name" class="block text-xs font-bold text-slate-500 uppercase tracking-wider mb-2">Nama Kategori</label>
                                                <input 
                                                    id="edit-category-name"
                                                    type="text" 
                                                    bind:value={editCategoryName} 
                                                    class="custom-category-input w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-2.5 text-sm focus:outline-none focus:border-blue-500 transition-all font-medium text-slate-800"
                                                />
                                            </div>

                                            <div>
                                                <label for="edit-category-parent" class="block text-xs font-bold text-slate-500 uppercase tracking-wider mb-2">Kategori Induk</label>
                                                <select 
                                                    id="edit-category-parent"
                                                    bind:value={editCategoryParentId} 
                                                    class="custom-category-input w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-2.5 text-sm focus:outline-none focus:border-blue-500 transition-all font-medium text-slate-800"
                                                >
                                                    <option value={null}>-- Tanpa Induk (Kategori Utama) --</option>
                                                    {#each categories.filter(c => c.id !== editingCategory?.id && !c.parent_id) as mainCat}
                                                        <option value={mainCat.id}>{mainCat.name}</option>
                                                    {/each}
                                                </select>
                                            </div>

                                            <div>
                                                <label for="edit-category-business-mode" class="block text-xs font-bold text-slate-500 uppercase tracking-wider mb-2">Mode Bisnis</label>
                                                <select 
                                                    id="edit-category-business-mode"
                                                    bind:value={editCategoryBusinessMode} 
                                                    class="custom-category-input w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-2.5 text-sm focus:outline-none focus:border-blue-500 transition-all font-medium text-slate-800"
                                                >
                                                    <option value={null}>Semua Mode Bisnis</option>
                                                    <option value="fb">Food & Beverage (F&B)</option>
                                                    <option value="retail">Retail (Toko/Eceran)</option>
                                                    <option value="jasa">Jasa (Layanan/Teknisi)</option>
                                                </select>
                                            </div>
                                        </div>

                                        <div class="flex justify-end gap-3 pt-4 border-t border-slate-100">
                                            <button 
                                                onclick={cancelEditCategory}
                                                class="bg-slate-100 hover:bg-slate-200 text-slate-700 font-bold text-sm py-2 px-4 rounded-xl transition-all cursor-pointer"
                                            >
                                                Batal
                                            </button>
                                            <button 
                                                onclick={saveCategoryEdit}
                                                disabled={isUpdatingCategory}
                                                class="bg-blue-600 hover:bg-blue-700 text-white font-bold text-sm py-2 px-4 rounded-xl transition-all cursor-pointer"
                                            >
                                                {isUpdatingCategory ? 'Menyimpan...' : 'Simpan'}
                                            </button>
                                        </div>
                                    </div>
                                </div>
                            {/if}

                            <!-- Categories Table -->
                            <div class="border border-slate-200/80 rounded-xl overflow-hidden bg-white">
                                <table class="w-full border-collapse text-left text-xs">
                                    <thead class="bg-slate-50 border-b border-slate-200">
                                        <tr>
                                            <th class="p-3 font-bold text-slate-600">Nama Kategori</th>
                                            <th class="p-3 font-bold text-slate-600">Kategori Induk</th>
                                            <th class="p-3 font-bold text-slate-600">Mode Bisnis</th>
                                            <th class="p-3 font-bold text-slate-600 text-right">Aksi</th>
                                        </tr>
                                    </thead>
                                    <tbody class="divide-y divide-slate-100">
                                        {#if categories.length === 0}
                                            <tr>
                                                <td colspan="4" class="p-6 text-center text-slate-400">
                                                    Tidak ada kategori terdaftar. Silakan tambahkan di atas.
                                                </td>
                                            </tr>
                                        {:else}
                                            {#each categories as cat}
                                                <tr class="hover:bg-slate-50/80 transition-colors">
                                                    <td class="p-3 text-sm">
                                                        {#if cat.parent_id}
                                                            <span class="text-slate-400 mr-1">↳</span>
                                                            <span class="font-medium text-slate-700">{cat.name}</span>
                                                        {:else}
                                                            <span class="font-bold text-slate-900">{cat.name}</span>
                                                        {/if}
                                                    </td>
                                                    <td class="p-3 text-sm text-slate-500 font-medium">
                                                        {cat.parent_name || '-'}
                                                    </td>
                                                    <td class="p-3 text-xs">
                                                        {#if cat.business_mode === 'fb'}
                                                            <span class="bg-orange-50 text-orange-600 px-2.5 py-1 rounded-full font-bold uppercase tracking-wider text-[10px]">Food & Beverage</span>
                                                        {:else if cat.business_mode === 'retail'}
                                                            <span class="bg-blue-50 text-blue-600 px-2.5 py-1 rounded-full font-bold uppercase tracking-wider text-[10px]">Retail</span>
                                                        {:else if cat.business_mode === 'jasa'}
                                                            <span class="bg-purple-50 text-purple-600 px-2.5 py-1 rounded-full font-bold uppercase tracking-wider text-[10px]">Jasa</span>
                                                        {:else}
                                                            <span class="bg-slate-100 text-slate-600 px-2.5 py-1 rounded-full font-bold uppercase tracking-wider text-[10px]">Semua Mode</span>
                                                        {/if}
                                                    </td>
                                                    <td class="p-3 text-right space-x-2">
                                                        <button 
                                                            onclick={() => startEditCategory(cat)}
                                                            class="bg-blue-50 text-blue-600 hover:bg-blue-100 font-bold px-3 py-1.5 rounded-lg text-xs transition-colors cursor-pointer"
                                                        >
                                                            Edit
                                                        </button>
                                                        <button 
                                                            onclick={() => removeCategory(cat.id)}
                                                            class="bg-rose-50 text-rose-600 hover:bg-rose-100 font-bold px-3 py-1.5 rounded-lg text-xs transition-colors cursor-pointer"
                                                        >
                                                            Hapus
                                                        </button>
                                                    </td>
                                                </tr>
                                            {/each}
                                        {/if}
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    </div>
                {/if}

                <!-- TAB 3: Tax & Discounts -->
                {#if activeTab === "tax_discount"}
                    <div class="space-y-6">
                        <div>
                            <h2 class="text-xl font-bold text-slate-800">Biaya, Pajak & Diskon Toko</h2>
                            <p class="text-xs text-slate-400 mt-1">Konfigurasi parameter keuangan default yang digunakan dalam perhitungan transaksi POS.</p>
                        </div>
                        <div class="border-t border-slate-100 pt-4 space-y-4">
                            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                                <div>
                                    <label for="tax-rate" class="block text-xs font-bold text-slate-500 uppercase tracking-wider mb-2">Pajak Transaksi (PPN) %</label>
                                    <div class="relative">
                                        <input 
                                            id="tax-rate"
                                            type="number" 
                                            bind:value={taxRate} 
                                            min="0" 
                                            max="100" 
                                            class="w-full bg-slate-50 border border-slate-200 rounded-xl pl-4 pr-10 py-3 text-sm focus:outline-none focus:border-blue-500 focus:bg-white transition-all font-medium text-slate-800"
                                        />
                                        <span class="absolute right-4 top-1/2 -translate-y-1/2 text-slate-400 font-bold text-sm">%</span>
                                    </div>
                                    <span class="text-[10px] text-slate-400 mt-1 block">Default regulasi Indonesia saat ini adalah 11%.</span>
                                </div>

                                <div>
                                    <label for="service-charge" class="block text-xs font-bold text-slate-500 uppercase tracking-wider mb-2">Biaya Layanan (Service Charge) %</label>
                                    <div class="relative">
                                        <input 
                                            id="service-charge"
                                            type="number" 
                                            bind:value={serviceCharge} 
                                            min="0" 
                                            max="100" 
                                            class="w-full bg-slate-50 border border-slate-200 rounded-xl pl-4 pr-10 py-3 text-sm focus:outline-none focus:border-blue-500 focus:bg-white transition-all font-medium text-slate-800"
                                        />
                                        <span class="absolute right-4 top-1/2 -translate-y-1/2 text-slate-400 font-bold text-sm">%</span>
                                    </div>
                                    <span class="text-[10px] text-slate-400 mt-1 block">Umumnya digunakan oleh tipe toko Food & Beverage.</span>
                                </div>
                            </div>

                            <div>
                                <label for="quick-discounts" class="block text-xs font-bold text-slate-500 uppercase tracking-wider mb-2">Pilihan Diskon Cepat (Dipisahkan Koma)</label>
                                <input 
                                    id="quick-discounts"
                                    type="text" 
                                    bind:value={quickDiscountsInput} 
                                    placeholder="e.g. 5000, 10000, 5%, 10%" 
                                    class="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-3 text-sm focus:outline-none focus:border-blue-500 focus:bg-white transition-all font-medium text-slate-800"
                                />
                                <span class="text-[10px] text-slate-400 mt-1.5 block">Format: Tulis nominal angka murni untuk potongan rupiah (misal: <code>5000</code>), atau akhiri dengan tanda % untuk potongan persen (misal: <code>10%</code>).</span>
                            </div>
                        </div>

                        <div class="border-t border-slate-100 pt-6 flex justify-end">
                            <button 
                                onclick={saveGeneralSettings}
                                disabled={isSavingSettings}
                                class="bg-blue-600 hover:bg-blue-700 text-white font-bold text-sm py-3 px-6 rounded-xl shadow-md hover:shadow-lg active:scale-98 transition-all cursor-pointer"
                            >
                                {isSavingSettings ? 'Menyimpan...' : 'Simpan Perubahan'}
                            </button>
                        </div>
                    </div>
                {/if}

                <!-- TAB 4: System Links -->
                {#if activeTab === "system"}
                    <div class="space-y-6">
                        <div>
                            <h2 class="text-xl font-bold text-slate-800">Pengaturan Sistem & Perangkat Keras</h2>
                            <p class="text-xs text-slate-400 mt-1">Akses menu pengaturan lanjutan dan interaksi hardware POSQ.</p>
                        </div>
                        <div class="border-t border-slate-100 pt-4 grid grid-cols-1 sm:grid-cols-2 gap-4">
                            <!-- Link 1: Printer & Scanner -->
                            <a href="/settings/hardware" class="p-5 rounded-2xl border border-slate-200/80 hover:border-blue-500 hover:bg-blue-50/10 transition-all flex flex-col gap-1.5 group text-left">
                                <span class="text-2xl group-hover:scale-110 transition-transform w-fit">🖨️</span>
                                <span class="text-sm font-bold text-slate-800 mt-1">Perangkat Keras (Printer & Scan)</span>
                                <span class="text-xs text-slate-500">Konfigurasi pemindai barcode, printer bluetooth, printer USB/Mock, dan kustomisasi struk thermal.</span>
                            </a>

                            <!-- Link 2: Mode Bisnis -->
                            <a href="/settings/business-mode" class="p-5 rounded-2xl border border-slate-200/80 hover:border-blue-500 hover:bg-blue-50/10 transition-all flex flex-col gap-1.5 group text-left">
                                <span class="text-2xl group-hover:scale-110 transition-transform w-fit">🏪</span>
                                <span class="text-sm font-bold text-slate-800 mt-1">Mode Bisnis (Retail, F&B & Jasa)</span>
                                <span class="text-xs text-slate-500">Sesuaikan mode operasi kasir Anda ke Retail (stok & retur), F&B (meja & tiket dapur), atau Jasa (penugasan teknisi/terapis).</span>
                            </a>

                            <!-- Link 3: Backup Data -->
                            <a href="/settings/backup" class="p-5 rounded-2xl border border-slate-200/80 hover:border-blue-500 hover:bg-blue-50/10 transition-all flex flex-col gap-1.5 group text-left">
                                <span class="text-2xl group-hover:scale-110 transition-transform w-fit">💾</span>
                                <span class="text-sm font-bold text-slate-800 mt-1">Backup & Pemulihan</span>
                                <span class="text-xs text-slate-500">Simpan cadangan database SQLite lokal Anda, atur kunci pemulihan darurat, atau impor kembali data transaksi.</span>
                            </a>

                            <!-- Link 4: Lisensi -->
                            <a href="/settings/license" class="p-5 rounded-2xl border border-slate-200/80 hover:border-blue-500 hover:bg-blue-50/10 transition-all flex flex-col gap-1.5 group text-left">
                                <span class="text-2xl group-hover:scale-110 transition-transform w-fit">🔑</span>
                                <span class="text-sm font-bold text-slate-800 mt-1">Lisensi Aplikasi</span>
                                <span class="text-xs text-slate-500">Lihat status masa aktif langganan kasir Anda, aktivasi perangkat, dan sinkronisasi lisensi baru.</span>
                            </a>

                            <!-- Link 5: Jaringan -->
                            <a href="/settings/network" class="p-5 rounded-2xl border border-slate-200/80 hover:border-blue-500 hover:bg-blue-50/10 transition-all flex flex-col gap-1.5 group text-left">
                                <span class="text-2xl group-hover:scale-110 transition-transform w-fit">🌐</span>
                                <span class="text-sm font-bold text-slate-800 mt-1">Jaringan & Sinkronisasi</span>
                                <span class="text-xs text-slate-500">Konfigurasi topologi Master/Client dan sinkronisasi otomatis Cloud (VPS).</span>
                            </a>

                            <!-- Link 6: Update -->
                            <a href="/settings/update" class="p-5 rounded-2xl border border-slate-200/80 hover:border-blue-500 hover:bg-blue-50/10 transition-all flex flex-col gap-1.5 group text-left sm:col-span-2">
                                <div class="flex items-center gap-3">
                                    <span class="text-2xl group-hover:scale-110 transition-transform">🔄</span>
                                    <div>
                                        <span class="text-sm font-bold text-slate-800 block">Pembaruan & Update Log</span>
                                        <span class="text-xs text-slate-500 mt-0.5 block">Periksa ketersediaan versi terbaru POSQ dan riwayat log pembaruan.</span>
                                    </div>
                                </div>
                            </a>
                        </div>
                    </div>
                {/if}
            </div>
        </div>
    </div>
</div>

<style>
    /* Override global app.css specificity for category inputs and selects */
    input[type="text"].custom-category-input,
    select.custom-category-input {
        font-size: 0.875rem !important;
        padding-top: 0.25rem !important;
        padding-bottom: 0.25rem !important;
        padding-left: 1rem !important;
        padding-right: 2rem !important;
        height: 2.75rem !important;
        line-height: normal !important;
        box-sizing: border-box !important;
    }
</style>
