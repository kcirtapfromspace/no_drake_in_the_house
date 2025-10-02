<script lang="ts">
  import { enforcementActions, enforcementStore, executionProgress } from '../stores/enforcement';
  
  $: plan = $enforcementStore.currentPlan;
  $: currentBatch = $enforcementStore.currentBatch;
  $: progress = $executionProgress;

  async function executePlan() {
    if (!plan) return;
    
    const confirmed = confirm(
      'Are you sure you want to execute this enforcement plan? This will modify your music library and some changes may not be reversible.'
    );
    
    if (confirmed) {
      await enforcementActions.executePlan(plan.planId);
    }
  }

  function getStatusColor(status: string) {
    switch (status) {
      case 'pending':
        return 'text-gray-600 bg-gray-100';
      case 'running':
        return 'text-blue-600 bg-blue-100';
      case 'completed':
        return 'text-green-600 bg-green-100';
      case 'failed':
        return 'text-red-600 bg-red-100';
      case 'cancelled':
        return 'text-yellow-600 bg-yellow-100';
      default:
        return 'text-gray-600 bg-gray-100';
    }
  }

  function getActionIcon(action: string) {
    switch (action) {
      case 'remove_liked_song':
        return 'M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z';
      case 'remove_playlist_track':
        return 'M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3';
      case 'unfollow_artist':
        return 'M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z';
      default:
        return 'M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z';
    }
  }


</script>

<div class="space-y-6">
  {#if !plan}
    <!-- No Plan Available -->
    <div class="text-center py-12">
      <svg aria-hidden="true" class="mx-auto icon-uswds icon-uswds--xl text-uswds-base-darker" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
      </svg>
      <h3 class="mt-2 text-uswds-sm font-medium text-uswds-base-darker">No enforcement plan available</h3>
      <p class="mt-1 text-uswds-sm text-uswds-base-darker">Create a plan first to execute enforcement.</p>
    </div>
  {:else if currentBatch}
    <!-- Execution in Progress -->
    <div class="bg-white shadow rounded-uswds-lg p-uswds-6">
      <div class="flex items-center justify-between mb-4">
        <div>
          <h3 class="text-uswds-lg font-medium text-uswds-base-darker">Enforcement Execution</h3>
          <p class="text-uswds-sm text-uswds-base-darker">
            Batch ID: <span class="font-mono">{currentBatch.id.slice(0, 8)}...</span>
          </p>
        </div>
        <span class="flex items-center px-2.5 py-0.5 rounded-full text-uswds-xs font-medium {getStatusColor(currentBatch.status)}">
          {currentBatch.status}
        </span>
      </div>

      <!-- Progress Bar -->
      {#if progress}
        <div class="mb-6">
          <div class="flex justify-between text-uswds-sm text-uswds-base-darker mb-2">
            <span>Progress</span>
            <span>{progress.processed} / {progress.total} ({progress.percentage}%)</span>
          </div>
          <div class="w-full bg-uswds-base-lightest rounded-full h-2">
            <div 
              class="bg-primary h-2 rounded-full transition-all duration-300"
              style="width: {progress.percentage}%"
            ></div>
          </div>
          <div class="flex justify-between text-uswds-xs text-uswds-base-darker mt-1">
            <span>{progress.completed} completed</span>
            <span>{progress.failed} failed</span>
            <span>{progress.skipped} skipped</span>
          </div>
        </div>
      {/if}

      <!-- Batch Summary -->
      <div class="grid grid-cols-1 gap-uswds-4 sm:grid-cols-4 mb-6">
        <div class="bg-uswds-base-lightest rounded-uswds-lg p-uswds-3 text-center">
          <div class="text-uswds-lg font-semibold text-uswds-base-darker">{currentBatch.summary.totalItems}</div>
          <div class="text-uswds-xs text-uswds-base-darker">Total Items</div>
        </div>
        <div class="bg-green-50 rounded-uswds-lg p-uswds-3 text-center">
          <div class="text-uswds-lg font-semibold text-uswds-green-50">{currentBatch.summary.completedItems}</div>
          <div class="text-uswds-xs text-uswds-base-darker">Completed</div>
        </div>
        <div class="bg-red-50 rounded-uswds-lg p-uswds-3 text-center">
          <div class="text-uswds-lg font-semibold text-uswds-red-50">{currentBatch.summary.failedItems}</div>
          <div class="text-uswds-xs text-uswds-base-darker">Failed</div>
        </div>
        <div class="bg-yellow-50 rounded-uswds-lg p-uswds-3 text-center">
          <div class="text-uswds-lg font-semibold text-warning">{currentBatch.summary.skippedItems}</div>
          <div class="text-uswds-xs text-uswds-base-darker">Skipped</div>
        </div>
      </div>

      <!-- Recent Actions -->
      {#if currentBatch.items.length > 0}
        <div>
          <h4 class="text-uswds-sm font-medium text-uswds-base-darker mb-3">Recent Actions</h4>
          <div class="space-y-2 max-h-64 overflow-y-auto">
            {#each currentBatch.items.slice(0, 10) as item}
              <div class="flex items-center justify-between py-2 px-3 bg-uswds-base-lightest rounded-uswds-md">
                <div class="flex items-center space-x-3">
                  <svg aria-hidden="true" class="icon-uswds icon-uswds--sm text-uswds-base-darker" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={getActionIcon(item.action)} />
                  </svg>
                  <div>
                    <div class="text-uswds-sm font-medium text-uswds-base-darker">
                      {item.action.replace(/_/g, ' ')}
                    </div>
                    <div class="text-uswds-xs text-uswds-base-darker">
                      {item.entityType}: {item.entityId.slice(0, 20)}...
                    </div>
                  </div>
                </div>
                <div class="flex items-center space-x-2">
                  <span class="inline-flex items-center px-2 py-0.5 rounded text-uswds-xs font-medium {getStatusColor(item.status)}">
                    {item.status}
                  </span>
                  {#if item.status === 'failed' && item.errorMessage}
                    <button
                      title={item.errorMessage}
                      class="text-uswds-red-50 hover:text-error"
                    >
                      <svg aria-hidden="true" class="icon-uswds icon-uswds--sm" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                      </svg>
                    </button>
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  {:else}
    <!-- Ready to Execute -->
    <div class="bg-white shadow rounded-uswds-lg p-uswds-6">
      <div class="text-center">
        <svg aria-hidden="true" class="mx-auto icon-uswds icon-uswds--xl text-indigo-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.828 14.828a4 4 0 01-5.656 0M9 10h1.01M15 10h1.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        <h3 class="mt-2 text-uswds-lg font-medium text-uswds-base-darker">Ready to Execute</h3>
        <p class="mt-1 text-uswds-sm text-uswds-base-darker">
          Your enforcement plan is ready. Click execute to apply changes to your music library.
        </p>
        
        <div class="mt-6">
          <button
            on:click={executePlan}
            disabled={$enforcementStore.isExecuting}
            class="inline-flex items-center px-6 py-3 border border-transparent text-uswds-base font-medium rounded-uswds-md shadow-sm text-white bg-error hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {#if $enforcementStore.isExecuting}
              <svg aria-hidden="true" class="animate-spin -ml-1 mr-3 icon-uswds icon-uswds--md text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
              Starting Execution...
            {:else}
              <svg aria-hidden="true" class="-ml-1 mr-3 icon-uswds icon-uswds--md" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.828 14.828a4 4 0 01-5.656 0M9 10h1.01M15 10h1.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              Execute Enforcement Plan
            {/if}
          </button>
        </div>

        <div class="mt-4 text-uswds-xs text-uswds-base-darker">
          <p>⚠️ This action will modify your music library</p>
          <p>Some changes may not be reversible</p>
        </div>
      </div>
    </div>
  {/if}

  <!-- Error Display -->
  {#if $enforcementStore.error}
    <div class="bg-red-50 border border-red-200 rounded-uswds-md p-uswds-4">
      <div class="flex">
        <div class="">
          <svg aria-hidden="true" class="icon-uswds icon-uswds--md text-uswds-red-50" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
          </svg>
        </div>
        <div class="ml-3">
          <p class="text-uswds-sm text-uswds-red-50">{$enforcementStore.error}</p>
          <button
            on:click={() => enforcementActions.clearError()}
            class="mt-2 text-uswds-sm text-uswds-red-50 hover:text-red-500"
          >
            Dismiss
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>