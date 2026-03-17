<script lang="ts">
  import type { ConfidenceLevel } from '../../stores/artist';
  import { getConfidenceLabel } from '../../stores/artist';

  export let level: ConfidenceLevel;
  export let showLabel: boolean = true;
  export let size: 'sm' | 'md' | 'lg' = 'md';

  $: label = getConfidenceLabel(level);
  $: filledBars = level === 'high' ? 3 : level === 'medium' ? 2 : 1;

  const levelColors: Record<ConfidenceLevel, string> = {
    high: '#10B981',
    medium: '#F59E0B',
    low: '#EF4444',
  };

  $: color = levelColors[level];
</script>

<div class="confidence confidence--{size}">
  <div class="confidence__bars">
    {#each [0, 1, 2] as i}
      <div
        class="confidence__bar"
        style="background: {i < filledBars ? color : 'var(--color-border-default)'};"
      ></div>
    {/each}
  </div>
  {#if showLabel}
    <span class="confidence__label">{label}</span>
  {/if}
</div>

<style>
  .confidence {
    display: inline-flex;
    align-items: center;
    padding: 0.25rem 0.75rem;
    border-radius: var(--radius-full);
    background-color: var(--color-bg-elevated);
  }

  .confidence__bars {
    display: flex;
  }

  .confidence--sm .confidence__bars { gap: 0.125rem; }
  .confidence--md .confidence__bars { gap: 0.125rem; }
  .confidence--lg .confidence__bars { gap: 0.25rem; }

  .confidence__bar {
    border-radius: 1px;
    transition: background-color var(--transition-fast);
  }

  .confidence--sm .confidence__bar { width: 0.375rem; height: 0.75rem; }
  .confidence--md .confidence__bar { width: 0.5rem; height: 1rem; }
  .confidence--lg .confidence__bar { width: 0.625rem; height: 1.25rem; }

  .confidence__label {
    color: var(--color-text-secondary);
    margin-left: 0.375rem;
  }

  .confidence--sm .confidence__label { font-size: var(--text-xs); }
  .confidence--md .confidence__label { font-size: var(--text-sm); }
  .confidence--lg .confidence__label { font-size: var(--text-base); }
</style>
