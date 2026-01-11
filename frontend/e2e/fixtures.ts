import { test as base, expect, Page } from '@playwright/test';

/**
 * E2E Test Fixtures for No Drake in the House
 *
 * Provides mock API responses and authentication helpers
 */

// Extended test with authentication fixtures
export const test = base.extend<{
  authenticatedPage: Page;
  mockApi: (page: Page) => Promise<void>;
}>({
  // Provide an authenticated page
  authenticatedPage: async ({ page }, use) => {
    await mockApiRoutes(page);
    await mockAuthentication(page);
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    await use(page);
  },

  // Provide API mocking function
  mockApi: async ({}, use) => {
    await use(mockApiRoutes);
  },
});

export { expect };

// Mock user profile
export const mockUser = {
  id: 'test-user-123',
  email: 'test@example.com',
  display_name: 'Test User',
  created_at: '2024-01-01T00:00:00Z',
};

// Mock categories
export const mockCategories = [
  {
    id: 'domestic_violence',
    name: 'Domestic Violence',
    description: 'Artists with domestic violence allegations or convictions',
    artist_count: 15,
    subscribed: false,
  },
  {
    id: 'sexual_misconduct',
    name: 'Sexual Misconduct',
    description: 'Artists with sexual misconduct allegations',
    artist_count: 12,
    subscribed: true,
  },
  {
    id: 'hate_speech',
    name: 'Hate Speech',
    description: 'Artists who have engaged in hate speech',
    artist_count: 8,
    subscribed: false,
  },
];

// Mock blocked artists
export const mockBlockedArtists = [
  {
    id: 'artist-1',
    name: 'Blocked Artist One',
    category: 'sexual_misconduct',
    severity: 'severe',
  },
  {
    id: 'artist-2',
    name: 'Blocked Artist Two',
    category: 'sexual_misconduct',
    severity: 'moderate',
  },
];

// Mock search results
export const mockSearchResults = {
  artists: [
    {
      id: 'drake-123',
      canonical_name: 'Drake',
      genres: ['Hip-Hop', 'R&B'],
      has_offenses: true,
      offense_count: 2,
    },
    {
      id: 'drake-bell',
      canonical_name: 'Drake Bell',
      genres: ['Pop', 'Rock'],
      has_offenses: true,
      offense_count: 1,
    },
    {
      id: 'drake-white',
      canonical_name: 'Drake White',
      genres: ['Country'],
      has_offenses: false,
      offense_count: 0,
    },
  ],
  total: 3,
  sources: { local: 2, catalog: 1 },
};

// Mock connections
export const mockConnections = {
  spotify: {
    id: 'conn-spotify-123',
    platform: 'spotify',
    status: 'active',
    display_name: 'Test Spotify',
    connected_at: '2024-01-01T00:00:00Z',
  },
  apple_music: {
    id: 'conn-apple-123',
    platform: 'apple_music',
    status: 'active',
    display_name: 'Test Apple Music',
    connected_at: '2024-01-01T00:00:00Z',
  },
};

// Set up authentication state
async function mockAuthentication(page: Page) {
  // Set localStorage to simulate authenticated state
  await page.addInitScript(() => {
    localStorage.setItem('auth_token', 'mock-jwt-token-12345');
    localStorage.setItem('user', JSON.stringify({
      id: 'test-user-123',
      email: 'test@example.com',
      display_name: 'Test User',
    }));
  });
}

// Mock all API routes
async function mockApiRoutes(page: Page) {
  // Auth profile endpoint
  await page.route('**/api/v1/auth/profile', async (route) => {
    if (route.request().method() === 'GET') {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(mockUser),
      });
    }
  });

  // Login endpoint
  await page.route('**/api/v1/auth/login', async (route) => {
    const body = route.request().postDataJSON();
    if (body?.email === 'test@example.com' && body?.password === 'password123') {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          user: mockUser,
          token: 'mock-jwt-token-12345',
        }),
      });
    } else {
      await route.fulfill({
        status: 401,
        contentType: 'application/json',
        body: JSON.stringify({ error: 'Invalid credentials' }),
      });
    }
  });

  // Register endpoint
  await page.route('**/api/v1/auth/register', async (route) => {
    const body = route.request().postDataJSON();
    if (body?.email && body?.password) {
      if (body.email === 'existing@example.com') {
        await route.fulfill({
          status: 409,
          contentType: 'application/json',
          body: JSON.stringify({ error: 'Email already exists' }),
        });
      } else {
        await route.fulfill({
          status: 201,
          contentType: 'application/json',
          body: JSON.stringify({ success: true }),
        });
      }
    }
  });

  // Categories endpoints
  await page.route('**/api/v1/categories', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(mockCategories),
    });
  });

  await page.route('**/api/v1/categories/blocked-artists', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(mockBlockedArtists),
    });
  });

  await page.route('**/api/v1/categories/*/subscribe', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ success: true }),
    });
  });

  // DNP list endpoints
  await page.route('**/api/v1/dnp/list', async (route) => {
    if (route.request().method() === 'GET') {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify([
          { artist_id: 'artist-1' },
          { artist_id: 'artist-2' },
        ]),
      });
    }
  });

  await page.route('**/api/v1/dnp/list/*', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ success: true }),
    });
  });

  // Search endpoint
  await page.route('**/api/v1/dnp/search*', async (route) => {
    const url = new URL(route.request().url());
    const query = url.searchParams.get('q')?.toLowerCase() || '';

    const filteredResults = {
      ...mockSearchResults,
      artists: mockSearchResults.artists.filter((a) =>
        a.canonical_name.toLowerCase().includes(query)
      ),
    };
    filteredResults.total = filteredResults.artists.length;

    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(filteredResults),
    });
  });

  // Offense query endpoint
  await page.route('**/api/v1/offenses/query*', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        artists: mockBlockedArtists,
      }),
    });
  });

  // Connections endpoints
  await page.route('**/api/v1/connections', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify([mockConnections.spotify, mockConnections.apple_music]),
    });
  });

  await page.route('**/api/v1/connections/spotify', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(mockConnections.spotify),
    });
  });

  await page.route('**/api/v1/connections/apple-music', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(mockConnections.apple_music),
    });
  });

  // Enforcement endpoints
  await page.route('**/api/v1/enforcement/apple-music/preview', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        songs_to_dislike: 25,
        albums_to_dislike: 5,
        blocked_artists: mockBlockedArtists,
        estimated_time_seconds: 30,
      }),
    });
  });

  await page.route('**/api/v1/enforcement/apple-music/run', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        run_id: 'run-123',
        status: 'running',
        message: 'Enforcement started',
      }),
    });
  });

  await page.route('**/api/v1/enforcement/apple-music/status/*', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        run_id: 'run-123',
        status: 'completed',
        songs_disliked: 25,
        albums_disliked: 5,
        errors: 0,
      }),
    });
  });

  await page.route('**/api/v1/enforcement/apple-music/history', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify([
        {
          id: 'run-123',
          status: 'completed',
          started_at: '2024-01-01T12:00:00Z',
          completed_at: '2024-01-01T12:01:00Z',
          songs_disliked: 25,
          albums_disliked: 5,
          errors: 0,
        },
      ]),
    });
  });
}
