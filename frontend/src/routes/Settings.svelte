<script>
  import { fade } from 'svelte/transition';
  import { authToken, currentView, connected } from '../lib/stores.js';
  import { disconnectWs } from '../lib/ws.js';

  let voiceEnabled = localStorage.getItem('agentboard_voice') !== 'false';
  let darkMode = localStorage.getItem('agentboard_theme') !== 'light';

  function toggleVoice() {
    voiceEnabled = !voiceEnabled;
    localStorage.setItem('agentboard_voice', voiceEnabled ? 'true' : 'false');
  }

  function toggleTheme() {
    darkMode = !darkMode;
    localStorage.setItem('agentboard_theme', darkMode ? 'dark' : 'light');
    // Theme toggle is a stub — full implementation would swap CSS vars
  }

  function disconnect() {
    const confirmed = confirm('Disconnect and re-pair? Your session token will be cleared.');
    if (!confirmed) return;
    disconnectWs();
    authToken.set(null);
    currentView.set('pair');
  }

  $: backendUrl = typeof location !== 'undefined'
    ? `${location.protocol}//${location.hostname}:8000`
    : 'http://localhost:8000';
</script>

<div transition:fade={{ duration: 200 }} class="flex flex-col min-h-screen max-w-2xl mx-auto px-4">

  <!-- Header -->
  <header class="flex items-center gap-4 py-5">
    <button
      on:click={() => currentView.set('home')}
      aria-label="Back to home"
      class="h-10 w-10 flex items-center justify-center rounded-xl bg-white/5 hover:bg-white/10
             border border-white/5 text-white/40 hover:text-white/70 transition-colors"
    >
      <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M15 19l-7-7 7-7" />
      </svg>
    </button>
    <h1 class="text-lg font-bold text-white">Settings</h1>
  </header>

  <main class="flex-1 space-y-4">

    <!-- Connection -->
    <section aria-label="Connection">
      <h2 class="text-xs font-semibold text-white/40 uppercase tracking-wider mb-2">Connection</h2>
      <div class="bg-white/5 border border-white/10 rounded-2xl overflow-hidden">

        <div class="flex items-center justify-between px-4 py-3.5 border-b border-white/5">
          <div>
            <p class="text-sm font-medium text-white">Backend</p>
            <p class="text-xs text-white/40 font-mono mt-0.5">{backendUrl}</p>
          </div>
          <span class="text-xs text-white/30 bg-white/5 px-2 py-1 rounded-lg">read-only</span>
        </div>

        <div class="flex items-center justify-between px-4 py-3.5">
          <div>
            <p class="text-sm font-medium text-white">Connection status</p>
            <p class="text-xs mt-0.5 {$connected ? 'text-green-400' : 'text-red-400'}">
              {$connected ? 'Connected' : 'Disconnected'}
            </p>
          </div>
          <span class="w-3 h-3 rounded-full flex-shrink-0 {$connected ? 'bg-green-400 animate-pulse' : 'bg-red-400'}"></span>
        </div>
      </div>
    </section>

    <!-- Preferences -->
    <section aria-label="Preferences">
      <h2 class="text-xs font-semibold text-white/40 uppercase tracking-wider mb-2">Preferences</h2>
      <div class="bg-white/5 border border-white/10 rounded-2xl overflow-hidden">

        <div class="flex items-center justify-between px-4 py-4 border-b border-white/5">
          <div>
            <p class="text-sm font-medium text-white">Voice input</p>
            <p class="text-xs text-white/40 mt-0.5">Use microphone for task input</p>
          </div>
          <button
            role="switch"
            aria-checked={voiceEnabled}
            aria-label="Toggle voice input"
            on:click={toggleVoice}
            class="relative w-11 h-6 rounded-full transition-colors duration-200
                   {voiceEnabled ? 'bg-blue-600' : 'bg-white/10'}"
          >
            <span class="absolute top-0.5 left-0.5 w-5 h-5 rounded-full bg-white shadow transition-transform duration-200
                         {voiceEnabled ? 'translate-x-5' : 'translate-x-0'}"></span>
          </button>
        </div>

        <div class="flex items-center justify-between px-4 py-4">
          <div>
            <p class="text-sm font-medium text-white">Dark mode</p>
            <p class="text-xs text-white/40 mt-0.5">Use dark color scheme</p>
          </div>
          <button
            role="switch"
            aria-checked={darkMode}
            aria-label="Toggle dark mode"
            on:click={toggleTheme}
            class="relative w-11 h-6 rounded-full transition-colors duration-200
                   {darkMode ? 'bg-blue-600' : 'bg-white/10'}"
          >
            <span class="absolute top-0.5 left-0.5 w-5 h-5 rounded-full bg-white shadow transition-transform duration-200
                         {darkMode ? 'translate-x-5' : 'translate-x-0'}"></span>
          </button>
        </div>
      </div>
    </section>

    <!-- About -->
    <section aria-label="About">
      <h2 class="text-xs font-semibold text-white/40 uppercase tracking-wider mb-2">About</h2>
      <div class="bg-white/5 border border-white/10 rounded-2xl px-4 py-4">
        <div class="flex items-center justify-between">
          <p class="text-sm text-white">Agentboard</p>
          <p class="text-xs text-white/30">v0.1.0</p>
        </div>
        <p class="text-xs text-white/30 mt-1">Orchestrate AI agent teams from your phone</p>
      </div>
    </section>

    <!-- Danger zone -->
    <section aria-label="Account" class="pt-2 pb-8">
      <button
        on:click={disconnect}
        aria-label="Disconnect and re-pair"
        class="w-full h-12 rounded-xl font-semibold text-sm transition-all
               bg-red-500/10 hover:bg-red-500/20 border border-red-500/15
               text-red-400 hover:text-red-300"
      >
        Disconnect &amp; Re-pair
      </button>
    </section>
  </main>
</div>
