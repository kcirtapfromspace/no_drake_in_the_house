<script lang="ts">
  import type { ArtistStatus } from '../../stores/artist';
  import { getStatusColor, getStatusLabel } from '../../stores/artist';

  export let status: ArtistStatus;
  export let size: 'sm' | 'md' | 'lg' = 'md';

  $: colors = getStatusColor(status);
  $: label = getStatusLabel(status);
</script>

<span
  class="status-badge status-badge--{size}"
  style="background: {colors.bg}; color: {colors.text};"
>
  {#if status === 'flagged'}
    <svg class="status-badge__icon" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
    </svg>
  {:else if status === 'certified_creeper'}
    <svg class="status-badge__icon" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
    </svg>
  {:else}
    <svg class="status-badge__icon" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
    </svg>
  {/if}
  {label}
</span>

<style>
  .status-badge {
    display: inline-flex;
    align-items: center;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    border-radius: var(--radius-full);
  }

  .status-badge--sm {
    padding: 0.125rem 0.5rem;
    font-size: var(--text-xs);
    gap: 0.125rem;
  }

  .status-badge--md {
    padding: 0.25rem 0.75rem;
    font-size: var(--text-sm);
    gap: 0.25rem;
  }

  .status-badge--lg {
    padding: 0.375rem 1rem;
    font-size: var(--text-base);
    gap: 0.25rem;
  }

  .status-badge__icon {
    flex-shrink: 0;
  }

  .status-badge--sm .status-badge__icon { width: 0.75rem; height: 0.75rem; }
  .status-badge--md .status-badge__icon { width: 0.875rem; height: 0.875rem; }
  .status-badge--lg .status-badge__icon { width: 1rem; height: 1rem; }
</style>
