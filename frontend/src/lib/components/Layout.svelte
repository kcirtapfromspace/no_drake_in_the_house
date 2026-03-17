<script lang="ts">
  import { currentRoute, navigateTo, type Route } from '../utils/simple-router';
  import { authActions, currentUser } from '../stores/auth';
  import { spotifyConnection, appleMusicConnection, connectionActions } from '../stores/connections';
  import { blockingStore } from '../stores/blocking';
  import BlockingToasts from './BlockingToasts.svelte';
  import { onMount } from 'svelte';

  let userMenuOpen = false;
  let isConnectingApple = false;

  const navItems: { route: Route; label: string }[] = [
    { route: 'home', label: 'Home' },
    { route: 'sync', label: 'Library' },
    { route: 'analytics', label: 'Analytics' },
    { route: 'graph', label: 'Network' },
  ];

  function handleNavigation(route: Route) {
    navigateTo(route);
    userMenuOpen = false;
  }

  async function handleLogout() {
    await authActions.logout();
    window.location.href = '/';
  }

  function toggleUserMenu() {
    userMenuOpen = !userMenuOpen;
  }

  function handleClickOutside(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (!target.closest('.user-menu-container')) {
      userMenuOpen = false;
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

<svelte:window on:click={handleClickOutside} />

<div class="app-shell">
  <nav class="app-shell__nav">
    <div class="app-shell__nav-inner">
      <button
        type="button"
        on:click={() => handleNavigation('home')}
        class="app-shell__brand"
      >
        <div class="app-shell__brand-icon">
          <svg class="app-shell__brand-icon-svg" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
          </svg>
        </div>
        <div class="app-shell__brand-copy">
          <span class="app-shell__brand-kicker">No Drake</span>
          <span class="app-shell__brand-title">in the House</span>
        </div>
      </button>

      <div class="app-shell__nav-links" aria-label="Primary">
        {#each navItems as item}
          <button
            type="button"
            on:click={() => handleNavigation(item.route)}
            class="app-shell__nav-link"
            class:app-shell__nav-link--active={$currentRoute === item.route || ($currentRoute === 'revenue-impact' && item.route === 'analytics')}
          >
            {item.label}
          </button>
        {/each}
      </div>

      <div class="app-shell__controls">
        <div class="app-shell__services">
          <button
            type="button"
            on:click={handleSpotifyClick}
            class="app-shell__service"
            class:app-shell__service--connected={$spotifyConnection?.status === 'active'}
            class:app-shell__service--spotify={$spotifyConnection?.status === 'active'}
            title={$spotifyConnection?.status === 'active' ? 'Spotify connected - Click to manage' : 'Connect Spotify'}
          >
            <svg class="app-shell__service-icon" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path d="M12 0C5.4 0 0 5.4 0 12s5.4 12 12 12 12-5.4 12-12S18.66 0 12 0zm5.521 17.34c-.24.359-.66.48-1.021.24-2.82-1.74-6.36-2.101-10.561-1.141-.418.122-.779-.179-.899-.539-.12-.421.18-.78.54-.9 4.56-1.021 8.52-.6 11.64 1.32.42.18.479.659.301 1.02zm1.44-3.3c-.301.42-.841.6-1.262.3-3.239-1.98-8.159-2.58-11.939-1.38-.479.12-1.02-.12-1.14-.6-.12-.48.12-1.021.6-1.141C9.6 9.9 15 10.561 18.72 12.84c.361.181.54.78.241 1.2zm.12-3.36C15.24 8.4 8.82 8.16 5.16 9.301c-.6.179-1.2-.181-1.38-.721-.18-.601.18-1.2.72-1.381 4.26-1.26 11.28-1.02 15.721 1.621.539.3.719 1.02.42 1.56-.299.421-1.02.599-1.559.3z"/>
            </svg>
          </button>

          <button
            type="button"
            on:click={handleAppleMusicClick}
            disabled={isConnectingApple}
            class="app-shell__service"
            class:app-shell__service--connected={$appleMusicConnection?.status === 'active'}
            class:app-shell__service--apple={$appleMusicConnection?.status === 'active'}
            title={$appleMusicConnection?.status === 'active' ? 'Apple Music connected - Click to manage' : 'Connect Apple Music'}
          >
            {#if isConnectingApple}
              <svg class="app-shell__service-spinner" fill="none" viewBox="0 0 24 24" aria-hidden="true">
                <circle class="app-shell__service-spinner-track" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="app-shell__service-spinner-head" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
            {:else}
              <svg class="app-shell__service-icon" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                <path d="M23.997 6.124c0-.738-.065-1.47-.24-2.19-.317-1.31-1.062-2.31-2.18-3.043C21.003.517 20.373.285 19.7.164c-.517-.093-1.038-.135-1.564-.15-.04-.001-.08-.004-.12-.004H5.986c-.04 0-.08.003-.12.004-.526.015-1.047.057-1.564.15-.673.121-1.303.353-1.877.727C1.307 1.624.562 2.624.245 3.934.07 4.654.005 5.386.005 6.124v11.748c0 .738.065 1.47.24 2.19.317 1.31 1.062 2.31 2.18 3.043.574.374 1.204.606 1.877.727.517.093 1.038.135 1.564.15.04.001.08.004.12.004h12.014c.04 0 .08-.003.12-.004.526-.015 1.047-.057 1.564-.15.673-.121 1.303-.353 1.877-.727 1.118-.733 1.863-1.733 2.18-3.043.175-.72.24-1.452.24-2.19V6.124zM12.001 4.009c2.47 0 4.471 2.001 4.471 4.471s-2.001 4.471-4.471 4.471-4.471-2.001-4.471-4.471 2.001-4.471 4.471-4.471zm0 7.542c1.693 0 3.071-1.378 3.071-3.071s-1.378-3.071-3.071-3.071-3.071 1.378-3.071 3.071 1.378 3.071 3.071 3.071z"/>
              </svg>
            {/if}
          </button>
        </div>

        <div class="app-shell__user user-menu-container">
          <button
            type="button"
            on:click|stopPropagation={toggleUserMenu}
            class="app-shell__user-trigger"
            aria-label="Open account menu"
          >
            <svg class="app-shell__user-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
            </svg>
          </button>

          {#if userMenuOpen}
            <div class="app-shell__menu">
              <div class="app-shell__menu-header">
                <p class="app-shell__menu-email">{$currentUser?.email || 'User'}</p>
                <p class="app-shell__menu-label">Account</p>
              </div>
              <button
                type="button"
                on:click={() => handleNavigation('settings')}
                class="app-shell__menu-action"
              >
                <svg class="app-shell__menu-action-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                </svg>
                Settings
              </button>
              <button
                type="button"
                on:click={handleLogout}
                class="app-shell__menu-action app-shell__menu-action--danger"
              >
                <svg class="app-shell__menu-action-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
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

  <main class="app-shell__main">
    <slot />
  </main>

  <BlockingToasts />
</div>

<style>
  .app-shell {
    min-height: 100vh;
    color: #fafafa;
    background:
      radial-gradient(circle at top, rgba(244, 63, 94, 0.16), transparent 28%),
      radial-gradient(circle at bottom right, rgba(59, 130, 246, 0.12), transparent 26%),
      linear-gradient(180deg, #09090b 0%, #111113 48%, #050507 100%);
  }

  .app-shell__nav {
    position: sticky;
    top: 0;
    z-index: 50;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    background: rgba(9, 9, 11, 0.72);
    backdrop-filter: blur(20px);
  }

  .app-shell__nav-inner {
    width: min(1100px, calc(100vw - 2rem));
    margin: 0 auto;
    padding: 1rem 0;
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: center;
    gap: 1rem;
  }

  .app-shell__brand {
    display: inline-flex;
    align-items: center;
    gap: 0.875rem;
    border: 0;
    padding: 0;
    background: transparent;
    color: inherit;
    cursor: pointer;
  }

  .app-shell__brand-icon {
    display: grid;
    place-items: center;
    width: 2.75rem;
    height: 2.75rem;
    border-radius: 999px;
    background:
      radial-gradient(circle at 30% 30%, rgba(255,255,255,0.14), transparent 36%),
      linear-gradient(145deg, #f43f5e, #e11d48);
    box-shadow: 0 12px 24px rgba(244, 63, 94, 0.22);
  }

  .app-shell__brand-icon-svg {
    width: 1.35rem;
    height: 1.35rem;
    color: #fff;
  }

  .app-shell__brand-copy {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.15rem;
    line-height: 1;
  }

  .app-shell__brand-kicker {
    font-size: 0.68rem;
    font-weight: 700;
    letter-spacing: 0.18em;
    text-transform: uppercase;
    color: #fda4af;
  }

  .app-shell__brand-title {
    font-size: 1rem;
    font-weight: 800;
    color: #fafafa;
  }

  .app-shell__nav-links {
    display: none;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
  }

  .app-shell__nav-link {
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.03);
    color: #d4d4d8;
    padding: 0.72rem 1rem;
    font-size: 0.9rem;
    font-weight: 600;
    transition: border-color 160ms ease, background 160ms ease, color 160ms ease, transform 160ms ease;
    cursor: pointer;
  }

  .app-shell__nav-link:hover {
    border-color: rgba(255, 255, 255, 0.16);
    color: #fafafa;
    transform: translateY(-1px);
  }

  .app-shell__nav-link--active {
    border-color: rgba(244, 63, 94, 0.3);
    background: rgba(244, 63, 94, 0.12);
    color: #ffe4e6;
    box-shadow: inset 0 0 0 1px rgba(244, 63, 94, 0.08);
  }

  .app-shell__controls {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 0.75rem;
  }

  .app-shell__services {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.3rem;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.03);
  }

  .app-shell__service,
  .app-shell__user-trigger {
    width: 2.5rem;
    height: 2.5rem;
    display: grid;
    place-items: center;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.04);
    color: #a1a1aa;
    transition: border-color 160ms ease, background 160ms ease, color 160ms ease, transform 160ms ease;
    cursor: pointer;
  }

  .app-shell__service:hover,
  .app-shell__user-trigger:hover {
    border-color: rgba(255, 255, 255, 0.16);
    background: rgba(255, 255, 255, 0.07);
    color: #fafafa;
    transform: translateY(-1px);
  }

  .app-shell__service:disabled {
    opacity: 0.55;
    cursor: progress;
    transform: none;
  }

  .app-shell__service--connected {
    color: #fafafa;
  }

  .app-shell__service--spotify {
    border-color: rgba(34, 197, 94, 0.28);
    background: rgba(34, 197, 94, 0.12);
    color: #86efac;
  }

  .app-shell__service--apple {
    border-color: rgba(244, 63, 94, 0.28);
    background: rgba(244, 63, 94, 0.12);
    color: #fda4af;
  }

  .app-shell__service-icon,
  .app-shell__user-icon {
    width: 1.15rem;
    height: 1.15rem;
  }

  .app-shell__service-spinner {
    width: 1rem;
    height: 1rem;
    animation: app-shell-spin 0.9s linear infinite;
  }

  .app-shell__service-spinner-track {
    opacity: 0.25;
  }

  .app-shell__service-spinner-head {
    opacity: 0.9;
  }

  .app-shell__user {
    position: relative;
  }

  .app-shell__menu {
    position: absolute;
    top: calc(100% + 0.75rem);
    right: 0;
    width: min(16rem, calc(100vw - 2rem));
    overflow: hidden;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 1rem;
    background: rgba(9, 9, 11, 0.94);
    box-shadow: 0 24px 48px rgba(0, 0, 0, 0.42);
    backdrop-filter: blur(20px);
  }

  .app-shell__menu-header {
    padding: 1rem 1rem 0.9rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  }

  .app-shell__menu-email {
    margin: 0;
    font-size: 0.95rem;
    font-weight: 600;
    color: #fafafa;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .app-shell__menu-label {
    margin: 0.35rem 0 0;
    font-size: 0.75rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: #71717a;
  }

  .app-shell__menu-action {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    border: 0;
    background: transparent;
    color: #fafafa;
    padding: 0.9rem 1rem;
    font-size: 0.95rem;
    text-align: left;
    cursor: pointer;
    transition: background 160ms ease, color 160ms ease;
  }

  .app-shell__menu-action:hover {
    background: rgba(255, 255, 255, 0.05);
  }

  .app-shell__menu-action--danger {
    color: #fda4af;
  }

  .app-shell__menu-action--danger:hover {
    background: rgba(244, 63, 94, 0.12);
  }

  .app-shell__menu-action-icon {
    width: 1rem;
    height: 1rem;
    flex-shrink: 0;
  }

  .app-shell__main {
    padding-bottom: 2rem;
  }

  @keyframes app-shell-spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  @media (min-width: 960px) {
    .app-shell__nav-links {
      display: inline-flex;
    }
  }

  @media (max-width: 719px) {
    .app-shell__nav-inner {
      width: min(100vw - 1.25rem, 1100px);
      grid-template-columns: auto 1fr auto;
      gap: 0.75rem;
      padding: 0.85rem 0;
    }

    .app-shell__brand-copy {
      display: none;
    }

    .app-shell__services {
      gap: 0.35rem;
      padding: 0.2rem;
    }

    .app-shell__service,
    .app-shell__user-trigger {
      width: 2.2rem;
      height: 2.2rem;
    }
  }
</style>
