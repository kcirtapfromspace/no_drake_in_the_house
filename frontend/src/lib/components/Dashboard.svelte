<script lang="ts">
  import { onMount } from 'svelte';
  import { dnpActions, dnpStore, dnpCount } from '../stores/dnp';
  import { connectionActions, hasActiveSpotifyConnection } from '../stores/connections';
  import { navigateTo } from '../utils/simple-router';
  import { currentUser } from '../stores/auth';
  import { Breadcrumb } from './ui';

  let searchQuery = '';
  let searchResults: any[] = [];
  let isSearching = false;
  let showBlockModal = false;
  let selectedArtist: any = null;
  let blockReason = '';
  let blockEvidence = '';
  let isBlocking = false;

  // Common reasons for blocking
  const commonReasons = [
    { label: 'Domestic violence', icon: '\u2716' },
    { label: 'Sexual misconduct', icon: '\u26A0' },
    { label: 'Hate speech', icon: '\u2691' },
    { label: 'Criminal behavior', icon: '\u2696' },
    { label: 'Harmful to children', icon: '\u26D4' },
    { label: 'Other', icon: '\u2710' },
  ];

  onMount(async () => {
    await dnpActions.fetchDnpList();
    await connectionActions.fetchConnections();
  });

  async function handleSearch() {
    if (!searchQuery.trim()) {
      searchResults = [];
      return;
    }

    isSearching = true;
    try {
      const result = await dnpActions.searchArtists(searchQuery);
      searchResults = result.artists || [];
    } catch (e) {
      console.error('Search failed:', e);
      searchResults = [];
    }
    isSearching = false;
  }

  function openBlockModal(artist: any) {
    selectedArtist = artist;
    blockReason = '';
    blockEvidence = '';
    showBlockModal = true;
  }

  function closeBlockModal() {
    showBlockModal = false;
    selectedArtist = null;
    blockReason = '';
    blockEvidence = '';
  }

  function selectReason(reason: string) {
    blockReason = reason;
  }

  async function confirmBlock() {
    if (!selectedArtist || !blockReason) return;

    isBlocking = true;
    try {
      const note = blockEvidence
        ? `${blockReason}\n\nEvidence: ${blockEvidence}`
        : blockReason;

      await dnpActions.addArtist(selectedArtist.id, [blockReason.toLowerCase().replace(/\s+/g, '-')], note);
      closeBlockModal();
      searchQuery = '';
      searchResults = [];
    } catch (e) {
      console.error('Failed to block artist:', e);
    }
    isBlocking = false;
  }

  // Get first name for greeting
  $: firstName = $currentUser?.email?.split('@')[0] || 'there';
  $: blockedArtists = $dnpStore.entries || [];
</script>

<div class="min-h-screen bg-gradient-page">
  <!-- Warm welcome header -->
  <div class="bg-zinc-800 border-b-2 border-zinc-700">
    <div class="max-w-4xl mx-auto px-4 py-8 sm:px-6 lg:px-8">
      <Breadcrumb />
      <h1 class="text-3xl font-bold text-white mb-2">
        Hey {firstName}! 👋
      </h1>
      <p class="text-lg text-zinc-400">
        Keep your family's music safe and aligned with your values.
      </p>
    </div>
  </div>

  <div class="max-w-4xl mx-auto px-4 py-6 sm:px-6 lg:px-8">
    <!-- Main search card - the primary action -->
    <div class="rounded-2xl shadow-lg p-6 mb-8 surface-card" >
      <div class="text-center mb-6">
        <h2 class="text-xl font-semibold text-white mb-2">
          Block an Artist
        </h2>
        <p class="text-zinc-400">
          Saw something in the news? Search for the artist and add them to your blocklist.
        </p>
      </div>

      <!-- Search input -->
      <div class="relative max-w-xl mx-auto">
        <input type="text" bind:value={searchQuery} on:input={handleSearch} placeholder="Search for an artist (e.g., Chris Brown, R. Kelly...)" class="w-full px-5 py-4 text-lg rounded-lg text-zinc-300 focus:border-indigo-500 focus:ring-2 focus:ring-indigo-200 transition-all surface-panel" />
        {#if isSearching}
          <div class="absolute right-4 top-1/2 -translate-y-1/2">
            <div class="w-5 h-5 border-2 border-indigo-500 border-t-transparent rounded-full animate-spin"></div>
          </div>
        {/if}
      </div>

      <!-- Search results -->
      {#if searchResults.length > 0}
        <div class="mt-4 max-w-xl mx-auto space-y-2">
          {#each searchResults as artist}
            <button
              type="button"
              on:click={() => openBlockModal(artist)}
              class="w-full flex items-center p-4 hover:bg-indigo-900 rounded-xl transition-colors text-left group bg-zinc-700"
             
            >
              <div class="w-12 h-12 rounded-full flex items-center justify-center text-xl mr-4 bg-zinc-600">
                🎤
              </div>
              <div class="flex-1 min-w-0">
                <div class="font-medium text-white group-hover:text-indigo-400 truncate">
                  {artist.canonical_name}
                </div>
                {#if artist.genres?.length}
                  <div class="text-sm text-zinc-400">
                    {artist.genres.slice(0, 3).join(', ')}
                  </div>
                {/if}
              </div>
              <div class="text-indigo-600 opacity-0 group-hover:opacity-100 transition-opacity">
                Block →
              </div>
            </button>
          {/each}
        </div>
      {/if}

      {#if searchQuery && !isSearching && searchResults.length === 0}
        <p class="text-center text-zinc-400 mt-4">
          No artists found. Try a different name.
        </p>
      {/if}
    </div>

    <!-- Stats row -->
    <div class="grid grid-cols-2 gap-4 mb-8">
      <div class="rounded-xl p-5 shadow-sm surface-card" >
        <div class="flex items-center">
          <div class="w-12 h-12 bg-red-900/50 rounded-full flex items-center justify-center mr-4">
            <svg class="w-6 h-6 text-red-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="4.93" y1="4.93" x2="19.07" y2="19.07"/></svg>
          </div>
          <div>
            <div class="text-2xl font-bold text-white">{$dnpCount}</div>
            <div class="text-sm text-zinc-400">Artists blocked</div>
          </div>
        </div>
      </div>

      <button
        type="button"
        on:click={() => navigateTo('connections')}
        class="rounded-xl p-5 shadow-sm hover:border-indigo-200 transition-colors text-left surface-card"
       
      >
        <div class="flex items-center">
          <div class="w-12 h-12 {$hasActiveSpotifyConnection ? 'bg-green-900/50' : 'bg-yellow-900/50'} rounded-full flex items-center justify-center mr-4">
            {#if $hasActiveSpotifyConnection}
              <svg class="w-6 h-6 text-green-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 11.08V12a10 10 0 11-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
            {:else}
              <svg class="w-6 h-6 text-yellow-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10 13a5 5 0 007.54.54l3-3a5 5 0 00-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 00-7.54-.54l-3 3a5 5 0 007.07 7.07l1.71-1.71"/></svg>
            {/if}
          </div>
          <div>
            <div class="text-lg font-semibold text-white">
              {$hasActiveSpotifyConnection ? 'Connected' : 'Connect'}
            </div>
            <div class="text-sm text-zinc-400">
              {$hasActiveSpotifyConnection ? 'Spotify linked' : 'Link Spotify'}
            </div>
          </div>
        </div>
      </button>
    </div>

    <!-- Recent blocks -->
    {#if blockedArtists.length > 0}
      <div class="mb-8">
        <div class="flex items-center justify-between mb-4">
          <h3 class="text-lg font-semibold text-white">Your Blocklist</h3>
          <button
            type="button"
            on:click={() => navigateTo('blocklist')}
            class="text-indigo-600 hover:text-indigo-700 text-sm font-medium"
          >
            View all →
          </button>
        </div>

        <div class="space-y-3">
          {#each blockedArtists.slice(0, 5) as entry}
            <div class="rounded-xl p-4 shadow-sm surface-card" >
              <div class="flex items-start">
                <div class="w-10 h-10 bg-red-900/50 rounded-full flex items-center justify-center mr-3 flex-shrink-0">
                  <svg class="w-5 h-5 text-red-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="4.93" y1="4.93" x2="19.07" y2="19.07"/></svg>
                </div>
                <div class="flex-1 min-w-0">
                  <div class="font-medium text-white truncate">{entry.artist.canonical_name}</div>
                  {#if entry.note}
                    <p class="text-sm text-zinc-400 mt-1 line-clamp-2">{entry.note}</p>
                  {/if}
                  {#if entry.tags?.length}
                    <div class="flex flex-wrap gap-1 mt-2">
                      {#each entry.tags.slice(0, 3) as tag}
                        <span class="px-2 py-0.5 text-zinc-300 text-xs rounded-full bg-zinc-700" >
                          {tag}
                        </span>
                      {/each}
                    </div>
                  {/if}
                </div>
              </div>
            </div>
          {/each}
        </div>
      </div>
    {:else}
      <!-- Empty state with guidance -->
      <div class="rounded-2xl p-8 text-center mb-8 bg-gradient-card">
        <div class="mb-4"><svg class="w-12 h-12 text-zinc-500 mx-auto" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M9 18V5l12-3v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="15" r="3"/></svg></div>
        <h3 class="text-xl font-semibold text-white mb-2">
          Your blocklist is empty
        </h3>
        <p class="text-zinc-400 max-w-md mx-auto">
          Use the search above to find and block artists whose behavior doesn't align with your values.
        </p>
      </div>
    {/if}

    <!-- Community lists teaser -->
    <button
      type="button"
      on:click={() => navigateTo('community')}
      class="w-full rounded-2xl p-6 shadow-sm hover:border-indigo-200 transition-colors text-left surface-card"
     
    >
      <div class="flex items-center">
        <div class="w-14 h-14 bg-purple-900/50 rounded-xl flex items-center justify-center mr-4">
          <svg class="w-7 h-7 text-purple-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 21v-2a4 4 0 00-4-4H5a4 4 0 00-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 00-3-3.87"/><path d="M16 3.13a4 4 0 010 7.75"/></svg>
        </div>
        <div class="flex-1">
          <h3 class="font-semibold text-white mb-1">Community Lists</h3>
          <p class="text-sm text-zinc-400">
            Browse lists curated by other parents and advocates. Subscribe to stay updated.
          </p>
        </div>
        <div class="text-zinc-400 ml-4">→</div>
      </div>
    </button>
  </div>
</div>

<!-- Block Modal -->
{#if showBlockModal && selectedArtist}
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center p-4 z-50" on:click={closeBlockModal} role="dialog" aria-modal="true">
    <div class="rounded-2xl max-w-lg w-full p-6 shadow-xl bg-zinc-800" on:click|stopPropagation role="document">
      <div class="flex items-center mb-6">
        <div class="w-14 h-14 bg-red-900/50 rounded-full flex items-center justify-center mr-4">
          <svg class="w-7 h-7 text-red-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="4.93" y1="4.93" x2="19.07" y2="19.07"/></svg>
        </div>
        <div>
          <h3 class="text-xl font-bold text-white">Block {selectedArtist.canonical_name}</h3>
          <p class="text-zinc-400">Why are you blocking this artist?</p>
        </div>
      </div>

      <!-- Reason selection -->
      <div class="grid grid-cols-2 gap-2 mb-4">
        {#each commonReasons as reason}
          <button
            type="button"
            on:click={() => selectReason(reason.label)}
            class="p-3 rounded-lg border-2 transition-all text-left text-zinc-300 {
              blockReason === reason.label
                ? 'border-indigo-500 bg-indigo-900'
                : 'border-zinc-600 hover:border-zinc-500'
            }"
            style="background: {blockReason === reason.label ? '' : '#3f3f46'};"
          >
            <span class="text-lg mr-2">{reason.icon}</span>
            <span class="text-sm font-medium">{reason.label}</span>
          </button>
        {/each}
      </div>

      <!-- Evidence/link input -->
      <div class="mb-6">
        <label for="evidence" class="block text-sm font-medium text-zinc-300 mb-2">
          Add evidence or news link (optional)
        </label>
        <textarea id="evidence" bind:value={blockEvidence} placeholder="Paste a news article link or describe what happened..." rows="3" class="w-full px-4 py-3 rounded-lg text-zinc-300 focus:border-indigo-500 focus:ring-2 focus:ring-indigo-200 resize-none surface-panel" ></textarea>
      </div>

      <!-- Actions -->
      <div class="flex gap-3">
        <button type="button" on:click={closeBlockModal} class="flex-1 px-4 py-3 text-zinc-300 rounded-lg hover:bg-zinc-700 font-medium transition-colors surface-panel" >
          Cancel
        </button>
        <button
          type="button"
          on:click={confirmBlock}
          disabled={!blockReason || isBlocking}
          class="flex-1 px-4 py-3 bg-red-600 text-white rounded-xl hover:bg-red-700 font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {#if isBlocking}
            Blocking...
          {:else}
            Block Artist
          {/if}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .line-clamp-2 {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
</style>
