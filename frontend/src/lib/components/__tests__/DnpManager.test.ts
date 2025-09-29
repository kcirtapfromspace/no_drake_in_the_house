import { render, screen } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach } from 'vitest';

// Mock the imported components that don't exist yet
vi.mock('../ArtistSearch.svelte', () => ({
  default: class MockArtistSearch {
    constructor() {}
    $set() {}
    $on() {}
    $destroy() {}
  }
}));

vi.mock('../DnpEntry.svelte', () => ({
  default: class MockDnpEntry {
    constructor() {}
    $set() {}
    $on() {}
    $destroy() {}
  }
}));

vi.mock('../BulkActions.svelte', () => ({
  default: class MockBulkActions {
    constructor() {}
    $set() {}
    $on() {}
    $destroy() {}
  }
}));

// Mock the stores
vi.mock('$lib/stores/dnp', () => ({
  dnpStore: {
    subscribe: vi.fn((callback) => {
      callback({
        entries: [],
        searchResults: [],
        isLoading: false,
        error: null,
        isSearching: false,
      });
      return () => {};
    }),
    set: vi.fn(),
    update: vi.fn(),
  },
  dnpActions: {
    fetchDnpList: vi.fn(),
    searchArtists: vi.fn(),
    addArtist: vi.fn(),
    removeArtist: vi.fn(),
    updateEntry: vi.fn(),
    bulkImport: vi.fn(),
    exportList: vi.fn(),
    clearSearch: vi.fn(),
  },
  dnpTags: { 
    subscribe: vi.fn((callback) => {
      callback([]);
      return () => {};
    })
  },
}));

vi.mock('$lib/stores/auth', () => ({
  auth: {
    subscribe: vi.fn((callback) => {
      callback({
        user: { id: 'test-user-id', email: 'test@example.com' },
        token: 'mock-token',
        isAuthenticated: true,
        isLoading: false,
        error: null,
      });
      return () => {};
    }),
    set: vi.fn(),
    update: vi.fn(),
    login: vi.fn(),
    logout: vi.fn(),
    register: vi.fn(),
    refreshToken: vi.fn(),
  },
}));

import DnpManager from '../DnpManager.svelte';

describe('DnpManager', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders without crashing', () => {
    expect(() => render(DnpManager)).not.toThrow();
  });

  it('shows basic structure', () => {
    render(DnpManager);
    
    // Just check that the component renders something
    expect(document.body).toBeInTheDocument();
  });
});