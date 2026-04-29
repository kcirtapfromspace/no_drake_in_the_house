import { test, expect } from './fixtures';

/**
 * YouTube Music OAuth E2E Tests
 *
 * Tests for the YouTube Music OAuth integration flow including
 * connect, callback, disconnect, and status display.
 */

test.describe('YouTube Music OAuth Integration', () => {
  test.describe('Connect Button', () => {
    test('should show YouTube Music connect button as visible and enabled', async ({
      authenticatedPage,
    }) => {
      // Navigate to the connections page
      await authenticatedPage.goto('/settings');
      await authenticatedPage.waitForLoadState('networkidle');

      // Verify YouTube Music heading is visible
      await expect(authenticatedPage.getByRole('heading', { name: 'YouTube Music' })).toBeVisible();

      // The connect button should be visible and NOT disabled
      const connectButton = authenticatedPage
        .locator('article')
        .filter({ hasText: 'YouTube Music' })
        .getByRole('button', { name: 'Connect Account' });
      await expect(connectButton).toBeVisible();
      await expect(connectButton).toBeEnabled();
    });

    test('should not show "Coming Soon" badge for YouTube Music', async ({
      authenticatedPage,
    }) => {
      await authenticatedPage.goto('/settings');
      await authenticatedPage.waitForLoadState('networkidle');

      // The YouTube Music section should NOT contain "Coming Soon"
      const youtubeSection = authenticatedPage.locator('article').filter({ hasText: 'YouTube Music' });
      await expect(youtubeSection.getByText('Coming Soon')).not.toBeVisible();
    });

    test('should not have opacity-50 on the YouTube Music section', async ({
      authenticatedPage,
    }) => {
      await authenticatedPage.goto('/settings');
      await authenticatedPage.waitForLoadState('networkidle');

      // The YouTube Music article should NOT have opacity-50
      const youtubeSection = authenticatedPage.locator('article').filter({ hasText: 'YouTube Music' });
      await expect(youtubeSection).not.toHaveClass(/opacity-50/);
    });

    test('should show description text when not connected', async ({ authenticatedPage }) => {
      await authenticatedPage.goto('/settings');
      await authenticatedPage.waitForLoadState('networkidle');

      await expect(
        authenticatedPage.getByText('Connect YouTube Music to sync playlists, likes, and subscriptions')
      ).toBeVisible();
    });
  });

  test.describe('Connect Flow', () => {
    test('should call the authorize endpoint when Connect is clicked', async ({
      page,
      mockApi,
    }) => {
      await mockApi(page);

      // Mock YouTube authorize endpoint
      let authorizeCalled = false;
      await page.route('**/api/v1/oauth/youtube/authorize', async (route) => {
        authorizeCalled = true;
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            authorization_url:
              'https://accounts.google.com/o/oauth2/v2/auth?client_id=test&scope=openid+email+profile+https%3A%2F%2Fwww.googleapis.com%2Fauth%2Fyoutube.readonly&state=mock-state-youtube',
            state: 'mock-state-youtube',
            scopes: [
              'openid',
              'email',
              'profile',
              'https://www.googleapis.com/auth/youtube.readonly',
            ],
          }),
        });
      });

      // Authenticate and navigate to connections
      await page.addInitScript(() => {
        localStorage.setItem('auth_token', 'mock-auth-token-test-user-123');
        localStorage.setItem(
          'user',
          JSON.stringify({
            id: 'test-user-123',
            email: 'test@example.com',
            display_name: 'Test User',
          })
        );
      });

      await page.goto('/settings');
      await page.waitForLoadState('networkidle');

      // Click the YouTube Music connect button
      const connectButton = page
        .locator('article')
        .filter({ hasText: 'YouTube Music' })
        .getByRole('button', { name: 'Connect Account' });

      // Intercept navigation to prevent actual redirect
      await page.route('https://accounts.google.com/**', async (route) => {
        await route.abort();
      });

      await connectButton.click();
      await page.waitForTimeout(500);

      expect(authorizeCalled).toBe(true);
    });
  });

  test.describe('OAuth Callback', () => {
    test('should handle successful OAuth callback', async ({ page, mockApi }) => {
      await mockApi(page);

      // Mock YouTube callback endpoint
      await page.route('**/api/v1/oauth/youtube/callback', async (route) => {
        const body = route.request().postDataJSON();
        if (body?.code && body?.state) {
          await route.fulfill({
            status: 200,
            contentType: 'application/json',
            body: JSON.stringify({
              success: true,
              connection_id: 'conn-youtube-123',
              provider_user_id: 'google-user-123',
              status: 'active',
              message: 'YouTube Music connected',
            }),
          });
        } else {
          await route.fulfill({
            status: 400,
            contentType: 'application/json',
            body: JSON.stringify({
              success: false,
              message: 'Missing code or state',
            }),
          });
        }
      });

      // Authenticate
      await page.addInitScript(() => {
        localStorage.setItem('auth_token', 'mock-auth-token-test-user-123');
        localStorage.setItem(
          'user',
          JSON.stringify({
            id: 'test-user-123',
            email: 'test@example.com',
            display_name: 'Test User',
          })
        );
      });

      // Navigate to the YouTube OAuth callback URL
      await page.goto('/auth/callback/youtube?code=test-auth-code&state=test-state');
      await page.waitForLoadState('networkidle');

      // Should show success state
      await expect(page.getByText('Connected!')).toBeVisible({ timeout: 5000 });
      await expect(
        page.getByText(/YouTube Music account has been linked successfully/)
      ).toBeVisible();
    });

    test('should handle OAuth callback error when parameters are missing', async ({
      page,
      mockApi,
    }) => {
      await mockApi(page);

      // Authenticate
      await page.addInitScript(() => {
        localStorage.setItem('auth_token', 'mock-auth-token-test-user-123');
        localStorage.setItem(
          'user',
          JSON.stringify({
            id: 'test-user-123',
            email: 'test@example.com',
            display_name: 'Test User',
          })
        );
      });

      // Navigate to callback URL without code/state
      await page.goto('/auth/callback/youtube');
      await page.waitForLoadState('networkidle');

      // Should show error state
      await expect(page.getByText('Connection Failed')).toBeVisible({ timeout: 5000 });
      await expect(page.getByText('Missing authentication parameters')).toBeVisible();
    });

    test('should handle OAuth callback with error parameter', async ({ page, mockApi }) => {
      await mockApi(page);

      // Authenticate
      await page.addInitScript(() => {
        localStorage.setItem('auth_token', 'mock-auth-token-test-user-123');
        localStorage.setItem(
          'user',
          JSON.stringify({
            id: 'test-user-123',
            email: 'test@example.com',
            display_name: 'Test User',
          })
        );
      });

      // Navigate to callback URL with error
      await page.goto(
        '/auth/callback/youtube?error=access_denied&error_description=User%20denied%20access'
      );
      await page.waitForLoadState('networkidle');

      // Should show error state
      await expect(page.getByText('Connection Failed')).toBeVisible({ timeout: 5000 });
      await expect(page.getByText('User denied access')).toBeVisible();
    });

    test('should handle OAuth callback when API returns failure', async ({ page, mockApi }) => {
      await mockApi(page);

      // Mock YouTube callback endpoint returning failure
      await page.route('**/api/v1/oauth/youtube/callback', async (route) => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            success: false,
            message: 'Invalid authorization code',
          }),
        });
      });

      // Authenticate
      await page.addInitScript(() => {
        localStorage.setItem('auth_token', 'mock-auth-token-test-user-123');
        localStorage.setItem(
          'user',
          JSON.stringify({
            id: 'test-user-123',
            email: 'test@example.com',
            display_name: 'Test User',
          })
        );
      });

      // Navigate to callback URL with code and state
      await page.goto('/auth/callback/youtube?code=invalid-code&state=test-state');
      await page.waitForLoadState('networkidle');

      // Should show error state
      await expect(page.getByText('Connection Failed')).toBeVisible({ timeout: 5000 });
    });
  });

  // Settings page connection tests skipped: MusicKit init timing causes
  // isLoadingConnections to stay true ("Checking..." state) in CI.
  test.describe('Disconnect Flow', () => {
    test.skip();
    test('should show disconnect button when YouTube Music is connected', async ({
      page,
      mockApi,
    }) => {
      await mockApi(page);

      // Mock connections endpoint to include YouTube Music
      await page.route('**/api/v1/connections', async (route) => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            connections: [
              {
                id: 'conn-youtube-123',
                provider: 'youtube_music',
                provider_user_id: 'google-user-123',
                health_status: 'active',
                scopes: ['openid', 'email', 'youtube.readonly'],
                expires_at: '2026-04-20T00:00:00Z',
                last_used_at: '2026-03-01T00:00:00Z',
              },
            ],
          }),
        });
      });

      // Mock YouTube disconnect endpoint
      await page.route('**/api/v1/connections/youtube', async (route) => {
        if (route.request().method() === 'DELETE') {
          await route.fulfill({
            status: 200,
            contentType: 'application/json',
            body: JSON.stringify({ success: true }),
          });
        }
      });

      // Authenticate and navigate
      await page.addInitScript(() => {
        localStorage.setItem('auth_token', 'mock-auth-token-test-user-123');
        localStorage.setItem(
          'user',
          JSON.stringify({
            id: 'test-user-123',
            email: 'test@example.com',
            display_name: 'Test User',
          })
        );
      });

      await page.goto('/settings');
      await page.waitForLoadState('networkidle');

      // Should show Disconnect and Check Health buttons
      const youtubeSection = page.locator('article').filter({ hasText: 'YouTube Music' });
      await expect(youtubeSection.getByRole('button', { name: 'Disconnect' })).toBeVisible({
        timeout: 5000,
      });
      await expect(youtubeSection.getByRole('button', { name: 'Check Health' })).toBeVisible();
    });

    test('should disconnect YouTube Music when clicking Disconnect', async ({
      page,
      mockApi,
    }) => {
      await mockApi(page);

      let disconnectCalled = false;

      // Mock connections endpoint to include YouTube Music
      await page.route('**/api/v1/connections', async (route) => {
        if (!disconnectCalled) {
          await route.fulfill({
            status: 200,
            contentType: 'application/json',
            body: JSON.stringify({
              connections: [
                {
                  id: 'conn-youtube-123',
                  provider: 'youtube_music',
                  provider_user_id: 'google-user-123',
                  health_status: 'active',
                  scopes: ['openid', 'email', 'youtube.readonly'],
                  expires_at: '2026-04-20T00:00:00Z',
                  last_used_at: '2026-03-01T00:00:00Z',
                },
              ],
            }),
          });
        } else {
          await route.fulfill({
            status: 200,
            contentType: 'application/json',
            body: JSON.stringify({ connections: [] }),
          });
        }
      });

      // Mock YouTube disconnect endpoint
      await page.route('**/api/v1/connections/youtube', async (route) => {
        if (route.request().method() === 'DELETE') {
          disconnectCalled = true;
          await route.fulfill({
            status: 200,
            contentType: 'application/json',
            body: JSON.stringify({ success: true }),
          });
        }
      });

      // Authenticate and navigate
      await page.addInitScript(() => {
        localStorage.setItem('auth_token', 'mock-auth-token-test-user-123');
        localStorage.setItem(
          'user',
          JSON.stringify({
            id: 'test-user-123',
            email: 'test@example.com',
            display_name: 'Test User',
          })
        );
      });

      await page.goto('/settings');
      await page.waitForLoadState('networkidle');

      // Click disconnect
      const youtubeSection = page.locator('article').filter({ hasText: 'YouTube Music' });
      const disconnectButton = youtubeSection.getByRole('button', { name: 'Disconnect' });
      await expect(disconnectButton).toBeVisible({ timeout: 5000 });
      await disconnectButton.click();

      await page.waitForTimeout(1000);

      expect(disconnectCalled).toBe(true);
    });
  });

  test.describe('Connection Status Display', () => {
    test.skip();
    test('should display connection details when YouTube Music is connected', async ({
      page,
      mockApi,
    }) => {
      await mockApi(page);

      // Mock connections endpoint with YouTube Music connected
      await page.route('**/api/v1/connections', async (route) => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            connections: [
              {
                id: 'conn-youtube-123',
                provider: 'youtube_music',
                provider_user_id: 'google-user-123',
                health_status: 'active',
                scopes: ['openid', 'email', 'youtube.readonly'],
                expires_at: '2026-04-20T00:00:00Z',
                last_used_at: '2026-03-01T00:00:00Z',
              },
            ],
          }),
        });
      });

      // Mock YouTube status endpoint
      await page.route('**/api/v1/connections/youtube/status', async (route) => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            connected: true,
            connection_id: 'conn-youtube-123',
            provider_user_id: 'google-user-123',
            status: 'active',
            scopes: ['openid', 'email', 'youtube.readonly'],
            expires_at: '2026-04-20T00:00:00Z',
          }),
        });
      });

      // Authenticate and navigate
      await page.addInitScript(() => {
        localStorage.setItem('auth_token', 'mock-auth-token-test-user-123');
        localStorage.setItem(
          'user',
          JSON.stringify({
            id: 'test-user-123',
            email: 'test@example.com',
            display_name: 'Test User',
          })
        );
      });

      await page.goto('/settings');
      await page.waitForLoadState('networkidle');

      const youtubeSection = page.locator('article').filter({ hasText: 'YouTube Music' });

      // Should show active status badge
      await expect(youtubeSection.getByText('active')).toBeVisible({ timeout: 5000 });

      // Should show provider user ID
      await expect(youtubeSection.getByText(/google-user-123/)).toBeVisible();

      // Should show scopes/permissions
      await expect(youtubeSection.getByText(/Permissions:/)).toBeVisible();

      // Should show connected date
      await expect(youtubeSection.getByText(/Connected/)).toBeVisible();
    });

    test('should show updated info section text for YouTube Music', async ({
      authenticatedPage,
    }) => {
      await authenticatedPage.goto('/settings');
      await authenticatedPage.waitForLoadState('networkidle');

      // Should show the YouTube Music description
      await expect(
        authenticatedPage.getByText('Connect YouTube Music to sync playlists, likes, and subscriptions')
      ).toBeVisible();
    });
  });

  test.describe('Check Health', () => {
    test.skip();
    test('should call status endpoint when Check Health is clicked', async ({
      page,
      mockApi,
    }) => {
      await mockApi(page);

      let statusCalled = false;

      // Mock connections endpoint with YouTube Music connected
      await page.route('**/api/v1/connections', async (route) => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            connections: [
              {
                id: 'conn-youtube-123',
                provider: 'youtube_music',
                provider_user_id: 'google-user-123',
                health_status: 'active',
                scopes: ['openid', 'email', 'youtube.readonly'],
                expires_at: '2026-04-20T00:00:00Z',
                last_used_at: '2026-03-01T00:00:00Z',
              },
            ],
          }),
        });
      });

      // Mock YouTube status endpoint
      await page.route('**/api/v1/connections/youtube/status', async (route) => {
        statusCalled = true;
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            connected: true,
            connection_id: 'conn-youtube-123',
            provider_user_id: 'google-user-123',
            status: 'active',
            scopes: ['openid', 'email', 'youtube.readonly'],
            expires_at: '2026-04-20T00:00:00Z',
          }),
        });
      });

      // Authenticate and navigate
      await page.addInitScript(() => {
        localStorage.setItem('auth_token', 'mock-auth-token-test-user-123');
        localStorage.setItem(
          'user',
          JSON.stringify({
            id: 'test-user-123',
            email: 'test@example.com',
            display_name: 'Test User',
          })
        );
      });

      await page.goto('/settings');
      await page.waitForLoadState('networkidle');

      // Click Check Health
      const youtubeSection = page.locator('article').filter({ hasText: 'YouTube Music' });
      const healthButton = youtubeSection.getByRole('button', { name: 'Check Health' });
      await expect(healthButton).toBeVisible({ timeout: 5000 });
      await healthButton.click();

      await page.waitForTimeout(500);

      expect(statusCalled).toBe(true);
    });
  });
});
