<script lang="ts">
  import { currentRoute, navigateTo, type Route } from '../utils/simple-router';
  import { authActions, currentUser } from '../stores/auth';
  import { dnpCount } from '../stores/dnp';
  import { fly, fade } from 'svelte/transition';

  let isMobileMenuOpen = false;

  // Check for reduced motion preference
  const prefersReducedMotion = typeof window !== 'undefined'
    ? window.matchMedia('(prefers-reduced-motion: reduce)').matches
    : false;

  function handleLogout() {
    authActions.logout();
    isMobileMenuOpen = false;
  }

  function navigate(route: Route) {
    navigateTo(route);
    isMobileMenuOpen = false;
  }

  function toggleMobileMenu() {
    isMobileMenuOpen = !isMobileMenuOpen;
  }

  function closeMobileMenu() {
    isMobileMenuOpen = false;
  }

  // Close menu on escape key
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape' && isMobileMenuOpen) {
      closeMobileMenu();
    }
  }

  // Navigation items for reuse
  const navItems: { route: Route; label: string; showCount?: boolean }[] = [
    { route: 'overview', label: 'Overview' },
    { route: 'connections', label: 'Connections' },
    { route: 'dnp', label: 'DNP List', showCount: true },
    { route: 'enforcement', label: 'Enforcement' },
    { route: 'community', label: 'Community Lists' },
  ];
</script>

<svelte:window on:keydown={handleKeydown} />

<!-- Top Navigation -->
<nav class="shadow-sm" style="background: #18181b; border-bottom: 1px solid #3f3f46;">
  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
    <div class="flex justify-between h-16">
      <!-- Logo and title -->
      <div class="flex items-center">
        <div class="flex items-center gap-3">
          <div class="w-8 h-8 rounded-full flex items-center justify-center bg-rose-500">
            <svg class="w-4 h-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
            </svg>
          </div>
          <h1 class="text-xl font-semibold text-white hidden sm:block">
            No Drake in the House
          </h1>
          <h1 class="text-lg font-semibold text-white sm:hidden">
            NDITH
          </h1>
        </div>
      </div>

      <!-- Desktop navigation -->
      <div class="hidden md:flex items-center space-x-4">
        <span class="text-sm text-zinc-300">
          {$currentUser?.email}
        </span>
        <button
          type="button"
          on:click|preventDefault={() => navigate('profile')}
          class="text-sm text-zinc-300 hover:text-white transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-rose-500 focus-visible:ring-offset-2 focus-visible:ring-offset-zinc-900 rounded px-2 py-1"
        >
          Settings
        </button>
        <button
          type="button"
          on:click|preventDefault={() => handleLogout()}
          class="text-sm text-zinc-300 hover:text-white transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-rose-500 focus-visible:ring-offset-2 focus-visible:ring-offset-zinc-900 rounded px-2 py-1"
        >
          Sign out
        </button>
      </div>

      <!-- Mobile menu button -->
      <div class="flex items-center md:hidden">
        <button
          type="button"
          class="inline-flex items-center justify-center p-2 rounded-lg text-zinc-400 hover:text-white hover:bg-zinc-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-rose-500"
          aria-expanded={isMobileMenuOpen}
          aria-controls="mobile-menu"
          aria-label={isMobileMenuOpen ? 'Close menu' : 'Open menu'}
          on:click={toggleMobileMenu}
        >
          {#if isMobileMenuOpen}
            <!-- Close icon -->
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          {:else}
            <!-- Hamburger icon -->
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
            </svg>
          {/if}
        </button>
      </div>
    </div>
  </div>
</nav>

<!-- Desktop Tab Navigation -->
<div class="hidden md:block shadow-sm" style="background: #27272a;">
  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
    <nav class="flex space-x-8" aria-label="Main navigation">
      {#each navItems as item}
        <button
          type="button"
          on:click|preventDefault={() => navigate(item.route)}
          class="py-4 px-1 border-b-2 font-medium text-sm transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-rose-500 focus-visible:ring-offset-2 focus-visible:ring-offset-zinc-800 {$currentRoute === item.route ? 'border-rose-500 text-white' : 'border-transparent text-zinc-300 hover:text-white hover:border-zinc-500'}"
          aria-current={$currentRoute === item.route ? 'page' : undefined}
        >
          {item.label}{#if item.showCount} ({$dnpCount}){/if}
        </button>
      {/each}
    </nav>
  </div>
</div>

<!-- Mobile Menu Overlay -->
{#if isMobileMenuOpen}
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
              on:click={() => navigate(item.route)}
              class="w-full flex items-center justify-between px-4 py-3 rounded-lg text-left transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-rose-500 {$currentRoute === item.route ? 'bg-rose-500/10 text-rose-400' : 'text-zinc-300 hover:bg-zinc-800 hover:text-white'}"
              aria-current={$currentRoute === item.route ? 'page' : undefined}
            >
              <span class="font-medium">{item.label}</span>
              {#if item.showCount}
                <span class="text-sm px-2 py-0.5 rounded-full {$currentRoute === item.route ? 'bg-rose-500/20 text-rose-400' : 'bg-zinc-700 text-zinc-400'}">
                  {$dnpCount}
                </span>
              {/if}
            </button>
          {/each}
        </div>
      </nav>

      <!-- Footer actions -->
      <div class="p-4 border-t border-zinc-700 space-y-2">
        <button
          type="button"
          on:click={() => navigate('profile')}
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
          class="w-full flex items-center gap-3 px-4 py-3 rounded-lg text-zinc-300 hover:bg-zinc-800 hover:text-white transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-rose-500"
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
