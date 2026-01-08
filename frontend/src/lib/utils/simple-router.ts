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
  | 'enforcement';

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
};

// Router store
export const currentRoute = writable<Route>('home');

// Derived store for route metadata
export const currentRouteMeta = derived(currentRoute, $route => routeMeta[$route]);

// Navigation function
export function navigateTo(route: Route) {
  currentRoute.set(route);

  if (typeof window !== 'undefined') {
    const path = routeToPath[route] || '/';
    const meta = routeMeta[route];
    window.history.pushState({ route }, meta.title, path);
    document.title = `${meta.title} - No Drake`;
  }
}

// Get route from path
function getRouteFromPath(path: string): Route {
  if (path.startsWith('/auth/callback')) return 'oauth-callback';
  if (path.startsWith('/auth/error')) return 'oauth-error';
  return pathToRoute[path] || 'home';
}

// Initialize router
export function initRouter() {
  if (typeof window !== 'undefined') {
    window.addEventListener('popstate', (event) => {
      const route = event.state?.route || getRouteFromPath(window.location.pathname);
      currentRoute.set(route);
    });

    const initialRoute = getRouteFromPath(window.location.pathname);
    currentRoute.set(initialRoute);
    document.title = `${routeMeta[initialRoute].title} - No Drake`;
  }
}

export function getPath(route: Route): string {
  return routeToPath[route] || '/';
}
