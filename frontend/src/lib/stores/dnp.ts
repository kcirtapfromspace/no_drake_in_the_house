import { writable, derived } from 'svelte/store';
import { apiClient, type ApiResponse } from '../utils/api-client';

export interface Artist {
  id: string;
  canonical_name: string;
  external_ids: {
    spotify?: string;
    apple?: string;
    musicbrainz?: string;
    isni?: string;
  };
  metadata: {
    image?: string;
    genres?: string[];
  };
}

export interface DnpEntry {
  artist: Artist;
  tags: string[];
  note?: string;
  created_at: string;
}

export interface DnpState {
  entries: DnpEntry[];
  isLoading: boolean;
  error: string | null;
  searchResults: Artist[];
  isSearching: boolean;
}

const initialState: DnpState = {
  entries: [],
  isLoading: false,
  error: null,
  searchResults: [],
  isSearching: false,
};

export const dnpStore = writable<DnpState>(initialState);

export const dnpArtists = derived(
  dnpStore,
  ($dnp) => $dnp.entries.map(entry => entry.artist)
);

export const dnpCount = derived(
  dnpStore,
  ($dnp) => ($dnp.entries && Array.isArray($dnp.entries)) ? $dnp.entries.length : 0
);

export const dnpTags = derived(
  dnpStore,
  ($dnp) => {
    if (!$dnp.entries || !Array.isArray($dnp.entries)) {
      return [];
    }
    const allTags = $dnp.entries.flatMap(entry => entry.tags || []);
    return [...new Set(allTags)].sort();
  }
);

// DNP actions
export const dnpActions = {
  fetchDnpList: async () => {
    dnpStore.update(state => ({ ...state, isLoading: true, error: null }));
    
    const response = await apiClient.authenticatedRequest<DnpEntry[]>(
      'GET',
      '/api/v1/dnp/list'
    );
    
    if (response.success && response.data) {
      // Ensure data is an array
      const entries = Array.isArray(response.data) ? response.data : [];
      dnpStore.update(state => ({
        ...state,
        entries,
        isLoading: false,
      }));
    } else {
      dnpStore.update(state => ({
        ...state,
        entries: [], // Reset to empty array on error
        error: response.message || 'Failed to fetch DNP list',
        isLoading: false,
      }));
    }
  },

  searchArtists: async (query: string, limit = 10) => {
    if (!query.trim()) {
      dnpStore.update(state => ({ ...state, searchResults: [] }));
      return;
    }

    dnpStore.update(state => ({ ...state, isSearching: true }));
    
    const response = await apiClient.authenticatedRequest<Artist[]>(
      'GET',
      `/api/v1/dnp/search?q=${encodeURIComponent(query)}&limit=${limit}`
    );
    
    if (response.success && response.data) {
      dnpStore.update(state => ({
        ...state,
        searchResults: response.data || [],
        isSearching: false,
      }));
    } else {
      dnpStore.update(state => ({
        ...state,
        error: response.message || 'Artist search failed',
        isSearching: false,
      }));
    }
  },

  addArtist: async (artistQuery: string, tags: string[] = [], note?: string) => {
    const response = await apiClient.authenticatedRequest<DnpEntry>(
      'POST',
      '/api/v1/dnp/list',
      {
        query: artistQuery,
        tags,
        note,
      }
    );
    
    if (response.success) {
      // Refresh the DNP list
      await dnpActions.fetchDnpList();
      return { success: true, data: response.data };
    } else {
      return { success: false, message: response.message || 'Failed to add artist to DNP list' };
    }
  },

  removeArtist: async (artistId: string) => {
    const response = await apiClient.authenticatedRequest<any>(
      'DELETE',
      `/api/v1/dnp/list/${artistId}`
    );
    
    if (response.success) {
      // Refresh the DNP list
      await dnpActions.fetchDnpList();
      return { success: true };
    } else {
      return { success: false, message: response.message || 'Failed to remove artist from DNP list' };
    }
  },

  updateEntry: async (artistId: string, tags: string[], note?: string) => {
    const response = await apiClient.authenticatedRequest<DnpEntry>(
      'PUT',
      `/api/v1/dnp/list/${artistId}`,
      { tags, note }
    );
    
    if (response.success) {
      // Refresh the DNP list
      await dnpActions.fetchDnpList();
      return { success: true, data: response.data };
    } else {
      return { success: false, message: response.message || 'Failed to update DNP entry' };
    }
  },

  bulkImport: async (data: string, format: 'csv' | 'json') => {
    const response = await apiClient.authenticatedRequest<any>(
      'POST',
      '/api/v1/dnp/import',
      { data, format }
    );
    
    if (response.success) {
      // Refresh the DNP list
      await dnpActions.fetchDnpList();
      return { success: true, data: response.data };
    } else {
      return { success: false, message: response.message || 'Bulk import failed' };
    }
  },

  exportList: async (format: 'csv' | 'json' = 'json') => {
    const response = await apiClient.authenticatedRequest<any>(
      'GET',
      `/api/v1/dnp/export?format=${format}`
    );
    
    if (response.success) {
      return { success: true, data: response.data };
    } else {
      return { success: false, message: response.message || 'Export failed' };
    }
  },

  clearSearch: () => {
    dnpStore.update(state => ({ ...state, searchResults: [] }));
  },
};