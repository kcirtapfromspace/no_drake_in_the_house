<script lang="ts">
  import { onMount } from 'svelte';
  import { apiClient } from '../utils/api-client';
  import { navigateTo, navigateToArtist } from '../utils/simple-router';
  import { blockingStore, type Platform } from '../stores/blocking';
  import { spotifyConnection, appleMusicConnection } from '../stores/connections';
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
  let expandedCategoryId: string | null = null;
  let categoryArtists: BlockedArtist[] = [];
  let isLoadingCategoryArtists = false;

  let blockedArtists: BlockedArtist[] = [];
  let isLoadingBlocked = false;

  let dnpList: Set<string> = new Set();

  // Excepted artists - individually unblocked while categories remain subscribed
  let exceptedArtists: Set<string> = new Set();

  // Load exceptions from localStorage on init
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

  // Save exceptions to localStorage
  function saveExceptions() {
    try {
      localStorage.setItem('exceptedArtists', JSON.stringify([...exceptedArtists]));
    } catch (e) {
      console.error('Failed to save exceptions:', e);
    }
  }

  // Deduplicated blocked artists, excluding excepted ones
  $: uniqueBlockedArtists = blockedArtists.reduce((acc, artist) => {
    // Skip if already added or if excepted
    if (!acc.some(a => a.id === artist.id) && !exceptedArtists.has(artist.id)) {
      acc.push(artist);
    }
    return acc;
  }, [] as BlockedArtist[]);
  $: activeCategoryCount = categoryLists.filter(category => category.subscribed).length;
  $: connectedPlatformCount = getConnectedPlatforms().length;

  // Get excepted artists that would be blocked by current categories
  $: exceptedFromCategories = blockedArtists.filter(a => exceptedArtists.has(a.id))
    .reduce((acc, artist) => {
      if (!acc.some(a => a.id === artist.id)) {
        acc.push(artist);
      }
      return acc;
    }, [] as BlockedArtist[]);

  // Category-specific colors for visual differentiation
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

  function getSeverityStyle(severity: string): { bg: string; text: string; label: string } {
    switch (severity) {
      case 'egregious':
        return { bg: 'bg-rose-500/20', text: 'text-rose-300', label: 'Egregious' };
      case 'severe':
        return { bg: 'bg-orange-500/20', text: 'text-orange-300', label: 'Severe' };
      case 'moderate':
        return { bg: 'bg-yellow-500/20', text: 'text-yellow-300', label: 'Moderate' };
      default:
        return { bg: 'bg-zinc-500/20', text: 'text-zinc-300', label: 'Minor' };
    }
  }

  onMount(async () => {
    loadExceptions();
    await Promise.all([
      loadCategories(),
      loadBlockedArtists(),
      loadDnpList(),
    ]);
  });

  async function loadCategories() {
    isLoadingCategories = true;
    try {
      const result = await apiClient.get<CategoryList[]>('/api/v1/categories');
      if (result.success && result.data) {
        categoryLists = result.data;
      }
    } catch (e) {
      console.error('Failed to load categories:', e);
    } finally {
      isLoadingCategories = false;
    }
  }

  async function loadBlockedArtists() {
    isLoadingBlocked = true;
    try {
      const result = await apiClient.get<BlockedArtist[]>('/api/v1/categories/blocked-artists');
      if (result.success && result.data) {
        blockedArtists = result.data;
      }
    } catch (e) {
      console.error('Failed to load blocked artists:', e);
    } finally {
      isLoadingBlocked = false;
    }
  }

  async function loadDnpList() {
    try {
      const result = await apiClient.get<Array<{artist_id: string}>>('/api/v1/dnp/list');
      if (result.success && result.data) {
        dnpList = new Set(result.data.map(item => item.artist_id));
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
      if (result.success && result.data?.artists) {
        categoryArtists = result.data.artists;
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

      // Trigger enforcement notification for category action
      const platforms = getConnectedPlatforms();
      if (platforms.length > 0) {
        const action = wasSubscribed ? 'unblock' : 'block';
        blockingStore.addToast({
          type: 'info',
          message: `${action === 'block' ? 'Blocking' : 'Unblocking'} ${category.artist_count} artists from ${formatCategoryName(category.id)}...`,
          dismissible: false,
          progress: 0,
        });

        // Simulate bulk enforcement (in production this would be a batch API call)
        await new Promise(resolve => setTimeout(resolve, 500 + category.artist_count * 20));

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

    // Get artist name for progress display
    const name = artistName || blockedArtists.find(a => a.id === artistId)?.name || 'Artist';

    // Start enforcement simulation for unblock
    simulateEnforcement(artistId, name, 'unblock');

    // Add to exceptions - artist will be excluded from category blocks
    exceptedArtists.add(artistId);
    exceptedArtists = exceptedArtists; // Trigger reactivity
    saveExceptions();

    // Also try to remove from individual DNP list if they were manually added
    try {
      await apiClient.delete(`/api/v1/dnp/list/${artistId}`);
      dnpList.delete(artistId);
      dnpList = dnpList;
    } catch (e) {
      // Ignore - they might only be blocked via category
    }
  }

  function reblockArtist(artistId: string, event: MouseEvent | KeyboardEvent, artistName?: string) {
    event.stopPropagation();
    event.preventDefault();

    // Get artist name for progress display
    const name = artistName || blockedArtists.find(a => a.id === artistId)?.name || 'Artist';

    // Start enforcement simulation for block
    simulateEnforcement(artistId, name, 'block');

    // Remove from exceptions - artist will be blocked again by their categories
    exceptedArtists.delete(artistId);
    exceptedArtists = exceptedArtists; // Trigger reactivity
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

  // Get list of connected platforms
  function getConnectedPlatforms(): Platform[] {
    const platforms: Platform[] = [];
    if ($spotifyConnection?.status === 'active') platforms.push('spotify');
    if ($appleMusicConnection?.status === 'active') platforms.push('apple_music');
    return platforms;
  }

  // Simulate enforcement with progress (in production this would call actual APIs)
  async function simulateEnforcement(artistId: string, artistName: string, action: 'block' | 'unblock') {
    const platforms = getConnectedPlatforms();
    if (platforms.length === 0) return;

    // Start the operation
    blockingStore.startOperation(artistId, artistName, action, platforms);

    // Simulate each platform enforcement with delays
    for (const platform of platforms) {
      blockingStore.updatePlatformStatus(artistId, platform, 'in_progress');

      // Simulate API call delay (300-800ms per platform)
      await new Promise(resolve => setTimeout(resolve, 300 + Math.random() * 500));

      // Simulate success (90% success rate for demo)
      const success = Math.random() > 0.1;
      blockingStore.updatePlatformStatus(
        artistId,
        platform,
        success ? 'completed' : 'failed',
        success ? undefined : 'API error'
      );
    }

    // Complete the operation
    blockingStore.completeOperation(artistId);
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="home">
  <!-- Hero Search Section -->
  <section class="home__hero-section">
    <div
      class="home__hero-card"
      style="background:
        radial-gradient(circle at top left, rgba(244, 63, 94, 0.16), transparent 38%),
        radial-gradient(circle at top right, rgba(56, 189, 248, 0.12), transparent 30%),
        linear-gradient(180deg, rgba(255,255,255,0.05), rgba(255,255,255,0.02)),
        rgba(9, 9, 11, 0.82);
        box-shadow: 0 28px 80px rgba(0, 0, 0, 0.42);"
    >
      <div class="home__hero-badges">
        <span class="home__hero-badge">
          Spotify + Apple Music
        </span>
        <span class="home__hero-badge home__hero-badge--brand">
          Evidence-led filters
        </span>
      </div>

      <div class="home__hero-grid">
        <div class="home__hero-copy">
          <p class="home__brand-kicker">No Drake in the House</p>
          <h1 class="home__hero-title">
            Clean your feed without flattening your taste.
          </h1>
          <p class="home__hero-subtitle">
            Search artists, block by category, and keep exceptions where you need them across Spotify and Apple Music.
          </p>
        </div>

        <div class="home__hero-metrics">
          <div class="home__metric-card">
            <span class="home__metric-value">{activeCategoryCount}</span>
            <span class="home__metric-label">Active categories</span>
          </div>
          <div class="home__metric-card">
            <span class="home__metric-value">{uniqueBlockedArtists.length}</span>
            <span class="home__metric-label">Artists blocked</span>
          </div>
          <div class="home__metric-card">
            <span class="home__metric-value">{connectedPlatformCount}</span>
            <span class="home__metric-label">Connected services</span>
          </div>
        </div>
      </div>

      <div class="home__search">
      <div class="home__search-icon">
        {#if isSearching}
          <div class="home__spinner home__spinner--small"></div>
        {:else}
          <svg class="home__search-icon-svg" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
        {/if}
      </div>
      <input
        type="text"
        bind:value={searchQuery}
        on:input={handleSearchInput}
        placeholder="Search any artist..."
        class="home__search-input"
      />

      <!-- Search Results Dropdown -->
      {#if searchResults.length > 0 || (searchQuery.length > 1 && !isSearching)}
        <div class="home__search-results">
          {#if searchResults.length > 0}
            <div class="home__search-results-list">
              {#each searchResults as artist, i}
                <button
                  type="button"
                  class="home__search-result"
                  class:home__search-result--separated={i !== searchResults.length - 1}
                  on:click={() => goToArtist(artist.id, artist.canonical_name)}
                >
                  <div class="home__search-result-copy">
                    <div class="home__search-result-head">
                      <span class="home__search-result-name">{artist.canonical_name}</span>
                      {#if artist.has_offenses}
                        <span class="home__search-result-badge">
                          {artist.offense_count} offense{artist.offense_count !== 1 ? 's' : ''}
                        </span>
                      {/if}
                    </div>
                    {#if artist.genres && artist.genres.length > 0}
                      <p class="home__search-result-meta">{artist.genres.slice(0, 2).join(', ')}</p>
                    {/if}
                  </div>
                  <svg class="home__search-result-arrow" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                  </svg>
                </button>
              {/each}
            </div>
          {:else}
            <div class="home__search-empty">
              <p class="home__search-empty-text">No artists found for "<span class="home__search-empty-query">{searchQuery}</span>"</p>
            </div>
          {/if}
        </div>
      {/if}
    </div>
    </div>
  </section>

  <!-- Categories -->
  <section
    class="home__panel"
    style="box-shadow: 0 20px 60px rgba(0, 0, 0, 0.32);"
  >
    <div class="home__section-head">
      <h2 class="home__section-title">Block by Category</h2>
      <span class="home__section-kicker">Evidence-led filters</span>
    </div>

    {#if isLoadingCategories}
      <div class="home__loading">
        <div class="home__spinner"></div>
      </div>
    {:else}
      <!-- Category Cards - Grid layout for consistent 2+ rows -->
      <div class="home__category-grid">
        {#each categoryLists as category}
          {@const catColor = getCategoryColor(category.id)}
          {@const isExpanded = expandedCategoryId === category.id}
          <div
            class="category-card"
            style="background: {category.subscribed ? catColor.bg : 'rgba(255,255,255,0.03)'}; border: 1px solid {category.subscribed ? catColor.icon : isExpanded ? 'rgba(255,255,255,0.24)' : 'rgba(255,255,255,0.08)'}; color: white; box-shadow: {isExpanded ? '0 18px 40px rgba(0,0,0,0.28)' : 'none'};"
          >
            <!-- Toggle -->
            <button
              type="button"
              class="category-card__toggle"
              on:click={(e) => { e.stopPropagation(); toggleCategorySubscription(category, e); }}
              title="{category.subscribed ? 'Unblock' : 'Block'} all {formatCategoryName(category.id)} artists"
            >
              {#if category.subscribed}
                <svg class="category-card__toggle-icon" style="color: {catColor.icon};" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                  <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
                </svg>
              {:else}
                <svg class="category-card__toggle-icon category-card__toggle-icon--muted" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v12m6-6H6"/>
                </svg>
              {/if}
            </button>

            <!-- Category Name - Click to expand -->
            <button
              type="button"
              class="category-card__content"
              on:click={() => toggleCategory(category.id)}
              title="View {formatCategoryName(category.id)} artists"
            >
              <span class="category-card__name">{formatCategoryName(category.id)}</span>
              <span class="category-card__count">{category.artist_count}</span>
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
        <div class="category-panel" style="background: rgba(9,9,11,0.78); border-color: {catColor.icon}; box-shadow: 0 18px 40px rgba(0,0,0,0.32);">
          <!-- Header -->
          <div class="category-panel__header" style="background: {catColor.icon};">
            <div class="category-panel__header-row">
              <div class="category-panel__header-copy">
                <h3 class="category-panel__title">{formatCategoryName(expandedCategoryId)}</h3>
                <p class="category-panel__summary">
                  {#if selectedCategory?.subscribed}
                    {blockedInCategory.length} blocked
                    {#if exceptedInCategory.length > 0}
                      <span class="opacity-70">· {exceptedInCategory.length} excepted</span>
                    {/if}
                  {:else}
                    {categoryArtists.length} artists available
                  {/if}
                </p>
              </div>
              <button
                type="button"
                on:click={() => { expandedCategoryId = null; categoryArtists = []; }}
                class="category-panel__close"
              >
                <svg class="category-panel__close-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            <!-- Action Buttons -->
            <div class="category-panel__actions">
              {#if selectedCategory}
                <button
                  type="button"
                  class="category-panel__action"
                  style="background: {selectedCategory.subscribed ? 'rgba(0,0,0,0.3)' : 'white'}; color: {selectedCategory.subscribed ? 'white' : catColor.icon};"
                  on:click={(e) => toggleCategorySubscription(selectedCategory, e)}
                >
                  {selectedCategory.subscribed ? 'Unsubscribe' : 'Block All'}
                </button>
              {/if}
              {#if exceptedInCategory.length > 0 && selectedCategory?.subscribed}
                <button
                  type="button"
                  class="category-panel__action category-panel__action--secondary"
                  style="background: rgba(255,255,255,0.2); color: white;"
                  on:click={() => {
                    exceptedInCategory.forEach(a => exceptedArtists.delete(a.id));
                    exceptedArtists = exceptedArtists;
                    saveExceptions();
                  }}
                >
                  Re-block All ({exceptedInCategory.length})
                </button>
              {/if}
            </div>
          </div>

          <!-- Artists Grid -->
          <div class="category-panel__body">
            {#if isLoadingCategoryArtists}
              <div class="home__loading home__loading--compact">
                <div class="home__spinner home__spinner--medium"></div>
              </div>
            {:else if categoryArtists.length > 0}
              <!-- Blocked Artists -->
              {#if blockedInCategory.length > 0 && selectedCategory?.subscribed}
                <div class="category-panel__group">
                  <p class="category-panel__group-label">Blocked</p>
                  <div class="category-panel__grid">
                    {#each blockedInCategory as artist}
                      {@const sev = getSeverityStyle(artist.severity)}
                      <div
                        class="artist-tile"
                        style="background: rgba(0,0,0,0.3);"
                      >
                        <button
                          type="button"
                          class="artist-tile__link"
                          on:click={() => goToArtist(artist.id, artist.name)}
                        >
                          <div class="artist-tile__head">
                            <p class="artist-tile__name">{artist.name}</p>
                            <EnforcementBadges artistId={artist.id} compact={true} />
                          </div>
                          <p class="artist-tile__severity" style="color: {artist.severity === 'egregious' ? '#fecaca' : artist.severity === 'severe' ? '#fed7aa' : artist.severity === 'moderate' ? '#fef08a' : '#a1a1aa'};">
                            {sev.label}
                          </p>
                        </button>
                        <button
                          type="button"
                          class="artist-tile__dismiss"
                          style="background: rgba(0,0,0,0.5); color: #a1a1aa;"
                          on:click={(e) => unblockArtist(artist.id, e, artist.name)}
                          title="Unblock this artist"
                        >
                          <svg class="artist-tile__dismiss-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                          </svg>
                        </button>
                      </div>
                    {/each}
                  </div>
                </div>
              {/if}

              <!-- Excepted Artists -->
              {#if exceptedInCategory.length > 0 && selectedCategory?.subscribed}
                <div class="category-panel__group category-panel__group--divided" style="border-top: 1px solid #3f3f46;">
                  <p class="category-panel__group-label category-panel__group-label--muted">Not Blocked (Excepted)</p>
                  <div class="category-panel__grid">
                    {#each exceptedInCategory as artist}
                      <div
                        class="artist-tile artist-tile--excepted"
                        style="background: rgba(0,0,0,0.15); border: 1px dashed #52525b;"
                      >
                        <button
                          type="button"
                          class="artist-tile__link"
                          on:click={() => goToArtist(artist.id, artist.name)}
                        >
                          <p class="artist-tile__name artist-tile__name--muted">{artist.name}</p>
                          <p class="artist-tile__status">Excepted</p>
                        </button>
                        <button
                          type="button"
                          class="artist-tile__reblock"
                          style="background: {catColor.icon}; color: white;"
                          on:click={(e) => reblockArtist(artist.id, e, artist.name)}
                          title="Re-block this artist"
                        >
                          Block
                        </button>
                      </div>
                    {/each}
                  </div>
                </div>
              {/if}

              <!-- Not Subscribed - Show all artists -->
              {#if !selectedCategory?.subscribed}
                <div class="category-panel__grid">
                  {#each categoryArtists as artist}
                    {@const sev = getSeverityStyle(artist.severity)}
                    <button
                      type="button"
                      class="artist-tile artist-tile--catalog"
                      style="background: rgba(0,0,0,0.3);"
                      on:click={() => goToArtist(artist.id, artist.name)}
                    >
                      <p class="artist-tile__name">{artist.name}</p>
                      <p class="artist-tile__severity" style="color: {artist.severity === 'egregious' ? '#fecaca' : artist.severity === 'severe' ? '#fed7aa' : artist.severity === 'moderate' ? '#fef08a' : '#a1a1aa'};">
                        {sev.label}
                      </p>
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

  <!-- Your Blocked Artists -->
  <section
    class="home__panel"
    style="box-shadow: 0 20px 60px rgba(0, 0, 0, 0.32);"
  >
    <div class="home__section-head">
      <h2 class="home__section-title">
        Your Blocked Artists
        {#if uniqueBlockedArtists.length > 0}
          <span class="home__section-count">({uniqueBlockedArtists.length})</span>
        {/if}
      </h2>
      <span class="home__section-kicker">Live blocklist</span>
    </div>

    {#if isLoadingBlocked}
      <div class="home__loading home__loading--compact">
        <div class="home__spinner home__spinner--medium"></div>
      </div>
    {:else if uniqueBlockedArtists.length === 0}
      <div class="home__empty-state">
        <div
          class="home__empty-state-icon"
          style="background:
            radial-gradient(circle at 30% 30%, rgba(255,255,255,0.12), transparent 36%),
            linear-gradient(145deg, rgba(244, 63, 94, 1), rgba(225, 29, 72, 1)); box-shadow: 0 10px 24px rgba(244, 63, 94, 0.22);"
        >
          <svg class="home__empty-state-icon-svg" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
          </svg>
        </div>
        <p class="home__empty-state-title">No artists blocked yet</p>
        <p class="home__empty-state-copy">Toggle categories above to start shaping your feed.</p>
      </div>
    {:else}
      <div class="home__chip-list">
        {#each uniqueBlockedArtists as artist}
          <button
            type="button"
            class="blocked-chip"
            style="background: rgba(255,255,255,0.03); border: 1px solid rgba(255,255,255,0.08);"
            data-testid="blocked-artist-chip"
            on:click={() => goToArtist(artist.id, artist.name)}
            title="View artist profile"
          >
            <span
              class="blocked-chip__name"
              style="color: #e4e4e7;"
              data-testid="blocked-artist-name"
            >
              {artist.name}
            </span>
            <!-- Enforcement badges -->
            <EnforcementBadges artistId={artist.id} compact={true} />
            <span
              role="button"
              tabindex="0"
              class="blocked-chip__remove"
              style="color: #a1a1aa;"
              on:click={(e) => unblockArtist(artist.id, e, artist.name)}
              on:keydown={(e) => e.key === 'Enter' && unblockArtist(artist.id, e, artist.name)}
              title="Remove from blocklist"
              data-testid="unblock-artist-button"
            >
              <svg class="blocked-chip__remove-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </span>
          </button>
        {/each}
      </div>
    {/if}

  </section>
</div>

<style>
  .home {
    width: min(1100px, calc(100vw - 2rem));
    margin: 0 auto;
    padding: 1.5rem 0 2rem;
    display: grid;
    gap: 1.75rem;
  }

  .home__hero-section,
  .home__panel {
    margin: 0;
  }

  .home__hero-card,
  .home__panel {
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 1.75rem;
    backdrop-filter: blur(20px);
  }

  .home__hero-card {
    padding: 1.6rem;
  }

  .home__hero-badges {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
  }

  .home__hero-badge {
    display: inline-flex;
    align-items: center;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.05);
    padding: 0.65rem 1rem;
    font-size: 0.73rem;
    font-weight: 700;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: #a1a1aa;
  }

  .home__hero-badge--brand {
    border-color: rgba(244, 63, 94, 0.24);
    background: rgba(244, 63, 94, 0.12);
    color: #fda4af;
  }

  .home__hero-grid {
    display: grid;
    gap: 2rem;
    margin-top: 1.75rem;
  }

  .home__hero-copy {
    max-width: 42rem;
  }

  .home__brand-kicker {
    margin: 0 0 0.9rem;
    font-size: 0.76rem;
    font-weight: 800;
    letter-spacing: 0.18em;
    text-transform: uppercase;
    color: #fda4af;
  }

  .home__hero-title {
    margin: 0;
    max-width: 11ch;
    font-size: clamp(2.5rem, 4vw, 4.5rem);
    font-weight: 900;
    line-height: 0.95;
    letter-spacing: -0.05em;
    color: #fafafa;
  }

  .home__hero-subtitle {
    margin: 1.25rem 0 0;
    max-width: 40rem;
    font-size: 1.02rem;
    line-height: 1.85;
    color: #d4d4d8;
  }

  .home__hero-metrics {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: 0.85rem;
    align-content: end;
  }

  .home__metric-card {
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 1.25rem;
    background: rgba(255, 255, 255, 0.04);
    padding: 1rem;
  }

  .home__metric-value {
    display: block;
    font-size: 2rem;
    font-weight: 800;
    line-height: 1;
    color: #fafafa;
  }

  .home__metric-label {
    display: block;
    margin-top: 0.6rem;
    font-size: 0.7rem;
    font-weight: 700;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: #71717a;
  }

  .home__search {
    position: relative;
    margin-top: 1.5rem;
    max-width: 44rem;
  }

  .home__search-icon {
    position: absolute;
    left: 1rem;
    top: 50%;
    z-index: 1;
    transform: translateY(-50%);
    color: #71717a;
  }

  .home__search-icon-svg {
    width: 1.15rem;
    height: 1.15rem;
  }

  .home__search-input {
    width: 100%;
    box-sizing: border-box;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 1.2rem;
    background: rgba(255, 255, 255, 0.04);
    color: #fafafa;
    padding: 1rem 1rem 1rem 3rem;
    font-size: 1rem;
    transition: border-color 160ms ease, box-shadow 160ms ease, background 160ms ease;
  }

  .home__search-input::placeholder {
    color: #71717a;
  }

  .home__search-input:focus {
    outline: none;
    border-color: rgba(244, 63, 94, 0.32);
    box-shadow: 0 0 0 4px rgba(244, 63, 94, 0.12);
    background: rgba(255, 255, 255, 0.05);
  }

  .home__search-results {
    position: absolute;
    top: calc(100% + 0.75rem);
    left: 0;
    right: 0;
    z-index: 20;
    overflow: hidden;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 1.2rem;
    background: rgba(9, 9, 11, 0.95);
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.34);
    backdrop-filter: blur(20px);
  }

  .home__search-results-list {
    max-height: 20rem;
    overflow-y: auto;
  }

  .home__search-result {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 0.85rem;
    border: 0;
    background: transparent;
    padding: 0.9rem 1rem;
    text-align: left;
    cursor: pointer;
    transition: background 160ms ease;
  }

  .home__search-result:hover {
    background: rgba(255, 255, 255, 0.04);
  }

  .home__search-result--separated {
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  }

  .home__search-result-copy {
    flex: 1;
    min-width: 0;
  }

  .home__search-result-head {
    display: flex;
    align-items: center;
    gap: 0.55rem;
  }

  .home__search-result-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-weight: 600;
    color: #fafafa;
  }

  .home__search-result-badge {
    flex-shrink: 0;
    padding: 0.25rem 0.55rem;
    border-radius: 999px;
    background: rgba(244, 63, 94, 0.16);
    font-size: 0.72rem;
    font-weight: 700;
    color: #fda4af;
  }

  .home__search-result-meta {
    margin: 0.32rem 0 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 0.88rem;
    color: #a1a1aa;
  }

  .home__search-result-arrow {
    width: 1rem;
    height: 1rem;
    flex-shrink: 0;
    color: #71717a;
  }

  .home__search-empty {
    padding: 1.25rem 1rem;
    text-align: center;
  }

  .home__search-empty-text {
    margin: 0;
    color: #a1a1aa;
  }

  .home__search-empty-query {
    color: #fafafa;
  }

  .home__panel {
    background: rgba(9, 9, 11, 0.72);
    padding: 1.4rem;
  }

  .home__section-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    margin-bottom: 1rem;
  }

  .home__section-title {
    margin: 0;
    font-size: 1.2rem;
    font-weight: 700;
    color: #fafafa;
  }

  .home__section-count {
    color: #71717a;
    font-weight: 500;
  }

  .home__section-kicker {
    font-size: 0.72rem;
    font-weight: 700;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: #71717a;
  }

  .home__loading {
    display: flex;
    justify-content: center;
    padding: 3rem 0;
  }

  .home__loading--compact {
    padding: 2rem 0;
  }

  .home__spinner {
    width: 2rem;
    height: 2rem;
    border: 2px solid rgba(255, 255, 255, 0.12);
    border-top-color: #fda4af;
    border-radius: 999px;
    animation: home-spin 0.9s linear infinite;
  }

  .home__spinner--small {
    width: 1.1rem;
    height: 1.1rem;
  }

  .home__spinner--medium {
    width: 1.5rem;
    height: 1.5rem;
  }

  .home__category-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
    gap: 0.8rem;
  }

  .category-card {
    display: flex;
    align-items: center;
    gap: 0.7rem;
    border-radius: 1.1rem;
    padding: 0.7rem 0.8rem;
    transition: transform 160ms ease, box-shadow 160ms ease, border-color 160ms ease;
  }

  .category-card:hover {
    transform: translateY(-1px);
  }

  .category-card__toggle,
  .category-card__content,
  .category-panel__close,
  .category-panel__action,
  .artist-tile__link,
  .artist-tile__dismiss,
  .artist-tile__reblock,
  .artist-tile--catalog,
  .blocked-chip,
  .blocked-chip__remove {
    cursor: pointer;
  }

  .category-card__toggle {
    width: 2rem;
    height: 2rem;
    display: grid;
    place-items: center;
    border: 0;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.06);
  }

  .category-card__toggle-icon {
    width: 0.95rem;
    height: 0.95rem;
  }

  .category-card__toggle-icon--muted {
    color: #71717a;
  }

  .category-card__content {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    border: 0;
    padding: 0;
    background: transparent;
    color: inherit;
    text-align: left;
  }

  .category-card__name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 0.9rem;
    font-weight: 600;
  }

  .category-card__count {
    flex-shrink: 0;
    font-size: 0.78rem;
    color: rgba(255, 255, 255, 0.58);
    font-variant-numeric: tabular-nums;
  }

  .category-panel {
    margin-top: 1rem;
    overflow: hidden;
    border: 1px solid;
    border-radius: 1.6rem;
  }

  .category-panel__header {
    padding: 1.2rem 1.25rem 1rem;
  }

  .category-panel__header-row {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 1rem;
  }

  .category-panel__header-copy {
    min-width: 0;
  }

  .category-panel__title {
    margin: 0;
    font-size: 1.35rem;
    font-weight: 800;
    color: #fff;
  }

  .category-panel__summary {
    margin: 0.4rem 0 0;
    font-size: 0.95rem;
    color: rgba(255, 255, 255, 0.82);
  }

  .category-panel__close {
    width: 2.4rem;
    height: 2.4rem;
    display: grid;
    place-items: center;
    border: 0;
    border-radius: 999px;
    background: rgba(0, 0, 0, 0.18);
    color: #fff;
    transition: background 160ms ease;
  }

  .category-panel__close:hover {
    background: rgba(0, 0, 0, 0.28);
  }

  .category-panel__close-icon {
    width: 1rem;
    height: 1rem;
  }

  .category-panel__actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.65rem;
    margin-top: 1rem;
  }

  .category-panel__action {
    border: 0;
    border-radius: 999px;
    padding: 0.75rem 1rem;
    font-size: 0.88rem;
    font-weight: 700;
    transition: transform 160ms ease, opacity 160ms ease;
  }

  .category-panel__action:hover {
    transform: translateY(-1px);
  }

  .category-panel__body {
    max-height: 24rem;
    overflow-y: auto;
    padding: 1.25rem;
  }

  .category-panel__group {
    margin-bottom: 1.25rem;
  }

  .category-panel__group:last-child {
    margin-bottom: 0;
  }

  .category-panel__group--divided {
    padding-top: 1.25rem;
  }

  .category-panel__group-label {
    margin: 0 0 0.85rem;
    font-size: 0.72rem;
    font-weight: 700;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: #d4d4d8;
  }

  .category-panel__group-label--muted {
    color: #71717a;
  }

  .category-panel__grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 0.75rem;
  }

  .artist-tile,
  .artist-tile--catalog {
    position: relative;
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 1rem;
    padding: 0.85rem;
    transition: transform 160ms ease, border-color 160ms ease, background 160ms ease;
  }

  .artist-tile:hover,
  .artist-tile--catalog:hover {
    transform: translateY(-1px);
    border-color: rgba(255, 255, 255, 0.16);
  }

  .artist-tile__link,
  .artist-tile--catalog {
    width: 100%;
    display: block;
    border: 0;
    padding: 0;
    background: transparent;
    text-align: left;
    color: inherit;
  }

  .artist-tile__head {
    display: flex;
    align-items: center;
    gap: 0.45rem;
  }

  .artist-tile__name {
    margin: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 0.92rem;
    font-weight: 600;
    color: #fafafa;
  }

  .artist-tile__name--muted {
    color: #a1a1aa;
  }

  .artist-tile__severity,
  .artist-tile__status {
    margin: 0.35rem 0 0;
    font-size: 0.78rem;
  }

  .artist-tile__status {
    color: #71717a;
  }

  .artist-tile__dismiss {
    position: absolute;
    top: 0.55rem;
    right: 0.55rem;
    width: 1.6rem;
    height: 1.6rem;
    display: grid;
    place-items: center;
    border: 0;
    border-radius: 999px;
    opacity: 0;
    transition: opacity 160ms ease, background 160ms ease;
  }

  .artist-tile:hover .artist-tile__dismiss {
    opacity: 1;
  }

  .artist-tile__dismiss-icon {
    width: 0.78rem;
    height: 0.78rem;
  }

  .artist-tile__reblock {
    position: absolute;
    top: 0.55rem;
    right: 0.55rem;
    border: 0;
    border-radius: 999px;
    padding: 0.28rem 0.55rem;
    font-size: 0.7rem;
    font-weight: 700;
    opacity: 0;
    transition: opacity 160ms ease, transform 160ms ease;
  }

  .artist-tile--excepted:hover .artist-tile__reblock {
    opacity: 1;
    transform: translateY(-1px);
  }

  .category-panel__empty {
    margin: 0;
    padding: 2rem 0;
    text-align: center;
    color: #a1a1aa;
  }

  .home__empty-state {
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 1.25rem;
    background: rgba(255, 255, 255, 0.04);
    padding: 2.25rem 1.25rem;
    text-align: center;
  }

  .home__empty-state-icon {
    display: grid;
    place-items: center;
    width: 3rem;
    height: 3rem;
    margin: 0 auto 1rem;
    border-radius: 999px;
  }

  .home__empty-state-icon-svg {
    width: 1.2rem;
    height: 1.2rem;
    color: #fff;
  }

  .home__empty-state-title {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 700;
    color: #fafafa;
  }

  .home__empty-state-copy {
    margin: 0.55rem 0 0;
    font-size: 0.92rem;
    color: #a1a1aa;
  }

  .home__chip-list {
    display: flex;
    flex-wrap: wrap;
    gap: 0.65rem;
  }

  .blocked-chip {
    display: inline-flex;
    align-items: center;
    gap: 0.45rem;
    border-radius: 999px;
    padding: 0.55rem 0.8rem;
    transition: background 160ms ease, border-color 160ms ease, transform 160ms ease;
  }

  .blocked-chip:hover {
    transform: translateY(-1px);
  }

  .blocked-chip__name {
    font-size: 0.88rem;
    font-weight: 600;
  }

  .blocked-chip__remove {
    display: grid;
    place-items: center;
    width: 1rem;
    height: 1rem;
    border-radius: 999px;
    opacity: 0.45;
    transition: opacity 160ms ease, background 160ms ease;
  }

  .blocked-chip__remove:hover {
    opacity: 1;
    background: rgba(255, 255, 255, 0.08);
  }

  .blocked-chip__remove-icon {
    width: 0.72rem;
    height: 0.72rem;
  }

  @keyframes home-spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  @media (min-width: 960px) {
    .home__hero-grid {
      grid-template-columns: minmax(0, 1.45fr) minmax(15rem, 0.85fr);
      align-items: end;
    }
  }

  @media (max-width: 719px) {
    .home {
      width: min(100vw - 1.25rem, 1100px);
      padding-top: 1rem;
      gap: 1rem;
    }

    .home__hero-card,
    .home__panel {
      padding: 1.1rem;
      border-radius: 1.35rem;
    }

    .home__hero-title {
      font-size: 2.2rem;
    }

    .home__hero-subtitle {
      font-size: 0.94rem;
      line-height: 1.7;
    }

    .home__section-head {
      flex-direction: column;
      align-items: flex-start;
    }

    .category-panel__header-row {
      align-items: center;
    }
  }
</style>
