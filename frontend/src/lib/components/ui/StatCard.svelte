<script lang="ts">
  /**
   * Stat card component for displaying key metrics
   */
  export let label: string;
  export let value: string | number;
  export let icon: string | undefined = undefined;
  export let trend: 'up' | 'down' | 'neutral' | undefined = undefined;
  export let trendValue: string | undefined = undefined;
  export let iconBg: string | undefined = undefined;
</script>

<div class="stat-card">
  <div class="stat-card__inner">
    {#if icon}
      <div class="stat-card__icon" style={iconBg ? `background-color: ${iconBg}` : ''}>
        {icon}
      </div>
    {/if}
    <div class="stat-card__content">
      <div class="stat-card__value">{value}</div>
      <div class="stat-card__label">
        {label}
        {#if trend && trendValue}
          <span class="stat-card__trend stat-card__trend--{trend}">
            {trend === 'up' ? '\u2191' : trend === 'down' ? '\u2193' : '\u2192'} {trendValue}
          </span>
        {/if}
      </div>
    </div>
  </div>
  <slot />
</div>

<style>
  .stat-card {
    background-color: var(--color-bg-elevated);
    border: 1px solid var(--color-border-strong);
    border-radius: var(--radius-xl);
    padding: 1.25rem;
  }

  .stat-card__inner {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .stat-card__icon {
    width: 3rem;
    height: 3rem;
    border-radius: var(--radius-full);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.25rem;
    background-color: var(--color-bg-interactive);
    flex-shrink: 0;
  }

  .stat-card__content {
    flex: 1;
    min-width: 0;
  }

  .stat-card__value {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--color-text-primary);
  }

  .stat-card__label {
    font-size: var(--text-sm);
    color: var(--color-text-muted);
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .stat-card__trend {
    font-size: var(--text-sm);
    font-weight: 500;
  }

  .stat-card__trend--up { color: var(--color-success); }
  .stat-card__trend--down { color: var(--color-error); }
  .stat-card__trend--neutral { color: var(--color-text-muted); }
</style>
