<script lang="ts">
  /**
   * Unified Card component for consistent styling
   * Variants:
   * - default: Standard card with border
   * - elevated: Card with shadow
   * - interactive: Clickable card with hover state
   * - stat: Compact card for statistics
   */

  export let variant: 'default' | 'elevated' | 'interactive' | 'stat' = 'default';
  export let padding: 'none' | 'sm' | 'md' | 'lg' = 'md';
  export let href: string | undefined = undefined;
  export let as: 'div' | 'article' | 'section' = 'div';

  $: paddingClass = padding === 'none' ? '' :
    padding === 'sm' ? 'card--pad-sm' :
    padding === 'lg' ? 'card--pad-lg' : 'card--pad-md';

  $: variantClass = variant === 'elevated' ? 'card--elevated' :
    variant === 'interactive' ? 'card--interactive' :
    variant === 'stat' ? 'card--stat' : '';

  $: classes = `card ${variantClass} ${paddingClass}`;
</script>

{#if href}
  <a
    {href}
    class={classes}
    {...$$restProps}
  >
    <slot />
  </a>
{:else if variant === 'interactive'}
  <button
    type="button"
    class="{classes} card--full-width"
    on:click
    on:keydown
    {...$$restProps}
  >
    <slot />
  </button>
{:else if as === 'article'}
  <article class={classes} {...$$restProps}>
    <slot />
  </article>
{:else if as === 'section'}
  <section class={classes} {...$$restProps}>
    <slot />
  </section>
{:else}
  <div class={classes} {...$$restProps}>
    <slot />
  </div>
{/if}

<style>
  .card {
    background-color: var(--color-bg-elevated);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-xl);
    color: inherit;
    text-decoration: none;
  }

  /* Padding variants */
  .card--pad-sm { padding: 0.75rem; }
  .card--pad-md { padding: 1rem; }
  .card--pad-lg { padding: 1.5rem; }

  @media (min-width: 640px) {
    .card--pad-md { padding: 1.5rem; }
    .card--pad-lg { padding: 2rem; }
  }

  /* Variant: elevated */
  .card--elevated {
    box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.2), 0 4px 6px -4px rgba(0, 0, 0, 0.2);
  }

  /* Variant: interactive */
  .card--interactive {
    cursor: pointer;
    transition: border-color var(--transition-fast), background-color var(--transition-fast);
    text-align: left;
    font: inherit;
  }

  .card--interactive:hover {
    border-color: var(--color-border-hover);
    background-color: var(--color-bg-hover);
  }

  .card--interactive:focus-visible {
    outline: none;
    box-shadow: 0 0 0 2px var(--color-brand-primary), 0 0 0 4px var(--color-bg-primary);
  }

  /* Variant: stat */
  .card--stat {
    border-color: var(--color-border-strong);
  }

  /* Full width for button variant */
  .card--full-width {
    width: 100%;
    border: 1px solid var(--color-border-default);
    background-color: var(--color-bg-elevated);
  }

  @media (prefers-reduced-motion: reduce) {
    .card--interactive {
      transition: none;
    }
  }
</style>
