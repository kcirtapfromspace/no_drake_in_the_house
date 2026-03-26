import { writable, derived } from 'svelte/store';
import { apiClient } from '../utils/api-client';

// ---- Types ----

export interface PlaylistSummary {
  id: string;
  provider: string;
  provider_playlist_id: string;
  name: string;
  /** @deprecated use `name` — kept for backward compat */
  playlist_name: string;
  description?: string;
  image_url?: string;
  owner_name?: string;
  is_public?: boolean;
  source_type: string;
  total_tracks: number;
  provider_track_count?: number;
  tracks_out_of_sync?: boolean;
  flagged_tracks: number;
  clean_ratio: number;
  grade: string;
  unique_artists: number;
  flagged_artists: string[];
  last_synced: string;
  cover_images: string[];
}

export type TrackStatus = 'clean' | 'flagged' | 'blocked';

export interface PlaylistTrack {
  id: string;
  position: number;
  provider_track_id: string;
  track_name: string;
  album_name?: string;
  artist_id?: string;
  artist_name: string;
  artist_image_url?: string;
  added_at?: string;
  status: TrackStatus;
}

export type BrowserView = 'grid' | 'detail';

export interface PlaylistBrowserState {
  playlists: PlaylistSummary[];
  selectedPlaylist: PlaylistSummary | null;
  tracks: PlaylistTrack[];
  view: BrowserView;
  providerFilter: string;
  searchQuery: string;
  isLoadingPlaylists: boolean;
  isLoadingTracks: boolean;
  error: string | null;
  selectedPlaylistIds: Set<string>;
  selectionMode: boolean;
}

const initialState: PlaylistBrowserState = {
  playlists: [],
  selectedPlaylist: null,
  tracks: [],
  view: 'grid',
  providerFilter: '',
  searchQuery: '',
  isLoadingPlaylists: false,
  isLoadingTracks: false,
  error: null,
  selectedPlaylistIds: new Set(),
  selectionMode: false,
};

// ---- Store ----

export const playlistBrowserStore = writable<PlaylistBrowserState>(initialState);

// ---- Derived Stores ----

export const filteredPlaylists = derived(
  playlistBrowserStore,
  ($s) => {
    let result = $s.playlists;
    if ($s.providerFilter) {
      result = result.filter((p) => p.provider === $s.providerFilter);
    }
    if ($s.searchQuery) {
      const q = $s.searchQuery.toLowerCase();
      result = result.filter((p) => {
        const name = p.name || p.playlist_name || '';
        return name.toLowerCase().includes(q);
      });
    }
    return result;
  }
);

export const trackStats = derived(
  playlistBrowserStore,
  ($s) => {
    const stats = { total: 0, clean: 0, flagged: 0, blocked: 0 };
    for (const t of $s.tracks) {
      stats.total++;
      stats[t.status]++;
    }
    return stats;
  }
);

export const selectedPlaylistCount = derived(
  playlistBrowserStore,
  ($s) => $s.selectedPlaylistIds.size
);

export const allFilteredSelected = derived(
  [playlistBrowserStore, filteredPlaylists],
  ([$s, $filtered]) =>
    $filtered.length > 0 && $filtered.every((p) => $s.selectedPlaylistIds.has(p.id))
);

export const selectedPlaylists = derived(
  playlistBrowserStore,
  ($s) => $s.playlists.filter((p) => $s.selectedPlaylistIds.has(p.id))
);

// ---- Actions ----

export const playlistBrowserActions = {
  fetchPlaylists: async (provider?: string) => {
    playlistBrowserStore.update((s) => ({
      ...s,
      isLoadingPlaylists: true,
      error: null,
    }));

    try {
      const params = provider ? `?provider=${encodeURIComponent(provider)}` : '';
      const response = await apiClient.get<{ playlists: PlaylistSummary[]; total: number }>(
        `/api/v1/library/playlists${params}`
      );

      if (response.success && response.data) {
        // Normalize: ensure playlist_name is populated for backward compat
        const playlists = (response.data.playlists ?? []).map((p) => ({
          ...p,
          playlist_name: p.playlist_name || p.name,
        }));
        playlistBrowserStore.update((s) => ({
          ...s,
          playlists,
          isLoadingPlaylists: false,
        }));
      } else {
        playlistBrowserStore.update((s) => ({
          ...s,
          error: response.message || 'Failed to fetch playlists',
          isLoadingPlaylists: false,
        }));
      }
    } catch (error) {
      const msg = error instanceof Error ? error.message : 'Failed to fetch playlists';
      playlistBrowserStore.update((s) => ({
        ...s,
        error: msg,
        isLoadingPlaylists: false,
      }));
    }
  },

  selectPlaylist: async (playlist: PlaylistSummary) => {
    playlistBrowserStore.update((s) => ({
      ...s,
      selectedPlaylist: playlist,
      view: 'detail',
      isLoadingTracks: true,
      error: null,
    }));

    try {
      let response;

      // Prefer ID-based lookup (normalized), fall back to name-based (legacy)
      if (playlist.id) {
        response = await apiClient.get<{ tracks: PlaylistTrack[]; total: number }>(
          `/api/v1/library/playlists/${playlist.id}/tracks`
        );
      } else {
        const params = new URLSearchParams({
          provider: playlist.provider,
          playlistName: playlist.playlist_name || playlist.name,
        });
        response = await apiClient.get<{ tracks: PlaylistTrack[]; total: number }>(
          `/api/v1/library/playlists/tracks?${params}`
        );
      }

      if (response.success && response.data) {
        playlistBrowserStore.update((s) => ({
          ...s,
          tracks: response.data!.tracks ?? [],
          isLoadingTracks: false,
        }));
      } else {
        playlistBrowserStore.update((s) => ({
          ...s,
          error: response.message || 'Failed to fetch tracks',
          isLoadingTracks: false,
        }));
      }
    } catch (error) {
      const msg = error instanceof Error ? error.message : 'Failed to fetch tracks';
      playlistBrowserStore.update((s) => ({
        ...s,
        error: msg,
        isLoadingTracks: false,
      }));
    }
  },

  backToGrid: () => {
    playlistBrowserStore.update((s) => ({
      ...s,
      selectedPlaylist: null,
      tracks: [],
      view: 'grid',
    }));
  },

  setProviderFilter: (provider: string) => {
    playlistBrowserStore.update((s) => ({
      ...s,
      providerFilter: provider,
    }));
  },

  setSearchQuery: (query: string) => {
    playlistBrowserStore.update((s) => ({
      ...s,
      searchQuery: query,
    }));
  },

  clearError: () => {
    playlistBrowserStore.update((s) => ({ ...s, error: null }));
  },

  reset: () => {
    playlistBrowserStore.set(initialState);
  },

  // ---- Selection actions ----

  enterSelectionMode: () => {
    playlistBrowserStore.update((s) => ({ ...s, selectionMode: true }));
  },

  exitSelectionMode: () => {
    playlistBrowserStore.update((s) => ({
      ...s,
      selectionMode: false,
      selectedPlaylistIds: new Set(),
    }));
  },

  togglePlaylistSelection: (playlistId: string) => {
    playlistBrowserStore.update((s) => {
      const next = new Set(s.selectedPlaylistIds);
      if (next.has(playlistId)) next.delete(playlistId);
      else next.add(playlistId);
      return {
        ...s,
        selectedPlaylistIds: next,
        selectionMode: next.size > 0 ? true : s.selectionMode,
      };
    });
  },

  selectAllFiltered: (filteredIds: string[]) => {
    playlistBrowserStore.update((s) => {
      const next = new Set(s.selectedPlaylistIds);
      for (const id of filteredIds) next.add(id);
      return { ...s, selectedPlaylistIds: next, selectionMode: true };
    });
  },

  deselectAll: () => {
    playlistBrowserStore.update((s) => ({
      ...s,
      selectedPlaylistIds: new Set(),
    }));
  },
};
