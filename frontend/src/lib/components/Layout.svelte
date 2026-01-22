<script lang="ts">
  import { currentRoute, navigateTo, type Route } from '../utils/simple-router';
  import { authActions, currentUser } from '../stores/auth';
  import { spotifyConnection, appleMusicConnection, connectionActions } from '../stores/connections';
  import { blockingStore } from '../stores/blocking';
  import BlockingToasts from './BlockingToasts.svelte';
  import { onMount } from 'svelte';
  import { fly, fade } from 'svelte/transition';

  let userMenuOpen = false;
  let mobileMenuOpen = false;
  let isConnectingApple = false;

  // Check for reduced motion preference
  const prefersReducedMotion = typeof window !== 'undefined'
    ? window.matchMedia('(prefers-reduced-motion: reduce)').matches
    : false;

  const navItems: { route: Route; label: string }[] = [
    { route: 'home', label: 'Home' },
    { route: 'sync', label: 'Library' },
    { route: 'analytics', label: 'Analytics' },
    { route: 'graph', label: 'Network' },
  ];

  function handleNavigation(route: Route) {
    navigateTo(route);
    userMenuOpen = false;
    mobileMenuOpen = false;
  }

  async function handleLogout() {
    await authActions.logout();
    window.location.href = '/';
  }

  function toggleUserMenu() {
    userMenuOpen = !userMenuOpen;
  }

  function toggleMobileMenu() {
    mobileMenuOpen = !mobileMenuOpen;
  }

  function closeMobileMenu() {
    mobileMenuOpen = false;
  }

  function handleClickOutside(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (!target.closest('.user-menu-container')) {
      userMenuOpen = false;
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape' && mobileMenuOpen) {
      closeMobileMenu();
    }
  }

  // Fetch connections and init blocking store on mount
  onMount(() => {
    connectionActions.fetchConnections();
    blockingStore.init();
  });

  // Handle Spotify connection
  async function handleSpotifyClick() {
    if ($spotifyConnection?.status === 'active') {
      navigateTo('connections');
    } else {
      await connectionActions.initiateSpotifyAuth();
    }
  }

  // Handle Apple Music connection
  async function handleAppleMusicClick() {
    if ($appleMusicConnection?.status === 'active') {
      navigateTo('connections');
    } else {
      isConnectingApple = true;
      await connectionActions.connectAppleMusic();
      isConnectingApple = false;
    }
  }
</script>

<svelte:window on:click={handleClickOutside} on:keydown={handleKeydown} />

<!-- Skip link for keyboard navigation -->
<a
  href="#main-content"
  class="skip-link sr-only focus:not-sr-only focus:absolute focus:top-0 focus:left-0 focus:z-[100] focus:px-4 focus:py-2 focus:bg-rose-500 focus:text-white focus:font-medium"
>
  Skip to main content
</a>

<div class="min-h-screen bg-black">
  <!-- Navigation Bar -->
  <nav class="sticky top-0 z-50 bg-black border-b border-zinc-800">
    <div class="max-w-6xl mx-auto px-6 py-4">
      <div class="flex items-center justify-between">
        <!-- Logo -->
        <button
          type="button"
          on:click={() => handleNavigation('home')}
          class="flex items-center gap-3"
        >
          <div class="w-10 h-10 rounded-full flex items-center justify-center bg-rose-500">
            <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
            </svg>
          </div>
          <span class="text-xl font-bold text-white hidden sm:block">No Drake in the House</span>
        </button>

        <!-- Nav Links -->
        <div class="hidden md:flex items-center gap-1">
          {#each navItems as item}
            <button
              type="button"
              on:click={() => handleNavigation(item.route)}
              class="px-4 py-2 text-sm font-medium rounded-full transition-all {$currentRoute === item.route || ($currentRoute === 'revenue-impact' && item.route === 'analytics')
                ? 'bg-zinc-800 text-black'
                : 'text-zinc-300 hover:text-white'}"
            >
              {item.label}
            </button>
          {/each}
        </div>

        <!-- Service Connection Icons -->
        <div class="flex items-center gap-2 mr-2">
          <!-- Spotify -->
          <button
            type="button"
            on:click={handleSpotifyClick}
            class="w-8 h-8 rounded-full flex items-center justify-center transition-all {$spotifyConnection?.status === 'active' ? 'text-green-500 hover:bg-green-500/20' : 'text-zinc-600 hover:text-zinc-400 hover:bg-zinc-800'}"
            title={$spotifyConnection?.status === 'active' ? 'Spotify connected - Click to manage' : 'Connect Spotify'}
          >
            <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
              <path d="M12 0C5.4 0 0 5.4 0 12s5.4 12 12 12 12-5.4 12-12S18.66 0 12 0zm5.521 17.34c-.24.359-.66.48-1.021.24-2.82-1.74-6.36-2.101-10.561-1.141-.418.122-.779-.179-.899-.539-.12-.421.18-.78.54-.9 4.56-1.021 8.52-.6 11.64 1.32.42.18.479.659.301 1.02zm1.44-3.3c-.301.42-.841.6-1.262.3-3.239-1.98-8.159-2.58-11.939-1.38-.479.12-1.02-.12-1.14-.6-.12-.48.12-1.021.6-1.141C9.6 9.9 15 10.561 18.72 12.84c.361.181.54.78.241 1.2zm.12-3.36C15.24 8.4 8.82 8.16 5.16 9.301c-.6.179-1.2-.181-1.38-.721-.18-.601.18-1.2.72-1.381 4.26-1.26 11.28-1.02 15.721 1.621.539.3.719 1.02.42 1.56-.299.421-1.02.599-1.559.3z"/>
            </svg>
          </button>

          <!-- Apple Music -->
          <button
            type="button"
            on:click={handleAppleMusicClick}
            disabled={isConnectingApple}
            class="w-8 h-8 rounded-full flex items-center justify-center transition-all disabled:opacity-50 {$appleMusicConnection?.status === 'active' ? 'text-rose-500 hover:bg-rose-500/20' : 'text-zinc-600 hover:text-zinc-400 hover:bg-zinc-800'}"
            title={$appleMusicConnection?.status === 'active' ? 'Apple Music connected - Click to manage' : 'Connect Apple Music'}
          >
            {#if isConnectingApple}
              <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
            {:else}
              <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                <path d="M23.997 6.124c0-.738-.065-1.47-.24-2.19-.317-1.31-1.062-2.31-2.18-3.043C21.003.517 20.373.285 19.7.164c-.517-.093-1.038-.135-1.564-.15-.04-.001-.08-.004-.12-.004H5.986c-.04 0-.08.003-.12.004-.526.015-1.047.057-1.564.15-.673.121-1.303.353-1.877.727C1.307 1.624.562 2.624.245 3.934.07 4.654.005 5.386.005 6.124v11.748c0 .738.065 1.47.24 2.19.317 1.31 1.062 2.31 2.18 3.043.574.374 1.204.606 1.877.727.517.093 1.038.135 1.564.15.04.001.08.004.12.004h12.014c.04 0 .08-.003.12-.004.526-.015 1.047-.057 1.564-.15.673-.121 1.303-.353 1.877-.727 1.118-.733 1.863-1.733 2.18-3.043.175-.72.24-1.452.24-2.19V6.124zM12.001 4.009c2.47 0 4.471 2.001 4.471 4.471s-2.001 4.471-4.471 4.471-4.471-2.001-4.471-4.471 2.001-4.471 4.471-4.471zm0 7.542c1.693 0 3.071-1.378 3.071-3.071s-1.378-3.071-3.071-3.071-3.071 1.378-3.071 3.071 1.378 3.071 3.071 3.071z"/>
              </svg>
            {/if}
          </button>
        </div>

        <!-- Mobile Menu Button -->
        <button
          type="button"
          class="md:hidden w-9 h-9 rounded-full flex items-center justify-center text-zinc-300 hover:text-white hover:bg-zinc-800 transition-all"
          aria-expanded={mobileMenuOpen}
          aria-controls="mobile-menu"
          aria-label={mobileMenuOpen ? 'Close menu' : 'Open menu'}
          on:click={toggleMobileMenu}
        >
          {#if mobileMenuOpen}
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          {:else}
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
            </svg>
          {/if}
        </button>

        <!-- User Menu (desktop) -->
        <div class="relative user-menu-container hidden md:block">
          <button
            type="button"
            on:click|stopPropagation={toggleUserMenu}
            class="w-9 h-9 rounded-full flex items-center justify-center text-zinc-300 hover:text-white hover:bg-zinc-800 transition-all"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
            </svg>
          </button>

          {#if userMenuOpen}
            <div class="absolute right-0 mt-2 w-56 rounded-lg shadow-2xl py-2 z-50 bg-zinc-900 border border-zinc-800">
              <div class="px-4 py-3 border-b border-zinc-800">
                <p class="text-sm font-medium text-white truncate">{$currentUser?.email || 'User'}</p>
                <p class="text-xs text-zinc-400">Account</p>
              </div>
              <button
                type="button"
                on:click={() => handleNavigation('settings')}
                class="w-full text-left px-4 py-3 text-sm text-white hover:bg-zinc-800 flex items-center gap-3 transition-colors"
              >
                <svg class="w-5 h-5 text-zinc-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                </svg>
                Settings
              </button>
              <button
                type="button"
                on:click={handleLogout}
                class="w-full text-left px-4 py-3 text-sm text-rose-400 hover:bg-rose-500/10 flex items-center gap-3 transition-colors"
              >
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
                </svg>
                Sign out
              </button>
            </div>
          {/if}
        </div>
      </div>
    </div>
  </nav>

  <!-- Mobile Menu Overlay -->
  {#if mobileMenuOpen}
    <!-- Backdrop -->
    <div
      class="fixed inset-0 z-40 bg-black/60 md:hidden"
      on:click={closeMobileMenu}
      on:keydown={(e) => e.key === 'Enter' && closeMobileMenu()}
      role="button"
      tabindex="-1"
      aria-label="Close menu"
      transition:fade={prefersReducedMotion ? { duration: 0 } : { duration: 200 }}
    />

    <!-- Mobile menu panel -->
    <div
      id="mobile-menu"
      class="fixed inset-y-0 right-0 z-50 w-72 bg-zinc-900 shadow-xl md:hidden"
      transition:fly={prefersReducedMotion ? { x: 0, duration: 0 } : { x: 288, duration: 300 }}
      role="dialog"
      aria-modal="true"
      aria-label="Mobile navigation menu"
    >
      <div class="flex flex-col h-full">
        <!-- Header -->
        <div class="flex items-center justify-between p-4 border-b border-zinc-700">
          <span class="text-sm font-medium text-white">Menu</span>
          <button
            type="button"
            class="p-2 rounded-lg text-zinc-400 hover:text-white hover:bg-zinc-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-rose-500"
            on:click={closeMobileMenu}
            aria-label="Close menu"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <!-- User info -->
        <div class="p-4 border-b border-zinc-700">
          <p class="text-xs text-zinc-500 uppercase tracking-wider mb-1">Signed in as</p>
          <p class="text-sm text-zinc-300 truncate">{$currentUser?.email}</p>
        </div>

        <!-- Navigation links -->
        <nav class="flex-1 overflow-y-auto py-4" aria-label="Mobile navigation">
          <div class="space-y-1 px-2">
            {#each navItems as item}
              <button
                type="button"
                on:click={() => handleNavigation(item.route)}
                class="w-full flex items-center justify-between px-4 py-3 rounded-lg text-left transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-rose-500 {$currentRoute === item.route || ($currentRoute === 'revenue-impact' && item.route === 'analytics') ? 'bg-rose-500/10 text-rose-400' : 'text-zinc-300 hover:bg-zinc-800 hover:text-white'}"
                aria-current={$currentRoute === item.route ? 'page' : undefined}
              >
                <span class="font-medium">{item.label}</span>
              </button>
            {/each}
          </div>
        </nav>

        <!-- Service Connections in mobile menu -->
        <div class="p-4 border-t border-zinc-700">
          <p class="text-xs text-zinc-500 uppercase tracking-wider mb-3">Connections</p>
          <div class="flex gap-3">
            <button
              type="button"
              on:click={handleSpotifyClick}
              class="flex-1 flex items-center gap-2 px-3 py-2 rounded-lg transition-all {$spotifyConnection?.status === 'active' ? 'bg-green-500/10 text-green-500' : 'bg-zinc-800 text-zinc-400 hover:text-zinc-300'}"
            >
              <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                <path d="M12 0C5.4 0 0 5.4 0 12s5.4 12 12 12 12-5.4 12-12S18.66 0 12 0zm5.521 17.34c-.24.359-.66.48-1.021.24-2.82-1.74-6.36-2.101-10.561-1.141-.418.122-.779-.179-.899-.539-.12-.421.18-.78.54-.9 4.56-1.021 8.52-.6 11.64 1.32.42.18.479.659.301 1.02zm1.44-3.3c-.301.42-.841.6-1.262.3-3.239-1.98-8.159-2.58-11.939-1.38-.479.12-1.02-.12-1.14-.6-.12-.48.12-1.021.6-1.141C9.6 9.9 15 10.561 18.72 12.84c.361.181.54.78.241 1.2zm.12-3.36C15.24 8.4 8.82 8.16 5.16 9.301c-.6.179-1.2-.181-1.38-.721-.18-.601.18-1.2.72-1.381 4.26-1.26 11.28-1.02 15.721 1.621.539.3.719 1.02.42 1.56-.299.421-1.02.599-1.559.3z"/>
              </svg>
              <span class="text-sm">Spotify</span>
            </button>
            <button
              type="button"
              on:click={handleAppleMusicClick}
              disabled={isConnectingApple}
              class="flex-1 flex items-center gap-2 px-3 py-2 rounded-lg transition-all disabled:opacity-50 {$appleMusicConnection?.status === 'active' ? 'bg-rose-500/10 text-rose-500' : 'bg-zinc-800 text-zinc-400 hover:text-zinc-300'}"
            >
              <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                <path d="M23.997 6.124c0-.738-.065-1.47-.24-2.19-.317-1.31-1.062-2.31-2.18-3.043C21.003.517 20.373.285 19.7.164c-.517-.093-1.038-.135-1.564-.15-.04-.001-.08-.004-.12-.004H5.986c-.04 0-.08.003-.12.004-.526.015-1.047.057-1.564.15-.673.121-1.303.353-1.877.727C1.307 1.624.562 2.624.245 3.934.07 4.654.005 5.386.005 6.124v11.748c0 .738.065 1.47.24 2.19.317 1.31 1.062 2.31 2.18 3.043.574.374 1.204.606 1.877.727.517.093 1.038.135 1.564.15.04.001.08.004.12.004h12.014c.04 0 .08-.003.12-.004.526-.015 1.047-.057 1.564-.15.673-.121 1.303-.353 1.877-.727 1.118-.733 1.863-1.733 2.18-3.043.175-.72.24-1.452.24-2.19V6.124zM12.001 4.009c2.47 0 4.471 2.001 4.471 4.471s-2.001 4.471-4.471 4.471-4.471-2.001-4.471-4.471 2.001-4.471 4.471-4.471zm0 7.542c1.693 0 3.071-1.378 3.071-3.071s-1.378-3.071-3.071-3.071-3.071 1.378-3.071 3.071 1.378 3.071 3.071 3.071z"/>
              </svg>
              <span class="text-sm">Apple</span>
            </button>
          </div>
        </div>

        <!-- Footer actions -->
        <div class="p-4 border-t border-zinc-700 space-y-2">
          <button
            type="button"
            on:click={() => handleNavigation('settings')}
            class="w-full flex items-center gap-3 px-4 py-3 rounded-lg text-zinc-300 hover:bg-zinc-800 hover:text-white transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-rose-500"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
            <span>Settings</span>
          </button>
          <button
            type="button"
            on:click={handleLogout}
            class="w-full flex items-center gap-3 px-4 py-3 rounded-lg text-rose-400 hover:bg-rose-500/10 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-rose-500"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
            </svg>
            <span>Sign out</span>
          </button>
        </div>
      </div>
    </div>
  {/if}

  <main id="main-content" class="pb-8" tabindex="-1">
    <slot />
  </main>

  <!-- Blocking Progress Toasts -->
  <BlockingToasts />
</div>
