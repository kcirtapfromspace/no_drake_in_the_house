<script lang="ts">
  import { onMount } from 'svelte';
  import { enforcementActions, enforcementStore, hasActivePlan } from '../stores/enforcement';
  import { hasActiveSpotifyConnection, hasActiveAppleMusicConnection } from '../stores/connections';
  import { dnpCount } from '../stores/dnp';
  import EnforcementOptions from './EnforcementOptions.svelte';
  import EnforcementPreview from './EnforcementPreview.svelte';
  import EnforcementExecution from './EnforcementExecution.svelte';
  import ActionHistory from './ActionHistory.svelte';
  
  let activeTab = 'plan';
  
  onMount(() => {
    enforcementActions.fetchActionHistory();
  });

  // Track which providers to include
  let selectedProviders: { spotify: boolean; appleMusic: boolean } = {
    spotify: true,
    appleMusic: true
  };

  $: hasAnyConnection = $hasActiveSpotifyConnection || $hasActiveAppleMusicConnection;

  // Check if at least one connected provider is selected
  $: hasSelectedProvider =
    ($hasActiveSpotifyConnection && selectedProviders.spotify) ||
    ($hasActiveAppleMusicConnection && selectedProviders.appleMusic);

  async function createPlan() {
    const providers = [];
    if ($hasActiveSpotifyConnection && selectedProviders.spotify) {
      providers.push('spotify');
    }
    if ($hasActiveAppleMusicConnection && selectedProviders.appleMusic) {
      providers.push('apple_music');
    }

    if (providers.length === 0) {
      return;
    }

    await enforcementActions.createPlan(providers, true);
  }

</script>

<div class="px-4 py-6 sm:px-0">
  <div class="mb-6">
    <h2 class="text-2xl font-bold text-zinc-300">Enforcement Planning</h2>
    <p class="mt-1 text-zinc-300">
      Plan and execute blocklist enforcement across your connected streaming services.
    </p>
  </div>

  <!-- Prerequisites Check -->
  {#if !hasAnyConnection || $dnpCount === 0}
    <div class="mb-6 rounded-uswds-md p-uswds-4" style="background: #3f3f46; border: 1px solid #52525b;">
      <div class="flex">
        <div class="">
          <svg aria-hidden="true" class="icon-uswds icon-uswds--md text-yellow-400" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
          </svg>
        </div>
        <div class="ml-3">
          <h3 class="font-medium text-yellow-400">
            Setup Required
          </h3>
          <div class="mt-2 text-zinc-300">
            <p>Before you can plan enforcement, you need:</p>
            <ul class="list-disc list-inside mt-1 space-y-1">
              {#if !hasAnyConnection}
                <li>Connect at least one streaming service (Spotify or Apple Music)</li>
              {/if}
              {#if $dnpCount === 0}
                <li>Add artists to your Do-Not-Play list</li>
              {/if}
            </ul>
          </div>
        </div>
      </div>
    </div>
  {/if}

  <!-- Tab Navigation -->
  <div class="shadow-sm rounded-uswds-lg mb-6" style="background: #27272a;">
    <nav class="flex space-x-8 px-6" aria-label="Tabs">
      <button
        on:click={() => activeTab = 'plan'}
        class="py-4 px-1 border-b-2 font-medium {activeTab === 'plan' ? 'border-indigo-500 text-indigo-600' : 'border-transparent text-zinc-400 hover:text-zinc-300 hover:border-zinc-500'}"
      >
        Plan Enforcement
      </button>
      <button
        on:click={() => activeTab = 'execute'}
        class="py-4 px-1 border-b-2 font-medium {activeTab === 'execute' ? 'border-indigo-500 text-indigo-600' : 'border-transparent text-zinc-400 hover:text-zinc-300 hover:border-zinc-500'}"
        disabled={!$hasActivePlan}
      >
        Execute Plan
      </button>
      <button
        on:click={() => activeTab = 'history'}
        class="py-4 px-1 border-b-2 font-medium {activeTab === 'history' ? 'border-indigo-500 text-indigo-600' : 'border-transparent text-zinc-400 hover:text-zinc-300 hover:border-zinc-500'}"
      >
        Action History
      </button>
    </nav>
  </div>

  <!-- Tab Content -->
  {#if activeTab === 'plan'}
    <div class="space-y-6">
      <!-- Enforcement Options -->
      <div class="shadow rounded-uswds-lg p-uswds-6" style="background: #27272a;">
        <h3 class="font-medium text-zinc-300 mb-4">Enforcement Options</h3>
        <EnforcementOptions />
      </div>

      <!-- Current Plan Preview -->
      {#if $hasActivePlan}
        <div class="shadow rounded-uswds-lg p-uswds-6" style="background: #27272a;">
          <div class="flex justify-between items-center mb-4">
            <h3 class="font-medium text-zinc-300">Enforcement Preview</h3>
            <button
              on:click={() => enforcementActions.clearPlan()}
              class="text-zinc-300 hover:text-zinc-300"
            >
              Clear Plan
            </button>
          </div>
          <EnforcementPreview />
        </div>
      {:else}
        <!-- Create Plan -->
        <div class="shadow rounded-uswds-lg p-uswds-6" style="background: #27272a;">
          <h3 class="font-medium text-zinc-300 mb-4">Create Enforcement Plan</h3>
          <p class="text-zinc-300 mb-4">
            Generate a dry-run preview to see what changes will be made to your music library.
          </p>

          <!-- Provider Selection -->
          <div class="mb-6">
            <h4 class="text-zinc-300 font-medium mb-3">Select Streaming Services</h4>
            <div class="space-y-3">
              <!-- Spotify -->
              <label class="flex items-center gap-3 p-3 rounded-lg cursor-pointer {$hasActiveSpotifyConnection ? 'bg-zinc-800 hover:bg-zinc-700' : 'bg-zinc-800/50 opacity-50 cursor-not-allowed'}">
                <input
                  type="checkbox"
                  bind:checked={selectedProviders.spotify}
                  disabled={!$hasActiveSpotifyConnection}
                  class="w-5 h-5 rounded border-zinc-600 text-green-500 focus:ring-green-500 disabled:opacity-50"
                />
                <div class="flex items-center gap-2">
                  <!-- Spotify Icon -->
                  <svg class="w-6 h-6" viewBox="0 0 24 24" fill={$hasActiveSpotifyConnection ? '#1DB954' : '#6b7280'}>
                    <path d="M12 0C5.4 0 0 5.4 0 12s5.4 12 12 12 12-5.4 12-12S18.66 0 12 0zm5.521 17.34c-.24.359-.66.48-1.021.24-2.82-1.74-6.36-2.101-10.561-1.141-.418.122-.779-.179-.899-.539-.12-.421.18-.78.54-.9 4.56-1.021 8.52-.6 11.64 1.32.42.18.479.659.301 1.02zm1.44-3.3c-.301.42-.841.6-1.262.3-3.239-1.98-8.159-2.58-11.939-1.38-.479.12-1.02-.12-1.14-.6-.12-.48.12-1.021.6-1.141C9.6 9.9 15 10.561 18.72 12.84c.361.181.54.78.241 1.2zm.12-3.36C15.24 8.4 8.82 8.16 5.16 9.301c-.6.179-1.2-.181-1.38-.721-.18-.601.18-1.2.72-1.381 4.26-1.26 11.28-1.02 15.721 1.621.539.3.719 1.02.419 1.56-.299.421-1.02.599-1.559.3z"/>
                  </svg>
                  <span class="text-zinc-200">Spotify</span>
                </div>
                {#if $hasActiveSpotifyConnection}
                  <span class="ml-auto text-xs text-green-400 bg-green-400/10 px-2 py-1 rounded">Connected</span>
                {:else}
                  <span class="ml-auto text-xs text-zinc-500 bg-zinc-700 px-2 py-1 rounded">Not Connected</span>
                {/if}
              </label>

              <!-- Apple Music -->
              <label class="flex items-center gap-3 p-3 rounded-lg cursor-pointer {$hasActiveAppleMusicConnection ? 'bg-zinc-800 hover:bg-zinc-700' : 'bg-zinc-800/50 opacity-50 cursor-not-allowed'}">
                <input
                  type="checkbox"
                  bind:checked={selectedProviders.appleMusic}
                  disabled={!$hasActiveAppleMusicConnection}
                  class="w-5 h-5 rounded border-zinc-600 text-pink-500 focus:ring-pink-500 disabled:opacity-50"
                />
                <div class="flex items-center gap-2">
                  <!-- Apple Music Icon -->
                  <svg class="w-6 h-6" viewBox="0 0 24 24" fill={$hasActiveAppleMusicConnection ? '#fc3c44' : '#6b7280'}>
                    <path d="M17.05 20.28c-.98.95-2.05.8-3.08.35-1.09-.46-2.09-.48-3.24 0-1.44.62-2.2.44-3.06-.35C2.79 15.25 3.51 7.59 9.05 7.31c1.35.07 2.29.74 3.08.8 1.18-.24 2.31-.93 3.57-.84 1.51.12 2.65.72 3.4 1.8-3.12 1.87-2.38 5.98.48 7.13-.57 1.5-1.31 2.99-2.54 4.09zM12.03 7.25c-.15-2.23 1.66-4.07 3.74-4.25.29 2.58-2.34 4.5-3.74 4.25z"/>
                  </svg>
                  <span class="text-zinc-200">Apple Music</span>
                </div>
                {#if $hasActiveAppleMusicConnection}
                  <span class="ml-auto text-xs text-pink-400 bg-pink-400/10 px-2 py-1 rounded">Connected</span>
                {:else}
                  <span class="ml-auto text-xs text-zinc-500 bg-zinc-700 px-2 py-1 rounded">Not Connected</span>
                {/if}
              </label>
            </div>
          </div>

          <button
            on:click={createPlan}
            disabled={$enforcementStore.isPlanning || !hasAnyConnection || $dnpCount === 0 || !hasSelectedProvider}
            class="flex items-center px-4 py-2 border border-transparent font-medium rounded-uswds-md shadow-sm text-white bg-primary hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {#if $enforcementStore.isPlanning}
              <svg aria-hidden="true" class="animate-spin -ml-1 mr-2 icon-uswds icon-uswds--sm text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
              Creating Plan...
            {:else}
              <svg aria-hidden="true" class="-ml-1 mr-2 icon-uswds icon-uswds--md" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
              </svg>
              Create Enforcement Plan
            {/if}
          </button>
        </div>
      {/if}

      <!-- Error Display -->
      {#if $enforcementStore.error}
        <div class="rounded-uswds-md p-uswds-4" style="background: #3f3f46; border: 1px solid #52525b;">
          <div class="flex">
            <div class="flex-shrink-0">
              <svg aria-hidden="true" class="icon-uswds icon-uswds--md text-zinc-400" viewBox="0 0 20 20" fill="currentColor">
                <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
              </svg>
            </div>
            <div class="ml-3">
              <p class="text-red-400">{$enforcementStore.error}</p>
              <button
                on:click={() => enforcementActions.clearError()}
                class="mt-2 text-red-400 hover:text-red-500"
              >
                Dismiss
              </button>
            </div>
          </div>
        </div>
      {/if}
    </div>
  {:else if activeTab === 'execute'}
    <EnforcementExecution />
  {:else if activeTab === 'history'}
    <ActionHistory />
  {/if}
</div>