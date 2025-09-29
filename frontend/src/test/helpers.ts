import { vi } from 'vitest';
import type { MockedFunction } from 'vitest';

// Mock API responses
export const mockApiResponse = (data: any, status = 200) => {
  return Promise.resolve({
    ok: status >= 200 && status < 300,
    status,
    json: () => Promise.resolve(data),
    text: () => Promise.resolve(JSON.stringify(data)),
  } as Response);
};

// Mock fetch with specific responses
export const mockFetch = (responses: Array<{ url?: string; response: any; status?: number }>) => {
  const fetchMock = vi.fn() as MockedFunction<typeof fetch>;
  
  responses.forEach(({ url, response, status = 200 }) => {
    if (url) {
      fetchMock.mockImplementationOnce((input) => {
        if (typeof input === 'string' && input.includes(url)) {
          return mockApiResponse(response, status);
        }
        return mockApiResponse({ error: 'Not found' }, 404);
      });
    } else {
      fetchMock.mockResolvedValueOnce(mockApiResponse(response, status) as any);
    }
  });
  
  globalThis.fetch = fetchMock;
  return fetchMock;
};

// Test data factories
export const createMockUser = (overrides = {}) => ({
  id: 'test-user-id',
  email: 'test@example.com',
  emailVerified: false,
  totpEnabled: false,
  createdAt: '2024-01-01T00:00:00Z',
  updatedAt: '2024-01-01T00:00:00Z',
  settings: {
    theme: 'light',
    notificationsEnabled: true,
    autoEnforcement: false,
    preferredPlatforms: ['spotify'],
    privacySettings: {},
  },
  ...overrides,
});

export const createMockArtist = (overrides = {}) => ({
  id: 'test-artist-id',
  canonicalName: 'Test Artist',
  externalIds: {
    spotify: 'spotify_123',
    appleMusic: 'apple_456',
  },
  metadata: {
    genres: ['rock', 'pop'],
    image: 'https://example.com/artist.jpg',
  },
  createdAt: '2024-01-01T00:00:00Z',
  ...overrides,
});

export const createMockDnpEntry = (overrides = {}) => ({
  artist: createMockArtist(),
  tags: ['test'],
  note: 'Test note',
  created_at: '2024-01-01T00:00:00Z',
  ...overrides,
});

export const createMockAuthResponse = (overrides = {}) => ({
  user: createMockUser(),
  accessToken: 'mock-access-token',
  refreshToken: 'mock-refresh-token',
  tokenType: 'Bearer',
  expiresIn: 86400,
  ...overrides,
});

// Store helpers for testing
export const createMockAuthStore = () => {
  const store = {
    subscribe: vi.fn(),
    set: vi.fn(),
    update: vi.fn(),
    login: vi.fn(),
    logout: vi.fn(),
    register: vi.fn(),
    refreshToken: vi.fn(),
  };
  
  // Mock the store value
  store.subscribe.mockImplementation((callback) => {
    callback({
      user: null,
      token: null,
      isAuthenticated: false,
      isLoading: false,
      error: null,
    });
    return () => {}; // Unsubscribe function
  });
  
  // Mock successful operations
  store.register.mockResolvedValue(createMockAuthResponse());
  store.login.mockResolvedValue(createMockAuthResponse());
  
  return store;
};

export const createMockDnpStore = () => {
  const store = {
    subscribe: vi.fn(),
    set: vi.fn(),
    update: vi.fn(),
  };
  
  store.subscribe.mockImplementation((callback) => {
    callback({
      entries: [],
      searchResults: [],
      isLoading: false,
      error: null,
      isSearching: false,
    });
    return () => {};
  });
  
  return store;
};

export const createMockDnpActions = () => ({
  fetchDnpList: vi.fn(),
  searchArtists: vi.fn(),
  addArtist: vi.fn(),
  removeArtist: vi.fn(),
  updateEntry: vi.fn(),
  bulkImport: vi.fn(),
  exportList: vi.fn(),
  clearSearch: vi.fn(),
});

// Component testing utilities
export const waitForElement = async (getByTestId: (id: string) => HTMLElement, testId: string, timeout = 1000) => {
  const start = Date.now();
  while (Date.now() - start < timeout) {
    try {
      return getByTestId(testId);
    } catch {
      await new Promise(resolve => setTimeout(resolve, 10));
    }
  }
  throw new Error(`Element with test-id "${testId}" not found within ${timeout}ms`);
};

// Form testing helpers
export const fillForm = async (container: HTMLElement, fields: Record<string, string>) => {
  for (const [name, value] of Object.entries(fields)) {
    const input = container.querySelector(`input[name="${name}"]`) as HTMLInputElement;
    if (input) {
      input.value = value;
      input.dispatchEvent(new Event('input', { bubbles: true }));
    }
  }
};

export const submitForm = (container: HTMLElement) => {
  const form = container.querySelector('form');
  if (form) {
    form.dispatchEvent(new Event('submit', { bubbles: true }));
  }
};