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
    if (artist.external_ids.spotify) badges.push({ name: 'Spotify', color: 'text-green-400 bg-green-400/15' });
    if (artist.external_ids.apple) badges.push({ name: 'Apple', color: 'text-zinc-300 bg-zinc-300/15' });
    if (artist.external_ids.musicbrainz) badges.push({ name: 'MusicBrainz', color: 'text-blue-400 bg-blue-400/15' });
    return badges;
  }
</script>

{#if list}
  <div class="space-y-6">
    <!-- Header -->
    <div class="rounded-lg p-6" style="background: #27272a; border: 2px solid #52525b;">
      <div class="flex items-center justify-between mb-4">
        <button
          type="button"
          on:click={goBack}
          class="flex items-center text-sm text-zinc-400 hover:text-white transition-colors"
        >
          <svg aria-hidden="true" class="-ml-1 mr-2 w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
          </svg>
          Back to lists
        </button>

        <div class="flex items-center space-x-2">
          {#if isSubscribed}
            <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium text-green-400 bg-green-400/15">
              Subscribed
            </span>
          {/if}
          <span class="text-sm text-zinc-400">v{list.version}</span>
        </div>
      </div>

      <div class="flex justify-between items-start">
        <div class="flex-1">
          <h1 class="text-2xl font-bold text-white">{list.name}</h1>
          <p class="mt-2 text-zinc-300">{list.description}</p>

          <div class="mt-4 grid grid-cols-1 gap-4 sm:grid-cols-3">
            <div class="text-center p-3 rounded-lg" style="background: #3f3f46;">
              <div class="text-lg font-semibold text-white">{list.artists?.length || 0}</div>
              <div class="text-sm text-zinc-400">Artists</div>
            </div>
            <div class="text-center p-3 rounded-lg" style="background: #3f3f46;">
              <div class="text-lg font-semibold text-white">{list.subscriber_count || 0}</div>
              <div class="text-sm text-zinc-400">Subscribers</div>
            </div>
            <div class="text-center p-3 rounded-lg" style="background: #3f3f46;">
              <div class="text-lg font-semibold text-white">{list.update_cadence}</div>
              <div class="text-sm text-zinc-400">Updates</div>
            </div>
          </div>
        </div>

        <div class="ml-6">
          <button
            type="button"
            on:click={toggleSubscription}
            class="inline-flex items-center px-4 py-2 text-sm font-medium rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2 {isSubscribed
              ? 'text-red-400 hover:text-red-300 focus:ring-red-500'
              : 'text-white bg-rose-500 hover:bg-rose-600 focus:ring-rose-500'}"
            style={isSubscribed ? 'background: rgba(239, 68, 68, 0.15); border: 1px solid rgba(239, 68, 68, 0.4);' : ''}
          >
            {#if isSubscribed}
              <svg aria-hidden="true" class="-ml-1 mr-2 w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
              Unsubscribe
            {:else}
              <svg aria-hidden="true" class="-ml-1 mr-2 w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
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
      <div class="rounded-lg p-6" style="background: #27272a; border: 2px solid #f43f5e;">
        <h3 class="text-lg font-medium text-white mb-4">Subscription Options</h3>

        <div class="space-y-4">
          <div>
            <h4 class="block text-sm font-medium text-white">Version Pinning</h4>
            <div class="mt-2 space-y-2">
              <div class="flex items-center">
                <input
                  id="auto-update"
                  type="radio"
                  bind:group={versionPinned}
                  value={null}
                  class="focus:ring-rose-500 w-4 h-4 text-rose-500"
                />
                <label for="auto-update" class="ml-3 block text-sm text-zinc-300">
                  Auto-update to latest version (recommended)
                </label>
              </div>
              <div class="flex items-center">
                <input
                  id="pin-version"
                  type="radio"
                  bind:group={versionPinned}
                  value={list.version}
                  class="focus:ring-rose-500 w-4 h-4 text-rose-500"
                />
                <label for="pin-version" class="ml-3 block text-sm text-zinc-300">
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
                class="focus:ring-rose-500 w-4 h-4 text-rose-500 rounded"
              />
            </div>
            <div class="ml-3 text-sm">
              <label for="auto-update-checkbox" class="font-medium text-zinc-300">
                Enable automatic updates
              </label>
              <p class="text-zinc-400">
                Receive notifications when the list is updated and apply changes automatically.
              </p>
            </div>
          </div>
        </div>

        <div class="mt-6 flex justify-end space-x-3">
          <button
            type="button"
            on:click={() => showSubscriptionOptions = false}
            class="px-4 py-2 rounded-lg text-sm font-medium text-zinc-300 hover:text-white transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-rose-500"
            style="background: #3f3f46; border: 1px solid #52525b;"
          >
            Cancel
          </button>
          <button
            type="button"
            on:click={confirmSubscription}
            class="px-4 py-2 rounded-lg text-sm font-medium text-white bg-rose-500 hover:bg-rose-600 transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-rose-500"
          >
            Subscribe
          </button>
        </div>
      </div>
    {/if}

    <!-- Criteria and Governance -->
    <div class="rounded-lg p-6" style="background: #27272a; border: 2px solid #52525b;">
      <h3 class="text-lg font-medium text-white mb-4">List Criteria & Governance</h3>

      <div class="space-y-4">
        <div>
          <h4 class="text-sm font-medium text-zinc-300">Inclusion Criteria</h4>
          <p class="mt-1 text-sm text-zinc-400">{list.criteria}</p>
        </div>

        <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
          <div>
            <h4 class="text-sm font-medium text-zinc-300">Update Cadence</h4>
            <p class="mt-1 text-sm text-zinc-400 capitalize">{list.update_cadence}</p>
          </div>
          <div>
            <h4 class="text-sm font-medium text-zinc-300">Last Updated</h4>
            <p class="mt-1 text-sm text-zinc-400">{formatDate(list.updated_at)}</p>
          </div>
        </div>

        {#if list.governance_url}
          <div>
            <h4 class="text-sm font-medium text-zinc-300">Governance Process</h4>
            <a
              href={list.governance_url}
              target="_blank"
              rel="noopener noreferrer"
              class="mt-1 text-sm text-rose-400 hover:text-rose-300 transition-colors"
            >
              View governance documentation →
            </a>
          </div>
        {/if}
      </div>
    </div>

    <!-- Artists List -->
    <div class="rounded-lg p-6" style="background: #27272a; border: 2px solid #52525b;">
      <h3 class="text-lg font-medium text-white mb-4">
        Artists ({list.artists?.length || 0})
      </h3>

      {#if $communityStore.isLoadingList}
        <div class="text-center py-6">
          <svg aria-hidden="true" class="animate-spin mx-auto w-8 h-8 text-rose-500" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          <p class="mt-2 text-sm text-zinc-400">Loading artists...</p>
        </div>
      {:else if list.artists && list.artists.length > 0}
        <div class="space-y-3 max-h-96 overflow-y-auto">
          {#each list.artists as item}
            <div class="flex items-center justify-between py-3 px-4 rounded-lg" style="background: #3f3f46;">
              <div class="flex items-center space-x-3">
                {#if item.artist.metadata.image}
                  <img
                    src={item.artist.metadata.image}
                    alt={item.artist.canonical_name}
                    class="w-10 h-10 rounded-full object-cover"
                  />
                {:else}
                  <div class="w-10 h-10 rounded-full flex items-center justify-center" style="background: #52525b;">
                    <svg aria-hidden="true" class="w-5 h-5 text-zinc-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                    </svg>
                  </div>
                {/if}

                <div>
                  <div class="text-sm font-medium text-white">
                    {item.artist.canonical_name}
                  </div>
                  {#if item.artist.metadata.genres && item.artist.metadata.genres.length > 0}
                    <div class="text-xs text-zinc-400">
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
                <div class="text-xs text-zinc-500">
                  Added {formatDate(item.added_at)}
                </div>
                {#if item.rationale_link}
                  <a
                    href={item.rationale_link}
                    target="_blank"
                    rel="noopener noreferrer"
                    class="text-xs text-rose-400 hover:text-rose-300 transition-colors"
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
          <svg aria-hidden="true" class="mx-auto w-12 h-12 text-zinc-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
          </svg>
          <p class="mt-2 text-sm text-zinc-400">No artists in this list yet.</p>
        </div>
      {/if}
    </div>
  </div>
{:else}
  <div class="text-center py-12">
    <svg aria-hidden="true" class="mx-auto w-16 h-16 text-zinc-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
    </svg>
    <h3 class="mt-2 text-sm font-medium text-white">No list selected</h3>
    <p class="mt-1 text-sm text-zinc-400">Select a list to view its details.</p>
  </div>
{/if}
