import { writable } from 'svelte/store';

// Simple, reliable router store
export const currentRoute = writable<string>('overview');

// Simple navigation function
export function navigateTo(route: string) {
  console.log('Simple router: Navigating to:', route);

  // Update the store immediately
  currentRoute.set(route);

  // Update URL
  if (typeof window !== 'undefined') {
    const paths: Record<string, string> = {
      overview: '/',
      connections: '/connections',
      dnp: '/dnp',
      enforcement: '/enforcement',
      community: '/community',
      profile: '/profile',
      'oauth-callback': '/auth/callback',
      'oauth-error': '/auth/error'
    };

    const path = paths[route] || '/';
    window.history.pushState({ route }, route, path);
    document.title = `${route} - No Drake in the House`;
  }
}

// Get route from path
function getRouteFromPath(path: string): string {
  if (path === '/connections') return 'connections';
  if (path === '/dnp') return 'dnp';
  if (path === '/enforcement') return 'enforcement';
  if (path === '/community') return 'community';
  if (path === '/profile') return 'profile';
  if (path.startsWith('/auth/callback')) return 'oauth-callback';
  if (path.startsWith('/auth/error')) return 'oauth-error';
  return 'overview';
}

// Initialize router
export function initRouter() {
  if (typeof window !== 'undefined') {
    // Handle browser back/forward
    window.addEventListener('popstate', () => {
      const path = window.location.pathname;
      const route = getRouteFromPath(path);
      currentRoute.set(route);
    });

    // Set initial route
    const path = window.location.pathname;
    const initialRoute = getRouteFromPath(path);
    currentRoute.set(initialRoute);
  }
}