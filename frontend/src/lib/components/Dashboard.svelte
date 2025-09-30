<script lang="ts">
  import { onMount } from 'svelte';
  import { connectionActions, connectedServices, hasActiveSpotifyConnection } from '../stores/connections';
  import { dnpActions, dnpCount } from '../stores/dnp';
  import { router, currentRoute } from '../utils/router';
  import { justRegistered, authActions } from '../stores/auth';
  import Navigation from './Navigation.svelte';
  import ServiceConnections from './ServiceConnections.svelte';
  import DnpManager from './DnpManager.svelte';
  import EnforcementPlanning from './EnforcementPlanning.svelte';
  import CommunityLists from './CommunityLists.svelte';
  import UserProfile from './UserProfile.svelte';
  
  let showWelcomeMessage = false;
  
  onMount(async () => {
    await connectionActions.fetchConnections();
    await dnpActions.fetchDnpList();
    
    // Show welcome message for newly registered users
    if ($justRegistered) {
      showWelcomeMessage = true;
      // Auto-hide welcome message after 5 seconds
      setTimeout(() => {
        showWelcomeMessage = false;
        authActions.clearJustRegistered();
      }, 5000);
    }
  });

  function setActiveTab(tab: string) {
    router.navigate(tab);
  }
  
  function dismissWelcome() {
    showWelcomeMessage = false;
    authActions.clearJustRegistered();
  }
</script>

<div class="min-h-screen bg-gray-50">
  <Navigation />

  <!-- Welcome Message for New Users -->
  {#if showWelcomeMessage}
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 pt-6">
      <div class="rounded-md bg-green-50 p-4 border border-green-200">
        <div class="flex">
          <div class="flex-shrink-0">
            <svg class="h-5 w-5 text-green-400" viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
            </svg>
          </div>
          <div class="ml-3 flex-1">
            <h3 class="text-sm font-medium text-green-800">
              Welcome to No Drake in the House! 🎵
            </h3>
            <div class="mt-2 text-sm text-green-700">
              <p>Your account has been created successfully. Get started by connecting your music streaming services and building your first DNP list.</p>
            </div>
            <div class="mt-4">
              <div class="-mx-2 -my-1.5 flex">
                <button
                  type="button"
                  on:click={() => setActiveTab('connections')}
                  class="bg-green-50 px-2 py-1.5 rounded-md text-sm font-medium text-green-800 hover:bg-green-100 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-green-50 focus:ring-green-600"
                >
                  Connect Services
                </button>
                <button
                  type="button"
                  on:click={dismissWelcome}
                  class="ml-3 bg-green-50 px-2 py-1.5 rounded-md text-sm font-medium text-green-800 hover:bg-green-100 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-green-50 focus:ring-green-600"
                >
                  Dismiss
                </button>
              </div>
            </div>
          </div>
          <div class="ml-auto pl-3">
            <div class="-mx-1.5 -my-1.5">
              <button
                type="button"
                on:click={dismissWelcome}
                class="inline-flex bg-green-50 rounded-md p-1.5 text-green-500 hover:bg-green-100 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-green-50 focus:ring-green-600"
              >
                <span class="sr-only">Dismiss</span>
                <svg class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                  <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
                </svg>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  {/if}

  <!-- Main Content -->
  <main class="max-w-7xl mx-auto py-6 sm:px-6 lg:px-8">
    {#if $currentRoute === 'overview'}
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
    {:else if $currentRoute === 'connections'}
      <ServiceConnections />
    {:else if $currentRoute === 'dnp'}
      <DnpManager />
    {:else if $currentRoute === 'enforcement'}
      <EnforcementPlanning />
    {:else if $currentRoute === 'community'}
      <CommunityLists />
    {:else if $currentRoute === 'profile'}
      <UserProfile />
    {/if}
  </main>
</div>