<script lang="ts">
  export let artist: {
    id: string;
    canonical_name: string;
    metadata?: {
      image?: string;
      genres?: string[];
    };
  };
  export let blockedAt: string;
  export let tags: string[] = [];
  export let note: string | undefined = undefined;
  export let onUnblock: () => void;

  let isUnblocking = false;
  let confirming = false;

  function requestUnblock() {
    confirming = true;
  }

  function cancelUnblock() {
    confirming = false;
  }

  async function confirmUnblock() {
    isUnblocking = true;
    confirming = false;
    try {
      await onUnblock();
    } finally {
      isUnblocking = false;
    }
  }

  function formatDate(dateString: string) {
    const date = new Date(dateString);
    const now = new Date();
    const diffTime = Math.abs(now.getTime() - date.getTime());
    const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24));

    if (diffDays === 1) return 'Yesterday';
    if (diffDays < 7) return `${diffDays} days ago`;
    if (diffDays < 30) return `${Math.ceil(diffDays / 7)} weeks ago`;
    return date.toLocaleDateString();
  }
</script>

<div class="card">
  <div class="card__inner">
    <!-- Artist image -->
    <div class="card__avatar" style="position: relative; overflow: hidden;">
      {#if artist.metadata?.image}
        <img
          src={artist.metadata.image}
          alt=""
          class="card__img"
          style="position: absolute; inset: 0;"
          on:error={(e) => { e.currentTarget.style.display = 'none'; }}
        />
      {/if}
      <div class="card__placeholder">
        <svg class="card__placeholder-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3" />
        </svg>
      </div>
    </div>

    <!-- Artist info -->
    <div class="card__body">
      <div class="card__header">
        <div class="card__info">
          <h3 class="card__name">{artist.canonical_name}</h3>

          {#if artist.metadata?.genres && artist.metadata.genres.length > 0}
            <p class="card__genres">
              {artist.metadata.genres.slice(0, 3).join(' \u00B7 ')}
            </p>
          {/if}

          <p class="card__date">
            Blocked {formatDate(blockedAt)}
          </p>
        </div>

        <!-- Unblock button with inline confirmation -->
        {#if confirming}
          <div class="card__confirm">
            <button type="button" on:click={confirmUnblock} class="card__confirm-yes">Yes</button>
            <button type="button" on:click={cancelUnblock} class="card__confirm-no">No</button>
          </div>
        {:else}
          <button type="button"
            on:click={requestUnblock}
            disabled={isUnblocking}
            class="card__unblock"
          >
            {#if isUnblocking}
              <svg class="card__spinner" fill="none" viewBox="0 0 24 24">
                <circle class="card__spinner-track" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="card__spinner-arc" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
            {:else}
              Unblock
            {/if}
          </button>
        {/if}
      </div>

      <!-- Tags -->
      {#if tags && tags.length > 0}
        <div class="card__tags">
          {#each tags as tag}
            <span class="card__tag">{tag}</span>
          {/each}
        </div>
      {/if}

      <!-- Note -->
      {#if note}
        <p class="card__note">"{note}"</p>
      {/if}
    </div>
  </div>
</div>

<style>
  .card {
    background-color: var(--color-bg-elevated);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-xl);
    padding: 1rem;
    transition: border-color var(--transition-fast), box-shadow var(--transition-fast);
  }

  .card:hover {
    border-color: var(--color-border-hover);
    box-shadow: var(--shadow-md);
  }

  .card__inner {
    display: flex;
    align-items: flex-start;
    gap: 0.875rem;
  }

  .card__avatar {
    flex-shrink: 0;
  }

  .card__img {
    width: 3.5rem;
    height: 3.5rem;
    border-radius: var(--radius-lg);
    object-fit: cover;
  }

  .card__placeholder {
    width: 3.5rem;
    height: 3.5rem;
    border-radius: var(--radius-lg);
    background-color: var(--color-bg-interactive);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .card__placeholder-icon {
    width: 1.5rem;
    height: 1.5rem;
    max-width: none;
    max-height: none;
    color: var(--color-text-muted);
  }

  .card__body {
    flex: 1;
    min-width: 0;
  }

  .card__header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 0.75rem;
  }

  .card__info {
    flex: 1;
    min-width: 0;
  }

  .card__name {
    font-size: var(--text-base);
    font-weight: 600;
    color: var(--color-text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.3;
  }

  .card__genres {
    font-size: var(--text-xs);
    color: var(--color-text-tertiary);
    margin-top: 0.1875rem;
  }

  .card__date {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    margin-top: 0.375rem;
  }

  .card__unblock {
    flex-shrink: 0;
    padding: 0.375rem 0.75rem;
    font-size: var(--text-xs);
    font-weight: 500;
    font-family: var(--font-family-sans);
    color: var(--color-brand-primary);
    background-color: var(--color-brand-primary-muted);
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--transition-fast);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 4.5rem;
  }

  .card__unblock:hover:not(:disabled) {
    background-color: var(--color-brand-primary);
    color: white;
  }

  .card__unblock:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .card__confirm {
    display: flex;
    gap: 0.25rem;
    flex-shrink: 0;
  }

  .card__confirm-yes {
    padding: 0.375rem 0.625rem;
    font-size: var(--text-xs);
    font-weight: 600;
    font-family: var(--font-family-sans);
    color: white;
    background-color: var(--color-brand-primary);
    border: none;
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: background-color var(--transition-fast);
  }

  .card__confirm-yes:hover {
    background-color: var(--color-brand-primary-hover);
  }

  .card__confirm-no {
    padding: 0.375rem 0.625rem;
    font-size: var(--text-xs);
    font-weight: 500;
    font-family: var(--font-family-sans);
    color: var(--color-text-secondary);
    background-color: var(--color-bg-interactive);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .card__confirm-no:hover {
    background-color: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .card__spinner {
    width: 1rem;
    height: 1rem;
    max-width: none;
    max-height: none;
    animation: spin 1s linear infinite;
  }

  .card__spinner-track {
    opacity: 0.25;
  }

  .card__spinner-arc {
    opacity: 0.75;
  }

  .card__tags {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
    margin-top: 0.5rem;
  }

  .card__tag {
    display: inline-flex;
    align-items: center;
    padding: 0.125rem 0.5rem;
    border-radius: var(--radius-full);
    font-size: 0.6875rem;
    font-weight: 500;
    color: var(--color-text-secondary);
    background-color: var(--color-bg-interactive);
  }

  .card__note {
    font-size: var(--text-xs);
    color: var(--color-text-tertiary);
    margin-top: 0.5rem;
    font-style: italic;
    line-height: 1.5;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
