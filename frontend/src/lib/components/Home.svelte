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

<div class="max-w-6xl mx-auto px-6 py-8">
  <!-- Hero Search Section -->
  <section class="mb-12">
    <h1 class="text-4xl md:text-5xl font-bold text-white mb-3">Clean Your Feed</h1>
    <p class="text-zinc-300 text-lg mb-8">Search and block artists with documented misconduct</p>

    <div class="relative max-w-2xl">
      <div class="absolute left-4 top-1/2 -translate-y-1/2 text-zinc-400 z-10">
        {#if isSearching}
          <div class="w-5 h-5 border-2 border-zinc-600 border-t-white rounded-full animate-spin"></div>
        {:else}
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
        {/if}
      </div>
      <input
        type="text"
        bind:value={searchQuery}
        on:input={handleSearchInput}
        placeholder="Search any artist..."
        class="w-full pl-12 pr-4 py-4 rounded-full bg-zinc-900 border border-zinc-700 text-white placeholder-neutral-500 focus:outline-none focus:border-white focus:ring-1 focus:ring-white transition-all"
      />

      <!-- Search Results Dropdown -->
      {#if searchResults.length > 0 || (searchQuery.length > 1 && !isSearching)}
        <div class="absolute top-full left-0 right-0 mt-2 rounded-2xl bg-zinc-900 border border-zinc-700 overflow-hidden shadow-xl z-20">
          {#if searchResults.length > 0}
            <div class="max-h-80 overflow-y-auto">
              {#each searchResults as artist, i}
                <button
                  type="button"
                  class="w-full flex items-center gap-3 px-4 py-3 text-left transition-all hover:bg-zinc-800 {i !== searchResults.length - 1 ? 'border-b border-zinc-800' : ''}"
                  on:click={() => goToArtist(artist.id, artist.canonical_name)}
                >
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2">
                      <span class="font-medium text-white">{artist.canonical_name}</span>
                      {#if artist.has_offenses}
                        <span class="flex-shrink-0 px-2 py-0.5 text-xs font-medium rounded-full bg-rose-500/20 text-rose-400">
                          {artist.offense_count} offense{artist.offense_count !== 1 ? 's' : ''}
                        </span>
                      {/if}
                    </div>
                    {#if artist.genres && artist.genres.length > 0}
                      <p class="text-sm text-zinc-400 truncate">{artist.genres.slice(0, 2).join(', ')}</p>
                    {/if}
                  </div>
                  <svg class="w-5 h-5 text-zinc-500 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                  </svg>
                </button>
              {/each}
            </div>
          {:else}
            <div class="px-4 py-6 text-center">
              <p class="text-zinc-400">No artists found for "<span class="text-white">{searchQuery}</span>"</p>
            </div>
          {/if}
        </div>
      {/if}
    </div>
  </section>

  <!-- Categories -->
  <section class="mb-8">
    <h2 class="text-lg font-semibold text-white mb-4">Block by Category</h2>

    {#if isLoadingCategories}
      <div class="flex justify-center py-12">
        <div class="w-8 h-8 border-2 border-zinc-700 border-t-white rounded-full animate-spin"></div>
      </div>
    {:else}
      <!-- Category Cards - Grid layout for consistent 2+ rows -->
      <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-3">
        {#each categoryLists as category}
          {@const catColor = getCategoryColor(category.id)}
          {@const isExpanded = expandedCategoryId === category.id}
          <div
            class="flex items-center gap-2 px-3 py-2.5 rounded-xl transition-all font-medium"
            style="background: {isExpanded ? '#3f3f46' : '#27272a'}; border: 2px solid {category.subscribed ? catColor.icon : isExpanded ? '#fff' : '#3f3f46'}; color: white;"
          >
            <!-- Toggle -->
            <button
              type="button"
              class="flex-shrink-0 flex items-center justify-center transition-all"
              on:click={(e) => { e.stopPropagation(); toggleCategorySubscription(category, e); }}
              title="{category.subscribed ? 'Unblock' : 'Block'} all {formatCategoryName(category.id)} artists"
            >
              {#if category.subscribed}
                <svg class="w-4 h-4" style="color: {catColor.icon};" fill="currentColor" viewBox="0 0 24 24">
                  <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
                </svg>
              {:else}
                <svg class="w-4 h-4 text-zinc-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v12m6-6H6"/>
                </svg>
              {/if}
            </button>

            <!-- Category Name - Click to expand -->
            <button
              type="button"
              class="flex-1 flex items-center justify-between gap-2 min-w-0 text-left"
              on:click={() => toggleCategory(category.id)}
              title="View {formatCategoryName(category.id)} artists"
            >
              <span class="truncate text-sm">{formatCategoryName(category.id)}</span>
              <span class="flex-shrink-0 text-xs opacity-50 tabular-nums">{category.artist_count}</span>
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
        <div class="mt-4 rounded-2xl overflow-hidden" style="background: #18181b; border: 2px solid {catColor.icon};">
          <!-- Header -->
          <div class="px-5 py-4" style="background: {catColor.icon};">
            <div class="flex items-center justify-between">
              <div>
                <h3 class="text-xl font-bold text-white">{formatCategoryName(expandedCategoryId)}</h3>
                <p class="text-white/80 text-sm mt-1">
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
                class="p-2 rounded-full text-white hover:bg-black/20"
              >
                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            <!-- Action Buttons -->
            <div class="flex flex-wrap items-center gap-2 mt-3">
              {#if selectedCategory}
                <button
                  type="button"
                  class="px-4 py-2 rounded-full text-sm font-semibold transition-all"
                  style="background: {selectedCategory.subscribed ? 'rgba(0,0,0,0.3)' : 'white'}; color: {selectedCategory.subscribed ? 'white' : catColor.icon};"
                  on:click={(e) => toggleCategorySubscription(selectedCategory, e)}
                >
                  {selectedCategory.subscribed ? 'Unsubscribe' : 'Block All'}
                </button>
              {/if}
              {#if exceptedInCategory.length > 0 && selectedCategory?.subscribed}
                <button
                  type="button"
                  class="px-4 py-2 rounded-full text-sm font-semibold transition-all"
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
          <div class="max-h-96 overflow-y-auto p-5">
            {#if isLoadingCategoryArtists}
              <div class="flex justify-center py-8">
                <div class="w-6 h-6 border-2 border-zinc-600 border-t-white rounded-full animate-spin"></div>
              </div>
            {:else if categoryArtists.length > 0}
              <!-- Blocked Artists -->
              {#if blockedInCategory.length > 0 && selectedCategory?.subscribed}
                <div class="mb-4">
                  <p class="text-xs uppercase tracking-wider text-zinc-400 mb-3">Blocked</p>
                  <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-3">
                    {#each blockedInCategory as artist}
                      {@const sev = getSeverityStyle(artist.severity)}
                      <div
                        class="group relative p-3 rounded-xl text-left transition-all hover:scale-[1.02]"
                        style="background: rgba(0,0,0,0.3);"
                      >
                        <button
                          type="button"
                          class="w-full text-left"
                          on:click={() => goToArtist(artist.id, artist.name)}
                        >
                          <div class="flex items-center gap-1.5">
                            <p class="font-medium text-white text-sm truncate">{artist.name}</p>
                            <EnforcementBadges artistId={artist.id} compact={true} />
                          </div>
                          <p class="text-xs mt-1" style="color: {artist.severity === 'egregious' ? '#fecaca' : artist.severity === 'severe' ? '#fed7aa' : artist.severity === 'moderate' ? '#fef08a' : '#a1a1aa'};">
                            {sev.label}
                          </p>
                        </button>
                        <button
                          type="button"
                          class="absolute top-2 right-2 p-1 rounded opacity-0 group-hover:opacity-100 transition-opacity"
                          style="background: rgba(0,0,0,0.5); color: #a1a1aa;"
                          on:click={(e) => unblockArtist(artist.id, e, artist.name)}
                          title="Unblock this artist"
                        >
                          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
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
                <div class="pt-4" style="border-top: 1px solid #3f3f46;">
                  <p class="text-xs uppercase tracking-wider text-zinc-500 mb-3">Not Blocked (Excepted)</p>
                  <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-3">
                    {#each exceptedInCategory as artist}
                      <div
                        class="group relative p-3 rounded-xl text-left transition-all"
                        style="background: rgba(0,0,0,0.15); border: 1px dashed #52525b;"
                      >
                        <button
                          type="button"
                          class="w-full text-left"
                          on:click={() => goToArtist(artist.id, artist.name)}
                        >
                          <p class="font-medium text-zinc-400 text-sm truncate">{artist.name}</p>
                          <p class="text-xs mt-1 text-zinc-500">Excepted</p>
                        </button>
                        <button
                          type="button"
                          class="absolute top-2 right-2 px-2 py-0.5 rounded text-xs font-medium opacity-0 group-hover:opacity-100 transition-opacity"
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
                <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-3">
                  {#each categoryArtists as artist}
                    {@const sev = getSeverityStyle(artist.severity)}
                    <button
                      type="button"
                      class="p-3 rounded-xl text-left transition-all hover:scale-[1.02]"
                      style="background: rgba(0,0,0,0.3);"
                      on:click={() => goToArtist(artist.id, artist.name)}
                    >
                      <p class="font-medium text-white text-sm truncate">{artist.name}</p>
                      <p class="text-xs mt-1" style="color: {artist.severity === 'egregious' ? '#fecaca' : artist.severity === 'severe' ? '#fed7aa' : artist.severity === 'moderate' ? '#fef08a' : '#a1a1aa'};">
                        {sev.label}
                      </p>
                    </button>
                  {/each}
                </div>
              {/if}
            {:else}
              <p class="text-center text-zinc-400 py-8">No artists in this category yet</p>
            {/if}
          </div>
        </div>
      {/if}
    {/if}
  </section>

  <!-- Your Blocked Artists -->
  <section class="mb-8">
    <h2 class="text-lg font-semibold text-white mb-4">
      Your Blocked Artists
      {#if uniqueBlockedArtists.length > 0}
        <span class="text-zinc-400 font-normal">({uniqueBlockedArtists.length})</span>
      {/if}
    </h2>

    {#if isLoadingBlocked}
      <div class="flex justify-center py-8">
        <div class="w-6 h-6 border-2 border-zinc-700 border-t-white rounded-full animate-spin"></div>
      </div>
    {:else if uniqueBlockedArtists.length === 0}
      <div class="text-center py-6 px-4 rounded-2xl bg-zinc-900 border border-zinc-800">
        <p class="text-zinc-300">No artists blocked yet</p>
        <p class="text-zinc-400 text-sm mt-1">Toggle categories above to start blocking</p>
      </div>
    {:else}
      <div class="flex flex-wrap gap-2">
        {#each uniqueBlockedArtists as artist}
          <button
            type="button"
            class="group flex items-center gap-1.5 pl-3 pr-2 py-1.5 rounded-full transition-all hover:bg-zinc-700/50"
            style="background: transparent; border: 1px solid #52525b;"
            data-testid="blocked-artist-chip"
            on:click={() => goToArtist(artist.id, artist.name)}
            title="View artist profile"
          >
            <span
              class="text-sm font-medium"
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
              class="flex items-center justify-center w-4 h-4 rounded-full opacity-40 hover:opacity-100 hover:bg-zinc-600 transition-all"
              style="color: #a1a1aa;"
              on:click={(e) => unblockArtist(artist.id, e, artist.name)}
              on:keydown={(e) => e.key === 'Enter' && unblockArtist(artist.id, e, artist.name)}
              title="Remove from blocklist"
              data-testid="unblock-artist-button"
            >
              <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </span>
          </button>
        {/each}
      </div>
    {/if}

  </section>
</div>
