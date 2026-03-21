import { test, expect } from './fixtures';
import { Page } from '@playwright/test';

/**
 * Spotify OAuth E2E Tests
 *
 * Tests the Spotify OAuth connect/disconnect flow, callback handling,
 * and connection status display.
 *
 * The Spotify connect/disconnect UI lives on the Settings page under
 * "Music Services". The OAuth callback is handled by OAuthCallback.svelte
 * at /auth/callback/spotify.
 */

// -- Mock data ----------------------------------------------------------------

const MOCK_SPOTIFY_AUTH_URL = 'https://accounts.spotify.com/authorize?client_id=test-client&response_type=code&redirect_uri=http%3A%2F%2Flocalhost%3A5000%2Fauth%2Fcallback%2Fspotify&scope=user-library-read+user-library-modify&state=mock-oauth-state';

const MOCK_SPOTIFY_CONNECTION = {
  id: 'conn-spotify-e2e',
  provider: 'spotify',
  provider_user_id: 'spotify-user-42',
  health_status: 'active' as const,
  scopes: ['user-library-read', 'user-library-modify'],
  expires_at: '2026-06-01T00:00:00Z',
  last_used_at: '2026-03-20T12:00:00Z',
};

// -- Helpers ------------------------------------------------------------------

/**
 * Register Spotify-specific API mock routes on the given page.
 * These extend (and in some cases override) the base fixture mocks.
 */
async function mockSpotifyApiRoutes(page: Page, options: {
  connected?: boolean;
  authorizeError?: boolean;
  callbackError?: boolean;
  disconnectError?: boolean;
} = {}) {
  const { connected = false, authorizeError = false, callbackError = false, disconnectError = false } = options;

  // GET /api/v1/connections — include spotify when connected
  await page.route('**/api/v1/connections', async (route) => {
    if (route.request().method() !== 'GET') {
      await route.fallback();
      return;
    }
    const connections = connected
      ? { connections: [MOCK_SPOTIFY_CONNECTION] }
      : { connections: [] };
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(connections),
    });
  });

  // GET /api/v1/connections/spotify/authorize
  await page.route('**/api/v1/connections/spotify/authorize', async (route) => {
    if (authorizeError) {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          success: false,
          message: 'Spotify service unavailable',
        }),
      });
      return;
    }
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        success: true,
        data: {
          authorization_url: MOCK_SPOTIFY_AUTH_URL,
          state: 'mock-oauth-state',
          scopes: ['user-library-read', 'user-library-modify'],
        },
      }),
    });
  });

  // POST /api/v1/connections/spotify/callback
  await page.route('**/api/v1/connections/spotify/callback', async (route) => {
    if (callbackError) {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          success: false,
          message: 'Invalid authorization code',
        }),
      });
      return;
    }
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        success: true,
        connection_id: MOCK_SPOTIFY_CONNECTION.id,
        provider_user_id: MOCK_SPOTIFY_CONNECTION.provider_user_id,
        status: 'active',
        message: 'Spotify connected successfully',
      }),
    });
  });

  // GET /api/v1/connections/spotify/status
  await page.route('**/api/v1/connections/spotify/status', async (route) => {
    if (!connected) {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          success: true,
          data: { connected: false },
        }),
      });
      return;
    }
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        success: true,
        data: {
          connected: true,
          connection_id: MOCK_SPOTIFY_CONNECTION.id,
          provider_user_id: MOCK_SPOTIFY_CONNECTION.provider_user_id,
          status: 'active',
          scopes: MOCK_SPOTIFY_CONNECTION.scopes,
          expires_at: MOCK_SPOTIFY_CONNECTION.expires_at,
        },
      }),
    });
  });

  // DELETE /api/v1/connections/spotify
  await page.route('**/api/v1/connections/spotify', async (route) => {
    if (route.request().method() !== 'DELETE') {
      await route.fallback();
      return;
    }
    if (disconnectError) {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          success: false,
          message: 'Failed to disconnect Spotify',
        }),
      });
      return;
    }
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ success: true, message: 'Disconnected' }),
    });
  });

  // Mock the health endpoint so the app initialises
  await page.route('**/health', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ status: 'ok' }),
    });
  });

  // Mock the users profile endpoint (used by fetchProfile in auth store)
  await page.route('**/api/v1/users/profile', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        success: true,
        data: {
          id: 'test-user-123',
          email: 'test@example.com',
          display_name: 'Test User',
          email_verified: true,
          totp_enabled: false,
          created_at: '2024-01-01T00:00:00Z',
        },
      }),
    });
  });

  // Mock Apple Music MusicKit config (prevents real network call)
  await page.route('**/api/v1/connections/apple-music/musickit-token', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ success: false, message: 'Not configured in test' }),
    });
  });
}

/**
 * Set up an authenticated page with Spotify mocks and navigate to Settings.
 */
async function setupSettingsPage(page: Page, options: Parameters<typeof mockSpotifyApiRoutes>[1] = {}) {
  await mockSpotifyApiRoutes(page, options);

  // Set auth token so the app considers the user logged in
  await page.addInitScript(() => {
    localStorage.setItem('auth_token', 'mock-jwt-token-12345');
    localStorage.setItem('user', JSON.stringify({
      id: 'test-user-123',
      email: 'test@example.com',
      display_name: 'Test User',
    }));
  });

  await page.goto('/settings');
  await page.waitForLoadState('networkidle');
}

// -- Tests --------------------------------------------------------------------

test.describe('Spotify OAuth Integration', () => {

  // ---- Connection visibility ------------------------------------------------

  test.describe('Connection display', () => {
    test('Spotify connect button is visible on the settings page when not connected', async ({ page, mockApi }) => {
      await mockApi(page);
      await setupSettingsPage(page, { connected: false });

      // The Settings page shows "Music Services" section with a card per service
      await expect(page.getByRole('heading', { name: 'Spotify' })).toBeVisible({ timeout: 10_000 });

      // When not connected the status pill should read "Not Connected"
      const spotifyCard = page.locator('article').filter({ hasText: 'Spotify' }).first();
      await expect(spotifyCard.getByText('Not Connected')).toBeVisible();

      // The "Connect Account" button should be visible
      await expect(spotifyCard.getByRole('button', { name: 'Connect Account' })).toBeVisible();
    });

    test('connection status is displayed correctly when Spotify is connected', async ({ page, mockApi }) => {
      await mockApi(page);
      await setupSettingsPage(page, { connected: true });

      const spotifyCard = page.locator('article').filter({ hasText: 'Spotify' }).first();
      await expect(spotifyCard).toBeVisible({ timeout: 10_000 });

      // When connected the status pill should read "Connected"
      await expect(spotifyCard.getByText('Connected')).toBeVisible();

      // The description changes to the connectedDescription
      await expect(spotifyCard.getByText(/Spotify is connected/)).toBeVisible();

      // "Disconnect" button should be available
      await expect(spotifyCard.getByRole('button', { name: 'Disconnect' })).toBeVisible();

      // "Open Library" button should also be visible
      await expect(spotifyCard.getByRole('button', { name: 'Open Library' })).toBeVisible();
    });
  });

  // ---- Authorize flow -------------------------------------------------------

  test.describe('Authorize flow', () => {
    test('clicking Connect calls the authorize endpoint and receives an authorization URL', async ({ page, mockApi }) => {
      await mockApi(page);

      // Intercept the navigation that the store triggers (window.location.href = auth_url)
      // by capturing the request to the authorize endpoint.
      let authorizeRequested = false;
      await page.route('**/api/v1/connections/spotify/authorize', async (route) => {
        authorizeRequested = true;
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            success: true,
            data: {
              authorization_url: MOCK_SPOTIFY_AUTH_URL,
              state: 'mock-oauth-state',
              scopes: ['user-library-read', 'user-library-modify'],
            },
          }),
        });
      });

      // Prevent the actual navigation to Spotify's authorize URL
      await page.route(MOCK_SPOTIFY_AUTH_URL, async (route) => {
        await route.abort();
      });

      await setupSettingsPage(page, { connected: false });

      const spotifyCard = page.locator('article').filter({ hasText: 'Spotify' }).first();
      const connectButton = spotifyCard.getByRole('button', { name: 'Connect Account' });
      await expect(connectButton).toBeVisible({ timeout: 10_000 });

      // Click connect — the store calls GET /authorize, stores the state in
      // sessionStorage, and sets window.location.href to the authorization URL.
      // We intercept the navigation so the page doesn't actually leave.
      await page.evaluate(() => {
        // Prevent the page from navigating away so Playwright stays on the page
        Object.defineProperty(window, 'location', {
          get: () => ({
            ...document.location,
            set href(url: string) {
              window.dispatchEvent(new CustomEvent('__test_nav__', { detail: url }));
            },
          }),
        });
      });

      // Collect navigation attempts
      const navPromise = page.evaluate(() =>
        new Promise<string>((resolve) => {
          window.addEventListener('__test_nav__', ((e: CustomEvent) => {
            resolve(e.detail);
          }) as EventListener, { once: true });
        })
      );

      await connectButton.click();

      // Verify the authorize endpoint was called
      expect(authorizeRequested).toBe(true);
    });
  });

  // ---- OAuth callback success -----------------------------------------------

  test.describe('OAuth callback page', () => {
    test('handles a successful callback correctly', async ({ page, mockApi }) => {
      await mockApi(page);
      await mockSpotifyApiRoutes(page, { connected: false });

      // Set auth token so the app considers the user logged in
      await page.addInitScript(() => {
        localStorage.setItem('auth_token', 'mock-jwt-token-12345');
      });

      // Navigate to the callback URL with code and state query params
      await page.goto('/auth/callback/spotify?code=test-auth-code&state=test-oauth-state');
      await page.waitForLoadState('networkidle');

      // OAuthCallback.svelte should show the "Connecting Spotify" loading state,
      // then transition to "Connected!" on success.
      await expect(page.getByText('Connected!')).toBeVisible({ timeout: 10_000 });
      await expect(page.getByText(/Spotify.*account has been linked successfully/)).toBeVisible();

      // A "Go to Settings" button should be visible (the component redirects to sync
      // for connection providers, but the button text says "Go to Settings")
      await expect(page.getByRole('button', { name: 'Go to Settings' })).toBeVisible();
    });

    test('handles callback errors correctly', async ({ page, mockApi }) => {
      await mockApi(page);
      await mockSpotifyApiRoutes(page, { connected: false, callbackError: true });

      await page.addInitScript(() => {
        localStorage.setItem('auth_token', 'mock-jwt-token-12345');
      });

      await page.goto('/auth/callback/spotify?code=bad-code&state=bad-state');
      await page.waitForLoadState('networkidle');

      // Should show error state
      await expect(page.getByText('Connection Failed')).toBeVisible({ timeout: 10_000 });
      await expect(page.getByText('Invalid authorization code')).toBeVisible();

      // "Try Again" and "Go Home" buttons should be present
      await expect(page.getByRole('button', { name: 'Try Again' })).toBeVisible();
      await expect(page.getByRole('button', { name: 'Go Home' })).toBeVisible();
    });

    test('handles provider-side errors in the callback URL', async ({ page, mockApi }) => {
      await mockApi(page);
      await mockSpotifyApiRoutes(page, { connected: false });

      await page.addInitScript(() => {
        localStorage.setItem('auth_token', 'mock-jwt-token-12345');
      });

      // Spotify redirects back with an error param when the user denies access
      await page.goto('/auth/callback/spotify?error=access_denied&error_description=User+cancelled+the+authorization');
      await page.waitForLoadState('networkidle');

      await expect(page.getByText('Connection Failed')).toBeVisible({ timeout: 10_000 });
      await expect(page.getByText('User cancelled the authorization')).toBeVisible();
    });

    test('handles missing auth parameters in the callback URL', async ({ page, mockApi }) => {
      await mockApi(page);
      await mockSpotifyApiRoutes(page, { connected: false });

      await page.addInitScript(() => {
        localStorage.setItem('auth_token', 'mock-jwt-token-12345');
      });

      // Navigate to callback without code or state
      await page.goto('/auth/callback/spotify');
      await page.waitForLoadState('networkidle');

      await expect(page.getByText('Connection Failed')).toBeVisible({ timeout: 10_000 });
      await expect(page.getByText('Missing authentication parameters')).toBeVisible();
    });
  });

  // ---- Disconnect flow ------------------------------------------------------

  test.describe('Disconnect flow', () => {
    test('disconnect flow works when Spotify is connected', async ({ page, mockApi }) => {
      await mockApi(page);

      // Start with Spotify connected, but after disconnect return empty connections
      let disconnected = false;
      await page.route('**/api/v1/connections', async (route) => {
        if (route.request().method() !== 'GET') {
          await route.fallback();
          return;
        }
        const connections = disconnected
          ? { connections: [] }
          : { connections: [MOCK_SPOTIFY_CONNECTION] };
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify(connections),
        });
      });

      await page.route('**/api/v1/connections/spotify', async (route) => {
        if (route.request().method() !== 'DELETE') {
          await route.fallback();
          return;
        }
        disconnected = true;
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({ success: true, message: 'Disconnected successfully' }),
        });
      });

      await setupSettingsPage(page, { connected: true });

      const spotifyCard = page.locator('article').filter({ hasText: 'Spotify' }).first();
      await expect(spotifyCard).toBeVisible({ timeout: 10_000 });

      // Click Disconnect — Settings shows a confirmation strip
      const disconnectButton = spotifyCard.getByRole('button', { name: 'Disconnect' });
      await expect(disconnectButton).toBeVisible();
      await disconnectButton.click();

      // A confirmation prompt "Disconnect Spotify?" with Yes/No appears
      await expect(spotifyCard.getByText('Disconnect Spotify?')).toBeVisible();
      const yesButton = spotifyCard.getByRole('button', { name: 'Yes' });
      await expect(yesButton).toBeVisible();

      // Confirm the disconnect
      await yesButton.click();

      // After successful disconnect a success banner should appear
      await expect(page.getByText(/Spotify disconnected/)).toBeVisible({ timeout: 10_000 });
    });
  });

  // ---- Error scenarios ------------------------------------------------------

  test.describe('Error handling', () => {
    test('shows error when authorize endpoint fails', async ({ page, mockApi }) => {
      await mockApi(page);
      await setupSettingsPage(page, { connected: false, authorizeError: true });

      const spotifyCard = page.locator('article').filter({ hasText: 'Spotify' }).first();
      const connectButton = spotifyCard.getByRole('button', { name: 'Connect Account' });
      await expect(connectButton).toBeVisible({ timeout: 10_000 });

      await connectButton.click();

      // The Settings component should display the connection error banner
      await expect(page.getByText(/Failed to initiate Spotify auth|Spotify service unavailable/)).toBeVisible({ timeout: 10_000 });
    });
  });
});
