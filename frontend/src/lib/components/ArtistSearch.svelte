<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { dnpActions, dnpStore } from '../stores/dnp';
  
  const dispatch = createEventDispatcher();
  
  let searchQuery = '';
  let selectedArtist: any = null;
  let tags = '';
  let note = '';
  let isAdding = false;
  let error = '';
  
  let searchTimeout: any;
  
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

  function selectArtist(artist: any) {
    selectedArtist = artist;
    searchQuery = artist.canonical_name;
    dnpActions.clearSearch();
  }

  function clearSelection() {
    selectedArtist = null;
    searchQuery = '';
    dnpActions.clearSearch();
  }

  async function handleSubmit() {
    if (!searchQuery.trim()) {
      error = 'Please enter an artist name';
      return;
    }

    isAdding = true;
    error = '';

    const tagArray = tags.split(',').map(t => t.trim()).filter(t => t);
    
    const result = await dnpActions.addArtist(
      searchQuery,
      tagArray,
      note.trim() || undefined
    );

    if (result.success) {
      // Reset form
      searchQuery = '';
      selectedArtist = null;
      tags = '';
      note = '';
      dispatch('artistAdded');
    } else {
      error = result.message || 'Failed to add artist';
    }

    isAdding = false;
  }

  function getProviderBadges(artist: any) {
    const badges = [];
    if (artist.external_ids.spotify) badges.push({ name: 'Spotify', color: 'bg-green-900 text-green-300' });
    if (artist.external_ids.apple) badges.push({ name: 'Apple', color: 'bg-zinc-700 text-zinc-300' });
    if (artist.external_ids.musicbrainz) badges.push({ name: 'MusicBrainz', color: 'bg-blue-900 text-blue-300' });
    return badges;
  }
</script>

<form on:submit|preventDefault={handleSubmit} class="space-y-4">
  <!-- Artist Search -->
  <div class="relative">
    <label for="artist-search" class="block text-sm font-medium text-zinc-300">
      Artist Name
    </label>
    <div class="mt-1 relative">
      <input
        id="artist-search"
        type="text"
        bind:value={searchQuery}
        placeholder="Search for an artist..."
        class="block w-full border border-zinc-600 rounded-lg px-3 py-2 bg-zinc-800 text-zinc-100 placeholder-zinc-500 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
      />
      
      {#if selectedArtist}
        <button
          type="button"
          on:click={clearSelection}
          class="absolute inset-y-0 right-0 pr-3 flex items-center"
        >
          <svg aria-hidden="true" class="h-5 w-5 text-zinc-400 hover:text-zinc-300" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      {/if}
    </div>

    <!-- Search Results -->
    {#if $dnpStore.searchResults.length > 0 && !selectedArtist}
      <div class="absolute z-10 mt-1 w-full bg-zinc-800 shadow-lg max-h-60 rounded-lg py-1 text-base ring-1 ring-zinc-600 overflow-auto focus:outline-none sm:text-sm">
        {#each $dnpStore.searchResults as artist}
          <button
            type="button"
            on:click={() => selectArtist(artist)}
            class="w-full text-left px-4 py-2 hover:bg-zinc-700 focus:bg-zinc-700 focus:outline-none"
          >
            <div class="flex items-center justify-between">
              <div class="flex items-center space-x-3">
                {#if artist.metadata.image}
                  <img
                    src={artist.metadata.image}
                    alt={artist.canonical_name}
                    class="icon icon-xl  rounded-full object-cover"
                  />
                {:else}
                  <div class="h-8 w-8 rounded-full bg-zinc-700 flex items-center justify-center">
                    <svg aria-hidden="true" class="h-4 w-4 text-zinc-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                    </svg>
                  </div>
                {/if}
                <div>
                  <div class="text-sm font-medium text-zinc-200">
                    {artist.canonical_name}
                  </div>
                  {#if artist.metadata.genres && artist.metadata.genres.length > 0}
                    <div class="text-xs text-zinc-400">
                      {artist.metadata.genres.slice(0, 2).join(', ')}
                    </div>
                  {/if}
                </div>
              </div>
              
              <div class="flex space-x-1">
                {#each getProviderBadges(artist) as badge}
                  <span class="flex items-center px-2 py-0.5 rounded text-xs font-medium {badge.color}">
                    {badge.name}
                  </span>
                {/each}
              </div>
            </div>
          </button>
        {/each}
      </div>
    {/if}

    {#if $dnpStore.isSearching}
      <div class="absolute z-10 mt-1 w-full bg-zinc-800 shadow-lg rounded-lg py-2 text-center">
        <svg aria-hidden="true" class="animate-spin mx-auto h-5 w-5 text-zinc-400" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
        <p class="text-sm text-zinc-400 mt-1">Searching...</p>
      </div>
    {/if}
  </div>

  <!-- Selected Artist Preview -->
  {#if selectedArtist}
    <div class="bg-zinc-800 rounded-lg p-4">
      <h4 class="text-sm font-medium text-zinc-300 mb-2">Selected Artist</h4>
      <div class="flex items-center space-x-3">
        {#if selectedArtist.metadata.image}
          <img
            src={selectedArtist.metadata.image}
            alt={selectedArtist.canonical_name}
            class="avatar avatar--xl object-cover"
          />
        {:else}
          <div class="avatar avatar--xl bg-zinc-700 avatar__placeholder">
            <svg aria-hidden="true" class="h-8 w-8 text-zinc-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
            </svg>
          </div>
        {/if}
        <div class="flex-1">
          <div class="text-sm font-medium text-zinc-200">
            {selectedArtist.canonical_name}
          </div>
          {#if selectedArtist.metadata.genres && selectedArtist.metadata.genres.length > 0}
            <div class="text-xs text-zinc-400">
              {selectedArtist.metadata.genres.join(', ')}
            </div>
          {/if}
          <div class="flex space-x-1 mt-1">
            {#each getProviderBadges(selectedArtist) as badge}
              <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium {badge.color}">
                {badge.name}
              </span>
            {/each}
          </div>
        </div>
      </div>
    </div>
  {/if}

  <!-- Tags -->
  <div>
    <label for="tags" class="block text-sm font-medium text-zinc-300">
      Tags (optional)
    </label>
    <input
      id="tags"
      type="text"
      bind:value={tags}
      placeholder="e.g., controversial, personal, explicit (comma-separated)"
      class="mt-1 block w-full border border-zinc-600 rounded-lg px-3 py-2 bg-zinc-800 text-zinc-100 placeholder-zinc-500 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
    />
    <p class="mt-1 text-xs text-zinc-400">
      Use tags to organize your DNP list. Separate multiple tags with commas.
    </p>
  </div>

  <!-- Note -->
  <div>
    <label for="note" class="block text-sm font-medium text-zinc-300">
      Note (optional)
    </label>
    <textarea
      id="note"
      bind:value={note}
      rows="2"
      placeholder="Add a personal note about why you're blocking this artist..."
      class="mt-1 block w-full border border-zinc-600 rounded-lg px-3 py-2 bg-zinc-800 text-zinc-100 placeholder-zinc-500 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
    ></textarea>
  </div>

  {#if error}
    <div class="text-red-400 text-sm">
      {error}
    </div>
  {/if}

  <!-- Submit Button -->
  <div class="flex justify-end space-x-3">
    <button
      type="button"
      on:click={() => dispatch('artistAdded')}
      class="px-4 py-2 border border-zinc-600 rounded-lg text-sm font-medium text-zinc-300 bg-zinc-800 hover:bg-zinc-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
    >
      Cancel
    </button>
    <button
      type="submit"
      disabled={isAdding || !searchQuery.trim()}
      class="px-4 py-2 border border-transparent rounded-lg shadow-sm text-sm font-medium text-white bg-primary hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"
    >
      {#if isAdding}
        <svg aria-hidden="true" class="animate-spin -ml-1 mr-2 h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
        Adding...
      {:else}
        Add to DNP List
      {/if}
    </button>
  </div>
</form>