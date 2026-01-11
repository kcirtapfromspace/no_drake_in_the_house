<script lang="ts">
  import { dnpActions, dnpStore } from '../stores/dnp';
  
  let searchQuery = '';
  let searchTimeout: any;
  let isAddingArtist = false;
  
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
        // Clear search after successful add
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

<div class="sticky top-0 bg-zinc-800 border-b border-zinc-600 p-4 z-10">
  <!-- Search input -->
  <div class="relative">
    <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
      <svg class="h-5 w-5 text-zinc-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
      </svg>
    </div>

    <input
      bind:value={searchQuery}
      type="text"
      placeholder="Search for artists to block..."
      class="block w-full pl-10 pr-10 py-3 border border-zinc-600 rounded-lg leading-5 bg-zinc-800 text-zinc-100 placeholder-zinc-500 focus:outline-none focus:placeholder-zinc-400 focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 text-lg"
    />

    {#if searchQuery}
      <button
        on:click={clearSearch}
        class="absolute inset-y-0 right-0 pr-3 flex items-center"
      >
        <svg class="h-5 w-5 text-zinc-400 hover:text-zinc-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    {/if}
  </div>
  
  <!-- Search results -->
  {#if searchQuery && ($dnpStore.searchResults.length > 0 || $dnpStore.isSearching)}
    <div class="mt-4 bg-zinc-800 border border-zinc-600 rounded-lg shadow-lg max-h-96 overflow-y-auto">
      {#if $dnpStore.isSearching}
        <!-- Loading state -->
        <div class="p-4">
          <div class="flex items-center justify-center">
            <svg class="animate-spin h-6 w-6 text-zinc-400" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            <span class="ml-2 text-zinc-400">Searching...</span>
          </div>
        </div>
      {:else if $dnpStore.searchResults.length > 0}
        <!-- Search results -->
        {#each $dnpStore.searchResults as artist}
          <div class="flex items-center justify-between p-4 border-b border-zinc-700 last:border-b-0 hover:bg-zinc-700">
            <div class="flex items-center space-x-3">
              {#if artist.metadata?.image}
                <img
                  src={artist.metadata.image}
                  alt={artist.canonical_name}
                  class="w-12 h-12 rounded-lg object-cover"
                />
              {:else}
                <div class="w-12 h-12 rounded-lg bg-zinc-700 flex items-center justify-center">
                  <svg class="w-6 h-6 text-zinc-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3" />
                  </svg>
                </div>
              {/if}

              <div>
                <h4 class="font-medium text-zinc-100">{artist.canonical_name}</h4>
                {#if artist.metadata?.genres && artist.metadata.genres.length > 0}
                  <p class="text-sm text-zinc-400">
                    {artist.metadata.genres.slice(0, 2).join(', ')}
                  </p>
                {/if}
              </div>
            </div>
            
            <button
              on:click={() => handleBlockArtist(artist)}
              disabled={isAddingArtist}
              class="px-4 py-2 bg-red-600 text-white text-sm font-medium rounded-md hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500 transition-colors duration-200 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {#if isAddingArtist}
                <svg class="animate-spin w-4 h-4" fill="none" viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
              {:else}
                Block
              {/if}
            </button>
          </div>
        {/each}
      {:else}
        <!-- No results -->
        <div class="p-4 text-center text-zinc-400">
          <svg class="mx-auto h-12 w-12 text-zinc-500 mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
          <p>No artists found for "{searchQuery}"</p>
          <p class="text-sm mt-1">Try a different search term</p>
        </div>
      {/if}
    </div>
  {/if}
</div>