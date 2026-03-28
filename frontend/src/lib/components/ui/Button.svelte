<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  // Props
  export let variant: 'primary' | 'secondary' | 'danger' | 'ghost' = 'primary';
  export let size: 'sm' | 'md' | 'lg' = 'md';
  export let loading: boolean = false;
  export let disabled: boolean = false;
  export let type: 'button' | 'submit' | 'reset' = 'button';
  export let fullWidth: boolean = false;

  const dispatch = createEventDispatcher();

  $: isDisabled = disabled || loading;

  function handleClick(event: MouseEvent) {
    if (!isDisabled) {
      dispatch('click', event);
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if ((event.key === 'Enter' || event.key === ' ') && !isDisabled) {
      event.preventDefault();
      dispatch('click', event);
    }
  }
</script>

<button
  {type}
  class="btn btn--{variant} btn--{size}"
  class:btn--full={fullWidth}
  disabled={isDisabled}
  aria-disabled={isDisabled}
  aria-busy={loading}
  on:click={handleClick}
  on:keydown={handleKeydown}
  {...$$restProps}
>
  <!-- Loading spinner -->
  {#if loading}
    <svg class="btn__spinner" fill="none" viewBox="0 0 24 24" aria-hidden="true">
      <circle class="btn__spinner-track" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
      <path class="btn__spinner-fill" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
    </svg>
    <span class="sr-only">Loading</span>
  {:else}
    <!-- Left icon slot -->
    {#if $$slots.iconLeft}
      <span class="btn__icon" aria-hidden="true">
        <slot name="iconLeft" />
      </span>
    {/if}
  {/if}

  <!-- Button content -->
  <slot />

  <!-- Right icon slot -->
  {#if $$slots.iconRight && !loading}
    <span class="btn__icon" aria-hidden="true">
      <slot name="iconRight" />
    </span>
  {/if}
</button>

<style>
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    font-weight: 500;
    border-radius: var(--radius-lg);
    border: 1px solid transparent;
    cursor: pointer;
    transition: background-color var(--transition-fast), border-color var(--transition-fast), color var(--transition-fast), transform var(--transition-fast);
    font-family: inherit;
  }

  .btn:focus-visible {
    outline: none;
    box-shadow: 0 0 0 2px var(--color-brand-primary), 0 0 0 4px var(--color-bg-primary);
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn:not(:disabled):hover {
    transform: scale(1.02);
  }

  .btn:not(:disabled):active {
    transform: scale(0.98);
  }

  /* Sizes */
  .btn--sm {
    padding: 0.375rem 0.75rem;
    font-size: var(--text-sm);
    gap: 0.375rem;
  }

  .btn--md {
    padding: 0.625rem 1rem;
    font-size: var(--text-base);
    gap: 0.5rem;
  }

  .btn--lg {
    padding: 0.75rem 1.5rem;
    font-size: 1.125rem;
    gap: 0.625rem;
  }

  /* Variants */
  .btn--primary {
    background-color: var(--color-brand-primary);
    color: var(--color-text-on-brand);
  }

  .btn--primary:not(:disabled):hover {
    background-color: var(--color-brand-primary-hover);
  }

  .btn--secondary {
    background-color: var(--color-bg-interactive);
    color: var(--color-text-primary);
    border-color: var(--color-border-default);
  }

  .btn--secondary:not(:disabled):hover {
    background-color: var(--color-bg-hover);
  }

  .btn--danger {
    background-color: var(--color-error);
    color: var(--color-text-on-brand);
  }

  .btn--danger:not(:disabled):hover {
    background-color: var(--color-error-hover);
  }

  .btn--ghost {
    background-color: transparent;
    color: var(--color-text-secondary);
  }

  .btn--ghost:not(:disabled):hover {
    background-color: var(--color-bg-interactive);
    color: var(--color-text-primary);
  }

  /* Full width */
  .btn--full {
    width: 100%;
  }

  /* Icon sizing follows button size */
  .btn--sm .btn__icon { width: 1rem; height: 1rem; }
  .btn--md .btn__icon { width: 1.25rem; height: 1.25rem; }
  .btn--lg .btn__icon { width: 1.5rem; height: 1.5rem; }

  /* Spinner */
  .btn__spinner {
    animation: btn-spin 1s linear infinite;
  }

  .btn--sm .btn__spinner { width: 1rem; height: 1rem; }
  .btn--md .btn__spinner { width: 1.25rem; height: 1.25rem; }
  .btn--lg .btn__spinner { width: 1.5rem; height: 1.5rem; }

  .btn__spinner-track { opacity: 0.25; }
  .btn__spinner-fill { opacity: 0.75; }

  /* Screen reader only */
  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border-width: 0;
  }

  @keyframes btn-spin {
    to { transform: rotate(360deg); }
  }

  @media (prefers-reduced-motion: reduce) {
    .btn {
      transition: none;
    }

    .btn:not(:disabled):hover,
    .btn:not(:disabled):active {
      transform: none;
    }

    .btn__spinner {
      animation: none;
    }
  }
</style>
