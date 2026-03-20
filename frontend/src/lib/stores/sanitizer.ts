import { writable, derived } from 'svelte/store';
import { apiClient } from '../utils/api-client';

// ---- Types ----

export interface BlockedArtistBreakdown {
  artist_id: string;
  artist_name: string;
  track_count: number;
  block_reason: string;
}

export interface BlockedTrackDetail {
  track_id: string;
  track_name: string;
  artist_id: string;
  artist_name: string;
  all_artist_names: string[];
  block_reason: string;
  position: number;
  duration_ms: number;
}

export interface PlaylistGrade {
  playlist_id: string;
  playlist_name: string;
  total_tracks: number;
  clean_tracks: number;
  blocked_tracks: number;
  cleanliness_score: number;
  grade_letter: string;
  artist_breakdown: BlockedArtistBreakdown[];
  blocked_track_details: BlockedTrackDetail[];
}

export interface ReplacementTrack {
  track_id: string;
  track_name: string;
  artist_name: string;
  artist_id: string;
  album_name: string;
  popularity: number;
  preview_url: string | null;
  duration_ms: number;
  spotify_uri: string;
}

export interface ReplacementSuggestion {
  original_track_id: string;
  original_track_name: string;
  original_artist_name: string;
  candidates: ReplacementTrack[];
}

export interface PublishResult {
  new_playlist_id: string;
  new_playlist_url: string;
  tracks_kept: number;
  tracks_replaced: number;
  tracks_removed: number;
  total_tracks: number;
}

export interface SanitizerState {
  currentGrade: PlaylistGrade | null;
  currentPlanId: string | null;
  replacements: ReplacementSuggestion[];
  selectedReplacements: Record<string, string>; // original_track_id -> replacement_track_id or "skip"
  targetPlaylistName: string;
  publishResult: PublishResult | null;
  isGrading: boolean;
  isSuggesting: boolean;
  isPublishing: boolean;
  error: string | null;
  step: 'grade' | 'replace' | 'publish';
}

const initialState: SanitizerState = {
  currentGrade: null,
  currentPlanId: null,
  replacements: [],
  selectedReplacements: {},
  targetPlaylistName: '',
  publishResult: null,
  isGrading: false,
  isSuggesting: false,
  isPublishing: false,
  error: null,
  step: 'grade',
};

// ---- Store ----

export const sanitizerStore = writable<SanitizerState>(initialState);

export const hasGrade = derived(
  sanitizerStore,
  ($s) => $s.currentGrade !== null
);

export const hasPlan = derived(
  sanitizerStore,
  ($s) => $s.currentPlanId !== null
);

export const allReplacementsSelected = derived(
  sanitizerStore,
  ($s) => {
    if ($s.replacements.length === 0) return false;
    return $s.replacements.every(
      (r) => r.original_track_id in $s.selectedReplacements
    );
  }
);

// ---- Actions ----

export const sanitizerActions = {
  /** Grade a playlist by ID or URL */
  gradePlaylist: async (playlistId: string) => {
    sanitizerStore.update((s) => ({
      ...s,
      isGrading: true,
      error: null,
      currentGrade: null,
      publishResult: null,
    }));

    try {
      const response = await apiClient.post<{ grade: PlaylistGrade }>(
        '/api/v1/sanitizer/grade',
        { playlist_id: playlistId }
      );

      if (response.success && response.data) {
        sanitizerStore.update((s) => ({
          ...s,
          currentGrade: response.data!.grade,
          targetPlaylistName: `${response.data!.grade.playlist_name} (Sanitized)`,
          isGrading: false,
          step: 'grade',
        }));
        return { success: true };
      } else {
        sanitizerStore.update((s) => ({
          ...s,
          error: response.message || 'Failed to grade playlist',
          isGrading: false,
        }));
        return { success: false, message: response.message };
      }
    } catch (error) {
      const msg = error instanceof Error ? error.message : 'Failed to grade playlist';
      sanitizerStore.update((s) => ({
        ...s,
        error: msg,
        isGrading: false,
      }));
      return { success: false, message: msg };
    }
  },

  /** Grade + suggest replacements, creating a draft plan */
  suggestReplacements: async (playlistId: string) => {
    sanitizerStore.update((s) => ({
      ...s,
      isSuggesting: true,
      error: null,
      publishResult: null,
    }));

    try {
      const response = await apiClient.post<{
        plan_id: string | null;
        grade: PlaylistGrade;
        replacements: ReplacementSuggestion[];
        message?: string;
      }>('/api/v1/sanitizer/suggest', { playlist_id: playlistId });

      if (response.success && response.data) {
        const { plan_id, grade, replacements } = response.data;

        // Auto-select first candidate for each replacement
        const autoSelected: Record<string, string> = {};
        for (const r of replacements) {
          if (r.candidates.length > 0) {
            autoSelected[r.original_track_id] = r.candidates[0].track_id;
          }
        }

        sanitizerStore.update((s) => ({
          ...s,
          currentGrade: grade,
          currentPlanId: plan_id,
          replacements,
          selectedReplacements: autoSelected,
          targetPlaylistName: `${grade.playlist_name} (Sanitized)`,
          isSuggesting: false,
          step: replacements.length > 0 ? 'replace' : 'grade',
        }));
        return { success: true };
      } else {
        sanitizerStore.update((s) => ({
          ...s,
          error: response.message || 'Failed to suggest replacements',
          isSuggesting: false,
        }));
        return { success: false, message: response.message };
      }
    } catch (error) {
      const msg = error instanceof Error ? error.message : 'Failed to suggest replacements';
      sanitizerStore.update((s) => ({
        ...s,
        error: msg,
        isSuggesting: false,
      }));
      return { success: false, message: msg };
    }
  },

  /** Select a replacement for a blocked track */
  selectReplacement: (originalTrackId: string, replacementTrackId: string) => {
    sanitizerStore.update((s) => ({
      ...s,
      selectedReplacements: {
        ...s.selectedReplacements,
        [originalTrackId]: replacementTrackId,
      },
    }));
  },

  /** Skip a blocked track (remove it instead of replacing) */
  skipTrack: (originalTrackId: string) => {
    sanitizerStore.update((s) => ({
      ...s,
      selectedReplacements: {
        ...s.selectedReplacements,
        [originalTrackId]: 'skip',
      },
    }));
  },

  /** Set the target playlist name */
  setTargetName: (name: string) => {
    sanitizerStore.update((s) => ({ ...s, targetPlaylistName: name }));
  },

  /** Confirm the plan and publish */
  confirmAndPublish: async () => {
    let planId: string | null = null;
    let selections: Record<string, string> = {};
    let targetName = '';

    sanitizerStore.update((s) => {
      planId = s.currentPlanId;
      selections = s.selectedReplacements;
      targetName = s.targetPlaylistName;
      return { ...s, isPublishing: true, error: null, step: 'publish' };
    });

    if (!planId) {
      sanitizerStore.update((s) => ({
        ...s,
        error: 'No plan to publish',
        isPublishing: false,
      }));
      return { success: false, message: 'No plan to publish' };
    }

    try {
      // Step 1: Confirm the plan
      await apiClient.put(`/api/v1/sanitizer/plan/${planId}`, {
        target_playlist_name: targetName,
        selected_replacements: selections,
      });

      // Step 2: Publish
      const response = await apiClient.post<{
        plan_id: string;
        result: PublishResult;
      }>(`/api/v1/sanitizer/publish/${planId}`);

      if (response.success && response.data) {
        sanitizerStore.update((s) => ({
          ...s,
          publishResult: response.data!.result,
          isPublishing: false,
        }));
        return { success: true };
      } else {
        sanitizerStore.update((s) => ({
          ...s,
          error: response.message || 'Failed to publish playlist',
          isPublishing: false,
        }));
        return { success: false, message: response.message };
      }
    } catch (error) {
      const msg = error instanceof Error ? error.message : 'Failed to publish playlist';
      sanitizerStore.update((s) => ({
        ...s,
        error: msg,
        isPublishing: false,
      }));
      return { success: false, message: msg };
    }
  },

  /** Navigate to a step */
  goToStep: (step: 'grade' | 'replace' | 'publish') => {
    sanitizerStore.update((s) => ({ ...s, step }));
  },

  /** Reset the sanitizer state */
  reset: () => {
    sanitizerStore.set(initialState);
  },

  /** Clear error */
  clearError: () => {
    sanitizerStore.update((s) => ({ ...s, error: null }));
  },
};
