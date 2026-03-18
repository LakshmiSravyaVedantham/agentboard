<script>
  import { fade, slide } from 'svelte/transition';
  import { teamsList, currentView, connected } from '../lib/stores.js';
  import { api } from '../lib/api.js';
  import TeamCard from '../components/TeamCard.svelte';
  import VoiceButton from '../components/VoiceButton.svelte';

  let input = '';
  let loading = false;
  let error = '';

  async function send() {
    const text = input.trim();
    if (!text || loading) return;
    loading = true;
    error = '';
    try {
      await api.submitTasks(text);
      input = '';
    } catch (e) {
      error = e.message || 'Failed to submit tasks';
    } finally {
      loading = false;
    }
  }

  function onKeydown(e) {
    if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) send();
  }

  function onVoiceResult(e) {
    input = (input ? input + ' ' : '') + e.detail;
  }
</script>

<div transition:fade={{ duration: 200 }} class="flex flex-col min-h-screen max-w-2xl mx-auto px-4">

  <!-- Header -->
  <header class="flex items-center justify-between py-5">
    <div class="flex items-center gap-2">
      <span class="text-lg font-bold text-white tracking-tight">agentboard</span>
      <span class="w-2 h-2 rounded-full {$connected ? 'bg-green-400 animate-pulse' : 'bg-red-400'}"
            aria-label={$connected ? 'Connected' : 'Disconnected'}></span>
    </div>
    <button
      on:click={() => currentView.set('settings')}
      aria-label="Settings"
      class="h-10 w-10 flex items-center justify-center rounded-xl bg-white/5 hover:bg-white/10
             border border-white/5 text-white/40 hover:text-white/70 transition-colors"
    >
      <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round"
          d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
        <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
      </svg>
    </button>
  </header>

  <!-- Task input area -->
  <section aria-label="Submit tasks" class="mb-6">
    <div class="bg-white/5 backdrop-blur-xl border border-white/10 rounded-2xl p-4">
      <textarea
        bind:value={input}
        on:keydown={onKeydown}
        placeholder="What do you want done? Describe tasks for your agent teams..."
        aria-label="Task description"
        rows="4"
        class="w-full bg-transparent text-white placeholder-white/30 resize-none
               focus:outline-none text-sm leading-relaxed"
      ></textarea>
      <div class="flex items-center justify-between mt-3 pt-3 border-t border-white/5">
        <div class="flex items-center gap-2">
          <VoiceButton on:result={onVoiceResult} />
          <span class="text-xs text-white/20">⌘↵ to send</span>
        </div>
        <button
          on:click={send}
          disabled={!input.trim() || loading}
          aria-label="Send tasks to agents"
          class="h-10 px-5 rounded-xl text-sm font-semibold transition-all duration-200
                 bg-gradient-to-r from-blue-500 to-purple-600
                 hover:from-blue-400 hover:to-purple-500
                 disabled:opacity-40 disabled:cursor-not-allowed text-white"
        >
          {#if loading}
            <span class="inline-flex items-center gap-2">
              <svg class="animate-spin w-4 h-4" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v8z"/>
              </svg>
              Sending...
            </span>
          {:else}
            Send
          {/if}
        </button>
      </div>
    </div>

    {#if error}
      <p transition:slide class="text-red-400 text-xs mt-2 px-1">{error}</p>
    {/if}
  </section>

  <!-- Teams list -->
  <main class="flex-1">
    {#if $teamsList.length === 0}
      <div class="text-center py-16">
        <div class="inline-flex items-center justify-center w-16 h-16 rounded-2xl
                    bg-white/5 border border-white/5 mb-4">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-7 h-7 text-white/20" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
            <path stroke-linecap="round" stroke-linejoin="round"
              d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0z" />
          </svg>
        </div>
        <p class="text-white/30 text-sm">No agent teams yet</p>
        <p class="text-white/20 text-xs mt-1">Submit a task above to get started</p>
      </div>
    {:else}
      <h2 class="text-xs font-semibold text-white/40 uppercase tracking-wider mb-3">Active Teams</h2>
      <div class="space-y-3">
        {#each $teamsList as team (team.id)}
          <div transition:slide>
            <TeamCard {team} />
          </div>
        {/each}
      </div>
    {/if}
  </main>

  <!-- Bottom nav -->
  <nav aria-label="Main navigation" class="flex items-center justify-around py-4 mt-4 border-t border-white/5">
    <button
      on:click={() => currentView.set('home')}
      aria-label="Home"
      class="flex flex-col items-center gap-1 text-blue-400 transition-colors"
    >
      <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
      </svg>
      <span class="text-xs font-medium">Home</span>
    </button>

    <button
      on:click={() => currentView.set('summary')}
      aria-label="Summary"
      class="flex flex-col items-center gap-1 text-white/40 hover:text-white/70 transition-colors"
    >
      <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
      </svg>
      <span class="text-xs font-medium">Summary</span>
    </button>
  </nav>
</div>
