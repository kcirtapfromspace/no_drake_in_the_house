import { writable, derived } from 'svelte/store';
import { apiClient } from '../utils/api-client';

// === TYPES ===

// Status represents the artist's risk level
export type ArtistStatus = 'clean' | 'certified_creeper' | 'flagged';

// Confidence represents how sure we are about the information
export type ConfidenceLevel = 'high' | 'medium' | 'low';

// Source reliability tiers
export type SourceTier = 'tier_a' | 'tier_b' | 'tier_c' | 'tier_d';

// Procedural state for tags
export type ProceduralState =
  | 'alleged'
  | 'arrested'
  | 'charged'
  | 'trial_ongoing'
  | 'convicted'
  | 'sentenced'
  | 'appeal_pending'
  | 'case_dismissed'
  | 'record_sealed'
  | 'expunged';

// Evidence strength
export type EvidenceStrength = 'strong' | 'moderate' | 'weak';

// Category system
export interface OffenseCategory {
  id: string;
  name: string;
  description: string;
  color: string;
  icon: string;
}

// Tag with procedural state
export interface OffenseTag {
  id: string;
  name: string;
  category_id: string;
  procedural_state: ProceduralState;
  jurisdiction?: string;
}

// Evidence source
export interface EvidenceSource {
  id: string;
  url: string;
  title: string;
  source_name: string;
  source_type: 'court_record' | 'news' | 'investigation' | 'social_media' | 'official_statement';
  tier: SourceTier;
  published_date?: string;
  jurisdiction?: string;
  excerpt?: string;
  archived_url?: string;
  credibility_score?: number;
}

// Evidence item
export interface Evidence {
  id: string;
  offense_id: string;
  source: EvidenceSource;
  date_added: string;
  added_by?: string;
  verified: boolean;
  verification_notes?: string;
}

// Offense/Incident
export interface Offense {
  id: string;
  artist_id: string;
  category: OffenseCategory;
  tags: OffenseTag[];
  title: string;
  description: string;
  incident_date?: string;
  procedural_state: ProceduralState;
  evidence: Evidence[];
  evidence_strength: EvidenceStrength;
  last_updated: string;
  created_at: string;
}

// Streaming metrics
export interface StreamingMetrics {
  total_streams: number;
  monthly_streams: number;
  monthly_trend: number; // percentage change
  platform_breakdown: {
    platform: string;
    streams: number;
    percentage: number;
  }[];
  top_tracks: {
    track_id: string;
    title: string;
    streams: number;
    revenue_estimate?: number;
  }[];
  estimated_monthly_revenue?: number;
}

// Collaborator
export interface Collaborator {
  id: string;
  name: string;
  image_url?: string;
  collaboration_count: number;
  is_flagged: boolean;
  status: ArtistStatus;
  collaboration_type: 'featured' | 'producer' | 'writer' | 'label';
  recent_tracks?: string[];
}

// Artist profile image
export interface ArtistImage {
  url: string;
  type?: 'mugshot' | 'court' | 'booking' | 'promotional' | 'editorial';
  source?: string;
  date?: string;
  jurisdiction?: string;
  caption?: string;
}

// Writer/Producer credit
export interface Credit {
  id: string;
  name: string;
  role: 'writer' | 'producer';
  track_count: number;
  is_flagged: boolean;
  image_url?: string | null;
  note?: string;
}

// Credits collection
export interface ArtistCredits {
  writers: Credit[];
  producers: Credit[];
}

// Full artist profile
export interface ArtistProfile {
  id: string;
  canonical_name: string;
  aliases: string[];
  external_ids: {
    spotify?: string;
    apple_music?: string;
    musicbrainz?: string;
    isni?: string;
    deezer?: string;
  };
  status: ArtistStatus;
  confidence: ConfidenceLevel;
  images: ArtistImage[];
  primary_image?: ArtistImage;
  genres: string[];
  offenses: Offense[];
  streaming_metrics?: StreamingMetrics;
  collaborators: Collaborator[];
  credits?: ArtistCredits;
  label?: string;
  last_reviewed?: string;
  created_at: string;
  updated_at: string;
}

// Store state
export interface ArtistState {
  profile: ArtistProfile | null;
  isLoading: boolean;
  error: string | null;
  isLoadingEvidence: boolean;
  isLoadingMetrics: boolean;
  isLoadingCollaborators: boolean;
}

// === STORE ===

const initialState: ArtistState = {
  profile: null,
  isLoading: false,
  error: null,
  isLoadingEvidence: false,
  isLoadingMetrics: false,
  isLoadingCollaborators: false,
};

export const artistStore = writable<ArtistState>(initialState);

// Derived stores
export const artistProfile = derived(artistStore, ($store) => $store.profile);
export const artistOffenses = derived(artistStore, ($store) => $store.profile?.offenses || []);
export const artistCollaborators = derived(artistStore, ($store) => $store.profile?.collaborators || []);
export const artistMetrics = derived(artistStore, ($store) => $store.profile?.streaming_metrics);

// === ACTIONS ===

export const artistActions = {
  fetchProfile: async (artistId: string) => {
    artistStore.update(state => ({ ...state, isLoading: true, error: null }));

    try {
      const result = await apiClient.get<ArtistProfile>(`/api/v1/artists/${artistId}/profile`);

      if (result.success && result.data) {
        artistStore.update(state => ({
          ...state,
          profile: result.data!,
          isLoading: false,
        }));
        return { success: true, data: result.data };
      } else {
        artistStore.update(state => ({
          ...state,
          error: result.message || 'Failed to fetch artist profile',
          isLoading: false,
        }));
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      artistStore.update(state => ({
        ...state,
        error: error.message || 'Network error',
        isLoading: false,
      }));
      return { success: false, message: error.message };
    }
  },

  fetchOffenses: async (artistId: string) => {
    artistStore.update(state => ({ ...state, isLoadingEvidence: true }));

    try {
      const result = await apiClient.get<{ offenses: Offense[] }>(`/api/v1/offenses/query?artist_id=${artistId}`);

      if (result.success && result.data) {
        artistStore.update(state => ({
          ...state,
          profile: state.profile ? {
            ...state.profile,
            offenses: result.data!.offenses || []
          } : null,
          isLoadingEvidence: false,
        }));
        return { success: true, data: result.data.offenses };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      artistStore.update(state => ({ ...state, isLoadingEvidence: false }));
      return { success: false, message: error.message };
    }
  },

  fetchStreamingMetrics: async (artistId: string) => {
    artistStore.update(state => ({ ...state, isLoadingMetrics: true }));

    try {
      const result = await apiClient.get<StreamingMetrics>(`/api/v1/artists/${artistId}/analytics`);

      if (result.success && result.data) {
        artistStore.update(state => ({
          ...state,
          profile: state.profile ? {
            ...state.profile,
            streaming_metrics: result.data!
          } : null,
          isLoadingMetrics: false,
        }));
        return { success: true, data: result.data };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      artistStore.update(state => ({ ...state, isLoadingMetrics: false }));
      return { success: false, message: error.message };
    }
  },

  fetchCollaborators: async (artistId: string) => {
    artistStore.update(state => ({ ...state, isLoadingCollaborators: true }));

    try {
      const result = await apiClient.get<{ collaborators: Collaborator[] }>(`/api/v1/graph/artists/${artistId}/collaborators`);

      if (result.success && result.data) {
        artistStore.update(state => ({
          ...state,
          profile: state.profile ? {
            ...state.profile,
            collaborators: result.data!.collaborators || []
          } : null,
          isLoadingCollaborators: false,
        }));
        return { success: true, data: result.data.collaborators };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      artistStore.update(state => ({ ...state, isLoadingCollaborators: false }));
      return { success: false, message: error.message };
    }
  },

  submitEvidence: async (offenseId: string, evidence: Partial<Evidence>) => {
    try {
      const result = await apiClient.post(`/api/v1/offenses/${offenseId}/evidence`, evidence);
      return { success: result.success, message: result.message };
    } catch (error: any) {
      return { success: false, message: error.message };
    }
  },

  reportError: async (artistId: string, description: string, category: string) => {
    try {
      const result = await apiClient.post('/api/v1/offenses/report-error', {
        artist_id: artistId,
        description,
        category,
      });
      return { success: result.success, message: result.message };
    } catch (error: any) {
      return { success: false, message: error.message };
    }
  },

  clearProfile: () => {
    artistStore.set(initialState);
  },

  clearError: () => {
    artistStore.update(state => ({ ...state, error: null }));
  },
};

// === HELPER FUNCTIONS ===

export function getStatusColor(status: ArtistStatus): { bg: string; text: string; border: string } {
  switch (status) {
    case 'flagged':
      return { bg: '#DC2626', text: '#FFFFFF', border: '#EF4444' };
    case 'certified_creeper':
      return { bg: '#DB2777', text: '#FFFFFF', border: '#EC4899' }; // Pink for certified creeper
    case 'clean':
      return { bg: '#10B981', text: '#FFFFFF', border: '#34D399' };
    default:
      return { bg: '#6B7280', text: '#FFFFFF', border: '#9CA3AF' };
  }
}

export function getStatusLabel(status: ArtistStatus): string {
  switch (status) {
    case 'flagged':
      return 'Flagged';
    case 'certified_creeper':
      return 'Certified Creeper';
    case 'clean':
      return 'Clean';
    default:
      return 'Unknown';
  }
}

export function getConfidenceIcon(level: ConfidenceLevel): string {
  switch (level) {
    case 'high':
      return '███'; // 3 bars filled
    case 'medium':
      return '██░'; // 2 bars filled
    case 'low':
      return '█░░'; // 1 bar filled
    default:
      return '░░░';
  }
}

export function getConfidenceLabel(level: ConfidenceLevel): string {
  switch (level) {
    case 'high':
      return 'High Confidence';
    case 'medium':
      return 'Medium Confidence';
    case 'low':
      return 'Low Confidence';
    default:
      return 'Unknown';
  }
}

export function getSourceTierLabel(tier: SourceTier): { label: string; description: string; color: string } {
  switch (tier) {
    case 'tier_a':
      return {
        label: 'Tier A',
        description: 'Primary Records (Court dockets, judgments, official filings)',
        color: '#10B981'
      };
    case 'tier_b':
      return {
        label: 'Tier B',
        description: 'Reputable Secondary (Major news, investigative journalism)',
        color: '#3B82F6'
      };
    case 'tier_c':
      return {
        label: 'Tier C',
        description: 'Corroborated Reports (Multiple independent sources)',
        color: '#F59E0B'
      };
    case 'tier_d':
      return {
        label: 'Tier D',
        description: 'Unverified / Single Source',
        color: '#EF4444'
      };
    default:
      return {
        label: 'Unknown',
        description: 'Source reliability unknown',
        color: '#6B7280'
      };
  }
}

export function getProceduralStateLabel(state: ProceduralState): string {
  const labels: Record<ProceduralState, string> = {
    alleged: 'Alleged',
    arrested: 'Arrested',
    charged: 'Charged',
    trial_ongoing: 'Trial Ongoing',
    convicted: 'Convicted',
    sentenced: 'Sentenced',
    appeal_pending: 'Appeal Pending',
    case_dismissed: 'Case Dismissed',
    record_sealed: 'Record Sealed',
    expunged: 'Expunged',
  };
  return labels[state] || state;
}

export function getEvidenceStrengthLabel(strength: EvidenceStrength): { label: string; color: string } {
  switch (strength) {
    case 'strong':
      return { label: 'Strong', color: '#10B981' };
    case 'moderate':
      return { label: 'Moderate', color: '#F59E0B' };
    case 'weak':
      return { label: 'Weak', color: '#EF4444' };
    default:
      return { label: 'Unknown', color: '#6B7280' };
  }
}

// Category colors matching the design system
export const CATEGORY_COLORS: Record<string, { icon: string; bg: string; name: string }> = {
  violence_physical_harm: { icon: '#DC2626', bg: 'rgba(220, 38, 38, 0.15)', name: 'Violence & Physical Harm' },
  sexual_misconduct: { icon: '#DB2777', bg: 'rgba(219, 39, 119, 0.15)', name: 'Sexual Misconduct' },
  exploitation_abuse: { icon: '#9333EA', bg: 'rgba(147, 51, 234, 0.15)', name: 'Exploitation & Abuse' },
  hate_extremism: { icon: '#7C3AED', bg: 'rgba(124, 58, 237, 0.15)', name: 'Hate & Extremism' },
  fraud_financial: { icon: '#059669', bg: 'rgba(5, 150, 105, 0.15)', name: 'Fraud & Financial Crime' },
  weapons_organized_crime: { icon: '#B91C1C', bg: 'rgba(185, 28, 28, 0.15)', name: 'Weapons & Organized Crime' },
  harassment_coercion: { icon: '#EA580C', bg: 'rgba(234, 88, 12, 0.15)', name: 'Harassment & Coercion' },
  substance_trafficking: { icon: '#0891B2', bg: 'rgba(8, 145, 178, 0.15)', name: 'Substance & Trafficking' },
  legal_proceedings: { icon: '#4B5563', bg: 'rgba(75, 85, 99, 0.15)', name: 'Legal Proceedings' },
  reputational_risk: { icon: '#6B7280', bg: 'rgba(107, 114, 128, 0.15)', name: 'Reputational Risk (Non-Criminal)' },
};

export function getCategoryColor(categoryId: string): { icon: string; bg: string; name: string } {
  return CATEGORY_COLORS[categoryId] || { icon: '#6B7280', bg: 'rgba(107, 114, 128, 0.15)', name: categoryId };
}
