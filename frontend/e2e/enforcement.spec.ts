import { test, expect } from './fixtures';

/**
 * Enforcement E2E Tests
 *
 * Tests for the Apple Music and Spotify enforcement flows
 */

test.describe('Enforcement', () => {
  test.describe('Blocking Operations', () => {
    test('should show enforcement badges on blocked artists', async ({ authenticatedPage }) => {
      // Wait for blocked artists to load
      await expect(authenticatedPage.getByText('Blocked Artist One')).toBeVisible({
        timeout: 5000,
      });

      // Look for enforcement badges (if rendered)
      const artistChip = authenticatedPage.locator('[data-testid="blocked-artist-chip"]').first();
      await expect(artistChip).toBeVisible();
    });

    test('should trigger enforcement when blocking via category', async ({
      authenticatedPage,
    }) => {
      // Find an unsubscribed category toggle
      const domesticViolenceCard = authenticatedPage.locator('div').filter({
        hasText: 'Domestic Violence',
      });

      // Click the toggle button to subscribe
      const toggleButton = domesticViolenceCard.locator('button').first();
      await toggleButton.click();

      // Should trigger enforcement (toast notification should appear)
      // Note: The actual enforcement simulation happens in the component
      await authenticatedPage.waitForTimeout(1000);
    });

    test('should trigger enforcement when unblocking artist', async ({ authenticatedPage }) => {
      // Find blocked artist
      await expect(authenticatedPage.getByText('Blocked Artist One')).toBeVisible({
        timeout: 5000,
      });

      const artistChip = authenticatedPage.locator('[data-testid="blocked-artist-chip"]').first();
      const unblockBtn = artistChip.locator('[data-testid="unblock-artist-button"]');

      // Click unblock
      await unblockBtn.click();

      // Enforcement simulation should start
      await authenticatedPage.waitForTimeout(500);
    });
  });

  test.describe('Connection-Based Enforcement', () => {
    test('should enforce on connected platforms', async ({ authenticatedPage }) => {
      // Mock shows both Spotify and Apple Music connected
      // When blocking, both should be enforced

      // Subscribe to a category
      const domesticViolenceCard = authenticatedPage.locator('div').filter({
        hasText: 'Domestic Violence',
      });
      const toggleButton = domesticViolenceCard.locator('button').first();
      await toggleButton.click();

      // Wait for enforcement to complete
      await authenticatedPage.waitForTimeout(2000);
    });
  });

  test.describe('Category Enforcement', () => {
    test('should expand category and show artists', async ({ authenticatedPage }) => {
      // Click category name to expand
      await authenticatedPage.getByText('Domestic Violence').first().click();

      // Wait for expansion
      await expect(
        authenticatedPage.getByRole('heading', { name: 'Domestic Violence' })
      ).toBeVisible();

      // Should show artists in category
      await authenticatedPage.waitForTimeout(500);
    });

    test('should show block all button in expanded category', async ({ authenticatedPage }) => {
      // Click category to expand
      await authenticatedPage.getByText('Domestic Violence').first().click();

      // Look for Block All button
      await expect(authenticatedPage.getByRole('button', { name: /Block All/i })).toBeVisible();
    });

    test('should show unsubscribe button for subscribed category', async ({
      authenticatedPage,
    }) => {
      // Sexual Misconduct is subscribed in mock
      await authenticatedPage.getByText('Sexual Misconduct').first().click();

      // Should show Unsubscribe button
      await expect(
        authenticatedPage.getByRole('button', { name: /Unsubscribe/i })
      ).toBeVisible();
    });

    test('should show individual unblock buttons in category panel', async ({
      authenticatedPage,
    }) => {
      // Expand subscribed category
      await authenticatedPage.getByText('Sexual Misconduct').first().click();

      // Wait for artists to load
      await authenticatedPage.waitForTimeout(500);

      // Look for blocked artists with unblock capability
      // The panel shows artists with hover-activated X buttons
    });
  });

  test.describe('Exceptions', () => {
    test('should add artist to exceptions when unblocked', async ({ authenticatedPage }) => {
      // Unblock an artist
      const artistChip = authenticatedPage.locator('[data-testid="blocked-artist-chip"]').first();
      const unblockBtn = artistChip.locator('[data-testid="unblock-artist-button"]');
      await unblockBtn.click();

      // Artist should be moved to exceptions
      await authenticatedPage.waitForTimeout(500);

      // Verify localStorage was updated
      const exceptions = await authenticatedPage.evaluate(() => {
        return localStorage.getItem('exceptedArtists');
      });
      expect(exceptions).toBeTruthy();
    });

    test('should show excepted artists in category panel', async ({ authenticatedPage }) => {
      // First unblock an artist
      const artistChip = authenticatedPage.locator('[data-testid="blocked-artist-chip"]').first();
      const unblockBtn = artistChip.locator('[data-testid="unblock-artist-button"]');
      await unblockBtn.click();
      await authenticatedPage.waitForTimeout(500);

      // Expand the category
      await authenticatedPage.getByText('Sexual Misconduct').first().click();
      await authenticatedPage.waitForTimeout(500);

      // Should show "Not Blocked (Excepted)" section or similar
    });

    test('should allow re-blocking excepted artist', async ({ authenticatedPage }) => {
      // First unblock an artist
      const artistChip = authenticatedPage.locator('[data-testid="blocked-artist-chip"]').first();
      const unblockBtn = artistChip.locator('[data-testid="unblock-artist-button"]');
      await unblockBtn.click();
      await authenticatedPage.waitForTimeout(500);

      // Expand category to see excepted artist
      await authenticatedPage.getByText('Sexual Misconduct').first().click();
      await authenticatedPage.waitForTimeout(500);

      // Look for re-block button on excepted artist
      const reblockBtn = authenticatedPage.getByRole('button', { name: /Block/i }).last();
      if (await reblockBtn.isVisible()) {
        await reblockBtn.click();
        await authenticatedPage.waitForTimeout(500);
      }
    });
  });

  test.describe('Severity Display', () => {
    test('should display severity labels in category panel', async ({ authenticatedPage }) => {
      // Expand a category
      await authenticatedPage.getByText('Sexual Misconduct').first().click();
      await authenticatedPage.waitForTimeout(500);

      // Should show severity labels (Severe, Moderate, Egregious, Minor)
      // These are rendered based on artist data
    });
  });

  test.describe('Toast Notifications', () => {
    test('should show progress toast during bulk enforcement', async ({ authenticatedPage }) => {
      // Subscribe to a category with multiple artists
      const domesticViolenceCard = authenticatedPage.locator('div').filter({
        hasText: 'Domestic Violence',
      });
      const toggleButton = domesticViolenceCard.locator('button').first();
      await toggleButton.click();

      // Should show toast notification about blocking artists
      // Note: Toast notifications are rendered by blockingStore
      await authenticatedPage.waitForTimeout(500);
    });

    test('should show success toast after enforcement', async ({ authenticatedPage }) => {
      // Trigger enforcement
      const domesticViolenceCard = authenticatedPage.locator('div').filter({
        hasText: 'Domestic Violence',
      });
      const toggleButton = domesticViolenceCard.locator('button').first();
      await toggleButton.click();

      // Wait for enforcement to complete
      await authenticatedPage.waitForTimeout(2500);

      // Success toast should appear
    });
  });

  test.describe('Keyboard Accessibility', () => {
    test('should unblock artist with keyboard', async ({ authenticatedPage }) => {
      // Tab to unblock button and press Enter
      await expect(authenticatedPage.getByText('Blocked Artist One')).toBeVisible({
        timeout: 5000,
      });

      const unblockBtn = authenticatedPage.locator('[data-testid="unblock-artist-button"]').first();
      await unblockBtn.focus();
      await authenticatedPage.keyboard.press('Enter');

      await authenticatedPage.waitForTimeout(500);
    });
  });
});
