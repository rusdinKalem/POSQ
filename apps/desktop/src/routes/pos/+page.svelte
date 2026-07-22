<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';
    import { listen } from '@tauri-apps/api/event';
    import { goto } from '$app/navigation';
    import { showToast } from '$lib/toast.svelte';

    // Subcomponents
    import ProductSearch from '$lib/components/pos/ProductSearch.svelte';
    import ProductGrid, { type Product, type CategoryItem } from '$lib/components/pos/ProductGrid.svelte';
    import CartPanel, { type Customer } from '$lib/components/pos/CartPanel.svelte';
    import { type CartItem } from '$lib/components/pos/CartItemRow.svelte';
    import PaymentModal from '$lib/components/pos/PaymentModal.svelte';
    import TransactionSuccessModal from '$lib/components/pos/TransactionSuccessModal.svelte';

    type TableInfo = {
        id: string;
        name: string;
        status: string; // 'available' | 'occupied' | 'reserved' | 'dirty' | 'disabled'
        version: number;
        active_session: any;
    };

    interface Category {
        id: string;
        name: string;
        parent_id?: string | null;
        parent_name?: string | null;
        business_mode?: string | null;
    }

    interface ModifierOption {
        name: string;
        price: number;
    }

    // Core Reactivity
    let products: Product[] = $state([]);
    let cart: CartItem[] = $state([]);
    let shiftActive: boolean = $state(true);
    let shiftId: string | null = $state(null);
    let businessMode: string = $state('fb');
    
    // Modal states
    let showShiftModal: boolean = $state(false);
    let showPaymentModal: boolean = $state(false);
    let showCustomItemModal: boolean = $state(false);
    let showCustomerModal: boolean = $state(false);
    let showHoldModal: boolean = $state(false);
    let showSavedBillsModal: boolean = $state(false);
    let showCartDiscountModal: boolean = $state(false);
    let showItemDiscountModal: boolean = $state(false);
    let showItemCustomizerModal: boolean = $state(false);
    let showSuccessModal: boolean = $state(false);

    // Success Modal Data
    let lastSuccessOrderId: string | null = $state(null);
    let lastSuccessGrandTotal: number = $state(0);
    let lastSuccessPaidTotal: number = $state(0);
    let lastSuccessChangeTotal: number = $state(0);
    let lastSuccessPaymentMethod: string = $state("CASH");

    // Filter & Search
    let searchQuery: string = $state("");
    let selectedMainCategory: string = $state("Semua");
    let selectedSubCategory: string = $state("Semua");

    // Form inputs & resources
    let startingCash: number = $state(0);
    let orderType: string = $state("dine_in"); // "dine_in" | "takeaway" | "delivery"
    let tableNumber: string = $state("");
    let showResourceModal: boolean = $state(false);

    let tables: TableInfo[] = $state([]);
    let isLoadingTables: boolean = $state(false);
    let staffs: string[] = $state(["Andi (Terapis)", "Budi (Terapis)", "Citra (Terapis)", "Dewi (Hairdresser)", "Eko (Barber)"]);
    let newResourceName: string = $state("");
    let activeServices: string[] = $state([]);

    // Custom item inputs
    let customItemName: string = $state("");
    let customItemPrice: number = $state(0);
    let customItemCategory: string = $state("Kustom");

    const DEFAULT_CUSTOMER: Customer = { id: '1', name: 'Umum / Walk-in', phone: '-' };
    let customers: Customer[] = $state([
        DEFAULT_CUSTOMER,
        { id: '2', name: 'Rusdin', phone: '08123456789' },
        { id: '3', name: 'Budi', phone: '08234567890' },
        { id: '4', name: 'Dewi', phone: '08345678901' }
    ]);
    let selectedCustomer: Customer = $state(DEFAULT_CUSTOMER);
    let newCustName: string = $state("");
    let newCustPhone: string = $state("");

    // Cart calculations
    let subtotal: number = $state(0);
    let tax: number = $state(0);
    let grandTotal: number = $state(0);
    let cartDiscountType: 'nominal' | 'percent' = $state('nominal');
    let cartDiscountValue: number = $state(0);
    let activeCartDiscount: number = $state(0);
    let activeTaxRate: number = $state(11);

    // Item Discount & Customize state
    let discountingItemId: string | null = $state(null);
    let itemDiscountType: 'nominal' | 'percent' = $state('nominal');
    let itemDiscountValue: number = $state(0);

    let customVariants: ModifierOption[] = $state([]);
    let customAddons: ModifierOption[] = $state([]);
    let showAddVariantForm = $state(false);
    let newVariantName = $state("");
    let newVariantPrice = $state(0);
    let showAddAddonForm = $state(false);
    let newAddonName = $state("");
    let newAddonPrice = $state(0);

    let customizingItem: CartItem | null = $state(null);
    let customizeSize: string = $state("Regular");
    let customizeModifiers: string[] = $state([]);
    let customizeNotes: string = $state("");

    // Hold / Saved Bills
    let holdBillName: string = $state("");
    let savedBills: any[] = $state([]);

    // ECR Integration State
    let ecrEnabled: boolean = $state(false);
    let ecrPortName: string = $state("COM3");
    let ecrBaudRate: number = $state(115200);
    let ecrWaiting: boolean = $state(false);
    let ecrStatusMessage: string = $state("");
    let unlistenEcrResult: (() => void) | null = null;
    let unlistenEcrStatus: (() => void) | null = null;

    // Responsive design helper
    let innerWidth: number = $state(1024);
    let activeTab: string = $state("products"); // "products" | "cart"
    let isMobile: boolean = $derived(innerWidth < 768);
    let isLoaded: boolean = $state(false);

    let dbCategories: Category[] = $state([]);

    // Busy resources derived
    let busyResources = $derived.by(() => {
        const busy: string[] = [];
        for (const tbl of tables) {
            if (tbl.status !== 'available') {
                busy.push(tbl.name);
            }
        }
        for (const bill of savedBills) {
            try {
                if (bill.cart_json) {
                    const data = JSON.parse(bill.cart_json);
                    if (data.tableNumber && !busy.includes(data.tableNumber)) {
                        busy.push(data.tableNumber);
                    }
                }
            } catch (e) {
                // ignore
            }
        }
        return busy;
    });

    async function fetchCategories() {
        try {
            dbCategories = await invoke('list_categories');
        } catch (e) {
            console.error('Failed to load categories:', e);
        }
    }

    let activeCategories = $derived(dbCategories.filter(cat => !cat.business_mode || cat.business_mode === businessMode));

    let mainCategories: CategoryItem[] = $derived([
        { id: "Semua", name: "Semua" },
        ...activeCategories.filter(cat => !cat.parent_id).map(c => ({ id: c.id, name: c.name }))
    ]);

    let subCategories: CategoryItem[] = $derived.by(() => {
        if (selectedMainCategory === "Semua") {
            return [];
        }
        const list = activeCategories.filter(cat => cat.parent_id === selectedMainCategory);
        if (list.length === 0) return [];
        return [
            { id: "Semua", name: `Semua ${mainCategories.find(c => c.id === selectedMainCategory)?.name || ''}` },
            ...list.map(c => ({ id: c.id, name: c.name }))
        ];
    });

    $effect(() => {
        if (selectedMainCategory) {
            selectedSubCategory = "Semua";
        }
    });

    $effect(() => {
        if (businessMode) {
            selectedMainCategory = "Semua";
            selectedSubCategory = "Semua";
        }
    });

    let filteredProducts = $derived(products.filter(p => {
        if (p.category_id) {
            const cat = dbCategories.find(c => c.id === p.category_id);
            if (cat && cat.business_mode && cat.business_mode !== businessMode) {
                return false;
            }
        }

        const matchesSearch = p.name.toLowerCase().includes(searchQuery.toLowerCase()) || 
                              p.sku.toLowerCase().includes(searchQuery.toLowerCase());
        if (!matchesSearch) return false;

        if (selectedMainCategory === "Semua") {
            return true;
        }

        if (selectedSubCategory === "Semua") {
            const isDirectMatch = p.category_id === selectedMainCategory;
            const subCategoryIds = activeCategories.filter(c => c.parent_id === selectedMainCategory).map(c => c.id);
            const isSubMatch = p.category_id ? subCategoryIds.includes(p.category_id) : false;
            return isDirectMatch || isSubMatch;
        } else {
            return p.category_id === selectedSubCategory;
        }
    }));

    onMount(async () => {
        businessMode = localStorage.getItem('businessMode') || 'fb';
        
        const storedVariants = localStorage.getItem('posq_custom_variants');
        if (storedVariants) {
            customVariants = JSON.parse(storedVariants);
        } else {
            customVariants = [
                { name: "Regular", price: 0 },
                { name: "Large", price: 5000 }
            ];
            localStorage.setItem('posq_custom_variants', JSON.stringify(customVariants));
        }

        const storedAddons = localStorage.getItem('posq_custom_addons');
        if (storedAddons) {
            customAddons = JSON.parse(storedAddons);
        } else {
            customAddons = [
                { name: "Extra Syrup", price: 3000 },
                { name: "Oat Milk", price: 6000 }
            ];
            localStorage.setItem('posq_custom_addons', JSON.stringify(customAddons));
        }

        const savedTax = localStorage.getItem("posq_tax_rate");
        if (savedTax !== null) {
            activeTaxRate = parseFloat(savedTax);
        }
        const savedServices = localStorage.getItem('posq_active_services');
        if (savedServices) {
            try {
                activeServices = JSON.parse(savedServices);
            } catch (e) {
                console.error(e);
            }
        }
        await checkShift();
        if (shiftActive) {
            await fetchCategories();
            await fetchProducts();
            await fetchSavedBills();
            await fetchTables();

            try {
                const urlParams = new URLSearchParams(window.location.search);
                const tableParam = urlParams.get('table');
                if (tableParam) {
                    tableNumber = decodeURIComponent(tableParam);
                    orderType = 'dine_in';
                }
            } catch (e) {
                // ignore
            }
            try {
                const draft: string | null = await invoke('get_cart_draft');
                if (draft) {
                    const parsed = JSON.parse(draft);
                    if (Array.isArray(parsed)) {
                        cart = parsed;
                    } else if (parsed && parsed.cart) {
                        cart = parsed.cart;
                        orderType = parsed.orderType || 'dine_in';
                        tableNumber = parsed.tableNumber || '';
                        selectedCustomer = parsed.selectedCustomer || DEFAULT_CUSTOMER;
                        cartDiscountType = parsed.cartDiscountType || 'nominal';
                        cartDiscountValue = parsed.cartDiscountValue || 0;
                    }
                    calculateCart();
                }
            } catch (e) {
                console.error('Failed to load cart draft:', e);
            }
        }
        isLoaded = true;
        
        ecrEnabled = localStorage.getItem("posq_ecr_enabled") === "true";
        ecrPortName = localStorage.getItem("posq_ecr_port") || "COM3";
        ecrBaudRate = parseInt(localStorage.getItem("posq_ecr_baud") || "115200");
        
        unlistenEcrResult = await listen('ecr-transaction-result', (event: any) => {
            const result = event.payload;
            ecrWaiting = false;
            if (result.success) {
                showToast('EDC Berhasil! Approval: ' + (result.approval_code || ''), 'success');
            } else {
                showToast('EDC Gagal: ' + result.message, 'error');
            }
            ecrStatusMessage = '';
        }) as unknown as () => void;
        
        unlistenEcrStatus = await listen('ecr-status', (event: any) => {
            ecrStatusMessage = event.payload as string;
        }) as unknown as () => void;
    });

    onDestroy(() => {
        if (unlistenEcrResult) unlistenEcrResult();
        if (unlistenEcrStatus) unlistenEcrStatus();
    });

    async function sendToEcr() {
        ecrWaiting = true;
        ecrStatusMessage = 'Membuka port serial...';
        try {
            await invoke('start_ecr_transaction', {
                amount: grandTotal,
                portName: ecrPortName,
                baudRate: ecrBaudRate
            });
        } catch (e) {
            ecrWaiting = false;
            showToast('Gagal mengirim ke EDC: ' + e, 'error');
        }
    }

    $effect(() => {
        const currentCart = cart;
        if (isLoaded) {
            const draftData = {
                cart: currentCart,
                orderType,
                tableNumber,
                selectedCustomer,
                cartDiscountType,
                cartDiscountValue
            };
            invoke('save_cart_draft', { cartJson: JSON.stringify(draftData) })
                .catch(e => console.error('Failed to auto-save cart:', e));
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
            await fetchCategories();
            await fetchProducts();
            await fetchSavedBills();
            showToast('Shift berhasil dibuka!', 'success');
        } catch (e) {
            showToast('Gagal membuka shift: ' + e, 'error');
        }
    }

    async function fetchProducts() {
        try {
            products = await invoke('get_products');
        } catch (e) {
            console.error('Failed to load products:', e);
        }
    }

    function calculateCart() {
        subtotal = cart.reduce((sum, item) => sum + (item.unit_price * item.qty - item.discount_total), 0);
        
        if (cartDiscountType === 'percent') {
            activeCartDiscount = Math.round(subtotal * (cartDiscountValue / 100));
        } else {
            activeCartDiscount = cartDiscountValue;
        }

        if (activeCartDiscount > subtotal) {
            activeCartDiscount = subtotal;
        }

        tax = Math.round((subtotal - activeCartDiscount) * (activeTaxRate / 100));
        grandTotal = subtotal - activeCartDiscount + tax;
        
        if (grandTotal < 0) grandTotal = 0;
    }

    function addToCart(product: Product) {
        let existing = cart.find(i => i.product_id === product.id && !i.variant_name && (!i.modifiers || i.modifiers.length === 0));
        if (existing) {
            if (existing.qty >= product.qty_on_hand) {
                showToast('Stok produk ini tidak mencukupi!', 'error');
                return;
            }
            existing.qty += 1;
            existing.line_total = existing.qty * existing.unit_price - existing.discount_total;
            cart = [...cart];
        } else {
            if (product.qty_on_hand < 1) {
                showToast('Stok produk ini habis!', 'error');
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

    function increaseQty(productId: string) {
        let item = cart.find(i => i.product_id === productId);
        if (item) {
            let product = products.find(p => p.id === productId);
            if (product && item.qty >= product.qty_on_hand && item.sku !== 'CUSTOM') {
                showToast('Stok produk ini tidak mencukupi!', 'error');
                return;
            }
            item.qty += 1;
            item.line_total = item.qty * item.unit_price - item.discount_total;
            cart = [...cart];
            calculateCart();
        }
    }

    function decreaseQty(productId: string) {
        let item = cart.find(i => i.product_id === productId);
        if (item) {
            if (item.qty > 1) {
                item.qty -= 1;
                item.line_total = item.qty * item.unit_price - item.discount_total;
                cart = [...cart];
            } else {
                cart = cart.filter(i => i.product_id !== productId);
            }
            calculateCart();
        }
    }

    function removeFromCart(productId: string) {
        cart = cart.filter(i => i.product_id !== productId);
        calculateCart();
    }

    function handleClearCart() {
        cart = [];
        calculateCart();
        showToast('Keranjang telah dikosongkan.', 'info');
    }

    function addCustomItem() {
        if (!customItemName) return;
        const tempId = `custom_${Date.now()}`;
        const selectedCat = activeCategories.find(c => c.name === customItemCategory);
        cart = [...cart, {
            product_id: tempId,
            sku: 'CUSTOM',
            name: customItemName,
            qty: 1,
            unit_price: customItemPrice || 0,
            discount_total: 0,
            line_total: customItemPrice || 0,
            category_name: customItemCategory,
            category_id: selectedCat ? selectedCat.id : null
        }];
        calculateCart();
        showCustomItemModal = false;
        customItemName = "";
        customItemPrice = 0;
        customItemCategory = "Kustom";
    }

    function openItemDiscount(productId: string) {
        discountingItemId = productId;
        const item = cart.find(i => i.product_id === productId);
        if (item) {
            itemDiscountValue = item.discount_total;
            itemDiscountType = 'nominal';
            showItemDiscountModal = true;
        }
    }

    function applyItemDiscount() {
        const item = cart.find(i => i.product_id === discountingItemId);
        if (item) {
            let discountAmt = 0;
            if (itemDiscountType === 'percent') {
                discountAmt = Math.round((item.unit_price * item.qty) * (itemDiscountValue / 100));
            } else {
                discountAmt = itemDiscountValue;
            }
            if (discountAmt > item.unit_price * item.qty) {
                discountAmt = item.unit_price * item.qty;
            }
            item.discount_total = discountAmt;
            item.line_total = (item.unit_price * item.qty) - discountAmt;
            cart = [...cart];
            calculateCart();
        }
        showItemDiscountModal = false;
    }

    function openItemCustomizer(item: CartItem) {
        customizingItem = item;
        customizeSize = item.variant_name || "Regular";
        customizeModifiers = item.modifiers || [];
        customizeNotes = item.notes || "";
        showItemCustomizerModal = true;
    }

    function saveItemCustomization() {
        if (customizingItem) {
            const item = cart.find(i => i.product_id === customizingItem!.product_id);
            if (item) {
                item.variant_name = customizeSize;
                item.modifiers = customizeModifiers;
                item.notes = customizeNotes;
                
                let basePrice = item.unit_price;
                const origProduct = products.find(p => p.id === item.product_id);
                if (origProduct) {
                    basePrice = origProduct.price;
                }
                
                let extraPrice = 0;
                const selectedVariant = customVariants.find(v => v.name === customizeSize);
                if (selectedVariant) {
                    extraPrice += selectedVariant.price;
                }
                customizeModifiers.forEach(modName => {
                    const addon = customAddons.find(a => a.name === modName);
                    if (addon) {
                        extraPrice += addon.price;
                    }
                });
                
                item.unit_price = basePrice + extraPrice;
                item.line_total = item.qty * item.unit_price - item.discount_total;
                
                cart = [...cart];
                calculateCart();
            }
        }
        showItemCustomizerModal = false;
    }

    async function handleHoldBillClick() {
        if (cart.length === 0) return;
        const isCustomerUmum = selectedCustomer.name === 'Umum / Walk-in';
        const isTableEmpty = !tableNumber || tableNumber.trim() === '';
        
        if (!isCustomerUmum || !isTableEmpty) {
            let parts = [];
            if (!isCustomerUmum) parts.push(selectedCustomer.name);
            if (!isTableEmpty) {
                parts.push(businessMode === 'jasa' ? `Teknisi ${tableNumber}` : `Meja ${tableNumber}`);
            }
            holdBillName = parts.join(' - ');
            await holdBill();
        } else {
            showHoldModal = true;
        }
    }

    async function holdBill() {
        if (cart.length === 0) return;
        const prefix = businessMode === 'jasa' ? 'Teknisi' : 'Meja';
        const name = holdBillName || `${prefix} ${tableNumber || '-'}`;
        const id = `hold_${Date.now()}`;
        const cartData = JSON.stringify({
            cart,
            orderType,
            tableNumber,
            selectedCustomer,
            cartDiscountType,
            cartDiscountValue
        });

        try {
            await invoke('save_hold_draft', { id, name, cartJson: cartData });
            cart = [];
            holdBillName = "";
            tableNumber = "";
            selectedCustomer = DEFAULT_CUSTOMER;
            orderType = 'dine_in';
            activeCartDiscount = 0;
            cartDiscountValue = 0;
            calculateCart();
            showHoldModal = false;
            
            await fetchSavedBills();
            showToast('Tagihan berhasil disimpan!', 'success');
        } catch (e) {
            showToast('Gagal menyimpan tagihan: ' + e, 'error');
        }
    }

    async function fetchSavedBills() {
        try {
            savedBills = await invoke('list_hold_drafts');
        } catch (e) {
            console.error('Gagal memuat tagihan tersimpan:', e);
        }
    }

    async function loadSavedBills() {
        await fetchSavedBills();
        showSavedBillsModal = true;
    }

    async function resumeBill(bill: any) {
        try {
            const data = JSON.parse(bill.cart_json);
            cart = data.cart || [];
            orderType = data.orderType || 'dine_in';
            tableNumber = data.tableNumber || '';
            selectedCustomer = data.selectedCustomer || DEFAULT_CUSTOMER;
            cartDiscountType = data.cartDiscountType || 'nominal';
            cartDiscountValue = data.cartDiscountValue || 0;
            calculateCart();
            
            await invoke('delete_hold_draft', { id: bill.id });
            await fetchSavedBills();
            showSavedBillsModal = false;
            showToast('Tagihan dipulihkan!', 'success');
        } catch (e) {
            showToast('Gagal memulihkan tagihan: ' + e, 'error');
        }
    }

    async function deleteBill(id: string) {
        if (!confirm('Hapus tagihan ini?')) return;
        try {
            await invoke('delete_hold_draft', { id });
            savedBills = savedBills.filter(b => b.id !== id);
            showToast('Tagihan dihapus', 'success');
        } catch (e) {
            showToast('Gagal menghapus tagihan: ' + e, 'error');
        }
    }

    function quickAddCustomer() {
        if (!newCustName) return;
        const newCust = {
            id: `cust_${Date.now()}`,
            name: newCustName,
            phone: newCustPhone || '-'
        };
        customers = [...customers, newCust];
        selectedCustomer = newCust;
        newCustName = "";
        newCustPhone = "";
        showCustomerModal = false;
    }

    async function fetchTables() {
        if (businessMode !== 'fb') return;
        isLoadingTables = true;
        try {
            tables = await invoke('get_all_tables_status') as TableInfo[];
        } catch (e) {
            console.error('Gagal memuat data meja:', e);
        } finally {
            isLoadingTables = false;
        }
    }

    async function quickAddResource() {
        if (!newResourceName.trim()) return;
        const name = newResourceName.trim();
        if (businessMode === 'fb') {
            try {
                const newTable = await invoke('add_new_table', { name }) as TableInfo;
                tables = [...tables, newTable];
                tableNumber = name;
                newResourceName = "";
                showResourceModal = false;
                showToast(`Meja '${name}' berhasil ditambahkan`, 'success');
            } catch (e) {
                showToast(`Gagal menambah meja: ${e}`, 'error');
            }
        } else if (businessMode === 'jasa') {
            if (!staffs.includes(name)) {
                staffs = [...staffs, name];
            }
            tableNumber = name;
            newResourceName = "";
            showResourceModal = false;
        }
    }

    async function releaseResource(name: string) {
        if (businessMode === 'fb') {
            try {
                await invoke('release_table_session', { tableName: name });
                await fetchTables();
                if (tableNumber === name) tableNumber = '';
                showToast(`Meja '${name}' telah dibebaskan`, 'success');
            } catch (e) {
                showToast(`Gagal membebaskan meja: ${e}`, 'error');
            }
        } else {
            activeServices = activeServices.filter(x => x !== name);
            localStorage.setItem('posq_active_services', JSON.stringify(activeServices));
        }
    }

    async function submitPaymentProcess(params: {
        paymentMethod: string;
        amountPaid: number;
        cashPaid: number;
        cardPaid: number;
        cardBank: string;
        cardApprovalCode: string;
        cardTraceNumber: string;
    }) {
        let totalPaid = 0;
        let pMethod = params.paymentMethod;
        
        if (params.paymentMethod === 'SPLIT') {
            totalPaid = params.cashPaid + params.cardPaid;
            pMethod = `SPLIT: CASH=${params.cashPaid}, CARD=${params.cardPaid}`;
        } else {
            totalPaid = params.amountPaid;
        }

        if (totalPaid < grandTotal) {
            showToast('Pembayaran kurang!', 'error');
            return;
        }
        
        let change = totalPaid - grandTotal;
        
        const checkoutItems = cart.map(item => {
            let noteParts = [];
            if (item.variant_name && item.variant_name !== 'Regular') {
                noteParts.push(`Varian: ${item.variant_name}`);
            }
            if (item.modifiers && item.modifiers.length > 0) {
                noteParts.push(`Add-ons: ${item.modifiers.join(', ')}`);
            }
            if (item.notes && item.notes.trim() !== '') {
                noteParts.push(item.notes);
            }
            return {
                product_id: item.product_id.startsWith('custom_') ? null : item.product_id,
                sku: item.sku,
                name: item.name,
                qty: item.qty,
                unit_price: item.unit_price,
                discount_total: item.discount_total,
                line_total: item.line_total,
                notes: noteParts.length > 0 ? noteParts.join(' | ') : null
            };
        });

        const payload: any = {
            shift_id: shiftId,
            subtotal,
            discount_total: activeCartDiscount,
            tax_total: tax,
            service_total: 0,
            grand_total: grandTotal,
            paid_total: totalPaid,
            change_total: change,
            payment_method: pMethod,
            order_type: orderType,
            table_number: orderType === 'dine_in' ? tableNumber : null,
            items: checkoutItems
        };

        if (params.paymentMethod === 'CARD') {
            if (!params.cardApprovalCode || params.cardApprovalCode.trim() === '') {
                showToast('Mohon isi Approval Code dari mesin EDC', 'error');
                return;
            }
            payload.card_details = {
                bank: params.cardBank,
                approval_code: params.cardApprovalCode,
                trace_number: params.cardTraceNumber,
                edc_mode: 'manual'
            };
        }

        try {
            let orderId: string = await invoke('checkout', { payload });
            
            // Save last success details before resetting cart
            lastSuccessOrderId = orderId;
            lastSuccessGrandTotal = grandTotal;
            lastSuccessPaidTotal = totalPaid;
            lastSuccessChangeTotal = change;
            lastSuccessPaymentMethod = pMethod;

            cart = [];
            if (tableNumber) {
                if (!activeServices.includes(tableNumber)) {
                    activeServices = [...activeServices, tableNumber];
                    localStorage.setItem('posq_active_services', JSON.stringify(activeServices));
                }
            }
            tableNumber = "";
            selectedCustomer = DEFAULT_CUSTOMER;
            await invoke('clear_cart_draft');
            showPaymentModal = false;
            
            // Show Success Modal
            showSuccessModal = true;
            showToast('Pembayaran Berhasil! Kembalian: Rp ' + change.toLocaleString('id-ID'), 'success');
        } catch (e: any) {
            showToast('Gagal memproses transaksi: ' + (e?.message || e), 'error');
        }
    }

    function handleStartNewTransaction() {
        showSuccessModal = false;
        lastSuccessOrderId = null;
        cart = [];
        calculateCart();
    }

    function handlePrintReceipt() {
        if (lastSuccessOrderId) {
            goto(`/receipt?order_id=${lastSuccessOrderId}`);
        }
    }

    // Barcode scanner buffer & listener
    let barcodeBuffer: string = "";
    let lastKeyTime: number = 0;
    let barcodeToastMessage: string = $state("");
    let barcodeToastVisible: boolean = $state(false);
    let barcodeToastTimeout: any;

    function handleGlobalKeydown(event: KeyboardEvent) {
        const target = event.target as HTMLElement;
        const isInput = target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.tagName === 'SELECT';
        const isSearchInput = target.getAttribute('placeholder')?.includes('Cari menu atau scan SKU');

        const currentTime = Date.now();
        
        if (event.key === 'Enter') {
            if (barcodeBuffer.length > 2) {
                processBarcode(barcodeBuffer);
                barcodeBuffer = "";
                event.preventDefault();
            } else if (isSearchInput && searchQuery) {
                processBarcode(searchQuery);
                event.preventDefault();
            }
        } else if (event.key.length === 1) {
            if (!isInput || isSearchInput) {
                if (currentTime - lastKeyTime > 50) {
                    if (!isInput) {
                        barcodeBuffer = event.key;
                    }
                } else {
                    barcodeBuffer += event.key;
                }
                lastKeyTime = currentTime;
            }
        }
    }

    function processBarcode(code: string) {
        const trimmed = code.trim().toLowerCase();
        const found = products.find(p => p.sku.toLowerCase() === trimmed || p.name.toLowerCase() === trimmed);
        if (found) {
            addToCart(found);
            searchQuery = "";
            
            try {
                const audioCtx = new (window.AudioContext || (window as any).webkitAudioContext)();
                const oscillator = audioCtx.createOscillator();
                const gainNode = audioCtx.createGain();
                oscillator.type = 'sine';
                oscillator.frequency.setValueAtTime(1400, audioCtx.currentTime);
                gainNode.gain.setValueAtTime(0.04, audioCtx.currentTime);
                oscillator.connect(gainNode);
                gainNode.connect(audioCtx.destination);
                oscillator.start();
                oscillator.stop(audioCtx.currentTime + 0.08);
            } catch (e) {
                console.error("Audio Context beep failed:", e);
            }
            
            showBarcodeToast(found.name);
        } else {
            showToast(`SKU/Barcode '${code}' tidak ditemukan`, 'error');
        }
    }

    function showBarcodeToast(productName: string) {
        barcodeToastMessage = `Produk masuk keranjang: ${productName}`;
        barcodeToastVisible = true;
        clearTimeout(barcodeToastTimeout);
        barcodeToastTimeout = setTimeout(() => {
            barcodeToastVisible = false;
        }, 2200);
    }

    function addCustomVariant() {
        if (!newVariantName.trim()) {
            showToast("Nama varian tidak boleh kosong", "error");
            return;
        }
        if (customVariants.some(v => v.name.toLowerCase() === newVariantName.trim().toLowerCase())) {
            showToast("Varian dengan nama ini sudah ada", "error");
            return;
        }
        customVariants = [...customVariants, { name: newVariantName.trim(), price: newVariantPrice }];
        localStorage.setItem('posq_custom_variants', JSON.stringify(customVariants));
        newVariantName = "";
        newVariantPrice = 0;
        showAddVariantForm = false;
        showToast("Varian berhasil ditambahkan", "success");
    }

    function deleteCustomVariant(name: string) {
        if (name === "Regular") {
            showToast("Varian Regular tidak dapat dihapus", "error");
            return;
        }
        customVariants = customVariants.filter(v => v.name !== name);
        localStorage.setItem('posq_custom_variants', JSON.stringify(customVariants));
        if (customizeSize === name) {
            customizeSize = "Regular";
        }
        showToast("Varian berhasil dihapus", "success");
    }

    function addCustomAddon() {
        if (!newAddonName.trim()) {
            showToast("Nama add-on tidak boleh kosong", "error");
            return;
        }
        if (customAddons.some(a => a.name.toLowerCase() === newAddonName.trim().toLowerCase())) {
            showToast("Add-on dengan nama ini sudah ada", "error");
            return;
        }
        customAddons = [...customAddons, { name: newAddonName.trim(), price: newAddonPrice }];
        localStorage.setItem('posq_custom_addons', JSON.stringify(customAddons));
        newAddonName = "";
        newAddonPrice = 0;
        showAddAddonForm = false;
        showToast("Add-on berhasil ditambahkan", "success");
    }

    function deleteCustomAddon(name: string) {
        customAddons = customAddons.filter(a => a.name !== name);
        localStorage.setItem('posq_custom_addons', JSON.stringify(customAddons));
        customizeModifiers = customizeModifiers.filter(m => m !== name);
        showToast("Add-on berhasil dihapus", "success");
    }
</script>

<svelte:window bind:innerWidth onkeydown={handleGlobalKeydown} />

<div class="pos-container flex flex-col md:flex-row h-[calc(100vh-4rem)] overflow-hidden bg-slate-50 font-sans">
    
    <!-- MOBILE TAB NAVIGATION -->
    {#if isMobile}
        <div class="bg-white border-b border-slate-200 px-4 py-2.5 flex items-center justify-between shadow-xs shrink-0">
            <h1 class="text-lg font-black text-slate-800 tracking-tight">POSQ Kasir</h1>
            <div class="flex bg-slate-100 p-1 rounded-xl border border-slate-200">
                <button 
                    type="button"
                    class="px-4 py-1.5 rounded-lg text-xs font-bold transition-all h-9 flex items-center justify-center gap-1.5 cursor-pointer"
                    class:bg-white={activeTab === 'products'}
                    class:text-blue-600={activeTab === 'products'}
                    class:shadow-sm={activeTab === 'products'}
                    class:text-slate-500={activeTab !== 'products'}
                    onclick={() => activeTab = 'products'}>
                    Produk
                </button>
                <button 
                    type="button"
                    class="px-4 py-1.5 rounded-lg text-xs font-bold transition-all h-9 flex items-center justify-center gap-1.5 relative cursor-pointer"
                    class:bg-white={activeTab === 'cart'}
                    class:text-blue-600={activeTab === 'cart'}
                    class:shadow-sm={activeTab === 'cart'}
                    class:text-slate-500={activeTab !== 'cart'}
                    onclick={() => activeTab = 'cart'}>
                    Keranjang
                    {#if cart.length > 0}
                        <span class="absolute -top-1 -right-1 bg-red-500 text-white rounded-full text-[9px] w-5 h-5 flex items-center justify-center font-bold border-2 border-white">
                            {cart.reduce((sum, item) => sum + item.qty, 0)}
                        </span>
                    {/if}
                </button>
            </div>
        </div>
    {/if}

    <!-- PRODUCT CATALOG SECTION -->
    <div class="product-section flex-1 flex flex-col overflow-hidden p-4 md:p-6" class:!hidden={isMobile && activeTab !== 'products'}>
        <ProductSearch 
            bind:searchQuery
            savedBillsCount={savedBills.length}
            onOpenCustomItem={() => showCustomItemModal = true}
            onOpenSavedBills={loadSavedBills}
        />

        <ProductGrid 
            {filteredProducts}
            {mainCategories}
            {subCategories}
            bind:selectedMainCategory
            bind:selectedSubCategory
            onAddToCart={addToCart}
        />
    </div>

    <!-- CART SIDEBAR SECTION -->
    <CartPanel 
        {cart}
        {businessMode}
        bind:orderType
        bind:tableNumber
        {selectedCustomer}
        {subtotal}
        {tax}
        {grandTotal}
        {activeCartDiscount}
        {activeTaxRate}
        {isMobile}
        {activeTab}
        onIncreaseQty={increaseQty}
        onDecreaseQty={decreaseQty}
        onRemoveFromCart={removeFromCart}
        onOpenItemCustomizer={openItemCustomizer}
        onOpenItemDiscount={openItemDiscount}
        onOpenCustomerModal={() => showCustomerModal = true}
        onOpenResourceModal={() => { showResourceModal = true; fetchTables(); }}
        onOpenCartDiscountModal={() => showCartDiscountModal = true}
        onClearCart={handleClearCart}
        onHoldBill={handleHoldBillClick}
        onOpenPayment={() => showPaymentModal = true}
    />
</div>

<!-- ============================================== -->
<!--                  MODALS CONTAINER              -->
<!-- ============================================== -->

<!-- 1. SHIFT MODAL -->
{#if showShiftModal}
<div class="modal-backdrop fixed inset-0 bg-slate-900/60 backdrop-blur-xs flex items-center justify-center z-50 p-4">
    <div class="bg-white rounded-2xl max-w-sm w-full p-6 shadow-2xl border border-slate-100 text-center">
        <h2 class="text-xl font-black text-slate-800 mb-1">Mulai Shift Baru</h2>
        <p class="mb-5 text-xs text-slate-400">Masukkan saldo laci uang kasir untuk memulai penjualan hari ini.</p>
        <input type="number" bind:value={startingCash} class="w-full text-xl p-3 text-center border border-slate-200 rounded-xl font-bold text-slate-800 focus:border-blue-500 focus:bg-slate-50/50 transition-all outline-none mb-5 h-12" placeholder="Rp 0" />
        <button class="bg-blue-600 hover:bg-blue-700 active:bg-blue-800 text-white font-bold py-3 px-6 rounded-xl transition w-full text-sm shadow-md cursor-pointer h-12 min-w-[48px]" onclick={openShift}>Mulai Shift</button>
    </div>
</div>
{/if}

<!-- 2. CUSTOM ITEM MODAL -->
{#if showCustomItemModal}
<div class="modal-backdrop fixed inset-0 bg-slate-900/60 backdrop-blur-xs flex items-center justify-center z-50 p-4">
    <div class="bg-white rounded-2xl max-w-md w-full p-6 shadow-2xl border border-slate-100">
        <h2 class="text-lg font-black text-slate-800 mb-4">Tambah Produk Kustom</h2>
        
        <div class="space-y-4 mb-6">
            <div>
                <label for="custom-item-name" class="block text-xs font-bold text-slate-500 mb-1">Nama Produk</label>
                <input id="custom-item-name" type="text" bind:value={customItemName} class="w-full p-3 border border-slate-200 rounded-xl text-sm outline-none focus:border-blue-500 h-12" placeholder="Contoh: Nasi Goreng Spesial" />
            </div>
            
            <div>
                <label for="custom-item-price" class="block text-xs font-bold text-slate-500 mb-1">Harga Jual (Rp)</label>
                <input id="custom-item-price" type="number" bind:value={customItemPrice} class="w-full p-3 border border-slate-200 rounded-xl text-sm outline-none focus:border-blue-500 h-12" placeholder="0" />
            </div>

            <div>
                <label for="custom-item-category" class="block text-xs font-bold text-slate-500 mb-1">Kategori</label>
                <select id="custom-item-category" bind:value={customItemCategory} class="w-full p-3 border border-slate-200 rounded-xl text-sm outline-none focus:border-blue-500 bg-white h-12">
                    {#each activeCategories as cat}
                        <option value={cat.name}>{cat.name}</option>
                    {/each}
                    <option value="Kustom">Kustom</option>
                </select>
            </div>
        </div>

        <div class="flex gap-3">
            <button class="bg-white border border-slate-200 text-slate-600 hover:bg-slate-50 font-bold py-2.5 px-6 rounded-xl transition w-1/2 text-xs cursor-pointer h-12 min-w-[48px]" onclick={() => showCustomItemModal = false}>Batal</button>
            <button class="bg-blue-600 hover:bg-blue-700 text-white font-bold py-2.5 px-6 rounded-xl transition w-1/2 text-xs cursor-pointer h-12 min-w-[48px]" onclick={addCustomItem} disabled={!customItemName}>Simpan</button>
        </div>
    </div>
</div>
{/if}

<!-- 3. CUSTOMER SELECTOR / CRM MODAL -->
{#if showCustomerModal}
<div class="modal-backdrop fixed inset-0 bg-slate-900/60 backdrop-blur-xs flex items-center justify-center z-50 p-4">
    <div class="bg-white rounded-2xl max-w-md w-full p-6 shadow-2xl border border-slate-100 flex flex-col max-h-[85vh]">
        <h2 class="text-lg font-black text-slate-800 mb-4">Pilih Pelanggan</h2>
        
        <!-- Customer list selection -->
        <div class="flex-1 overflow-y-auto space-y-2 mb-6">
            {#each customers as cust}
                <button 
                    type="button" 
                    onclick={() => { selectedCustomer = cust; showCustomerModal = false; }}
                    class="w-full p-3 border rounded-xl text-left hover:bg-slate-50 transition flex justify-between items-center cursor-pointer min-h-[52px] {selectedCustomer.id === cust.id ? 'border-blue-500 bg-blue-50' : 'border-slate-200'}">
                    <div>
                        <div class="text-sm font-bold text-slate-800">{cust.name}</div>
                        <div class="text-xs text-slate-400 font-mono">{cust.phone}</div>
                    </div>
                    {#if selectedCustomer.id === cust.id}
                        <span class="text-blue-600 font-bold text-xs">Aktif</span>
                    {/if}
                </button>
            {/each}
        </div>

        <!-- Create new customer section -->
        <div class="border-t border-slate-100 pt-4 space-y-3 shrink-0">
            <div class="text-xs font-bold text-slate-700 uppercase tracking-wider">Tambah Pelanggan Baru</div>
            <div class="grid grid-cols-2 gap-2">
                <input type="text" bind:value={newCustName} placeholder="Nama..." class="p-2.5 border border-slate-200 rounded-lg text-xs outline-none h-11" />
                <input type="text" bind:value={newCustPhone} placeholder="No. HP..." class="p-2.5 border border-slate-200 rounded-lg text-xs outline-none h-11" />
            </div>
            <button class="bg-green-600 hover:bg-green-700 text-white font-bold py-2 rounded-xl text-xs transition w-full cursor-pointer h-11" onclick={quickAddCustomer} disabled={!newCustName}>Tambah & Pilih</button>
        </div>

        <button class="mt-4 bg-white border border-slate-200 text-slate-600 hover:bg-slate-50 font-bold py-2 rounded-xl text-xs transition cursor-pointer h-11 shrink-0" onclick={() => showCustomerModal = false}>Tutup</button>
    </div>
</div>
{/if}

<!-- 3b. TABLE / STAFF SELECTOR MODAL -->
{#if showResourceModal}
<div class="modal-backdrop fixed inset-0 bg-slate-900/60 backdrop-blur-xs flex items-center justify-center z-50 p-4">
    <div class="bg-white rounded-2xl max-w-md w-full p-6 shadow-2xl border border-slate-100 flex flex-col max-h-[85vh]">
        <h2 class="text-lg font-black text-slate-800 mb-4 flex justify-between items-center">
            <span>{businessMode === 'fb' ? 'Pilih Meja' : 'Pilih Teknisi / Terapis'}</span>
            <span class="text-[10px] bg-slate-100 px-2 py-0.5 rounded-full text-slate-500 font-bold uppercase tracking-wider">Status: Tersedia</span>
        </h2>
        
        <!-- Resource list selection -->
        <div class="flex-1 overflow-y-auto space-y-2 mb-6">
            {#if businessMode === 'fb'}
                {#if isLoadingTables}
                    <div class="text-center text-slate-400 text-xs py-4">Memuat data meja...</div>
                {:else}
                {#each tables as tbl}
                    {@const isSelected = tableNumber === tbl.name}
                    {@const isOccupied = tbl.status === 'occupied'}
                    {@const isReserved = tbl.status === 'reserved'}
                    {@const isDirty = tbl.status === 'dirty'}
                    {@const isAvailable = tbl.status === 'available'}
                    <div class="w-full p-3 border rounded-xl flex justify-between items-center transition min-h-[52px] {isSelected ? 'border-blue-500 bg-blue-50' : isAvailable ? 'border-slate-100 hover:bg-slate-50' : 'border-slate-200 bg-slate-50 opacity-90'}">
                        <div>
                            <span class="text-sm font-bold {isAvailable ? 'text-slate-800' : 'text-slate-400'}">{tbl.name}</span>
                            {#if isOccupied}<div class="text-[10px] text-red-400 font-medium mt-0.5">Terisi ({tbl.active_session?.bills?.length ?? 0} tagihan)</div>
                            {:else if isReserved}<div class="text-[10px] text-amber-400 font-medium mt-0.5">Dipesan</div>
                            {:else if isDirty}<div class="text-[10px] text-purple-400 font-medium mt-0.5">Perlu Dibersihkan</div>
                            {:else}<div class="text-[10px] text-green-500 font-medium mt-0.5">Tersedia</div>
                            {/if}
                        </div>
                        <div class="flex items-center gap-2">
                            {#if isSelected}
                                <span class="text-blue-600 font-bold text-xs bg-blue-100/50 px-2 py-0.5 rounded-md">Terpilih</span>
                            {:else if isOccupied}
                                <span class="text-red-500 font-bold text-[11px] bg-red-50 px-2 py-0.5 rounded-full">Terisi</span>
                                <button 
                                    type="button" 
                                    onclick={() => releaseResource(tbl.name)}
                                    class="bg-green-600 hover:bg-green-700 text-white font-bold text-[10px] px-2.5 py-1 rounded-lg transition cursor-pointer h-9">
                                    Bebaskan
                                </button>
                            {:else if isReserved || isDirty}
                                <span class="text-amber-500 font-bold text-[11px] bg-amber-50 px-2 py-0.5 rounded-full">{isReserved ? 'Dipesan' : 'Kotor'}</span>
                            {:else}
                                <button 
                                    type="button" 
                                    onclick={() => { tableNumber = tbl.name; showResourceModal = false; }}
                                    class="bg-blue-600 hover:bg-blue-700 text-white font-bold text-xs px-3.5 py-1.5 rounded-lg transition cursor-pointer h-9">
                                    Pilih
                                </button>
                            {/if}
                        </div>
                    </div>
                {/each}
                {/if}
            {:else if businessMode === 'jasa'}
                {#each staffs as stf}
                    <div class="w-full p-3 border rounded-xl flex justify-between items-center transition min-h-[52px] {tableNumber === stf ? 'border-blue-500 bg-blue-50' : busyResources.includes(stf) ? 'border-slate-200 bg-slate-50 opacity-90' : 'border-slate-100 hover:bg-slate-50'}">
                        <span class="text-sm font-bold {busyResources.includes(stf) ? 'text-slate-400 font-medium' : 'text-slate-800'}">{stf}</span>
                        <div class="flex items-center gap-2">
                            {#if tableNumber === stf}
                                <span class="text-blue-600 font-bold text-xs bg-blue-100/50 px-2 py-0.5 rounded-md">Terpilih</span>
                            {:else if activeServices.includes(stf)}
                                <span class="text-red-500 font-bold text-[11px] bg-red-50 px-2 py-0.5 rounded-full">Sedang Digunakan</span>
                                <button 
                                    type="button" 
                                    onclick={() => releaseResource(stf)}
                                    class="bg-green-600 hover:bg-green-700 text-white font-bold text-[10px] px-2.5 py-1 rounded-lg transition cursor-pointer h-9">
                                    Selesai
                                </button>
                            {:else if busyResources.includes(stf)}
                                <span class="text-amber-500 font-bold text-[11px] bg-amber-50 px-2 py-0.5 rounded-full">Antrean (Hold)</span>
                            {:else}
                                <button 
                                    type="button" 
                                    onclick={() => { tableNumber = stf; showResourceModal = false; }}
                                    class="bg-blue-600 hover:bg-blue-700 text-white font-bold text-xs px-3.5 py-1.5 rounded-lg transition cursor-pointer h-9">
                                    Pilih
                                </button>
                            {/if}
                        </div>
                    </div>
                {/each}
            {/if}
        </div>

        <!-- Create new resource section -->
        <div class="border-t border-slate-100 pt-4 space-y-3 shrink-0">
            <div class="text-xs font-bold text-slate-700 uppercase tracking-wider">
                {businessMode === 'fb' ? 'Tambah Meja Baru' : 'Tambah Teknisi Baru'}
            </div>
            <div class="flex gap-2">
                <input 
                    type="text" 
                    bind:value={newResourceName} 
                    placeholder={businessMode === 'fb' ? 'Nama/Nomor meja...' : 'Nama teknisi...'} 
                    class="flex-1 p-2.5 border border-slate-200 rounded-lg text-xs outline-none h-11" 
                    onkeydown={(e) => { if (e.key === 'Enter') quickAddResource(); }}
                />
                <button class="bg-green-600 hover:bg-green-700 text-white font-bold px-4 py-2 rounded-lg text-xs transition cursor-pointer h-11" onclick={quickAddResource} disabled={!newResourceName.trim()}>Tambah</button>
            </div>
        </div>

        <div class="flex gap-2 mt-4 shrink-0">
            {#if tableNumber}
                <button class="bg-red-50 border border-red-100 text-red-600 hover:bg-red-100 font-bold py-2 rounded-xl text-xs transition cursor-pointer w-1/2 h-11" onclick={() => { tableNumber = ""; showResourceModal = false; }}>Kosongkan</button>
            {/if}
            <button class="bg-white border border-slate-200 text-slate-600 hover:bg-slate-50 font-bold py-2 rounded-xl text-xs transition cursor-pointer {tableNumber ? 'w-1/2' : 'w-full'} h-11" onclick={() => showResourceModal = false}>Tutup</button>
        </div>
    </div>
</div>
{/if}

<!-- 4. SAVE HOLD MODAL -->
{#if showHoldModal}
<div class="modal-backdrop fixed inset-0 bg-slate-900/60 backdrop-blur-xs flex items-center justify-center z-50 p-4">
    <div class="bg-white rounded-2xl max-w-sm w-full p-6 shadow-2xl border border-slate-100">
        <h2 class="text-lg font-black text-slate-800 mb-1">
            {businessMode === 'fb' ? 'Simpan Pesanan' : businessMode === 'jasa' ? 'Tunda Layanan' : 'Simpan Antrean'}
        </h2>
        <p class="text-xs text-slate-400 mb-4">
            {businessMode === 'fb' 
                ? 'Simpan keranjang saat ini sebagai pesanan terbuka (open order) untuk meja ini.' 
                : businessMode === 'jasa' 
                ? 'Simpan keranjang saat ini sebagai antrean layanan terapis yang sedang berjalan.' 
                : 'Simpan keranjang saat ini sebagai transaksi antrean tertunda.'}
        </p>
        
        <input 
            type="text" 
            bind:value={holdBillName} 
            placeholder={businessMode === 'fb' 
                ? 'Masukkan nama pesanan (cth: Meja 4 / Rusdin)...' 
                : businessMode === 'jasa' 
                ? 'Masukkan nama layanan (cth: Pijat - Budi)...' 
                : 'Masukkan nama pelanggan / antrean (cth: Ani / Antrean 3)...'} 
            class="w-full p-3 border border-slate-200 rounded-xl text-sm outline-none mb-6 focus:border-blue-500 h-12"
        />

        <div class="flex gap-3">
            <button class="bg-white border border-slate-200 text-slate-600 hover:bg-slate-50 font-bold py-2.5 px-6 rounded-xl transition w-1/2 text-xs cursor-pointer h-12 min-w-[48px]" onclick={() => showHoldModal = false}>Batal</button>
            <button class="bg-blue-600 hover:bg-blue-700 text-white font-bold py-2.5 px-6 rounded-xl transition w-1/2 text-xs cursor-pointer h-12 min-w-[48px]" onclick={holdBill}>Simpan</button>
        </div>
    </div>
</div>
{/if}

<!-- 5. SAVED BILLS LIST MODAL -->
{#if showSavedBillsModal}
<div class="modal-backdrop fixed inset-0 bg-slate-900/60 backdrop-blur-xs flex items-center justify-center z-50 p-4">
    <div class="bg-white rounded-2xl max-w-md w-full p-6 shadow-2xl border border-slate-100 flex flex-col max-h-[80vh]">
        <h2 class="text-lg font-black text-slate-800 mb-4">Daftar Tagihan Tersimpan</h2>
        
        <div class="flex-1 overflow-y-auto space-y-3 pr-1">
            {#each savedBills as bill}
                <div class="p-3 bg-slate-50 border border-slate-200 rounded-xl flex justify-between items-center">
                    <div>
                        <div class="font-bold text-slate-800 text-sm">{bill.name || 'Tanpa Nama'}</div>
                        <div class="text-[10px] text-slate-400 mt-0.5">Disimpan: {new Date(bill.updated_at).toLocaleString('id-ID')}</div>
                    </div>
                    <div class="flex items-center gap-2">
                        <button class="bg-blue-600 hover:bg-blue-700 text-white text-[10px] font-bold px-3 py-2 rounded-lg cursor-pointer h-9 min-w-[44px]" onclick={() => resumeBill(bill)}>Buka</button>
                        <button class="bg-red-50 hover:bg-red-100 text-red-600 text-[10px] font-bold px-3 py-2 rounded-lg cursor-pointer h-9 min-w-[44px]" onclick={() => deleteBill(bill.id)}>Hapus</button>
                    </div>
                </div>
            {/each}

            {#if savedBills.length === 0}
                <div class="text-center py-12 text-slate-400">
                    <span class="text-3xl block mb-2">📂</span>
                    <span class="text-xs font-bold">Tidak ada tagihan tertunda.</span>
                </div>
            {/if}
        </div>

        <button class="mt-6 bg-slate-200 hover:bg-slate-300 text-slate-800 font-bold py-2.5 rounded-xl transition text-xs cursor-pointer h-12 shrink-0" onclick={() => showSavedBillsModal = false}>Tutup</button>
    </div>
</div>
{/if}

<!-- 6. CART DISCOUNT MODAL -->
{#if showCartDiscountModal}
<div class="modal-backdrop fixed inset-0 bg-slate-900/60 backdrop-blur-xs flex items-center justify-center z-50 p-4">
    <div class="bg-white rounded-2xl max-w-sm w-full p-6 shadow-2xl border border-slate-100">
        <h2 class="text-lg font-black text-slate-800 mb-4">Diskon Transaksi</h2>
        
        <div class="flex bg-slate-100 p-1 rounded-xl border border-slate-200 mb-4">
            <button 
                type="button" 
                class="w-1/2 py-2 text-xs font-bold rounded-lg transition min-h-[44px]"
                class:bg-white={cartDiscountType === 'nominal'}
                class:text-blue-600={cartDiscountType === 'nominal'}
                onclick={() => cartDiscountType = 'nominal'}>
                Nominal (Rupiah)
            </button>
            <button 
                type="button" 
                class="w-1/2 py-2 text-xs font-bold rounded-lg transition min-h-[44px]"
                class:bg-white={cartDiscountType === 'percent'}
                class:text-blue-600={cartDiscountType === 'percent'}
                onclick={() => cartDiscountType = 'percent'}>
                Persentase (%)
            </button>
        </div>

        <div class="mb-6">
            <label for="cart-discount-value" class="block text-xs font-bold text-slate-400 uppercase mb-1">Nilai Diskon</label>
            <input 
                id="cart-discount-value"
                type="number" 
                bind:value={cartDiscountValue} 
                class="w-full text-xl p-3 border border-slate-200 rounded-xl font-bold text-slate-800 outline-none focus:border-blue-500 h-12" 
                placeholder="0"
            />
        </div>

        <div class="flex gap-3">
            <button class="bg-white border border-slate-200 text-slate-600 hover:bg-slate-50 font-bold py-2.5 px-6 rounded-xl transition w-1/2 text-xs cursor-pointer h-12 min-w-[48px]" onclick={() => showCartDiscountModal = false}>Batal</button>
            <button class="bg-blue-600 hover:bg-blue-700 text-white font-bold py-2.5 px-6 rounded-xl transition w-1/2 text-xs cursor-pointer h-12 min-w-[48px]" onclick={() => { calculateCart(); showCartDiscountModal = false; }}>Terapkan</button>
        </div>
    </div>
</div>
{/if}

<!-- 7. ITEM DISCOUNT MODAL -->
{#if showItemDiscountModal}
<div class="modal-backdrop fixed inset-0 bg-slate-900/60 backdrop-blur-xs flex items-center justify-center z-50 p-4">
    <div class="bg-white rounded-2xl max-w-sm w-full p-6 shadow-2xl border border-slate-100">
        <h2 class="text-lg font-black text-slate-800 mb-4">Diskon Produk</h2>
        
        <div class="flex bg-slate-100 p-1 rounded-xl border border-slate-200 mb-4">
            <button 
                type="button" 
                class="w-1/2 py-2 text-xs font-bold rounded-lg transition min-h-[44px]"
                class:bg-white={itemDiscountType === 'nominal'}
                class:text-blue-600={itemDiscountType === 'nominal'}
                onclick={() => itemDiscountType = 'nominal'}>
                Nominal (Rupiah)
            </button>
            <button 
                type="button" 
                class="w-1/2 py-2 text-xs font-bold rounded-lg transition min-h-[44px]"
                class:bg-white={itemDiscountType === 'percent'}
                class:text-blue-600={itemDiscountType === 'percent'}
                onclick={() => itemDiscountType = 'percent'}>
                Persentase (%)
            </button>
        </div>

        <div class="mb-6">
            <label for="item-discount-value" class="block text-xs font-bold text-slate-400 uppercase mb-1">Nilai Diskon</label>
            <input 
                id="item-discount-value"
                type="number" 
                bind:value={itemDiscountValue} 
                class="w-full text-xl p-3 border border-slate-200 rounded-xl font-bold text-slate-800 outline-none focus:border-blue-500 h-12" 
                placeholder="0"
            />
        </div>

        <div class="flex gap-3">
            <button class="bg-white border border-slate-200 text-slate-600 hover:bg-slate-50 font-bold py-2.5 px-6 rounded-xl transition w-1/2 text-xs cursor-pointer h-12 min-w-[48px]" onclick={() => showItemDiscountModal = false}>Batal</button>
            <button class="bg-blue-600 hover:bg-blue-700 text-white font-bold py-2.5 px-6 rounded-xl transition w-1/2 text-xs cursor-pointer h-12 min-w-[48px]" onclick={applyItemDiscount}>Terapkan</button>
        </div>
    </div>
</div>
{/if}

<!-- 8. ITEM CUSTOMIZER MODAL -->
{#if showItemCustomizerModal}
<div class="modal-backdrop fixed inset-0 bg-slate-900/60 backdrop-blur-xs flex items-center justify-center z-50 p-4">
    <div class="bg-white rounded-2xl max-w-md w-full p-6 shadow-2xl border border-slate-100 flex flex-col max-h-[90vh]">
        <h2 class="text-lg font-black text-slate-800 mb-4 flex-none">Kustomisasi Menu</h2>
        
        <div class="space-y-4 mb-6 overflow-y-auto flex-1 pr-1">
            <!-- Size Variant Selection -->
            <div>
                <div class="flex justify-between items-center mb-2">
                    <span class="block text-xs font-bold text-slate-500">Pilihan Varian / Ukuran</span>
                    <button 
                        type="button" 
                        onclick={() => showAddVariantForm = !showAddVariantForm}
                        class="text-[10px] text-blue-600 hover:text-blue-700 font-bold flex items-center gap-0.5 cursor-pointer">
                        {showAddVariantForm ? 'Tutup' : '+ Tambah'}
                    </button>
                </div>
                
                {#if showAddVariantForm}
                <div class="bg-slate-50 p-3 rounded-xl mb-3 space-y-2 border border-slate-200">
                    <div class="grid grid-cols-2 gap-2">
                        <input 
                            type="text" 
                            bind:value={newVariantName} 
                            placeholder="Nama varian" 
                            class="p-2 border border-slate-200 rounded-lg text-xs outline-none bg-white focus:border-blue-500 h-10" 
                        />
                        <input 
                            type="number" 
                            bind:value={newVariantPrice} 
                            placeholder="Harga Tambahan" 
                            class="p-2 border border-slate-200 rounded-lg text-xs outline-none bg-white focus:border-blue-500 h-10" 
                        />
                    </div>
                    <button 
                        type="button" 
                        onclick={addCustomVariant}
                        class="w-full bg-blue-600 hover:bg-blue-700 text-white font-bold py-1.5 px-3 rounded-lg text-[10px] transition cursor-pointer h-9">
                        Simpan Varian
                    </button>
                </div>
                {/if}

                <div class="grid grid-cols-2 gap-2 max-h-36 overflow-y-auto pr-1">
                    {#each customVariants as v}
                        <div class="relative group">
                            <button 
                                type="button" 
                                onclick={() => customizeSize = v.name}
                                class="w-full p-2 border rounded-xl text-center text-xs font-bold cursor-pointer transition-all duration-200 flex flex-col justify-center items-center min-h-[52px]"
                                class:border-blue-500={customizeSize === v.name}
                                class:bg-blue-50={customizeSize === v.name}
                                class:text-blue-700={customizeSize === v.name}>
                                <span>{v.name}</span>
                                <span class="text-[10px] text-slate-400 font-normal">
                                    {v.price > 0 ? `+Rp ${v.price.toLocaleString()}` : 'Normal'}
                                </span>
                            </button>
                            {#if v.name !== 'Regular'}
                            <button 
                                type="button" 
                                onclick={() => deleteCustomVariant(v.name)}
                                class="absolute -top-1.5 -right-1.5 bg-red-100 hover:bg-red-200 text-red-600 rounded-full w-5 h-5 flex items-center justify-center text-[10px] border border-red-200 cursor-pointer opacity-0 group-hover:opacity-100 transition-opacity animate-in fade-in"
                                title="Hapus varian">
                                &times;
                            </button>
                            {/if}
                        </div>
                    {/each}
                </div>
            </div>

            <!-- Modifiers List Selection -->
            <div>
                <div class="flex justify-between items-center mb-2">
                    <span class="block text-xs font-bold text-slate-500">Tambahan (Add-ons)</span>
                    <button 
                        type="button" 
                        onclick={() => showAddAddonForm = !showAddAddonForm}
                        class="text-[10px] text-blue-600 hover:text-blue-700 font-bold flex items-center gap-0.5 cursor-pointer">
                        {showAddAddonForm ? 'Tutup' : '+ Tambah'}
                    </button>
                </div>

                {#if showAddAddonForm}
                <div class="bg-slate-50 p-3 rounded-xl mb-3 space-y-2 border border-slate-200">
                    <div class="grid grid-cols-2 gap-2">
                        <input 
                            type="text" 
                            bind:value={newAddonName} 
                            placeholder="Nama add-on" 
                            class="p-2 border border-slate-200 rounded-lg text-xs outline-none bg-white focus:border-blue-500 h-10" 
                        />
                        <input 
                            type="number" 
                            bind:value={newAddonPrice} 
                            placeholder="Harga Tambahan" 
                            class="p-2 border border-slate-200 rounded-lg text-xs outline-none bg-white focus:border-blue-500 h-10" 
                        />
                    </div>
                    <button 
                        type="button" 
                        onclick={addCustomAddon}
                        class="w-full bg-blue-600 hover:bg-blue-700 text-white font-bold py-1.5 px-3 rounded-lg text-[10px] transition cursor-pointer h-9">
                        Simpan Add-on
                    </button>
                </div>
                {/if}

                <div class="space-y-2 max-h-48 overflow-y-auto pr-1">
                    {#each customAddons as a}
                        <div class="flex items-center justify-between p-2 border rounded-xl hover:bg-slate-50 transition group min-h-[44px]">
                            <label class="flex items-center gap-2 cursor-pointer flex-1 select-none">
                                <input 
                                    type="checkbox" 
                                    checked={customizeModifiers.includes(a.name)} 
                                    onclick={(e) => {
                                        if (e.currentTarget.checked) customizeModifiers = [...customizeModifiers, a.name];
                                        else customizeModifiers = customizeModifiers.filter(m => m !== a.name);
                                    }}
                                    class="rounded text-blue-600 focus:ring-blue-500 w-4 h-4 border-slate-300 cursor-pointer"
                                />
                                <span class="text-xs font-bold text-slate-700">{a.name}</span>
                                <span class="text-[10px] text-slate-400 font-normal">
                                    (+Rp {a.price.toLocaleString()})
                                </span>
                            </label>
                            <button 
                                type="button" 
                                onclick={() => deleteCustomAddon(a.name)}
                                class="bg-slate-100 hover:bg-red-50 text-slate-400 hover:text-red-600 rounded-lg p-1 text-[10px] cursor-pointer opacity-0 group-hover:opacity-100 transition-opacity"
                                title="Hapus add-on">
                                <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                                </svg>
                            </button>
                        </div>
                    {/each}
                </div>
            </div>

            <!-- Notes Field -->
            <div>
                <label for="customize-notes" class="block text-xs font-bold text-slate-500 mb-1">Catatan Pesanan</label>
                <input 
                    id="customize-notes"
                    type="text" 
                    bind:value={customizeNotes} 
                    class="w-full p-3 border border-slate-200 rounded-xl text-sm outline-none focus:border-blue-500 h-12" 
                    placeholder="Contoh: es sedikit, tidak terlalu manis" 
                />
            </div>
        </div>

        <div class="flex gap-3 flex-none">
            <button class="bg-white border border-slate-200 text-slate-600 hover:bg-slate-50 font-bold py-2.5 px-6 rounded-xl transition w-1/2 text-xs cursor-pointer h-12 min-w-[48px]" onclick={() => showItemCustomizerModal = false}>Batal</button>
            <button class="bg-blue-600 hover:bg-blue-700 text-white font-bold py-2.5 px-6 rounded-xl transition w-1/2 text-xs cursor-pointer h-12 min-w-[48px]" onclick={saveItemCustomization}>Simpan</button>
        </div>
    </div>
</div>
{/if}

<!-- 9. PAYMENT MODAL WITH MULTI-METHOD / SPLIT PAYMENT & DOUBLE SUBMIT DEFENSE -->
<PaymentModal 
    show={showPaymentModal}
    {grandTotal}
    {ecrEnabled}
    {ecrPortName}
    {ecrBaudRate}
    {ecrWaiting}
    {ecrStatusMessage}
    onSubmitPayment={submitPaymentProcess}
    onSendToEcr={sendToEcr}
    onCancelEcr={() => { ecrWaiting = false; ecrStatusMessage = ''; }}
    onClose={() => showPaymentModal = false}
/>

<!-- 10. TRANSACTION SUCCESS MODAL -->
<TransactionSuccessModal 
    show={showSuccessModal}
    orderId={lastSuccessOrderId}
    grandTotal={lastSuccessGrandTotal}
    paidTotal={lastSuccessPaidTotal}
    changeTotal={lastSuccessChangeTotal}
    paymentMethod={lastSuccessPaymentMethod}
    onPrintReceipt={handlePrintReceipt}
    onNewTransaction={handleStartNewTransaction}
/>

<!-- Barcode scan overlay toast -->
{#if barcodeToastVisible}
<div class="fixed bottom-20 left-1/2 -translate-x-1/2 bg-slate-900/90 text-white px-5 py-3 rounded-2xl shadow-xl flex items-center gap-3 z-50 transition-all border border-slate-700/50 backdrop-blur-xs">
    <span class="text-base">🏷️</span>
    <div class="text-xs font-bold">{barcodeToastMessage}</div>
</div>
{/if}

<style>
    .pos-container {
        height: calc(100vh - 4rem);
        overflow: hidden;
    }
</style>
