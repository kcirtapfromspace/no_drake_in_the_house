/// <reference types="vite/client" />
/// <reference types="vitest/globals" />
/// <reference types="@testing-library/jest-dom" />

interface ImportMetaEnv {
  readonly VITE_API_URL: string
  readonly VITE_API_VERSION: string
  readonly VITE_APP_NAME: string
  readonly VITE_ENVIRONMENT: string
  readonly VITE_ENABLE_2FA: string
  readonly VITE_ENABLE_COMMUNITY_LISTS: string
  readonly VITE_ENABLE_ANALYTICS: string
  readonly VITE_HOT_RELOAD: string
  readonly VITE_DEBUG_MODE: string
  readonly VITE_SPOTIFY_CLIENT_ID: string
  readonly VITE_APPLE_MUSIC_DEVELOPER_TOKEN: string
  readonly VITE_DEFAULT_THEME: string
  readonly VITE_ENABLE_SERVICE_WORKER: string
  readonly VITE_CACHE_DURATION: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}