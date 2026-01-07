import { writable, derived } from 'svelte/store';

// Simplified route definitions - just 3 main routes
export type Route =
  | 'home'
  | 'settings'
  | 'oauth-callback'
  | 'oauth-error';

// Path mappings
const pathToRoute: Record<string, Route> = {
  '/': 'home',
  '/home': 'home',
  '/settings': 'settings',
};

const routeToPath: Record<Route, string> = {
  'home': '/',
  'settings': '/settings',
  'oauth-callback': '/auth/callback',
  'oauth-error': '/auth/error',
};

// Route metadata
const routeMeta: Record<Route, { title: string; description: string }> = {
  'home': { title: 'Home', description: 'Your music blocklist dashboard' },
  'settings': { title: 'Settings', description: 'Account and connection settings' },
  'oauth-callback': { title: 'Connecting...', description: 'Processing authentication' },
  'oauth-error': { title: 'Connection Error', description: 'There was a problem connecting' },
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
