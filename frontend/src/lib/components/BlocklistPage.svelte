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

<div class="min-h-screen bg-gray-50">
  <!-- Search bar (sticky) -->
  <ArtistSearchBar />
  
  <!-- Main content -->
  <div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
    <!-- Header -->
    <div class="mb-6">
      <h1 class="text-2xl font-bold text-gray-900 mb-2">Your Blocklist</h1>
      <p class="text-gray-600">
        {blockedArtists.length} artist{blockedArtists.length !== 1 ? 's' : ''} blocked
      </p>
    </div>
    
    <!-- Loading state -->
    {#if $dnpStore.isLoading}
      <div class="space-y-4">
        {#each [1, 2, 3, 4] as _}
          <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-4 animate-pulse">
            <div class="flex items-start space-x-4">
              <div class="w-16 h-16 bg-gray-300 rounded-lg"></div>
              <div class="flex-1">
                <div class="h-4 bg-gray-300 rounded w-32 mb-2"></div>
                <div class="h-3 bg-gray-300 rounded w-24 mb-2"></div>
                <div class="h-3 bg-gray-300 rounded w-16"></div>
              </div>
              <div class="w-16 h-8 bg-gray-300 rounded"></div>
            </div>
          </div>
        {/each}
      </div>
    {:else if $dnpStore.error}
      <!-- Error state -->
      <div class="bg-white rounded-lg shadow-sm border border-red-200 p-6">
        <div class="flex items-center">
          <svg class="w-6 h-6 text-red-600 mr-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <div>
            <h3 class="text-lg font-medium text-red-900">Error Loading Blocklist</h3>
            <p class="text-red-700 mt-1">{$dnpStore.error}</p>
            <button
              on:click={() => dnpActions.fetchDnpList()}
              class="mt-2 text-sm text-red-800 underline hover:text-red-900"
            >
              Try again
            </button>
          </div>
        </div>
      </div>
    {:else if blockedArtists.length === 0}
      <!-- Empty state -->
      <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-12 text-center">
        <svg class="mx-auto h-16 w-16 text-gray-400 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728L5.636 5.636m12.728 12.728L5.636 5.636" />
        </svg>
        <h3 class="text-xl font-medium text-gray-900 mb-2">No Artists Blocked Yet</h3>
        <p class="text-gray-600 mb-6 max-w-md mx-auto">
          Start building your blocklist by searching for artists you want to avoid in the search bar above.
        </p>
        <div class="flex items-center justify-center space-x-2 text-sm text-gray-500">
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7h8m0 0v8m0-8l-8 8-4-4-6 6" />
          </svg>
          <span>Search above to get started</span>
        </div>
      </div>
    {:else}
      <!-- Blocked artists feed -->
      <div class="space-y-4">
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
      
      <!-- Load more placeholder (for future pagination) -->
      {#if blockedArtists.length >= 20}
        <div class="mt-8 text-center">
          <button class="px-6 py-3 border border-gray-300 text-gray-700 rounded-md hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500">
            Load More Artists
          </button>
        </div>
      {/if}
    {/if}
  </div>
</div>