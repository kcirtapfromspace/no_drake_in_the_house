<script lang="ts">
  /**
   * Stat card component for displaying key metrics
   */
  export let label: string;
  export let value: string | number;
  export let icon: string | undefined = undefined;
  export let trend: 'up' | 'down' | 'neutral' | undefined = undefined;
  export let trendValue: string | undefined = undefined;
  export let iconBg: string = 'bg-zinc-700';

  const trendColors = {
    up: 'text-green-400',
    down: 'text-red-400',
    neutral: 'text-zinc-400',
  };

  const trendIcons = {
    up: '↑',
    down: '↓',
    neutral: '→',
  };
</script>

<div class="bg-zinc-800 border border-zinc-600 rounded-xl p-5">
  <div class="flex items-center gap-3">
    {#if icon}
      <div class="w-12 h-12 rounded-full flex items-center justify-center text-xl {iconBg}">
        {icon}
      </div>
    {/if}
    <div class="flex-1">
      <div class="text-2xl font-bold text-white">
        {value}
      </div>
      <div class="text-sm text-zinc-400 flex items-center gap-2">
        {label}
        {#if trend && trendValue}
          <span class={trendColors[trend]}>
            {trendIcons[trend]} {trendValue}
          </span>
        {/if}
      </div>
    </div>
  </div>
  <slot />
</div>
