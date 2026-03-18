<script>
  import { onMount } from 'svelte';
  import { currentView, authToken } from './lib/stores.js';
  import { connectWs } from './lib/ws.js';
  import Pair from './routes/Pair.svelte';
  import Home from './routes/Home.svelte';
  import Plan from './routes/Plan.svelte';
  import Team from './routes/Team.svelte';
  import Summary from './routes/Summary.svelte';
  import Settings from './routes/Settings.svelte';

  onMount(() => {
    if ($authToken) {
      connectWs();
    }
  });
</script>

<div class="min-h-screen bg-gradient-to-br from-slate-900 via-slate-800 to-slate-900 text-white">
  {#if $currentView === 'pair'}
    <Pair />
  {:else if $currentView === 'plan'}
    <Plan />
  {:else if $currentView === 'team'}
    <Team />
  {:else if $currentView === 'summary'}
    <Summary />
  {:else if $currentView === 'settings'}
    <Settings />
  {:else}
    <Home />
  {/if}
</div>
