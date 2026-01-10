/**
 * Apple Music MusicKit JS Integration
 *
 * This module handles the MusicKit JS library for Apple Music authentication
 * and API access. The flow is:
 *
 * 1. Load MusicKit JS script
 * 2. Get developer token from backend
 * 3. Configure MusicKit with developer token
 * 4. User authorizes via MusicKit popup
 * 5. Send Music User Token to backend for storage
 */

import { apiClient } from './api-client';

// MusicKit type declarations
declare global {
  interface Window {
    MusicKit: typeof MusicKit;
  }
}

declare namespace MusicKit {
  function configure(config: Configuration): Promise<MusicKitInstance>;
  function getInstance(): MusicKitInstance;

  interface Configuration {
    developerToken: string;
    app: {
      name: string;
      build: string;
    };
  }

  interface MusicKitInstance {
    authorize(): Promise<string>;
    unauthorize(): Promise<void>;
    isAuthorized: boolean;
    musicUserToken: string;
    api: MusicKitAPI;
    player: MusicKitPlayer;
  }

  interface MusicKitAPI {
    library: {
      songs(options?: any): Promise<any>;
      albums(options?: any): Promise<any>;
      artists(options?: any): Promise<any>;
    };
  }

  interface MusicKitPlayer {
    // Player methods if needed
  }
}

// Track initialization state
let musickitLoaded = false;
let musickitConfigured = false;
let musicKitInstance: MusicKit.MusicKitInstance | null = null;

/**
 * Load the MusicKit JS script
 */
export async function loadMusicKitScript(): Promise<void> {
  if (musickitLoaded) {
    console.log('[MusicKit] Script already loaded');
    return;
  }

  console.log('[MusicKit] Loading MusicKit JS script...');
  return new Promise((resolve, reject) => {
    // Check if already loaded
    if (window.MusicKit) {
      console.log('[MusicKit] MusicKit already available on window');
      musickitLoaded = true;
      resolve();
      return;
    }

    const script = document.createElement('script');
    script.src = 'https://js-cdn.music.apple.com/musickit/v3/musickit.js';
    script.async = true;
    script.crossOrigin = 'anonymous';

    script.onload = () => {
      console.log('[MusicKit] Script loaded successfully');
      musickitLoaded = true;
      resolve();
    };

    script.onerror = (e) => {
      console.error('[MusicKit] Failed to load script:', e);
      reject(new Error('Failed to load MusicKit JS'));
    };

    document.head.appendChild(script);
  });
}

/**
 * Get developer token from backend
 */
export async function getDeveloperToken(): Promise<string> {
  console.log('[MusicKit] Fetching developer token from backend...');
  const response = await apiClient.get<{developer_token: string; expires_at: string}>(
    '/api/v1/apple-music/auth/developer-token',
    false // Don't include auth for this public endpoint
  );

  if (response.success && response.data?.developer_token) {
    console.log('[MusicKit] Got developer token');
    return response.data.developer_token;
  }

  console.error('[MusicKit] Failed to get developer token:', response.message);
  throw new Error(response.message || 'Failed to get developer token');
}

/**
 * Configure MusicKit with developer token
 */
export async function configureMusicKit(): Promise<MusicKit.MusicKitInstance> {
  if (musicKitInstance && musickitConfigured) {
    console.log('[MusicKit] Already configured, returning instance');
    return musicKitInstance;
  }

  // Ensure script is loaded
  await loadMusicKitScript();

  // Get developer token
  const developerToken = await getDeveloperToken();

  // Configure MusicKit
  console.log('[MusicKit] Configuring MusicKit...');
  musicKitInstance = await window.MusicKit.configure({
    developerToken,
    app: {
      name: 'No Drake In The House',
      build: '1.0.0',
    },
  });

  console.log('[MusicKit] MusicKit configured successfully');
  musickitConfigured = true;
  return musicKitInstance;
}

/**
 * Authorize user with Apple Music
 * Opens Apple's authorization popup
 */
export async function authorizeAppleMusic(): Promise<string> {
  const instance = await configureMusicKit();

  // This opens Apple's authorization popup
  const musicUserToken = await instance.authorize();

  return musicUserToken;
}

/**
 * Check if user is authorized with Apple Music
 */
export async function isAppleMusicAuthorized(): Promise<boolean> {
  try {
    const instance = await configureMusicKit();
    return instance.isAuthorized;
  } catch {
    return false;
  }
}

/**
 * Get the current Music User Token
 */
export async function getMusicUserToken(): Promise<string | null> {
  try {
    const instance = await configureMusicKit();
    return instance.isAuthorized ? instance.musicUserToken : null;
  } catch {
    return null;
  }
}

/**
 * Unauthorize from Apple Music (local only)
 */
export async function unauthorizeAppleMusic(): Promise<void> {
  if (musicKitInstance) {
    await musicKitInstance.unauthorize();
  }
}

/**
 * Connect Apple Music account - full flow
 * 1. Load MusicKit JS
 * 2. Get developer token from backend
 * 3. Configure MusicKit
 * 4. Authorize user via Apple popup
 * 5. Send token to backend
 */
export async function connectAppleMusic(): Promise<{success: boolean; message?: string; connectionId?: string}> {
  try {
    console.log('[MusicKit] Starting Apple Music connection flow...');

    // Step 1-3: Load, configure, and authorize
    console.log('[MusicKit] Authorizing with Apple Music...');
    const musicUserToken = await authorizeAppleMusic();
    console.log('[MusicKit] Got music user token');

    // Step 4: Send token to backend
    console.log('[MusicKit] Sending token to backend...');
    const response = await apiClient.post<{success: boolean; connection_id: string; message: string}>(
      '/api/v1/apple-music/auth/connect',
      { music_user_token: musicUserToken },
      true // Include auth
    );

    if (response.success && response.data) {
      console.log('[MusicKit] Connection successful!');
      return {
        success: true,
        connectionId: response.data.connection_id,
        message: response.data.message,
      };
    }

    console.error('[MusicKit] Backend returned error:', response.message);
    return {
      success: false,
      message: response.message || 'Failed to connect Apple Music',
    };
  } catch (error) {
    console.error('[MusicKit] Error during connection:', error);
    const message = error instanceof Error ? error.message : 'Unknown error connecting Apple Music';
    return {
      success: false,
      message,
    };
  }
}

/**
 * Disconnect Apple Music account from backend
 */
export async function disconnectAppleMusic(): Promise<{success: boolean; message?: string}> {
  try {
    // Unauthorize locally
    await unauthorizeAppleMusic();

    // Disconnect from backend
    const response = await apiClient.delete<{success: boolean; message: string}>(
      '/api/v1/apple-music/auth/disconnect',
      true
    );

    if (response.success) {
      return { success: true, message: 'Apple Music disconnected' };
    }

    return {
      success: false,
      message: response.message || 'Failed to disconnect Apple Music',
    };
  } catch (error) {
    const message = error instanceof Error ? error.message : 'Unknown error disconnecting Apple Music';
    return {
      success: false,
      message,
    };
  }
}

/**
 * Get Apple Music connection status from backend
 */
export async function getAppleMusicStatus(): Promise<{
  connected: boolean;
  connectionId?: string;
  status?: string;
  lastHealthCheck?: string;
}> {
  const response = await apiClient.get<{
    connected: boolean;
    connection_id?: string;
    status?: string;
    last_health_check?: string;
  }>(
    '/api/v1/apple-music/auth/status',
    true
  );

  if (response.success && response.data) {
    return {
      connected: response.data.connected,
      connectionId: response.data.connection_id,
      status: response.data.status,
      lastHealthCheck: response.data.last_health_check,
    };
  }

  return { connected: false };
}

/**
 * Verify Apple Music connection health
 */
export async function verifyAppleMusicConnection(): Promise<{
  healthy: boolean;
  needsRefresh: boolean;
  error?: string;
}> {
  const response = await apiClient.post<{
    healthy: boolean;
    needs_refresh: boolean;
    error?: string;
  }>(
    '/api/v1/apple-music/auth/verify',
    {},
    true
  );

  if (response.success && response.data) {
    return {
      healthy: response.data.healthy,
      needsRefresh: response.data.needs_refresh,
      error: response.data.error,
    };
  }

  return {
    healthy: false,
    needsRefresh: false,
    error: response.message || 'Failed to verify connection',
  };
}
