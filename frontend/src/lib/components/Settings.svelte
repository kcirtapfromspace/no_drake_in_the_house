<script lang="ts">
  import { onMount } from 'svelte';
  import { currentUser, authActions } from '../stores/auth';
  import { navigateTo } from '../utils/simple-router';
  import { apiClient } from '../utils/api-client';

  // Triadic color palette
  const colors = {
    green: { primary: '#30AF22', light: '#59BA48' },
    blue: { primary: '#009BD7', light: '#00B4FF' },
    pink: { primary: '#FF2C6E', light: '#FF728F' }
  };

  interface ConnectedAccount {
    provider: string;
    provider_user_id: string;
    email?: string;
    display_name?: string;
    linked_at: string;
  }

  let isLoggingOut = false;
  let connectedAccounts: ConnectedAccount[] = [];
  let isLoadingConnections = true;
  let connectingProvider: string | null = null;
  let connectionError: string | null = null;

  let blockFeatured = true;
  let blockProducers = false;
  let notifications = true;

  onMount(async () => {
    await loadConnections();
  });

  async function loadConnections() {
    isLoadingConnections = true;
    try {
      const result = await apiClient.get<ConnectedAccount[]>('/api/v1/auth/oauth/accounts');
      if (result.success && result.data) {
        connectedAccounts = result.data;
      }
    } catch (e) {
      console.error('Failed to load connections:', e);
    } finally {
      isLoadingConnections = false;
    }
  }

  function isConnected(provider: string): boolean {
    return connectedAccounts.some(a => a.provider === provider);
  }

  async function initiateOAuth(provider: string) {
    connectingProvider = provider;
    connectionError = null;
    try {
      const result = await apiClient.post<{ auth_url: string }>(`/api/v1/auth/oauth/${provider}/link`);
      if (result.success && result.data?.auth_url) {
        window.location.href = result.data.auth_url;
      } else {
        connectionError = 'Failed to initiate connection';
      }
    } catch (e) {
      connectionError = 'Failed to connect. Please try again.';
    } finally {
      connectingProvider = null;
    }
  }

  async function disconnectService(provider: string) {
    connectingProvider = provider;
    try {
      const result = await apiClient.delete(`/api/v1/auth/oauth/${provider}/unlink`);
      if (result.success) {
        connectedAccounts = connectedAccounts.filter(a => a.provider !== provider);
      }
    } catch (e) {
      console.error('Disconnect failed:', e);
    } finally {
      connectingProvider = null;
    }
  }

  async function handleLogout() {
    isLoggingOut = true;
    try {
      await authActions.logout();
      window.location.href = '/';
    } catch (e) {
      isLoggingOut = false;
    }
  }
</script>

<div class="min-h-screen bg-white">
  <header class="sticky top-0 z-50 bg-white border-b border-gray-200">
    <div class="max-w-2xl mx-auto px-4 py-4 flex items-center gap-4">
      <button
        on:click={() => navigateTo('home')}
        class="p-2 rounded-lg text-gray-500 hover:text-gray-900 hover:bg-gray-100 transition-colors"
        style="--focus-color: {colors.blue.primary};"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
        </svg>
      </button>
      <h1 class="text-xl font-semibold text-gray-900">Settings</h1>
    </div>
  </header>

  <main class="max-w-2xl mx-auto px-4 py-6 space-y-6">
    <!-- Account -->
    <section class="bg-gray-50 border border-gray-200 rounded-xl overflow-hidden">
      <div class="px-5 py-4 border-b border-gray-200 bg-white">
        <h2 class="font-semibold text-gray-900">Account</h2>
      </div>
      <div class="p-5 bg-white">
        <p class="text-sm text-gray-500">Email</p>
        <p class="mt-1 font-medium text-gray-900">{$currentUser?.email || 'Not signed in'}</p>
      </div>
    </section>

    <!-- Music Services -->
    <section class="bg-gray-50 border border-gray-200 rounded-xl overflow-hidden">
      <div class="px-5 py-4 border-b border-gray-200 bg-white">
        <h2 class="font-semibold text-gray-900">Music Services</h2>
        <p class="text-sm text-gray-500 mt-1">Connect your streaming accounts to sync your blocklist</p>
      </div>

      {#if connectionError}
        <div class="mx-5 mt-4 p-4 rounded-lg" style="background-color: #FFE5ED; border: 1px solid {colors.pink.light};">
          <p class="text-sm" style="color: {colors.pink.primary};">{connectionError}</p>
        </div>
      {/if}

      <div class="bg-white divide-y divide-gray-100">
        <!-- Spotify -->
        <div class="p-5 flex items-center justify-between">
          <div class="flex items-center gap-4">
            <div class="w-12 h-12 bg-[#1DB954] rounded-full flex items-center justify-center">
              <svg class="w-7 h-7 text-white" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 0C5.4 0 0 5.4 0 12s5.4 12 12 12 12-5.4 12-12S18.66 0 12 0zm5.521 17.34c-.24.359-.66.48-1.021.24-2.82-1.74-6.36-2.101-10.561-1.141-.418.122-.779-.179-.899-.539-.12-.421.18-.78.54-.9 4.56-1.021 8.52-.6 11.64 1.32.42.18.479.659.301 1.02zm1.44-3.3c-.301.42-.841.6-1.262.3-3.239-1.98-8.159-2.58-11.939-1.38-.479.12-1.02-.12-1.14-.6-.12-.48.12-1.021.6-1.141C9.6 9.9 15 10.561 18.72 12.84c.361.181.54.78.241 1.2zm.12-3.36C15.24 8.4 8.82 8.16 5.16 9.301c-.6.179-1.2-.181-1.38-.721-.18-.601.18-1.2.72-1.381 4.26-1.26 11.28-1.02 15.721 1.621.539.3.719 1.02.419 1.56-.299.421-1.02.599-1.559.3z"/>
              </svg>
            </div>
            <div>
              <p class="font-medium text-gray-900">Spotify</p>
              {#if isLoadingConnections}
                <p class="text-sm text-gray-500">Loading...</p>
              {:else if isConnected('spotify')}
                <p class="text-sm font-medium" style="color: {colors.green.primary};">Connected</p>
              {:else}
                <p class="text-sm text-gray-500">Not connected</p>
              {/if}
            </div>
          </div>
          {#if isConnected('spotify')}
            <button
              on:click={() => disconnectService('spotify')}
              disabled={connectingProvider === 'spotify'}
              class="px-4 py-2 bg-gray-100 text-gray-700 rounded-lg text-sm font-medium hover:bg-gray-200 transition-colors disabled:opacity-50"
            >
              {connectingProvider === 'spotify' ? 'Disconnecting...' : 'Disconnect'}
            </button>
          {:else}
            <button
              on:click={() => initiateOAuth('spotify')}
              disabled={connectingProvider === 'spotify'}
              class="px-4 py-2 bg-[#1DB954] text-white rounded-lg text-sm font-medium hover:bg-[#1ed760] transition-colors disabled:opacity-50"
            >
              {connectingProvider === 'spotify' ? 'Connecting...' : 'Connect'}
            </button>
          {/if}
        </div>

        <!-- Apple Music -->
        <div class="p-5 flex items-center justify-between">
          <div class="flex items-center gap-4">
            <div class="w-12 h-12 bg-gradient-to-br from-[#FA2D48] to-[#A833B9] rounded-full flex items-center justify-center">
              <svg class="w-7 h-7 text-white" viewBox="0 0 24 24" fill="currentColor">
                <path d="M23.994 6.124a9.23 9.23 0 00-.24-2.19c-.317-1.31-1.062-2.31-2.18-3.043a5.022 5.022 0 00-1.877-.726 10.496 10.496 0 00-1.564-.15c-.04-.003-.083-.01-.124-.013H5.986c-.152.01-.303.017-.455.026-.747.043-1.49.123-2.193.4-1.336.53-2.3 1.452-2.865 2.78-.192.448-.292.925-.363 1.408-.056.392-.088.785-.1 1.18 0 .032-.007.062-.01.093v12.223c.01.14.017.283.027.424.05.815.154 1.624.497 2.373.65 1.42 1.738 2.353 3.234 2.801.42.127.856.187 1.293.228.555.053 1.11.06 1.667.06h11.03a12.5 12.5 0 001.57-.1c.822-.106 1.596-.35 2.295-.81a5.046 5.046 0 001.88-2.207c.186-.42.293-.87.37-1.324.113-.675.138-1.358.137-2.04-.002-3.8 0-7.595-.003-11.393zm-6.423 3.99v5.712c0 .417-.058.827-.244 1.206-.29.59-.76.962-1.388 1.14-.35.1-.706.157-1.07.173-.95.042-1.785-.476-2.144-1.32-.238-.56-.223-1.136-.017-1.7.303-.825.96-1.277 1.743-1.49.294-.08.595-.13.893-.18.323-.054.65-.1.973-.157.274-.048.47-.202.53-.486a.707.707 0 00.017-.146c.002-1.633.002-3.265.002-4.898v-.07l-.06-.01c-2.097.4-4.194.8-6.29 1.202-.014.002-.032.014-.037.026-.006.016-.003.037-.003.056v7.36c0 .418-.052.832-.227 1.218-.282.622-.76 1.02-1.416 1.207-.313.09-.634.138-.96.166-.906.08-1.732-.4-2.134-1.203-.268-.534-.278-1.1-.096-1.66.267-.817.864-1.304 1.64-1.55.376-.12.763-.185 1.148-.25.278-.047.558-.088.832-.145.317-.065.522-.25.58-.574a.504.504 0 00.007-.115v-8.41c0-.25.042-.493.15-.72.183-.385.486-.62.882-.728.17-.047.346-.073.522-.11 2.55-.526 5.1-1.05 7.65-1.573.093-.02.19-.03.285-.03.316.004.528.2.613.5.032.113.044.233.044.35v5.9z"/>
              </svg>
            </div>
            <div>
              <p class="font-medium text-gray-900">Apple Music</p>
              {#if isLoadingConnections}
                <p class="text-sm text-gray-500">Loading...</p>
              {:else if isConnected('apple')}
                <p class="text-sm font-medium" style="color: {colors.green.primary};">Connected</p>
              {:else}
                <p class="text-sm text-gray-500">Not connected</p>
              {/if}
            </div>
          </div>
          {#if isConnected('apple')}
            <button
              on:click={() => disconnectService('apple')}
              disabled={connectingProvider === 'apple'}
              class="px-4 py-2 bg-gray-100 text-gray-700 rounded-lg text-sm font-medium hover:bg-gray-200 transition-colors disabled:opacity-50"
            >
              {connectingProvider === 'apple' ? 'Disconnecting...' : 'Disconnect'}
            </button>
          {:else}
            <button
              on:click={() => initiateOAuth('apple')}
              disabled={connectingProvider === 'apple'}
              class="px-4 py-2 bg-gradient-to-r from-[#FA2D48] to-[#A833B9] text-white rounded-lg text-sm font-medium hover:opacity-90 transition-opacity disabled:opacity-50"
            >
              {connectingProvider === 'apple' ? 'Connecting...' : 'Connect'}
            </button>
          {/if}
        </div>
      </div>
    </section>

    <!-- Preferences -->
    <section class="bg-gray-50 border border-gray-200 rounded-xl overflow-hidden">
      <div class="px-5 py-4 border-b border-gray-200 bg-white">
        <h2 class="font-semibold text-gray-900">Preferences</h2>
      </div>
      <div class="bg-white divide-y divide-gray-100">
        <div class="p-5 flex items-center justify-between">
          <div>
            <p class="font-medium text-gray-900">Block featured artists</p>
            <p class="text-sm text-gray-500 mt-0.5">Also block songs where artist is featured</p>
          </div>
          <button
            type="button"
            on:click={() => blockFeatured = !blockFeatured}
            class="relative w-11 h-6 rounded-full transition-colors"
            style="background-color: {blockFeatured ? colors.blue.primary : '#d1d5db'};"
          >
            <span
              class="absolute top-0.5 w-5 h-5 bg-white rounded-full shadow transition-transform"
              style="left: {blockFeatured ? '22px' : '2px'};"
            ></span>
          </button>
        </div>
        <div class="p-5 flex items-center justify-between">
          <div>
            <p class="font-medium text-gray-900">Block producer credits</p>
            <p class="text-sm text-gray-500 mt-0.5">Block songs produced by blocked artists</p>
          </div>
          <button
            type="button"
            on:click={() => blockProducers = !blockProducers}
            class="relative w-11 h-6 rounded-full transition-colors"
            style="background-color: {blockProducers ? colors.blue.primary : '#d1d5db'};"
          >
            <span
              class="absolute top-0.5 w-5 h-5 bg-white rounded-full shadow transition-transform"
              style="left: {blockProducers ? '22px' : '2px'};"
            ></span>
          </button>
        </div>
        <div class="p-5 flex items-center justify-between">
          <div>
            <p class="font-medium text-gray-900">News notifications</p>
            <p class="text-sm text-gray-500 mt-0.5">Get notified when new artists are added</p>
          </div>
          <button
            type="button"
            on:click={() => notifications = !notifications}
            class="relative w-11 h-6 rounded-full transition-colors"
            style="background-color: {notifications ? colors.blue.primary : '#d1d5db'};"
          >
            <span
              class="absolute top-0.5 w-5 h-5 bg-white rounded-full shadow transition-transform"
              style="left: {notifications ? '22px' : '2px'};"
            ></span>
          </button>
        </div>
      </div>
    </section>

    <!-- Account Actions -->
    <section class="bg-gray-50 border border-gray-200 rounded-xl overflow-hidden">
      <div class="px-5 py-4 border-b border-gray-200 bg-white">
        <h2 class="font-semibold text-gray-900">Account Actions</h2>
      </div>
      <div class="p-5 bg-white space-y-3">
        <button
          on:click={handleLogout}
          disabled={isLoggingOut}
          class="w-full px-4 py-3 bg-gray-100 text-gray-700 rounded-lg text-sm font-medium hover:bg-gray-200 transition-colors flex items-center justify-center gap-2 disabled:opacity-50"
        >
          {#if isLoggingOut}
            <div
              class="w-4 h-4 border-2 rounded-full animate-spin"
              style="border-color: {colors.blue.primary}; border-top-color: transparent;"
            ></div>
            <span>Signing out...</span>
          {:else}
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
            </svg>
            <span>Sign out</span>
          {/if}
        </button>
        <button
          class="w-full px-4 py-3 rounded-lg text-sm font-medium transition-colors"
          style="background-color: #FFE5ED; color: {colors.pink.primary}; border: 1px solid {colors.pink.light};"
        >
          Delete account
        </button>
      </div>
    </section>

    <p class="text-center text-sm text-gray-400">
      No Drake in the House v1.0.0
    </p>
  </main>
</div>
