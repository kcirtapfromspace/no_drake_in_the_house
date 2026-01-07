import { writable, derived } from 'svelte/store';
import { apiClient } from '../utils/api-client';

// Types matching the backend
export type OffenseCategory =
  | 'domestic_violence'
  | 'sexual_misconduct'
  | 'sexual_assault'
  | 'child_abuse'
  | 'hate_speech'
  | 'racism'
  | 'homophobia'
  | 'antisemitism'
  | 'violent_crime'
  | 'drug_trafficking'
  | 'fraud'
  | 'animal_abuse'
  | 'other';

export type OffenseSeverity = 'minor' | 'moderate' | 'severe' | 'egregious';

export interface OffenseSummary {
  category: OffenseCategory;
  title: string;
  date: string;
  evidence_count: number;
}

export interface FlaggedArtist {
  id: string;
  name: string;
  track_count: number;
  severity: OffenseSeverity;
  offenses: OffenseSummary[];
}

export interface LibraryScanResult {
  total_tracks: number;
  total_artists: number;
  flagged_artists: FlaggedArtist[];
  flagged_tracks: number;
}

export interface ImportTrack {
  provider_track_id: string;
  track_name: string;
  album_name?: string;
  artist_name: string;
  source_type?: string;
  playlist_name?: string;
  added_at?: string;
}

export interface LibraryTrack {
  id: string;
  user_id: string;
  provider: string;
  provider_track_id: string;
  track_name?: string;
  album_name?: string;
  artist_id?: string;
  artist_name?: string;
  source_type?: string;
  playlist_name?: string;
  added_at?: string;
  last_synced: string;
}

interface LibraryState {
  tracks: LibraryTrack[];
  scanResult: LibraryScanResult | null;
  isScanning: boolean;
  isImporting: boolean;
  scanProgress: number;
  error: string | null;
}

const initialState: LibraryState = {
  tracks: [],
  scanResult: null,
  isScanning: false,
  isImporting: false,
  scanProgress: 0,
  error: null,
};

export const libraryStore = writable<LibraryState>(initialState);

// Derived stores
export const flaggedArtists = derived(
  libraryStore,
  ($store) => $store.scanResult?.flagged_artists || []
);

export const scanStats = derived(libraryStore, ($store) =>
  $store.scanResult
    ? {
        totalTracks: $store.scanResult.total_tracks,
        totalArtists: $store.scanResult.total_artists,
        flaggedTracks: $store.scanResult.flagged_tracks,
        flaggedArtists: $store.scanResult.flagged_artists.length,
      }
    : null
);

// Severity colors and labels
export const severityConfig: Record<OffenseSeverity, { color: string; label: string; description: string }> = {
  egregious: { color: 'bg-red-600', label: 'Egregious', description: 'Multiple severe offenses, ongoing patterns' },
  severe: { color: 'bg-orange-500', label: 'Severe', description: 'Convictions, proven abuse' },
  moderate: { color: 'bg-yellow-500', label: 'Moderate', description: 'Arrests, credible allegations' },
  minor: { color: 'bg-gray-400', label: 'Minor', description: 'Controversial statements' },
};

// Category display names
export const categoryLabels: Record<OffenseCategory, string> = {
  domestic_violence: 'Domestic Violence',
  sexual_misconduct: 'Sexual Misconduct',
  sexual_assault: 'Sexual Assault',
  child_abuse: 'Child Abuse',
  hate_speech: 'Hate Speech',
  racism: 'Racism',
  homophobia: 'Homophobia',
  antisemitism: 'Antisemitism',
  violent_crime: 'Violent Crime',
  drug_trafficking: 'Drug Trafficking',
  fraud: 'Fraud',
  animal_abuse: 'Animal Abuse',
  other: 'Other',
};

// Library actions
export const libraryActions = {
  /**
   * Scan user's library against offense database
   */
  scanLibrary: async (): Promise<LibraryScanResult | null> => {
    libraryStore.update((s) => ({ ...s, isScanning: true, scanProgress: 0, error: null }));

    // Simulate progress while waiting for API
    const progressInterval = setInterval(() => {
      libraryStore.update((s) => ({
        ...s,
        scanProgress: Math.min(s.scanProgress + 10, 90),
      }));
    }, 200);

    try {
      const result = await apiClient.get<LibraryScanResult>('/api/v1/library/scan');

      clearInterval(progressInterval);

      if (result.success && result.data) {
        libraryStore.update((s) => ({
          ...s,
          scanResult: result.data!,
          isScanning: false,
          scanProgress: 100,
        }));
        return result.data;
      } else {
        libraryStore.update((s) => ({
          ...s,
          isScanning: false,
          scanProgress: 0,
          error: result.message || 'Failed to scan library',
        }));
        return null;
      }
    } catch (e) {
      clearInterval(progressInterval);
      const message = e instanceof Error ? e.message : 'Failed to scan library';
      libraryStore.update((s) => ({
        ...s,
        isScanning: false,
        scanProgress: 0,
        error: message,
      }));
      return null;
    }
  },

  /**
   * Import library tracks from parsed data
   */
  importLibrary: async (provider: string, tracks: ImportTrack[]): Promise<{ imported: number } | null> => {
    libraryStore.update((s) => ({ ...s, isImporting: true, error: null }));

    try {
      const result = await apiClient.post<{ imported: number; message: string }>('/api/v1/library/import', {
        provider,
        tracks,
      });

      if (result.success && result.data) {
        libraryStore.update((s) => ({ ...s, isImporting: false }));
        return { imported: result.data.imported };
      } else {
        libraryStore.update((s) => ({
          ...s,
          isImporting: false,
          error: result.message || 'Failed to import library',
        }));
        return null;
      }
    } catch (e) {
      const message = e instanceof Error ? e.message : 'Failed to import library';
      libraryStore.update((s) => ({
        ...s,
        isImporting: false,
        error: message,
      }));
      return null;
    }
  },

  /**
   * Get user's library tracks
   */
  fetchLibrary: async (provider?: string): Promise<LibraryTrack[]> => {
    try {
      const endpoint = provider ? `/api/v1/library/tracks?provider=${provider}` : '/api/v1/library/tracks';
      const result = await apiClient.get<LibraryTrack[]>(endpoint);

      if (result.success && result.data) {
        libraryStore.update((s) => ({ ...s, tracks: result.data! }));
        return result.data;
      }
      return [];
    } catch (e) {
      console.error('Failed to fetch library:', e);
      return [];
    }
  },

  /**
   * Reset scan results
   */
  resetScan: () => {
    libraryStore.update((s) => ({
      ...s,
      scanResult: null,
      scanProgress: 0,
      error: null,
    }));
  },

  /**
   * Clear error
   */
  clearError: () => {
    libraryStore.update((s) => ({ ...s, error: null }));
  },
};

/**
 * Get all flagged artists from the public database
 */
export async function getFlaggedArtistsDatabase(
  severity?: OffenseSeverity,
  limit: number = 50,
  offset: number = 0
): Promise<FlaggedArtist[]> {
  try {
    let endpoint = `/api/v1/offenses?limit=${limit}&offset=${offset}`;
    if (severity) {
      endpoint += `&severity=${severity}`;
    }

    const result = await apiClient.get<FlaggedArtist[]>(endpoint, false);

    if (result.success && result.data) {
      return result.data;
    }
    return [];
  } catch (e) {
    console.error('Failed to fetch flagged artists:', e);
    return [];
  }
}

// Types for offense/evidence management
export interface OffenseEvidence {
  id: string;
  offense_id: string;
  url: string;
  source_name?: string;
  source_type?: string;
  title?: string;
  excerpt?: string;
  published_date?: string;
  archived_url?: string;
  is_primary_source: boolean;
  credibility_score?: number;
  submitted_by?: string;
  created_at: string;
}

export interface ArtistOffense {
  id: string;
  artist_id: string;
  category: OffenseCategory;
  severity: OffenseSeverity;
  title: string;
  description: string;
  incident_date?: string;
  incident_date_approximate: boolean;
  arrested: boolean;
  charged: boolean;
  convicted: boolean;
  settled: boolean;
  status: 'pending' | 'verified' | 'disputed' | 'rejected';
  verified_at?: string;
  verified_by?: string;
  submitted_by?: string;
  created_at: string;
  updated_at: string;
}

export interface OffenseWithEvidence {
  offense: ArtistOffense;
  evidence: OffenseEvidence[];
  artist_name: string;
}

export interface CreateOffenseRequest {
  artist_id: string;
  category: OffenseCategory;
  severity: OffenseSeverity;
  title: string;
  description: string;
  incident_date?: string;
  incident_date_approximate?: boolean;
  arrested?: boolean;
  charged?: boolean;
  convicted?: boolean;
  settled?: boolean;
}

export interface AddEvidenceRequest {
  offense_id: string;
  url: string;
  source_name?: string;
  source_type?: string;
  title?: string;
  excerpt?: string;
  published_date?: string;
  is_primary_source?: boolean;
  credibility_score?: number;
}

/**
 * Get offense details with evidence
 */
export async function getOffenseWithEvidence(offenseId: string): Promise<OffenseWithEvidence | null> {
  try {
    const result = await apiClient.get<OffenseWithEvidence>(`/api/v1/offenses/${offenseId}`, false);
    if (result.success && result.data) {
      return result.data;
    }
    return null;
  } catch (e) {
    console.error('Failed to fetch offense details:', e);
    return null;
  }
}

/**
 * Create a new offense report
 */
export async function createOffense(request: CreateOffenseRequest): Promise<ArtistOffense | null> {
  try {
    const result = await apiClient.post<ArtistOffense>('/api/v1/offenses/submit', request);
    if (result.success && result.data) {
      return result.data;
    }
    return null;
  } catch (e) {
    console.error('Failed to create offense:', e);
    return null;
  }
}

/**
 * Add evidence to an existing offense
 */
export async function addEvidence(request: AddEvidenceRequest): Promise<OffenseEvidence | null> {
  try {
    const result = await apiClient.post<OffenseEvidence>('/api/v1/offenses/evidence', request);
    if (result.success && result.data) {
      return result.data;
    }
    return null;
  } catch (e) {
    console.error('Failed to add evidence:', e);
    return null;
  }
}

/**
 * Search artists by name (for selecting when creating offense)
 */
export async function searchArtists(query: string): Promise<{ id: string; name: string }[]> {
  try {
    const result = await apiClient.get<{ id: string; name: string }[]>(
      `/api/v1/artists/search?q=${encodeURIComponent(query)}`,
      true
    );
    if (result.success && result.data) {
      return result.data;
    }
    return [];
  } catch (e) {
    console.error('Failed to search artists:', e);
    return [];
  }
}
