import { writable } from 'svelte/store';

export interface Route {
  path: string;
  component: any;
  title: string;
}

export const currentRoute = writable<string>('overview');

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
    currentRoute.set(route);
    // Update URL without page reload
    if (typeof window !== 'undefined') {
      const routeConfig = routes[route];
      if (routeConfig) {
        window.history.pushState({}, routeConfig.title, routeConfig.path);
        document.title = `${routeConfig.title} - No Drake in the House`;
      }
    }
  },

  init: () => {
    if (typeof window !== 'undefined') {
      // Handle browser back/forward buttons
      window.addEventListener('popstate', () => {
        const path = window.location.pathname;
        const route = Object.keys(routes).find(key => routes[key].path === path) || 'overview';
        currentRoute.set(route);
      });

      // Set initial route based on URL
      const path = window.location.pathname;
      const route = Object.keys(routes).find(key => routes[key].path === path) || 'overview';
      currentRoute.set(route);
    }
  }
};