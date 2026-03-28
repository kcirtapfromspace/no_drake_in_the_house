<script lang="ts">
  import { dnpActions, dnpStore } from '../stores/dnp';

  function hideImgOnError(e: Event) { (e.currentTarget as HTMLImageElement).style.display = 'none'; }

  let searchQuery = '';
  let searchTimeout: any;
  let isAddingArtist = false;
  let activeIndex = -1;

  function handleKeydown(event: KeyboardEvent) {
    const results = $dnpStore.searchResults;
    if (!results || results.length === 0) return;

    if (event.key === 'ArrowDown') {
      event.preventDefault();
      activeIndex = (activeIndex + 1) % results.length;
    } else if (event.key === 'ArrowUp') {
      event.preventDefault();
      activeIndex = activeIndex <= 0 ? results.length - 1 : activeIndex - 1;
    } else if (event.key === 'Enter' && activeIndex >= 0 && activeIndex < results.length) {
      event.preventDefault();
      handleBlockArtist(results[activeIndex]);
    } else if (event.key === 'Escape') {
      clearSearch();
    }
  }

  // Reset active index when results change
  $: if ($dnpStore.searchResults) activeIndex = -1;

  // Debounced search
  $: {
    if (searchTimeout) clearTimeout(searchTimeout);
    searchTimeout = setTimeout(() => {
      if (searchQuery.trim()) {
        dnpActions.searchArtists(searchQuery);
      } else {
        dnpActions.clearSearch();
      }
    }, 300);
  }

  async function handleBlockArtist(artist: any) {
    isAddingArtist = true;
    try {
      const result = await dnpActions.addArtist(artist.canonical_name, [], '');
      if (result.success) {
        searchQuery = '';
        dnpActions.clearSearch();
      } else {
        alert(`Failed to block artist: ${result.message}`);
      }
    } finally {
      isAddingArtist = false;
    }
  }

  function clearSearch() {
    searchQuery = '';
    dnpActions.clearSearch();
  }
</script>

<div class="search-bar">
  <!-- Search input -->
  <div class="search-bar__field">
    <div class="search-bar__icon">
      <svg fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
      </svg>
    </div>

    <input
      bind:value={searchQuery}
      on:keydown={handleKeydown}
      type="text"
      placeholder="Search artists to block..."
      aria-label="Search for artists to block"
      aria-autocomplete="list"
      aria-controls={searchQuery && $dnpStore.searchResults.length > 0 ? 'search-results-list' : undefined}
      aria-activedescendant={activeIndex >= 0 ? `search-result-${activeIndex}` : undefined}
      role="combobox"
      aria-expanded={!!searchQuery && $dnpStore.searchResults.length > 0}
      class="search-bar__input"
    />

    {#if searchQuery}
      <button type="button"
        on:click={clearSearch}
        class="search-bar__clear"
        aria-label="Clear search"
      >
        <svg fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    {/if}
  </div>

  <!-- Search results -->
  {#if searchQuery && ($dnpStore.searchResults.length > 0 || $dnpStore.isSearching)}
    <div class="search-bar__results">
      {#if $dnpStore.isSearching}
        <div class="search-bar__loading">
          <svg class="search-bar__spinner" fill="none" viewBox="0 0 24 24">
            <circle class="search-bar__spinner-track" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="search-bar__spinner-arc" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          <span>Searching...</span>
        </div>
      {:else if $dnpStore.searchResults.length > 0}
        <div id="search-results-list" role="listbox" aria-label="Search results">
        {#each $dnpStore.searchResults as artist, i}
          <div
            class="search-bar__result"
            class:search-bar__result--active={i === activeIndex}
            id="search-result-{i}"
            role="option"
            aria-selected={i === activeIndex}
          >
            <div class="search-bar__result-info">
              <div class="search-bar__result-placeholder" style="position: relative; overflow: hidden;">
                {#if artist.metadata?.image}
                  <img
                    src={artist.metadata.image}
                    alt=""
                    class="search-bar__result-img"
                    style="position: absolute; inset: 0;"
                    on:error={hideImgOnError}
                  />
                {/if}
                <svg fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3" />
                </svg>
              </div>

              <div class="search-bar__result-text">
                <span class="search-bar__result-name">{artist.canonical_name}</span>
                {#if artist.metadata?.genres && artist.metadata.genres.length > 0}
                  <span class="search-bar__result-genres">
                    {artist.metadata.genres.slice(0, 2).join(', ')}
                  </span>
                {/if}
              </div>
            </div>

            <button type="button"
              on:click={() => handleBlockArtist(artist)}
              disabled={isAddingArtist}
              class="search-bar__block-btn"
            >
              {#if isAddingArtist}
                <svg class="search-bar__spinner search-bar__spinner--sm" fill="none" viewBox="0 0 24 24">
                  <circle class="search-bar__spinner-track" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                  <path class="search-bar__spinner-arc" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
              {:else}
                Block
              {/if}
            </button>
          </div>
        {/each}
        </div>
      {:else}
        <div class="search-bar__empty">
          <p>No artists found for "{searchQuery}"</p>
          <p class="search-bar__empty-hint">Try a different search term</p>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .search-bar {
    position: sticky;
    top: 0;
    background-color: var(--color-bg-elevated);
    border-bottom: 1px solid var(--color-border-default);
    padding: 0.75rem 1rem;
    z-index: var(--z-sticky);
  }

  .search-bar__field {
    position: relative;
  }

  .search-bar__icon {
    position: absolute;
    inset: 0 auto 0 0;
    padding-left: 0.75rem;
    display: flex;
    align-items: center;
    pointer-events: none;
    color: var(--color-text-muted);
  }

  .search-bar__icon svg {
    width: 1.125rem;
    height: 1.125rem;
    max-width: none;
    max-height: none;
  }

  .search-bar__input {
    display: block;
    width: 100%;
    padding: 0.625rem 2.5rem 0.625rem 2.5rem;
    font-family: var(--font-family-sans);
    font-size: var(--text-base);
    color: var(--color-text-primary);
    background-color: var(--input-bg);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-xl);
    line-height: 1.4;
    transition: border-color var(--transition-fast), box-shadow var(--transition-fast);
  }

  .search-bar__input::placeholder {
    color: var(--color-text-muted);
  }

  .search-bar__input:focus {
    outline: none;
    border-color: var(--color-brand-primary);
    box-shadow: 0 0 0 3px var(--color-brand-primary-muted);
  }

  .search-bar__clear {
    position: absolute;
    inset: 0 0 0 auto;
    padding-right: 0.75rem;
    display: flex;
    align-items: center;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--color-text-muted);
    transition: color var(--transition-fast);
  }

  .search-bar__clear:hover {
    color: var(--color-text-secondary);
  }

  .search-bar__clear svg {
    width: 1.125rem;
    height: 1.125rem;
    max-width: none;
    max-height: none;
  }

  .search-bar__results {
    margin-top: 0.5rem;
    background-color: var(--color-bg-elevated);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-xl);
    box-shadow: var(--shadow-dropdown);
    max-height: 22rem;
    overflow-y: auto;
  }

  .search-bar__loading {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 1.25rem;
    color: var(--color-text-tertiary);
    font-size: var(--text-sm);
  }

  .search-bar__spinner {
    width: 1.25rem;
    height: 1.25rem;
    max-width: none;
    max-height: none;
    animation: spin 1s linear infinite;
  }

  .search-bar__spinner--sm {
    width: 1rem;
    height: 1rem;
  }

  .search-bar__spinner-track {
    opacity: 0.25;
  }

  .search-bar__spinner-arc {
    opacity: 0.75;
  }

  .search-bar__result {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--color-border-subtle);
    transition: background-color var(--transition-fast);
  }

  .search-bar__result:last-child {
    border-bottom: none;
  }

  .search-bar__result:hover,
  .search-bar__result--active {
    background-color: var(--color-bg-hover);
  }

  .search-bar__result-info {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    min-width: 0;
    flex: 1;
  }

  .search-bar__result-img {
    width: 2.5rem;
    height: 2.5rem;
    border-radius: var(--radius-md);
    object-fit: cover;
    flex-shrink: 0;
  }

  .search-bar__result-placeholder {
    width: 2.5rem;
    height: 2.5rem;
    border-radius: var(--radius-md);
    background-color: var(--color-bg-interactive);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .search-bar__result-placeholder svg {
    width: 1.25rem;
    height: 1.25rem;
    max-width: none;
    max-height: none;
    color: var(--color-text-muted);
  }

  .search-bar__result-text {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .search-bar__result-name {
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--color-text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .search-bar__result-genres {
    font-size: var(--text-xs);
    color: var(--color-text-tertiary);
    margin-top: 0.125rem;
  }

  .search-bar__block-btn {
    flex-shrink: 0;
    margin-left: 0.75rem;
    padding: 0.375rem 0.875rem;
    font-family: var(--font-family-sans);
    font-size: var(--text-xs);
    font-weight: 600;
    color: white;
    background-color: var(--color-brand-primary);
    border: none;
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--transition-fast);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 3.5rem;
  }

  .search-bar__block-btn:hover:not(:disabled) {
    background-color: var(--color-brand-primary-hover);
    transform: translateY(-1px);
    box-shadow: 0 2px 8px rgba(244, 63, 94, 0.25);
  }

  .search-bar__block-btn:active:not(:disabled) {
    transform: translateY(0);
  }

  .search-bar__block-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .search-bar__empty {
    padding: 1.5rem;
    text-align: center;
    color: var(--color-text-tertiary);
    font-size: var(--text-sm);
  }

  .search-bar__empty-hint {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    margin-top: 0.25rem;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
