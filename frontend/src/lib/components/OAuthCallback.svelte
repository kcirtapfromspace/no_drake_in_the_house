<script lang="ts">
  import { onMount } from 'svelte';
  import { apiClient } from '../utils/api-client';
  import { navigateTo } from '../utils/simple-router';

  let status: 'loading' | 'success' | 'error' = 'loading';
  let errorMessage = '';
  let provider = '';

  onMount(async () => {
    // Parse URL parameters
    const params = new URLSearchParams(window.location.search);
    const code = params.get('code');
    const state = params.get('state');
    const error = params.get('error');
    const errorDescription = params.get('error_description');

    // Extract provider from path (e.g., /auth/callback/spotify)
    const pathParts = window.location.pathname.split('/');
    provider = pathParts[pathParts.length - 1] || 'unknown';

    // Handle OAuth errors
    if (error) {
      status = 'error';
      errorMessage = errorDescription || error || 'Authentication was cancelled or denied';
      return;
    }

    if (!code || !state) {
      status = 'error';
      errorMessage = 'Missing authentication parameters';
      return;
    }

    try {
      // Complete the OAuth link flow
      const result = await apiClient.post(`/api/v1/auth/oauth/${provider}/link-callback`, {
        code,
        state
      });

      if (result.success) {
        status = 'success';
        // Redirect to settings after a brief moment
        setTimeout(() => {
          navigateTo('settings');
        }, 1500);
      } else {
        status = 'error';
        errorMessage = result.message || 'Failed to link account';
      }
    } catch (e) {
      status = 'error';
      errorMessage = e instanceof Error ? e.message : 'An unexpected error occurred';
    }
  });

  function goToSettings() {
    navigateTo('settings');
  }

  function goHome() {
    navigateTo('home');
  }

  function getProviderName(p: string): string {
    switch (p) {
      case 'spotify': return 'Spotify';
      case 'apple': return 'Apple Music';
      case 'google': return 'Google';
      case 'github': return 'GitHub';
      default: return p;
    }
  }
</script>

<div class="min-h-screen bg-gray-900 text-white flex items-center justify-center p-4">
  <div class="bg-gray-800 rounded-xl border border-gray-600 p-8 max-w-md w-full text-center">
    {#if status === 'loading'}
      <div class="mb-6">
        <div class="w-16 h-16 border-4 border-rose-500 border-t-transparent rounded-full animate-spin mx-auto"></div>
      </div>
      <h1 class="text-xl font-bold mb-2">Connecting {getProviderName(provider)}</h1>
      <p class="text-gray-400">Please wait while we complete the connection...</p>
    {:else if status === 'success'}
      <div class="mb-6">
        <div class="w-16 h-16 bg-green-500 rounded-full flex items-center justify-center mx-auto">
          <svg class="w-10 h-10 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
        </div>
      </div>
      <h1 class="text-xl font-bold mb-2 text-green-400">Connected!</h1>
      <p class="text-gray-400 mb-6">Your {getProviderName(provider)} account has been linked successfully.</p>
      <button
        on:click={goToSettings}
        class="px-6 py-3 bg-rose-600 hover:bg-rose-700 rounded-lg font-medium transition-colors"
      >
        Go to Settings
      </button>
    {:else}
      <div class="mb-6">
        <div class="w-16 h-16 bg-red-500 rounded-full flex items-center justify-center mx-auto">
          <svg class="w-10 h-10 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </div>
      </div>
      <h1 class="text-xl font-bold mb-2 text-red-400">Connection Failed</h1>
      <p class="text-gray-400 mb-6">{errorMessage}</p>
      <div class="flex gap-3 justify-center">
        <button
          on:click={goToSettings}
          class="px-6 py-3 bg-gray-700 hover:bg-gray-600 rounded-lg font-medium transition-colors"
        >
          Try Again
        </button>
        <button
          on:click={goHome}
          class="px-6 py-3 bg-rose-600 hover:bg-rose-700 rounded-lg font-medium transition-colors"
        >
          Go Home
        </button>
      </div>
    {/if}
  </div>
</div>
