<script lang="ts">
  import { currentRoute, navigateTo, type Route } from '../utils/simple-router';
  import { authActions, currentUser } from '../stores/auth';
  import { spotifyConnection, appleMusicConnection, connectionActions } from '../stores/connections';
  import { blockingStore } from '../stores/blocking';
  import { theme, resolvedTheme } from '../stores/theme';
  import BlockingToasts from './BlockingToasts.svelte';
  import { onMount, tick } from 'svelte';
  import { fly, fade } from 'svelte/transition';

  let userMenuOpen = false;
  let mobileMenuOpen = false;
  let isConnectingApple = false;

  function focusTrap(node: HTMLElement) {
    const FOCUSABLE = 'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';
    let previouslyFocused: HTMLElement | null = null;

    function handleKeydown(e: KeyboardEvent) {
      if (e.key !== 'Tab') return;
      const focusable = Array.from(node.querySelectorAll(FOCUSABLE)) as HTMLElement[];
      if (focusable.length === 0) return;
      const first = focusable[0];
      const last = focusable[focusable.length - 1];
      if (e.shiftKey && document.activeElement === first) {
        e.preventDefault();
        last.focus();
      } else if (!e.shiftKey && document.activeElement === last) {
        e.preventDefault();
        first.focus();
      }
    }

    previouslyFocused = document.activeElement as HTMLElement;
    node.addEventListener('keydown', handleKeydown);
    // Focus the first focusable element in the panel
    tick().then(() => {
      const first = node.querySelector(FOCUSABLE) as HTMLElement | null;
      if (first) first.focus();
    });

    return {
      destroy() {
        node.removeEventListener('keydown', handleKeydown);
        if (previouslyFocused) previouslyFocused.focus();
      }
    };
  }

  const prefersReducedMotion = typeof window !== 'undefined'
    ? window.matchMedia('(prefers-reduced-motion: reduce)').matches
    : false;

  const navItems: { route: Route; label: string }[] = [
    { route: 'home', label: 'Home' },
    { route: 'sync', label: 'Library' },
    { route: 'playlist-sanitizer', label: 'Playlists' },
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

  function handleThemeToggle() {
    theme.cycle();
  }

  onMount(() => {
    connectionActions.fetchConnections().catch(() => {});
    blockingStore.init();
  });

  function handleSpotifyClick() {
    if ($spotifyConnection?.status === 'active') {
      navigateTo('connections');
    } else {
      navigateTo('sync');
    }
  }

  async function handleAppleMusicClick() {
    if ($appleMusicConnection?.status === 'active') {
      navigateTo('connections');
    } else {
      isConnectingApple = true;
      await connectionActions.connectAppleMusic();
      isConnectingApple = false;
    }
  }

  function isActiveRoute(route: Route): boolean {
    return $currentRoute === route;
  }

  $: themeIcon = $theme === 'system' ? 'auto' : $resolvedTheme === 'dark' ? 'sun' : 'moon';
  $: themeLabel = $theme === 'system' ? 'Using system theme, switch to light' : $resolvedTheme === 'dark' ? 'Switch to light mode' : 'Switch to dark mode';
</script>

<svelte:window on:click={handleClickOutside} on:keydown={handleKeydown} />

<a href="#main-content" class="skip-link">Skip to main content</a>

<div class="layout">
  <!-- Navigation Bar -->
  <nav class="nav" aria-label="Main navigation">
    <div class="nav__inner">
      <!-- Logo -->
      <button
        type="button"
        on:click={() => handleNavigation('home')}
        class="nav__logo"
      >
        <div class="nav__logo-mark">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="10" />
            <line x1="4.93" y1="4.93" x2="19.07" y2="19.07" />
          </svg>
        </div>
        <div class="nav__logo-copy">
          <span class="nav__logo-kicker">No Drake</span>
          <span class="nav__logo-text">in the House</span>
        </div>
      </button>

      <!-- Desktop Nav Links -->
      <div class="nav__links">
        {#each navItems as item}
          <button
            type="button"
            on:click={() => handleNavigation(item.route)}
            class="nav__link"
            class:nav__link--active={isActiveRoute(item.route)}
            aria-current={isActiveRoute(item.route) ? 'page' : undefined}
          >
            {item.label}
          </button>
        {/each}
      </div>

      <!-- Right section -->
      <div class="nav__actions">
        <!-- Service dots -->
        <div class="nav__services">
          <button
            type="button"
            on:click={handleSpotifyClick}
            class="nav__service-dot"
            class:nav__service-dot--active={$spotifyConnection?.status === 'active'}
            title={$spotifyConnection?.status === 'active' ? 'Spotify connected' : 'Connect Spotify'}
            aria-label={$spotifyConnection?.status === 'active' ? 'Spotify connected, manage connection' : 'Connect Spotify'}
            style={$spotifyConnection?.status === 'active' ? '--dot-color: var(--color-spotify);' : ''}
          >
            <svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 0C5.4 0 0 5.4 0 12s5.4 12 12 12 12-5.4 12-12S18.66 0 12 0zm5.521 17.34c-.24.359-.66.48-1.021.24-2.82-1.74-6.36-2.101-10.561-1.141-.418.122-.779-.179-.899-.539-.12-.421.18-.78.54-.9 4.56-1.021 8.52-.6 11.64 1.32.42.18.479.659.301 1.02zm1.44-3.3c-.301.42-.841.6-1.262.3-3.239-1.98-8.159-2.58-11.939-1.38-.479.12-1.02-.12-1.14-.6-.12-.48.12-1.021.6-1.141C9.6 9.9 15 10.561 18.72 12.84c.361.181.54.78.241 1.2zm.12-3.36C15.24 8.4 8.82 8.16 5.16 9.301c-.6.179-1.2-.181-1.38-.721-.18-.601.18-1.2.72-1.381 4.26-1.26 11.28-1.02 15.721 1.621.539.3.719 1.02.42 1.56-.299.421-1.02.599-1.559.3z"/></svg>
          </button>
          <button
            type="button"
            on:click={handleAppleMusicClick}
            disabled={isConnectingApple}
            class="nav__service-dot"
            class:nav__service-dot--active={$appleMusicConnection?.status === 'active'}
            title={$appleMusicConnection?.status === 'active' ? 'Apple Music connected' : 'Connect Apple Music'}
            aria-label={$appleMusicConnection?.status === 'active' ? 'Apple Music connected, manage connection' : 'Connect Apple Music'}
            style={$appleMusicConnection?.status === 'active' ? '--dot-color: var(--color-apple);' : ''}
          >
            {#if isConnectingApple}
              <div class="nav__spinner"></div>
            {:else}
              <svg viewBox="0 0 24 24" fill="currentColor"><path d="M23.994 6.124a9.23 9.23 0 00-.24-2.19c-.317-1.31-1.062-2.31-2.18-3.043a5.022 5.022 0 00-1.877-.726 10.496 10.496 0 00-1.564-.15c-.04-.003-.083-.01-.124-.013H5.986c-.152.01-.303.017-.455.026-.747.043-1.49.123-2.193.4-1.336.53-2.3 1.452-2.865 2.78-.192.448-.292.925-.363 1.408-.056.392-.088.785-.1 1.18 0 .032-.007.062-.01.093v12.223c.01.14.017.283.027.424.05.815.154 1.624.497 2.373.65 1.42 1.738 2.353 3.234 2.801.42.127.856.187 1.293.228.555.053 1.11.06 1.667.06h11.03a12.5 12.5 0 001.57-.1c.822-.106 1.596-.35 2.295-.81a5.046 5.046 0 001.88-2.207c.186-.42.293-.87.37-1.324.113-.675.138-1.358.137-2.04-.002-3.8 0-7.595-.003-11.393zm-6.423 3.99v5.712c0 .417-.058.827-.244 1.206-.29.59-.76.962-1.388 1.14-.35.1-.706.157-1.07.173-.95.042-1.785-.476-2.144-1.32-.238-.56-.223-1.136-.017-1.7.303-.825.96-1.277 1.743-1.49.294-.08.595-.13.893-.18.323-.054.65-.1.973-.157.274-.048.47-.202.53-.486a.707.707 0 00.017-.146c.002-1.633.002-3.265.002-4.898v-.07l-.06-.01c-2.097.4-4.194.8-6.29 1.202-.014.002-.032.014-.037.026-.006.016-.003.037-.003.056v7.36c0 .418-.052.832-.227 1.218-.282.622-.76 1.02-1.416 1.207-.313.09-.634.138-.96.166-.906.08-1.732-.4-2.134-1.203-.268-.534-.278-1.1-.096-1.66.267-.817.864-1.304 1.64-1.55.376-.12.763-.185 1.148-.25.278-.047.558-.088.832-.145.317-.065.522-.25.58-.574a.504.504 0 00.007-.115v-8.41c0-.25.042-.493.15-.72.183-.385.486-.62.882-.728.17-.047.346-.073.522-.11 2.55-.526 5.1-1.05 7.65-1.573.093-.02.19-.03.285-.03.316.004.528.2.613.5.032.113.044.233.044.35v5.9z"/></svg>
            {/if}
          </button>
        </div>

        <!-- Theme toggle -->
        <button
          type="button"
          on:click={handleThemeToggle}
          class="nav__icon-btn"
          aria-label={themeLabel}
          title={themeLabel}
        >
          {#if themeIcon === 'sun'}
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="5"/><line x1="12" y1="1" x2="12" y2="3"/><line x1="12" y1="21" x2="12" y2="23"/><line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/><line x1="1" y1="12" x2="3" y2="12"/><line x1="21" y1="12" x2="23" y2="12"/><line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
            </svg>
          {:else if themeIcon === 'auto'}
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <rect x="2" y="3" width="20" height="14" rx="2" ry="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/>
            </svg>
          {:else}
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M21 12.79A9 9 0 1111.21 3 7 7 0 0021 12.79z"/>
            </svg>
          {/if}
        </button>

        <!-- Mobile Menu Button -->
        <button
          type="button"
          class="nav__icon-btn nav__hamburger"
          aria-expanded={mobileMenuOpen}
          aria-controls="mobile-menu"
          aria-label={mobileMenuOpen ? 'Close menu' : 'Open menu'}
          on:click={toggleMobileMenu}
        >
          {#if mobileMenuOpen}
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
          {:else}
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><line x1="3" y1="6" x2="21" y2="6"/><line x1="3" y1="12" x2="21" y2="12"/><line x1="3" y1="18" x2="21" y2="18"/></svg>
          {/if}
        </button>

        <!-- User Menu (desktop) -->
        <div class="user-menu-container nav__user">
          <button
            type="button"
            on:click|stopPropagation={toggleUserMenu}
            class="nav__avatar"
            aria-label="User menu"
          >
            {#if $currentUser?.email}
              {$currentUser.email.charAt(0).toUpperCase()}
            {:else}
              ?
            {/if}
          </button>

          {#if userMenuOpen}
            <div class="nav__dropdown" transition:fade={prefersReducedMotion ? { duration: 0 } : { duration: 120 }}>
              <div class="nav__dropdown-header">
                <p class="nav__dropdown-name">{$currentUser?.email || 'User'}</p>
                <p class="nav__dropdown-meta">Account</p>
              </div>
              <button
                type="button"
                on:click|stopPropagation={() => handleNavigation('settings')}
                class="nav__dropdown-item"
              >
                <svg class="nav__dropdown-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-2 2 2 2 0 01-2-2v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 01-2-2 2 2 0 012-2h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 012-2 2 2 0 012 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06a1.65 1.65 0 00-.33 1.82V9a1.65 1.65 0 001.51 1H21a2 2 0 012 2 2 2 0 01-2 2h-.09a1.65 1.65 0 00-1.51 1z"/></svg>
                Settings
              </button>
              {#if $currentUser?.roles?.includes('owner')}
                <button
                  type="button"
                  on:click|stopPropagation={() => handleNavigation('admin')}
                  class="nav__dropdown-item"
                >
                  <svg class="nav__dropdown-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 20V10"/><path d="M18 20V4"/><path d="M6 20v-4"/></svg>
                  Admin Dashboard
                </button>
              {/if}
              <button
                type="button"
                on:click|stopPropagation={handleLogout}
                class="nav__dropdown-item nav__dropdown-item--danger"
              >
                <svg class="nav__dropdown-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 21H5a2 2 0 01-2-2V5a2 2 0 012-2h4"/><polyline points="16 17 21 12 16 7"/><line x1="21" y1="12" x2="9" y2="12"/></svg>
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
    <div
      class="mobile-backdrop"
      on:click={closeMobileMenu}
      on:keydown={(e) => e.key === 'Enter' && closeMobileMenu()}
      role="button"
      tabindex="-1"
      aria-label="Close menu"
      transition:fade={prefersReducedMotion ? { duration: 0 } : { duration: 200 }}
    />

    <div
      id="mobile-menu"
      class="mobile-panel"
      use:focusTrap
      transition:fly={prefersReducedMotion ? { x: 0, duration: 0 } : { x: 288, duration: 300 }}
      role="dialog"
      aria-modal="true"
      aria-label="Mobile navigation menu"
    >
      <div class="mobile-panel__inner">
        <div class="mobile-panel__header">
          <span class="mobile-panel__title">Menu</span>
          <button
            type="button"
            class="nav__icon-btn"
            on:click={closeMobileMenu}
            aria-label="Close menu"
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
          </button>
        </div>

        <div class="mobile-panel__user">
          <p class="mobile-panel__user-label">Signed in as</p>
          <p class="mobile-panel__user-email">{$currentUser?.email}</p>
        </div>

        <nav class="mobile-panel__nav" aria-label="Mobile navigation">
          {#each navItems as item}
            <button
              type="button"
              on:click={() => handleNavigation(item.route)}
              class="mobile-panel__link"
              class:mobile-panel__link--active={isActiveRoute(item.route)}
              aria-current={isActiveRoute(item.route) ? 'page' : undefined}
            >
              {item.label}
            </button>
          {/each}
        </nav>

        <div class="mobile-panel__services">
          <p class="mobile-panel__section-label">Connections</p>
          <div class="mobile-panel__service-row">
            <button
              type="button"
              on:click={handleSpotifyClick}
              class="mobile-panel__service-btn"
              class:mobile-panel__service-btn--active={$spotifyConnection?.status === 'active'}
            >
              <svg class="mobile-panel__service-icon" viewBox="0 0 24 24" fill="currentColor"><path d="M12 0C5.4 0 0 5.4 0 12s5.4 12 12 12 12-5.4 12-12S18.66 0 12 0zm5.521 17.34c-.24.359-.66.48-1.021.24-2.82-1.74-6.36-2.101-10.561-1.141-.418.122-.779-.179-.899-.539-.12-.421.18-.78.54-.9 4.56-1.021 8.52-.6 11.64 1.32.42.18.479.659.301 1.02zm1.44-3.3c-.301.42-.841.6-1.262.3-3.239-1.98-8.159-2.58-11.939-1.38-.479.12-1.02-.12-1.14-.6-.12-.48.12-1.021.6-1.141C9.6 9.9 15 10.561 18.72 12.84c.361.181.54.78.241 1.2zm.12-3.36C15.24 8.4 8.82 8.16 5.16 9.301c-.6.179-1.2-.181-1.38-.721-.18-.601.18-1.2.72-1.381 4.26-1.26 11.28-1.02 15.721 1.621.539.3.719 1.02.42 1.56-.299.421-1.02.599-1.559.3z"/></svg>
              Spotify
            </button>
            <button
              type="button"
              on:click={handleAppleMusicClick}
              disabled={isConnectingApple}
              class="mobile-panel__service-btn"
              class:mobile-panel__service-btn--active={$appleMusicConnection?.status === 'active'}
            >
              <svg class="mobile-panel__service-icon" viewBox="0 0 24 24" fill="currentColor"><path d="M23.994 6.124a9.23 9.23 0 00-.24-2.19c-.317-1.31-1.062-2.31-2.18-3.043a5.022 5.022 0 00-1.877-.726 10.496 10.496 0 00-1.564-.15c-.04-.003-.083-.01-.124-.013H5.986c-.152.01-.303.017-.455.026-.747.043-1.49.123-2.193.4-1.336.53-2.3 1.452-2.865 2.78-.192.448-.292.925-.363 1.408-.056.392-.088.785-.1 1.18 0 .032-.007.062-.01.093v12.223c.01.14.017.283.027.424.05.815.154 1.624.497 2.373.65 1.42 1.738 2.353 3.234 2.801.42.127.856.187 1.293.228.555.053 1.11.06 1.667.06h11.03a12.5 12.5 0 001.57-.1c.822-.106 1.596-.35 2.295-.81a5.046 5.046 0 001.88-2.207c.186-.42.293-.87.37-1.324.113-.675.138-1.358.137-2.04-.002-3.8 0-7.595-.003-11.393zm-6.423 3.99v5.712c0 .417-.058.827-.244 1.206-.29.59-.76.962-1.388 1.14-.35.1-.706.157-1.07.173-.95.042-1.785-.476-2.144-1.32-.238-.56-.223-1.136-.017-1.7.303-.825.96-1.277 1.743-1.49.294-.08.595-.13.893-.18.323-.054.65-.1.973-.157.274-.048.47-.202.53-.486a.707.707 0 00.017-.146c.002-1.633.002-3.265.002-4.898v-.07l-.06-.01c-2.097.4-4.194.8-6.29 1.202-.014.002-.032.014-.037.026-.006.016-.003.037-.003.056v7.36c0 .418-.052.832-.227 1.218-.282.622-.76 1.02-1.416 1.207-.313.09-.634.138-.96.166-.906.08-1.732-.4-2.134-1.203-.268-.534-.278-1.1-.096-1.66.267-.817.864-1.304 1.64-1.55.376-.12.763-.185 1.148-.25.278-.047.558-.088.832-.145.317-.065.522-.25.58-.574a.504.504 0 00.007-.115v-8.41c0-.25.042-.493.15-.72.183-.385.486-.62.882-.728.17-.047.346-.073.522-.11 2.55-.526 5.1-1.05 7.65-1.573.093-.02.19-.03.285-.03.316.004.528.2.613.5.032.113.044.233.044.35v5.9z"/></svg>
              Apple
            </button>
          </div>
        </div>

        <div class="mobile-panel__footer">
          <button
            type="button"
            on:click={() => handleNavigation('settings')}
            class="mobile-panel__link"
          >
            Settings
          </button>
          <button
            type="button"
            on:click={handleLogout}
            class="mobile-panel__link mobile-panel__link--danger"
          >
            Sign out
          </button>
        </div>
      </div>
    </div>
  {/if}

  <main id="main-content" class="layout__main" tabindex="-1">
    <slot />
  </main>

  <BlockingToasts />
</div>

<style>
  .layout {
    position: relative;
    isolation: isolate;
    min-height: 100vh;
    color: var(--color-text-primary);
    background:
      radial-gradient(circle at top left, var(--color-bg-glow-primary), transparent 18rem),
      radial-gradient(circle at top right, var(--color-bg-glow-secondary), transparent 14rem),
      var(--color-bg-page);
  }

  .skip-link {
    position: fixed;
    top: 0.75rem;
    left: 1rem;
    z-index: calc(var(--z-sticky, 40) + 20);
    padding: 0.7rem 1rem;
    border-radius: 999px;
    background: var(--color-bg-skip-link);
    color: var(--color-text-primary);
    border: 1px solid var(--color-brand-glow-border);
    box-shadow: var(--shadow-xl);
    font-size: 0.875rem;
    font-weight: 600;
    letter-spacing: 0.01em;
    text-decoration: none;
    transform: translateY(-180%);
    opacity: 0;
    pointer-events: none;
    transition:
      transform var(--transition-fast),
      opacity var(--transition-fast),
      border-color var(--transition-fast);
  }

  .skip-link:focus-visible {
    transform: translateY(0);
    opacity: 1;
    pointer-events: auto;
    outline: none;
    border-color: var(--color-brand-glow-focus);
  }

  /* ===== NAVIGATION ===== */
  .nav {
    position: sticky;
    top: 0;
    z-index: var(--z-sticky, 40);
    background: var(--nav-bg);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    border-bottom: 1px solid var(--nav-border);
    overflow: visible;
  }

  .nav__inner {
    max-width: 72rem;
    margin: 0 auto;
    padding: 0 1.5rem;
    min-height: 4rem;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    overflow: visible;
  }

  .nav__logo {
    display: flex;
    align-items: center;
    gap: 0.625rem;
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
    text-decoration: none;
    flex-shrink: 0;
  }

  .nav__logo-mark {
    width: 2.25rem;
    height: 2.25rem;
    color: var(--color-brand-primary);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 999px;
    background: radial-gradient(circle at 30% 30%, var(--color-overlay-glass), transparent 45%), var(--color-brand-primary-muted);
    box-shadow: 0 0 0 1px var(--color-brand-glow-ring);
  }

  .nav__logo-mark svg {
    width: 100%;
    height: 100%;
    max-width: none;
    max-height: none;
  }

  .nav__logo-copy {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    line-height: 1;
  }

  .nav__logo-kicker {
    font-size: 0.625rem;
    font-weight: 700;
    color: var(--color-brand-accent);
    letter-spacing: 0.14em;
    text-transform: uppercase;
  }

  .nav__logo-text {
    font-size: 0.95rem;
    font-weight: 700;
    color: var(--color-text-primary);
    letter-spacing: -0.03em;
    margin-top: 0.15rem;
  }

  .nav__links {
    display: none;
    align-items: center;
    gap: 0.25rem;
  }

  @media (min-width: 768px) {
    .nav__links { display: flex; }
  }

  .nav__link {
    position: relative;
    padding: 0.375rem 0.75rem;
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--color-text-tertiary);
    background: none;
    border: none;
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: color var(--transition-fast), background-color var(--transition-fast);
    white-space: nowrap;
  }

  .nav__link:hover {
    color: var(--color-text-primary);
    background: var(--color-bg-interactive);
  }

  .nav__link--active {
    color: var(--color-text-primary);
    font-weight: 600;
    background: var(--color-bg-interactive);
  }

  .nav__link--active::after {
    content: '';
    position: absolute;
    bottom: -0.5rem;
    left: 50%;
    transform: translateX(-50%);
    width: 1rem;
    height: 2px;
    border-radius: 1px;
    background: var(--color-brand-primary);
  }

  .nav__actions {
    display: flex;
    align-items: center;
    gap: 0.375rem;
  }

  .nav__services {
    display: flex;
    align-items: center;
    gap: 0.125rem;
  }

  .nav__service-dot {
    width: 2rem;
    height: 2rem;
    border-radius: var(--radius-full);
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--color-text-muted);
    transition: all var(--transition-fast);
    padding: 0;
  }

  .nav__service-dot svg {
    width: 1rem;
    height: 1rem;
    max-width: none;
    max-height: none;
  }

  .nav__service-dot:hover {
    color: var(--color-text-secondary);
    background: var(--color-bg-interactive);
  }

  .nav__service-dot--active {
    color: var(--dot-color, var(--color-text-primary));
  }

  .nav__service-dot--active:hover {
    color: var(--dot-color, var(--color-text-primary));
    background: var(--color-bg-interactive);
  }

  .nav__spinner {
    width: 0.875rem;
    height: 0.875rem;
    border: 2px solid var(--color-border-default);
    border-top-color: var(--color-text-secondary);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  .nav__icon-btn {
    width: 2rem;
    height: 2rem;
    border-radius: var(--radius-md);
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--color-text-tertiary);
    transition: all var(--transition-fast);
    padding: 0;
  }

  .nav__icon-btn svg {
    width: 1.125rem;
    height: 1.125rem;
    max-width: none;
    max-height: none;
  }

  .nav__icon-btn:hover {
    color: var(--color-text-primary);
    background: var(--color-bg-interactive);
  }

  .nav__hamburger {
    display: flex;
  }

  @media (min-width: 768px) {
    .nav__hamburger { display: none; }
  }

  .nav__user {
    position: relative;
    display: none;
    isolation: isolate;
    z-index: var(--z-dropdown, 60);
  }

  @media (min-width: 768px) {
    .nav__user { display: block; }
  }

  .nav__avatar {
    width: 2rem;
    height: 2rem;
    border-radius: var(--radius-full);
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--color-brand-primary-muted);
    color: var(--color-brand-primary);
    font-size: var(--text-xs);
    font-weight: 600;
    border: none;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .nav__avatar:hover {
    background: var(--color-brand-primary);
    color: var(--color-text-on-brand);
  }

  .nav__dropdown {
    position: absolute;
    right: 0;
    top: calc(100% + 0.5rem);
    width: 14rem;
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-xl);
    box-shadow: var(--shadow-dropdown);
    overflow: hidden;
    z-index: var(--z-dropdown, 60);
  }

  .nav__dropdown-header {
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .nav__dropdown-name {
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--color-text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .nav__dropdown-meta {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    margin-top: 0.125rem;
  }

  .nav__dropdown-item {
    display: flex;
    align-items: center;
    gap: 0.625rem;
    width: 100%;
    padding: 0.625rem 1rem;
    font-size: var(--text-sm);
    color: var(--color-text-secondary);
    background: none;
    border: none;
    cursor: pointer;
    transition: all var(--transition-fast);
    text-align: left;
  }

  .nav__dropdown-item svg {
    width: 1rem;
    height: 1rem;
    flex-shrink: 0;
    max-width: none;
    max-height: none;
  }

  .nav__dropdown-icon {
    width: 1rem !important;
    height: 1rem !important;
    min-width: 1rem;
    min-height: 1rem;
    display: block;
    flex-shrink: 0;
  }

  .nav__dropdown-item:hover {
    background: var(--color-bg-interactive);
    color: var(--color-text-primary);
  }

  .nav__dropdown-item--danger {
    color: var(--color-error);
  }

  .nav__dropdown-item--danger:hover {
    background: var(--color-error-muted);
    color: var(--color-error);
  }

  /* ===== MOBILE MENU ===== */
  .mobile-backdrop {
    position: fixed;
    inset: 0;
    z-index: var(--z-modal-backdrop, 80);
    background: var(--color-overlay-light);
    backdrop-filter: blur(2px);
  }

  .mobile-panel {
    position: fixed;
    inset: 0 0 0 auto;
    z-index: var(--z-modal, 90);
    width: 18rem;
    background: var(--color-bg-elevated);
    border-left: 1px solid var(--color-border-default);
    box-shadow: var(--shadow-xl);
  }

  .mobile-panel__inner {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .mobile-panel__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .mobile-panel__title {
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .mobile-panel__user {
    padding: 0.875rem 1.25rem;
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .mobile-panel__user-label {
    font-size: 0.6875rem;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .mobile-panel__user-email {
    font-size: var(--text-sm);
    color: var(--color-text-secondary);
    margin-top: 0.125rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .mobile-panel__nav {
    flex: 1;
    overflow-y: auto;
    padding: 0.75rem 0.75rem;
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
  }

  .mobile-panel__link {
    display: flex;
    align-items: center;
    padding: 0.625rem 0.75rem;
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--color-text-secondary);
    background: none;
    border: none;
    border-radius: var(--radius-lg);
    cursor: pointer;
    transition: all var(--transition-fast);
    text-align: left;
    width: 100%;
  }

  .mobile-panel__link:hover {
    background: var(--color-bg-interactive);
    color: var(--color-text-primary);
  }

  .mobile-panel__link--active {
    background: var(--color-brand-primary-muted);
    color: var(--color-brand-primary);
  }

  .mobile-panel__link--danger {
    color: var(--color-error);
  }

  .mobile-panel__link--danger:hover {
    background: var(--color-error-muted);
  }

  .mobile-panel__services {
    padding: 0.875rem 1.25rem;
    border-top: 1px solid var(--color-border-subtle);
  }

  .mobile-panel__section-label {
    font-size: 0.6875rem;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    margin-bottom: 0.625rem;
  }

  .mobile-panel__service-row {
    display: flex;
    gap: 0.5rem;
  }

  .mobile-panel__service-btn {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    font-size: var(--text-sm);
    color: var(--color-text-muted);
    background: var(--color-bg-interactive);
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--radius-lg);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .mobile-panel__service-btn:hover {
    color: var(--color-text-secondary);
    border-color: var(--color-border-default);
  }

  .mobile-panel__service-btn--active {
    color: var(--color-success);
    border-color: var(--color-success-muted);
  }

  .mobile-panel__service-icon {
    width: 1.125rem;
    height: 1.125rem;
    flex-shrink: 0;
  }

  .mobile-panel__service-icon :global(svg) {
    max-width: none;
    max-height: none;
  }

  .mobile-panel__footer {
    padding: 0.75rem;
    border-top: 1px solid var(--color-border-subtle);
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
  }

  /* ===== MAIN CONTENT ===== */
  .layout__main {
    min-height: calc(100vh - 4rem);
    padding-bottom: 3rem;
  }

  @media (max-width: 640px) {
    .nav__inner {
      padding-inline: 1rem;
      min-height: 3.75rem;
    }

    .nav__logo-mark {
      width: 2rem;
      height: 2rem;
    }

    .nav__logo-kicker {
      font-size: 0.58rem;
      letter-spacing: 0.12em;
    }

    .nav__logo-text {
      font-size: 0.85rem;
    }
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
