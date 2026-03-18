<script>
  import { createVoiceRecognition } from '../lib/voice.js';
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher();

  let recording = false;
  let recognition = null;

  function toggle() {
    if (recording) {
      recognition?.stop();
      recording = false;
      return;
    }

    recognition = createVoiceRecognition((transcript) => {
      dispatch('result', transcript);
      recording = false;
    });

    if (!recognition) {
      dispatch('unsupported');
      return;
    }

    recognition.onerror = () => { recording = false; };
    recognition.onend = () => { recording = false; };
    recognition.start();
    recording = true;
  }
</script>

<button
  type="button"
  on:click={toggle}
  aria-label={recording ? 'Stop recording' : 'Start voice input'}
  class="relative h-12 w-12 flex items-center justify-center rounded-xl transition-all duration-200
    {recording
      ? 'bg-red-500/30 border border-red-500/50 text-red-400'
      : 'bg-white/10 border border-white/10 text-white/60 hover:bg-white/15 hover:text-white/80'}"
>
  {#if recording}
    <!-- pulse rings -->
    <span class="absolute inset-0 rounded-xl animate-ping bg-red-500/20"></span>
  {/if}

  <!-- mic icon -->
  <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
    <path stroke-linecap="round" stroke-linejoin="round"
      d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z" />
    <path stroke-linecap="round" stroke-linejoin="round"
      d="M19 10v2a7 7 0 0 1-14 0v-2" />
    <line x1="12" y1="19" x2="12" y2="23" stroke-linecap="round" />
    <line x1="8" y1="23" x2="16" y2="23" stroke-linecap="round" />
  </svg>
</button>
