import { writable } from 'svelte/store';

export interface Route {
  path: string;
  component: any;
  title: string;
}

// Create a more robust store with debugging
function createRouteStore() {
  const { subscribe, set, update } = writable<string>('overview');
  
  return {
    subscribe,
    set: (value: string) => {
      console.log('RouteStore: Setting route to:', value);
      set(value);
    },
    update
  };
}

export const currentRoute = createRouteStore();

export const routes: Record<string, Route> = {
  overview: {
    path: '/',
    component: null, // Will be handled in Dashboard component
    title: 'Overview'
  },
  connections: {
    path: '/connections',
    component: null,
    title: 'Service Connections'
  },
  dnp: {
    path: '/dnp',
    component: null,
    title: 'DNP List'
  },
  enforcement: {
    path: '/enforcement',
    component: null,
    title: 'Enforcement'
  },
  community: {
    path: '/community',
    component: null,
    title: 'Community Lists'
  },
  profile: {
    path: '/profile',
    component: null,
    title: 'Profile & Settings'
  }
};

export const router = {
  navigate: (route: string) => {
    console.log('Router navigating to:', route);
    
    // Validate route exists
    if (!routes[route]) {
      console.warn('Invalid route:', route);
      return;
    }
    
    // Force update store with a slight delay to ensure reactivity
    setTimeout(() => {
      currentRoute.set(route);
      console.log('Route store updated to:', route);
    }, 0);
    
    // Update URL without page reload
    if (typeof window !== 'undefined') {
      const routeConfig = routes[route];
      try {
        window.history.pushState({ route }, routeConfig.title, routeConfig.path);
        document.title = `${routeConfig.title} - No Drake in the House`;
        console.log('URL updated to:', routeConfig.path);
      } catch (error) {
        console.error('Failed to update URL:', error);
      }
    }
  },

  init: () => {
    if (typeof window !== 'undefined') {
      // Handle browser back/forward buttons
      window.addEventListener('popstate', (event) => {
        console.log('Popstate event:', event.state);
        const path = window.location.pathname;
        const route = Object.keys(routes).find(key => routes[key].path === path) || 'overview';
        console.log('Setting route from popstate:', route);
        currentRoute.set(route);
      });

      // Set initial route based on URL
      const path = window.location.pathname;
      const route = Object.keys(routes).find(key => routes[key].path === path) || 'overview';
      console.log('Initial route:', route, 'from path:', path);
      currentRoute.set(route);
    }
  }
};