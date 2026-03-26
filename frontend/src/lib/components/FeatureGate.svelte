<script lang="ts">
  import { featureAccess } from '../stores/billing';
  import UpgradePrompt from './UpgradePrompt.svelte';

  export let feature: string;
  export let accessible: boolean | undefined = undefined;
  export let loading: boolean = false;

  // If accessible prop is explicitly provided, use it; otherwise derive from featureAccess
  $: isAccessible = accessible !== undefined ? accessible : featureAccess(feature);
</script>

{#if loading}
  <div data-testid="feature-gate-loading" class="feature-gate-loading">
    <div class="skeleton skeleton--lg" />
    <div class="skeleton skeleton--md" />
    <div class="skeleton skeleton--sm" />
  </div>
{:else if isAccessible}
  <slot />
{:else}
  <UpgradePrompt {feature} />
{/if}

<style>
  .feature-gate-loading {
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .skeleton {
    background: linear-gradient(90deg, #27272a 25%, #3f3f46 50%, #27272a 75%);
    background-size: 200% 100%;
    animation: shimmer 1.5s infinite;
    border-radius: 0.375rem;
    height: 1rem;
  }

  .skeleton--lg {
    height: 1.5rem;
    width: 60%;
  }

  .skeleton--md {
    width: 80%;
  }

  .skeleton--sm {
    width: 40%;
  }

  @keyframes shimmer {
    0% { background-position: -200% 0; }
    100% { background-position: 200% 0; }
  }
</style>
