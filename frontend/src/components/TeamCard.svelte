<script>
  import { currentView, selectedTeamId } from '../lib/stores.js';
  import StatusBadge from './StatusBadge.svelte';

  export let team;

  function formatAge(ts) {
    if (!ts) return '';
    const diff = Date.now() - new Date(ts).getTime();
    const mins = Math.floor(diff / 60000);
    if (mins < 1) return 'just now';
    if (mins < 60) return `${mins}m ago`;
    const hrs = Math.floor(mins / 60);
    if (hrs < 24) return `${hrs}h ago`;
    return `${Math.floor(hrs / 24)}d ago`;
  }

  function open() {
    selectedTeamId.set(team.id);
    currentView.set('team');
  }
</script>

<div
  role="button"
  tabindex="0"
  on:click={open}
  on:keydown={(e) => e.key === 'Enter' && open()}
  class="bg-white/5 backdrop-blur-xl border border-white/10 rounded-2xl p-4 cursor-pointer
         hover:bg-white/8 hover:border-white/20 transition-all duration-200 active:scale-[0.98]"
>
  <div class="flex items-start justify-between gap-3 mb-2">
    <div class="flex-1 min-w-0">
      <h3 class="text-sm font-semibold text-white truncate">{team.name || `Team ${team.id}`}</h3>
      {#if team.task}
        <p class="text-xs text-white/50 mt-0.5 line-clamp-2">{team.task}</p>
      {/if}
    </div>
    <StatusBadge status={team.status || 'Queued'} />
  </div>

  <div class="flex items-center justify-between mt-3">
    {#if team.working_dir}
      <span class="text-xs text-white/30 font-mono truncate max-w-[60%]">{team.working_dir}</span>
    {:else}
      <span></span>
    {/if}
    <span class="text-xs text-white/30">{formatAge(team.created_at)}</span>
  </div>

  {#if team.status === 'Running' && team.output?.length}
    <div class="mt-3 bg-black/30 rounded-lg p-2.5">
      <p class="text-xs font-mono text-green-300/80 truncate">{team.output[team.output.length - 1]}</p>
    </div>
  {/if}

  {#if team.status === 'Done' && team.summary}
    <div class="mt-3 pt-3 border-t border-white/5">
      <p class="text-xs text-white/50 line-clamp-2">{team.summary}</p>
    </div>
  {/if}
</div>
