<script>
  import { fade, slide } from 'svelte/transition';
  import { teams, selectedTeamId, currentView } from '../lib/stores.js';
  import { api } from '../lib/api.js';
  import StatusBadge from '../components/StatusBadge.svelte';
  import TerminalOutput from '../components/TerminalOutput.svelte';

  let team = null;
  let message = '';
  let sendingMsg = false;
  let killing = false;
  let error = '';

  // Reactive: keep team in sync with store
  $: team = $selectedTeamId ? $teams[$selectedTeamId] || null : null;

  function back() {
    currentView.set('home');
  }

  async function sendMessage() {
    const text = message.trim();
    if (!text || sendingMsg || !team) return;
    sendingMsg = true;
    error = '';
    try {
      await api.sendMessage(team.id, text);
      message = '';
    } catch (e) {
      error = e.message || 'Failed to send message';
    } finally {
      sendingMsg = false;
    }
  }

  async function killTeam() {
    if (!team || killing) return;
    const confirmed = confirm(`Kill team "${team.name || team.id}"? This cannot be undone.`);
    if (!confirmed) return;
    killing = true;
    try {
      await api.killTeam(team.id);
    } catch (e) {
      error = e.message || 'Failed to kill team';
    } finally {
      killing = false;
    }
  }

  function onKeydown(e) {
    if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) sendMessage();
  }
</script>

<div transition:fade={{ duration: 200 }} class="flex flex-col min-h-screen max-w-2xl mx-auto px-4">

  <!-- Header -->
  <header class="flex items-center gap-3 py-5">
    <button
      on:click={back}
      aria-label="Back to home"
      class="h-10 w-10 flex items-center justify-center rounded-xl bg-white/5 hover:bg-white/10
             border border-white/5 text-white/40 hover:text-white/70 transition-colors flex-shrink-0"
    >
      <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M15 19l-7-7 7-7" />
      </svg>
    </button>
    <div class="flex-1 min-w-0">
      <h1 class="text-lg font-bold text-white truncate">{team?.name || `Team ${team?.id || ''}`}</h1>
    </div>
    {#if team}
      <StatusBadge status={team.status || 'Queued'} />
    {/if}
  </header>

  {#if !team}
    <div class="flex-1 flex items-center justify-center">
      <p class="text-white/30 text-sm">Team not found</p>
    </div>
  {:else}
    <main class="flex-1 space-y-4">

      <!-- Task info -->
      {#if team.task}
        <section aria-label="Task">
          <div class="bg-white/5 border border-white/10 rounded-xl p-4">
            <p class="text-xs text-white/40 uppercase tracking-wider font-semibold mb-1.5">Task</p>
            <p class="text-sm text-white/80 leading-relaxed">{team.task}</p>
            {#if team.working_dir}
              <p class="text-xs font-mono text-white/30 mt-2">{team.working_dir}</p>
            {/if}
          </div>
        </section>
      {/if}

      <!-- Summary (when done) -->
      {#if team.status === 'Done' && team.summary}
        <section aria-label="Summary" transition:slide>
          <div class="bg-green-500/10 border border-green-500/20 rounded-xl p-4">
            <p class="text-xs text-green-400 uppercase tracking-wider font-semibold mb-1.5">Summary</p>
            <p class="text-sm text-white/80 leading-relaxed">{team.summary}</p>
          </div>
        </section>
      {/if}

      <!-- Error (when failed) -->
      {#if team.status === 'Failed' && team.error}
        <section aria-label="Error" transition:slide>
          <div class="bg-red-500/10 border border-red-500/20 rounded-xl p-4">
            <p class="text-xs text-red-400 uppercase tracking-wider font-semibold mb-1.5">Error</p>
            <p class="text-sm text-white/80 font-mono leading-relaxed">{team.error}</p>
          </div>
        </section>
      {/if}

      <!-- Terminal output -->
      <section aria-label="Terminal output">
        <p class="text-xs text-white/40 uppercase tracking-wider font-semibold mb-2">Output</p>
        <TerminalOutput lines={team.output || []} maxHeight="360px" />
      </section>

      <!-- Follow-up message -->
      {#if team.status === 'Running'}
        <section aria-label="Send follow-up message" transition:slide>
          <div class="bg-white/5 border border-white/10 rounded-xl p-4">
            <p class="text-xs text-white/40 uppercase tracking-wider font-semibold mb-2">Send Message</p>
            <textarea
              bind:value={message}
              on:keydown={onKeydown}
              placeholder="Send a follow-up instruction..."
              rows="3"
              class="w-full bg-white/10 border border-white/10 rounded-xl px-3 py-2.5 text-sm text-white
                     placeholder-white/30 resize-none focus:outline-none focus:border-blue-500/50"
            ></textarea>
            <div class="flex justify-between items-center mt-2">
              <span class="text-xs text-white/20">⌘↵ to send</span>
              <button
                on:click={sendMessage}
                disabled={!message.trim() || sendingMsg}
                class="h-9 px-4 rounded-lg text-sm font-semibold transition-all
                       bg-gradient-to-r from-blue-500 to-purple-600
                       hover:from-blue-400 hover:to-purple-500
                       disabled:opacity-40 disabled:cursor-not-allowed text-white"
              >
                {sendingMsg ? 'Sending...' : 'Send'}
              </button>
            </div>
          </div>
        </section>
      {/if}

      {#if error}
        <p transition:slide class="text-red-400 text-xs">{error}</p>
      {/if}
    </main>

    <!-- Kill button -->
    {#if team.status === 'Running' || team.status === 'Queued'}
      <div class="py-6" transition:slide>
        <button
          on:click={killTeam}
          disabled={killing}
          aria-label="Kill this team"
          class="w-full h-12 rounded-xl font-semibold text-sm transition-all
                 bg-red-500/15 hover:bg-red-500/25 border border-red-500/20
                 text-red-400 hover:text-red-300 disabled:opacity-40"
        >
          {killing ? 'Stopping...' : 'Kill Team'}
        </button>
      </div>
    {:else}
      <div class="py-6"></div>
    {/if}
  {/if}
</div>
