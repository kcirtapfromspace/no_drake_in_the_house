import { writable, derived } from 'svelte/store';
import { apiClient } from '../utils/api-client';

// Types
export interface GraphArtist {
  id: string;
  name: string;
  genres: string[];
  is_blocked: boolean;
  image_url?: string;
}

export interface Collaboration {
  artist_id: string;
  artist_name: string;
  track_id?: string;
  track_title?: string;
  collab_type: 'feature' | 'producer' | 'writer' | 'remix';
  year?: number;
}

export interface NetworkNode {
  id: string;
  name: string;
  type: 'artist' | 'label' | 'track';
  is_blocked: boolean;
  genres?: string[];
  image_url?: string;
  x?: number;
  y?: number;
}

export interface NetworkEdge {
  source: string;
  target: string;
  type: 'collaborated_with' | 'signed_to' | 'mentioned_in';
  weight: number;
  metadata?: {
    track_title?: string;
    year?: number;
  };
}

export interface ArtistNetwork {
  nodes: NetworkNode[];
  edges: NetworkEdge[];
  center_artist_id: string;
  depth: number;
}

export interface PathResult {
  path: NetworkNode[];
  edges: NetworkEdge[];
  total_distance: number;
}

export interface BlockedNetworkAnalysis {
  at_risk_artists: {
    artist: GraphArtist;
    blocked_collaborators: number;
    risk_score: number;
  }[];
  blocked_clusters: {
    cluster_id: string;
    artists: GraphArtist[];
    internal_collaborations: number;
  }[];
  summary: {
    total_blocked: number;
    total_at_risk: number;
    avg_collaborations_per_blocked: number;
  };
}

export interface GraphHealth {
  status: 'healthy' | 'degraded' | 'unhealthy';
  node_count: number;
  edge_count: number;
  last_sync: string;
  sync_lag_seconds: number;
}

export interface GraphStats {
  artist_count: number;
  collaboration_count: number;
  label_count: number;
  track_count: number;
}

// State
export interface GraphState {
  currentNetwork: ArtistNetwork | null;
  selectedArtist: GraphArtist | null;
  collaborators: Collaboration[];
  pathResult: PathResult | null;
  blockedAnalysis: BlockedNetworkAnalysis | null;
  health: GraphHealth | null;
  stats: GraphStats | null;
  isLoading: boolean;
  isLoadingPath: boolean;
  error: string | null;
}

const initialState: GraphState = {
  currentNetwork: null,
  selectedArtist: null,
  collaborators: [],
  pathResult: null,
  blockedAnalysis: null,
  health: null,
  stats: null,
  isLoading: false,
  isLoadingPath: false,
  error: null,
};

// Store
export const graphStore = writable<GraphState>(initialState);

// Derived stores
export const networkNodes = derived(
  graphStore,
  ($graph) => $graph.currentNetwork?.nodes ?? []
);

export const networkEdges = derived(
  graphStore,
  ($graph) => $graph.currentNetwork?.edges ?? []
);

export const blockedNodes = derived(
  graphStore,
  ($graph) => $graph.currentNetwork?.nodes.filter(n => n.is_blocked) ?? []
);

export const atRiskArtists = derived(
  graphStore,
  ($graph) => $graph.blockedAnalysis?.at_risk_artists ?? []
);

export const isGraphHealthy = derived(
  graphStore,
  ($graph) => $graph.health?.status === 'healthy'
);

// Actions
export const graphActions = {
  fetchArtistNetwork: async (artistId: string, depth: number = 2) => {
    graphStore.update(s => ({ ...s, isLoading: true, error: null }));

    try {
      const result = await apiClient.authenticatedRequest<ArtistNetwork>(
        'GET',
        `/api/v1/graph/artists/${artistId}/network?depth=${depth}`
      );

      if (result.success && result.data) {
        graphStore.update(s => ({
          ...s,
          currentNetwork: result.data!,
          isLoading: false,
        }));
        return { success: true };
      } else {
        graphStore.update(s => ({
          ...s,
          isLoading: false,
          error: result.message || 'Failed to fetch artist network',
        }));
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      graphStore.update(s => ({
        ...s,
        isLoading: false,
        error: error.message || 'Network error',
      }));
      return { success: false, message: error.message };
    }
  },

  fetchCollaborators: async (artistId: string, limit: number = 20) => {
    graphStore.update(s => ({ ...s, isLoading: true, error: null }));

    try {
      const result = await apiClient.authenticatedRequest<{ collaborators: Collaboration[] }>(
        'GET',
        `/api/v1/graph/artists/${artistId}/collaborators?limit=${limit}`
      );

      if (result.success && result.data) {
        graphStore.update(s => ({
          ...s,
          collaborators: result.data!.collaborators,
          isLoading: false,
        }));
        return { success: true };
      } else {
        graphStore.update(s => ({
          ...s,
          isLoading: false,
          error: result.message || 'Failed to fetch collaborators',
        }));
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      graphStore.update(s => ({
        ...s,
        isLoading: false,
        error: error.message || 'Network error',
      }));
      return { success: false, message: error.message };
    }
  },

  findPath: async (sourceId: string, targetId: string) => {
    graphStore.update(s => ({ ...s, isLoadingPath: true, error: null, pathResult: null }));

    try {
      const result = await apiClient.authenticatedRequest<PathResult>(
        'GET',
        `/api/v1/graph/artists/${sourceId}/path-to/${targetId}`
      );

      if (result.success && result.data) {
        graphStore.update(s => ({
          ...s,
          pathResult: result.data!,
          isLoadingPath: false,
        }));
        return { success: true, path: result.data };
      } else {
        graphStore.update(s => ({
          ...s,
          isLoadingPath: false,
          error: result.message || 'No path found',
        }));
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      graphStore.update(s => ({
        ...s,
        isLoadingPath: false,
        error: error.message || 'Network error',
      }));
      return { success: false, message: error.message };
    }
  },

  analyzeBlockedNetwork: async () => {
    graphStore.update(s => ({ ...s, isLoading: true, error: null }));

    try {
      const result = await apiClient.authenticatedRequest<BlockedNetworkAnalysis>(
        'POST',
        '/api/v1/graph/blocked-network'
      );

      if (result.success && result.data) {
        graphStore.update(s => ({
          ...s,
          blockedAnalysis: result.data!,
          isLoading: false,
        }));
        return { success: true, analysis: result.data };
      } else {
        graphStore.update(s => ({
          ...s,
          isLoading: false,
          error: result.message || 'Failed to analyze blocked network',
        }));
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      graphStore.update(s => ({
        ...s,
        isLoading: false,
        error: error.message || 'Network error',
      }));
      return { success: false, message: error.message };
    }
  },

  fetchHealth: async () => {
    try {
      const result = await apiClient.authenticatedRequest<GraphHealth>(
        'GET',
        '/api/v1/graph/health'
      );

      if (result.success && result.data) {
        graphStore.update(s => ({
          ...s,
          health: result.data!,
        }));
        return { success: true, health: result.data };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      return { success: false, message: error.message };
    }
  },

  fetchStats: async () => {
    try {
      const result = await apiClient.authenticatedRequest<GraphStats>(
        'GET',
        '/api/v1/graph/stats'
      );

      if (result.success && result.data) {
        graphStore.update(s => ({
          ...s,
          stats: result.data!,
        }));
        return { success: true, stats: result.data };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error: any) {
      return { success: false, message: error.message };
    }
  },

  selectArtist: (artist: GraphArtist | null) => {
    graphStore.update(s => ({ ...s, selectedArtist: artist }));
  },

  clearNetwork: () => {
    graphStore.update(s => ({
      ...s,
      currentNetwork: null,
      collaborators: [],
      pathResult: null,
    }));
  },

  clearError: () => {
    graphStore.update(s => ({ ...s, error: null }));
  },
};
