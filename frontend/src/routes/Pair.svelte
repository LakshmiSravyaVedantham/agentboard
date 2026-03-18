<script>
  import { fade } from 'svelte/transition';
  import { authToken, currentView } from '../lib/stores.js';
  import { api } from '../lib/api.js';
  import { connectWs } from '../lib/ws.js';

  let code = '';
  let loading = false;
  let error = '';

  // Split the code into 6 individual character inputs
  let digits = ['', '', '', '', '', ''];
  let inputs = [];

  function onDigitInput(i, e) {
    const val = e.target.value.replace(/\D/g, '').slice(-1);
    digits[i] = val;
    digits = [...digits];
    code = digits.join('');

    if (val && i < 5) {
      inputs[i + 1]?.focus();
    }
    if (digits.every(d => d !== '')) {
      submit();
    }
  }

  function onDigitKeydown(i, e) {
    if (e.key === 'Backspace' && !digits[i] && i > 0) {
      inputs[i - 1]?.focus();
    }
  }

  function onPaste(e) {
    const pasted = e.clipboardData.getData('text').replace(/\D/g, '').slice(0, 6);
    if (pasted.length) {
      for (let i = 0; i < 6; i++) digits[i] = pasted[i] || '';
      digits = [...digits];
      code = digits.join('');
      e.preventDefault();
      if (code.length === 6) submit();
    }
  }

  async function submit() {
    if (loading) return;
    const c = code.trim();
    if (c.length < 6) { error = 'Enter the 6-digit pairing code'; return; }
    loading = true;
    error = '';
    try {
      const data = await api.pair(c);
      if (data.token) {
        authToken.set(data.token);
        connectWs();
        currentView.set('home');
      } else {
        error = data.error || 'Pairing failed — check the code and try again';
        digits = ['', '', '', '', '', ''];
        code = '';
        inputs[0]?.focus();
      }
    } catch (e) {
      error = 'Could not reach the server. Is agentboard running?';
      digits = ['', '', '', '', '', ''];
      code = '';
      inputs[0]?.focus();
    } finally {
      loading = false;
    }
  }
</script>

<main
  transition:fade={{ duration: 200 }}
  class="min-h-screen flex items-center justify-center px-4"
>
  <div class="w-full max-w-sm">
    <!-- Logo / wordmark -->
    <div class="text-center mb-10">
      <div class="inline-flex items-center justify-center w-16 h-16 rounded-2xl
                  bg-gradient-to-br from-blue-500/20 to-purple-600/20
                  border border-white/10 mb-4">
        <!-- agent icon -->
        <svg xmlns="http://www.w3.org/2000/svg" class="w-8 h-8 text-blue-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
          <path stroke-linecap="round" stroke-linejoin="round"
            d="M9 3H5a2 2 0 00-2 2v4m6-6h10a2 2 0 012 2v4M9 3v18m0 0h10a2 2 0 002-2V9M9 21H5a2 2 0 01-2-2V9m0 0h18" />
        </svg>
      </div>
      <h1 class="text-2xl font-bold text-white tracking-tight">agentboard</h1>
      <p class="text-white/40 text-sm mt-1">Enter your pairing code to connect</p>
    </div>

    <!-- Glassmorphism card -->
    <div class="bg-white/5 backdrop-blur-xl border border-white/10 rounded-2xl p-8">
      <p class="block text-xs font-semibold text-white/40 uppercase tracking-wider mb-4 text-center">
        Pairing Code
      </p>

      <!-- 6-digit inputs -->
      <div class="flex gap-3 justify-center mb-6" on:paste={onPaste}>
        {#each digits as digit, i}
          <input
            bind:this={inputs[i]}
            type="text"
            inputmode="numeric"
            pattern="[0-9]*"
            maxlength="1"
            value={digit}
            on:input={(e) => onDigitInput(i, e)}
            on:keydown={(e) => onDigitKeydown(i, e)}
            aria-label="Digit {i + 1} of 6"
            class="w-12 h-14 text-center text-xl font-mono font-bold text-white
                   bg-white/10 border border-white/10 rounded-xl
                   focus:outline-none focus:border-blue-500/60 focus:bg-white/15
                   transition-all duration-150 select-none"
          />
        {/each}
      </div>

      {#if error}
        <p transition:fade={{ duration: 150 }} class="text-red-400 text-sm text-center mb-4">{error}</p>
      {/if}

      <button
        on:click={submit}
        disabled={loading || code.length < 6}
        aria-label="Connect to agentboard"
        class="w-full h-12 rounded-xl font-semibold text-sm transition-all duration-200
               bg-gradient-to-r from-blue-500 to-purple-600
               hover:from-blue-400 hover:to-purple-500
               disabled:opacity-40 disabled:cursor-not-allowed
               text-white shadow-lg shadow-blue-500/20"
      >
        {#if loading}
          <span class="inline-flex items-center gap-2">
            <svg class="animate-spin w-4 h-4" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v8z"/>
            </svg>
            Connecting...
          </span>
        {:else}
          Connect
        {/if}
      </button>
    </div>

    <p class="text-center text-white/20 text-xs mt-6">
      Run <code class="font-mono text-white/30 bg-white/5 px-1.5 py-0.5 rounded">agentboard pair</code> to get a code
    </p>
  </div>
</main>
