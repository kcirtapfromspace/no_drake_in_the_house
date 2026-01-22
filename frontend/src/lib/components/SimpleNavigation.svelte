<script lang="ts">
  import { currentRoute, navigateTo, type Route } from '../utils/simple-router';
  import { authActions, currentUser } from '../stores/auth';

  let userMenuOpen = false;
  let mobileMenuOpen = false;

  // Navigation items
  const navItems: { route: Route; label: string; icon: string }[] = [
    { route: 'dashboard', label: 'Dashboard', icon: 'home' },
    { route: 'blocklist', label: 'Blocklist', icon: 'block' },
    { route: 'library-scan', label: 'Scan Library', icon: 'search' },
    { route: 'offense-database', label: 'Database', icon: 'document' },
    { route: 'connections', label: 'Connections', icon: 'link' },
    { route: 'community', label: 'Community', icon: 'users' },
  ];

  function handleNavigation(route: Route) {
    navigateTo(route);
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

  // Close menus when clicking outside
  function handleClickOutside(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (!target.closest('.user-menu-container')) {
      userMenuOpen = false;
    }
  }
</script>

<svelte:window on:click={handleClickOutside} />

<nav class="shadow-sm sticky top-0 z-50" style="background: #27272a; border-bottom: 1px solid #52525b;">
  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
    <div class="flex justify-between h-16">
      <!-- Logo and main navigation -->
      <div class="flex">
        <!-- Logo -->
        <button
          type="button"
          on:click={() => handleNavigation('dashboard')}
          class="flex-shrink-0 flex items-center cursor-pointer hover:opacity-80 transition-opacity"
        >
          <span class="text-2xl mr-2" role="img" aria-label="No Drake in the House">🚫🦆</span>
          <h1 class="text-lg font-bold text-white hidden sm:block">No Drake in the House</h1>
        </button>

        <!-- Desktop navigation -->
        <div class="hidden md:ml-8 md:flex md:space-x-1">
          {#each navItems as item}
            <button
              type="button"
              on:click={() => handleNavigation(item.route)}
              class="inline-flex items-center px-3 py-2 rounded-md text-sm font-medium transition-all duration-200 {
                $currentRoute === item.route
                  ? 'bg-indigo-900 text-indigo-300'
                  : 'text-zinc-300 hover:text-white hover:bg-zinc-700'
              }"
            >
              {#if item.icon === 'home'}
                <svg class="w-4 h-4 mr-1.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
                </svg>
              {:else if item.icon === 'block'}
                <svg class="w-4 h-4 mr-1.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
                </svg>
              {:else if item.icon === 'link'}
                <svg class="w-4 h-4 mr-1.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
                </svg>
              {:else if item.icon === 'users'}
                <svg class="w-4 h-4 mr-1.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z" />
                </svg>
              {:else if item.icon === 'shield'}
                <svg class="w-4 h-4 mr-1.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
                </svg>
              {:else if item.icon === 'search'}
                <svg class="w-4 h-4 mr-1.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                </svg>
              {:else if item.icon === 'document'}
                <svg class="w-4 h-4 mr-1.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                </svg>
              {/if}
              {item.label}
            </button>
          {/each}
        </div>
      </div>

      <!-- User menu and mobile toggle -->
      <div class="flex items-center space-x-2">
        <!-- User dropdown (desktop) -->
        <div class="hidden md:block relative user-menu-container">
          <button
            type="button"
            on:click|stopPropagation={toggleUserMenu}
            class="flex items-center space-x-2 px-3 py-2 rounded-md text-sm font-medium text-zinc-300 hover:text-white hover:bg-zinc-700 transition-colors"
          >
            <div class="w-8 h-8 rounded-full bg-indigo-900 flex items-center justify-center">
              <span class="text-indigo-300 font-medium text-sm">
                {$currentUser?.email?.charAt(0).toUpperCase() || 'U'}
              </span>
            </div>
            <svg class="w-4 h-4 transition-transform {userMenuOpen ? 'rotate-180' : ''}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
            </svg>
          </button>

          {#if userMenuOpen}
            <div class="absolute right-0 mt-2 w-48 rounded-md shadow-lg py-1 z-50" style="background: #27272a; border: 1px solid #52525b;">
              <div class="px-4 py-2" style="border-bottom: 1px solid #52525b;">
                <p class="text-sm font-medium text-white truncate">{$currentUser?.email || 'User'}</p>
              </div>
              <button
                type="button"
                on:click={() => { handleNavigation('profile'); userMenuOpen = false; }}
                class="w-full text-left px-4 py-2 text-sm text-zinc-300 hover:bg-zinc-700 flex items-center"
              >
                <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                </svg>
                Profile
              </button>
              <button
                type="button"
                on:click={handleLogout}
                class="w-full text-left px-4 py-2 text-sm text-red-600 hover:bg-red-50 flex items-center"
              >
                <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
                </svg>
                Sign out
              </button>
            </div>
          {/if}
        </div>

        <!-- Mobile menu button -->
        <button
          type="button"
          on:click={toggleMobileMenu}
          class="md:hidden p-2 rounded-md text-zinc-400 hover:text-white hover:bg-zinc-700 focus:outline-none focus:ring-2 focus:ring-inset focus:ring-indigo-500"
          aria-expanded={mobileMenuOpen}
          aria-label="Toggle navigation menu"
        >
          {#if mobileMenuOpen}
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          {:else}
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
            </svg>
          {/if}
        </button>
      </div>
    </div>
  </div>

  <!-- Mobile navigation -->
  {#if mobileMenuOpen}
    <div class="md:hidden" style="border-top: 1px solid #52525b; background: #27272a;">
      <div class="px-2 pt-2 pb-3 space-y-1">
        {#each navItems as item}
          <button
            type="button"
            on:click={() => handleNavigation(item.route)}
            class="w-full flex items-center px-3 py-3 rounded-md text-base font-medium transition-colors {
              $currentRoute === item.route
                ? 'bg-indigo-900 text-indigo-300'
                : 'text-zinc-300 hover:text-white hover:bg-zinc-700'
            }"
          >
            {#if item.icon === 'home'}
              <svg class="w-5 h-5 mr-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
              </svg>
            {:else if item.icon === 'block'}
              <svg class="w-5 h-5 mr-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
              </svg>
            {:else if item.icon === 'link'}
              <svg class="w-5 h-5 mr-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
              </svg>
            {:else if item.icon === 'users'}
              <svg class="w-5 h-5 mr-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z" />
              </svg>
            {:else if item.icon === 'shield'}
              <svg class="w-5 h-5 mr-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
              </svg>
            {:else if item.icon === 'search'}
              <svg class="w-5 h-5 mr-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
              </svg>
            {:else if item.icon === 'document'}
              <svg class="w-5 h-5 mr-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
            {/if}
            {item.label}
          </button>
        {/each}
      </div>

      <!-- Mobile user section -->
      <div class="px-4 py-4" style="border-top: 1px solid #52525b;">
        <div class="flex items-center mb-3">
          <div class="w-10 h-10 rounded-full bg-indigo-900 flex items-center justify-center">
            <span class="text-indigo-300 font-medium">
              {$currentUser?.email?.charAt(0).toUpperCase() || 'U'}
            </span>
          </div>
          <div class="ml-3">
            <p class="text-sm font-medium text-white truncate">{$currentUser?.email || 'User'}</p>
          </div>
        </div>
        <div class="space-y-1">
          <button
            type="button"
            on:click={() => handleNavigation('profile')}
            class="w-full flex items-center px-3 py-2 rounded-md text-base font-medium text-zinc-300 hover:text-white hover:bg-zinc-700"
          >
            <svg class="w-5 h-5 mr-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
            </svg>
            Profile
          </button>
          <button
            type="button"
            on:click={handleLogout}
            class="w-full flex items-center px-3 py-2 rounded-md text-base font-medium text-red-600 hover:bg-red-50"
          >
            <svg class="w-5 h-5 mr-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
            </svg>
            Sign out
          </button>
        </div>
      </div>
    </div>
  {/if}
</nav>
