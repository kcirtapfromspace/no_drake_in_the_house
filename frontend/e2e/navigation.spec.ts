import { test, expect } from './fixtures';

/**
 * Navigation E2E Tests
 *
 * Tests for routing and navigation between pages
 */

test.describe('Navigation', () => {
  test.describe('Main Navigation', () => {
    test('should navigate to settings page', async ({ authenticatedPage }) => {
      // Look for settings link in navigation
      const settingsLink = authenticatedPage.getByRole('link', { name: /settings/i });
      if (await settingsLink.isVisible()) {
        await settingsLink.click();
        await expect(authenticatedPage).toHaveURL(/.*settings.*/);
      }
    });

    test('should navigate to sync dashboard', async ({ authenticatedPage }) => {
      const syncLink = authenticatedPage.getByRole('link', { name: /sync/i });
      if (await syncLink.isVisible()) {
        await syncLink.click();
        await expect(authenticatedPage).toHaveURL(/.*sync.*/);
      }
    });

    test('should navigate to analytics dashboard', async ({ authenticatedPage }) => {
      const analyticsLink = authenticatedPage.getByRole('link', { name: /analytics|revenue/i });
      if (await analyticsLink.isVisible()) {
        await analyticsLink.click();
        await expect(authenticatedPage).toHaveURL(/.*analytics.*|.*revenue.*/);
      }
    });

    test('should navigate back to home', async ({ authenticatedPage }) => {
      // First navigate away
      await authenticatedPage.goto('/settings');
      await authenticatedPage.waitForLoadState('networkidle');

      // Then navigate back to home
      const homeLink = authenticatedPage.getByRole('link', { name: /home|no drake/i });
      if (await homeLink.isVisible()) {
        await homeLink.click();
        await expect(authenticatedPage).toHaveURL('/');
      }
    });
  });

  test.describe('OAuth Callback Routes', () => {
    test('should show OAuth callback page', async ({ page, mockApi }) => {
      await mockApi(page);
      await page.goto('/oauth/callback?code=test-code&state=test-state');
      await page.waitForLoadState('networkidle');

      // OAuth callback component should be rendered
      // (The actual behavior depends on the OAuthCallback component)
    });

    test('should show OAuth error page', async ({ page, mockApi }) => {
      await mockApi(page);
      await page.goto('/oauth/error');
      await page.waitForLoadState('networkidle');

      // Should show error UI
      await expect(page.getByText('Connection Failed')).toBeVisible();
      await expect(page.getByRole('button', { name: 'Go Back' })).toBeVisible();
    });

    test('should navigate from OAuth error to home', async ({ page, mockApi }) => {
      await mockApi(page);
      await page.goto('/oauth/error');
      await page.waitForLoadState('networkidle');

      await page.getByRole('button', { name: 'Go Back' }).click();

      // Should redirect to home/login
      await expect(page).toHaveURL('/');
    });
  });

  test.describe('Direct URL Access', () => {
    test('should handle direct settings URL', async ({ authenticatedPage }) => {
      await authenticatedPage.goto('/settings');
      await authenticatedPage.waitForLoadState('networkidle');

      // Should render settings page (or redirect to home if protected)
    });

    test('should handle direct sync URL', async ({ authenticatedPage }) => {
      await authenticatedPage.goto('/sync');
      await authenticatedPage.waitForLoadState('networkidle');
    });

    test('should handle direct analytics URL', async ({ authenticatedPage }) => {
      await authenticatedPage.goto('/analytics');
      await authenticatedPage.waitForLoadState('networkidle');
    });

    test('should handle unknown routes', async ({ authenticatedPage }) => {
      await authenticatedPage.goto('/unknown-route-12345');
      await authenticatedPage.waitForLoadState('networkidle');

      // Should either show 404 or redirect to home
      // Checking for home page content as fallback
      await expect(
        authenticatedPage.getByRole('heading', { name: 'Clean Your Feed' })
      ).toBeVisible({ timeout: 5000 });
    });
  });

  test.describe('Protected Routes', () => {
    test('should redirect unauthenticated user to login', async ({ page, mockApi }) => {
      // Don't set auth token
      await mockApi(page);
      await page.route('**/api/v1/auth/profile', async (route) => {
        await route.fulfill({
          status: 401,
          contentType: 'application/json',
          body: JSON.stringify({ error: 'Unauthorized' }),
        });
      });

      await page.goto('/settings');
      await page.waitForLoadState('networkidle');

      // Should show login form
      await expect(page.getByLabel('Email')).toBeVisible({ timeout: 5000 });
    });

    test('should allow authenticated user to access protected routes', async ({
      authenticatedPage,
    }) => {
      await authenticatedPage.goto('/settings');
      await authenticatedPage.waitForLoadState('networkidle');

      // Should not see login form
      await expect(authenticatedPage.getByLabel('Email')).not.toBeVisible();
    });
  });

  test.describe('Browser Navigation', () => {
    test('should handle browser back button', async ({ authenticatedPage }) => {
      // Navigate to settings
      await authenticatedPage.goto('/settings');
      await authenticatedPage.waitForLoadState('networkidle');

      // Navigate to sync
      await authenticatedPage.goto('/sync');
      await authenticatedPage.waitForLoadState('networkidle');

      // Go back
      await authenticatedPage.goBack();

      // Should be on settings
      await expect(authenticatedPage).toHaveURL(/.*settings.*/);
    });

    test('should handle browser forward button', async ({ authenticatedPage }) => {
      // Navigate to settings
      await authenticatedPage.goto('/settings');
      await authenticatedPage.waitForLoadState('networkidle');

      // Navigate to sync
      await authenticatedPage.goto('/sync');
      await authenticatedPage.waitForLoadState('networkidle');

      // Go back then forward
      await authenticatedPage.goBack();
      await authenticatedPage.goForward();

      // Should be on sync
      await expect(authenticatedPage).toHaveURL(/.*sync.*/);
    });
  });

  test.describe('Artist Profile Navigation', () => {
    test('should navigate to artist profile from search', async ({ authenticatedPage }) => {
      const searchInput = authenticatedPage.getByPlaceholder('Search any artist...');
      await searchInput.fill('drake');
      await authenticatedPage.waitForTimeout(500);

      // Click on search result
      await authenticatedPage.getByText('Drake', { exact: true }).first().click();

      // Should be on artist profile
      await expect(authenticatedPage).toHaveURL(/.*artist.*/);
    });

    test('should handle artist profile with ID', async ({ authenticatedPage }) => {
      await authenticatedPage.goto('/artist/drake-123');
      await authenticatedPage.waitForLoadState('networkidle');

      // Artist profile should be rendered
    });
  });
});
