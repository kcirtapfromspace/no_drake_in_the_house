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
  <nav aria-label="Breadcrumb" class="breadcrumb">
    <ol class="breadcrumb__list">
      {#each breadcrumbs as crumb, index}
        <li class="breadcrumb__item">
          {#if index > 0}
            <svg
              class="breadcrumb__separator"
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
            <span class="breadcrumb__current" aria-current="page">
              {crumb.label}
            </span>
          {:else}
            <button
              type="button"
              on:click|preventDefault={() => navigateTo(crumb.route)}
              class="breadcrumb__link"
            >
              {crumb.label}
            </button>
          {/if}
        </li>
      {/each}
    </ol>
  </nav>
{/if}

<style>
  .breadcrumb {
    margin-bottom: 1rem;
  }

  .breadcrumb__list {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.25rem;
    font-size: var(--text-sm);
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .breadcrumb__item {
    display: flex;
    align-items: center;
  }

  .breadcrumb__separator {
    width: 1rem;
    height: 1rem;
    margin: 0 0.25rem;
    color: var(--color-text-muted);
    flex-shrink: 0;
  }

  .breadcrumb__current {
    color: var(--color-text-secondary);
    font-weight: 500;
  }

  .breadcrumb__link {
    color: var(--color-text-muted);
    background: none;
    border: none;
    cursor: pointer;
    padding: 0.125rem 0.25rem;
    border-radius: 2px;
    font: inherit;
    font-size: inherit;
    transition: color var(--transition-fast);
  }

  .breadcrumb__link:hover {
    color: var(--color-text-primary);
  }

  .breadcrumb__link:focus-visible {
    outline: none;
    box-shadow: 0 0 0 2px var(--color-brand-primary), 0 0 0 4px var(--color-bg-primary);
    border-radius: var(--radius-sm);
  }
</style>
