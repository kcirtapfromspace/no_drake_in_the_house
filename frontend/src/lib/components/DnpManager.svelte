<script lang="ts">
  import { onMount } from 'svelte';
  import { dnpActions, dnpStore, dnpTags } from '../stores/dnp';
  import ArtistSearch from './ArtistSearch.svelte';
  import DnpEntry from './DnpEntry.svelte';
  import BulkActions from './BulkActions.svelte';
  
  let searchQuery = '';
  let selectedTag = '';
  let showAddForm = false;
  let selectedEntries = new Set();

  $: filteredEntries = $dnpStore.entries.filter(entry => {
    const matchesSearch = !searchQuery || 
      entry.artist.canonical_name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      entry.tags.some(tag => tag.toLowerCase().includes(searchQuery.toLowerCase())) ||
      (entry.note && entry.note.toLowerCase().includes(searchQuery.toLowerCase()));
    
    const matchesTag = !selectedTag || entry.tags.includes(selectedTag);
    
    return matchesSearch && matchesTag;
  });

  function toggleSelectAll() {
    if (selectedEntries.size === filteredEntries.length) {
      selectedEntries.clear();
    } else {
      selectedEntries = new Set(filteredEntries.map(entry => entry.artist.id));
    }
    selectedEntries = selectedEntries; // Trigger reactivity
  }

  function toggleSelectEntry(artistId: string) {
    if (selectedEntries.has(artistId)) {
      selectedEntries.delete(artistId);
    } else {
      selectedEntries.add(artistId);
    }
    selectedEntries = selectedEntries; // Trigger reactivity
  }

  function clearSelection() {
    selectedEntries.clear();
    selectedEntries = selectedEntries; // Trigger reactivity
  }

  async function handleBulkDelete() {
    if (selectedEntries.size === 0) return;
    
    if (confirm(`Are you sure you want to remove ${selectedEntries.size} artist(s) from your DNP list?`)) {
      const promises = Array.from(selectedEntries).map(artistId => 
        dnpActions.removeArtist(artistId)
      );
      
      await Promise.all(promises);
      clearSelection();
    }
  }

  function handleArtistAdded() {
    showAddForm = false;
  }
</script>

<div class="px-4 py-6 sm:px-0">
  <div class="mb-6">
    <div class="flex justify-between items-center">
      <div>
        <h2 class="text-2xl font-bold text-gray-900">Do-Not-Play List</h2>
        <p class="mt-1 text-sm text-gray-600">
          Manage artists you want to avoid across your streaming services.
        </p>
      </div>
      <button
        on:click={() => showAddForm = !showAddForm}
        class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
      >
        <svg class="-ml-1 mr-2 h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
        </svg>
        Add Artist
      </button>
    </div>
  </div>

  <!-- Add Artist Form -->
  {#if showAddForm}
    <div class="mb-6 bg-white shadow rounded-lg p-6">
      <h3 class="text-lg font-medium text-gray-900 mb-4">Add Artist to DNP List</h3>
      <ArtistSearch on:artistAdded={handleArtistAdded} />
    </div>
  {/if}

  <!-- Filters and Search -->
  <div class="mb-6 bg-white shadow rounded-lg p-4">
    <div class="flex flex-col sm:flex-row gap-4">
      <div class="flex-1">
        <label for="search" class="sr-only">Search artists</label>
        <div class="relative">
          <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
            <svg class="h-5 w-5 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
          </div>
          <input
            id="search"
            bind:value={searchQuery}
            type="text"
            placeholder="Search artists, tags, or notes..."
            class="block w-full pl-10 pr-3 py-2 border border-gray-300 rounded-md leading-5 bg-white placeholder-gray-500 focus:outline-none focus:placeholder-gray-400 focus:ring-1 focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
          />
        </div>
      </div>
      
      <div class="sm:w-48">
        <label for="tag-filter" class="sr-only">Filter by tag</label>
        <select
          id="tag-filter"
          bind:value={selectedTag}
          class="block w-full pl-3 pr-10 py-2 text-base border border-gray-300 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm rounded-md"
        >
          <option value="">All tags</option>
          {#each $dnpTags as tag}
            <option value={tag}>{tag}</option>
          {/each}
        </select>
      </div>
    </div>
  </div>

  <!-- Bulk Actions -->
  {#if selectedEntries.size > 0}
    <div class="mb-4">
      <BulkActions 
        selectedCount={selectedEntries.size}
        on:bulkDelete={handleBulkDelete}
        on:clearSelection={clearSelection}
      />
    </div>
  {/if}

  <!-- DNP List -->
  <div class="bg-white shadow overflow-hidden sm:rounded-md">
    {#if $dnpStore.isLoading}
      <div class="p-6 text-center">
        <svg class="animate-spin mx-auto h-8 w-8 text-gray-400" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
        <p class="mt-2 text-sm text-gray-500">Loading DNP list...</p>
      </div>
    {:else if $dnpStore.error}
      <div class="p-6 text-center">
        <svg class="mx-auto h-8 w-8 text-red-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        <p class="mt-2 text-sm text-red-600">{$dnpStore.error}</p>
        <button
          on:click={() => dnpActions.fetchDnpList()}
          class="mt-2 text-sm text-indigo-600 hover:text-indigo-500"
        >
          Try again
        </button>
      </div>
    {:else if filteredEntries.length === 0}
      <div class="p-6 text-center">
        {#if $dnpStore.entries.length === 0}
          <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3" />
          </svg>
          <h3 class="mt-2 text-sm font-medium text-gray-900">No artists in your DNP list</h3>
          <p class="mt-1 text-sm text-gray-500">Get started by adding artists you want to avoid.</p>
          <div class="mt-6">
            <button
              on:click={() => showAddForm = true}
              class="inline-flex items-center px-4 py-2 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
            >
              <svg class="-ml-1 mr-2 h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
              </svg>
              Add your first artist
            </button>
          </div>
        {:else}
          <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
          <h3 class="mt-2 text-sm font-medium text-gray-900">No artists match your search</h3>
          <p class="mt-1 text-sm text-gray-500">Try adjusting your search terms or filters.</p>
        {/if}
      </div>
    {:else}
      <div class="px-4 py-3 bg-gray-50 border-b border-gray-200 sm:px-6">
        <div class="flex items-center justify-between">
          <div class="flex items-center">
            <input
              id="select-all"
              type="checkbox"
              checked={selectedEntries.size === filteredEntries.length && filteredEntries.length > 0}
              on:change={toggleSelectAll}
              class="h-4 w-4 text-indigo-600 focus:ring-indigo-500 border-gray-300 rounded"
            />
            <label for="select-all" class="ml-3 text-sm text-gray-900">
              {filteredEntries.length} artist{filteredEntries.length !== 1 ? 's' : ''}
              {#if selectedEntries.size > 0}
                ({selectedEntries.size} selected)
              {/if}
            </label>
          </div>
          
          {#if searchQuery || selectedTag}
            <button
              on:click={() => { searchQuery = ''; selectedTag = ''; }}
              class="text-sm text-indigo-600 hover:text-indigo-500"
            >
              Clear filters
            </button>
          {/if}
        </div>
      </div>
      
      <ul class="divide-y divide-gray-200">
        {#each filteredEntries as entry (entry.artist.id)}
          <DnpEntry 
            {entry}
            selected={selectedEntries.has(entry.artist.id)}
            on:toggleSelect={() => toggleSelectEntry(entry.artist.id)}
          />
        {/each}
      </ul>
    {/if}
  </div>

  <!-- Stats -->
  {#if $dnpStore.entries.length > 0}
    <div class="mt-6 bg-gray-50 rounded-lg p-4">
      <div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
        <div class="text-center">
          <div class="text-2xl font-bold text-gray-900">{$dnpStore.entries.length}</div>
          <div class="text-sm text-gray-500">Total Artists</div>
        </div>
        <div class="text-center">
          <div class="text-2xl font-bold text-gray-900">{$dnpTags.length}</div>
          <div class="text-sm text-gray-500">Unique Tags</div>
        </div>
        <div class="text-center">
          <div class="text-2xl font-bold text-gray-900">
            {$dnpStore.entries.filter(e => e.note).length}
          </div>
          <div class="text-sm text-gray-500">With Notes</div>
        </div>
      </div>
    </div>
  {/if}
</div>