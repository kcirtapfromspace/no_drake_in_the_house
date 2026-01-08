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
<nav class="bg-white shadow-sm border-b border-gray-200">
  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
    <div class="flex justify-between h-16">
      <div class="flex items-center">
        <h1 class="text-uswds-xl font-semibold text-uswds-base-darker">
          No Drake in the House
        </h1>
      </div>
      
      <div class="flex items-center space-x-4">
        <span class="text-uswds-sm text-uswds-base-darker">
          {$currentUser?.email}
        </span>
        <button
          type="button"
          on:click|preventDefault={() => navigate('profile')}
          class="text-uswds-sm text-uswds-base-darker hover:text-gray-700"
        >
          Settings
        </button>
        <button
          type="button"
          on:click|preventDefault={() => handleLogout()}
          class="text-uswds-sm text-uswds-base-darker hover:text-gray-700"
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
        type="button"
        on:click|preventDefault={() => navigate('overview')}
        class="py-4 px-1 border-b-2 font-medium text-uswds-sm {$currentRoute === 'overview' ? 'border-indigo-500 text-primary' : 'border-transparent text-uswds-base-darker hover:text-gray-700 hover:border-gray-300'}"
      >
        Overview
      </button>
      <button
        type="button"
        on:click|preventDefault={() => navigate('connections')}
        class="py-4 px-1 border-b-2 font-medium text-uswds-sm {$currentRoute === 'connections' ? 'border-indigo-500 text-indigo-600' : 'border-transparent text-uswds-base-darker hover:text-gray-700 hover:border-gray-300'}"
      >
        Connections
      </button>
      <button
        type="button"
        on:click|preventDefault={() => navigate('dnp')}
        class="py-4 px-1 border-b-2 font-medium text-uswds-sm {$currentRoute === 'dnp' ? 'border-indigo-500 text-indigo-600' : 'border-transparent text-uswds-base-darker hover:text-gray-700 hover:border-gray-300'}"
      >
        DNP List ({$dnpCount})
      </button>
      <button
        type="button"
        on:click|preventDefault={() => navigate('enforcement')}
        class="py-4 px-1 border-b-2 font-medium text-uswds-sm {$currentRoute === 'enforcement' ? 'border-indigo-500 text-indigo-600' : 'border-transparent text-uswds-base-darker hover:text-gray-700 hover:border-gray-300'}"
      >
        Enforcement
      </button>
      <button
        type="button"
        on:click|preventDefault={() => navigate('community')}
        class="py-4 px-1 border-b-2 font-medium text-uswds-sm {$currentRoute === 'community' ? 'border-indigo-500 text-indigo-600' : 'border-transparent text-uswds-base-darker hover:text-gray-700 hover:border-gray-300'}"
      >
        Community Lists
      </button>
    </nav>
  </div>
</div>