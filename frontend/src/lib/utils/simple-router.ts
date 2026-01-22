import { writable, derived } from 'svelte/store';

// Route definitions
export type Route =
  | 'home'
  | 'dashboard'
  | 'settings'
  | 'profile'
  | 'blocklist'
  | 'library-scan'
  | 'offense-database'
  | 'connections'
  | 'community'
  | 'sync'
  | 'analytics'
  | 'revenue-impact'
  | 'graph'
  | 'oauth-callback'
  | 'oauth-error'
  | 'overview'
  | 'dnp'
  | 'enforcement'
  | 'artist-profile';

// Path mappings
const pathToRoute: Record<string, Route> = {
  '/': 'home',
  '/home': 'home',
  '/dashboard': 'dashboard',
  '/settings': 'settings',
  '/profile': 'profile',
  '/blocklist': 'blocklist',
  '/library-scan': 'library-scan',
  '/offense-database': 'offense-database',
  '/connections': 'connections',
  '/community': 'community',
  '/sync': 'sync',
  '/analytics': 'analytics',
  '/revenue-impact': 'revenue-impact',
  '/graph': 'graph',
  '/overview': 'overview',
  '/dnp': 'dnp',
  '/enforcement': 'enforcement',
};

const routeToPath: Record<Route, string> = {
  'home': '/',
  'dashboard': '/dashboard',
  'settings': '/settings',
  'profile': '/profile',
  'blocklist': '/blocklist',
  'library-scan': '/library-scan',
  'offense-database': '/offense-database',
  'connections': '/connections',
  'community': '/community',
  'sync': '/sync',
  'analytics': '/analytics',
  'revenue-impact': '/revenue-impact',
  'graph': '/graph',
  'oauth-callback': '/auth/callback',
  'oauth-error': '/auth/error',
  'overview': '/overview',
  'dnp': '/dnp',
  'enforcement': '/enforcement',
  'artist-profile': '/artist',
};

// Route metadata
const routeMeta: Record<Route, { title: string; description: string }> = {
  'home': { title: 'Home', description: 'Your music blocklist dashboard' },
  'dashboard': { title: 'Dashboard', description: 'Your music blocklist dashboard' },
  'settings': { title: 'Settings', description: 'Account and connection settings' },
  'profile': { title: 'Profile', description: 'Your profile settings' },
  'blocklist': { title: 'Blocklist', description: 'Manage your blocked artists' },
  'library-scan': { title: 'Scan Library', description: 'Scan your music library' },
  'offense-database': { title: 'Database', description: 'Browse offense database' },
  'connections': { title: 'Connections', description: 'Manage streaming connections' },
  'community': { title: 'Community', description: 'Community lists' },
  'sync': { title: 'Catalog Sync', description: 'Synchronize artist catalogs across platforms' },
  'analytics': { title: 'Analytics', description: 'View system metrics and trends' },
  'revenue-impact': { title: 'Revenue Impact', description: 'See where your streaming revenue goes' },
  'graph': { title: 'Graph Explorer', description: 'Explore artist collaboration networks' },
  'oauth-callback': { title: 'Connecting...', description: 'Processing authentication' },
  'oauth-error': { title: 'Connection Error', description: 'There was a problem connecting' },
  'overview': { title: 'Overview', description: 'Dashboard overview' },
  'dnp': { title: 'DNP List', description: 'Your Do Not Play list' },
  'enforcement': { title: 'Enforcement', description: 'Blocklist enforcement status' },
  'artist-profile': { title: 'Artist Profile', description: 'View artist details and evidence' },
};

// Router store
export const currentRoute = writable<Route>('home');

// Route params store (for dynamic routes like /artist/:id)
export const routeParams = writable<Record<string, string>>({});

// Derived store for route metadata
export const currentRouteMeta = derived(currentRoute, $route => routeMeta[$route]);

// Navigation function
export function navigateTo(route: Route, params?: Record<string, string>) {
  currentRoute.set(route);
  routeParams.set(params || {});

  if (typeof window !== 'undefined') {
    let path = routeToPath[route] || '/';

    // Handle dynamic routes
    if (route === 'artist-profile' && params?.id) {
      path = `/artist/${params.id}`;
    }

    const meta = routeMeta[route];
    window.history.pushState({ route, params }, meta.title, path);
    document.title = `${meta.title} - No Drake in the House`;
  }
}

// Navigate to artist profile
export function navigateToArtist(artistId: string) {
  navigateTo('artist-profile', { id: artistId });
}

// Parse route and extract params from path
function parseRoute(path: string): { route: Route; params: Record<string, string> } {
  if (path.startsWith('/auth/callback')) return { route: 'oauth-callback', params: {} };
  if (path.startsWith('/auth/error')) return { route: 'oauth-error', params: {} };

  // Handle dynamic artist profile route: /artist/:id
  const artistMatch = path.match(/^\/artist\/([^\/]+)/);
  if (artistMatch) {
    return { route: 'artist-profile', params: { id: artistMatch[1] } };
  }

  return { route: pathToRoute[path] || 'home', params: {} };
}

// Initialize router
export function initRouter() {
  if (typeof window !== 'undefined') {
    window.addEventListener('popstate', (event) => {
      if (event.state?.route) {
        currentRoute.set(event.state.route);
        routeParams.set(event.state.params || {});
      } else {
        const { route, params } = parseRoute(window.location.pathname);
        currentRoute.set(route);
        routeParams.set(params);
      }
    });

    const { route: initialRoute, params: initialParams } = parseRoute(window.location.pathname);
    currentRoute.set(initialRoute);
    routeParams.set(initialParams);
    document.title = `${routeMeta[initialRoute].title} - No Drake in the House`;
  }
}

export function getPath(route: Route): string {
  return routeToPath[route] || '/';
}
