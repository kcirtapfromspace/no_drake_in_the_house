<script lang="ts">
  export let variant: 'text' | 'circular' | 'rectangular' | 'card' = 'text';
  export let width: string = '100%';
  export let height: string | undefined = undefined;
  export let lines: number = 1;
  export let animate: boolean = true;

  // Respect reduced motion preference
  const prefersReducedMotion = typeof window !== 'undefined'
    ? window.matchMedia('(prefers-reduced-motion: reduce)').matches
    : false;

  $: shouldAnimate = animate && !prefersReducedMotion;
</script>

{#if variant === 'text'}
  <div class="skeleton-text" style="width: {width};">
    {#each Array(lines) as _, i}
      <div
        class="skeleton-line"
        class:skeleton--animated={shouldAnimate}
        style="width: {i === lines - 1 && lines > 1 ? '75%' : '100%'};"
      ></div>
    {/each}
  </div>
{:else if variant === 'circular'}
  <div
    class="skeleton-circle"
    class:skeleton--animated={shouldAnimate}
    style="width: {width}; height: {height || width};"
  ></div>
{:else if variant === 'rectangular'}
  <div
    class="skeleton-rect"
    class:skeleton--animated={shouldAnimate}
    style="width: {width}; height: {height || '100px'};"
  ></div>
{:else if variant === 'card'}
  <div class="skeleton-card">
    <div class="skeleton-card__inner">
      <div
        class="skeleton-card__thumb"
        class:skeleton--animated={shouldAnimate}
      ></div>
      <div class="skeleton-card__content">
        <div class="skeleton-card__title" class:skeleton--animated={shouldAnimate}></div>
        <div class="skeleton-card__sub" class:skeleton--animated={shouldAnimate}></div>
        <div class="skeleton-card__text" class:skeleton--animated={shouldAnimate}></div>
      </div>
    </div>
  </div>
{/if}

<style>
  /* Shared skeleton shimmer */
  .skeleton-line,
  .skeleton-circle,
  .skeleton-rect,
  .skeleton-card__thumb,
  .skeleton-card__title,
  .skeleton-card__sub,
  .skeleton-card__text {
    background-color: var(--color-bg-interactive, #3f3f46);
  }

  .skeleton--animated {
    animation: skeleton-pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
  }

  /* Text variant */
  .skeleton-text {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .skeleton-line {
    height: 1rem;
    border-radius: var(--radius-sm);
  }

  /* Circular variant */
  .skeleton-circle {
    border-radius: var(--radius-full);
  }

  /* Rectangular variant */
  .skeleton-rect {
    border-radius: var(--radius-lg);
  }

  /* Card variant */
  .skeleton-card {
    border-radius: var(--radius-xl);
    padding: 1rem;
    background-color: var(--color-bg-elevated);
    border: 1px solid var(--color-border-default);
  }

  .skeleton-card__inner {
    display: flex;
    gap: 1rem;
  }

  .skeleton-card__thumb {
    width: 5rem;
    height: 5rem;
    border-radius: var(--radius-lg);
    flex-shrink: 0;
  }

  .skeleton-card__content {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .skeleton-card__title {
    height: 1.25rem;
    width: 60%;
    border-radius: var(--radius-sm);
  }

  .skeleton-card__sub {
    height: 1rem;
    width: 40%;
    border-radius: var(--radius-sm);
    background-color: var(--color-border-default, #52525b);
  }

  .skeleton-card__text {
    height: 0.75rem;
    width: 80%;
    border-radius: var(--radius-sm);
    background-color: var(--color-border-default, #52525b);
  }

  @keyframes skeleton-pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }
</style>
