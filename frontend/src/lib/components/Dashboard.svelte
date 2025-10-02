<script lang="ts">
  import { onMount } from 'svelte';
  import { connectionActions, connectedServices, hasActiveSpotifyConnection } from '../stores/connections';
  import { dnpActions, dnpCount } from '../stores/dnp';
  import { currentRoute, navigateTo } from '../utils/simple-router';
  import { justRegistered, authActions } from '../stores/auth';
  import Navigation from './Navigation.svelte';
  import ServiceConnections from './ServiceConnections.svelte';
  import DnpManager from './DnpManager.svelte';
  import EnforcementPlanning from './EnforcementPlanning.svelte';
  import CommunityLists from './CommunityLists.svelte';
  import UserProfile from './UserProfile.svelte';
  import SimpleTest from './SimpleTest.svelte';
  
  let showWelcomeMessage = false;
  
  onMount(async () => {
    try {
      await connectionActions.fetchConnections();
    } catch (error) {
      console.log('Connection fetch failed (backend not running):', error);
    }
    
    try {
      await dnpActions.fetchDnpList();
    } catch (error) {
      console.log('DNP list fetch failed (backend not running):', error);
    }
    
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
    console.log('Dashboard: Navigating to:', tab);
    navigateTo(tab);
  }
  
  // Debug: Log route changes
  $: {
    console.log('Dashboard: Current route changed to:', $currentRoute);
  }
  
  function dismissWelcome() {
    showWelcomeMessage = false;
    authActions.clearJustRegistered();
  }
</script>



<div class="min-h-screen bg-uswds-base-lightest">
  <Navigation />

  <!-- Welcome Message for New Users -->
  {#if showWelcomeMessage}
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 pt-6">
      <div class="rounded-uswds-md bg-green-50 p-uswds-4 border border-green-200">
        <div class="flex">
          <div class="">
            <svg class="icon-uswds icon-uswds--lg icon-uswds--success" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
              <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
            </svg>
          </div>
          <div class="ml-3 flex-1">
            <h3 class="text-uswds-sm font-medium text-uswds-green-50">
              Welcome to No Drake in the House! 🎵
            </h3>
            <div class="mt-2 text-uswds-sm text-uswds-green-50">
              <p>Your account has been created successfully. Get started by connecting your music streaming services and building your first DNP list.</p>
            </div>
            <div class="mt-4">
              <div class="-mx-2 -my-1.5 flex">
                <button
                  type="button"
                  on:click|preventDefault={() => setActiveTab('connections')}
                  class="bg-green-50 px-2 py-1.5 rounded-uswds-md text-uswds-sm font-medium text-uswds-green-50 hover:bg-green-100 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-green-50 focus:ring-green-600"
                >
                  Connect Services
                </button>
                <button
                  type="button"
                  on:click|preventDefault={() => dismissWelcome()}
                  class="ml-3 bg-green-50 px-2 py-1.5 rounded-uswds-md text-uswds-sm font-medium text-uswds-green-50 hover:bg-green-100 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-green-50 focus:ring-green-600"
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
                on:click|preventDefault={() => dismissWelcome()}
                class="flex bg-green-50 rounded-uswds-md p-uswds-1.5 text-uswds-green-50 hover:bg-green-100 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-green-50 focus:ring-green-600"
              >
                <span class="sr-only">Dismiss</span>
                <svg class="icon-uswds icon-uswds--lg" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
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
    <!-- Debug Test Component -->
    <SimpleTest />
    
    {#if $currentRoute === 'overview'}
      <!-- Overview Tab -->
      <div class="px-4 py-6 sm:px-0">
        <div class="grid grid-cols-1 gap-uswds-6 sm:grid-cols-2 lg:grid-cols-3">
          <!-- Connected Services Card -->
          <div class="bg-white overflow-hidden shadow rounded-uswds-lg">
            <div class="p-uswds-5">
              <div class="flex items-center">
                <div class="flex-shrink-0">
                  <svg class="icon-uswds icon-uswds--lg icon-uswds--neutral" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
                  </svg>
                </div>
                <div class="ml-5 w-0 flex-1">
                  <dl>
                    <dt class="text-uswds-sm font-medium text-uswds-base-darker truncate">
                      Connected Services
                    </dt>
                    <dd class="text-uswds-lg font-medium text-uswds-base-darker">
                      {$connectedServices.length}
                    </dd>
                  </dl>
                </div>
              </div>
            </div>
            <div class="bg-uswds-base-lightest px-5 py-3">
              <div class="text-uswds-sm">
                <button
                  type="button"
                  on:click|preventDefault={() => setActiveTab('connections')}
                  class="font-medium text-indigo-700 hover:text-indigo-900"
                >
                  Manage connections
                </button>
              </div>
            </div>
          </div>

          <!-- DNP List Card -->
          <div class="bg-white overflow-hidden shadow rounded-uswds-lg">
            <div class="p-uswds-5">
              <div class="flex items-center">
                <div class="flex-shrink-0">
                  <svg class="icon-uswds icon-uswds--lg icon-uswds--neutral" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728L5.636 5.636m12.728 12.728L5.636 5.636" />
                  </svg>
                </div>
                <div class="ml-5 w-0 flex-1">
                  <dl>
                    <dt class="text-uswds-sm font-medium text-uswds-base-darker truncate">
                      Blocked Artists
                    </dt>
                    <dd class="text-uswds-lg font-medium text-uswds-base-darker">
                      {$dnpCount}
                    </dd>
                  </dl>
                </div>
              </div>
            </div>
            <div class="bg-uswds-base-lightest px-5 py-3">
              <div class="text-uswds-sm">
                <button
                  type="button"
                  on:click|preventDefault={() => setActiveTab('dnp')}
                  class="font-medium text-indigo-700 hover:text-indigo-900"
                >
                  Manage DNP list
                </button>
              </div>
            </div>
          </div>

          <!-- Status Card -->
          <div class="bg-white overflow-hidden shadow rounded-uswds-lg">
            <div class="p-uswds-5">
              <div class="flex items-center">
                <div class="flex-shrink-0">
                  <svg class="icon-uswds icon-uswds--lg {$hasActiveSpotifyConnection ? 'icon-uswds--success' : 'icon-uswds--neutral'}" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                </div>
                <div class="ml-5 w-0 flex-1">
                  <dl>
                    <dt class="text-uswds-sm font-medium text-uswds-base-darker truncate">
                      System Status
                    </dt>
                    <dd class="text-uswds-lg font-medium text-uswds-base-darker">
                      {$hasActiveSpotifyConnection ? 'Active' : 'Setup Required'}
                    </dd>
                  </dl>
                </div>
              </div>
            </div>
            <div class="bg-uswds-base-lightest px-5 py-3">
              <div class="text-uswds-sm">
                {#if !$hasActiveSpotifyConnection}
                  <button
                    type="button"
                    on:click|preventDefault={() => setActiveTab('connections')}
                    class="font-medium text-indigo-700 hover:text-indigo-900"
                  >
                    Connect Spotify
                  </button>
                {:else}
                  <span class="text-uswds-green-50">Ready to use</span>
                {/if}
              </div>
            </div>
          </div>
        </div>

        <!-- Quick Actions -->
        <div class="mt-8">
          <h3 class="text-uswds-lg leading-6 font-medium text-uswds-base-darker mb-4">
            Quick Actions
          </h3>
          <div class="grid grid-cols-1 gap-uswds-4 sm:grid-cols-2">
            <button
              type="button"
              on:click|preventDefault={() => setActiveTab('dnp')}
              class="relative block w-full border-2 border-gray-300 border-dashed rounded-uswds-lg p-uswds-6 text-center hover:border-gray-400 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
            >
              <svg class="icon-uswds icon-uswds--xl icon-uswds--neutral mx-auto" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
              </svg>
              <span class="mt-2 block text-uswds-sm font-medium text-uswds-base-darker">
                Add Artist to DNP List
              </span>
            </button>
            
            {#if $hasActiveSpotifyConnection && $dnpCount > 0}
              <button
                type="button"
                on:click|preventDefault={() => setActiveTab('enforcement')}
                class="relative block w-full border-2 border-gray-300 border-dashed rounded-uswds-lg p-uswds-6 text-center hover:border-gray-400 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
              >
                <svg class="icon-uswds icon-uswds--xl icon-uswds--neutral mx-auto" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.828 14.828a4 4 0 01-5.656 0M9 10h1.01M15 10h1.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                <span class="mt-2 block text-uswds-sm font-medium text-uswds-base-darker">
                  Plan Enforcement
                </span>
              </button>
            {:else}
              <button
                type="button"
                on:click|preventDefault={() => setActiveTab('connections')}
                class="relative block w-full border-2 border-gray-300 border-dashed rounded-uswds-lg p-uswds-6 text-center hover:border-gray-400 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
              >
                <svg class="icon-uswds icon-uswds--xl icon-uswds--neutral mx-auto" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
                </svg>
                <span class="mt-2 block text-uswds-sm font-medium text-uswds-base-darker">
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