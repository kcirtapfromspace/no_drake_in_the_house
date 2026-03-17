<script lang="ts">
  import type { SourceTier } from '../../stores/artist';
  import { getSourceTierLabel } from '../../stores/artist';

  export let tier: SourceTier;
  export let showDescription: boolean = false;
  export let size: 'sm' | 'md' | 'lg' = 'md';

  $: tierInfo = getSourceTierLabel(tier);
</script>

<div class="tier-badge tier-badge--{size}" title={tierInfo.description}>
  <div
    class="tier-badge__icon"
    style="background: {tierInfo.color}20; color: {tierInfo.color};"
  >
    {tierInfo.label.replace('Tier ', '')}
  </div>
  {#if showDescription}
    <div class="tier-badge__info">
      <p class="tier-badge__label">{tierInfo.label}</p>
      <p class="tier-badge__desc">{tierInfo.description}</p>
    </div>
  {/if}
</div>

<style>
  .tier-badge {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
  }

  .tier-badge__icon {
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-sm);
    font-weight: 700;
  }

  .tier-badge--sm .tier-badge__icon { width: 1.5rem; height: 1.5rem; font-size: 0.625rem; }
  .tier-badge--md .tier-badge__icon { width: 2rem; height: 2rem; font-size: var(--text-xs); }
  .tier-badge--lg .tier-badge__icon { width: 2.5rem; height: 2.5rem; font-size: var(--text-sm); }

  .tier-badge__label {
    color: var(--color-text-primary);
    font-weight: 500;
    margin: 0;
  }

  .tier-badge--sm .tier-badge__label { font-size: var(--text-xs); }
  .tier-badge--md .tier-badge__label { font-size: var(--text-sm); }
  .tier-badge--lg .tier-badge__label { font-size: var(--text-base); }

  .tier-badge__desc {
    color: var(--color-text-muted);
    font-size: var(--text-xs);
    margin: 0;
  }
</style>
