<script lang="ts">
  import { subscription, currentPlan, initiatePortal } from '../stores/billing';
  import { navigateTo } from '../utils/simple-router';

  function formatDate(iso: string): string {
    try {
      const d = new Date(iso);
      return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
    } catch {
      return iso;
    }
  }

  function capitalize(s: string): string {
    return s.charAt(0).toUpperCase() + s.slice(1);
  }

  function handleManageBilling() {
    initiatePortal();
  }

  function handleUpgrade() {
    navigateTo('pricing');
  }

  $: sub = $subscription;
  $: plan = $currentPlan;
</script>

<div class="billing-settings">
  {#if sub === undefined}
    <!-- Loading state -->
    <div data-testid="billing-loading" class="billing-loading">
      <div class="billing-skeleton billing-skeleton--title" />
      <div class="billing-skeleton billing-skeleton--text" />
      <div class="billing-skeleton billing-skeleton--text" />
      <div class="billing-skeleton billing-skeleton--btn" />
    </div>
  {:else if sub && sub.error}
    <!-- Error state -->
    <div data-testid="billing-error" class="billing-error">
      <svg class="billing-error__icon" viewBox="0 0 20 20" fill="currentColor">
        <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
      </svg>
      <p class="billing-error__text">{sub.error}</p>
    </div>
  {:else}
    <div class="billing-content">
      <!-- Plan Info -->
      <div class="billing-plan-row">
        <div class="billing-plan-info">
          <h3 class="billing-label">Current Plan</h3>
          <div class="billing-plan-name-row">
            <span data-testid="current-plan-name" class="billing-plan-name">
              {capitalize(plan)}
            </span>
            {#if sub}
              <span
                data-testid="plan-status-badge"
                class="plan-badge"
                class:plan-badge--active={sub.status === 'active'}
                class:plan-badge--canceled={sub.status === 'canceled'}
                class:plan-badge--past-due={sub.status === 'past_due'}
              >
                {capitalize(sub.status === 'past_due' ? 'past due' : sub.status)}
              </span>
            {:else}
              <span data-testid="plan-status-badge" class="plan-badge plan-badge--active">Active</span>
            {/if}
          </div>
        </div>
      </div>

      <!-- Billing Period -->
      {#if sub && sub.currentPeriodStart && sub.currentPeriodEnd}
        <div data-testid="billing-period" class="billing-period">
          <h3 class="billing-label">Billing Period</h3>
          <p class="billing-period__dates">
            {formatDate(sub.currentPeriodStart)} &mdash; {formatDate(sub.currentPeriodEnd)}
          </p>
        </div>
      {/if}

      <!-- Cancel Notice -->
      {#if sub && sub.cancelAtPeriodEnd}
        <div data-testid="cancel-notice" class="cancel-notice">
          <svg class="cancel-notice__icon" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
          </svg>
          <p>Your subscription will cancel at the end of the current billing period.</p>
        </div>
      {/if}

      <!-- Actions -->
      <div class="billing-actions">
        {#if sub && plan !== 'free'}
          <button
            type="button"
            data-testid="manage-billing-btn"
            class="billing-btn billing-btn--secondary"
            on:click={handleManageBilling}
          >
            Manage Billing
          </button>
          <button
            type="button"
            class="billing-btn billing-btn--outline"
            on:click={handleUpgrade}
          >
            Change Plan
          </button>
        {:else}
          <button
            type="button"
            data-testid="upgrade-btn"
            class="billing-btn billing-btn--primary"
            on:click={handleUpgrade}
          >
            Upgrade Plan
          </button>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .billing-settings {
    background: var(--color-bg-elevated, #18181b);
    border: 1px solid var(--color-border-default, #3f3f46);
    border-radius: 0.75rem;
    padding: 1.5rem;
  }

  .billing-loading {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .billing-skeleton {
    background: linear-gradient(90deg, #27272a 25%, #3f3f46 50%, #27272a 75%);
    background-size: 200% 100%;
    animation: shimmer 1.5s infinite;
    border-radius: 0.375rem;
  }

  .billing-skeleton--title {
    height: 1.5rem;
    width: 40%;
  }

  .billing-skeleton--text {
    height: 1rem;
    width: 70%;
  }

  .billing-skeleton--btn {
    height: 2.5rem;
    width: 50%;
    margin-top: 0.5rem;
  }

  @keyframes shimmer {
    0% { background-position: -200% 0; }
    100% { background-position: 200% 0; }
  }

  .billing-error {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 1rem;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    border-radius: 0.5rem;
  }

  .billing-error__icon {
    width: 1.25rem;
    height: 1.25rem;
    color: #ef4444;
    flex-shrink: 0;
  }

  .billing-error__text {
    font-size: 0.875rem;
    color: #fca5a5;
  }

  .billing-content {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }

  .billing-label {
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--color-text-tertiary, #71717a);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: 0.375rem;
  }

  .billing-plan-name-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .billing-plan-name {
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--color-text-primary, white);
  }

  .plan-badge {
    font-size: 0.6875rem;
    font-weight: 600;
    padding: 0.125rem 0.5rem;
    border-radius: 9999px;
    text-transform: capitalize;
  }

  .plan-badge--active {
    background: rgba(16, 185, 129, 0.15);
    color: #34d399;
  }

  .plan-badge--canceled {
    background: rgba(239, 68, 68, 0.15);
    color: #fca5a5;
  }

  .plan-badge--past-due {
    background: rgba(245, 158, 11, 0.15);
    color: #fbbf24;
  }

  .billing-period__dates {
    font-size: 0.875rem;
    color: var(--color-text-secondary, #a1a1aa);
  }

  .cancel-notice {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1rem;
    background: rgba(245, 158, 11, 0.1);
    border: 1px solid rgba(245, 158, 11, 0.3);
    border-radius: 0.5rem;
  }

  .cancel-notice__icon {
    width: 1.25rem;
    height: 1.25rem;
    color: #f59e0b;
    flex-shrink: 0;
  }

  .cancel-notice p {
    font-size: 0.875rem;
    color: #fbbf24;
  }

  .billing-actions {
    display: flex;
    gap: 0.75rem;
    padding-top: 0.5rem;
  }

  .billing-btn {
    padding: 0.625rem 1.25rem;
    border-radius: 0.5rem;
    font-size: 0.875rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
    border: 1px solid transparent;
  }

  .billing-btn--primary {
    background: #10b981;
    color: white;
    border-color: #10b981;
  }

  .billing-btn--primary:hover {
    background: #059669;
  }

  .billing-btn--secondary {
    background: var(--color-bg-interactive, #27272a);
    color: var(--color-text-primary, white);
    border-color: var(--color-border-default, #3f3f46);
  }

  .billing-btn--secondary:hover {
    border-color: var(--color-border-hover, #52525b);
  }

  .billing-btn--outline {
    background: transparent;
    color: var(--color-text-secondary, #a1a1aa);
    border-color: var(--color-border-default, #3f3f46);
  }

  .billing-btn--outline:hover {
    color: var(--color-text-primary, white);
    border-color: var(--color-border-hover, #52525b);
  }
</style>
