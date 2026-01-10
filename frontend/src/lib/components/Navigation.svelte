<script lang="ts">
  import { currentRoute, navigateTo, type Route } from '../utils/simple-router';
  import { authActions, currentUser } from '../stores/auth';
  import { dnpCount } from '../stores/dnp';

  function handleLogout() {
    authActions.logout();
  }

  function navigate(route: Route) {
    navigateTo(route);
  }
</script>

<!-- Top Navigation -->
<nav class="shadow-sm" style="background: #18181b; border-bottom: 1px solid #3f3f46;">
  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
    <div class="flex justify-between h-16">
      <div class="flex items-center">
        <div class="flex items-center gap-3">
          <div class="w-8 h-8 rounded-full flex items-center justify-center bg-rose-500">
            <svg class="w-4 h-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
            </svg>
          </div>
          <h1 class="text-xl font-semibold text-white">
            No Drake in the House
          </h1>
        </div>
      </div>

      <div class="flex items-center space-x-4">
        <span class="text-sm text-zinc-400">
          {$currentUser?.email}
        </span>
        <button
          type="button"
          on:click|preventDefault={() => navigate('profile')}
          class="text-sm text-zinc-300 hover:text-white transition-colors"
        >
          Settings
        </button>
        <button
          type="button"
          on:click|preventDefault={() => handleLogout()}
          class="text-sm text-zinc-300 hover:text-white transition-colors"
        >
          Sign out
        </button>
      </div>
    </div>
  </div>
</nav>

<!-- Tab Navigation -->
<div class="shadow-sm" style="background: #27272a;">
  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
    <nav class="flex space-x-8" aria-label="Tabs">
      <button
        type="button"
        on:click|preventDefault={() => navigate('overview')}
        class="py-4 px-1 border-b-2 font-medium text-sm transition-colors {$currentRoute === 'overview' ? 'border-rose-500 text-white' : 'border-transparent text-zinc-400 hover:text-zinc-200 hover:border-zinc-500'}"
      >
        Overview
      </button>
      <button
        type="button"
        on:click|preventDefault={() => navigate('connections')}
        class="py-4 px-1 border-b-2 font-medium text-sm transition-colors {$currentRoute === 'connections' ? 'border-rose-500 text-white' : 'border-transparent text-zinc-400 hover:text-zinc-200 hover:border-zinc-500'}"
      >
        Connections
      </button>
      <button
        type="button"
        on:click|preventDefault={() => navigate('dnp')}
        class="py-4 px-1 border-b-2 font-medium text-sm transition-colors {$currentRoute === 'dnp' ? 'border-rose-500 text-white' : 'border-transparent text-zinc-400 hover:text-zinc-200 hover:border-zinc-500'}"
      >
        DNP List ({$dnpCount})
      </button>
      <button
        type="button"
        on:click|preventDefault={() => navigate('enforcement')}
        class="py-4 px-1 border-b-2 font-medium text-sm transition-colors {$currentRoute === 'enforcement' ? 'border-rose-500 text-white' : 'border-transparent text-zinc-400 hover:text-zinc-200 hover:border-zinc-500'}"
      >
        Enforcement
      </button>
      <button
        type="button"
        on:click|preventDefault={() => navigate('community')}
        class="py-4 px-1 border-b-2 font-medium text-sm transition-colors {$currentRoute === 'community' ? 'border-rose-500 text-white' : 'border-transparent text-zinc-400 hover:text-zinc-200 hover:border-zinc-500'}"
      >
        Community Lists
      </button>
    </nav>
  </div>
</div>
