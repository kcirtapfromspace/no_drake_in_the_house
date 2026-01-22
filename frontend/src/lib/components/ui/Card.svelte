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

  // Check for reduced motion
  const prefersReducedMotion = typeof window !== 'undefined'
    ? window.matchMedia('(prefers-reduced-motion: reduce)').matches
    : false;

  const paddingClasses = {
    none: '',
    sm: 'p-3',
    md: 'p-4 sm:p-6',
    lg: 'p-6 sm:p-8',
  };

  const variantClasses = {
    default: 'bg-zinc-800 border border-zinc-700 rounded-xl',
    elevated: 'bg-zinc-800 border border-zinc-700 rounded-xl shadow-lg shadow-black/20',
    interactive: `bg-zinc-800 border border-zinc-700 rounded-xl cursor-pointer ${!prefersReducedMotion ? 'hover:border-zinc-600 hover:bg-zinc-750 transition-all duration-150' : ''} focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-rose-500 focus-visible:ring-offset-2 focus-visible:ring-offset-zinc-900`,
    stat: 'bg-zinc-800 border border-zinc-600 rounded-xl',
  };

  $: classes = `${variantClasses[variant]} ${paddingClasses[padding]}`;
</script>

{#if href}
  <a
    {href}
    class={classes}
    class:interactive={variant === 'interactive'}
    {...$$restProps}
  >
    <slot />
  </a>
{:else if variant === 'interactive'}
  <button
    type="button"
    class={`${classes} w-full text-left`}
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
  /* Smooth background transition for interactive cards */
  .interactive {
    background-color: #27272a;
  }

  @media (prefers-reduced-motion: no-preference) {
    .interactive:hover {
      background-color: #3f3f46;
    }
  }
</style>
