<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  // Props
  export let variant: 'primary' | 'secondary' | 'danger' | 'ghost' = 'primary';
  export let size: 'sm' | 'md' | 'lg' = 'md';
  export let loading: boolean = false;
  export let disabled: boolean = false;
  export let type: 'button' | 'submit' | 'reset' = 'button';
  export let fullWidth: boolean = false;

  // Optional icon slots are handled via $$slots

  const dispatch = createEventDispatcher();

  // Check for reduced motion preference
  const prefersReducedMotion = typeof window !== 'undefined'
    ? window.matchMedia('(prefers-reduced-motion: reduce)').matches
    : false;

  // Variant styles using design tokens
  const variantClasses = {
    primary: 'bg-rose-500 hover:bg-rose-600 text-white border-transparent',
    secondary: 'bg-zinc-700 hover:bg-zinc-600 text-white border-zinc-600',
    danger: 'bg-red-500 hover:bg-red-600 text-white border-transparent',
    ghost: 'bg-transparent hover:bg-zinc-700 text-zinc-300 border-transparent',
  };

  // Size styles
  const sizeClasses = {
    sm: 'px-3 py-1.5 text-sm gap-1.5',
    md: 'px-4 py-2.5 text-base gap-2',
    lg: 'px-6 py-3 text-lg gap-2.5',
  };

  // Icon sizes based on button size
  const iconSizes = {
    sm: 'w-4 h-4',
    md: 'w-5 h-5',
    lg: 'w-6 h-6',
  };

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
  class="
    inline-flex items-center justify-center font-medium rounded-lg
    border transition-all
    focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-rose-500 focus-visible:ring-offset-2 focus-visible:ring-offset-zinc-900
    disabled:opacity-50 disabled:cursor-not-allowed
    {variantClasses[variant]}
    {sizeClasses[size]}
    {fullWidth ? 'w-full' : ''}
    {!prefersReducedMotion && !isDisabled ? 'hover:scale-[1.02] active:scale-[0.98]' : ''}
  "
  disabled={isDisabled}
  aria-disabled={isDisabled}
  aria-busy={loading}
  on:click={handleClick}
  on:keydown={handleKeydown}
  {...$$restProps}
>
  <!-- Loading spinner -->
  {#if loading}
    <svg
      class="{iconSizes[size]} {prefersReducedMotion ? '' : 'animate-spin'}"
      fill="none"
      viewBox="0 0 24 24"
      aria-hidden="true"
    >
      <circle
        class="opacity-25"
        cx="12"
        cy="12"
        r="10"
        stroke="currentColor"
        stroke-width="4"
      />
      <path
        class="opacity-75"
        fill="currentColor"
        d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
      />
    </svg>
    <span class="sr-only">Loading</span>
  {:else}
    <!-- Left icon slot -->
    {#if $$slots.iconLeft}
      <span class="{iconSizes[size]}" aria-hidden="true">
        <slot name="iconLeft" />
      </span>
    {/if}
  {/if}

  <!-- Button content -->
  <slot />

  <!-- Right icon slot -->
  {#if $$slots.iconRight && !loading}
    <span class="{iconSizes[size]}" aria-hidden="true">
      <slot name="iconRight" />
    </span>
  {/if}
</button>

<style>
  /* Ensure smooth transitions */
  button {
    transition-property: background-color, border-color, color, transform, box-shadow;
    transition-duration: 150ms;
    transition-timing-function: ease-in-out;
  }

  /* Respect reduced motion preference */
  @media (prefers-reduced-motion: reduce) {
    button {
      transition-duration: 0ms;
    }
  }
</style>
