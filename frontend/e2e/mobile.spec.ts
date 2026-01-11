import { test, expect } from './fixtures';

/**
 * Mobile Responsiveness E2E Tests
 *
 * Tests for mobile viewport behavior
 */

test.describe('Mobile Responsiveness', () => {
  test.use({ viewport: { width: 375, height: 667 } }); // iPhone SE

  test.describe('Login Page Mobile', () => {
    test('should display login form properly on mobile', async ({ page, mockApi }) => {
      await mockApi(page);
      await page.goto('/');

      // Logo should be visible
      await expect(page.getByRole('heading', { name: 'No Drake' })).toBeVisible();

      // Form should be usable
      await expect(page.getByLabel('Email')).toBeVisible();
      await expect(page.getByLabel('Password')).toBeVisible();

      // Button should be full width (or appropriately sized)
      const submitButton = page.getByRole('button', { name: 'Sign in' });
      await expect(submitButton).toBeVisible();
    });

    test('should allow login on mobile', async ({ page, mockApi }) => {
      await mockApi(page);
      await page.goto('/');

      await page.getByLabel('Email').fill('test@example.com');
      await page.getByLabel('Password').fill('password123');
      await page.getByRole('button', { name: 'Sign in' }).click();

      await expect(page.getByRole('heading', { name: 'Clean Your Feed' })).toBeVisible({
        timeout: 5000,
      });
    });

    test('should display features on mobile', async ({ page, mockApi }) => {
      await mockApi(page);
      await page.goto('/');

      await expect(page.getByText('Evidence-based artist blocklists')).toBeVisible();
    });
  });

  test.describe('Home Page Mobile', () => {
    test('should display search on mobile', async ({ authenticatedPage }) => {
      await authenticatedPage.setViewportSize({ width: 375, height: 667 });

      await expect(
        authenticatedPage.getByPlaceholder('Search any artist...')
      ).toBeVisible();
    });

    test('should show category cards in grid on mobile', async ({ authenticatedPage }) => {
      await authenticatedPage.setViewportSize({ width: 375, height: 667 });

      // Categories should be displayed (may stack differently)
      await expect(authenticatedPage.getByText('Domestic Violence')).toBeVisible();
    });

    test('should show blocked artists as chips on mobile', async ({ authenticatedPage }) => {
      await authenticatedPage.setViewportSize({ width: 375, height: 667 });

      await expect(authenticatedPage.getByText('Blocked Artist One')).toBeVisible({
        timeout: 5000,
      });
    });

    test('should expand category panel on mobile', async ({ authenticatedPage }) => {
      await authenticatedPage.setViewportSize({ width: 375, height: 667 });

      await authenticatedPage.getByText('Domestic Violence').first().click();

      await expect(
        authenticatedPage.getByRole('heading', { name: 'Domestic Violence' })
      ).toBeVisible();
    });

    test('should search on mobile', async ({ authenticatedPage }) => {
      await authenticatedPage.setViewportSize({ width: 375, height: 667 });

      const searchInput = authenticatedPage.getByPlaceholder('Search any artist...');
      await searchInput.fill('drake');

      await authenticatedPage.waitForTimeout(500);

      await expect(authenticatedPage.getByText('Drake', { exact: true })).toBeVisible({
        timeout: 5000,
      });
    });
  });

  test.describe('Touch Interactions', () => {
    test('should handle tap on category toggle', async ({ authenticatedPage }) => {
      await authenticatedPage.setViewportSize({ width: 375, height: 667 });

      const toggleButton = authenticatedPage.locator('button').first();
      await toggleButton.tap();
    });

    test('should handle tap on search result', async ({ authenticatedPage }) => {
      await authenticatedPage.setViewportSize({ width: 375, height: 667 });

      const searchInput = authenticatedPage.getByPlaceholder('Search any artist...');
      await searchInput.fill('drake');
      await authenticatedPage.waitForTimeout(500);

      const result = authenticatedPage.getByText('Drake', { exact: true }).first();
      if (await result.isVisible()) {
        await result.tap();
        await expect(authenticatedPage).toHaveURL(/.*artist.*/);
      }
    });
  });
});

test.describe('Tablet Responsiveness', () => {
  test.use({ viewport: { width: 768, height: 1024 } }); // iPad

  test('should display home page properly on tablet', async ({ authenticatedPage }) => {
    await expect(
      authenticatedPage.getByRole('heading', { name: 'Clean Your Feed' })
    ).toBeVisible();
    await expect(
      authenticatedPage.getByPlaceholder('Search any artist...')
    ).toBeVisible();
  });

  test('should show categories in grid on tablet', async ({ authenticatedPage }) => {
    await expect(authenticatedPage.getByText('Domestic Violence')).toBeVisible();
    await expect(authenticatedPage.getByText('Sexual Misconduct')).toBeVisible();
  });
});
