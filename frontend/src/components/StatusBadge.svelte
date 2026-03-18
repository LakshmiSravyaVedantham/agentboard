<script>
  export let status = 'Queued';

  const config = {
    Done:    { cls: 'bg-green-500/20 text-green-400 border-green-500/30',  dot: 'bg-green-400' },
    Running: { cls: 'bg-blue-500/20 text-blue-400 border-blue-500/30',    dot: 'bg-blue-400 animate-pulse' },
    Queued:  { cls: 'bg-amber-500/20 text-amber-400 border-amber-500/30', dot: 'bg-amber-400' },
    Failed:  { cls: 'bg-red-500/20 text-red-400 border-red-500/30',       dot: 'bg-red-400' },
    Killed:  { cls: 'bg-gray-500/20 text-gray-400 border-gray-500/30',    dot: 'bg-gray-400' },
  };

  $: cfg = config[status] || config['Queued'];
</script>

<span class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium border {cfg.cls}">
  <span class="w-1.5 h-1.5 rounded-full {cfg.dot}"></span>
  {status}
</span>
