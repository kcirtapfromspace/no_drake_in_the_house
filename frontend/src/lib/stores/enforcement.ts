import { writable, derived } from 'svelte/store';

export interface EnforcementOptions {
  aggressiveness: 'conservative' | 'moderate' | 'aggressive';
  blockCollabs: boolean;
  blockFeaturing: boolean;
  blockSongwriterOnly: boolean;
}

export interface EnforcementImpact {
  provider: string;
  likedSongs?: {
    toRemove: number;
    collabsFound: number;
  };
  playlists?: {
    toScrub: number;
    tracksToRemove: number;
    featuringFound: number;
  };
  following?: {
    toUnfollow: number;
  };
  radioSeeds?: {
    toFilter: number;
  };
}

export interface EnforcementPlan {
  planId: string;
  idempotencyKey: string;
  impact: Record<string, EnforcementImpact>;
  capabilities: Record<string, Record<string, string>>;
  estimatedDuration: string;
  resumable: boolean;
}

export interface ActionItem {
  id: string;
  entityType: string;
  entityId: string;
  action: string;
  beforeState?: any;
  afterState?: any;
  status: 'pending' | 'completed' | 'failed' | 'skipped';
  errorMessage?: string;
}

export interface ActionBatch {
  id: string;
  provider: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';
  options: EnforcementOptions;
  summary: {
    totalItems: number;
    completedItems: number;
    failedItems: number;
    skippedItems: number;
  };
  items: ActionItem[];
  createdAt: string;
  completedAt?: string;
}

export interface EnforcementState {
  currentPlan: EnforcementPlan | null;
  isPlanning: boolean;
  isExecuting: boolean;
  currentBatch: ActionBatch | null;
  actionHistory: ActionBatch[];
  options: EnforcementOptions;
  error: string | null;
}

const defaultOptions: EnforcementOptions = {
  aggressiveness: 'moderate',
  blockCollabs: true,
  blockFeaturing: true,
  blockSongwriterOnly: false,
};

const initialState: EnforcementState = {
  currentPlan: null,
  isPlanning: false,
  isExecuting: false,
  currentBatch: null,
  actionHistory: [],
  options: defaultOptions,
  error: null,
};

export const enforcementStore = writable<EnforcementState>(initialState);

export const hasActivePlan = derived(
  enforcementStore,
  ($enforcement) => $enforcement.currentPlan !== null
);

export const executionProgress = derived(
  enforcementStore,
  ($enforcement) => {
    if (!$enforcement.currentBatch) return null;
    
    const { totalItems, completedItems, failedItems, skippedItems } = $enforcement.currentBatch.summary;
    const processedItems = completedItems + failedItems + skippedItems;
    
    return {
      total: totalItems,
      processed: processedItems,
      completed: completedItems,
      failed: failedItems,
      skipped: skippedItems,
      percentage: totalItems > 0 ? Math.round((processedItems / totalItems) * 100) : 0,
    };
  }
);

export const canRollback = derived(
  enforcementStore,
  ($enforcement) => $enforcement.actionHistory.some(batch => 
    batch.status === 'completed' && batch.items.some(item => item.status === 'completed')
  )
);

// Enforcement actions
export const enforcementActions = {
  updateOptions: (options: Partial<EnforcementOptions>) => {
    enforcementStore.update(state => ({
      ...state,
      options: { ...state.options, ...options },
    }));
  },

  createPlan: async (providers: string[], dryRun = true) => {
    let currentOptions = defaultOptions;
    enforcementStore.update(state => {
      currentOptions = state.options;
      return { ...state, isPlanning: true, error: null };
    });
    
    try {
      const token = localStorage.getItem('auth_token');
      const response = await fetch('http://localhost:3000/api/v1/spotify/library/plan', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          providers,
          options: currentOptions,
          dryRun,
        }),
      });

      const result = await response.json();
      
      if (result.success) {
        enforcementStore.update(state => ({
          ...state,
          currentPlan: result.data,
          isPlanning: false,
        }));
        return { success: true, data: result.data };
      } else {
        enforcementStore.update(state => ({
          ...state,
          error: result.message,
          isPlanning: false,
        }));
        return { success: false, message: result.message };
      }
    } catch (error) {
      enforcementStore.update(state => ({
        ...state,
        error: 'Failed to create enforcement plan',
        isPlanning: false,
      }));
      return { success: false, message: 'Failed to create enforcement plan' };
    }
  },

  executePlan: async (planId: string) => {
    enforcementStore.update(state => ({ ...state, isExecuting: true, error: null }));
    
    try {
      const token = localStorage.getItem('auth_token');
      const response = await fetch('http://localhost:3000/api/v1/spotify/enforcement/execute', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          planId,
          dryRun: false,
        }),
      });

      const result = await response.json();
      
      if (result.success) {
        const batch = result.data;
        enforcementStore.update(state => ({
          ...state,
          currentBatch: batch,
          isExecuting: false,
        }));
        
        // Start polling for progress
        enforcementActions.pollProgress(batch.id);
        return { success: true, data: batch };
      } else {
        enforcementStore.update(state => ({
          ...state,
          error: result.message,
          isExecuting: false,
        }));
        return { success: false, message: result.message };
      }
    } catch (error) {
      enforcementStore.update(state => ({
        ...state,
        error: 'Failed to execute enforcement plan',
        isExecuting: false,
      }));
      return { success: false, message: 'Failed to execute enforcement plan' };
    }
  },

  pollProgress: async (batchId: string) => {
    const pollInterval = setInterval(async () => {
      try {
        const token = localStorage.getItem('auth_token');
        const response = await fetch(`http://localhost:3000/api/v1/spotify/enforcement/progress/${batchId}`, {
          headers: {
            'Authorization': `Bearer ${token}`,
          },
        });

        const result = await response.json();
        
        if (result.success) {
          const batch = result.data;
          enforcementStore.update(state => ({
            ...state,
            currentBatch: batch,
          }));
          
          // Stop polling if batch is complete
          if (batch.status === 'completed' || batch.status === 'failed' || batch.status === 'cancelled') {
            clearInterval(pollInterval);
            
            // Move to history
            enforcementStore.update(state => ({
              ...state,
              actionHistory: [batch, ...state.actionHistory],
              currentBatch: null,
              currentPlan: null,
            }));
          }
        }
      } catch (error) {
        console.error('Failed to poll progress:', error);
      }
    }, 2000); // Poll every 2 seconds
  },

  rollbackBatch: async (batchId: string) => {
    try {
      const token = localStorage.getItem('auth_token');
      const response = await fetch('http://localhost:3000/api/v1/spotify/enforcement/rollback', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ batchId }),
      });

      const result = await response.json();
      
      if (result.success) {
        // Refresh action history
        await enforcementActions.fetchActionHistory();
        return { success: true };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error) {
      return { success: false, message: 'Failed to rollback actions' };
    }
  },

  fetchActionHistory: async () => {
    try {
      const token = localStorage.getItem('auth_token');
      const response = await fetch('http://localhost:3000/api/v1/spotify/enforcement/history', {
        headers: {
          'Authorization': `Bearer ${token}`,
        },
      });

      const result = await response.json();
      
      if (result.success) {
        enforcementStore.update(state => ({
          ...state,
          actionHistory: result.data,
        }));
      }
    } catch (error) {
      console.error('Failed to fetch action history:', error);
    }
  },

  clearPlan: () => {
    enforcementStore.update(state => ({
      ...state,
      currentPlan: null,
      error: null,
    }));
  },

  clearError: () => {
    enforcementStore.update(state => ({
      ...state,
      error: null,
    }));
  },
};