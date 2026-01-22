<script lang="ts">
  import { currentRoute, navigateTo, type Route } from '../../utils/simple-router';

  // Route hierarchy and display names
  const routeConfig: Record<Route, { label: string; parent?: Route }> = {
    'home': { label: 'Home' },
    'overview': { label: 'Overview', parent: 'home' },
    'dashboard': { label: 'Dashboard', parent: 'home' },
    'connections': { label: 'Connections', parent: 'home' },
    'dnp': { label: 'DNP List', parent: 'home' },
    'enforcement': { label: 'Enforcement', parent: 'home' },
    'community': { label: 'Community Lists', parent: 'home' },
    'blocklist': { label: 'Blocklist', parent: 'home' },
    'library-scan': { label: 'Library Scan', parent: 'connections' },
    'offense-database': { label: 'Offense Database', parent: 'home' },
    'sync': { label: 'Catalog Sync', parent: 'home' },
    'analytics': { label: 'Analytics', parent: 'home' },
    'revenue-impact': { label: 'Revenue Impact', parent: 'analytics' },
    'graph': { label: 'Graph Explorer', parent: 'home' },
    'settings': { label: 'Settings', parent: 'home' },
    'profile': { label: 'Profile', parent: 'settings' },
    'artist-profile': { label: 'Artist Profile', parent: 'dnp' },
    'oauth-callback': { label: 'Connecting', parent: 'connections' },
    'oauth-error': { label: 'Connection Error', parent: 'connections' },
  };

  // Build breadcrumb trail from current route
  function buildBreadcrumbs(route: Route): { route: Route; label: string }[] {
    const crumbs: { route: Route; label: string }[] = [];
    let current: Route | undefined = route;

    while (current) {
      const config = routeConfig[current];
      if (config) {
        crumbs.unshift({ route: current, label: config.label });
        current = config.parent;
      } else {
        break;
      }
    }

    return crumbs;
  }

  $: breadcrumbs = buildBreadcrumbs($currentRoute);
</script>

{#if breadcrumbs.length > 1}
  <nav aria-label="Breadcrumb" class="mb-4">
    <ol class="flex items-center flex-wrap gap-1 text-sm">
      {#each breadcrumbs as crumb, index}
        <li class="flex items-center">
          {#if index > 0}
            <svg
              class="w-4 h-4 mx-1 text-zinc-500 flex-shrink-0"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
              aria-hidden="true"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M9 5l7 7-7 7"
              />
            </svg>
          {/if}

          {#if index === breadcrumbs.length - 1}
            <!-- Current page (not a link) -->
            <span
              class="text-zinc-300 font-medium"
              aria-current="page"
            >
              {crumb.label}
            </span>
          {:else}
            <!-- Link to previous page -->
            <button
              type="button"
              on:click|preventDefault={() => navigateTo(crumb.route)}
              class="text-zinc-400 hover:text-white transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-rose-500 focus-visible:ring-offset-2 focus-visible:ring-offset-zinc-900 rounded px-1 py-0.5"
            >
              {crumb.label}
            </button>
          {/if}
        </li>
      {/each}
    </ol>
  </nav>
{/if}
