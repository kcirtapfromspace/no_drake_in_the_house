// Configuration utilities for environment-based settings

const normalizeBaseUrl = (value: string) => value.replace(/\/+$/, '');
const RELATIVE_API_SENTINEL = '__RELATIVE__';
const normalizeEnvValue = (value: string) => value.replace(/^(['"])(.*)\1$/, '$2');
const POSTHOG_PROXY_HOST = 'https://t.nodrakeinthe.house';
const POSTHOG_DEFAULT_DIRECT_HOST = 'https://us.i.posthog.com';
const resolveApiUrl = (value: string) => {
  const normalized = normalizeEnvValue(value);
  return normalized === RELATIVE_API_SENTINEL ? '' : normalizeBaseUrl(normalized);
};

const resolveOptionalBaseUrl = (value: string) => {
  const normalized = normalizeEnvValue(value || '');
  return normalized ? normalizeBaseUrl(normalized) : '';
};

const convexUrl = resolveOptionalBaseUrl(import.meta.env.VITE_CONVEX_URL || '');
const runtimeEnv = typeof window !== 'undefined' ? (window as any).__ENV__ : undefined;
const runtimeHostname = typeof window !== 'undefined' ? (window.location?.hostname || '').toLowerCase() : '';
const isLocalRuntimeHost =
  runtimeHostname === 'localhost' ||
  runtimeHostname === '127.0.0.1' ||
  runtimeHostname === '0.0.0.0' ||
  runtimeHostname.endsWith('.local');
const explicitEnvironment = normalizeEnvValue(import.meta.env.VITE_ENVIRONMENT || '').toLowerCase();
const isProductionEnvironment = explicitEnvironment === 'production' || (!!runtimeHostname && !isLocalRuntimeHost);
const resolvedEnvironment = isProductionEnvironment ? 'production' : (explicitEnvironment || 'development');
const resolvePostHogApiHost = () => {
  const runtimeHost = normalizeEnvValue(runtimeEnv?.VITE_POSTHOG_HOST || '');
  if (runtimeHost) {
    return normalizeBaseUrl(runtimeHost);
  }

  return isProductionEnvironment ? POSTHOG_PROXY_HOST : POSTHOG_DEFAULT_DIRECT_HOST;
};

export const config = {
  // API Configuration - defaults to empty string for relative URLs (nginx proxy)
  apiUrl: resolveApiUrl(import.meta.env.VITE_API_URL || ''),
  apiVersion: import.meta.env.VITE_API_VERSION || 'v1',
  
  // App Configuration
  appName: normalizeEnvValue(import.meta.env.VITE_APP_NAME || 'No Drake in the House'),
  environment: resolvedEnvironment,
  
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

  // PostHog Analytics (runtime-injected via window.__ENV__ by render-entrypoint.sh)
  posthog: {
    apiKey: runtimeEnv?.VITE_POSTHOG_API_KEY || '',
    apiHost: resolvePostHogApiHost(),
  },

  auth: {
    mode: 'legacy' as const,
  },

  convex: {
    url: convexUrl,
    signedUpdateUrl: normalizeEnvValue(import.meta.env.VITE_EXTENSION_SIGNED_UPDATE_URL || ''),
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
  
  resolveUrl: (path: string) => {
    if (/^https?:\/\//i.test(path)) {
      return path;
    }

    const normalizedPath = path.startsWith('/') ? path : `/${path}`;
    return `${config.apiUrl}${normalizedPath}`;
  },
  getBackendEndpoint: (path: string) => {
    const normalizedPath = path.startsWith('/') ? path : `/${path}`;
    if (config.apiUrl) {
      return config.resolveUrl(normalizedPath);
    }

    // When the frontend is reverse-proxying requests, backend root health endpoints
    // are exposed under /api/* to avoid colliding with the frontend container health checks.
    if (normalizedPath === '/health' || normalizedPath.startsWith('/health/')) {
      return `/api${normalizedPath}`;
    }

    return normalizedPath;
  },
  // Helper methods
  isDevelopment: () => config.environment === 'development',
  isProduction: () => config.environment === 'production',
  getApiEndpoint: (path: string) => {
    const normalizedPath = path.startsWith('/') ? path : `/${path}`;
    const apiPath = normalizedPath.startsWith('/api/') ? normalizedPath : `/api/${config.apiVersion}${normalizedPath}`;
    return config.resolveUrl(apiPath);
  }
};

export default config;
