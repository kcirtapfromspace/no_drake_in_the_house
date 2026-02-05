<script lang="ts">
  import { onMount } from 'svelte';
  import { dnpActions, dnpStore } from '../stores/dnp';
  import ArtistSearchBar from './ArtistSearchBar.svelte';
  import BlockedArtistCard from './BlockedArtistCard.svelte';

  onMount(async () => {
    await dnpActions.fetchDnpList();
  });

  async function handleUnblockArtist(artistId: string) {
    const result = await dnpActions.removeArtist(artistId);
    if (!result.success) {
      alert(`Failed to unblock artist: ${result.message}`);
    }
  }

  // Ensure entries is always an array
  $: blockedArtists = ($dnpStore.entries && Array.isArray($dnpStore.entries)) ? $dnpStore.entries : [];
</script>

<div class="blocklist">
  <!-- Search bar (sticky) -->
  <ArtistSearchBar />

  <!-- Main content -->
  <div class="blocklist__content">
    <!-- Header -->
    <div class="blocklist__header">
      <h1 class="blocklist__title">Your Blocklist</h1>
      <p class="blocklist__count">
        {blockedArtists.length} artist{blockedArtists.length !== 1 ? 's' : ''} blocked
      </p>
    </div>

    <!-- Loading state -->
    {#if $dnpStore.isLoading}
      <div class="blocklist__skeleton-list">
        {#each [1, 2, 3, 4] as _}
          <div class="blocklist__skeleton-card">
            <div class="blocklist__skeleton-row">
              <div class="blocklist__skeleton-avatar"></div>
              <div class="blocklist__skeleton-text">
                <div class="blocklist__skeleton-line blocklist__skeleton-line--lg"></div>
                <div class="blocklist__skeleton-line blocklist__skeleton-line--md"></div>
                <div class="blocklist__skeleton-line blocklist__skeleton-line--sm"></div>
              </div>
              <div class="blocklist__skeleton-btn"></div>
            </div>
          </div>
        {/each}
      </div>
    {:else if $dnpStore.error}
      <!-- Error state -->
      <div class="blocklist__error">
        <div class="blocklist__error-inner">
          <svg class="blocklist__error-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <div>
            <h3 class="blocklist__error-title">Error Loading Blocklist</h3>
            <p class="blocklist__error-msg">{$dnpStore.error}</p>
            <button type="button"
              on:click={() => dnpActions.fetchDnpList()}
              class="blocklist__retry"
            >
              Try again
            </button>
          </div>
        </div>
      </div>
    {:else if blockedArtists.length === 0}
      <!-- Empty state -->
      <div class="blocklist__empty">
        <div class="blocklist__empty-icon-wrap">
          <svg class="blocklist__empty-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728L5.636 5.636m12.728 12.728L5.636 5.636" />
          </svg>
        </div>
        <h3 class="blocklist__empty-title">No Artists Blocked Yet</h3>
        <p class="blocklist__empty-text">
          Start building your blocklist by searching for artists you want to avoid in the search bar above.
        </p>
        <div class="blocklist__empty-hint">
          <svg fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7h8m0 0v8m0-8l-8 8-4-4-6 6" />
          </svg>
          <span>Search above to get started</span>
        </div>
      </div>
    {:else}
      <!-- Blocked artists feed -->
      <div class="blocklist__feed">
        {#each blockedArtists as entry (entry.artist.id)}
          <BlockedArtistCard
            artist={entry.artist}
            blockedAt={entry.created_at}
            tags={entry.tags || []}
            note={entry.note}
            onUnblock={() => handleUnblockArtist(entry.artist.id)}
          />
        {/each}
      </div>

      <!-- Load more placeholder -->
      {#if blockedArtists.length >= 20}
        <div class="blocklist__load-more">
          <button type="button" class="blocklist__load-more-btn">
            Load More Artists
          </button>
        </div>
      {/if}
    {/if}
  </div>
</div>

<style>
  .blocklist {
    min-height: 100vh;
    background-color: var(--color-bg-page);
  }

  .blocklist__content {
    max-width: 42rem;
    margin: 0 auto;
    padding: 1.5rem 1rem;
  }

  .blocklist__header {
    margin-bottom: 1.5rem;
  }

  .blocklist__title {
    font-size: var(--text-2xl);
    font-weight: 700;
    color: var(--color-text-primary);
    letter-spacing: -0.02em;
  }

  .blocklist__count {
    font-size: var(--text-sm);
    color: var(--color-text-tertiary);
    margin-top: 0.25rem;
  }

  /* Skeleton loading */
  .blocklist__skeleton-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .blocklist__skeleton-card {
    background-color: var(--color-bg-elevated);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-xl);
    padding: 1rem;
  }

  .blocklist__skeleton-row {
    display: flex;
    align-items: flex-start;
    gap: 0.875rem;
  }

  .blocklist__skeleton-avatar {
    width: 3.5rem;
    height: 3.5rem;
    border-radius: var(--radius-lg);
    background: linear-gradient(90deg, var(--color-bg-interactive) 25%, var(--color-bg-hover) 50%, var(--color-bg-interactive) 75%);
    background-size: 200% 100%;
    animation: skeleton-pulse 1.5s infinite;
    flex-shrink: 0;
  }

  .blocklist__skeleton-text {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .blocklist__skeleton-line {
    height: 0.75rem;
    border-radius: var(--radius-sm);
    background: linear-gradient(90deg, var(--color-bg-interactive) 25%, var(--color-bg-hover) 50%, var(--color-bg-interactive) 75%);
    background-size: 200% 100%;
    animation: skeleton-pulse 1.5s infinite;
  }

  .blocklist__skeleton-line--lg { width: 8rem; }
  .blocklist__skeleton-line--md { width: 6rem; }
  .blocklist__skeleton-line--sm { width: 4rem; }

  .blocklist__skeleton-btn {
    width: 4rem;
    height: 1.75rem;
    border-radius: var(--radius-md);
    background: linear-gradient(90deg, var(--color-bg-interactive) 25%, var(--color-bg-hover) 50%, var(--color-bg-interactive) 75%);
    background-size: 200% 100%;
    animation: skeleton-pulse 1.5s infinite;
    flex-shrink: 0;
  }

  @keyframes skeleton-pulse {
    0% { background-position: 200% 0; }
    100% { background-position: -200% 0; }
  }

  /* Error state */
  .blocklist__error {
    background-color: var(--color-error-muted);
    border: 1px solid var(--color-error);
    border-radius: var(--radius-xl);
    padding: 1.25rem;
  }

  .blocklist__error-inner {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
  }

  .blocklist__error-icon {
    width: 1.25rem;
    height: 1.25rem;
    max-width: none;
    max-height: none;
    color: var(--color-error);
    flex-shrink: 0;
    margin-top: 0.125rem;
  }

  .blocklist__error-title {
    font-size: var(--text-base);
    font-weight: 600;
    color: var(--color-error);
  }

  .blocklist__error-msg {
    font-size: var(--text-sm);
    color: var(--color-text-secondary);
    margin-top: 0.25rem;
  }

  .blocklist__retry {
    margin-top: 0.5rem;
    font-family: var(--font-family-sans);
    font-size: var(--text-sm);
    color: var(--color-brand-primary);
    background: none;
    border: none;
    cursor: pointer;
    text-decoration: underline;
    padding: 0;
  }

  .blocklist__retry:hover {
    color: var(--color-brand-primary-hover);
  }

  /* Empty state */
  .blocklist__empty {
    background-color: var(--color-bg-elevated);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-2xl);
    padding: 3rem 1.5rem;
    text-align: center;
  }

  .blocklist__empty-icon-wrap {
    margin-bottom: 1rem;
  }

  .blocklist__empty-icon {
    width: 3rem;
    height: 3rem;
    max-width: none;
    max-height: none;
    color: var(--color-text-muted);
    margin: 0 auto;
  }

  .blocklist__empty-title {
    font-size: var(--text-xl);
    font-weight: 600;
    color: var(--color-text-primary);
    margin-bottom: 0.5rem;
  }

  .blocklist__empty-text {
    font-size: var(--text-sm);
    color: var(--color-text-tertiary);
    max-width: 24rem;
    margin: 0 auto 1.5rem;
    line-height: 1.6;
  }

  .blocklist__empty-hint {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }

  .blocklist__empty-hint svg {
    width: 0.875rem;
    height: 0.875rem;
    max-width: none;
    max-height: none;
  }

  /* Feed */
  .blocklist__feed {
    display: flex;
    flex-direction: column;
    gap: 0.625rem;
  }

  /* Virtual rendering: skip layout/paint for off-screen cards */
  .blocklist__feed > :global(*) {
    content-visibility: auto;
    contain-intrinsic-size: auto 5.5rem;
  }

  /* Load more */
  .blocklist__load-more {
    margin-top: 2rem;
    text-align: center;
  }

  .blocklist__load-more-btn {
    padding: 0.625rem 1.5rem;
    font-family: var(--font-family-sans);
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--color-text-secondary);
    background-color: var(--color-bg-interactive);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-lg);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .blocklist__load-more-btn:hover {
    background-color: var(--color-bg-hover);
    border-color: var(--color-border-hover);
    color: var(--color-text-primary);
  }
</style>
