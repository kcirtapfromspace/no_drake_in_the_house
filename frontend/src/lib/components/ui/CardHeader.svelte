<script lang="ts">
  /**
   * Card header component for consistent card titles
   */
  export let title: string;
  export let subtitle: string | undefined = undefined;
</script>

<div class="mb-4">
  <h3 class="text-lg font-semibold text-white">
    {title}
  </h3>
  {#if subtitle}
    <p class="mt-1 text-sm text-zinc-400">
      {subtitle}
    </p>
  {/if}
  <slot />
</div>
