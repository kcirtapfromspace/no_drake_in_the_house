<script lang="ts">
  import { onMount } from 'svelte';
  import { connectionActions, spotifyConnection } from '../stores/connections';
  
  let isConnecting = false;
  let error = '';

  onMount(() => {
    // Handle OAuth callback if present
    const urlParams = new URLSearchParams(window.location.search);
    const code = urlParams.get('code');
    const state = urlParams.get('state');
    
    if (code && state) {
      handleSpotifyCallback(code, state);
      // Clean up URL
      window.history.replaceState({}, document.title, window.location.pathname);
    }
  });

  async function handleSpotifyCallback(code: string, state: string) {
    isConnecting = true;
    error = '';
    
    const result = await connectionActions.handleSpotifyCallback(code, state);
    
    if (!result.success) {
      error = result.message || 'Failed to connect Spotify';
    }
    
    isConnecting = false;
  }

  async function connectSpotify() {
    isConnecting = true;
    error = '';
    
    try {
      await connectionActions.initiateSpotifyAuth();
    } catch (err) {
      error = 'Failed to initiate Spotify connection';
      isConnecting = false;
    }
  }

  async function disconnectSpotify() {
    const result = await connectionActions.disconnectSpotify();
    
    if (!result.success) {
      error = result.message || 'Failed to disconnect Spotify';
    }
  }

  async function checkHealth() {
    await connectionActions.checkSpotifyHealth();
  }

  function getStatusColor(status: string) {
    switch (status) {
      case 'active': return 'text-green-600 bg-green-100';
      case 'expired': return 'text-yellow-600 bg-yellow-100';
      case 'error': return 'text-red-600 bg-red-100';
      default: return 'text-gray-600 bg-gray-100';
    }
  }

  function formatDate(dateString: string) {
    return new Date(dateString).toLocaleDateString();
  }
</script>

<div class="px-4 py-6 sm:px-0">
  <div class="mb-6">
    <h2 class="text-2xl font-bold text-gray-900">Service Connections</h2>
    <p class="mt-1 text-sm text-gray-600">
      Connect your streaming service accounts to manage your blocklist across platforms.
    </p>
  </div>

  {#if error}
    <div class="mb-6 bg-red-50 border border-red-200 rounded-md p-4">
      <div class="flex">
        <div class="flex-shrink-0">
          <svg class="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
          </svg>
        </div>
        <div class="ml-3">
          <p class="text-sm text-red-800">{error}</p>
        </div>
      </div>
    </div>
  {/if}

  <div class="bg-white shadow overflow-hidden sm:rounded-md">
    <ul class="divide-y divide-gray-200">
      <!-- Spotify Connection -->
      <li>
        <div class="px-4 py-4 flex items-center justify-between">
          <div class="flex items-center">
            <div class="flex-shrink-0 h-10 w-10">
              <div class="h-10 w-10 rounded-full bg-green-500 flex items-center justify-center">
                <svg class="h-6 w-6 text-white" fill="currentColor" viewBox="0 0 24 24">
                  <path d="M12 0C5.4 0 0 5.4 0 12s5.4 12 12 12 12-5.4 12-12S18.66 0 12 0zm5.521 17.34c-.24.359-.66.48-1.021.24-2.82-1.74-6.36-2.101-10.561-1.141-.418.122-.779-.179-.899-.539-.12-.421.18-.78.54-.9 4.56-1.021 8.52-.6 11.64 1.32.42.18.479.659.301 1.02zm1.44-3.3c-.301.42-.841.6-1.262.3-3.239-1.98-8.159-2.58-11.939-1.38-.479.12-1.02-.12-1.14-.6-.12-.48.12-1.021.6-1.141C9.6 9.9 15 10.561 18.72 12.84c.361.181.54.78.241 1.2zm.12-3.36C15.24 8.4 8.82 8.16 5.16 9.301c-.6.179-1.2-.181-1.38-.721-.18-.601.18-1.2.72-1.381 4.26-1.26 11.28-1.02 15.721 1.621.539.3.719 1.02.42 1.56-.299.421-1.02.599-1.559.3z"/>
                </svg>
              </div>
            </div>
            <div class="ml-4">
              <div class="flex items-center">
                <p class="text-sm font-medium text-gray-900">Spotify</p>
                {#if $spotifyConnection}
                  <span class="ml-2 inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {getStatusColor($spotifyConnection.status)}">
                    {$spotifyConnection.status}
                  </span>
                {/if}
              </div>
              <div class="mt-1">
                {#if $spotifyConnection}
                  <p class="text-sm text-gray-500">
                    Connected {formatDate($spotifyConnection.created_at)}
                    {#if $spotifyConnection.provider_user_id}
                      • User ID: {$spotifyConnection.provider_user_id}
                    {/if}
                  </p>
                  {#if $spotifyConnection.scopes.length > 0}
                    <p class="text-xs text-gray-400 mt-1">
                      Permissions: {$spotifyConnection.scopes.join(', ')}
                    </p>
                  {/if}
                {:else}
                  <p class="text-sm text-gray-500">
                    Connect your Spotify account to manage your music library
                  </p>
                {/if}
              </div>
            </div>
          </div>
          
          <div class="flex items-center space-x-2">
            {#if $spotifyConnection}
              <button
                on:click={checkHealth}
                class="inline-flex items-center px-3 py-2 border border-gray-300 shadow-sm text-sm leading-4 font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
              >
                Check Health
              </button>
              <button
                on:click={disconnectSpotify}
                class="inline-flex items-center px-3 py-2 border border-transparent text-sm leading-4 font-medium rounded-md text-red-700 bg-red-100 hover:bg-red-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500"
              >
                Disconnect
              </button>
            {:else}
              <button
                on:click={connectSpotify}
                disabled={isConnecting}
                class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {#if isConnecting}
                  <svg class="animate-spin -ml-1 mr-2 h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                  </svg>
                  Connecting...
                {:else}
                  Connect
                {/if}
              </button>
            {/if}
          </div>
        </div>
      </li>

      <!-- Apple Music (Coming Soon) -->
      <li>
        <div class="px-4 py-4 flex items-center justify-between opacity-50">
          <div class="flex items-center">
            <div class="flex-shrink-0 h-10 w-10">
              <div class="h-10 w-10 rounded-full bg-gray-400 flex items-center justify-center">
                <svg class="h-6 w-6 text-white" fill="currentColor" viewBox="0 0 24 24">
                  <path d="M23.997 6.124c0-.738-.065-1.47-.24-2.19-.317-1.31-1.062-2.31-2.18-3.043C21.003.517 20.373.285 19.7.164c-.517-.093-1.038-.135-1.564-.15-.04-.001-.08-.004-.12-.004H5.986c-.04 0-.08.003-.12.004-.526.015-1.047.057-1.564.15-.673.121-1.303.353-1.877.727C1.307 1.624.562 2.624.245 3.934.07 4.654.005 5.386.005 6.124v11.748c0 .738.065 1.47.24 2.19.317 1.31 1.062 2.31 2.18 3.043.574.374 1.204.606 1.877.727.517.093 1.038.135 1.564.15.04.001.08.004.12.004h12.014c.04 0 .08-.003.12-.004.526-.015 1.047-.057 1.564-.15.673-.121 1.303-.353 1.877-.727 1.118-.733 1.863-1.733 2.18-3.043.175-.72.24-1.452.24-2.19V6.124zM12.001 4.009c2.47 0 4.471 2.001 4.471 4.471s-2.001 4.471-4.471 4.471-4.471-2.001-4.471-4.471 2.001-4.471 4.471-4.471zm0 7.542c1.693 0 3.071-1.378 3.071-3.071s-1.378-3.071-3.071-3.071-3.071 1.378-3.071 3.071 1.378 3.071 3.071 3.071z"/>
                </svg>
              </div>
            </div>
            <div class="ml-4">
              <div class="flex items-center">
                <p class="text-sm font-medium text-gray-900">Apple Music</p>
                <span class="ml-2 inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium text-gray-600 bg-gray-100">
                  Coming Soon
                </span>
              </div>
              <p class="text-sm text-gray-500 mt-1">
                Apple Music integration will be available in a future update
              </p>
            </div>
          </div>
          
          <button
            disabled
            class="inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-400 bg-gray-100 cursor-not-allowed"
          >
            Coming Soon
          </button>
        </div>
      </li>

      <!-- YouTube Music (Coming Soon) -->
      <li>
        <div class="px-4 py-4 flex items-center justify-between opacity-50">
          <div class="flex items-center">
            <div class="flex-shrink-0 h-10 w-10">
              <div class="h-10 w-10 rounded-full bg-red-500 flex items-center justify-center">
                <svg class="h-6 w-6 text-white" fill="currentColor" viewBox="0 0 24 24">
                  <path d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z"/>
                </svg>
              </div>
            </div>
            <div class="ml-4">
              <div class="flex items-center">
                <p class="text-sm font-medium text-gray-900">YouTube Music</p>
                <span class="ml-2 inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium text-gray-600 bg-gray-100">
                  Coming Soon
                </span>
              </div>
              <p class="text-sm text-gray-500 mt-1">
                YouTube Music integration will be available in a future update
              </p>
            </div>
          </div>
          
          <button
            disabled
            class="inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-400 bg-gray-100 cursor-not-allowed"
          >
            Coming Soon
          </button>
        </div>
      </li>
    </ul>
  </div>

  <!-- Connection Info -->
  <div class="mt-6 bg-blue-50 border border-blue-200 rounded-md p-4">
    <div class="flex">
      <div class="flex-shrink-0">
        <svg class="h-5 w-5 text-blue-400" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd" />
        </svg>
      </div>
      <div class="ml-3">
        <h3 class="text-sm font-medium text-blue-800">
          About Service Connections
        </h3>
        <div class="mt-2 text-sm text-blue-700">
          <p>
            Service connections allow you to apply your Do-Not-Play list across different streaming platforms. 
            Each connection is secured with OAuth 2.0 and only requests the minimum permissions needed to manage your blocklist.
          </p>
          <ul class="list-disc list-inside mt-2 space-y-1">
            <li>Spotify: Full library management and playlist modification</li>
            <li>Apple Music: Limited library access (coming soon)</li>
            <li>YouTube Music: Browser extension support only (coming soon)</li>
          </ul>
        </div>
      </div>
    </div>
  </div>
</div>