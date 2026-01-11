<script lang="ts">
  import type { SourceTier } from '../../stores/artist';
  import { getSourceTierLabel } from '../../stores/artist';

  export let tier: SourceTier;
  export let showDescription: boolean = false;
  export let size: 'sm' | 'md' | 'lg' = 'md';

  $: tierInfo = getSourceTierLabel(tier);

  const sizeConfig = {
    sm: { badge: 'w-6 h-6 text-[10px]', text: 'text-xs' },
    md: { badge: 'w-8 h-8 text-xs', text: 'text-sm' },
    lg: { badge: 'w-10 h-10 text-sm', text: 'text-base' },
  };

  $: config = sizeConfig[size];
</script>

<div class="inline-flex items-center gap-2" title={tierInfo.description}>
  <div
    class="{config.badge} rounded flex items-center justify-center font-bold"
    style="background: {tierInfo.color}20; color: {tierInfo.color};"
  >
    {tierInfo.label.replace('Tier ', '')}
  </div>
  {#if showDescription}
    <div class="{config.text}">
      <p class="text-white font-medium">{tierInfo.label}</p>
      <p class="text-zinc-500 text-xs">{tierInfo.description}</p>
    </div>
  {/if}
</div>
