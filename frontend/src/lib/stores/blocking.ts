import { writable, derived } from 'svelte/store';

export type Platform = 'spotify' | 'apple_music' | 'youtube_music';

export type EnforcementStatus = 'pending' | 'in_progress' | 'completed' | 'failed' | 'not_connected';

export interface PlatformEnforcement {
  platform: Platform;
  status: EnforcementStatus;
  error?: string;
  completedAt?: string;
}

export interface BlockingOperation {
  artistId: string;
  artistName: string;
  action: 'block' | 'unblock';
  startedAt: string;
  platforms: PlatformEnforcement[];
  overallStatus: 'in_progress' | 'completed' | 'partial' | 'failed';
}

export interface ArtistEnforcementStatus {
  artistId: string;
  platforms: {
    spotify?: EnforcementStatus;
    apple_music?: EnforcementStatus;
    youtube_music?: EnforcementStatus;
  };
  lastUpdated?: string;
}

export interface BlockingState {
  // Current blocking operations in progress
  activeOperations: BlockingOperation[];
  // Persistent enforcement status per artist
  artistEnforcements: Map<string, ArtistEnforcementStatus>;
  // Toast notifications
  toasts: Toast[];
}

export interface Toast {
  id: string;
  type: 'info' | 'success' | 'warning' | 'error';
  message: string;
  artistName?: string;
  progress?: number; // 0-100
  dismissible: boolean;
  duration?: number; // ms, undefined = manual dismiss
  createdAt: number;
}

const initialState: BlockingState = {
  activeOperations: [],
  artistEnforcements: new Map(),
  toasts: [],
};

function createBlockingStore() {
  const { subscribe, update, set } = writable<BlockingState>(initialState);

  // Load saved enforcement statuses from localStorage
  function loadFromStorage() {
    try {
      const stored = localStorage.getItem('artistEnforcements');
      if (stored) {
        const parsed = JSON.parse(stored);
        update(state => ({
          ...state,
          artistEnforcements: new Map(Object.entries(parsed)),
        }));
      }
    } catch (e) {
      console.error('Failed to load enforcement statuses:', e);
    }
  }

  // Save enforcement statuses to localStorage
  function saveToStorage(enforcements: Map<string, ArtistEnforcementStatus>) {
    try {
      const obj = Object.fromEntries(enforcements);
      localStorage.setItem('artistEnforcements', JSON.stringify(obj));
    } catch (e) {
      console.error('Failed to save enforcement statuses:', e);
    }
  }

  return {
    subscribe,

    init: () => {
      loadFromStorage();
    },

    // Start a blocking/unblocking operation
    startOperation: (
      artistId: string,
      artistName: string,
      action: 'block' | 'unblock',
      connectedPlatforms: Platform[]
    ) => {
      const operation: BlockingOperation = {
        artistId,
        artistName,
        action,
        startedAt: new Date().toISOString(),
        platforms: connectedPlatforms.map(platform => ({
          platform,
          status: 'pending',
        })),
        overallStatus: 'in_progress',
      };

      update(state => {
        // Add toast notification
        const toastId = `${artistId}-${Date.now()}`;
        const toast: Toast = {
          id: toastId,
          type: 'info',
          message: `${action === 'block' ? 'Blocking' : 'Unblocking'} ${artistName}...`,
          artistName,
          progress: 0,
          dismissible: false,
          createdAt: Date.now(),
        };

        return {
          ...state,
          activeOperations: [...state.activeOperations, operation],
          toasts: [...state.toasts, toast],
        };
      });

      return operation;
    },

    // Update platform status within an operation
    updatePlatformStatus: (
      artistId: string,
      platform: Platform,
      status: EnforcementStatus,
      error?: string
    ) => {
      update(state => {
        const operations = state.activeOperations.map(op => {
          if (op.artistId !== artistId) return op;

          const platforms = op.platforms.map(p => {
            if (p.platform !== platform) return p;
            return {
              ...p,
              status,
              error,
              completedAt: status === 'completed' || status === 'failed' ? new Date().toISOString() : undefined,
            };
          });

          // Calculate overall status
          const allCompleted = platforms.every(p => p.status === 'completed');
          const allFailed = platforms.every(p => p.status === 'failed');
          const anyFailed = platforms.some(p => p.status === 'failed');
          const anyInProgress = platforms.some(p => p.status === 'in_progress' || p.status === 'pending');

          let overallStatus: BlockingOperation['overallStatus'];
          if (anyInProgress) {
            overallStatus = 'in_progress';
          } else if (allCompleted) {
            overallStatus = 'completed';
          } else if (allFailed) {
            overallStatus = 'failed';
          } else if (anyFailed) {
            overallStatus = 'partial';
          } else {
            overallStatus = 'in_progress';
          }

          // Update progress in toast
          const completedCount = platforms.filter(p => p.status === 'completed' || p.status === 'failed').length;
          const progress = Math.round((completedCount / platforms.length) * 100);

          return { ...op, platforms, overallStatus };
        });

        // Update toast progress
        const toasts = state.toasts.map(toast => {
          if (!toast.artistName) return toast;
          const op = operations.find(o => o.artistName === toast.artistName);
          if (!op) return toast;

          const completedCount = op.platforms.filter(p => p.status === 'completed' || p.status === 'failed').length;
          const progress = Math.round((completedCount / op.platforms.length) * 100);

          return { ...toast, progress };
        });

        return { ...state, activeOperations: operations, toasts };
      });
    },

    // Complete an operation and update persistent enforcement status
    completeOperation: (artistId: string) => {
      update(state => {
        const operation = state.activeOperations.find(op => op.artistId === artistId);
        if (!operation) return state;

        // Update persistent enforcement status
        const enforcement: ArtistEnforcementStatus = {
          artistId,
          platforms: {},
          lastUpdated: new Date().toISOString(),
        };

        operation.platforms.forEach(p => {
          if (operation.action === 'block') {
            enforcement.platforms[p.platform] = p.status === 'completed' ? 'completed' : 'failed';
          } else {
            // On unblock, remove enforcement status
            enforcement.platforms[p.platform] = undefined;
          }
        });

        const newEnforcements = new Map(state.artistEnforcements);
        if (operation.action === 'unblock') {
          newEnforcements.delete(artistId);
        } else {
          newEnforcements.set(artistId, enforcement);
        }

        // Save to storage
        saveToStorage(newEnforcements);

        // Update toast to show completion
        const toasts = state.toasts.map(toast => {
          if (toast.artistName !== operation.artistName) return toast;

          const isSuccess = operation.overallStatus === 'completed';
          const isPartial = operation.overallStatus === 'partial';

          return {
            ...toast,
            type: isSuccess ? 'success' : isPartial ? 'warning' : 'error',
            message: isSuccess
              ? `${operation.action === 'block' ? 'Blocked' : 'Unblocked'} ${operation.artistName}`
              : isPartial
              ? `Partially ${operation.action === 'block' ? 'blocked' : 'unblocked'} ${operation.artistName}`
              : `Failed to ${operation.action} ${operation.artistName}`,
            progress: 100,
            dismissible: true,
            duration: 5000,
          } as Toast;
        });

        // Remove from active operations
        const activeOperations = state.activeOperations.filter(op => op.artistId !== artistId);

        return {
          ...state,
          activeOperations,
          artistEnforcements: newEnforcements,
          toasts,
        };
      });
    },

    // Get enforcement status for an artist
    getEnforcementStatus: (artistId: string): ArtistEnforcementStatus | undefined => {
      let result: ArtistEnforcementStatus | undefined;
      subscribe(state => {
        result = state.artistEnforcements.get(artistId);
      })();
      return result;
    },

    // Add a custom toast
    addToast: (toast: Omit<Toast, 'id' | 'createdAt'>) => {
      update(state => ({
        ...state,
        toasts: [
          ...state.toasts,
          {
            ...toast,
            id: `toast-${Date.now()}`,
            createdAt: Date.now(),
          },
        ],
      }));
    },

    // Remove a toast
    removeToast: (id: string) => {
      update(state => ({
        ...state,
        toasts: state.toasts.filter(t => t.id !== id),
      }));
    },

    // Clear all toasts
    clearToasts: () => {
      update(state => ({
        ...state,
        toasts: [],
      }));
    },

    // Set enforcement status directly (e.g., from API response)
    setEnforcementStatus: (artistId: string, platforms: ArtistEnforcementStatus['platforms']) => {
      update(state => {
        const newEnforcements = new Map(state.artistEnforcements);
        newEnforcements.set(artistId, {
          artistId,
          platforms,
          lastUpdated: new Date().toISOString(),
        });
        saveToStorage(newEnforcements);
        return {
          ...state,
          artistEnforcements: newEnforcements,
        };
      });
    },
  };
}

export const blockingStore = createBlockingStore();

// Derived store for active toasts
export const activeToasts = derived(blockingStore, $state => $state.toasts);

// Derived store to check if any operation is in progress
export const isBlocking = derived(
  blockingStore,
  $state => $state.activeOperations.length > 0
);

// Helper to get enforcement badges for an artist
export function getEnforcementBadges(artistId: string) {
  let badges: { platform: Platform; status: EnforcementStatus }[] = [];

  blockingStore.subscribe(state => {
    const enforcement = state.artistEnforcements.get(artistId);
    if (enforcement) {
      badges = Object.entries(enforcement.platforms)
        .filter(([_, status]) => status)
        .map(([platform, status]) => ({
          platform: platform as Platform,
          status: status as EnforcementStatus,
        }));
    }
  })();

  return badges;
}
