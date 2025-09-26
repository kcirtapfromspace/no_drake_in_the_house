<script lang="ts">
  import { onMount } from 'svelte';
  import { authActions, currentUser } from '../stores/auth';
  import { connectionActions, connectionsStore, connectedServices, hasActiveSpotifyConnection } from '../stores/connections';
  import { dnpActions, dnpStore, dnpCount } from '../stores/dnp';
  import ServiceConnections from './ServiceConnections.svelte';
  import DnpManager from './DnpManager.svelte';
  import EnforcementPlanning from './EnforcementPlanning.svelte';
  import CommunityLists from './CommunityLists.svelte';
  
  let activeTab = 'overview';
  
  onMount(async () => {
    await connectionActions.fetchConnections();
    await dnpActions.fetchDnpList();
  });

  function handleLogout() {
    authActions.logout();
  }

  function setActiveTab(tab: string) {
    activeTab = tab;
  }
</script>

<div class="min-h-screen bg-gray-50">
  <!-- Navigation -->
  <nav class="bg-white shadow-sm border-b border-gray-200">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
      <div class="flex justify-between h-16">
        <div class="flex items-center">
          <h1 class="text-xl font-semibold text-gray-900">
            Music Streaming Blocklist Manager
          </h1>
        </div>
        
        <div class="flex items-center space-x-4">
          <span class="text-sm text-gray-700">
            {$currentUser?.email}
          </span>
          <button
            on:click={handleLogout}
            class="text-sm text-gray-500 hover:text-gray-700"
          >
            Sign out
          </button>
        </div>
      </div>
    </div>
  </nav>

  <!-- Tab Navigation -->
  <div class="bg-white shadow-sm">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
      <nav class="flex space-x-8" aria-label="Tabs">
        <button
          on:click={() => setActiveTab('overview')}
          class="py-4 px-1 border-b-2 font-medium text-sm {activeTab === 'overview' ? 'border-indigo-500 text-indigo-600' : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'}"
        >
          Overview
        </button>
        <button
          on:click={() => setActiveTab('connections')}
          class="py-4 px-1 border-b-2 font-medium text-sm {activeTab === 'connections' ? 'border-indigo-500 text-indigo-600' : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'}"
        >
          Connections
        </button>
        <button
          on:click={() => setActiveTab('dnp')}
          class="py-4 px-1 border-b-2 font-medium text-sm {activeTab === 'dnp' ? 'border-indigo-500 text-indigo-600' : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'}"
        >
          DNP List ({$dnpCount})
        </button>
        <button
          on:click={() => setActiveTab('enforcement')}
          class="py-4 px-1 border-b-2 font-medium text-sm {activeTab === 'enforcement' ? 'border-indigo-500 text-indigo-600' : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'}"
        >
          Enforcement
        </button>
        <button
          on:click={() => setActiveTab('community')}
          class="py-4 px-1 border-b-2 font-medium text-sm {activeTab === 'community' ? 'border-indigo-500 text-indigo-600' : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'}"
        >
          Community Lists
        </button>
      </nav>
    </div>
  </div>

  <!-- Main Content -->
  <main class="max-w-7xl mx-auto py-6 sm:px-6 lg:px-8">
    {#if activeTab === 'overview'}
      <!-- Overview Tab -->
      <div class="px-4 py-6 sm:px-0">
        <div class="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
          <!-- Connected Services Card -->
          <div class="bg-white overflow-hidden shadow rounded-lg">
            <div class="p-5">
              <div class="flex items-center">
                <div class="flex-shrink-0">
                  <svg class="h-6 w-6 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
                  </svg>
                </div>
                <div class="ml-5 w-0 flex-1">
                  <dl>
                    <dt class="text-sm font-medium text-gray-500 truncate">
                      Connected Services
                    </dt>
                    <dd class="text-lg font-medium text-gray-900">
                      {$connectedServices.length}
                    </dd>
                  </dl>
                </div>
              </div>
            </div>
            <div class="bg-gray-50 px-5 py-3">
              <div class="text-sm">
                <button
                  on:click={() => setActiveTab('connections')}
                  class="font-medium text-indigo-700 hover:text-indigo-900"
                >
                  Manage connections
                </button>
              </div>
            </div>
          </div>

          <!-- DNP List Card -->
          <div class="bg-white overflow-hidden shadow rounded-lg">
            <div class="p-5">
              <div class="flex items-center">
                <div class="flex-shrink-0">
                  <svg class="h-6 w-6 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728L5.636 5.636m12.728 12.728L5.636 5.636" />
                  </svg>
                </div>
                <div class="ml-5 w-0 flex-1">
                  <dl>
                    <dt class="text-sm font-medium text-gray-500 truncate">
                      Blocked Artists
                    </dt>
                    <dd class="text-lg font-medium text-gray-900">
                      {$dnpCount}
                    </dd>
                  </dl>
                </div>
              </div>
            </div>
            <div class="bg-gray-50 px-5 py-3">
              <div class="text-sm">
                <button
                  on:click={() => setActiveTab('dnp')}
                  class="font-medium text-indigo-700 hover:text-indigo-900"
                >
                  Manage DNP list
                </button>
              </div>
            </div>
          </div>

          <!-- Status Card -->
          <div class="bg-white overflow-hidden shadow rounded-lg">
            <div class="p-5">
              <div class="flex items-center">
                <div class="flex-shrink-0">
                  <svg class="h-6 w-6 {$hasActiveSpotifyConnection ? 'text-green-400' : 'text-gray-400'}" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                </div>
                <div class="ml-5 w-0 flex-1">
                  <dl>
                    <dt class="text-sm font-medium text-gray-500 truncate">
                      System Status
                    </dt>
                    <dd class="text-lg font-medium text-gray-900">
                      {$hasActiveSpotifyConnection ? 'Active' : 'Setup Required'}
                    </dd>
                  </dl>
                </div>
              </div>
            </div>
            <div class="bg-gray-50 px-5 py-3">
              <div class="text-sm">
                {#if !$hasActiveSpotifyConnection}
                  <button
                    on:click={() => setActiveTab('connections')}
                    class="font-medium text-indigo-700 hover:text-indigo-900"
                  >
                    Connect Spotify
                  </button>
                {:else}
                  <span class="text-green-700">Ready to use</span>
                {/if}
              </div>
            </div>
          </div>
        </div>

        <!-- Quick Actions -->
        <div class="mt-8">
          <h3 class="text-lg leading-6 font-medium text-gray-900 mb-4">
            Quick Actions
          </h3>
          <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
            <button
              on:click={() => setActiveTab('dnp')}
              class="relative block w-full border-2 border-gray-300 border-dashed rounded-lg p-6 text-center hover:border-gray-400 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
            >
              <svg class="mx-auto h-8 w-8 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
              </svg>
              <span class="mt-2 block text-sm font-medium text-gray-900">
                Add Artist to DNP List
              </span>
            </button>
            
            {#if $hasActiveSpotifyConnection && $dnpCount > 0}
              <button
                on:click={() => setActiveTab('enforcement')}
                class="relative block w-full border-2 border-gray-300 border-dashed rounded-lg p-6 text-center hover:border-gray-400 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
              >
                <svg class="mx-auto h-8 w-8 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.828 14.828a4 4 0 01-5.656 0M9 10h1.01M15 10h1.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                <span class="mt-2 block text-sm font-medium text-gray-900">
                  Plan Enforcement
                </span>
              </button>
            {:else}
              <button
                on:click={() => setActiveTab('connections')}
                class="relative block w-full border-2 border-gray-300 border-dashed rounded-lg p-6 text-center hover:border-gray-400 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
              >
                <svg class="mx-auto h-8 w-8 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
                </svg>
                <span class="mt-2 block text-sm font-medium text-gray-900">
                  Connect Spotify
                </span>
              </button>
            {/if}
          </div>
        </div>
      </div>
    {:else if activeTab === 'connections'}
      <ServiceConnections />
    {:else if activeTab === 'dnp'}
      <DnpManager />
    {:else if activeTab === 'enforcement'}
      <EnforcementPlanning />
    {:else if activeTab === 'community'}
      <CommunityLists />
    {/if}
  </main>
</div>