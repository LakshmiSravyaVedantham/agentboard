<script>
  import { fade, slide } from 'svelte/transition';
  import { pendingPlan, currentView } from '../lib/stores.js';
  import { api } from '../lib/api.js';
  import PlanCard from '../components/PlanCard.svelte';

  let plan = null;
  let localItems = [];
  let loading = false;
  let error = '';

  // Subscribe to pendingPlan and build local editable copy
  pendingPlan.subscribe(val => {
    plan = val;
    localItems = val?.teams ? val.teams.map(t => ({ ...t })) : [];
  });

  function onEdit(e) {
    const { index, task, working_dir } = e.detail;
    localItems[index] = { ...localItems[index], task, working_dir };
    localItems = [...localItems];
  }

  async function approve() {
    if (!plan || loading) return;
    loading = true;
    error = '';
    try {
      // If items were edited, send the edits first
      const edited = localItems.some((item, i) =>
        item.task !== plan.teams[i]?.task || item.working_dir !== plan.teams[i]?.working_dir
      );
      if (edited) {
        await api.editPlan(plan.plan_id, localItems);
      }
      await api.approvePlan(plan.plan_id);
      pendingPlan.set(null);
      currentView.set('home');
    } catch (e) {
      error = e.message || 'Failed to approve plan';
    } finally {
      loading = false;
    }
  }

  function cancel() {
    pendingPlan.set(null);
    currentView.set('home');
  }
</script>

<div transition:fade={{ duration: 200 }} class="flex flex-col min-h-screen max-w-2xl mx-auto px-4">

  <!-- Header -->
  <header class="flex items-center gap-4 py-5">
    <button
      on:click={cancel}
      aria-label="Cancel plan"
      class="h-10 w-10 flex items-center justify-center rounded-xl bg-white/5 hover:bg-white/10
             border border-white/5 text-white/40 hover:text-white/70 transition-colors flex-shrink-0"
    >
      <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
      </svg>
    </button>
    <div>
      <h1 class="text-lg font-bold text-white">Review Plan</h1>
      <p class="text-white/40 text-xs">
        {localItems.length} team{localItems.length !== 1 ? 's' : ''} — review and approve to start
      </p>
    </div>
  </header>

  <!-- Task summary if available -->
  {#if plan?.task}
    <div class="bg-white/5 border border-white/10 rounded-xl p-4 mb-4">
      <p class="text-xs text-white/40 uppercase tracking-wider font-semibold mb-1">Task</p>
      <p class="text-sm text-white/80">{plan.task}</p>
    </div>
  {/if}

  <!-- Plan items -->
  <main class="flex-1 space-y-3 mb-6">
    {#each localItems as item, i (i)}
      <div transition:slide>
        <PlanCard {item} index={i} on:edit={onEdit} />
      </div>
    {/each}

    {#if localItems.length === 0}
      <div class="text-center py-12 text-white/30 text-sm">No teams in this plan</div>
    {/if}
  </main>

  {#if error}
    <p transition:slide class="text-red-400 text-sm mb-4">{error}</p>
  {/if}

  <!-- Action buttons -->
  <div class="flex gap-3 pb-8">
    <button
      on:click={cancel}
      class="flex-1 h-12 rounded-xl font-semibold text-sm
             bg-white/5 hover:bg-white/10 border border-white/10
             text-white/60 hover:text-white/80 transition-all"
    >
      Cancel
    </button>
    <button
      on:click={approve}
      disabled={loading || localItems.length === 0}
      aria-label="Approve plan and start agents"
      class="flex-[2] h-12 rounded-xl font-semibold text-sm transition-all duration-200
             bg-gradient-to-r from-blue-500 to-purple-600
             hover:from-blue-400 hover:to-purple-500
             disabled:opacity-40 disabled:cursor-not-allowed text-white"
    >
      {#if loading}
        <span class="inline-flex items-center justify-center gap-2">
          <svg class="animate-spin w-4 h-4" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v8z"/>
          </svg>
          Approving...
        </span>
      {:else}
        Approve All — Start Agents
      {/if}
    </button>
  </div>
</div>
