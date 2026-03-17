<script lang="ts">
  /**
   * Card header component for consistent card titles
   */
  export let title: string;
  export let subtitle: string | undefined = undefined;
</script>

<div class="card-header">
  <h3 class="card-header__title">{title}</h3>
  {#if subtitle}
    <p class="card-header__subtitle">{subtitle}</p>
  {/if}
  <slot />
</div>

<style>
  .card-header {
    margin-bottom: 1rem;
  }

  .card-header__title {
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
  }

  .card-header__subtitle {
    margin-top: 0.25rem;
    font-size: var(--text-sm);
    color: var(--color-text-muted);
  }
</style>
