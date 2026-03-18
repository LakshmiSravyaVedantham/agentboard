<script>
  import { fade } from 'svelte/transition';
  import { teamsList, completedCount, totalCount, currentView, selectedTeamId } from '../lib/stores.js';
  import StatusBadge from '../components/StatusBadge.svelte';

  $: progress = $totalCount > 0 ? ($completedCount / $totalCount) * 100 : 0;

  function openTeam(id) {
    selectedTeamId.set(id);
    currentView.set('team');
  }
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
    <h1 class="text-lg font-bold text-white">Summary Dashboard</h1>
  </header>

  <main class="flex-1 space-y-6">

    <!-- Progress card -->
    <section aria-label="Overall progress">
      <div class="bg-white/5 backdrop-blur-xl border border-white/10 rounded-2xl p-6">
        <div class="flex items-end justify-between mb-4">
          <div>
            <p class="text-4xl font-bold text-white">{$completedCount}</p>
            <p class="text-white/40 text-sm mt-0.5">of {$totalCount} team{$totalCount !== 1 ? 's' : ''} done</p>
          </div>
          <div class="text-right">
            <p class="text-2xl font-bold text-white">{Math.round(progress)}%</p>
            <p class="text-white/40 text-sm">complete</p>
          </div>
        </div>

        <!-- Progress bar -->
        <div class="h-2 bg-white/10 rounded-full overflow-hidden" role="progressbar"
             aria-valuenow={Math.round(progress)} aria-valuemin="0" aria-valuemax="100">
          <div
            class="h-full bg-gradient-to-r from-blue-500 to-purple-600 rounded-full transition-all duration-700"
            style="width: {progress}%"
          ></div>
        </div>

        <!-- Status breakdown -->
        {#if $totalCount > 0}
          <div class="grid grid-cols-3 gap-3 mt-5">
            {#each [
              { label: 'Running', status: 'Running', cls: 'text-blue-400' },
              { label: 'Done',    status: 'Done',    cls: 'text-green-400' },
              { label: 'Failed',  status: 'Failed',  cls: 'text-red-400' },
            ] as s}
              {@const count = $teamsList.filter(t => t.status === s.status).length}
              <div class="bg-white/5 rounded-xl p-3 text-center">
                <p class="text-xl font-bold {s.cls}">{count}</p>
                <p class="text-white/40 text-xs mt-0.5">{s.label}</p>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </section>

    <!-- All teams list -->
    {#if $teamsList.length === 0}
      <div class="text-center py-12">
        <p class="text-white/30 text-sm">No teams yet</p>
      </div>
    {:else}
      <section aria-label="All teams">
        <h2 class="text-xs font-semibold text-white/40 uppercase tracking-wider mb-3">All Teams</h2>
        <div class="space-y-2">
          {#each $teamsList as team (team.id)}
            <div
              role="button"
              tabindex="0"
              on:click={() => openTeam(team.id)}
              on:keydown={(e) => e.key === 'Enter' && openTeam(team.id)}
              class="bg-white/5 border border-white/5 rounded-xl p-4 cursor-pointer
                     hover:bg-white/8 hover:border-white/10 transition-all"
            >
              <div class="flex items-start justify-between gap-3">
                <div class="flex-1 min-w-0">
                  <h3 class="text-sm font-semibold text-white truncate">{team.name || `Team ${team.id}`}</h3>
                  {#if team.task}
                    <p class="text-xs text-white/40 mt-0.5 line-clamp-1">{team.task}</p>
                  {/if}
                  {#if team.status === 'Done' && team.summary}
                    <p class="text-xs text-white/50 mt-2 line-clamp-2">{team.summary}</p>
                  {/if}
                  {#if team.status === 'Failed' && team.error}
                    <p class="text-xs text-red-400/70 mt-2 line-clamp-1 font-mono">{team.error}</p>
                  {/if}
                </div>
                <StatusBadge status={team.status || 'Queued'} />
              </div>
            </div>
          {/each}
        </div>
      </section>
    {/if}
  </main>

  <!-- Bottom nav -->
  <nav aria-label="Main navigation" class="flex items-center justify-around py-4 mt-4 border-t border-white/5">
    <button
      on:click={() => currentView.set('home')}
      aria-label="Home"
      class="flex flex-col items-center gap-1 text-white/40 hover:text-white/70 transition-colors"
    >
      <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
      </svg>
      <span class="text-xs font-medium">Home</span>
    </button>
    <button
      on:click={() => currentView.set('summary')}
      aria-label="Summary"
      class="flex flex-col items-center gap-1 text-blue-400 transition-colors"
    >
      <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
      </svg>
      <span class="text-xs font-medium">Summary</span>
    </button>
  </nav>
</div>
