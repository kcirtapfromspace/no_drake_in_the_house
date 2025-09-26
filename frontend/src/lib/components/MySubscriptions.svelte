<script lang="ts">
  import { communityActions, communityStore } from '../stores/community';
  
  $: subscriptions = $communityStore.subscriptions;

  async function updateSubscription(listId: string, versionPinned?: number, autoUpdate?: boolean) {
    const result = await communityActions.updateSubscription(listId, versionPinned, autoUpdate);
    if (!result.success) {
      alert(`Failed to update subscription: ${result.message}`);
    }
  }

  async function unsubscribe(listId: string, listName: string) {
    const confirmed = confirm(`Are you sure you want to unsubscribe from "${listName}"?`);
    if (confirmed) {
      const result = await communityActions.unsubscribe(listId);
      if (!result.success) {
        alert(`Failed to unsubscribe: ${result.message}`);
      }
    }
  }

  async function viewListDetails(listId: string) {
    await communityActions.fetchListDetails(listId);
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

<div class="space-y-6">
  <div>
    <h3 class="text-lg font-medium text-gray-900">My Subscriptions</h3>
    <p class="text-sm text-gray-500">
      Manage your community list subscriptions and update preferences.
    </p>
  </div>

  {#if subscriptions.length === 0}
    <!-- Empty State -->
    <div class="text-center py-12">
      <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
      </svg>
      <h3 class="mt-2 text-sm font-medium text-gray-900">No subscriptions yet</h3>
      <p class="mt-1 text-sm text-gray-500">
        Browse community lists to find ones that match your preferences.
      </p>
    </div>
  {:else}
    <!-- Subscriptions List -->
    <div class="bg-white shadow overflow-hidden sm:rounded-md">
      <ul class="divide-y divide-gray-200">
        {#each subscriptions as subscription}
          <li>
            <div class="px-4 py-4 sm:px-6">
              <div class="flex items-center justify-between">
                <div class="flex items-center">
                  <div class="flex-shrink-0">
                    <div class="h-10 w-10 rounded-full bg-indigo-100 flex items-center justify-center">
                      <svg class="h-5 w-5 text-indigo-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
                      </svg>
                    </div>
                  </div>
                  <div class="ml-4">
                    <div class="flex items-center">
                      <p class="text-sm font-medium text-gray-900">
                        {subscription.list.name}
                      </p>
                      <span class="ml-2 inline-flex items-center px-2 py-0.5 rounded text-xs font-medium {getUpdateCadenceColor(subscription.list.update_cadence)}">
                        {subscription.list.update_cadence}
                      </span>
                    </div>
                    <div class="mt-1 flex items-center text-sm text-gray-500">
                      <p>
                        Subscribed {formatDate(subscription.created_at)}
                        • v{subscription.list.version}
                        {#if subscription.version_pinned}
                          (pinned to v{subscription.version_pinned})
                        {/if}
                      </p>
                    </div>
                    <p class="mt-1 text-sm text-gray-600 line-clamp-1">
                      {subscription.list.description}
                    </p>
                  </div>
                </div>
                
                <div class="flex items-center space-x-2">
                  <button
                    on:click={() => viewListDetails(subscription.list_id)}
                    class="text-indigo-600 hover:text-indigo-900 text-sm font-medium"
                  >
                    View Details
                  </button>
                  <button
                    on:click={() => unsubscribe(subscription.list_id, subscription.list.name)}
                    class="text-red-600 hover:text-red-900 text-sm font-medium"
                  >
                    Unsubscribe
                  </button>
                </div>
              </div>
              
              <!-- Subscription Settings -->
              <div class="mt-4 pt-4 border-t border-gray-200">
                <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
                  <!-- Version Pinning -->
                  <div>
                    <h5 class="block text-xs font-medium text-gray-700 mb-2">Version Preference</h5>
                    <div class="space-y-2">
                      <div class="flex items-center">
                        <input
                          id="auto-{subscription.list_id}"
                          type="radio"
                          checked={!subscription.version_pinned}
                          on:change={() => updateSubscription(subscription.list_id, undefined, subscription.auto_update)}
                          class="focus:ring-indigo-500 h-3 w-3 text-indigo-600 border-gray-300"
                        />
                        <label for="auto-{subscription.list_id}" class="ml-2 block text-xs text-gray-700">
                          Auto-update to latest
                        </label>
                      </div>
                      <div class="flex items-center">
                        <input
                          id="pin-{subscription.list_id}"
                          type="radio"
                          checked={!!subscription.version_pinned}
                          on:change={() => updateSubscription(subscription.list_id, subscription.list.version, subscription.auto_update)}
                          class="focus:ring-indigo-500 h-3 w-3 text-indigo-600 border-gray-300"
                        />
                        <label for="pin-{subscription.list_id}" class="ml-2 block text-xs text-gray-700">
                          Pin to v{subscription.list.version}
                        </label>
                      </div>
                    </div>
                  </div>
                  
                  <!-- Auto Update -->
                  <div>
                    <h5 class="block text-xs font-medium text-gray-700 mb-2">Update Notifications</h5>
                    <div class="flex items-start">
                      <div class="flex items-center h-4">
                        <input
                          id="auto-update-{subscription.list_id}"
                          type="checkbox"
                          checked={subscription.auto_update}
                          on:change={(e) => updateSubscription(subscription.list_id, subscription.version_pinned, e.target.checked)}
                          class="focus:ring-indigo-500 h-3 w-3 text-indigo-600 border-gray-300 rounded"
                        />
                      </div>
                      <div class="ml-2 text-xs">
                        <label for="auto-update-{subscription.list_id}" class="font-medium text-gray-700">
                          Enable automatic updates
                        </label>
                        <p class="text-gray-500">
                          Apply changes when the list is updated
                        </p>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </li>
        {/each}
      </ul>
    </div>
  {/if}

  <!-- Subscription Management Info -->
  <div class="bg-blue-50 border border-blue-200 rounded-md p-4">
    <div class="flex">
      <div class="flex-shrink-0">
        <svg class="h-5 w-5 text-blue-400" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd" />
        </svg>
      </div>
      <div class="ml-3">
        <h3 class="text-sm font-medium text-blue-800">
          Managing Your Subscriptions
        </h3>
        <div class="mt-2 text-sm text-blue-700">
          <ul class="list-disc list-inside space-y-1">
            <li><strong>Auto-update:</strong> Automatically apply changes when lists are updated</li>
            <li><strong>Version pinning:</strong> Stay on a specific version to avoid unexpected changes</li>
            <li><strong>Notifications:</strong> Get notified about list updates and changes</li>
            <li><strong>Impact preview:</strong> See what changes will be made before they're applied</li>
          </ul>
        </div>
      </div>
    </div>
  </div>
</div>