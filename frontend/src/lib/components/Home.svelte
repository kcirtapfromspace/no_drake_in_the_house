<script lang="ts">
  import { onMount } from 'svelte';
  import { apiClient } from '../utils/api-client';
  import { navigateTo, navigateToArtist } from '../utils/simple-router';
  import { blockingStore, type Platform } from '../stores/blocking';
  import { spotifyConnection, appleMusicConnection, connectionActions } from '../stores/connections';
  import EnforcementBadges from './EnforcementBadges.svelte';

  interface CategoryList {
    id: string;
    name: string;
    description: string;
    artist_count: number;
    subscribed: boolean;
  }

  interface SearchArtist {
    id: string;
    canonical_name: string;
    genres?: string[];
    image_url?: string;
    offense_count?: number;
    has_offenses?: boolean;
    source?: string;
  }

  interface SearchResponse {
    artists: SearchArtist[];
    total: number;
    sources?: { local: number; catalog: number };
  }

  interface BlockedArtist {
    id: string;
    name: string;
    category: string;
    severity: string;
  }

  let searchQuery = '';
  let searchResults: SearchArtist[] = [];
  let isSearching = false;
  let searchTimeout: ReturnType<typeof setTimeout>;

  let categoryLists: CategoryList[] = [];
  let isLoadingCategories = true;
  let categoriesError: string | null = null;
  let expandedCategoryId: string | null = null;
  let categoryArtists: BlockedArtist[] = [];
  let isLoadingCategoryArtists = false;

  let blockedArtists: BlockedArtist[] = [];
  let isLoadingBlocked = false;
  let blockedError: string | null = null;

  let dnpList: Set<string> = new Set();
  let exceptedArtists: Set<string> = new Set();

  function extractArray<T>(value: unknown, keys: string[] = []): T[] {
    if (Array.isArray(value)) {
      return value as T[];
    }

    if (value && typeof value === 'object') {
      for (const key of keys) {
        const nested = (value as Record<string, unknown>)[key];
        if (Array.isArray(nested)) {
          return nested as T[];
        }
      }
    }

    return [];
  }

  function normalizeCategoryLists(value: unknown): CategoryList[] {
    return extractArray<CategoryList>(value, ['categories', 'entries', 'items', 'data']);
  }

  function normalizeArtists(value: unknown): BlockedArtist[] {
    return extractArray<BlockedArtist>(value, ['artists', 'blocked_artists', 'entries', 'items', 'data']);
  }

  function normalizeDnpArtistIds(value: unknown): string[] {
    return extractArray<{ artist_id?: string; id?: string }>(value, ['entries', 'artists', 'items', 'data'])
      .map((item) => item.artist_id || item.id || '')
      .filter(Boolean);
  }

  function loadExceptions() {
    try {
      const stored = localStorage.getItem('exceptedArtists');
      if (stored) {
        exceptedArtists = new Set(JSON.parse(stored));
      }
    } catch (e) {
      console.error('Failed to load exceptions:', e);
    }
  }

  function saveExceptions() {
    try {
      localStorage.setItem('exceptedArtists', JSON.stringify([...exceptedArtists]));
    } catch (e) {
      console.error('Failed to save exceptions:', e);
    }
  }

  $: uniqueBlockedArtists = (Array.isArray(blockedArtists) ? blockedArtists : []).reduce((acc, artist) => {
    if (!acc.some(a => a.id === artist.id) && !exceptedArtists.has(artist.id)) {
      acc.push(artist);
    }
    return acc;
  }, [] as BlockedArtist[]);
  $: activeCategoryCount = (Array.isArray(categoryLists) ? categoryLists : []).filter(category => category.subscribed).length;
  $: connectedPlatformCount = getConnectedPlatforms().length;

  $: exceptedFromCategories = (Array.isArray(blockedArtists) ? blockedArtists : []).filter(a => exceptedArtists.has(a.id))
    .reduce((acc, artist) => {
      if (!acc.some(a => a.id === artist.id)) {
        acc.push(artist);
      }
      return acc;
    }, [] as BlockedArtist[]);

  const categoryColors: Record<string, { icon: string; bg: string }> = {
    domestic_violence: { icon: '#F43F5E', bg: 'rgba(244, 63, 94, 0.15)' },
    sexual_misconduct: { icon: '#EC4899', bg: 'rgba(236, 72, 153, 0.15)' },
    certified_creeper: { icon: '#8B5CF6', bg: 'rgba(139, 92, 246, 0.15)' },
    hate_speech: { icon: '#A855F7', bg: 'rgba(168, 85, 247, 0.15)' },
    racism: { icon: '#F97316', bg: 'rgba(249, 115, 22, 0.15)' },
    antisemitism: { icon: '#EAB308', bg: 'rgba(234, 179, 8, 0.15)' },
    financial_crimes: { icon: '#10B981', bg: 'rgba(16, 185, 129, 0.15)' },
    substance_abuse: { icon: '#06B6D4', bg: 'rgba(6, 182, 212, 0.15)' },
    violence: { icon: '#EF4444', bg: 'rgba(239, 68, 68, 0.15)' },
    default: { icon: '#6B7280', bg: 'rgba(107, 114, 128, 0.15)' }
  };

  function getCategoryColor(categoryId: string): { icon: string; bg: string } {
    return categoryColors[categoryId] || categoryColors.default;
  }

  function formatCategoryName(id: string): string {
    return id.split('_').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ');
  }

  function getSeverityStyle(severity: string): { label: string; color: string } {
    switch (severity) {
      case 'egregious': return { label: 'Egregious', color: '#f87171' };
      case 'severe': return { label: 'Severe', color: '#fb923c' };
      case 'moderate': return { label: 'Moderate', color: '#fbbf24' };
      default: return { label: 'Minor', color: 'var(--color-text-tertiary)' };
    }
  }

  onMount(async () => {
    loadExceptions();
    await Promise.all([
      loadCategories(),
      loadBlockedArtists(),
      loadDnpList(),
      connectionActions.fetchConnections(),
    ]);
  });

  async function loadCategories() {
    isLoadingCategories = true;
    categoriesError = null;
    try {
      const result = await apiClient.get<CategoryList[]>('/api/v1/categories');
      const categories = normalizeCategoryLists(result.data);
      if (result.success) {
        categoryLists = categories;
        if (categories.length === 0) {
          categoriesError = 'No categories available yet';
        }
      } else {
        categoriesError = 'Failed to load categories';
      }
    } catch (e) {
      categoriesError = 'Could not connect to server. Please try again.';
    } finally {
      isLoadingCategories = false;
    }
  }

  async function loadBlockedArtists() {
    isLoadingBlocked = true;
    blockedError = null;
    try {
      const result = await apiClient.get<BlockedArtist[]>('/api/v1/categories/blocked-artists');
      if (result.success) {
        blockedArtists = normalizeArtists(result.data);
      } else {
        blockedError = 'Failed to load blocked artists';
      }
    } catch (e) {
      blockedError = 'Could not connect to server. Please try again.';
    } finally {
      isLoadingBlocked = false;
    }
  }

  async function loadDnpList() {
    try {
      const result = await apiClient.get<Array<{artist_id: string}>>('/api/v1/dnp/list');
      if (result.success) {
        dnpList = new Set(normalizeDnpArtistIds(result.data));
      }
    } catch (e) {
      console.error('Failed to load DNP list:', e);
    }
  }

  function handleSearchInput() {
    clearTimeout(searchTimeout);
    if (!searchQuery.trim()) {
      searchResults = [];
      isSearching = false;
      return;
    }
    isSearching = true;
    searchTimeout = setTimeout(async () => {
      try {
        const result = await apiClient.get<SearchResponse>(`/api/v1/dnp/search?q=${encodeURIComponent(searchQuery.trim())}&limit=20`);
        if (result.success && result.data?.artists) {
          searchResults = result.data.artists;
        } else {
          searchResults = [];
        }
      } catch (e) {
        console.error('Search error:', e);
        searchResults = [];
      } finally {
        isSearching = false;
      }
    }, 300);
  }

  async function toggleCategory(categoryId: string) {
    if (expandedCategoryId === categoryId) {
      expandedCategoryId = null;
      categoryArtists = [];
    } else {
      expandedCategoryId = categoryId;
      await loadCategoryArtists(categoryId);
    }
  }

  async function loadCategoryArtists(categoryId: string) {
    isLoadingCategoryArtists = true;
    try {
      const result = await apiClient.get<{artists: BlockedArtist[]}>(`/api/v1/offenses/query?category=${categoryId}`);
      if (result.success) {
        categoryArtists = normalizeArtists(result.data);
      } else {
        categoryArtists = [];
      }
    } catch (e) {
      categoryArtists = [];
    } finally {
      isLoadingCategoryArtists = false;
    }
  }

  async function toggleCategorySubscription(category: CategoryList, event: MouseEvent) {
    event.stopPropagation();
    const wasSubscribed = category.subscribed;
    categoryLists = categoryLists.map(c =>
      c.id === category.id ? { ...c, subscribed: !wasSubscribed } : c
    );
    try {
      if (wasSubscribed) {
        await apiClient.delete(`/api/v1/categories/${category.id}/subscribe`);
      } else {
        await apiClient.post(`/api/v1/categories/${category.id}/subscribe`);
      }
      await loadBlockedArtists();

      const platforms = getConnectedPlatforms();
      if (platforms.length > 0) {
        const action = wasSubscribed ? 'unblock' : 'block';
        blockingStore.addToast({
          type: 'success',
          message: `${action === 'block' ? 'Blocked' : 'Unblocked'} ${formatCategoryName(category.id)} category (${category.artist_count} artists)`,
          dismissible: true,
          duration: 5000,
        });
      }
    } catch (e) {
      categoryLists = categoryLists.map(c =>
        c.id === category.id ? { ...c, subscribed: wasSubscribed } : c
      );
    }
  }

  async function goToArtist(artistId: string, artistName: string) {
    navigateToArtist(artistId);
  }

  async function unblockArtist(artistId: string, event: MouseEvent | KeyboardEvent, artistName?: string) {
    event.stopPropagation();
    event.preventDefault();
    const name = artistName || blockedArtists.find(a => a.id === artistId)?.name || 'Artist';
    simulateEnforcement(artistId, name, 'unblock');
    exceptedArtists.add(artistId);
    exceptedArtists = exceptedArtists;
    saveExceptions();
    try {
      await apiClient.delete(`/api/v1/dnp/list/${artistId}`);
      dnpList.delete(artistId);
      dnpList = dnpList;
    } catch (e) { /* ignore */ }
  }

  function reblockArtist(artistId: string, event: MouseEvent | KeyboardEvent, artistName?: string) {
    event.stopPropagation();
    event.preventDefault();
    const name = artistName || blockedArtists.find(a => a.id === artistId)?.name || 'Artist';
    simulateEnforcement(artistId, name, 'block');
    exceptedArtists.delete(artistId);
    exceptedArtists = exceptedArtists;
    saveExceptions();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      if (searchResults.length > 0) {
        searchResults = [];
        searchQuery = '';
      }
    }
  }

  function getConnectedPlatforms(): Platform[] {
    const platforms: Platform[] = [];
    if ($spotifyConnection?.status === 'active') platforms.push('spotify');
    if ($appleMusicConnection?.status === 'active') platforms.push('apple_music');
    return platforms;
  }

  async function simulateEnforcement(artistId: string, artistName: string, action: 'block' | 'unblock') {
    const platforms = getConnectedPlatforms();
    if (platforms.length === 0) return;
    blockingStore.startOperation(artistId, artistName, action, platforms);
    for (const platform of platforms) {
      blockingStore.updatePlatformStatus(artistId, platform, 'in_progress');
      await new Promise(resolve => setTimeout(resolve, 300 + Math.random() * 500));
      const success = Math.random() > 0.1;
      blockingStore.updatePlatformStatus(artistId, platform, success ? 'completed' : 'failed', success ? undefined : 'API error');
    }
    blockingStore.completeOperation(artistId);
  }

  function clearSearch() {
    searchQuery = '';
    searchResults = [];
    isSearching = false;
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="home brand-page surface-page">
  <div class="brand-page__inner brand-page__stack">
    <section class="hero brand-hero">
      <div class="hero__header">
        <div class="hero__copy">
          <div class="brand-kickers">
            <span class="brand-kicker">Spotify + Apple Music</span>
            <span class="brand-kicker brand-kicker--accent">Evidence-led filters</span>
          </div>

          <h1 class="hero__title">
            <span class="hero__title-brand">No Drake in the House</span>
            <span class="hero__title-main">Clean your feed without flattening your taste.</span>
          </h1>
          <p class="brand-subtitle hero__subtitle">Search artists, block by category, and keep exceptions where you need them across Spotify and Apple Music.</p>

          <div class="brand-meta">
            <span class="brand-meta__item">
              {connectedPlatformCount > 0
                ? `${connectedPlatformCount} connected service${connectedPlatformCount === 1 ? '' : 's'}`
                : 'No services connected yet'}
            </span>
            <span class="brand-meta__item">{activeCategoryCount} active categories</span>
          </div>
        </div>

        <div class="hero__stats brand-stat-grid" aria-label="Account overview">
          <div class="brand-stat">
            <span class="brand-stat__value">{activeCategoryCount}</span>
            <span class="brand-stat__label">Active categories</span>
          </div>
          <div class="brand-stat">
            <span class="brand-stat__value">{uniqueBlockedArtists.length}</span>
            <span class="brand-stat__label">Artists blocked</span>
          </div>
          <div class="brand-stat">
            <span class="brand-stat__value">{connectedPlatformCount}</span>
            <span class="brand-stat__label">Connected services</span>
          </div>
        </div>
      </div>
    </section>

    <div class="search home__search">
      <div class="search__icon">
        {#if isSearching}
          <div class="search__spinner"></div>
        {:else}
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
        {/if}
      </div>
      <input
        type="text"
        bind:value={searchQuery}
        on:input={handleSearchInput}
        placeholder="Search any artist..."
        aria-label="Search artists"
        class="search__input"
      />
      {#if searchQuery}
        <button type="button" class="search__clear" on:click={clearSearch} aria-label="Clear search">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
        </button>
      {/if}

      {#if searchResults.length > 0 || (searchQuery.length > 1 && !isSearching)}
        <div class="search__dropdown">
          {#if searchResults.length > 0}
            <div class="search__results">
              {#each searchResults as artist, i}
                <button
                  type="button"
                  class="search__result"
                  on:click={() => goToArtist(artist.id, artist.canonical_name)}
                >
                  <div class="search__result-info">
                    <span class="search__result-name">{artist.canonical_name}</span>
                    {#if artist.has_offenses}
                      <span class="search__result-badge">{artist.offense_count} offense{artist.offense_count !== 1 ? 's' : ''}</span>
                    {/if}
                  </div>
                  {#if artist.genres && artist.genres.length > 0}
                    <p class="search__result-genres">{artist.genres.slice(0, 2).join(', ')}</p>
                  {/if}
                </button>
              {/each}
            </div>
          {:else}
            <div class="search__empty">
              No artists found for "<strong>{searchQuery}</strong>"
            </div>
          {/if}
        </div>
      {/if}
    </div>

    <div class="content brand-page__stack">
      <!-- Categories -->
      <section class="section section--card surface-card">
        <div class="section__header">
          <div>
            <h2 class="section__title">Block by Category</h2>
            <p class="section__description">Toggle category-wide filters, inspect the artists inside each bucket, and keep exceptions under control.</p>
          </div>
        </div>

        {#if isLoadingCategories}
          <div class="loader">
            <div class="loader__spinner"></div>
          </div>
        {:else if categoriesError}
          <div class="brand-alert brand-alert--error">
            <p>{categoriesError}</p>
            <button type="button" class="brand-alert__dismiss" on:click={loadCategories}>Try again</button>
          </div>
        {:else}
          <div class="categories">
            {#each categoryLists as category}
              {@const catColor = getCategoryColor(category.id)}
              {@const isExpanded = expandedCategoryId === category.id}
              <div
                class="category-chip"
                class:category-chip--subscribed={category.subscribed}
                class:category-chip--expanded={isExpanded}
                style="--cat-color: {catColor.icon}; --cat-bg: {catColor.bg};"
              >
                <button
                  type="button"
                  class="category-chip__toggle"
                  on:click={(e) => toggleCategorySubscription(category, e)}
                  title="{category.subscribed ? 'Unblock' : 'Block'} all {formatCategoryName(category.id)} artists"
                >
                  {#if category.subscribed}
                    <svg viewBox="0 0 24 24" fill="currentColor"><path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/></svg>
                  {:else}
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="12" y1="6" x2="12" y2="18"/><line x1="6" y1="12" x2="18" y2="12"/></svg>
                  {/if}
                </button>
                <button
                  type="button"
                  class="category-chip__label"
                  on:click={() => toggleCategory(category.id)}
                  title="View {formatCategoryName(category.id)} artists"
                >
                  <span class="category-chip__name">{formatCategoryName(category.id)}</span>
                  <span class="category-chip__count">{category.artist_count}</span>
                </button>
              </div>
            {/each}
          </div>

          <!-- Expanded Category Panel -->
          {#if expandedCategoryId}
            {@const catColor = getCategoryColor(expandedCategoryId)}
            {@const selectedCategory = categoryLists.find(c => c.id === expandedCategoryId)}
            {@const exceptedInCategory = categoryArtists.filter(a => exceptedArtists.has(a.id))}
            {@const blockedInCategory = categoryArtists.filter(a => !exceptedArtists.has(a.id))}
            <div class="category-panel" style="--cat-color: {catColor.icon};">
              <div class="category-panel__header">
                <div>
                  <h3 class="category-panel__title">{formatCategoryName(expandedCategoryId)}</h3>
                  <p class="category-panel__meta">
                    {#if selectedCategory?.subscribed}
                      {blockedInCategory.length} blocked{#if exceptedInCategory.length > 0} · {exceptedInCategory.length} excepted{/if}
                    {:else}
                      {categoryArtists.length} artists available
                    {/if}
                  </p>
                </div>
                <div class="category-panel__actions">
                  {#if selectedCategory}
                    <button
                      type="button"
                      class="category-panel__action-btn"
                      class:category-panel__action-btn--active={selectedCategory.subscribed}
                      on:click={(e) => toggleCategorySubscription(selectedCategory, e)}
                    >
                      {selectedCategory.subscribed ? 'Unsubscribe' : 'Block All'}
                    </button>
                  {/if}
                  {#if exceptedInCategory.length > 0 && selectedCategory?.subscribed}
                    <button
                      type="button"
                      class="category-panel__action-btn category-panel__action-btn--secondary"
                      on:click={() => {
                        exceptedInCategory.forEach(a => exceptedArtists.delete(a.id));
                        exceptedArtists = exceptedArtists;
                        saveExceptions();
                      }}
                    >
                      Re-block All ({exceptedInCategory.length})
                    </button>
                  {/if}
                  <button
                    type="button"
                    class="category-panel__close"
                    on:click={() => { expandedCategoryId = null; categoryArtists = []; }}
                    aria-label="Close panel"
                  >
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
                  </button>
                </div>
              </div>

              <div class="category-panel__body">
                {#if isLoadingCategoryArtists}
                  <div class="loader"><div class="loader__spinner"></div></div>
                {:else if categoryArtists.length > 0}
                  {#if blockedInCategory.length > 0 && selectedCategory?.subscribed}
                    <div class="category-panel__section">
                      <p class="category-panel__section-label">Blocked</p>
                      <div class="artist-grid">
                        {#each blockedInCategory as artist}
                          {@const sev = getSeverityStyle(artist.severity)}
                          <div class="artist-tile group">
                            <button type="button" class="artist-tile__main" on:click={() => goToArtist(artist.id, artist.name)}>
                              <span class="artist-tile__name">{artist.name}</span>
                              <EnforcementBadges artistId={artist.id} compact={true} />
                              <span class="artist-tile__severity" style="color: {sev.color}">{sev.label}</span>
                            </button>
                            <button
                              type="button"
                              class="artist-tile__remove"
                              on:click={(e) => unblockArtist(artist.id, e, artist.name)}
                              title="Unblock"
                            >
                              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
                            </button>
                          </div>
                        {/each}
                      </div>
                    </div>
                  {/if}

                  {#if exceptedInCategory.length > 0 && selectedCategory?.subscribed}
                    <div class="category-panel__section category-panel__section--excepted">
                      <p class="category-panel__section-label">Not Blocked (Excepted)</p>
                      <div class="artist-grid">
                        {#each exceptedInCategory as artist}
                          <div class="artist-tile artist-tile--excepted group">
                            <button type="button" class="artist-tile__main" on:click={() => goToArtist(artist.id, artist.name)}>
                              <span class="artist-tile__name">{artist.name}</span>
                              <span class="artist-tile__severity">Excepted</span>
                            </button>
                            <button
                              type="button"
                              class="artist-tile__reblock"
                              on:click={(e) => reblockArtist(artist.id, e, artist.name)}
                              title="Re-block"
                            >
                              Block
                            </button>
                          </div>
                        {/each}
                      </div>
                    </div>
                  {/if}

                  {#if !selectedCategory?.subscribed}
                    <div class="artist-grid">
                      {#each categoryArtists as artist}
                        {@const sev = getSeverityStyle(artist.severity)}
                        <button type="button" class="artist-tile" on:click={() => goToArtist(artist.id, artist.name)}>
                          <span class="artist-tile__name">{artist.name}</span>
                          <span class="artist-tile__severity" style="color: {sev.color}">{sev.label}</span>
                        </button>
                      {/each}
                    </div>
                  {/if}
                {:else}
                  <p class="category-panel__empty">No artists in this category yet</p>
                {/if}
              </div>
            </div>
          {/if}
        {/if}
      </section>

      <!-- Blocked Artists -->
      <section class="section section--card surface-card">
        <div class="section__header">
          <div>
            <h2 class="section__title">
              Your Blocked Artists
              {#if uniqueBlockedArtists.length > 0}
                <span class="section__count">{uniqueBlockedArtists.length}</span>
              {/if}
            </h2>
            <p class="section__description">Artists currently filtered from your connected listening surface, with one-click exceptions when you need them.</p>
          </div>
        </div>

        {#if isLoadingBlocked}
          <div class="loader"><div class="loader__spinner"></div></div>
        {:else if blockedError}
          <div class="brand-alert brand-alert--error">
            <p>{blockedError}</p>
            <button type="button" class="brand-alert__dismiss" on:click={loadBlockedArtists}>Try again</button>
          </div>
        {:else if uniqueBlockedArtists.length === 0}
          <div class="brand-empty">
            <p class="brand-empty__title">No artists blocked yet</p>
            <p class="brand-empty__copy">Toggle categories above to start blocking and keep the rest of your listening graph intact.</p>
          </div>
        {:else}
          <div class="blocked-chips">
            {#each uniqueBlockedArtists as artist}
              <div
                class="blocked-chip group"
                data-testid="blocked-artist-chip"
              >
                <button
                  type="button"
                  class="blocked-chip__name"
                  data-testid="blocked-artist-name"
                  on:click={() => goToArtist(artist.id, artist.name)}
                  title="View artist profile"
                >{artist.name}</button>
                <EnforcementBadges artistId={artist.id} compact={true} />
                <button
                  type="button"
                  class="blocked-chip__remove"
                  on:click={(e) => unblockArtist(artist.id, e, artist.name)}
                  title="Remove from blocklist"
                  data-testid="unblock-artist-button"
                >
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
                </button>
              </div>
            {/each}
          </div>
        {/if}
      </section>
    </div>
  </div>
</div>

<style>
  .home {
    min-height: calc(100vh - 4.5rem);
  }

  /* ===== HERO ===== */
  .hero__header {
    display: grid;
    grid-template-columns: minmax(0, 1.25fr) minmax(18rem, 0.9fr);
    gap: 1.5rem;
    align-items: flex-end;
  }

  .hero__title {
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
    margin: 0;
    max-width: 16ch;
  }

  .hero__title-brand {
    font-size: 0.72rem;
    font-weight: 700;
    color: #fda4af;
    letter-spacing: 0.16em;
    text-transform: uppercase;
  }

  .hero__title-main {
    font-size: clamp(2rem, 5vw, 3.5rem);
    font-weight: 700;
    color: var(--color-text-primary);
    letter-spacing: -0.05em;
    line-height: 1.1;
  }

  .hero__subtitle {
    max-width: 40rem;
  }

  .hero__stats {
    align-self: end;
  }

  /* ===== SEARCH ===== */
  .search {
    position: relative;
    margin-top: 1.5rem;
    max-width: 46rem;
  }

  .home__search {
    margin-top: -0.5rem;
    z-index: var(--z-dropdown, 100);
  }

  .search__icon {
    position: absolute;
    left: 0.875rem;
    top: 50%;
    transform: translateY(-50%);
    color: var(--color-text-muted);
    z-index: 1;
    pointer-events: none;
    width: 1.125rem;
    height: 1.125rem;
  }

  .search__icon svg {
    width: 100%;
    height: 100%;
    max-width: none;
    max-height: none;
  }

  .search__spinner {
    width: 1.125rem;
    height: 1.125rem;
    border: 2px solid var(--color-border-default);
    border-top-color: var(--color-text-secondary);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  .search__input {
    width: 100%;
    min-height: 3rem;
    padding: 0.8rem 2.9rem 0.8rem 3rem;
    font-family: var(--font-family-sans);
    font-size: 0.95rem;
    color: var(--color-text-primary);
    background:
      linear-gradient(180deg, rgba(255, 255, 255, 0.03), rgba(255, 255, 255, 0.01)),
      rgba(24, 24, 27, 0.92);
    border: 1px solid rgba(255, 255, 255, 0.09);
    border-radius: 1rem;
    transition: border-color var(--transition-fast), box-shadow var(--transition-fast), transform var(--transition-fast);
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.03);
  }

  .search__input::placeholder {
    color: var(--color-text-muted);
  }

  .search__input:focus {
    outline: none;
    transform: translateY(-1px);
    border-color: rgba(244, 63, 94, 0.42);
    box-shadow: 0 0 0 3px rgba(244, 63, 94, 0.16);
  }

  .search__clear {
    position: absolute;
    right: 0.625rem;
    top: 50%;
    transform: translateY(-50%);
    width: 1.5rem;
    height: 1.5rem;
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--color-text-muted);
    border-radius: var(--radius-sm);
    transition: color var(--transition-fast);
    padding: 0;
  }

  .search__clear svg {
    width: 0.875rem;
    height: 0.875rem;
    max-width: none;
    max-height: none;
  }

  .search__clear:hover {
    color: var(--color-text-secondary);
  }

  .search__dropdown {
    position: absolute;
    top: calc(100% + 0.375rem);
    left: 0;
    right: 0;
    background:
      linear-gradient(180deg, rgba(255, 255, 255, 0.04), rgba(255, 255, 255, 0.018)),
      rgba(17, 17, 19, 0.94);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 1.15rem;
    box-shadow: 0 24px 48px rgba(0, 0, 0, 0.28);
    z-index: var(--z-dropdown);
    overflow: hidden;
    backdrop-filter: blur(14px);
  }

  .search__results {
    max-height: 20rem;
    overflow-y: auto;
  }

  .search__result {
    display: block;
    width: 100%;
    padding: 0.625rem 1rem;
    text-align: left;
    background: none;
    border: none;
    border-bottom: 1px solid var(--color-border-subtle);
    cursor: pointer;
    transition: background var(--transition-fast);
  }

  .search__result:last-child {
    border-bottom: none;
  }

  .search__result:hover {
    background: rgba(255, 255, 255, 0.04);
  }

  .search__result-info {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .search__result-name {
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .search__result-badge {
    font-size: 0.6875rem;
    font-weight: 500;
    padding: 1px 0.375rem;
    border-radius: var(--radius-full);
    background: var(--color-brand-primary-muted);
    color: var(--color-brand-primary);
  }

  .search__result-genres {
    font-size: var(--text-xs);
    color: var(--color-text-tertiary);
    margin-top: 0.125rem;
  }

  .search__empty {
    padding: 1.5rem 1rem;
    text-align: center;
    font-size: var(--text-sm);
    color: var(--color-text-tertiary);
  }

  .search__empty strong {
    color: var(--color-text-primary);
  }

  /* ===== CONTENT ===== */
  .content {
    gap: 1.25rem;
  }

  .section {
    margin: 0;
  }

  .section--card {
    padding: 1.1rem 1.15rem 1.2rem;
    border-radius: 1.5rem;
  }

  .section__title {
    font-size: 1.1rem;
    font-weight: 700;
    color: var(--color-text-primary);
    margin: 0;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .section__header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 1rem;
    margin-bottom: 1rem;
  }

  .section__description {
    margin: 0.45rem 0 0;
    max-width: 42rem;
    color: var(--color-text-secondary);
    line-height: 1.55;
    font-size: 0.92rem;
  }

  .section__count {
    font-size: var(--text-xs);
    font-weight: 500;
    color: #fda4af;
    background: rgba(244, 63, 94, 0.1);
    border: 1px solid rgba(244, 63, 94, 0.18);
    padding: 0.1rem 0.5rem;
    border-radius: var(--radius-full);
  }

  /* ===== CATEGORIES ===== */
  .categories {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .category-chip {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.25rem;
    padding-right: 0.125rem;
    border-radius: 1rem;
    background:
      linear-gradient(180deg, rgba(255, 255, 255, 0.03), rgba(255, 255, 255, 0.01)),
      rgba(24, 24, 27, 0.92);
    border: 1px solid rgba(255, 255, 255, 0.08);
    transition: all var(--transition-fast);
  }

  .category-chip--subscribed {
    border-color: var(--cat-color);
    background: var(--cat-bg);
  }

  @media (max-width: 900px) {
    .hero__header {
      grid-template-columns: 1fr;
    }

    .hero__title-main {
      max-width: 14ch;
    }
  }

  @media (max-width: 640px) {
    .hero__title-brand {
      font-size: 0.72rem;
    }
  }

  .category-chip--expanded {
    border-color: rgba(244, 63, 94, 0.22);
    box-shadow: 0 14px 28px rgba(0, 0, 0, 0.12);
  }

  .category-chip__toggle {
    width: 1.5rem;
    height: 1.5rem;
    border-radius: var(--radius-md);
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--color-text-muted);
    transition: color var(--transition-fast);
    padding: 0;
    flex-shrink: 0;
  }

  .category-chip__toggle svg {
    width: 0.875rem;
    height: 0.875rem;
    max-width: none;
    max-height: none;
  }

  .category-chip--subscribed .category-chip__toggle {
    color: var(--cat-color);
  }

  .category-chip__label {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    background: none;
    border: none;
    cursor: pointer;
    padding: 0.25rem 0.5rem 0.25rem 0;
    min-width: 0;
  }

  .category-chip__name {
    font-size: var(--text-xs);
    font-weight: 500;
    color: var(--color-text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .category-chip__count {
    font-size: 0.6875rem;
    color: var(--color-text-muted);
    font-variant-numeric: tabular-nums;
    flex-shrink: 0;
  }

  /* ===== CATEGORY PANEL ===== */
  .category-panel {
    margin-top: 0.75rem;
    border-radius: 1.35rem;
    border: 1px solid rgba(255, 255, 255, 0.08);
    background:
      linear-gradient(180deg, rgba(255, 255, 255, 0.04), rgba(255, 255, 255, 0.015)),
      rgba(17, 17, 19, 0.9);
    overflow: hidden;
    box-shadow: 0 20px 44px rgba(0, 0, 0, 0.18);
    backdrop-filter: blur(14px);
  }

  .category-panel__header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 1rem;
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--color-border-subtle);
    background: var(--color-bg-surface);
  }

  .category-panel__title {
    font-size: var(--text-lg);
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .category-panel__meta {
    font-size: var(--text-sm);
    color: var(--color-text-tertiary);
    margin-top: 0.125rem;
  }

  .category-panel__actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-shrink: 0;
  }

  .category-panel__action-btn {
    padding: 0.375rem 0.875rem;
    font-size: var(--text-xs);
    font-weight: 600;
    border-radius: var(--radius-full);
    border: none;
    cursor: pointer;
    transition: all var(--transition-fast);
    background: var(--cat-color);
    color: white;
  }

  .category-panel__action-btn--active {
    background: var(--color-bg-interactive);
    color: var(--color-text-secondary);
    border: 1px solid var(--color-border-default);
  }

  .category-panel__action-btn--secondary {
    background: var(--color-bg-interactive);
    color: var(--color-text-secondary);
    border: 1px solid var(--color-border-default);
  }

  .category-panel__close {
    width: 2rem;
    height: 2rem;
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--color-text-muted);
    border-radius: var(--radius-md);
    transition: all var(--transition-fast);
    padding: 0;
  }

  .category-panel__close svg {
    width: 1.125rem;
    height: 1.125rem;
    max-width: none;
    max-height: none;
  }

  .category-panel__close:hover {
    color: var(--color-text-primary);
    background: var(--color-bg-hover);
  }

  .category-panel__body {
    padding: 1rem 1.25rem;
    max-height: 24rem;
    overflow-y: auto;
  }

  .category-panel__section {
    margin-bottom: 1rem;
  }

  .category-panel__section--excepted {
    padding-top: 1rem;
    border-top: 1px solid var(--color-border-subtle);
  }

  .category-panel__section-label {
    font-size: 0.6875rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--color-text-muted);
    margin-bottom: 0.5rem;
  }

  .category-panel__empty {
    text-align: center;
    color: var(--color-text-tertiary);
    padding: 2rem 0;
    font-size: var(--text-sm);
  }

  /* ===== ARTIST GRID ===== */
  .artist-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(8rem, 1fr));
    gap: 0.375rem;
  }

  .artist-tile {
    position: relative;
    padding: 0.625rem;
    border-radius: 0.95rem;
    background:
      linear-gradient(180deg, rgba(255, 255, 255, 0.03), rgba(255, 255, 255, 0.01)),
      rgba(24, 24, 27, 0.92);
    border: none;
    cursor: pointer;
    transition: all var(--transition-fast);
    text-align: left;
  }

  .artist-tile:hover {
    background:
      linear-gradient(180deg, rgba(255, 255, 255, 0.05), rgba(255, 255, 255, 0.012)),
      rgba(32, 32, 35, 0.94);
  }

  .artist-tile--excepted {
    opacity: 0.6;
    border: 1px dashed var(--color-border-default);
    background: transparent;
  }

  .artist-tile__main {
    display: block;
    width: 100%;
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
    text-align: left;
  }

  .artist-tile__name {
    display: block;
    font-size: var(--text-xs);
    font-weight: 500;
    color: var(--color-text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .artist-tile__severity {
    display: block;
    font-size: 0.6875rem;
    margin-top: 0.125rem;
    color: var(--color-text-muted);
  }

  .artist-tile__remove {
    position: absolute;
    top: 0.25rem;
    right: 0.25rem;
    width: 1.25rem;
    height: 1.25rem;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--color-bg-page);
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    color: var(--color-text-muted);
    opacity: 0;
    transition: opacity var(--transition-fast);
    padding: 0;
  }

  .artist-tile__remove svg {
    width: 0.625rem;
    height: 0.625rem;
    max-width: none;
    max-height: none;
  }

  :global(.group):hover .artist-tile__remove {
    opacity: 1;
  }

  .artist-tile__reblock {
    position: absolute;
    top: 0.25rem;
    right: 0.25rem;
    padding: 0.125rem 0.375rem;
    font-size: 0.625rem;
    font-weight: 600;
    background: var(--cat-color, var(--color-brand-primary));
    color: white;
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    opacity: 0;
    transition: opacity var(--transition-fast);
  }

  :global(.group):hover .artist-tile__reblock {
    opacity: 1;
  }

  /* ===== BLOCKED CHIPS ===== */
  .blocked-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .blocked-chip {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.35rem 0.45rem 0.35rem 0.75rem;
    border-radius: var(--radius-full);
    background:
      linear-gradient(180deg, rgba(255, 255, 255, 0.03), rgba(255, 255, 255, 0.01)),
      rgba(24, 24, 27, 0.9);
    border: 1px solid rgba(255, 255, 255, 0.08);
    cursor: pointer;
    transition: all var(--transition-fast);
    font-family: var(--font-family-sans);
  }

  .blocked-chip:hover {
    background:
      linear-gradient(180deg, rgba(255, 255, 255, 0.05), rgba(255, 255, 255, 0.015)),
      rgba(32, 32, 35, 0.94);
    border-color: rgba(244, 63, 94, 0.18);
  }

  .blocked-chip__name {
    font-size: var(--text-xs);
    font-weight: 500;
    color: var(--color-text-secondary);
  }

  .blocked-chip__remove {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 1rem;
    height: 1rem;
    border-radius: var(--radius-full);
    color: var(--color-text-muted);
    opacity: 0.4;
    transition: all var(--transition-fast);
    cursor: pointer;
  }

  .blocked-chip__remove svg {
    width: 0.625rem;
    height: 0.625rem;
    max-width: none;
    max-height: none;
  }

  .blocked-chip__remove:hover {
    opacity: 1;
    background: var(--color-bg-hover);
  }

  /* ===== SHARED ===== */
  .loader {
    display: flex;
    justify-content: center;
    padding: 2rem 0;
  }

  .loader__spinner {
    width: 1.5rem;
    height: 1.5rem;
    border: 2px solid var(--color-border-default);
    border-top-color: var(--color-text-secondary);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
