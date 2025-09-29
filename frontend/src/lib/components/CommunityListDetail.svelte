<script lang="ts">
  import { communityActions, communityStore, subscribedListIds } from '../stores/community';
  
  $: list = $communityStore.currentList;
  $: isSubscribed = list ? $subscribedListIds.has(list.id) : false;
  
  let showSubscriptionOptions = false;
  let versionPinned: number | null = null;
  let autoUpdate = true;

  function goBack() {
    communityActions.clearCurrentList();
  }

  async function toggleSubscription() {
    if (!list) return;
    
    if (isSubscribed) {
      const result = await communityActions.unsubscribe(list.id);
      if (!result.success) {
        alert(`Failed to unsubscribe: ${result.message}`);
      }
    } else {
      showSubscriptionOptions = true;
    }
  }

  async function confirmSubscription() {
    if (!list) return;
    
    // Get impact preview
    const impact = await communityActions.getSubscriptionImpact(list.id);
    if (impact.success) {
      const confirmed = confirm(
        `This list will add ${impact.data.artists_to_add} artists to your DNP list. Continue?`
      );
      if (confirmed) {
        const result = await communityActions.subscribe(
          list.id, 
          versionPinned || undefined, 
          autoUpdate
        );
        if (result.success) {
          showSubscriptionOptions = false;
        } else {
          alert(`Failed to subscribe: ${result.message}`);
        }
      }
    }
  }

  function formatDate(dateString: string) {
    return new Date(dateString).toLocaleDateString();
  }

  function getProviderBadges(artist: any) {
    const badges = [];
    if (artist.external_ids.spotify) badges.push({ name: 'Spotify', color: 'bg-green-100 text-green-800' });
    if (artist.external_ids.apple) badges.push({ name: 'Apple', color: 'bg-gray-100 text-gray-800' });
    if (artist.external_ids.musicbrainz) badges.push({ name: 'MusicBrainz', color: 'bg-blue-100 text-blue-800' });
    return badges;
  }
</script>

{#if list}
  <div class="space-y-6">
    <!-- Header -->
    <div class="bg-white shadow rounded-lg p-6">
      <div class="flex items-center justify-between mb-4">
        <button
          on:click={goBack}
          class="inline-flex items-center text-sm text-gray-500 hover:text-gray-700"
        >
          <svg class="-ml-1 mr-2 h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
          </svg>
          Back to lists
        </button>
        
        <div class="flex items-center space-x-2">
          {#if isSubscribed}
            <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium text-green-800 bg-green-100">
              Subscribed
            </span>
          {/if}
          <span class="text-sm text-gray-500">v{list.version}</span>
        </div>
      </div>
      
      <div class="flex justify-between items-start">
        <div class="flex-1">
          <h1 class="text-2xl font-bold text-gray-900">{list.name}</h1>
          <p class="mt-2 text-gray-600">{list.description}</p>
          
          <div class="mt-4 grid grid-cols-1 gap-4 sm:grid-cols-3">
            <div class="text-center p-3 bg-gray-50 rounded-lg">
              <div class="text-lg font-semibold text-gray-900">{list.artists?.length || 0}</div>
              <div class="text-sm text-gray-500">Artists</div>
            </div>
            <div class="text-center p-3 bg-gray-50 rounded-lg">
              <div class="text-lg font-semibold text-gray-900">{list.subscriber_count || 0}</div>
              <div class="text-sm text-gray-500">Subscribers</div>
            </div>
            <div class="text-center p-3 bg-gray-50 rounded-lg">
              <div class="text-lg font-semibold text-gray-900">{list.update_cadence}</div>
              <div class="text-sm text-gray-500">Updates</div>
            </div>
          </div>
        </div>
        
        <div class="ml-6">
          <button
            on:click={toggleSubscription}
            class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm {isSubscribed 
              ? 'text-red-700 bg-red-100 hover:bg-red-200 focus:ring-red-500' 
              : 'text-white bg-indigo-600 hover:bg-indigo-700 focus:ring-indigo-500'} focus:outline-none focus:ring-2 focus:ring-offset-2"
          >
            {#if isSubscribed}
              <svg class="-ml-1 mr-2 h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
              Unsubscribe
            {:else}
              <svg class="-ml-1 mr-2 h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
              </svg>
              Subscribe
            {/if}
          </button>
        </div>
      </div>
    </div>

    <!-- Subscription Options Modal -->
    {#if showSubscriptionOptions}
      <div class="bg-white shadow rounded-lg p-6 border-2 border-indigo-200">
        <h3 class="text-lg font-medium text-gray-900 mb-4">Subscription Options</h3>
        
        <div class="space-y-4">
          <div>
            <h4 class="block text-sm font-medium text-gray-700">Version Pinning</h4>
            <div class="mt-2 space-y-2">
              <div class="flex items-center">
                <input
                  id="auto-update"
                  type="radio"
                  bind:group={versionPinned}
                  value={null}
                  class="focus:ring-indigo-500 h-4 w-4 text-indigo-600 border-gray-300"
                />
                <label for="auto-update" class="ml-3 block text-sm text-gray-700">
                  Auto-update to latest version (recommended)
                </label>
              </div>
              <div class="flex items-center">
                <input
                  id="pin-version"
                  type="radio"
                  bind:group={versionPinned}
                  value={list.version}
                  class="focus:ring-indigo-500 h-4 w-4 text-indigo-600 border-gray-300"
                />
                <label for="pin-version" class="ml-3 block text-sm text-gray-700">
                  Pin to current version (v{list.version})
                </label>
              </div>
            </div>
          </div>
          
          <div class="flex items-start">
            <div class="flex items-center h-5">
              <input
                id="auto-update-checkbox"
                type="checkbox"
                bind:checked={autoUpdate}
                class="focus:ring-indigo-500 h-4 w-4 text-indigo-600 border-gray-300 rounded"
              />
            </div>
            <div class="ml-3 text-sm">
              <label for="auto-update-checkbox" class="font-medium text-gray-700">
                Enable automatic updates
              </label>
              <p class="text-gray-500">
                Receive notifications when the list is updated and apply changes automatically.
              </p>
            </div>
          </div>
        </div>
        
        <div class="mt-6 flex justify-end space-x-3">
          <button
            on:click={() => showSubscriptionOptions = false}
            class="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
          >
            Cancel
          </button>
          <button
            on:click={confirmSubscription}
            class="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
          >
            Subscribe
          </button>
        </div>
      </div>
    {/if}

    <!-- Criteria and Governance -->
    <div class="bg-white shadow rounded-lg p-6">
      <h3 class="text-lg font-medium text-gray-900 mb-4">List Criteria & Governance</h3>
      
      <div class="space-y-4">
        <div>
          <h4 class="text-sm font-medium text-gray-900">Inclusion Criteria</h4>
          <p class="mt-1 text-sm text-gray-600">{list.criteria}</p>
        </div>
        
        <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
          <div>
            <h4 class="text-sm font-medium text-gray-900">Update Cadence</h4>
            <p class="mt-1 text-sm text-gray-600 capitalize">{list.update_cadence}</p>
          </div>
          <div>
            <h4 class="text-sm font-medium text-gray-900">Last Updated</h4>
            <p class="mt-1 text-sm text-gray-600">{formatDate(list.updated_at)}</p>
          </div>
        </div>
        
        {#if list.governance_url}
          <div>
            <h4 class="text-sm font-medium text-gray-900">Governance Process</h4>
            <a 
              href={list.governance_url} 
              target="_blank" 
              class="mt-1 text-sm text-indigo-600 hover:text-indigo-500"
            >
              View governance documentation →
            </a>
          </div>
        {/if}
      </div>
    </div>

    <!-- Artists List -->
    <div class="bg-white shadow rounded-lg p-6">
      <h3 class="text-lg font-medium text-gray-900 mb-4">
        Artists ({list.artists?.length || 0})
      </h3>
      
      {#if $communityStore.isLoadingList}
        <div class="text-center py-6">
          <svg class="animate-spin mx-auto h-6 w-6 text-gray-400" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          <p class="mt-2 text-sm text-gray-500">Loading artists...</p>
        </div>
      {:else if list.artists && list.artists.length > 0}
        <div class="space-y-3 max-h-96 overflow-y-auto">
          {#each list.artists as item}
            <div class="flex items-center justify-between py-3 px-4 bg-gray-50 rounded-lg">
              <div class="flex items-center space-x-3">
                {#if item.artist.metadata.image}
                  <img
                    src={item.artist.metadata.image}
                    alt={item.artist.canonical_name}
                    class="h-10 w-10 rounded-full object-cover"
                  />
                {:else}
                  <div class="h-10 w-10 rounded-full bg-gray-300 flex items-center justify-center">
                    <svg class="h-5 w-5 text-gray-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                    </svg>
                  </div>
                {/if}
                
                <div>
                  <div class="text-sm font-medium text-gray-900">
                    {item.artist.canonical_name}
                  </div>
                  {#if item.artist.metadata.genres && item.artist.metadata.genres.length > 0}
                    <div class="text-xs text-gray-500">
                      {item.artist.metadata.genres.slice(0, 2).join(', ')}
                    </div>
                  {/if}
                  <div class="flex space-x-1 mt-1">
                    {#each getProviderBadges(item.artist) as badge}
                      <span class="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium {badge.color}">
                        {badge.name}
                      </span>
                    {/each}
                  </div>
                </div>
              </div>
              
              <div class="text-right">
                <div class="text-xs text-gray-500">
                  Added {formatDate(item.added_at)}
                </div>
                {#if item.rationale_link}
                  <a 
                    href={item.rationale_link} 
                    target="_blank" 
                    class="text-xs text-indigo-600 hover:text-indigo-500"
                  >
                    View rationale
                  </a>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      {:else}
        <div class="text-center py-6">
          <svg class="mx-auto h-8 w-8 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
          </svg>
          <p class="mt-2 text-sm text-gray-500">No artists in this list yet.</p>
        </div>
      {/if}
    </div>
  </div>
{:else}
  <div class="text-center py-12">
    <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
    </svg>
    <h3 class="mt-2 text-sm font-medium text-gray-900">No list selected</h3>
    <p class="mt-1 text-sm text-gray-500">Select a list to view its details.</p>
  </div>
{/if}