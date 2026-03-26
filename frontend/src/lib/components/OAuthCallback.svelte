<script lang="ts">
  import { onMount } from 'svelte';
  import { apiClient } from '../utils/api-client';
  import { navigateTo } from '../utils/simple-router';
  import {
    getProviderFromPath,
    getProviderName,
    isConnectionProvider,
    resolveOAuthCallback,
  } from '../utils/oauth-callback';

  let status: 'loading' | 'success' | 'error' = 'loading';
  let errorMessage = '';
  let provider = '';

  // Detect if we're running inside an OAuth popup (opened by window.open)
  const isPopup = !!window.opener && window.opener !== window;

  onMount(async () => {
    provider = getProviderFromPath(window.location.pathname);

    const result = await resolveOAuthCallback(
      window.location,
      (url, body) => apiClient.post(url, body),
      () => apiClient.getAuthToken()
    );

    provider = result.provider;

    if (result.status === 'success') {
      // If running in a popup, close it — the parent window polls popup.closed
      // and will refresh connections automatically.
      if (isPopup && isConnectionProvider(result.provider)) {
        window.close();
        return;
      }
      status = 'success';
      setTimeout(() => {
        navigateTo(isConnectionProvider(result.provider) ? 'sync' : 'settings');
      }, 1500);
    } else {
      status = 'error';
      errorMessage = result.errorMessage;
    }
  });

  function goToSettings() {
    navigateTo(isConnectionProvider(provider) ? 'sync' : 'settings');
  }

  function goHome() {
    navigateTo('home');
  }
</script>

<div class="min-h-screen text-white flex items-center justify-center p-4" style="background: #27272a;">
  <div class="rounded-xl p-8 max-w-md w-full text-center" style="background: #3f3f46; border: 1px solid #52525b;">
    {#if status === 'loading'}
      <div class="mb-6">
        <div class="w-16 h-16 border-4 border-rose-500 border-t-transparent rounded-full animate-spin mx-auto"></div>
      </div>
      <h1 class="text-xl font-bold mb-2">Connecting {getProviderName(provider)}</h1>
      <p class="text-zinc-400">Please wait while we complete the connection...</p>
    {:else if status === 'success'}
      <div class="mb-6">
        <div class="w-16 h-16 bg-green-500 rounded-full flex items-center justify-center mx-auto">
          <svg class="w-10 h-10 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
        </div>
      </div>
      <h1 class="text-xl font-bold mb-2 text-green-400">Connected!</h1>
      <p class="text-zinc-400 mb-6">Your {getProviderName(provider)} account has been linked successfully.</p>
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
      <p class="text-zinc-400 mb-6">{errorMessage}</p>
      <div class="flex gap-3 justify-center">
        <button
          on:click={goToSettings}
          class="px-6 py-3 rounded-lg font-medium transition-colors" style="background: #3f3f46; border: 1px solid #52525b;"
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
