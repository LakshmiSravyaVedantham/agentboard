<script>
  import { afterUpdate } from 'svelte';

  export let lines = [];
  export let maxHeight = '320px';

  let container;

  afterUpdate(() => {
    if (container) {
      container.scrollTop = container.scrollHeight;
    }
  });
</script>

<div
  bind:this={container}
  class="bg-black/50 rounded-xl p-4 font-mono text-sm text-green-300 overflow-y-auto
         border border-white/5 leading-relaxed"
  style="max-height: {maxHeight}; min-height: 120px;"
  role="log"
  aria-label="Terminal output"
  aria-live="polite"
>
  {#if lines.length === 0}
    <span class="text-white/20">Waiting for output...</span>
  {:else}
    {#each lines as line}
      <div class="whitespace-pre-wrap break-all">{line}</div>
    {/each}
  {/if}
</div>
