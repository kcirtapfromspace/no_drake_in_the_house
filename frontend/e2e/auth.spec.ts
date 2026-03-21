import { test, expect, mockUser } from './fixtures';

/**
 * Authentication E2E Tests
 *
 * Tests for login, registration, and authentication flows
 */

test.describe('Authentication', () => {
  test.describe('Login Flow', () => {
    test('should display login form by default', async ({ page, mockApi }) => {
      await mockApi(page);
      await page.goto('/');

      // Check for login form elements
      await expect(page.getByRole('heading', { name: 'No Drake in the House' })).toBeVisible();
      await expect(page.getByText('Clean your feed')).toBeVisible();
      await expect(page.getByLabel('Email')).toBeVisible();
      await expect(page.getByLabel('Password')).toBeVisible();
      await expect(page.getByRole('button', { name: 'Sign in' })).toBeVisible();
    });

    test('should show validation error for empty fields', async ({ page, mockApi }) => {
      await mockApi(page);
      await page.goto('/');

      // Try to submit with empty fields
      await page.getByRole('button', { name: 'Sign in' }).click();

      // Browser validation should prevent submission
      const emailInput = page.getByLabel('Email');
      await expect(emailInput).toHaveAttribute('required');
    });

    test('should show error for invalid credentials', async ({ page, mockApi }) => {
      await mockApi(page);
      await page.goto('/');

      // Enter invalid credentials
      await page.getByLabel('Email').fill('wrong@example.com');
      await page.getByLabel('Password').fill('wrongpassword');
      await page.getByRole('button', { name: 'Sign in' }).click();

      // Should show error message
      await expect(page.getByText('Invalid credentials')).toBeVisible({ timeout: 5000 });
    });

    test('should login successfully with valid credentials', async ({ page, mockApi }) => {
      await mockApi(page);
      await page.goto('/');

      // Enter valid credentials
      await page.getByLabel('Email').fill('test@example.com');
      await page.getByLabel('Password').fill('password123');
      await page.getByRole('button', { name: 'Sign in' }).click();

      // Should redirect to home page
      await expect(page.getByRole('heading', { name: 'Clean Your Feed' })).toBeVisible({
        timeout: 5000,
      });
    });

    test('should show loading state while logging in', async ({ page, mockApi }) => {
      await mockApi(page);

      // Slow down the login response so we can observe the loading state
      await page.route('**/api/v1/auth/login', async (route) => {
        await new Promise((resolve) => setTimeout(resolve, 500));
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({ user: { id: 'test', email: 'test@example.com' }, token: 'tok' }),
        });
      });

      await page.goto('/');

      await page.getByLabel('Email').fill('test@example.com');
      await page.getByLabel('Password').fill('password123');

      // Start login and check for loading state
      const submitButton = page.getByRole('button', { name: 'Sign in' });
      await submitButton.click();

      // Button should be disabled during loading
      await expect(submitButton).toBeDisabled();
    });
  });

  test.describe('Registration Flow', () => {
    test('should switch to registration mode', async ({ page, mockApi }) => {
      await mockApi(page);
      await page.goto('/');

      // Click sign up link
      await page.getByRole('button', { name: 'Sign up' }).click();

      // Should show registration form
      await expect(page.getByRole('button', { name: 'Create account' })).toBeVisible();
      await expect(page.getByLabel('Confirm Password')).toBeVisible();
    });

    test('should validate password confirmation', async ({ page, mockApi }) => {
      await mockApi(page);
      await page.goto('/');

      // Switch to registration mode
      await page.getByRole('button', { name: 'Sign up' }).click();

      // Enter mismatched passwords
      await page.getByLabel('Email').fill('new@example.com');
      await page.getByLabel('Password', { exact: true }).fill('password123');
      await page.getByLabel('Confirm Password').fill('different123');
      await page.getByRole('button', { name: 'Create account' }).click();

      // Should show error
      await expect(page.getByText('Passwords do not match')).toBeVisible();
    });

    test('should register successfully', async ({ page, mockApi }) => {
      await mockApi(page);
      await page.goto('/');

      // Switch to registration mode
      await page.getByRole('button', { name: 'Sign up' }).click();

      // Fill registration form
      await page.getByLabel('Email').fill('new@example.com');
      await page.getByLabel('Password', { exact: true }).fill('password123');
      await page.getByLabel('Confirm Password').fill('password123');
      await page.getByRole('button', { name: 'Create account' }).click();

      // Should show success message
      await expect(page.getByText('Account created!')).toBeVisible({ timeout: 5000 });

      // Should switch back to login mode
      await expect(page.getByRole('button', { name: 'Sign in' })).toBeVisible();
    });

    test('should show error for existing email', async ({ page, mockApi }) => {
      await mockApi(page);
      await page.goto('/');

      // Switch to registration mode
      await page.getByRole('button', { name: 'Sign up' }).click();

      // Try to register with existing email
      await page.getByLabel('Email').fill('existing@example.com');
      await page.getByLabel('Password', { exact: true }).fill('password123');
      await page.getByLabel('Confirm Password').fill('password123');
      await page.getByRole('button', { name: 'Create account' }).click();

      // Should show error
      await expect(page.getByText(/already exists|Registration failed/)).toBeVisible({
        timeout: 5000,
      });
    });

    test('should switch between login and register modes', async ({ page, mockApi }) => {
      await mockApi(page);
      await page.goto('/');

      // Start on login
      await expect(page.getByRole('button', { name: 'Sign in' })).toBeVisible();

      // Switch to register
      await page.getByRole('button', { name: 'Sign up' }).click();
      await expect(page.getByRole('button', { name: 'Create account' })).toBeVisible();

      // Switch back to login
      await page.getByRole('button', { name: 'Sign in' }).last().click();
      await expect(page.getByRole('button', { name: 'Sign in' })).toBeVisible();
    });
  });

  test.describe('Features Display', () => {
    test('should display feature list on login page', async ({ page, mockApi }) => {
      await mockApi(page);
      await page.goto('/');

      // Check for feature highlights (may be scrollable on smaller viewports)
      await expect(page.getByText('Evidence-led artist blocklists')).toBeAttached();
      await expect(page.getByText('Spotify + Apple Music').first()).toBeAttached();
      await expect(page.getByText('Features and collaborations included')).toBeAttached();
    });
  });

  test.describe('Authenticated State', () => {
    test('should show home page when already authenticated', async ({ authenticatedPage }) => {
      // authenticatedPage fixture already handles authentication
      await expect(authenticatedPage.getByRole('heading', { name: 'Clean Your Feed' })).toBeVisible();
    });

    test('should not show login form when authenticated', async ({ authenticatedPage }) => {
      await expect(authenticatedPage.getByLabel('Email')).not.toBeVisible();
      await expect(authenticatedPage.getByLabel('Password')).not.toBeVisible();
    });
  });
});
