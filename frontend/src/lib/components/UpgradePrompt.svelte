<script lang="ts">
  import { currentPlan } from '../stores/billing';
  import { navigateTo } from '../utils/simple-router';

  export let feature: string = 'connections';

  let dismissed = false;

  interface FeatureMessage {
    title: string;
    description: string;
    icon: string;
  }

  const featureMessages: Record<string, FeatureMessage> = {
    connections: {
      title: 'More streaming connections',
      description: 'Upgrade to connect more streaming connections and protect all your music libraries.',
      icon: 'link',
    },
    scans: {
      title: 'Automated library scan',
      description: 'Upgrade to unlock automated library scan scheduling and never miss a problematic artist.',
      icon: 'search',
    },
    enforcement: {
      title: 'Auto enforcement',
      description: 'Upgrade to enable automatic enforcement and let us handle blocking across your playlists.',
      icon: 'shield',
    },
    export: {
      title: 'Data export',
      description: 'Upgrade to unlock data export and download your blocklist, analytics, and reports.',
      icon: 'download',
    },
  };

  function getMessage(): FeatureMessage {
    return featureMessages[feature] || {
      title: 'Premium feature',
      description: `Upgrade to access this feature.`,
      icon: 'star',
    };
  }

  function handleUpgrade() {
    navigateTo('pricing');
  }

  function handleDismiss() {
    dismissed = true;
  }

  function capitalize(s: string): string {
    return s.charAt(0).toUpperCase() + s.slice(1);
  }

  $: plan = $currentPlan;
  $: message = getMessage();
</script>

{#if !dismissed}
  <div data-testid="upgrade-prompt" class="upgrade-prompt">
    <div class="upgrade-prompt__content">
      <div class="upgrade-prompt__icon-wrapper">
        {#if message.icon === 'link'}
          <svg aria-hidden="true" class="upgrade-prompt__icon" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M12.586 4.586a2 2 0 112.828 2.828l-3 3a2 2 0 01-2.828 0 1 1 0 00-1.414 1.414 4 4 0 005.656 0l3-3a4 4 0 00-5.656-5.656l-1.5 1.5a1 1 0 101.414 1.414l1.5-1.5zm-5 5a2 2 0 012.828 0 1 1 0 101.414-1.414 4 4 0 00-5.656 0l-3 3a4 4 0 105.656 5.656l1.5-1.5a1 1 0 10-1.414-1.414l-1.5 1.5a2 2 0 11-2.828-2.828l3-3z" clip-rule="evenodd" />
          </svg>
        {:else if message.icon === 'search'}
          <svg aria-hidden="true" class="upgrade-prompt__icon" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z" clip-rule="evenodd" />
          </svg>
        {:else if message.icon === 'shield'}
          <svg aria-hidden="true" class="upgrade-prompt__icon" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M2.166 4.999A11.954 11.954 0 0010 1.944 11.954 11.954 0 0017.834 5c.11.65.166 1.32.166 2.001 0 5.225-3.34 9.67-8 11.317C5.34 16.67 2 12.225 2 7c0-.682.057-1.35.166-2.001zm11.541 3.708a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
          </svg>
        {:else if message.icon === 'download'}
          <svg aria-hidden="true" class="upgrade-prompt__icon" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M3 17a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zm3.293-7.707a1 1 0 011.414 0L9 10.586V3a1 1 0 112 0v7.586l1.293-1.293a1 1 0 111.414 1.414l-3 3a1 1 0 01-1.414 0l-3-3a1 1 0 010-1.414z" clip-rule="evenodd" />
          </svg>
        {:else}
          <svg aria-hidden="true" class="upgrade-prompt__icon" viewBox="0 0 20 20" fill="currentColor">
            <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
          </svg>
        {/if}
      </div>

      <div class="upgrade-prompt__text">
        <h3 class="upgrade-prompt__title">{message.title}</h3>
        <p class="upgrade-prompt__description">{message.description}</p>
        <p data-testid="current-plan-info" class="upgrade-prompt__plan">
          Current plan: <strong>{capitalize(plan)}</strong>
        </p>
      </div>
    </div>

    <div class="upgrade-prompt__actions">
      <button
        type="button"
        data-testid="upgrade-cta-btn"
        class="upgrade-prompt__btn upgrade-prompt__btn--primary"
        on:click={handleUpgrade}
      >
        Upgrade to Pro
      </button>
      <button
        type="button"
        data-testid="dismiss-btn"
        class="upgrade-prompt__btn upgrade-prompt__btn--ghost"
        on:click={handleDismiss}
      >
        Dismiss
      </button>
    </div>
  </div>
{/if}

<style>
  .upgrade-prompt {
    background: var(--color-bg-elevated, #18181b);
    border: 1px solid var(--color-border-default, #3f3f46);
    border-left: 3px solid var(--color-success);
    border-radius: 0.75rem;
    padding: 1.25rem 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  @media (min-width: 640px) {
    .upgrade-prompt {
      flex-direction: row;
      align-items: center;
      justify-content: space-between;
    }
  }

  .upgrade-prompt__content {
    display: flex;
    gap: 1rem;
    align-items: flex-start;
  }

  .upgrade-prompt__icon-wrapper {
    flex-shrink: 0;
    width: 2.5rem;
    height: 2.5rem;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #0d3b2e;
    border-radius: 0.5rem;
  }

  .upgrade-prompt__icon {
    width: 1.25rem;
    height: 1.25rem;
    color: var(--color-success);
  }

  .upgrade-prompt__text {
    flex: 1;
  }

  .upgrade-prompt__title {
    font-size: 0.9375rem;
    font-weight: 600;
    color: var(--color-text-primary, white);
    margin-bottom: 0.25rem;
  }

  .upgrade-prompt__description {
    font-size: 0.8125rem;
    color: var(--color-text-secondary, #a1a1aa);
    line-height: 1.5;
    margin-bottom: 0.25rem;
  }

  .upgrade-prompt__plan {
    font-size: 0.75rem;
    color: var(--color-text-tertiary, #71717a);
  }

  .upgrade-prompt__plan strong {
    color: var(--color-text-secondary, #a1a1aa);
  }

  .upgrade-prompt__actions {
    display: flex;
    gap: 0.5rem;
    flex-shrink: 0;
  }

  .upgrade-prompt__btn {
    padding: 0.5rem 1rem;
    border-radius: 0.375rem;
    font-size: 0.8125rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
    border: 1px solid transparent;
    white-space: nowrap;
  }

  .upgrade-prompt__btn--primary {
    background: var(--color-success);
    color: var(--color-text-inverse);
    border-color: var(--color-success);
  }

  .upgrade-prompt__btn--primary:hover {
    background: #059669;
  }

  .upgrade-prompt__btn--ghost {
    background: transparent;
    color: var(--color-text-tertiary, #71717a);
    border-color: transparent;
  }

  .upgrade-prompt__btn--ghost:hover {
    color: var(--color-text-secondary, #a1a1aa);
    background: var(--color-bg-hover);
  }

  .upgrade-prompt__btn:focus-visible,
  .upgrade-prompt__dismiss:focus-visible {
    outline: 2px solid var(--color-brand-primary, #10b981);
    outline-offset: 2px;
  }
</style>
