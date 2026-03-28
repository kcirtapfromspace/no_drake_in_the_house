<script lang="ts">
  export let provider: string;
  export let displayName: string;
  export let description: string;
  export let status: 'connected' | 'disconnected' | 'error' | 'connecting';
  export let lastConnected: string | undefined = undefined;
  export let errorMessage: string | undefined = undefined;
  export let onConnect: () => void;
  export let onDisconnect: () => void;

  function getStatusText(status: string) {
    switch (status) {
      case 'connected': return 'Connected';
      case 'connecting': return 'Connecting...';
      case 'error': return 'Error';
      default: return 'Not Connected';
    }
  }

  function getProviderIcon(provider: string) {
    switch (provider) {
      case 'spotify':
        return `<svg class="conn-icon__svg" fill="currentColor" viewBox="0 0 24 24">
          <path d="M12 0C5.4 0 0 5.4 0 12s5.4 12 12 12 12-5.4 12-12S18.66 0 12 0zm5.521 17.34c-.24.359-.66.48-1.021.24-2.82-1.74-6.36-2.101-10.561-1.141-.418.122-.779-.179-.899-.539-.12-.421.18-.78.54-.9 4.56-1.021 8.52-.6 11.64 1.32.42.18.479.659.301 1.02zm1.44-3.3c-.301.42-.841.6-1.262.3-3.239-1.98-8.159-2.58-11.939-1.38-.479.12-1.02-.12-1.14-.6-.12-.48.12-1.021.6-1.141C9.6 9.9 15 10.561 18.72 12.84c.361.181.54.78.241 1.2zm.12-3.36C15.24 8.4 8.82 8.16 5.16 9.301c-.6.179-1.2-.181-1.38-.721-.18-.601.18-1.2.72-1.381 4.26-1.26 11.28-1.02 15.721 1.621.539.3.719 1.02.42 1.56-.299.421-1.02.599-1.559.3z"/>
        </svg>`;
      case 'apple':
        return `<svg class="conn-icon__svg" fill="currentColor" viewBox="0 0 24 24">
          <path d="M18.71 19.5c-.83 1.24-1.71 2.45-3.05 2.47-1.34.03-1.77-.79-3.29-.79-1.53 0-2 .77-3.27.82-1.31.05-2.3-1.32-3.14-2.53C4.25 17 2.94 12.45 4.7 9.39c.87-1.52 2.43-2.48 4.12-2.51 1.28-.02 2.5.87 3.29.87.78 0 2.26-1.07 3.81-.91.65.03 2.47.26 3.64 1.98-.09.06-2.17 1.28-2.15 3.81.03 3.02 2.65 4.03 2.68 4.04-.03.07-.42 1.44-1.38 2.83M13 3.5c.73-.83 1.94-1.46 2.94-1.5.13 1.17-.34 2.35-1.04 3.19-.69.85-1.83 1.51-2.95 1.42-.15-1.15.41-2.35 1.05-3.11z"/>
        </svg>`;
      case 'youtube':
        return `<svg class="conn-icon__svg" fill="currentColor" viewBox="0 0 24 24">
          <path d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z"/>
        </svg>`;
      default:
        return `<svg class="conn-icon__svg" fill="currentColor" viewBox="0 0 24 24">
          <path d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1"/>
        </svg>`;
    }
  }

  function formatDate(dateString: string) {
    return new Date(dateString).toLocaleDateString();
  }
</script>

<div class="conn-card">
  <div class="conn-card__header">
    <div class="conn-card__info">
      <div class="conn-icon">
        {@html getProviderIcon(provider)}
      </div>
      <div>
        <h3 class="conn-card__name">{displayName}</h3>
        <p class="conn-card__desc">{description}</p>
      </div>
    </div>

    <span class="conn-status conn-status--{status}">
      {getStatusText(status)}
    </span>
  </div>

  {#if status === 'connected' && lastConnected}
    <div class="conn-card__detail">
      Connected on {formatDate(lastConnected)}
    </div>
  {/if}

  {#if status === 'error' && errorMessage}
    <div class="conn-card__error">
      <p>{errorMessage}</p>
    </div>
  {/if}

  <div class="conn-card__actions">
    {#if status === 'connected'}
      <button on:click={onDisconnect} class="conn-btn conn-btn--danger">
        Disconnect
      </button>
      <button on:click={onConnect} class="conn-btn conn-btn--secondary">
        Refresh
      </button>
    {:else if status === 'connecting'}
      <button disabled class="conn-btn conn-btn--info conn-btn--loading">
        <svg class="conn-spinner" fill="none" viewBox="0 0 24 24" aria-hidden="true">
          <circle class="conn-spinner__track" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="conn-spinner__head" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
        Connecting...
      </button>
    {:else}
      <button on:click={onConnect} class="conn-btn conn-btn--primary">
        Connect {displayName}
      </button>
      {#if status === 'error'}
        <button on:click={onConnect} class="conn-btn conn-btn--secondary">
          Retry
        </button>
      {/if}
    {/if}
  </div>
</div>

<style>
  .conn-card {
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-xl);
    padding: var(--space-6);
    box-shadow: var(--shadow-card);
    transition: box-shadow var(--transition-fast);
  }

  .conn-card:hover {
    box-shadow: var(--shadow-md);
  }

  .conn-card__header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    margin-bottom: var(--space-4);
  }

  .conn-card__info {
    display: flex;
    align-items: center;
  }

  .conn-icon {
    color: var(--color-text-secondary);
    margin-right: var(--space-3);
  }

  .conn-icon :global(.conn-icon__svg) {
    width: 2rem;
    height: 2rem;
  }

  .conn-card__name {
    font-size: var(--text-lg);
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .conn-card__desc {
    font-size: var(--text-sm);
    color: var(--color-text-tertiary);
  }

  .conn-status {
    display: inline-flex;
    align-items: center;
    padding: 0.125rem var(--space-2);
    border-radius: var(--radius-full);
    font-size: var(--text-xs);
    font-weight: 500;
  }

  .conn-status--connected {
    color: var(--color-success);
    background: var(--color-success-muted);
  }

  .conn-status--connecting {
    color: var(--color-info);
    background: var(--color-info-muted);
  }

  .conn-status--error {
    color: var(--color-error);
    background: var(--color-error-muted);
  }

  .conn-status--disconnected {
    color: var(--color-text-tertiary);
    background: var(--color-bg-interactive);
  }

  .conn-card__detail {
    margin-bottom: var(--space-4);
    font-size: var(--text-sm);
    color: var(--color-text-tertiary);
  }

  .conn-card__error {
    margin-bottom: var(--space-4);
    padding: var(--space-3);
    background: var(--color-error-muted);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-lg);
    font-size: var(--text-sm);
    color: var(--color-error);
  }

  .conn-card__actions {
    display: flex;
    gap: var(--space-3);
  }

  .conn-btn {
    flex: 1;
    padding: var(--space-2) var(--space-4);
    border-radius: var(--radius-lg);
    font-size: var(--text-sm);
    font-weight: 500;
    border: none;
    cursor: pointer;
    transition: all var(--transition-fast);
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .conn-btn:focus-visible {
    outline: 2px solid var(--color-brand-primary);
    outline-offset: 2px;
  }

  .conn-btn--primary {
    background: var(--color-brand-primary);
    color: white;
  }

  .conn-btn--primary:hover {
    background: var(--color-brand-primary-hover);
  }

  .conn-btn--danger {
    background: var(--color-error);
    color: white;
  }

  .conn-btn--danger:hover {
    opacity: 0.9;
  }

  .conn-btn--info {
    background: var(--color-info);
    color: white;
  }

  .conn-btn--secondary {
    flex: none;
    background: transparent;
    color: var(--color-text-secondary);
    border: 1px solid var(--color-border-default);
  }

  .conn-btn--secondary:hover {
    background: var(--color-bg-hover);
  }

  .conn-btn--loading {
    opacity: 0.75;
    cursor: not-allowed;
  }

  .conn-spinner {
    width: 1rem;
    height: 1rem;
    margin-right: var(--space-2);
    animation: spin 1s linear infinite;
  }

  .conn-spinner__track {
    opacity: 0.25;
  }

  .conn-spinner__head {
    opacity: 0.75;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
