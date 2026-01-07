import { writable, derived } from 'svelte/store';

export interface CommunityList {
  id: string;
  owner_user_id: string;
  name: string;
  description: string;
  criteria: string;
  governance_url?: string;
  update_cadence: string;
  version: number;
  visibility: 'public' | 'private';
  created_at: string;
  updated_at: string;
  artist_count?: number;
  subscriber_count?: number;
}

export interface CommunityListWithArtists extends CommunityList {
  artists: Array<{
    artist: {
      id: string;
      canonical_name: string;
      external_ids: Record<string, string>;
      metadata: Record<string, any>;
    };
    rationale_link?: string;
    added_at: string;
  }>;
}

export interface Subscription {
  list_id: string;
  list: CommunityList;
  version_pinned?: number;
  auto_update: boolean;
  created_at: string;
}

export interface SubscriptionImpact {
  list_id: string;
  list_name: string;
  artists_to_add: number;
  artists_to_remove: number;
  preview_artists: Array<{
    artist: any;
    action: 'add' | 'remove';
  }>;
}

export interface CommunityState {
  lists: CommunityList[];
  currentList: CommunityListWithArtists | null;
  subscriptions: Subscription[];
  subscriptionImpact: SubscriptionImpact | null;
  isLoading: boolean;
  isLoadingList: boolean;
  isLoadingImpact: boolean;
  searchQuery: string;
  sortBy: 'name' | 'created_at' | 'updated_at' | 'artist_count' | 'subscriber_count';
  sortOrder: 'asc' | 'desc';
  error: string | null;
}

const initialState: CommunityState = {
  lists: [],
  currentList: null,
  subscriptions: [],
  subscriptionImpact: null,
  isLoading: false,
  isLoadingList: false,
  isLoadingImpact: false,
  searchQuery: '',
  sortBy: 'updated_at',
  sortOrder: 'desc',
  error: null,
};

export const communityStore = writable<CommunityState>(initialState);

export const filteredLists = derived(
  communityStore,
  ($community) => {
    let filtered = $community.lists;
    
    // Apply search filter
    if ($community.searchQuery.trim()) {
      const query = $community.searchQuery.toLowerCase();
      filtered = filtered.filter(list => 
        list.name.toLowerCase().includes(query) ||
        list.description.toLowerCase().includes(query) ||
        list.criteria.toLowerCase().includes(query)
      );
    }
    
    // Apply sorting
    filtered.sort((a, b) => {
      let aValue: any = a[$community.sortBy];
      let bValue: any = b[$community.sortBy];
      
      if ($community.sortBy === 'created_at' || $community.sortBy === 'updated_at') {
        aValue = new Date(aValue).getTime();
        bValue = new Date(bValue).getTime();
      }
      
      if (typeof aValue === 'string') {
        aValue = aValue.toLowerCase();
        bValue = bValue.toLowerCase();
      }
      
      const comparison = aValue < bValue ? -1 : aValue > bValue ? 1 : 0;
      return $community.sortOrder === 'asc' ? comparison : -comparison;
    });
    
    return filtered;
  }
);

export const subscribedListIds = derived(
  communityStore,
  ($community) => new Set($community.subscriptions.map(sub => sub.list_id))
);

export const isSubscribed = derived(
  subscribedListIds,
  ($subscribedIds) => (listId: string) => $subscribedIds.has(listId)
);

// Community actions
export const communityActions = {
  fetchLists: async () => {
    communityStore.update(state => ({ ...state, isLoading: true, error: null }));
    
    try {
      const token = localStorage.getItem('auth_token');
      const response = await fetch('http://localhost:3000/api/v1/community/lists', {
        headers: {
          'Authorization': `Bearer ${token}`,
        },
      });

      const result = await response.json();
      
      if (result.success) {
        communityStore.update(state => ({
          ...state,
          lists: result.data,
          isLoading: false,
        }));
      } else {
        communityStore.update(state => ({
          ...state,
          error: result.message,
          isLoading: false,
        }));
      }
    } catch (error) {
      communityStore.update(state => ({
        ...state,
        error: 'Failed to fetch community lists',
        isLoading: false,
      }));
    }
  },

  fetchListDetails: async (listId: string) => {
    communityStore.update(state => ({ ...state, isLoadingList: true, error: null }));
    
    try {
      const token = localStorage.getItem('auth_token');
      const response = await fetch(`http://localhost:3000/api/v1/community/lists/${listId}/artists`, {
        headers: {
          'Authorization': `Bearer ${token}`,
        },
      });

      const result = await response.json();
      
      if (result.success) {
        communityStore.update(state => ({
          ...state,
          currentList: result.data,
          isLoadingList: false,
        }));
      } else {
        communityStore.update(state => ({
          ...state,
          error: result.message,
          isLoadingList: false,
        }));
      }
    } catch (error) {
      communityStore.update(state => ({
        ...state,
        error: 'Failed to fetch list details',
        isLoadingList: false,
      }));
    }
  },

  fetchSubscriptions: async () => {
    try {
      const token = localStorage.getItem('auth_token');
      const response = await fetch('http://localhost:3000/api/v1/community/subscriptions', {
        headers: {
          'Authorization': `Bearer ${token}`,
        },
      });

      // Check if response is ok and has content
      if (!response.ok) {
        if (response.status === 404) {
          // Endpoint doesn't exist yet, set empty subscriptions
          communityStore.update(state => ({
            ...state,
            subscriptions: [],
          }));
          return;
        }
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const contentType = response.headers.get('content-type');
      if (!contentType || !contentType.includes('application/json')) {
        // Not JSON response, set empty subscriptions
        communityStore.update(state => ({
          ...state,
          subscriptions: [],
        }));
        return;
      }

      const result = await response.json();
      
      if (result.success) {
        communityStore.update(state => ({
          ...state,
          subscriptions: result.data || [],
        }));
      } else {
        communityStore.update(state => ({
          ...state,
          subscriptions: [],
        }));
      }
    } catch (error) {
      console.error('Failed to fetch subscriptions:', error);
      // Set empty subscriptions on error
      communityStore.update(state => ({
        ...state,
        subscriptions: [],
      }));
    }
  },

  getSubscriptionImpact: async (listId: string) => {
    communityStore.update(state => ({ ...state, isLoadingImpact: true, error: null }));
    
    try {
      const token = localStorage.getItem('auth_token');
      const response = await fetch(`http://localhost:3000/api/v1/community/lists/${listId}/impact`, {
        headers: {
          'Authorization': `Bearer ${token}`,
        },
      });

      const result = await response.json();
      
      if (result.success) {
        communityStore.update(state => ({
          ...state,
          subscriptionImpact: result.data,
          isLoadingImpact: false,
        }));
        return { success: true, data: result.data };
      } else {
        communityStore.update(state => ({
          ...state,
          error: result.message,
          isLoadingImpact: false,
        }));
        return { success: false, message: result.message };
      }
    } catch (error) {
      communityStore.update(state => ({
        ...state,
        error: 'Failed to get subscription impact',
        isLoadingImpact: false,
      }));
      return { success: false, message: 'Failed to get subscription impact' };
    }
  },

  subscribe: async (listId: string, versionPinned?: number, autoUpdate = true) => {
    try {
      const token = localStorage.getItem('auth_token');
      const response = await fetch(`http://localhost:3000/api/v1/community/lists/${listId}/subscribe`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          version_pinned: versionPinned,
          auto_update: autoUpdate,
        }),
      });

      const result = await response.json();
      
      if (result.success) {
        // Refresh subscriptions
        await communityActions.fetchSubscriptions();
        return { success: true };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error) {
      return { success: false, message: 'Failed to subscribe to list' };
    }
  },

  unsubscribe: async (listId: string) => {
    try {
      const token = localStorage.getItem('auth_token');
      const response = await fetch(`http://localhost:3000/api/v1/community/lists/${listId}/unsubscribe`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${token}`,
        },
      });

      const result = await response.json();
      
      if (result.success) {
        // Refresh subscriptions
        await communityActions.fetchSubscriptions();
        return { success: true };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error) {
      return { success: false, message: 'Failed to unsubscribe from list' };
    }
  },

  updateSubscription: async (listId: string, versionPinned?: number, autoUpdate?: boolean) => {
    try {
      const token = localStorage.getItem('auth_token');
      const response = await fetch(`http://localhost:3000/api/v1/community/lists/${listId}/subscription`, {
        method: 'PUT',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          version_pinned: versionPinned,
          auto_update: autoUpdate,
        }),
      });

      const result = await response.json();
      
      if (result.success) {
        // Refresh subscriptions
        await communityActions.fetchSubscriptions();
        return { success: true };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error) {
      return { success: false, message: 'Failed to update subscription' };
    }
  },

  createList: async (listData: {
    name: string;
    description: string;
    criteria: string;
    governance_url?: string;
    update_cadence: string;
    visibility: 'public' | 'private';
  }) => {
    try {
      const token = localStorage.getItem('auth_token');
      const response = await fetch('http://localhost:3000/api/v1/community/lists', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(listData),
      });

      const result = await response.json();
      
      if (result.success) {
        // Refresh lists
        await communityActions.fetchLists();
        return { success: true, data: result.data };
      } else {
        return { success: false, message: result.message };
      }
    } catch (error) {
      return { success: false, message: 'Failed to create community list' };
    }
  },

  updateSearch: (query: string) => {
    communityStore.update(state => ({ ...state, searchQuery: query }));
  },

  updateSort: (sortBy: CommunityState['sortBy'], sortOrder: CommunityState['sortOrder']) => {
    communityStore.update(state => ({ ...state, sortBy, sortOrder }));
  },

  clearCurrentList: () => {
    communityStore.update(state => ({ ...state, currentList: null }));
  },

  clearError: () => {
    communityStore.update(state => ({ ...state, error: null }));
  },
};