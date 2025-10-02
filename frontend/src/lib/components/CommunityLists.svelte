<script lang="ts">
  import { onMount } from 'svelte';
  import { communityActions, communityStore, filteredLists, subscribedListIds } from '../stores/community';
  import CommunityListCard from './CommunityListCard.svelte';
  import CommunityListDetail from './CommunityListDetail.svelte';
  import CreateCommunityList from './CreateCommunityList.svelte';
  import MySubscriptions from './MySubscriptions.svelte';
  
  let activeTab = 'browse';
  let showCreateForm = false;
  
  onMount(async () => {
    await communityActions.fetchLists();
    await communityActions.fetchSubscriptions();
  });

  function setActiveTab(tab: string) {
    activeTab = tab;
    showCreateForm = false;
    communityActions.clearCurrentList();
  }

  function handleSearch(event: Event) {
    const target = event.target as HTMLInputElement;
    communityActions.updateSearch(target.value);
  }

  function handleSort(event: Event) {
    const target = event.target as HTMLSelectElement;
    const [sortBy, sortOrder] = target.value.split(':');
    communityActions.updateSort(sortBy as any, sortOrder as any);
  }
</script>

<div class="px-4 py-6 sm:px-0">
  <div class="mb-6">
    <div class="flex justify-between items-center">
      <div>
        <h2 class="text-uswds-2xl font-bold text-uswds-base-darker">Community Lists</h2>
        <p class="mt-1 text-uswds-sm text-uswds-base-darker">
          Discover and subscribe to community-curated blocklists.
        </p>
      </div>
      <button
        on:click={() => showCreateForm = !showCreateForm}
        class="flex items-center px-4 py-2 border border-transparent text-uswds-sm font-medium rounded-uswds-md shadow-sm text-white bg-primary hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
      >
        <svg aria-hidden="true" class="-ml-1 mr-2 icon-uswds icon-uswds--md" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
        </svg>
        Create List
      </button>
    </div>
  </div>

  <!-- Create List Form -->
  {#if showCreateForm}
    <div class="mb-6 bg-white shadow rounded-uswds-lg p-uswds-6">
      <h3 class="text-uswds-lg font-medium text-uswds-base-darker mb-4">Create Community List</h3>
      <CreateCommunityList on:listCreated={() => showCreateForm = false} />
    </div>
  {/if}

  <!-- Tab Navigation -->
  <div class="bg-white shadow-sm rounded-uswds-lg mb-6">
    <nav class="flex space-x-8 px-6" aria-label="Tabs">
      <button
        on:click={() => setActiveTab('browse')}
        class="py-4 px-1 border-b-2 font-medium text-uswds-sm {activeTab === 'browse' ? 'border-indigo-500 text-primary' : 'border-transparent text-uswds-base-darker hover:text-gray-700 hover:border-gray-300'}"
      >
        Browse Lists
      </button>
      <button
        on:click={() => setActiveTab('subscriptions')}
        class="py-4 px-1 border-b-2 font-medium text-uswds-sm {activeTab === 'subscriptions' ? 'border-indigo-500 text-indigo-600' : 'border-transparent text-uswds-base-darker hover:text-gray-700 hover:border-gray-300'}"
      >
        My Subscriptions ({$subscribedListIds.size})
      </button>
    </nav>
  </div>

  <!-- Tab Content -->
  {#if activeTab === 'browse'}
    <!-- List Detail View -->
    {#if $communityStore.currentList}
      <CommunityListDetail />
    {:else}
      <!-- Browse Lists -->
      <div class="space-y-6">
        <!-- Search and Filter -->
        <div class="bg-white shadow rounded-uswds-lg p-uswds-4">
          <div class="flex flex-col sm:flex-row gap-uswds-4">
            <div class="flex-1">
              <label for="search" class="sr-only">Search lists</label>
              <div class="relative">
                <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                  <svg aria-hidden="true" class="icon-uswds icon-uswds--md text-uswds-base-darker" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                  </svg>
                </div>
                <input
                  id="search"
                  type="text"
                  placeholder="Search lists by name, description, or criteria..."
                  value={$communityStore.searchQuery}
                  on:input={handleSearch}
                  class="block w-full pl-10 pr-3 py-2 border border-gray-300 rounded-uswds-md leading-5 bg-white placeholder-gray-500 focus:outline-none focus:placeholder-gray-400 focus:ring-1 focus:ring-indigo-500 focus:border-indigo-500 sm:text-uswds-sm"
                />
              </div>
            </div>
            
            <div class="sm:w-48">
              <label for="sort" class="sr-only">Sort by</label>
              <select
                id="sort"
                on:change={handleSort}
                class="block w-full pl-3 pr-10 py-2 text-uswds-base border border-gray-300 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-uswds-sm rounded-uswds-md"
              >
                <option value="updated_at:desc">Recently Updated</option>
                <option value="created_at:desc">Newest First</option>
                <option value="name:asc">Name A-Z</option>
                <option value="name:desc">Name Z-A</option>
                <option value="artist_count:desc">Most Artists</option>
                <option value="subscriber_count:desc">Most Subscribers</option>
              </select>
            </div>
          </div>
        </div>

        <!-- Lists Grid -->
        {#if $communityStore.isLoading}
          <div class="text-center py-12">
            <svg aria-hidden="true" class="animate-spin mx-auto icon-uswds icon-uswds--lg text-uswds-base-darker" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            <p class="mt-2 text-uswds-sm text-uswds-base-darker">Loading community lists...</p>
          </div>
        {:else if $communityStore.error}
          <div class="text-center py-12">
            <svg aria-hidden="true" class="mx-auto icon-uswds icon-uswds--lg text-uswds-red-50" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <p class="mt-2 text-uswds-sm text-uswds-red-50">{$communityStore.error}</p>
            <button
              on:click={() => communityActions.fetchLists()}
              class="mt-2 text-uswds-sm text-indigo-600 hover:text-indigo-500"
            >
              Try again
            </button>
          </div>
        {:else if $filteredLists.length === 0}
          <div class="text-center py-12">
            {#if $communityStore.lists.length === 0}
              <svg aria-hidden="true" class="mx-auto icon-uswds icon-uswds--xl text-uswds-base-darker" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
              </svg>
              <h3 class="mt-2 text-uswds-sm font-medium text-uswds-base-darker">No community lists yet</h3>
              <p class="mt-1 text-uswds-sm text-uswds-base-darker">Be the first to create a community list.</p>
              <div class="mt-6">
                <button
                  on:click={() => showCreateForm = true}
                  class="inline-flex items-center px-4 py-2 border border-transparent shadow-sm text-uswds-sm font-medium rounded-uswds-md text-white btn btn-primary focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
                >
                  <svg aria-hidden="true" class="-ml-1 mr-2 icon-uswds icon-uswds--md" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
                  </svg>
                  Create your first list
                </button>
              </div>
            {:else}
              <svg aria-hidden="true" class="mx-auto icon-uswds icon-uswds--xl text-uswds-base-darker" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
              </svg>
              <h3 class="mt-2 text-uswds-sm font-medium text-uswds-base-darker">No lists match your search</h3>
              <p class="mt-1 text-uswds-sm text-uswds-base-darker">Try adjusting your search terms or filters.</p>
            {/if}
          </div>
        {:else}
          <div class="grid grid-cols-1 gap-uswds-6 sm:grid-cols-2 lg:grid-cols-3">
            {#each $filteredLists as list (list.id)}
              <CommunityListCard {list} />
            {/each}
          </div>
        {/if}
      </div>
    {/if}
  {:else if activeTab === 'subscriptions'}
    <MySubscriptions />
  {/if}

  <!-- Info Box -->
  <div class="mt-8 bg-uswds-blue-50 border border-blue-200 rounded-uswds-md p-uswds-4">
    <div class="flex">
      <div class="">
        <svg aria-hidden="true" class="icon-uswds icon-uswds--md text-uswds-blue-50" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd" />
        </svg>
      </div>
      <div class="ml-3">
        <h3 class="text-uswds-sm font-medium text-uswds-blue-50">
          About Community Lists
        </h3>
        <div class="mt-2 text-uswds-sm text-uswds-blue-50">
          <p>
            Community lists are curated blocklists created and maintained by other users. 
            Each list has clear criteria and governance processes to ensure quality and transparency.
          </p>
          <ul class="list-disc list-inside mt-2 space-y-1">
            <li>Subscribe to lists that match your preferences</li>
            <li>Pin specific versions or enable auto-updates</li>
            <li>Preview impact before subscribing</li>
            <li>Appeal decisions through structured processes</li>
          </ul>
        </div>
      </div>
    </div>
  </div>
</div>