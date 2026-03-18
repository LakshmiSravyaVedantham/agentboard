<script>
  import { createEventDispatcher } from 'svelte';

  export let item;
  export let index;

  const dispatch = createEventDispatcher();

  let editing = false;
  let editTask = item.task || '';
  let editDir = item.working_dir || '';

  function save() {
    dispatch('edit', { index, task: editTask, working_dir: editDir });
    editing = false;
  }

  function cancel() {
    editTask = item.task || '';
    editDir = item.working_dir || '';
    editing = false;
  }
</script>

<div class="bg-white/5 border border-white/10 rounded-xl p-4">
  <div class="flex items-start justify-between gap-3">
    <div class="flex-1 min-w-0">
      <div class="flex items-center gap-2 mb-2">
        <span class="text-xs font-semibold text-white/40 uppercase tracking-wider">Team {index + 1}</span>
        {#if item.name}
          <span class="text-xs text-white/60 font-medium">{item.name}</span>
        {/if}
      </div>

      {#if editing}
        <div class="space-y-2">
          <div>
            <label class="text-xs text-white/40 block mb-1" for="task-{index}">Task</label>
            <textarea
              id="task-{index}"
              bind:value={editTask}
              rows="3"
              class="w-full bg-white/10 border border-white/20 rounded-lg px-3 py-2 text-sm text-white
                     placeholder-white/30 resize-none focus:outline-none focus:border-blue-500/50"
            ></textarea>
          </div>
          <div>
            <label class="text-xs text-white/40 block mb-1" for="dir-{index}">Working directory</label>
            <input
              id="dir-{index}"
              type="text"
              bind:value={editDir}
              class="w-full bg-white/10 border border-white/20 rounded-lg px-3 py-2 text-sm text-white
                     font-mono placeholder-white/30 focus:outline-none focus:border-blue-500/50"
            />
          </div>
          <div class="flex gap-2 pt-1">
            <button
              on:click={save}
              class="h-9 px-4 bg-blue-600/80 hover:bg-blue-600 text-white text-sm rounded-lg transition-colors"
            >Save</button>
            <button
              on:click={cancel}
              class="h-9 px-4 bg-white/10 hover:bg-white/15 text-white/70 text-sm rounded-lg transition-colors"
            >Cancel</button>
          </div>
        </div>
      {:else}
        <p class="text-sm text-white/80 leading-relaxed mb-2">{item.task || 'No task specified'}</p>
        {#if item.working_dir}
          <p class="text-xs font-mono text-white/30">{item.working_dir}</p>
        {/if}
      {/if}
    </div>

    {#if !editing}
      <button
        on:click={() => editing = true}
        aria-label="Edit plan item"
        class="h-9 w-9 flex items-center justify-center rounded-lg bg-white/5 hover:bg-white/10
               text-white/40 hover:text-white/70 transition-colors flex-shrink-0"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round"
            d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
        </svg>
      </button>
    {/if}
  </div>
</div>
