<script lang="ts">
  import { enforcementActions, enforcementStore, canRollback } from '../stores/enforcement';

  $: actionHistory = $enforcementStore.actionHistory;

  async function rollbackBatch(batchId: string) {
    const confirmed = confirm(
      'Are you sure you want to rollback this batch? This will attempt to undo the changes made during this enforcement.'
    );

    if (confirmed) {
      const result = await enforcementActions.rollbackBatch(batchId);
      if (!result.success) {
        alert(`Rollback failed: ${result.message}`);
      }
    }
  }

  function getStatusColor(status: string) {
    switch (status) {
      case 'pending':
        return 'text-zinc-400 bg-zinc-400/15';
      case 'running':
        return 'text-blue-400 bg-blue-400/15';
      case 'completed':
        return 'text-green-400 bg-green-400/15';
      case 'failed':
        return 'text-red-400 bg-red-400/15';
      case 'cancelled':
        return 'text-yellow-400 bg-yellow-400/15';
      default:
        return 'text-zinc-400 bg-zinc-400/15';
    }
  }

  function formatDate(dateString: string) {
    return new Date(dateString).toLocaleString();
  }

  function getSuccessRate(batch: any) {
    const total = batch.summary.totalItems;
    const completed = batch.summary.completedItems;
    return total > 0 ? Math.round((completed / total) * 100) : 0;
  }
</script>

<div class="space-y-6">
  <div class="flex justify-between items-center">
    <div>
      <h3 class="text-lg font-medium text-white">Action History</h3>
      <p class="text-sm text-zinc-400">
        View and manage your past enforcement executions.
      </p>
    </div>

    {#if $canRollback}
      <div class="text-sm text-zinc-300">
        <svg aria-hidden="true" class="inline w-4 h-4 text-green-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        Rollback available
      </div>
    {/if}
  </div>

  {#if actionHistory.length === 0}
    <!-- Empty State -->
    <div class="text-center py-12">
      <svg aria-hidden="true" class="mx-auto w-16 h-16 text-zinc-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
      <h3 class="mt-2 text-sm font-medium text-white">No enforcement history</h3>
      <p class="mt-1 text-sm text-zinc-400">
        Your enforcement executions will appear here after you run them.
      </p>
    </div>
  {:else}
    <!-- History List -->
    <div class="overflow-hidden rounded-lg" style="background: #27272a; border: 2px solid #52525b;">
      <ul class="divide-y" style="border-color: #3f3f46;">
        {#each actionHistory as batch}
          <li>
            <div class="px-4 py-4 sm:px-6">
              <div class="flex items-center justify-between">
                <div class="flex items-center">
                  <div class="flex-shrink-0">
                    <div class="w-10 h-10 rounded-lg flex items-center justify-center" style="background: rgba(244, 63, 94, 0.15);">
                      <svg class="w-5 h-5 text-rose-400" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                        <path d="M12 0C5.4 0 0 5.4 0 12s5.4 12 12 12 12-5.4 12-12S18.66 0 12 0zm5.521 17.34c-.24.359-.66.48-1.021.24-2.82-1.74-6.36-2.101-10.561-1.141-.418.122-.779-.179-.899-.539-.12-.421.18-.78.54-.9 4.56-1.021 8.52-.6 11.64 1.32.42.18.479.659.301 1.02zm1.44-3.3c-.301.42-.841.6-1.262.3-3.239-1.98-8.159-2.58-11.939-1.38-.479.12-1.02-.12-1.14-.6-.12-.48.12-1.021.6-1.141C9.6 9.9 15 10.561 18.72 12.84c.361.181.54.78.241 1.2zm.12-3.36C15.24 8.4 8.82 8.16 5.16 9.301c-.6.179-1.2-.181-1.38-.721-.18-.601.18-1.2.72-1.381 4.26-1.26 11.28-1.02 15.721 1.621.539.3.719 1.02.42 1.56-.299.421-1.02.599-1.559.3z"/>
                      </svg>
                    </div>
                  </div>
                  <div class="ml-4">
                    <div class="flex items-center">
                      <p class="text-sm font-medium text-white capitalize">
                        {batch.provider} Enforcement
                      </p>
                      <span class="ml-2 flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {getStatusColor(batch.status)}">
                        {batch.status}
                      </span>
                    </div>
                    <div class="mt-1 flex items-center text-sm text-zinc-400">
                      <p>
                        Executed {formatDate(batch.createdAt)}
                        {#if batch.completedAt}
                          • Completed {formatDate(batch.completedAt)}
                        {/if}
                      </p>
                    </div>
                    <div class="mt-1 text-xs text-zinc-500">
                      Batch ID: <span class="font-mono">{batch.id.slice(0, 8)}...</span>
                    </div>
                  </div>
                </div>

                <div class="flex items-center space-x-4">
                  <!-- Stats -->
                  <div class="text-right">
                    <div class="text-sm font-medium text-white">
                      {getSuccessRate(batch)}% success
                    </div>
                    <div class="text-xs text-zinc-400">
                      {batch.summary.completedItems} / {batch.summary.totalItems} items
                    </div>
                  </div>

                  <!-- Actions -->
                  {#if batch.status === 'completed' && batch.summary.completedItems > 0}
                    <button
                      type="button"
                      on:click={() => rollbackBatch(batch.id)}
                      class="inline-flex items-center px-3 py-2 text-sm leading-4 font-medium rounded-lg text-zinc-300 hover:text-white transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-rose-500"
                      style="background: #3f3f46; border: 1px solid #52525b;"
                    >
                      <svg aria-hidden="true" class="-ml-0.5 mr-2 w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 10h10a8 8 0 018 8v2M3 10l6 6m-6-6l6-6" />
                      </svg>
                      Rollback
                    </button>
                  {/if}
                </div>
              </div>

              <!-- Detailed Stats -->
              <div class="mt-4">
                <div class="grid grid-cols-2 gap-4 sm:grid-cols-4">
                  <div class="text-center p-2 rounded-lg" style="background: #3f3f46;">
                    <div class="text-lg font-semibold text-white">{batch.summary.totalItems}</div>
                    <div class="text-xs text-zinc-400">Total</div>
                  </div>
                  <div class="text-center p-2 rounded-lg" style="background: #3f3f46;">
                    <div class="text-lg font-semibold text-green-400">{batch.summary.completedItems}</div>
                    <div class="text-xs text-zinc-400">Completed</div>
                  </div>
                  <div class="text-center p-2 rounded-lg" style="background: #3f3f46;">
                    <div class="text-lg font-semibold text-red-400">{batch.summary.failedItems}</div>
                    <div class="text-xs text-zinc-400">Failed</div>
                  </div>
                  <div class="text-center p-2 rounded-lg" style="background: #3f3f46;">
                    <div class="text-lg font-semibold text-yellow-400">{batch.summary.skippedItems}</div>
                    <div class="text-xs text-zinc-400">Skipped</div>
                  </div>
                </div>
              </div>

              <!-- Options Used -->
              <div class="mt-3 pt-3" style="border-top: 1px solid #3f3f46;">
                <div class="text-xs text-zinc-400">
                  <span class="font-medium text-zinc-300">Options:</span>
                  {batch.options.aggressiveness} aggressiveness
                  {#if batch.options.blockCollabs}, block collaborations{/if}
                  {#if batch.options.blockFeaturing}, block featuring{/if}
                  {#if batch.options.blockSongwriterOnly}, block songwriter credits{/if}
                </div>
              </div>
            </div>
          </li>
        {/each}
      </ul>
    </div>
  {/if}

  <!-- Info Box -->
  <div class="rounded-lg p-4" style="background: rgba(59, 130, 246, 0.15); border: 1px solid rgba(59, 130, 246, 0.4);">
    <div class="flex">
      <div class="flex-shrink-0">
        <svg aria-hidden="true" class="w-5 h-5 text-blue-400" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd" />
        </svg>
      </div>
      <div class="ml-3">
        <h3 class="text-sm font-medium text-blue-300">
          About Rollbacks
        </h3>
        <div class="mt-2 text-sm text-blue-200">
          <p>
            Rollback attempts to undo changes made during enforcement. Success depends on platform capabilities:
          </p>
          <ul class="list-disc list-inside mt-1 space-y-1 text-blue-300">
            <li>Re-adding liked songs and follows: Usually successful</li>
            <li>Re-adding playlist tracks: May not preserve original order</li>
            <li>Radio seed changes: May not be fully reversible</li>
          </ul>
        </div>
      </div>
    </div>
  </div>
</div>
