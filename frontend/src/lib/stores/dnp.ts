import { writable, derived } from 'svelte/store';

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
  ($dnp) => $dnp.entries.length
);

export const dnpTags = derived(
  dnpStore,
  ($dnp) => {
    const allTags = $dnp.entries.flatMap(entry => entry.tags);
    return [...new Set(allTags)].sort();
  }
);

// DNP actions
export const dnpActions = {
  fetchDnpList: async () => {
    dnpStore.update(state => ({ ...state, isLoading: true, error: null }));
    
    try {
      const token = localStorage.getItem('auth_token');
      const response = await fetch('http://localhost:3000/api/v1/dnp/list', {
        headers: {
          'Authorization': `Bearer ${token}`,
        },
      });

      const result = await response.json();
      
      if (result.success) {
        dnpStore.update(state => ({
          ...state,
          entries: result.data,
          isLoading: false,
        }));
      } else {
        dnpStore.update(state => ({
          ...state,
          error: result.message,
          isLoading: false,
        }));
      }
    } catch (error) {
      dnpStore.update(state => ({
        ...state,
        error: 'Failed to fetch DNP list',
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
    
    try {
      const token = localStorage.getItem('auth_token');
      const response = await fetch(`http://localhost:3000/api/v1/dnp/search?q=${encodeURIComponent(query)}&limit=${limit}`, {
        headers: {
          'Authorization': `Bearer ${token}`,
        },
      });

      const result = await response.json();
      
      if (result.success) {
        dnpStore.update(state => ({
          ...state,
          searchResults: result.data,
          isSearching: false,
        }));
      } else {
        dnpStore.update(state => ({
          ...state,
          error: result.message,
          isSearching: false,
        }));
      }
    } catch (error) {
      dnpStore.update(state => ({
        ...state,
        error: 'Artist search failed',
        isSearching: false,
      }));
    }
  },

  addArtist: async (artistQuery: string, tags: string[] = [], note?: string) => {
    try {
      const token = localStorage.getItem('auth_token');
      const response = await fetch('http://localhost:3000/api/v1/dnp/artists', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          query: artistQuery,
          tags,
          note,
        }),
      });

      const result = await response.json();
      
      if (result.success) {
        // Refresh the DNP list
        await dnpActions.fetchDnpList();
        return { success: true, data: result.data };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error) {
      return { success: false, message: 'Failed to add artist to DNP list' };
    }
  },

  removeArtist: async (artistId: string) => {
    try {
      const token = localStorage.getItem('auth_token');
      const response = await fetch(`http://localhost:3000/api/v1/dnp/artists/${artistId}`, {
        method: 'DELETE',
        headers: {
          'Authorization': `Bearer ${token}`,
        },
      });

      const result = await response.json();
      
      if (result.success) {
        // Refresh the DNP list
        await dnpActions.fetchDnpList();
        return { success: true };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error) {
      return { success: false, message: 'Failed to remove artist from DNP list' };
    }
  },

  updateEntry: async (artistId: string, tags: string[], note?: string) => {
    try {
      const token = localStorage.getItem('auth_token');
      const response = await fetch(`http://localhost:3000/api/v1/dnp/artists/${artistId}`, {
        method: 'PUT',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ tags, note }),
      });

      const result = await response.json();
      
      if (result.success) {
        // Refresh the DNP list
        await dnpActions.fetchDnpList();
        return { success: true, data: result.data };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error) {
      return { success: false, message: 'Failed to update DNP entry' };
    }
  },

  bulkImport: async (data: string, format: 'csv' | 'json') => {
    try {
      const token = localStorage.getItem('auth_token');
      const response = await fetch('http://localhost:3000/api/v1/dnp/import', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ data, format }),
      });

      const result = await response.json();
      
      if (result.success) {
        // Refresh the DNP list
        await dnpActions.fetchDnpList();
        return { success: true, data: result.data };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error) {
      return { success: false, message: 'Bulk import failed' };
    }
  },

  exportList: async (format: 'csv' | 'json' = 'json') => {
    try {
      const token = localStorage.getItem('auth_token');
      const response = await fetch(`http://localhost:3000/api/v1/dnp/export?format=${format}`, {
        headers: {
          'Authorization': `Bearer ${token}`,
        },
      });

      const result = await response.json();
      
      if (result.success) {
        return { success: true, data: result.data };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error) {
      return { success: false, message: 'Export failed' };
    }
  },

  clearSearch: () => {
    dnpStore.update(state => ({ ...state, searchResults: [] }));
  },
};