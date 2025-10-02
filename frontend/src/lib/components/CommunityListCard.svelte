<script lang="ts">
  import { communityActions, subscribedListIds, type CommunityList } from '../stores/community';
  
  export let list: CommunityList;
  
  $: isSubscribed = $subscribedListIds.has(list.id);

  async function viewDetails() {
    await communityActions.fetchListDetails(list.id);
  }

  async function toggleSubscription() {
    if (isSubscribed) {
      const result = await communityActions.unsubscribe(list.id);
      if (!result.success) {
        alert(`Failed to unsubscribe: ${result.message}`);
      }
    } else {
      // Show impact preview first
      const impact = await communityActions.getSubscriptionImpact(list.id);
      if (impact.success) {
        const confirmed = confirm(
          `This list will add ${impact.data.artists_to_add} artists to your DNP list. Continue?`
        );
        if (confirmed) {
          const result = await communityActions.subscribe(list.id);
          if (!result.success) {
            alert(`Failed to subscribe: ${result.message}`);
          }
        }
      }
    }
  }

  function formatDate(dateString: string) {
    return new Date(dateString).toLocaleDateString();
  }

  function getUpdateCadenceColor(cadence: string) {
    switch (cadence.toLowerCase()) {
      case 'daily':
        return 'text-red-600 bg-red-100';
      case 'weekly':
        return 'text-yellow-600 bg-yellow-100';
      case 'monthly':
        return 'text-green-600 bg-green-100';
      case 'as-needed':
        return 'text-blue-600 bg-blue-100';
      default:
        return 'text-gray-600 bg-gray-100';
    }
  }
</script>

<div class="bg-white overflow-hidden shadow rounded-uswds-lg hover:shadow-md transition-shadow">
  <div class="p-uswds-6">
    <div class="flex items-center justify-between">
      <div class="flex items-center">
        <h3 class="text-uswds-lg font-medium text-uswds-base-darker truncate">
          {list.name}
        </h3>
        {#if isSubscribed}
          <span class="ml-2 flex items-center px-2 py-0.5 rounded text-uswds-xs font-medium text-uswds-green-50 bg-green-100">
            Subscribed
          </span>
        {/if}
      </div>
      <span class="inline-flex items-center px-2 py-0.5 rounded text-uswds-xs font-medium {getUpdateCadenceColor(list.update_cadence)}">
        {list.update_cadence}
      </span>
    </div>
    
    <p class="mt-2 text-uswds-sm text-uswds-base-darker line-clamp-uswds-2">
      {list.description}
    </p>
    
    <div class="mt-3">
      <h4 class="text-uswds-xs font-medium text-uswds-base-darker uppercase tracking-wide">Criteria</h4>
      <p class="mt-1 text-uswds-sm text-uswds-base-darker line-clamp-uswds-2">
        {list.criteria}
      </p>
    </div>
    
    <div class="mt-4 flex items-center justify-between text-uswds-sm text-uswds-base-darker">
      <div class="flex items-center space-x-4">
        <span>
          <svg aria-hidden="true" class="inline icon-uswds icon-uswds--sm text-uswds-base-darker mr-1" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
          </svg>
          {list.artist_count || 0} artists
        </span>
        <span>
          <svg aria-hidden="true" class="inline icon-uswds icon-uswds--sm text-uswds-base-darker mr-1" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
          </svg>
          {list.subscriber_count || 0} subscribers
        </span>
      </div>
      <span>v{list.version}</span>
    </div>
    
    <div class="mt-4 text-uswds-xs text-uswds-base-darker">
      Updated {formatDate(list.updated_at)}
      {#if list.governance_url}
        • <a href={list.governance_url} target="_blank" class="text-primary hover:text-indigo-500">Governance</a>
      {/if}
    </div>
  </div>
  
  <div class="bg-uswds-base-lightest px-6 py-3">
    <div class="flex justify-between items-center">
      <button
        on:click={viewDetails}
        class="text-uswds-sm text-indigo-600 hover:text-indigo-500 font-medium"
      >
        View Details
      </button>
      
      <button
        on:click={toggleSubscription}
        class="inline-flex items-center px-3 py-2 border border-transparent text-uswds-sm leading-4 font-medium rounded-uswds-md {isSubscribed 
          ? 'text-uswds-red-50 bg-red-100 hover:bg-red-200 focus:ring-red-500' 
          : 'text-indigo-700 bg-indigo-100 hover:bg-indigo-200 focus:ring-indigo-500'} focus:outline-none focus:ring-2 focus:ring-offset-2"
      >
        {#if isSubscribed}
          <svg aria-hidden="true" class="-ml-0.5 mr-2 icon-uswds icon-uswds--sm" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
          Unsubscribe
        {:else}
          <svg aria-hidden="true" class="-ml-0.5 mr-2 icon-uswds icon-uswds--sm" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
          </svg>
          Subscribe
        {/if}
      </button>
    </div>
  </div>
</div>