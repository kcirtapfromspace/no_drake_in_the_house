<script lang="ts">
  import { currentRoute, router } from '../utils/router';
  import { authActions, currentUser } from '../stores/auth';
  import { dnpCount } from '../stores/dnp';

  function handleLogout() {
    authActions.logout();
  }

  function navigate(route: string) {
    router.navigate(route);
  }
</script>

<!-- Top Navigation -->
<nav class="bg-white shadow-sm border-b border-gray-200">
  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
    <div class="flex justify-between h-16">
      <div class="flex items-center">
        <h1 class="text-xl font-semibold text-gray-900">
          No Drake in the House
        </h1>
      </div>
      
      <div class="flex items-center space-x-4">
        <span class="text-sm text-gray-700">
          {$currentUser?.email}
        </span>
        <button
          on:click={() => navigate('profile')}
          class="text-sm text-gray-500 hover:text-gray-700"
        >
          Settings
        </button>
        <button
          on:click={handleLogout}
          class="text-sm text-gray-500 hover:text-gray-700"
        >
          Sign out
        </button>
      </div>
    </div>
  </div>
</nav>

<!-- Tab Navigation -->
<div class="bg-white shadow-sm">
  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
    <nav class="flex space-x-8" aria-label="Tabs">
      <button
        on:click={() => navigate('overview')}
        class="py-4 px-1 border-b-2 font-medium text-sm {$currentRoute === 'overview' ? 'border-indigo-500 text-indigo-600' : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'}"
      >
        Overview
      </button>
      <button
        on:click={() => navigate('connections')}
        class="py-4 px-1 border-b-2 font-medium text-sm {$currentRoute === 'connections' ? 'border-indigo-500 text-indigo-600' : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'}"
      >
        Connections
      </button>
      <button
        on:click={() => navigate('dnp')}
        class="py-4 px-1 border-b-2 font-medium text-sm {$currentRoute === 'dnp' ? 'border-indigo-500 text-indigo-600' : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'}"
      >
        DNP List ({$dnpCount})
      </button>
      <button
        on:click={() => navigate('enforcement')}
        class="py-4 px-1 border-b-2 font-medium text-sm {$currentRoute === 'enforcement' ? 'border-indigo-500 text-indigo-600' : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'}"
      >
        Enforcement
      </button>
      <button
        on:click={() => navigate('community')}
        class="py-4 px-1 border-b-2 font-medium text-sm {$currentRoute === 'community' ? 'border-indigo-500 text-indigo-600' : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'}"
      >
        Community Lists
      </button>
    </nav>
  </div>
</div>