<script lang="ts">
  import { onMount } from 'svelte';
  import { apiClient } from '../utils/api-client';
  import { navigateTo } from '../utils/simple-router';

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
  }

  interface BlockedArtist {
    id: string;
    name: string;
    category: string;
    severity: string;
  }

  interface ArtistOffense {
    id: string;
    category: string;
    severity: string;
    title: string;
    description: string;
    incident_date?: string;
    status: string;
    evidence: Evidence[];
  }

  interface Evidence {
    id: string;
    source_url: string;
    source_name: string;
    source_type: string;
    title?: string;
    excerpt?: string;
    published_date?: string;
    credibility_score?: number;
  }

  interface ArtistDetail {
    id: string;
    canonical_name: string;
    genres?: string[];
    image_url?: string;
    offenses: ArtistOffense[];
  }

  type View = 'home' | 'artist';
  let currentView: View = 'home';

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

  let selectedArtist: ArtistDetail | null = null;
  let isLoadingArtistDetail = false;

  let blockingArtistId: string | null = null;
  let dnpList: Set<string> = new Set();

  // Triadic palette colors
  const colors = {
    green: { primary: '#30AF22', light: '#59BA48' },
    blue: { primary: '#009BD7', light: '#00B4FF' },
    pink: { primary: '#FF2C6E', light: '#FF728F' }
  };

  function formatCategoryName(id: string): string {
    return id.split('_').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ');
  }

  function getSeverityStyle(severity: string): { bg: string; text: string; label: string } {
    switch (severity) {
      case 'egregious':
        return { bg: 'background-color: #FFE5ED;', text: 'color: #FF2C6E;', label: 'Egregious' };
      case 'severe':
        return { bg: 'background-color: #FFECF1;', text: 'color: #FF728F;', label: 'Severe' };
      case 'moderate':
        return { bg: 'background-color: #E5F6FF;', text: 'color: #009BD7;', label: 'Moderate' };
      default:
        return { bg: 'background-color: #E8F8E6;', text: 'color: #30AF22;', label: 'Minor' };
    }
  }

  onMount(async () => {
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
        const result = await apiClient.get<SearchArtist[]>(`/api/v1/artists/search?q=${encodeURIComponent(searchQuery.trim())}`);
        if (result.success && result.data) {
          searchResults = result.data;
        } else {
          searchResults = [];
        }
      } catch (e) {
        searchResults = [];
      } finally {
        isSearching = false;
      }
    }, 150);
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
    } catch (e) {
      categoryLists = categoryLists.map(c =>
        c.id === category.id ? { ...c, subscribed: wasSubscribed } : c
      );
    }
  }

  async function goToArtist(artistId: string, artistName: string) {
    currentView = 'artist';
    isLoadingArtistDetail = true;
    selectedArtist = null;
    try {
      const result = await apiClient.get<ArtistDetail>(`/api/v1/offenses/query?artist_id=${artistId}`);
      if (result.success && result.data) {
        selectedArtist = result.data;
      } else {
        selectedArtist = { id: artistId, canonical_name: artistName, offenses: [] };
      }
    } catch (e) {
      selectedArtist = { id: artistId, canonical_name: artistName, offenses: [] };
    } finally {
      isLoadingArtistDetail = false;
    }
  }

  function goBack() {
    currentView = 'home';
    selectedArtist = null;
    searchResults = [];
    searchQuery = '';
  }

  async function toggleArtistBlock(artistId: string) {
    blockingArtistId = artistId;
    const isCurrentlyBlocked = dnpList.has(artistId);
    try {
      if (isCurrentlyBlocked) {
        await apiClient.delete(`/api/v1/dnp/list/${artistId}`);
        dnpList.delete(artistId);
      } else {
        await apiClient.post('/api/v1/dnp/list', { artist_id: artistId });
        dnpList.add(artistId);
      }
      dnpList = new Set(dnpList);
    } catch (e) {
      console.error('Failed to toggle block:', e);
    } finally {
      blockingArtistId = null;
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      if (currentView === 'artist') {
        goBack();
      } else if (searchResults.length > 0) {
        searchResults = [];
        searchQuery = '';
      }
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="min-h-screen bg-white">
  {#if currentView === 'home'}
    <header class="sticky top-0 z-50 bg-white border-b border-gray-200">
      <div class="max-w-3xl mx-auto px-4 py-4 flex justify-between items-center">
        <div>
          <h1 class="text-xl font-semibold text-gray-900">No Drake in the House</h1>
          <p class="text-sm" style="color: #666;">Curate your streaming</p>
        </div>
        <div class="flex items-center gap-2">
          <button
            on:click={() => navigateTo('sync')}
            class="p-2 rounded-lg text-gray-500 hover:bg-gray-100 transition-colors"
            title="Catalog Sync"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
          </button>
          <button
            on:click={() => navigateTo('analytics')}
            class="p-2 rounded-lg text-gray-500 hover:bg-gray-100 transition-colors"
            title="Analytics"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
            </svg>
          </button>
          <button
            on:click={() => navigateTo('graph')}
            class="p-2 rounded-lg text-gray-500 hover:bg-gray-100 transition-colors"
            title="Graph Explorer"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
            </svg>
          </button>
          <button
            on:click={() => navigateTo('settings')}
            class="p-2 rounded-lg text-gray-500 hover:bg-gray-100 transition-colors"
            title="Settings"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
          </button>
        </div>
      </div>
    </header>

    <main class="max-w-3xl mx-auto px-4 py-6 space-y-8">
      <!-- Search -->
      <section>
        <div class="relative">
          <input
            type="text"
            bind:value={searchQuery}
            on:input={handleSearchInput}
            placeholder="Search artists..."
            class="w-full px-4 py-3 bg-gray-50 border border-gray-200 rounded-lg text-gray-900 placeholder-gray-400 focus:outline-none focus:border-transparent transition-all"
            style="focus:ring: 2px solid {colors.blue.primary};"
          />
          <div class="absolute right-3 top-1/2 -translate-y-1/2">
            {#if isSearching}
              <div class="w-5 h-5 border-2 rounded-full animate-spin" style="border-color: {colors.blue.primary}; border-top-color: transparent;"></div>
            {:else}
              <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
              </svg>
            {/if}
          </div>
        </div>

        {#if searchResults.length > 0}
          <div class="mt-2 bg-white border border-gray-200 rounded-lg shadow-lg overflow-hidden">
            {#each searchResults as artist, i}
              <button
                type="button"
                class="w-full flex items-center gap-3 px-4 py-3 text-left hover:bg-gray-50 transition-colors {i > 0 ? 'border-t border-gray-100' : ''}"
                on:click={() => goToArtist(artist.id, artist.canonical_name)}
              >
                <div class="w-10 h-10 rounded-full flex items-center justify-center text-sm font-medium text-white" style="background: linear-gradient(135deg, {colors.blue.primary}, {colors.blue.light});">
                  {artist.canonical_name.charAt(0)}
                </div>
                <div class="flex-1 min-w-0">
                  <p class="font-medium text-gray-900 truncate">{artist.canonical_name}</p>
                  {#if artist.genres && artist.genres.length > 0}
                    <p class="text-sm text-gray-500 truncate">{artist.genres.slice(0, 3).join(', ')}</p>
                  {/if}
                </div>
                <svg class="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                </svg>
              </button>
            {/each}
          </div>
        {:else if searchQuery.length > 1 && !isSearching}
          <div class="mt-2 bg-gray-50 border border-gray-200 rounded-lg p-6 text-center">
            <p class="text-gray-500">No artists found for "<span class="font-medium text-gray-700">{searchQuery}</span>"</p>
          </div>
        {/if}
      </section>

      <!-- Categories -->
      <section>
        <h2 class="text-lg font-semibold text-gray-900 mb-4">Block by Category</h2>

        {#if isLoadingCategories}
          <div class="flex justify-center py-12">
            <div class="w-6 h-6 border-2 rounded-full animate-spin" style="border-color: {colors.blue.primary}; border-top-color: transparent;"></div>
          </div>
        {:else}
          <div class="space-y-3">
            {#each categoryLists as category}
              <div>
                <button
                  type="button"
                  class="w-full p-4 bg-white border border-gray-200 rounded-lg hover:border-gray-300 hover:shadow-sm transition-all text-left"
                  style="{expandedCategoryId === category.id ? `border-color: ${colors.blue.primary}; box-shadow: 0 0 0 2px ${colors.blue.primary}20;` : ''}"
                  on:click={() => toggleCategory(category.id)}
                >
                  <div class="flex items-center justify-between">
                    <div class="flex items-center gap-3">
                      <div class="w-10 h-10 rounded-lg flex items-center justify-center" style="background: linear-gradient(135deg, {colors.pink.primary}15, {colors.pink.light}10);">
                        <svg class="w-5 h-5" style="color: {colors.pink.primary};" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                        </svg>
                      </div>
                      <div>
                        <p class="font-medium text-gray-900">{formatCategoryName(category.id)}</p>
                        <p class="text-sm text-gray-500">{category.artist_count} artists</p>
                      </div>
                    </div>
                    <button
                      type="button"
                      class="px-4 py-2 text-sm font-medium rounded-lg transition-all"
                      style="{category.subscribed ? `background: ${colors.pink.primary}; color: white;` : `background: white; border: 1px solid #e5e7eb; color: #374151;`}"
                      on:click={(e) => toggleCategorySubscription(category, e)}
                    >
                      {category.subscribed ? 'Blocking' : 'Block All'}
                    </button>
                  </div>
                </button>

                {#if expandedCategoryId === category.id}
                  <div class="mt-2 bg-white border border-gray-200 rounded-lg overflow-hidden">
                    {#if isLoadingCategoryArtists}
                      <div class="flex justify-center py-6">
                        <div class="w-5 h-5 border-2 rounded-full animate-spin" style="border-color: {colors.blue.primary}; border-top-color: transparent;"></div>
                      </div>
                    {:else if categoryArtists.length === 0}
                      <p class="text-center py-6 text-gray-500">No artists in this category</p>
                    {:else}
                      <div class="max-h-64 overflow-y-auto divide-y divide-gray-100">
                        {#each categoryArtists as artist}
                          {@const sev = getSeverityStyle(artist.severity)}
                          <button
                            type="button"
                            class="w-full flex items-center gap-3 px-4 py-3 text-left hover:bg-gray-50 transition-colors"
                            on:click={() => goToArtist(artist.id, artist.name)}
                          >
                            <div class="w-8 h-8 bg-gray-100 rounded-full flex items-center justify-center text-xs font-medium text-gray-600">
                              {artist.name.charAt(0)}
                            </div>
                            <p class="flex-1 font-medium text-gray-900 truncate">{artist.name}</p>
                            <span class="px-2 py-0.5 text-xs font-medium rounded" style="{sev.bg} {sev.text}">
                              {sev.label}
                            </span>
                            <svg class="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                            </svg>
                          </button>
                        {/each}
                      </div>
                    {/if}
                  </div>
                {/if}
              </div>
            {/each}
          </div>
        {/if}
      </section>

      <!-- Blocked Artists -->
      <section>
        <h2 class="text-lg font-semibold text-gray-900 mb-4">
          Your Blocked Artists
          {#if blockedArtists.length > 0}
            <span class="text-gray-500 font-normal">({blockedArtists.length})</span>
          {/if}
        </h2>

        {#if isLoadingBlocked}
          <div class="flex justify-center py-12">
            <div class="w-6 h-6 border-2 rounded-full animate-spin" style="border-color: {colors.blue.primary}; border-top-color: transparent;"></div>
          </div>
        {:else if blockedArtists.length === 0}
          <div class="bg-gray-50 border border-gray-200 rounded-lg p-10 text-center">
            <p class="text-gray-600">No artists blocked yet</p>
            <p class="text-sm text-gray-500 mt-1">Subscribe to categories or search for artists</p>
          </div>
        {:else}
          <div class="bg-white border border-gray-200 rounded-lg overflow-hidden">
            <div class="max-h-80 overflow-y-auto divide-y divide-gray-100">
              {#each blockedArtists as artist}
                {@const sev = getSeverityStyle(artist.severity)}
                <button
                  type="button"
                  class="w-full flex items-center gap-4 px-4 py-3 text-left hover:bg-gray-50 transition-colors"
                  on:click={() => goToArtist(artist.id, artist.name)}
                >
                  <div class="w-10 h-10 rounded-full flex items-center justify-center text-sm font-medium text-white" style="background: linear-gradient(135deg, {colors.pink.primary}, {colors.pink.light});">
                    {artist.name.charAt(0)}
                  </div>
                  <div class="flex-1 min-w-0">
                    <p class="font-medium text-gray-900 truncate">{artist.name}</p>
                    <p class="text-sm text-gray-500">{formatCategoryName(artist.category)}</p>
                  </div>
                  <span class="px-2 py-0.5 text-xs font-medium rounded" style="{sev.bg} {sev.text}">
                    {sev.label}
                  </span>
                  <svg class="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                  </svg>
                </button>
              {/each}
            </div>
          </div>
        {/if}
      </section>
    </main>

  {:else if currentView === 'artist'}
    <header class="sticky top-0 z-50 bg-white border-b border-gray-200">
      <div class="max-w-3xl mx-auto px-4 py-4">
        <button
          type="button"
          class="flex items-center gap-2 text-gray-500 hover:text-gray-900 transition-colors"
          on:click={goBack}
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
          </svg>
          Back
        </button>
      </div>
    </header>

    <main class="max-w-3xl mx-auto px-4 py-6">
      {#if isLoadingArtistDetail}
        <div class="flex justify-center py-20">
          <div class="w-8 h-8 border-2 rounded-full animate-spin" style="border-color: {colors.blue.primary}; border-top-color: transparent;"></div>
        </div>
      {:else if selectedArtist}
        <div class="bg-white border border-gray-200 rounded-lg overflow-hidden mb-6">
          <div class="p-6" style="background: linear-gradient(135deg, {colors.blue.primary}08, {colors.blue.light}05);">
            <div class="flex items-center gap-4">
              <div class="w-16 h-16 rounded-xl flex items-center justify-center text-2xl font-bold text-white" style="background: linear-gradient(135deg, {colors.blue.primary}, {colors.blue.light});">
                {selectedArtist.canonical_name.charAt(0)}
              </div>
              <div>
                <h1 class="text-2xl font-bold text-gray-900">{selectedArtist.canonical_name}</h1>
                {#if selectedArtist.genres && selectedArtist.genres.length > 0}
                  <p class="text-gray-500 mt-1">{selectedArtist.genres.join(', ')}</p>
                {/if}
              </div>
            </div>
          </div>
          <div class="px-6 py-4 border-t border-gray-100 flex items-center justify-between">
            <div>
              <p class="text-sm text-gray-500">Status</p>
              <p class="font-medium" style="color: {dnpList.has(selectedArtist.id) ? colors.pink.primary : colors.green.primary};">
                {dnpList.has(selectedArtist.id) ? 'Blocked' : 'Not Blocked'}
              </p>
            </div>
            <button
              type="button"
              on:click={() => selectedArtist && toggleArtistBlock(selectedArtist.id)}
              disabled={blockingArtistId === selectedArtist?.id}
              class="px-5 py-2.5 rounded-lg font-medium transition-all disabled:opacity-50"
              style="{dnpList.has(selectedArtist.id) ? 'background: #f3f4f6; color: #374151;' : `background: ${colors.pink.primary}; color: white;`}"
            >
              {#if blockingArtistId === selectedArtist?.id}
                <span class="inline-block w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin"></span>
              {:else if dnpList.has(selectedArtist.id)}
                Unblock
              {:else}
                Block Artist
              {/if}
            </button>
          </div>
        </div>

        <h2 class="text-lg font-semibold text-gray-900 mb-4">Documented Incidents</h2>

        {#if selectedArtist.offenses && selectedArtist.offenses.length > 0}
          <div class="space-y-4">
            {#each selectedArtist.offenses as offense}
              {@const sev = getSeverityStyle(offense.severity)}
              <div class="bg-white border border-gray-200 rounded-lg overflow-hidden">
                <div class="p-5">
                  <div class="flex items-center gap-2 mb-3">
                    <span class="px-2.5 py-1 text-xs font-medium rounded bg-gray-100 text-gray-700">
                      {formatCategoryName(offense.category)}
                    </span>
                    <span class="px-2.5 py-1 text-xs font-medium rounded" style="{sev.bg} {sev.text}">
                      {sev.label}
                    </span>
                  </div>
                  <h3 class="text-lg font-semibold text-gray-900 mb-2">{offense.title}</h3>
                  <p class="text-gray-600 leading-relaxed">{offense.description}</p>

                  {#if offense.incident_date}
                    <p class="text-sm text-gray-500 mt-4">
                      {new Date(offense.incident_date).toLocaleDateString('en-US', { year: 'numeric', month: 'long', day: 'numeric' })}
                    </p>
                  {/if}

                  {#if offense.evidence && offense.evidence.length > 0}
                    <div class="mt-5 pt-5 border-t border-gray-100">
                      <p class="text-xs font-medium text-gray-500 uppercase tracking-wider mb-3">
                        Sources ({offense.evidence.length})
                      </p>
                      <div class="space-y-2">
                        {#each offense.evidence as evidence}
                          <a
                            href={evidence.source_url}
                            target="_blank"
                            rel="noopener noreferrer"
                            class="flex items-center gap-3 p-3 bg-gray-50 rounded-lg hover:bg-gray-100 transition-colors"
                          >
                            <svg class="w-4 h-4 flex-shrink-0" style="color: {colors.blue.primary};" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                            </svg>
                            <div class="flex-1 min-w-0">
                              <p class="text-sm font-medium text-gray-900 truncate">
                                {evidence.title || evidence.source_name || 'View Source'}
                              </p>
                              {#if evidence.source_name && evidence.title}
                                <p class="text-xs text-gray-500">{evidence.source_name}</p>
                              {/if}
                            </div>
                            {#if evidence.credibility_score}
                              <div class="flex gap-0.5" style="color: {colors.green.light};">
                                {#each Array(evidence.credibility_score) as _}
                                  <span class="text-xs">★</span>
                                {/each}
                              </div>
                            {/if}
                          </a>
                        {/each}
                      </div>
                    </div>
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        {:else}
          <div class="bg-gray-50 border border-gray-200 rounded-lg p-10 text-center">
            <p class="text-gray-600">No documented incidents</p>
            <p class="text-sm text-gray-500 mt-1">No offense records found for this artist</p>
          </div>
        {/if}
      {/if}
    </main>
  {/if}
</div>
