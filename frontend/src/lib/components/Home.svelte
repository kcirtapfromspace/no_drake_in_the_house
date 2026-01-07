<script lang="ts">
  import { onMount } from 'svelte';
  import { apiClient } from '../utils/api-client';

  // Types
  interface NewsItem {
    id: string;
    artist_name: string;
    category: string;
    headline: string;
    source: string;
    source_url: string;
    timestamp: string;
  }

  interface CategoryList {
    id: string;
    name: string;
    description: string;
    artist_count: number;
    subscribed: boolean;
    color: string;
  }

  interface SearchResult {
    id: string;
    name: string;
    image_url?: string;
    genres?: string[];
    blocked: boolean;
  }

  interface ConnectedService {
    provider: string;
    connected: boolean;
    username?: string;
  }

  // State
  let searchQuery = '';
  let searchResults: SearchResult[] = [];
  let isSearching = false;
  let searchTimeout: ReturnType<typeof setTimeout>;

  let newsFeed: NewsItem[] = [];
  let isLoadingNews = true;

  let categoryLists: CategoryList[] = [];
  let isLoadingCategories = true;

  let connectedServices: ConnectedService[] = [];
  let blockingArtistId: string | null = null;

  // Category colors - improved contrast and reduced red overuse
  const categoryColors: Record<string, string> = {
    'sexual_misconduct': 'bg-rose-600',
    'sexual_assault': 'bg-rose-700',
    'domestic_violence': 'bg-red-600',
    'child_abuse': 'bg-red-800',
    'violent_crime': 'bg-red-500',
    'drug_trafficking': 'bg-purple-600',
    'hate_speech': 'bg-orange-600',
    'racism': 'bg-orange-700',
    'homophobia': 'bg-amber-600',
    'antisemitism': 'bg-amber-700',
    'fraud': 'bg-blue-600',
    'animal_abuse': 'bg-emerald-600',
    'other': 'bg-slate-500',
  };

  const categoryLabels: Record<string, string> = {
    'sexual_misconduct': 'Sexual Misconduct',
    'violence': 'Violence',
    'domestic_abuse': 'Domestic Abuse',
    'drug_trafficking': 'Drug Trafficking',
    'hate_speech': 'Hate Speech',
    'fraud': 'Fraud',
    'child_abuse': 'Child Abuse',
    'other': 'Other',
  };

  onMount(async () => {
    await Promise.all([
      loadNewsFeed(),
      loadCategories(),
      loadConnectedServices(),
    ]);
  });

  async function loadNewsFeed() {
    isLoadingNews = true;
    try {
      // Mock data for now - will be replaced with AI-curated news API
      newsFeed = [
        {
          id: '1',
          artist_name: 'Example Artist',
          category: 'violence',
          headline: 'Artist convicted on federal charges',
          source: 'AP News',
          source_url: 'https://apnews.com',
          timestamp: new Date(Date.now() - 3600000).toISOString(),
        },
        {
          id: '2',
          artist_name: 'Another Artist',
          category: 'domestic_abuse',
          headline: 'Multiple victims come forward with allegations',
          source: 'Rolling Stone',
          source_url: 'https://rollingstone.com',
          timestamp: new Date(Date.now() - 86400000).toISOString(),
        },
        {
          id: '3',
          artist_name: 'Third Artist',
          category: 'fraud',
          headline: 'Charged with wire fraud and money laundering',
          source: 'Billboard',
          source_url: 'https://billboard.com',
          timestamp: new Date(Date.now() - 172800000).toISOString(),
        },
      ];
    } catch (e) {
      console.error('Failed to load news feed:', e);
    } finally {
      isLoadingNews = false;
    }
  }

  async function loadCategories() {
    isLoadingCategories = true;
    try {
      const result = await apiClient.get<CategoryList[]>('/api/v1/categories');
      if (result.success && result.data) {
        categoryLists = result.data.map(cat => ({
          ...cat,
          color: categoryColors[cat.id] || 'bg-gray-500'
        }));
      }
    } catch (e) {
      console.error('Failed to load categories:', e);
    } finally {
      isLoadingCategories = false;
    }
  }

  async function loadConnectedServices() {
    try {
      connectedServices = [
        { provider: 'spotify', connected: false },
        { provider: 'apple_music', connected: false },
      ];
      // Will fetch real connection status from API
    } catch (e) {
      console.error('Failed to load services:', e);
    }
  }

  function handleSearchInput() {
    clearTimeout(searchTimeout);
    if (searchQuery.length < 2) {
      searchResults = [];
      return;
    }
    searchTimeout = setTimeout(() => searchArtists(), 300);
  }

  async function searchArtists() {
    if (searchQuery.length < 2) return;
    isSearching = true;
    try {
      const result = await apiClient.get<SearchResult[]>(`/api/v1/dnp/search?q=${encodeURIComponent(searchQuery)}`);
      if (result.success && result.data) {
        searchResults = result.data;
      }
    } catch (e) {
      console.error('Search failed:', e);
    } finally {
      isSearching = false;
    }
  }

  async function blockArtist(artist: SearchResult) {
    blockingArtistId = artist.id;
    try {
      const result = await apiClient.post('/api/v1/dnp/list', {
        artist_id: artist.id,
        reason: 'User blocked',
      });
      if (result.success) {
        // Update local state
        searchResults = searchResults.map(a =>
          a.id === artist.id ? { ...a, blocked: true } : a
        );
      }
    } catch (e) {
      console.error('Failed to block artist:', e);
    } finally {
      blockingArtistId = null;
    }
  }

  async function unblockArtist(artist: SearchResult) {
    blockingArtistId = artist.id;
    try {
      const result = await apiClient.delete(`/api/v1/dnp/list/${artist.id}`);
      if (result.success) {
        searchResults = searchResults.map(a =>
          a.id === artist.id ? { ...a, blocked: false } : a
        );
      }
    } catch (e) {
      console.error('Failed to unblock artist:', e);
    } finally {
      blockingArtistId = null;
    }
  }

  async function toggleCategory(category: CategoryList) {
    const wasSubscribed = category.subscribed;
    // Optimistic update
    categoryLists = categoryLists.map(c =>
      c.id === category.id ? { ...c, subscribed: !wasSubscribed } : c
    );

    try {
      if (wasSubscribed) {
        await apiClient.delete(`/api/v1/categories/${category.id}/subscribe`);
      } else {
        await apiClient.post(`/api/v1/categories/${category.id}/subscribe`);
      }
    } catch (e) {
      // Revert on error
      categoryLists = categoryLists.map(c =>
        c.id === category.id ? { ...c, subscribed: wasSubscribed } : c
      );
      console.error('Failed to toggle category:', e);
    }
  }

  function formatTimeAgo(timestamp: string): string {
    const seconds = Math.floor((Date.now() - new Date(timestamp).getTime()) / 1000);
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
    if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
    return `${Math.floor(seconds / 86400)}d ago`;
  }

  function connectService(provider: string) {
    // Will trigger OAuth flow
    window.location.href = `/api/v1/auth/oauth/${provider}/initiate`;
  }
</script>

<div class="min-h-screen bg-gray-900 text-white">
  <!-- Header -->
  <header class="bg-gray-800 border-b border-gray-600 sticky top-0 z-50">
    <div class="max-w-6xl mx-auto px-4 py-4">
      <div class="flex items-center justify-between">
        <div class="flex items-center space-x-3">
          <span class="text-3xl">🚫</span>
          <h1 class="text-xl font-bold">No Drake</h1>
        </div>
        <button
          class="p-2 rounded-lg hover:bg-gray-700 transition-colors"
          on:click={() => window.location.href = '/settings'}
        >
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
          </svg>
        </button>
      </div>
    </div>
  </header>

  <main class="max-w-6xl mx-auto px-4 py-6 space-y-8">
    <!-- Search Section -->
    <section>
      <div class="relative">
        <input
          type="text"
          bind:value={searchQuery}
          on:input={handleSearchInput}
          placeholder="Search artist to block..."
          class="w-full bg-gray-800 border border-gray-600 rounded-xl px-5 py-4 text-lg placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-rose-500 focus:border-transparent"
        />
        {#if isSearching}
          <div class="absolute right-4 top-1/2 -translate-y-1/2">
            <div class="w-5 h-5 border-2 border-red-500 border-t-transparent rounded-full animate-spin"></div>
          </div>
        {/if}
      </div>

      <!-- Search Results -->
      {#if searchResults.length > 0}
        <div class="mt-3 bg-gray-800 rounded-xl border border-gray-600 overflow-hidden">
          {#each searchResults as artist}
            <div class="flex items-center justify-between p-4 hover:bg-gray-700 border-b border-gray-600 last:border-0">
              <div class="flex items-center space-x-4">
                {#if artist.image_url}
                  <img src={artist.image_url} alt={artist.name} class="w-12 h-12 rounded-full object-cover" />
                {:else}
                  <div class="w-12 h-12 rounded-full bg-gray-700 flex items-center justify-center">
                    <span class="text-xl">{artist.name.charAt(0)}</span>
                  </div>
                {/if}
                <div>
                  <p class="font-medium">{artist.name}</p>
                  {#if artist.genres && artist.genres.length > 0}
                    <p class="text-sm text-gray-300">{artist.genres.slice(0, 2).join(', ')}</p>
                  {/if}
                </div>
              </div>
              <button
                on:click={() => artist.blocked ? unblockArtist(artist) : blockArtist(artist)}
                disabled={blockingArtistId === artist.id}
                class="px-4 py-2 rounded-lg font-medium transition-all {
                  artist.blocked
                    ? 'bg-gray-600 text-gray-200 hover:bg-gray-500'
                    : 'bg-rose-600 text-white hover:bg-rose-700'
                } disabled:opacity-50"
              >
                {#if blockingArtistId === artist.id}
                  <span class="inline-block w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></span>
                {:else if artist.blocked}
                  Blocked
                {:else}
                  Block
                {/if}
              </button>
            </div>
          {/each}
        </div>
      {/if}
    </section>

    <!-- Two Column Layout -->
    <div class="grid lg:grid-cols-3 gap-6">
      <!-- News Feed -->
      <section class="lg:col-span-2">
        <h2 class="text-lg font-semibold mb-4 flex items-center">
          <span class="w-2 h-2 bg-red-500 rounded-full mr-2 animate-pulse"></span>
          Recent Additions
        </h2>

        {#if isLoadingNews}
          <div class="space-y-3">
            {#each [1, 2, 3] as _}
              <div class="bg-gray-800 rounded-xl p-4 animate-pulse">
                <div class="h-4 bg-gray-700 rounded w-3/4 mb-2"></div>
                <div class="h-3 bg-gray-700 rounded w-1/2"></div>
              </div>
            {/each}
          </div>
        {:else}
          <div class="space-y-3">
            {#each newsFeed as item}
              <div class="bg-gray-800 rounded-xl p-4 border border-gray-600 hover:border-gray-500 transition-colors">
                <div class="flex items-start justify-between">
                  <div class="flex-1">
                    <div class="flex items-center space-x-2 mb-1">
                      <span class="px-2 py-0.5 rounded text-xs font-medium {categoryColors[item.category] || 'bg-gray-500'}">
                        {categoryLabels[item.category] || item.category}
                      </span>
                      <span class="text-xs text-gray-400">{formatTimeAgo(item.timestamp)}</span>
                    </div>
                    <p class="font-medium">{item.artist_name}</p>
                    <p class="text-sm text-gray-300 mt-1">{item.headline}</p>
                    <a href={item.source_url} target="_blank" rel="noopener" class="text-xs text-blue-400 hover:underline mt-2 inline-block">
                      {item.source}
                    </a>
                  </div>
                  <button class="ml-4 px-3 py-1.5 bg-rose-600 hover:bg-rose-700 rounded-lg text-sm font-medium transition-colors">
                    Block
                  </button>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </section>

      <!-- Sidebar -->
      <aside class="space-y-6">
        <!-- Category Blocklists -->
        <section>
          <h2 class="text-lg font-semibold mb-4">Blocklists</h2>

          {#if isLoadingCategories}
            <div class="space-y-2">
              {#each [1, 2, 3, 4] as _}
                <div class="bg-gray-800 rounded-lg p-3 animate-pulse">
                  <div class="h-4 bg-gray-700 rounded w-2/3"></div>
                </div>
              {/each}
            </div>
          {:else}
            <div class="space-y-2">
              {#each categoryLists as category}
                <button
                  on:click={() => toggleCategory(category)}
                  class="w-full bg-gray-800 rounded-lg p-3 border border-gray-600 hover:border-gray-500 transition-all text-left group"
                >
                  <div class="flex items-center justify-between">
                    <div class="flex items-center space-x-3">
                      <div class="w-3 h-3 rounded-full {category.color}"></div>
                      <div>
                        <p class="font-medium text-sm">{category.name}</p>
                        <p class="text-xs text-gray-400">{category.artist_count} artists</p>
                      </div>
                    </div>
                    <div class="w-5 h-5 rounded border-2 flex items-center justify-center transition-colors {
                      category.subscribed
                        ? 'bg-rose-500 border-rose-500'
                        : 'border-gray-500 group-hover:border-gray-400'
                    }">
                      {#if category.subscribed}
                        <svg class="w-3 h-3 text-white" fill="currentColor" viewBox="0 0 20 20">
                          <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                        </svg>
                      {/if}
                    </div>
                  </div>
                </button>
              {/each}
            </div>
          {/if}
        </section>

        <!-- Connected Services -->
        <section>
          <h2 class="text-lg font-semibold mb-4">Music Services</h2>
          <div class="space-y-2">
            {#each connectedServices as service}
              <div class="bg-gray-800 rounded-lg p-3 border border-gray-600">
                <div class="flex items-center justify-between">
                  <div class="flex items-center space-x-3">
                    {#if service.provider === 'spotify'}
                      <div class="w-8 h-8 rounded-full bg-green-500 flex items-center justify-center">
                        <svg class="w-5 h-5 text-white" viewBox="0 0 24 24" fill="currentColor">
                          <path d="M12 0C5.4 0 0 5.4 0 12s5.4 12 12 12 12-5.4 12-12S18.66 0 12 0zm5.521 17.34c-.24.359-.66.48-1.021.24-2.82-1.74-6.36-2.101-10.561-1.141-.418.122-.779-.179-.899-.539-.12-.421.18-.78.54-.9 4.56-1.021 8.52-.6 11.64 1.32.42.18.479.659.301 1.02zm1.44-3.3c-.301.42-.841.6-1.262.3-3.239-1.98-8.159-2.58-11.939-1.38-.479.12-1.02-.12-1.14-.6-.12-.48.12-1.021.6-1.141C9.6 9.9 15 10.561 18.72 12.84c.361.181.54.78.241 1.2zm.12-3.36C15.24 8.4 8.82 8.16 5.16 9.301c-.6.179-1.2-.181-1.38-.721-.18-.601.18-1.2.72-1.381 4.26-1.26 11.28-1.02 15.721 1.621.539.3.719 1.02.419 1.56-.299.421-1.02.599-1.559.3z"/>
                        </svg>
                      </div>
                    {:else if service.provider === 'apple_music'}
                      <div class="w-8 h-8 rounded-full bg-gradient-to-br from-red-500 to-pink-500 flex items-center justify-center">
                        <svg class="w-5 h-5 text-white" viewBox="0 0 24 24" fill="currentColor">
                          <path d="M23.994 6.124a9.23 9.23 0 00-.24-2.19c-.317-1.31-1.062-2.31-2.18-3.043a5.022 5.022 0 00-1.877-.726 10.496 10.496 0 00-1.564-.15c-.04-.003-.083-.01-.124-.013H5.986c-.152.01-.303.017-.455.026-.747.043-1.49.123-2.193.4-1.336.53-2.3 1.452-2.865 2.78-.192.448-.292.925-.363 1.408-.056.392-.088.785-.1 1.18 0 .032-.007.062-.01.093v12.223c.01.14.017.283.027.424.05.815.154 1.624.497 2.373.65 1.42 1.738 2.353 3.234 2.801.42.127.856.187 1.293.228.555.053 1.11.06 1.667.06h11.03a12.5 12.5 0 001.57-.1c.822-.106 1.596-.35 2.295-.81a5.046 5.046 0 001.88-2.207c.186-.42.293-.87.37-1.324.113-.675.138-1.358.137-2.04-.002-3.8 0-7.595-.003-11.393zm-6.423 3.99v5.712c0 .417-.058.827-.244 1.206-.29.59-.76.962-1.388 1.14-.35.1-.706.157-1.07.173-.95.042-1.785-.476-2.144-1.32-.238-.56-.223-1.136-.017-1.7.303-.825.96-1.277 1.743-1.49.294-.08.595-.13.893-.18.323-.054.65-.1.973-.157.274-.048.47-.202.53-.486a.707.707 0 00.017-.146c.002-1.633.002-3.265.002-4.898v-.07l-.06-.01c-2.097.4-4.194.8-6.29 1.202-.014.002-.032.014-.037.026-.006.016-.003.037-.003.056v7.36c0 .418-.052.832-.227 1.218-.282.622-.76 1.02-1.416 1.207-.313.09-.634.138-.96.166-.906.08-1.732-.4-2.134-1.203-.268-.534-.278-1.1-.096-1.66.267-.817.864-1.304 1.64-1.55.376-.12.763-.185 1.148-.25.278-.047.558-.088.832-.145.317-.065.522-.25.58-.574a.504.504 0 00.007-.115v-8.41c0-.25.042-.493.15-.72.183-.385.486-.62.882-.728.17-.047.346-.073.522-.11 2.55-.526 5.1-1.05 7.65-1.573.093-.02.19-.03.285-.03.316.004.528.2.613.5.032.113.044.233.044.35v5.9z"/>
                        </svg>
                      </div>
                    {/if}
                    <span class="font-medium text-sm capitalize">{service.provider.replace('_', ' ')}</span>
                  </div>
                  {#if service.connected}
                    <span class="text-xs text-green-400">Connected</span>
                  {:else}
                    <button
                      on:click={() => connectService(service.provider)}
                      class="text-xs text-blue-400 hover:text-blue-300"
                    >
                      Connect
                    </button>
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        </section>

        <!-- Stats -->
        <section class="bg-gray-800 rounded-xl p-4 border border-gray-600">
          <h3 class="text-sm font-medium text-gray-300 mb-3">Your Blocks</h3>
          <div class="text-3xl font-bold">
            {categoryLists.filter(c => c.subscribed).reduce((sum, c) => sum + c.artist_count, 0)}
          </div>
          <p class="text-sm text-gray-400 mt-1">artists blocked</p>
        </section>
      </aside>
    </div>
  </main>
</div>

<!-- Removed custom bg-gray-750, using standard Tailwind gray-700 for better hover states -->
