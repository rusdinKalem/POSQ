<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let healthStatus = $state("");
  let dbStatus = $state("");
  let orderResult = $state("");
  let ordersList = $state("");
  
  let checkingHealth = $state(false);
  let checkingDb = $state(false);
  let creatingOrder = $state(false);
  let listingOrders = $state(false);
  let seedStatus = $state("");
  let seedingDb = $state(false);

  async function checkHealth() {
    checkingHealth = true;
    try {
      const res = await invoke("health_check");
      healthStatus = JSON.stringify(res, null, 2);
    } catch (e: any) {
      healthStatus = "Error: " + e;
    } finally {
      checkingHealth = false;
    }
  }

  async function checkDb() {
    checkingDb = true;
    try {
      const res = await invoke("db_health_check");
      dbStatus = JSON.stringify(res, null, 2);
    } catch (e: any) {
      dbStatus = "Error: " + e;
    } finally {
      checkingDb = false;
    }
  }

  async function createOrder() {
    creatingOrder = true;
    try {
      const res = await invoke("create_dummy_order");
      orderResult = JSON.stringify(res, null, 2);
      listOrders(); // refresh list
    } catch (e: any) {
      orderResult = "Error: " + e;
    } finally {
      creatingOrder = false;
    }
  }

  async function listOrders() {
    listingOrders = true;
    try {
      const res = await invoke("list_orders");
      ordersList = JSON.stringify(res, null, 2);
    } catch (e: any) {
      ordersList = "Error: " + e;
    } finally {
      listingOrders = false;
    }
  }

  async function seedDb() {
    seedingDb = true;
    try {
      await invoke("seed_database");
      seedStatus = "Database successfully seeded with default tenant, admin, roles, permissions, and dummy products!";
    } catch (e: any) {
      seedStatus = "Error: " + e;
    } finally {
      seedingDb = false;
    }
  }
</script>

<div class="bg-slate-50 text-slate-800 font-sans flex-grow">
  <!-- Main Container -->
  <main class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-10">
    <div class="mb-8">
      <h1 class="text-3xl font-extrabold text-slate-900 tracking-tight">Dashboard Teknis POSQ</h1>
      <p class="text-slate-500 mt-1">Gunakan panel di bawah ini untuk memverifikasi fungsionalitas backend dan modul inti POSQ.</p>
    </div>

    <!-- Grid Layout -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
      
      <!-- App Health Check -->
      <div class="bg-white border border-slate-200/80 rounded-2xl p-6 shadow-xs hover:shadow-md transition-all flex flex-col justify-between">
        <div>
          <div class="flex items-center gap-3 mb-4">
            <span class="text-2xl">⚡</span>
            <h3 class="text-lg font-bold text-slate-800">App Health Check</h3>
          </div>
          <p class="text-slate-500 text-sm mb-4">Periksa apakah sistem dasar Tauri dan Node server berkomunikasi dengan benar.</p>
          <pre class="bg-slate-900 text-slate-100 p-4 rounded-xl text-xs font-mono overflow-auto h-36 max-h-36 mb-4">{healthStatus || "// Hasil health check akan tampil di sini..."}</pre>
        </div>
        <button 
          onclick={checkHealth} 
          disabled={checkingHealth}
          class="w-full bg-blue-600 hover:bg-blue-700 disabled:bg-blue-400 text-white font-bold py-3 px-4 rounded-xl shadow-md transition-all active:scale-98 cursor-pointer text-sm"
        >
          {checkingHealth ? "Memeriksa..." : "Jalankan Health Check"}
        </button>
      </div>

      <!-- DB Health Check -->
      <div class="bg-white border border-slate-200/80 rounded-2xl p-6 shadow-xs hover:shadow-md transition-all flex flex-col justify-between">
        <div>
          <div class="flex items-center gap-3 mb-4">
            <span class="text-2xl">🗄️</span>
            <h3 class="text-lg font-bold text-slate-800">DB Health Check</h3>
          </div>
          <p class="text-slate-500 text-sm mb-4">Periksa status koneksi PostgreSQL lokal yang terhubung ke aplikasi.</p>
          <pre class="bg-slate-900 text-slate-100 p-4 rounded-xl text-xs font-mono overflow-auto h-36 max-h-36 mb-4">{dbStatus || "// Hasil koneksi database akan tampil di sini..."}</pre>
        </div>
        <button 
          onclick={checkDb} 
          disabled={checkingDb}
          class="w-full bg-blue-600 hover:bg-blue-700 disabled:bg-blue-400 text-white font-bold py-3 px-4 rounded-xl shadow-md transition-all active:scale-98 cursor-pointer text-sm"
        >
          {checkingDb ? "Memeriksa..." : "Jalankan DB Check"}
        </button>
      </div>

      <!-- Create Dummy Order -->
      <div class="bg-white border border-slate-200/80 rounded-2xl p-6 shadow-xs hover:shadow-md transition-all flex flex-col justify-between">
        <div>
          <div class="flex items-center gap-3 mb-4">
            <span class="text-2xl">🛍️</span>
            <h3 class="text-lg font-bold text-slate-800">Create Dummy Order</h3>
          </div>
          <p class="text-slate-500 text-sm mb-4">Buat satu data pesanan dummy secara otomatis untuk mensimulasikan checkout transaksi.</p>
          <pre class="bg-slate-900 text-slate-100 p-4 rounded-xl text-xs font-mono overflow-auto h-36 max-h-36 mb-4">{orderResult || "// Respon pembuatan order akan tampil di sini..."}</pre>
        </div>
        <button 
          onclick={createOrder} 
          disabled={creatingOrder}
          class="w-full bg-blue-600 hover:bg-blue-700 disabled:bg-blue-400 text-white font-bold py-3 px-4 rounded-xl shadow-md transition-all active:scale-98 cursor-pointer text-sm"
        >
          {creatingOrder ? "Membuat..." : "Buat Order Baru"}
        </button>
      </div>

      <!-- List Orders -->
      <div class="bg-white border border-slate-200/80 rounded-2xl p-6 shadow-xs hover:shadow-md transition-all flex flex-col justify-between">
        <div>
          <div class="flex items-center gap-3 mb-4">
            <span class="text-2xl">📋</span>
            <h3 class="text-lg font-bold text-slate-800">List Orders</h3>
          </div>
          <p class="text-slate-500 text-sm mb-4">Ambil 5 transaksi terakhir dari database untuk memverifikasi riwayat pesanan.</p>
          <pre class="bg-slate-900 text-slate-100 p-4 rounded-xl text-xs font-mono overflow-auto h-36 max-h-36 mb-4">{ordersList || "// Daftar transaksi terakhir akan tampil di sini..."}</pre>
        </div>
        <button 
          onclick={listOrders} 
          disabled={listingOrders}
          class="w-full bg-blue-600 hover:bg-blue-700 disabled:bg-blue-400 text-white font-bold py-3 px-4 rounded-xl shadow-md transition-all active:scale-98 cursor-pointer text-sm"
        >
          {listingOrders ? "Memuat..." : "Tampilkan Riwayat Order"}
        </button>
      </div>

      <!-- Seed Database -->
      <div class="bg-white border border-slate-200/80 rounded-2xl p-6 shadow-xs hover:shadow-md transition-all flex flex-col justify-between">
        <div>
          <div class="flex items-center gap-3 mb-4">
            <span class="text-2xl">🌱</span>
            <h3 class="text-lg font-bold text-slate-800">Seed Database</h3>
          </div>
          <p class="text-slate-500 text-sm mb-4">Isi database lokal dengan data default (pemilik, toko demo, produk kopi, dan izin akses).</p>
          <pre class="bg-slate-900 text-slate-100 p-4 rounded-xl text-xs font-mono overflow-auto h-36 max-h-36 mb-4">{seedStatus || "// Hasil seeding database akan tampil di sini..."}</pre>
        </div>
        <button 
          onclick={seedDb} 
          disabled={seedingDb}
          class="w-full bg-blue-600 hover:bg-blue-700 disabled:bg-blue-400 text-white font-bold py-3 px-4 rounded-xl shadow-md transition-all active:scale-98 cursor-pointer text-sm"
        >
          {seedingDb ? "Seeding..." : "Jalankan Database Seeder"}
        </button>
      </div>

    </div>
  </main>
</div>
