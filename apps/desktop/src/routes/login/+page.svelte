<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { login, authState } from "$lib/auth.svelte";
  import { showToast } from "$lib/toast.svelte";

  let pin = $state("");
  let errorMsg = $state("");
  let isSubmitting = $state(false);
  let shake = $state(false);

  // Helper demo PINs to make testing/reviewing super easy for the user
  const demoUsers = [
    { name: "Owner", pin: "123456", desc: "Akses Penuh" },
    { name: "Supervisor", pin: "1234", desc: "Refund & Void" },
    { name: "Cashier", pin: "1111", desc: "Transaksi Kasir" }
  ];

  function handleNumberPress(num: string) {
    if (isSubmitting) return;
    errorMsg = "";
    if (pin.length < 8) {
      pin += num;
    }
  }

  function handleBackspace() {
    if (isSubmitting) return;
    errorMsg = "";
    pin = pin.slice(0, -1);
  }

  function handleClear() {
    if (isSubmitting) return;
    errorMsg = "";
    pin = "";
  }

  async function submitPin(pinToSubmit: string) {
    if (isSubmitting) return;
    isSubmitting = true;
    errorMsg = "";
    try {
      await login(pinToSubmit);
      showToast(`Selamat datang kembali, ${authState.session?.user_name}!`, "success");
      goto("/");
    } catch (err: any) {
      console.error(err);
      errorMsg = err.toString() || "PIN salah atau akun terkunci";
      showToast(errorMsg, "error");
      
      // Trigger shake animation
      shake = true;
      setTimeout(() => {
        shake = false;
      }, 500);
      
      pin = ""; // Reset PIN
    } finally {
      isSubmitting = false;
    }
  }

  // Auto submit when PIN reaches appropriate length (like standard POS)
  $effect(() => {
    // Demo PINs are 4 digits or 6 digits
    if (pin.length === 4 || pin.length === 6) {
      // Small timeout to let the last dot light up before submitting
      const t = setTimeout(() => {
        // Double check length is still valid
        if (pin.length === 4 || pin.length === 6) {
          submitPin(pin);
        }
      }, 150);
      return () => clearTimeout(t);
    }
  });

  // Check keypress events for physical keyboard
  function handleKeyDown(event: KeyboardEvent) {
    if (event.key >= "0" && event.key <= "9") {
      handleNumberPress(event.key);
    } else if (event.key === "Backspace") {
      handleBackspace();
    } else if (event.key === "Escape" || event.key === "c" || event.key === "C") {
      handleClear();
    }
  }

  onMount(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  });
</script>

<div class="min-h-screen bg-slate-900 flex flex-col justify-center items-center p-4 relative overflow-hidden">
  <!-- Glowing Background Orbs -->
  <div class="absolute -top-40 -left-40 w-96 h-96 bg-blue-600/20 rounded-full blur-3xl"></div>
  <div class="absolute -bottom-40 -right-40 w-96 h-96 bg-indigo-600/20 rounded-full blur-3xl"></div>

  <!-- Main Login Card -->
  <div 
    class="w-full max-w-md bg-slate-800/80 backdrop-blur-xl border border-slate-700/60 rounded-3xl p-8 shadow-2xl z-10 transition-all duration-300 {shake ? 'animate-shake' : ''}"
  >
    <div class="text-center mb-6">
      <div class="inline-flex h-14 w-14 rounded-2xl bg-blue-600 items-center justify-center text-white font-black text-2xl shadow-lg shadow-blue-500/20 mb-4">
        P
      </div>
      <h2 class="text-2xl font-extrabold text-white tracking-tight">Otentikasi Kasir</h2>
      <p class="text-slate-400 text-sm mt-1">Masukkan PIN keamanan Anda untuk masuk</p>
    </div>

    <!-- PIN Visual Indicators (Dots) -->
    <div class="flex justify-center gap-3.5 my-8">
      {#each Array(6) as _, i}
        <div 
          class="w-4.5 h-4.5 rounded-full border-2 transition-all duration-200 
            {i < pin.length 
              ? 'bg-blue-500 border-blue-500 scale-110 shadow-sm shadow-blue-500/50' 
              : 'bg-transparent border-slate-600'}"
        ></div>
      {/each}
    </div>

    {#if errorMsg}
      <div class="bg-rose-500/10 border border-rose-500/30 text-rose-300 text-xs rounded-xl p-3 text-center mb-6 animate-pulse">
        {errorMsg}
      </div>
    {/if}

    <!-- PIN Pad Grid -->
    <div class="grid grid-cols-3 gap-4 mb-6">
      {#each ["1", "2", "3", "4", "5", "6", "7", "8", "9"] as num}
        <button
          type="button"
          onclick={() => handleNumberPress(num)}
          disabled={isSubmitting}
          class="h-16 rounded-2xl bg-slate-700/50 hover:bg-slate-700 active:scale-95 text-white font-bold text-xl border border-slate-700/40 transition-all flex items-center justify-center cursor-pointer select-none"
        >
          {num}
        </button>
      {/each}

      <!-- Clear (C) -->
      <button
        type="button"
        onclick={handleClear}
        disabled={isSubmitting}
        class="h-16 rounded-2xl bg-slate-800 hover:bg-slate-700/30 active:scale-95 text-slate-400 hover:text-white font-bold text-lg border border-transparent transition-all flex items-center justify-center cursor-pointer select-none"
      >
        Clear
      </button>

      <!-- 0 -->
      <button
        type="button"
        onclick={() => handleNumberPress("0")}
        disabled={isSubmitting}
        class="h-16 rounded-2xl bg-slate-700/50 hover:bg-slate-700 active:scale-95 text-white font-bold text-xl border border-slate-700/40 transition-all flex items-center justify-center cursor-pointer select-none"
      >
        0
      </button>

      <!-- Backspace (⌫) -->
      <button
        type="button"
        onclick={handleBackspace}
        disabled={isSubmitting}
        class="h-16 rounded-2xl bg-slate-800 hover:bg-slate-700/30 active:scale-95 text-slate-400 hover:text-white font-bold text-lg border border-transparent transition-all flex items-center justify-center cursor-pointer select-none"
        aria-label="Hapus"
      >
        ⌫
      </button>
    </div>

    <!-- Demo Helper Helper Panel -->
    <div class="border-t border-slate-700/50 pt-6 mt-2">
      <p class="text-xs font-semibold text-slate-500 mb-3 text-center uppercase tracking-wider">Demo / Testing PINs</p>
      <div class="grid grid-cols-3 gap-2">
        {#each demoUsers as user}
          <button
            type="button"
            onclick={() => {
              pin = user.pin;
            }}
            class="flex flex-col items-center justify-center p-2 rounded-xl bg-blue-600/5 hover:bg-blue-600/10 border border-blue-500/10 hover:border-blue-500/30 text-left transition-all active:scale-95 cursor-pointer"
          >
            <span class="text-xs font-bold text-blue-400">{user.name}</span>
            <span class="text-[10px] text-slate-500 font-mono mt-0.5">{user.pin}</span>
          </button>
        {/each}
      </div>
    </div>
  </div>
</div>

<style>
  /* Shake animation for errors */
  @keyframes shake {
    0%, 100% { transform: translateX(0); }
    10%, 30%, 50%, 70%, 90% { transform: translateX(-6px); }
    20%, 40%, 60%, 80% { transform: translateX(6px); }
  }
  
  .animate-shake {
    animation: shake 0.5s ease-in-out;
  }
</style>
