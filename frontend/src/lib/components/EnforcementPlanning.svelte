<script lang="ts">
  import { onMount } from 'svelte';
  import { enforcementActions, enforcementStore, hasActivePlan } from '../stores/enforcement';
  import { hasActiveSpotifyConnection } from '../stores/connections';
  import { dnpCount } from '../stores/dnp';
  import EnforcementOptions from './EnforcementOptions.svelte';
  import EnforcementPreview from './EnforcementPreview.svelte';
  import EnforcementExecution from './EnforcementExecution.svelte';
  import ActionHistory from './ActionHistory.svelte';
  
  let activeTab = 'plan';
  
  onMount(() => {
    enforcementActions.fetchActionHistory();
  });

  async function createPlan() {
    const providers = [];
    if ($hasActiveSpotifyConnection) {
      providers.push('spotify');
    }
    
    if (providers.length === 0) {
      return;
    }
    
    await enforcementActions.createPlan(providers, true);
  }

  function setActiveTab(tab: string) {
    activeTab = tab;
  }
</script>

<div class="px-4 py-6 sm:px-0">
  <div class="mb-6">
    <h2 class="text-2xl font-bold text-gray-900">Enforcement Planning</h2>
    <p class="mt-1 text-sm text-gray-600">
      Plan and execute blocklist enforcement across your connected streaming services.
    </p>
  </div>

  <!-- Prerequisites Check -->
  {#if !$hasActiveSpotifyConnection || $dnpCount === 0}
    <div class="mb-6 bg-yellow-50 border border-yellow-200 rounded-md p-4">
      <div class="flex">
        <div class="flex-shrink-0">
          <svg class="h-5 w-5 text-yellow-400" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
          </svg>
        </div>
        <div class="ml-3">
          <h3 class="text-sm font-medium text-yellow-800">
            Setup Required
          </h3>
          <div class="mt-2 text-sm text-yellow-700">
            <p>Before you can plan enforcement, you need:</p>
            <ul class="list-disc list-inside mt-1 space-y-1">
              {#if !$hasActiveSpotifyConnection}
                <li>Connect at least one streaming service (Spotify)</li>
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
  <div class="bg-white shadow-sm rounded-lg mb-6">
    <nav class="flex space-x-8 px-6" aria-label="Tabs">
      <button
        on:click={() => setActiveTab('plan')}
        class="py-4 px-1 border-b-2 font-medium text-sm {activeTab === 'plan' ? 'border-indigo-500 text-indigo-600' : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'}"
      >
        Plan Enforcement
      </button>
      <button
        on:click={() => setActiveTab('execute')}
        class="py-4 px-1 border-b-2 font-medium text-sm {activeTab === 'execute' ? 'border-indigo-500 text-indigo-600' : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'}"
        disabled={!$hasActivePlan}
      >
        Execute Plan
      </button>
      <button
        on:click={() => setActiveTab('history')}
        class="py-4 px-1 border-b-2 font-medium text-sm {activeTab === 'history' ? 'border-indigo-500 text-indigo-600' : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'}"
      >
        Action History
      </button>
    </nav>
  </div>

  <!-- Tab Content -->
  {#if activeTab === 'plan'}
    <div class="space-y-6">
      <!-- Enforcement Options -->
      <div class="bg-white shadow rounded-lg p-6">
        <h3 class="text-lg font-medium text-gray-900 mb-4">Enforcement Options</h3>
        <EnforcementOptions />
      </div>

      <!-- Current Plan Preview -->
      {#if $hasActivePlan}
        <div class="bg-white shadow rounded-lg p-6">
          <div class="flex justify-between items-center mb-4">
            <h3 class="text-lg font-medium text-gray-900">Enforcement Preview</h3>
            <button
              on:click={() => enforcementActions.clearPlan()}
              class="text-sm text-gray-500 hover:text-gray-700"
            >
              Clear Plan
            </button>
          </div>
          <EnforcementPreview />
        </div>
      {:else}
        <!-- Create Plan -->
        <div class="bg-white shadow rounded-lg p-6">
          <h3 class="text-lg font-medium text-gray-900 mb-4">Create Enforcement Plan</h3>
          <p class="text-sm text-gray-600 mb-4">
            Generate a dry-run preview to see what changes will be made to your music library.
          </p>
          
          <button
            on:click={createPlan}
            disabled={$enforcementStore.isPlanning || !$hasActiveSpotifyConnection || $dnpCount === 0}
            class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {#if $enforcementStore.isPlanning}
              <svg class="animate-spin -ml-1 mr-2 h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
              Creating Plan...
            {:else}
              <svg class="-ml-1 mr-2 h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
              </svg>
              Create Enforcement Plan
            {/if}
          </button>
        </div>
      {/if}

      <!-- Error Display -->
      {#if $enforcementStore.error}
        <div class="bg-red-50 border border-red-200 rounded-md p-4">
          <div class="flex">
            <div class="flex-shrink-0">
              <svg class="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
                <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
              </svg>
            </div>
            <div class="ml-3">
              <p class="text-sm text-red-800">{$enforcementStore.error}</p>
              <button
                on:click={() => enforcementActions.clearError()}
                class="mt-2 text-sm text-red-600 hover:text-red-500"
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