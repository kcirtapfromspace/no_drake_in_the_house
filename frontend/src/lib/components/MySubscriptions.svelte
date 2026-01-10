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
        return 'text-red-400 bg-red-400/15';
      case 'weekly':
        return 'text-yellow-400 bg-yellow-400/15';
      case 'monthly':
        return 'text-green-400 bg-green-400/15';
      case 'as-needed':
        return 'text-blue-400 bg-blue-400/15';
      default:
        return 'text-zinc-400 bg-zinc-400/15';
    }
  }

  function handleAutoUpdateChange(listId: string, versionPinned: number | undefined, event: Event) {
    const target = event.target as HTMLInputElement;
    updateSubscription(listId, versionPinned, target.checked);
  }
</script>

<div class="space-y-6">
  <div>
    <h3 class="text-lg font-medium text-white">My Subscriptions</h3>
    <p class="text-sm text-zinc-400">
      Manage your community list subscriptions and update preferences.
    </p>
  </div>

  {#if subscriptions.length === 0}
    <!-- Empty State -->
    <div class="text-center py-12">
      <svg aria-hidden="true" class="mx-auto w-16 h-16 text-zinc-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
      </svg>
      <h3 class="mt-2 text-sm font-medium text-white">No subscriptions yet</h3>
      <p class="mt-1 text-sm text-zinc-400">
        Browse community lists to find ones that match your preferences.
      </p>
    </div>
  {:else}
    <!-- Subscriptions List -->
    <div class="overflow-hidden rounded-lg" style="background: #27272a; border: 2px solid #52525b;">
      <ul class="divide-y" style="border-color: #3f3f46;">
        {#each subscriptions as subscription}
          <li>
            <div class="px-4 py-4 sm:px-6">
              <div class="flex items-center justify-between">
                <div class="flex items-center">
                  <div class="flex-shrink-0">
                    <div class="w-10 h-10 rounded-lg flex items-center justify-center" style="background: rgba(244, 63, 94, 0.15);">
                      <svg aria-hidden="true" class="w-5 h-5 text-rose-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
                      </svg>
                    </div>
                  </div>
                  <div class="ml-4">
                    <div class="flex items-center">
                      <p class="text-sm font-medium text-white">
                        {subscription.list.name}
                      </p>
                      <span class="ml-2 flex items-center px-2 py-0.5 rounded text-xs font-medium {getUpdateCadenceColor(subscription.list.update_cadence)}">
                        {subscription.list.update_cadence}
                      </span>
                    </div>
                    <div class="mt-1 flex items-center text-sm text-zinc-400">
                      <p>
                        Subscribed {formatDate(subscription.created_at)}
                        • v{subscription.list.version}
                        {#if subscription.version_pinned}
                          (pinned to v{subscription.version_pinned})
                        {/if}
                      </p>
                    </div>
                    <p class="mt-1 text-sm text-zinc-300 line-clamp-1">
                      {subscription.list.description}
                    </p>
                  </div>
                </div>

                <div class="flex items-center space-x-3">
                  <button
                    type="button"
                    on:click={() => viewListDetails(subscription.list_id)}
                    class="text-rose-400 hover:text-rose-300 text-sm font-medium transition-colors"
                  >
                    View Details
                  </button>
                  <button
                    type="button"
                    on:click={() => unsubscribe(subscription.list_id, subscription.list.name)}
                    class="text-red-400 hover:text-red-300 text-sm font-medium transition-colors"
                  >
                    Unsubscribe
                  </button>
                </div>
              </div>

              <!-- Subscription Settings -->
              <div class="mt-4 pt-4" style="border-top: 1px solid #3f3f46;">
                <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
                  <!-- Version Pinning -->
                  <div>
                    <h5 class="block text-xs font-medium text-zinc-400 mb-2">Version Preference</h5>
                    <div class="space-y-2">
                      <div class="flex items-center">
                        <input
                          id="auto-{subscription.list_id}"
                          type="radio"
                          checked={!subscription.version_pinned}
                          on:change={() => updateSubscription(subscription.list_id, undefined, subscription.auto_update)}
                          class="focus:ring-rose-500 h-3 w-3 text-rose-500"
                        />
                        <label for="auto-{subscription.list_id}" class="ml-2 block text-xs text-zinc-300">
                          Auto-update to latest
                        </label>
                      </div>
                      <div class="flex items-center">
                        <input
                          id="pin-{subscription.list_id}"
                          type="radio"
                          checked={!!subscription.version_pinned}
                          on:change={() => updateSubscription(subscription.list_id, subscription.list.version, subscription.auto_update)}
                          class="focus:ring-rose-500 h-3 w-3 text-rose-500"
                        />
                        <label for="pin-{subscription.list_id}" class="ml-2 block text-xs text-zinc-300">
                          Pin to v{subscription.list.version}
                        </label>
                      </div>
                    </div>
                  </div>

                  <!-- Auto Update -->
                  <div>
                    <h5 class="block text-xs font-medium text-zinc-400 mb-2">Update Notifications</h5>
                    <div class="flex items-start">
                      <div class="flex items-center h-4">
                        <input
                          id="auto-update-{subscription.list_id}"
                          type="checkbox"
                          checked={subscription.auto_update}
                          on:change={(e) => handleAutoUpdateChange(subscription.list_id, subscription.version_pinned, e)}
                          class="focus:ring-rose-500 h-3 w-3 text-rose-500 rounded"
                        />
                      </div>
                      <div class="ml-2 text-xs">
                        <label for="auto-update-{subscription.list_id}" class="font-medium text-zinc-300">
                          Enable automatic updates
                        </label>
                        <p class="text-zinc-500">
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
  <div class="rounded-lg p-4" style="background: rgba(59, 130, 246, 0.15); border: 1px solid rgba(59, 130, 246, 0.4);">
    <div class="flex">
      <div class="flex-shrink-0">
        <svg aria-hidden="true" class="w-5 h-5 text-blue-400" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd" />
        </svg>
      </div>
      <div class="ml-3">
        <h3 class="text-sm font-medium text-blue-300">
          Managing Your Subscriptions
        </h3>
        <div class="mt-2 text-sm text-blue-200">
          <ul class="list-disc list-inside space-y-1">
            <li><strong class="text-blue-300">Auto-update:</strong> Automatically apply changes when lists are updated</li>
            <li><strong class="text-blue-300">Version pinning:</strong> Stay on a specific version to avoid unexpected changes</li>
            <li><strong class="text-blue-300">Notifications:</strong> Get notified about list updates and changes</li>
            <li><strong class="text-blue-300">Impact preview:</strong> See what changes will be made before they're applied</li>
          </ul>
        </div>
      </div>
    </div>
  </div>
</div>
