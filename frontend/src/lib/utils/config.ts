// Configuration utilities for environment-based settings

export const config = {
  // API Configuration
  apiUrl: import.meta.env.VITE_API_URL || 'http://localhost:3000',
  apiVersion: import.meta.env.VITE_API_VERSION || 'v1',
  
  // App Configuration
  appName: import.meta.env.VITE_APP_NAME || 'No Drake in the House',
  environment: import.meta.env.VITE_ENVIRONMENT || 'development',
  
  // Feature Flags
  features: {
    twoFactorAuth: import.meta.env.VITE_ENABLE_2FA === 'true',
    communityLists: import.meta.env.VITE_ENABLE_COMMUNITY_LISTS === 'true',
    analytics: import.meta.env.VITE_ENABLE_ANALYTICS === 'true',
  },
  
  // Development Configuration
  development: {
    hotReload: import.meta.env.VITE_HOT_RELOAD === 'true',
    debugMode: import.meta.env.VITE_DEBUG_MODE === 'true',
  },
  
  // External Services
  external: {
    spotifyClientId: import.meta.env.VITE_SPOTIFY_CLIENT_ID,
    appleMusicToken: import.meta.env.VITE_APPLE_MUSIC_DEVELOPER_TOKEN,
  },
  
  // UI Configuration
  ui: {
    defaultTheme: import.meta.env.VITE_DEFAULT_THEME || 'light',
  },
  
  // Performance Configuration
  performance: {
    enableServiceWorker: import.meta.env.VITE_ENABLE_SERVICE_WORKER === 'true',
    cacheDuration: parseInt(import.meta.env.VITE_CACHE_DURATION || '300000'),
  },
  
  // Helper methods
  isDevelopment: () => config.environment === 'development',
  isProduction: () => config.environment === 'production',
  getApiEndpoint: (path: string) => {
    const normalizedPath = path.startsWith('/') ? path : `/${path}`;
    const apiPath = normalizedPath.startsWith('/api/') ? normalizedPath : `/api/${config.apiVersion}${normalizedPath}`;
    return `${config.apiUrl}${apiPath}`;
  }
};

export default config;