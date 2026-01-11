<script lang="ts">
  import type { ConfidenceLevel } from '../../stores/artist';
  import { getConfidenceLabel } from '../../stores/artist';

  export let level: ConfidenceLevel;
  export let showLabel: boolean = true;
  export let size: 'sm' | 'md' | 'lg' = 'md';

  $: label = getConfidenceLabel(level);
  $: filledBars = level === 'high' ? 3 : level === 'medium' ? 2 : 1;

  const sizeConfig = {
    sm: { barWidth: 'w-1.5', barHeight: 'h-3', gap: 'gap-0.5', text: 'text-xs' },
    md: { barWidth: 'w-2', barHeight: 'h-4', gap: 'gap-0.5', text: 'text-sm' },
    lg: { barWidth: 'w-2.5', barHeight: 'h-5', gap: 'gap-1', text: 'text-base' },
  };

  const levelColors = {
    high: '#10B981',
    medium: '#F59E0B',
    low: '#EF4444',
  };

  $: config = sizeConfig[size];
  $: color = levelColors[level];
</script>

<div class="inline-flex items-center {config.gap} px-3 py-1 rounded-full bg-zinc-800/10 backdrop-blur-sm">
  <div class="flex {config.gap}">
    {#each [0, 1, 2] as i}
      <div
        class="{config.barWidth} {config.barHeight} rounded-sm transition-colors"
        style="background: {i < filledBars ? color : '#374151'};"
      ></div>
    {/each}
  </div>
  {#if showLabel}
    <span class="{config.text} text-zinc-300 ml-1">{label}</span>
  {/if}
</div>
