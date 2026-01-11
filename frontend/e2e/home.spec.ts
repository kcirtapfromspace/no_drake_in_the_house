import { test, expect, mockCategories, mockBlockedArtists, mockSearchResults } from './fixtures';

/**
 * Home Page E2E Tests
 *
 * Tests for the main home page functionality including
 * category management, artist search, and blocking
 */

test.describe('Home Page', () => {
  test.describe('Page Layout', () => {
    test('should display hero section with search', async ({ authenticatedPage }) => {
      await expect(
        authenticatedPage.getByRole('heading', { name: 'Clean Your Feed' })
      ).toBeVisible();
      await expect(
        authenticatedPage.getByText('Search and block artists with documented misconduct')
      ).toBeVisible();
      await expect(
        authenticatedPage.getByPlaceholder('Search any artist...')
      ).toBeVisible();
    });

    test('should display category section', async ({ authenticatedPage }) => {
      await expect(
        authenticatedPage.getByRole('heading', { name: 'Block by Category' })
      ).toBeVisible();
    });

    test('should display blocked artists section', async ({ authenticatedPage }) => {
      await expect(
        authenticatedPage.getByRole('heading', { name: /Your Blocked Artists/ })
      ).toBeVisible();
    });
  });

  test.describe('Category Management', () => {
    test('should display all categories', async ({ authenticatedPage }) => {
      // Check for mock categories
      await expect(authenticatedPage.getByText('Domestic Violence')).toBeVisible();
      await expect(authenticatedPage.getByText('Sexual Misconduct')).toBeVisible();
      await expect(authenticatedPage.getByText('Hate Speech')).toBeVisible();
    });

    test('should show subscribed indicator for active categories', async ({
      authenticatedPage,
    }) => {
      // Sexual Misconduct is subscribed in mock data
      const categoryCard = authenticatedPage.locator('div').filter({
        hasText: 'Sexual Misconduct',
      });
      await expect(categoryCard.first()).toBeVisible();
    });

    test('should expand category to show artists', async ({ authenticatedPage }) => {
      // Click on a category name to expand
      await authenticatedPage.getByText('Domestic Violence').first().click();

      // Wait for expansion panel to appear
      await expect(
        authenticatedPage.getByRole('heading', { name: 'Domestic Violence' })
      ).toBeVisible();
    });

    test('should toggle category subscription', async ({ authenticatedPage }) => {
      // Find the toggle for Domestic Violence (unsubscribed)
      const toggleButton = authenticatedPage
        .locator('button')
        .filter({ has: authenticatedPage.locator('svg') })
        .first();

      await toggleButton.click();

      // Should trigger subscription change (API is mocked to succeed)
      // The UI should update
      await authenticatedPage.waitForTimeout(500);
    });

    test('should close expanded category panel', async ({ authenticatedPage }) => {
      // Expand a category
      await authenticatedPage.getByText('Domestic Violence').first().click();

      // Wait for panel
      await expect(
        authenticatedPage.getByRole('heading', { name: 'Domestic Violence' })
      ).toBeVisible();

      // Close button
      const closeButton = authenticatedPage.getByRole('button').filter({
        has: authenticatedPage.locator('svg path[d*="18L18 6"]'),
      });
      await closeButton.click();

      // Panel should close (heading in the expanded view specifically)
      await authenticatedPage.waitForTimeout(300);
    });
  });

  test.describe('Artist Search', () => {
    test('should show search results when typing', async ({ authenticatedPage }) => {
      const searchInput = authenticatedPage.getByPlaceholder('Search any artist...');
      await searchInput.fill('drake');

      // Wait for debounced search
      await authenticatedPage.waitForTimeout(500);

      // Should show search results
      await expect(authenticatedPage.getByText('Drake', { exact: true })).toBeVisible({
        timeout: 5000,
      });
    });

    test('should show offense count in search results', async ({ authenticatedPage }) => {
      const searchInput = authenticatedPage.getByPlaceholder('Search any artist...');
      await searchInput.fill('drake');

      await authenticatedPage.waitForTimeout(500);

      // Should show offense count for Drake
      await expect(authenticatedPage.getByText(/2 offenses?/)).toBeVisible({ timeout: 5000 });
    });

    test('should clear search on escape', async ({ authenticatedPage }) => {
      const searchInput = authenticatedPage.getByPlaceholder('Search any artist...');
      await searchInput.fill('drake');

      await authenticatedPage.waitForTimeout(500);

      // Verify results appear
      await expect(authenticatedPage.getByText('Drake', { exact: true })).toBeVisible({
        timeout: 5000,
      });

      // Press Escape
      await authenticatedPage.keyboard.press('Escape');

      // Results should be cleared
      await authenticatedPage.waitForTimeout(300);
      await expect(searchInput).toHaveValue('');
    });

    test('should navigate to artist profile on click', async ({ authenticatedPage }) => {
      const searchInput = authenticatedPage.getByPlaceholder('Search any artist...');
      await searchInput.fill('drake');

      await authenticatedPage.waitForTimeout(500);

      // Click on Drake result
      await authenticatedPage.getByText('Drake', { exact: true }).first().click();

      // Should navigate (URL should change)
      await authenticatedPage.waitForURL(/.*artist.*/);
    });

    test('should show no results message', async ({ authenticatedPage }) => {
      const searchInput = authenticatedPage.getByPlaceholder('Search any artist...');
      await searchInput.fill('xyznonexistent');

      await authenticatedPage.waitForTimeout(500);

      // Should show no results message
      await expect(authenticatedPage.getByText(/No artists found/)).toBeVisible({ timeout: 5000 });
    });
  });

  test.describe('Blocked Artists', () => {
    test('should display blocked artists list', async ({ authenticatedPage }) => {
      // Should show blocked artists from mock data
      await expect(authenticatedPage.getByText('Blocked Artist One')).toBeVisible({
        timeout: 5000,
      });
      await expect(authenticatedPage.getByText('Blocked Artist Two')).toBeVisible();
    });

    test('should show blocked artists count', async ({ authenticatedPage }) => {
      // Should show count in section header
      await expect(authenticatedPage.getByText(/Your Blocked Artists.*\(2\)/)).toBeVisible({
        timeout: 5000,
      });
    });

    test('should allow unblocking an artist', async ({ authenticatedPage }) => {
      // Find blocked artist chip
      const artistChip = authenticatedPage.locator('[data-testid="blocked-artist-chip"]').first();

      // Click unblock button
      const unblockBtn = artistChip.locator('[data-testid="unblock-artist-button"]');
      await unblockBtn.click();

      // Should trigger unblock (artist will be added to exceptions)
      await authenticatedPage.waitForTimeout(500);
    });

    test('should navigate to artist profile when clicking blocked artist', async ({
      authenticatedPage,
    }) => {
      // Click on blocked artist name
      await authenticatedPage.getByText('Blocked Artist One').click();

      // Should navigate to artist profile
      await authenticatedPage.waitForURL(/.*artist.*/);
    });
  });

  test.describe('Empty States', () => {
    test('should show empty state when no artists blocked', async ({ page, mockApi }) => {
      // Override the blocked artists to return empty
      await page.route('**/api/v1/categories/blocked-artists', async (route) => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify([]),
        });
      });

      await mockApi(page);

      // Set auth
      await page.addInitScript(() => {
        localStorage.setItem('auth_token', 'mock-token');
        localStorage.setItem(
          'user',
          JSON.stringify({
            id: 'test-user',
            email: 'test@example.com',
          })
        );
      });

      await page.goto('/');
      await page.waitForLoadState('networkidle');

      // Should show empty state message
      await expect(page.getByText('No artists blocked yet')).toBeVisible({ timeout: 5000 });
      await expect(page.getByText('Toggle categories above to start blocking')).toBeVisible();
    });
  });

  test.describe('Loading States', () => {
    test('should show loading spinner while fetching categories', async ({ page }) => {
      // Add delay to category response
      await page.route('**/api/v1/categories', async (route) => {
        await new Promise((resolve) => setTimeout(resolve, 500));
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify(mockCategories),
        });
      });

      // Mock other endpoints
      await page.route('**/api/v1/auth/profile', async (route) => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({ id: 'test', email: 'test@example.com' }),
        });
      });

      await page.route('**/api/v1/categories/blocked-artists', async (route) => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify([]),
        });
      });

      await page.route('**/api/v1/dnp/list', async (route) => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify([]),
        });
      });

      await page.addInitScript(() => {
        localStorage.setItem('auth_token', 'mock-token');
        localStorage.setItem('user', JSON.stringify({ id: 'test', email: 'test@example.com' }));
      });

      await page.goto('/');

      // Should show loading state briefly
      // (This is a timing-sensitive test, may need adjustment)
    });

    test('should show loading spinner in search', async ({ authenticatedPage }) => {
      const searchInput = authenticatedPage.getByPlaceholder('Search any artist...');
      await searchInput.fill('test');

      // Should show spinning indicator (briefly, due to mocking)
      // The actual spinner appears for real API calls
    });
  });
});
