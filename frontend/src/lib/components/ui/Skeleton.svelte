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
  <div class="space-y-2" style="width: {width};">
    {#each Array(lines) as _, i}
      <div
        class="rounded h-4 {shouldAnimate ? 'animate-pulse' : ''}"
        style="background: #3f3f46; width: {i === lines - 1 && lines > 1 ? '75%' : '100%'};"
      ></div>
    {/each}
  </div>
{:else if variant === 'circular'}
  <div
    class="rounded-full {shouldAnimate ? 'animate-pulse' : ''}"
    style="background: #3f3f46; width: {width}; height: {height || width};"
  ></div>
{:else if variant === 'rectangular'}
  <div
    class="rounded-lg {shouldAnimate ? 'animate-pulse' : ''}"
    style="background: #3f3f46; width: {width}; height: {height || '100px'};"
  ></div>
{:else if variant === 'card'}
  <div
    class="rounded-xl p-4 {animate ? '' : ''}"
    style="background: #27272a; border: 1px solid #52525b;"
  >
    <div class="flex gap-4">
      <div
        class="rounded-lg flex-shrink-0 {shouldAnimate ? 'animate-pulse' : ''}"
        style="background: #3f3f46; width: 80px; height: 80px;"
      ></div>
      <div class="flex-1 space-y-3">
        <div
          class="h-5 rounded {shouldAnimate ? 'animate-pulse' : ''}"
          style="background: #3f3f46; width: 60%;"
        ></div>
        <div
          class="h-4 rounded {shouldAnimate ? 'animate-pulse' : ''}"
          style="background: #52525b; width: 40%;"
        ></div>
        <div
          class="h-3 rounded {shouldAnimate ? 'animate-pulse' : ''}"
          style="background: #52525b; width: 80%;"
        ></div>
      </div>
    </div>
  </div>
{/if}

<style>
  @keyframes pulse {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.5;
    }
  }

  .animate-pulse {
    animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
  }
</style>
