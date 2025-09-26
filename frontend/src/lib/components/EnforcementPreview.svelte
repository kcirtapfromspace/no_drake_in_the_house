<script lang="ts">
  import { enforcementStore } from '../stores/enforcement';
  
  $: plan = $enforcementStore.currentPlan;
  
  function formatDuration(duration: string) {
    const seconds = parseInt(duration.replace('s', ''));
    if (seconds < 60) return `${seconds} seconds`;
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = seconds % 60;
    return `${minutes}m ${remainingSeconds}s`;
  }

  function getProviderIcon(provider: string) {
    switch (provider) {
      case 'spotify':
        return 'M12 0C5.4 0 0 5.4 0 12s5.4 12 12 12 12-5.4 12-12S18.66 0 12 0zm5.521 17.34c-.24.359-.66.48-1.021.24-2.82-1.74-6.36-2.101-10.561-1.141-.418.122-.779-.179-.899-.539-.12-.421.18-.78.54-.9 4.56-1.021 8.52-.6 11.64 1.32.42.18.479.659.301 1.02zm1.44-3.3c-.301.42-.841.6-1.262.3-3.239-1.98-8.159-2.58-11.939-1.38-.479.12-1.02-.12-1.14-.6-.12-.48.12-1.021.6-1.141C9.6 9.9 15 10.561 18.72 12.84c.361.181.54.78.241 1.2zm.12-3.36C15.24 8.4 8.82 8.16 5.16 9.301c-.6.179-1.2-.181-1.38-.721-.18-.601.18-1.2.72-1.381 4.26-1.26 11.28-1.02 15.721 1.621.539.3.719 1.02.42 1.56-.299.421-1.02.599-1.559.3z';
      default:
        return 'M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1';
    }
  }

  function getCapabilityColor(capability: string) {
    switch (capability) {
      case 'SUPPORTED':
        return 'text-green-600 bg-green-100';
      case 'LIMITED':
        return 'text-yellow-600 bg-yellow-100';
      case 'UNSUPPORTED':
        return 'text-red-600 bg-red-100';
      default:
        return 'text-gray-600 bg-gray-100';
    }
  }
</script>

{#if plan}
  <div class="space-y-6">
    <!-- Plan Summary -->
    <div class="bg-gray-50 rounded-lg p-4">
      <div class="flex items-center justify-between">
        <div>
          <h4 class="text-sm font-medium text-gray-900">Plan Summary</h4>
          <p class="text-sm text-gray-500">
            Estimated duration: {formatDuration(plan.estimatedDuration)}
            {#if plan.resumable}
              • Resumable if interrupted
            {/if}
          </p>
        </div>
        <div class="text-right">
          <div class="text-sm font-medium text-gray-900">Plan ID</div>
          <div class="text-xs text-gray-500 font-mono">{plan.planId.slice(0, 8)}...</div>
        </div>
      </div>
    </div>

    <!-- Provider Impact -->
    {#each Object.entries(plan.impact) as [provider, impact]}
      <div class="border border-gray-200 rounded-lg p-6">
        <div class="flex items-center mb-4">
          <div class="flex-shrink-0">
            <svg class="h-8 w-8 text-green-500" fill="currentColor" viewBox="0 0 24 24">
              <path d={getProviderIcon(provider)} />
            </svg>
          </div>
          <div class="ml-3">
            <h4 class="text-lg font-medium text-gray-900 capitalize">{provider}</h4>
            <p class="text-sm text-gray-500">Impact preview for your {provider} library</p>
          </div>
        </div>

        <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
          <!-- Liked Songs -->
          {#if impact.likedSongs}
            <div class="bg-white border border-gray-200 rounded-lg p-4">
              <div class="flex items-center">
                <div class="flex-shrink-0">
                  <svg class="h-6 w-6 text-red-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z" />
                  </svg>
                </div>
                <div class="ml-3">
                  <p class="text-sm font-medium text-gray-900">Liked Songs</p>
                  <p class="text-sm text-gray-500">
                    {impact.likedSongs.toRemove} to remove
                    {#if impact.likedSongs.collabsFound > 0}
                      <br /><span class="text-xs">({impact.likedSongs.collabsFound} collaborations)</span>
                    {/if}
                  </p>
                </div>
              </div>
            </div>
          {/if}

          <!-- Playlists -->
          {#if impact.playlists}
            <div class="bg-white border border-gray-200 rounded-lg p-4">
              <div class="flex items-center">
                <div class="flex-shrink-0">
                  <svg class="h-6 w-6 text-blue-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3" />
                  </svg>
                </div>
                <div class="ml-3">
                  <p class="text-sm font-medium text-gray-900">Playlists</p>
                  <p class="text-sm text-gray-500">
                    {impact.playlists.toScrub} playlists affected
                    <br /><span class="text-xs">{impact.playlists.tracksToRemove} tracks to remove</span>
                    {#if impact.playlists.featuringFound > 0}
                      <br /><span class="text-xs">({impact.playlists.featuringFound} featuring)</span>
                    {/if}
                  </p>
                </div>
              </div>
            </div>
          {/if}

          <!-- Following -->
          {#if impact.following}
            <div class="bg-white border border-gray-200 rounded-lg p-4">
              <div class="flex items-center">
                <div class="flex-shrink-0">
                  <svg class="h-6 w-6 text-purple-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                  </svg>
                </div>
                <div class="ml-3">
                  <p class="text-sm font-medium text-gray-900">Following</p>
                  <p class="text-sm text-gray-500">
                    {impact.following.toUnfollow} to unfollow
                  </p>
                </div>
              </div>
            </div>
          {/if}

          <!-- Radio Seeds -->
          {#if impact.radioSeeds}
            <div class="bg-white border border-gray-200 rounded-lg p-4">
              <div class="flex items-center">
                <div class="flex-shrink-0">
                  <svg class="h-6 w-6 text-orange-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 4V2a1 1 0 011-1h8a1 1 0 011 1v2m-9 0h10m-10 0a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V6a2 2 0 00-2-2M7 4h10" />
                  </svg>
                </div>
                <div class="ml-3">
                  <p class="text-sm font-medium text-gray-900">Radio Seeds</p>
                  <p class="text-sm text-gray-500">
                    {impact.radioSeeds.toFilter} to filter
                  </p>
                </div>
              </div>
            </div>
          {/if}
        </div>

        <!-- Capabilities -->
        {#if plan.capabilities[provider]}
          <div class="mt-4 pt-4 border-t border-gray-200">
            <h5 class="text-sm font-medium text-gray-900 mb-2">Platform Capabilities</h5>
            <div class="flex flex-wrap gap-2">
              {#each Object.entries(plan.capabilities[provider]) as [capability, support]}
                <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {getCapabilityColor(support)}">
                  {capability.replace(/_/g, ' ').toLowerCase()}
                </span>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    {/each}

    <!-- Important Notes -->
    <div class="bg-blue-50 border border-blue-200 rounded-md p-4">
      <div class="flex">
        <div class="flex-shrink-0">
          <svg class="h-5 w-5 text-blue-400" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd" />
          </svg>
        </div>
        <div class="ml-3">
          <h3 class="text-sm font-medium text-blue-800">
            Before You Execute
          </h3>
          <div class="mt-2 text-sm text-blue-700">
            <ul class="list-disc list-inside space-y-1">
              <li>This is a preview - no changes have been made yet</li>
              <li>Execution will modify your actual music library</li>
              <li>Some actions may not be reversible depending on platform limitations</li>
              <li>The process can be interrupted and resumed if needed</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  </div>
{:else}
  <div class="text-center py-6">
    <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
    </svg>
    <h3 class="mt-2 text-sm font-medium text-gray-900">No enforcement plan</h3>
    <p class="mt-1 text-sm text-gray-500">Create a plan to see the preview.</p>
  </div>
{/if}