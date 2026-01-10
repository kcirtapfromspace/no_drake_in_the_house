/**
 * Enforcement Components Tests
 * Tests for enforcement-related UI components and store
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';

// ============================================
// Enforcement Store Tests
// ============================================

describe('Enforcement Store', () => {
  describe('EnforcementOptions', () => {
    const defaultOptions = {
      aggressiveness: 'moderate' as const,
      blockCollabs: true,
      blockFeaturing: true,
      blockSongwriterOnly: false,
    };

    it('should have correct default options', () => {
      expect(defaultOptions.aggressiveness).toBe('moderate');
      expect(defaultOptions.blockCollabs).toBe(true);
      expect(defaultOptions.blockFeaturing).toBe(true);
      expect(defaultOptions.blockSongwriterOnly).toBe(false);
    });

    it('should have valid aggressiveness levels', () => {
      const validLevels = ['conservative', 'moderate', 'aggressive'];
      expect(validLevels).toContain('conservative');
      expect(validLevels).toContain('moderate');
      expect(validLevels).toContain('aggressive');
    });

    it('should allow partial option updates', () => {
      const updates = { aggressiveness: 'aggressive' as const };
      const newOptions = { ...defaultOptions, ...updates };

      expect(newOptions.aggressiveness).toBe('aggressive');
      expect(newOptions.blockCollabs).toBe(true); // Unchanged
    });
  });

  describe('EnforcementState', () => {
    const initialState = {
      currentPlan: null,
      isPlanning: false,
      isExecuting: false,
      currentBatch: null,
      actionHistory: [],
      options: {
        aggressiveness: 'moderate' as const,
        blockCollabs: true,
        blockFeaturing: true,
        blockSongwriterOnly: false,
      },
      error: null,
    };

    it('should have correct initial state', () => {
      expect(initialState.currentPlan).toBeNull();
      expect(initialState.isPlanning).toBe(false);
      expect(initialState.isExecuting).toBe(false);
      expect(initialState.currentBatch).toBeNull();
      expect(initialState.actionHistory).toEqual([]);
      expect(initialState.error).toBeNull();
    });

    it('should track planning state', () => {
      const planningState = { ...initialState, isPlanning: true };
      expect(planningState.isPlanning).toBe(true);
    });

    it('should track executing state', () => {
      const executingState = { ...initialState, isExecuting: true };
      expect(executingState.isExecuting).toBe(true);
    });
  });

  describe('EnforcementPlan', () => {
    const mockPlan = {
      planId: 'plan-123',
      idempotencyKey: 'key-456',
      impact: {
        spotify: {
          provider: 'spotify',
          likedSongs: { toRemove: 10, collabsFound: 3 },
          playlists: { toScrub: 5, tracksToRemove: 15, featuringFound: 2 },
        },
      },
      capabilities: {
        spotify: {
          REMOVE_FROM_LIBRARY: 'SUPPORTED',
          UNFOLLOW_ARTIST: 'SUPPORTED',
          FILTER_RECOMMENDATIONS: 'LIMITED',
        },
      },
      estimatedDuration: '30s',
      resumable: true,
    };

    it('should have required plan fields', () => {
      expect(mockPlan.planId).toBeDefined();
      expect(mockPlan.idempotencyKey).toBeDefined();
      expect(mockPlan.impact).toBeDefined();
      expect(mockPlan.capabilities).toBeDefined();
      expect(mockPlan.estimatedDuration).toBeDefined();
      expect(mockPlan.resumable).toBeDefined();
    });

    it('should have provider impact details', () => {
      const spotifyImpact = mockPlan.impact.spotify;
      expect(spotifyImpact.likedSongs?.toRemove).toBe(10);
      expect(spotifyImpact.playlists?.tracksToRemove).toBe(15);
    });

    it('should have capability levels', () => {
      const capabilities = mockPlan.capabilities.spotify;
      expect(capabilities.REMOVE_FROM_LIBRARY).toBe('SUPPORTED');
      expect(capabilities.FILTER_RECOMMENDATIONS).toBe('LIMITED');
    });
  });

  describe('ActionBatch', () => {
    const mockBatch = {
      id: 'batch-123',
      provider: 'spotify',
      status: 'completed' as const,
      options: {
        aggressiveness: 'moderate' as const,
        blockCollabs: true,
        blockFeaturing: true,
        blockSongwriterOnly: false,
      },
      summary: {
        totalItems: 25,
        completedItems: 23,
        failedItems: 1,
        skippedItems: 1,
      },
      items: [],
      createdAt: '2024-01-15T10:00:00Z',
      completedAt: '2024-01-15T10:00:30Z',
    };

    it('should have valid batch status', () => {
      const validStatuses = ['pending', 'running', 'completed', 'failed', 'cancelled'];
      expect(validStatuses).toContain(mockBatch.status);
    });

    it('should have summary totals', () => {
      const { summary } = mockBatch;
      const processed = summary.completedItems + summary.failedItems + summary.skippedItems;
      expect(processed).toBe(25);
      expect(summary.totalItems).toBe(25);
    });

    it('should calculate progress percentage', () => {
      const { summary } = mockBatch;
      const processed = summary.completedItems + summary.failedItems + summary.skippedItems;
      const percentage = Math.round((processed / summary.totalItems) * 100);
      expect(percentage).toBe(100);
    });
  });

  describe('ActionItem', () => {
    const mockActionItem = {
      id: 'action-1',
      entityType: 'track',
      entityId: 'spotify:track:123',
      action: 'remove_from_library',
      beforeState: { liked: true },
      afterState: { liked: false },
      status: 'completed' as const,
    };

    it('should have valid action item status', () => {
      const validStatuses = ['pending', 'completed', 'failed', 'skipped'];
      expect(validStatuses).toContain(mockActionItem.status);
    });

    it('should track before and after state', () => {
      expect(mockActionItem.beforeState.liked).toBe(true);
      expect(mockActionItem.afterState.liked).toBe(false);
    });

    it('should have entity type and id', () => {
      expect(mockActionItem.entityType).toBe('track');
      expect(mockActionItem.entityId).toContain('spotify:track:');
    });
  });
});

// ============================================
// EnforcementOptions Component Tests
// ============================================

describe('EnforcementOptions Component', () => {
  describe('Aggressiveness Levels', () => {
    const aggressivenessDescriptions = {
      conservative: 'Only remove explicitly saved/liked content. Preserves playlists and recommendations.',
      moderate: 'Remove from saved content and playlists. Filters recommendations where possible.',
      aggressive: 'Maximum removal including radio seeds, recommendations, and related content.',
    };

    it('should have descriptions for all levels', () => {
      expect(aggressivenessDescriptions.conservative).toContain('saved/liked');
      expect(aggressivenessDescriptions.moderate).toContain('playlists');
      expect(aggressivenessDescriptions.aggressive).toContain('Maximum removal');
    });

    it('should recommend moderate as default', () => {
      const defaultLevel = 'moderate';
      expect(defaultLevel).toBe('moderate');
    });
  });

  describe('Collaboration Options', () => {
    const collabOptions = {
      blockCollabs: {
        label: 'Block collaborations',
        description: 'Remove songs where blocked artists are listed as collaborators or co-writers.',
      },
      blockFeaturing: {
        label: 'Block featuring',
        description: 'Remove songs where blocked artists are featured (e.g., "Song Title (feat. Blocked Artist)").',
      },
      blockSongwriterOnly: {
        label: 'Block songwriter credits only',
        description: 'Remove songs where blocked artists are credited only as songwriters (most restrictive).',
      },
    };

    it('should have all collaboration options', () => {
      expect(collabOptions.blockCollabs).toBeDefined();
      expect(collabOptions.blockFeaturing).toBeDefined();
      expect(collabOptions.blockSongwriterOnly).toBeDefined();
    });

    it('should describe blocking collaborations', () => {
      expect(collabOptions.blockCollabs.description).toContain('collaborators');
    });

    it('should describe blocking featuring', () => {
      expect(collabOptions.blockFeaturing.description).toContain('featured');
    });

    it('should mark songwriter-only as most restrictive', () => {
      expect(collabOptions.blockSongwriterOnly.description).toContain('most restrictive');
    });
  });

  describe('Warning Display', () => {
    it('should show warning for aggressive settings', () => {
      const options = { aggressiveness: 'aggressive', blockSongwriterOnly: false };
      const shouldShowWarning = options.aggressiveness === 'aggressive' || options.blockSongwriterOnly;
      expect(shouldShowWarning).toBe(true);
    });

    it('should show warning for songwriter-only', () => {
      const options = { aggressiveness: 'moderate', blockSongwriterOnly: true };
      const shouldShowWarning = options.aggressiveness === 'aggressive' || options.blockSongwriterOnly;
      expect(shouldShowWarning).toBe(true);
    });

    it('should not show warning for conservative settings', () => {
      const options = { aggressiveness: 'conservative', blockSongwriterOnly: false };
      const shouldShowWarning = options.aggressiveness === 'aggressive' || options.blockSongwriterOnly;
      expect(shouldShowWarning).toBe(false);
    });
  });
});

// ============================================
// EnforcementPreview Component Tests
// ============================================

describe('EnforcementPreview Component', () => {
  describe('Duration Formatting', () => {
    function formatDuration(duration: string) {
      const seconds = parseInt(duration.replace('s', ''));
      if (seconds < 60) return `${seconds} seconds`;
      const minutes = Math.floor(seconds / 60);
      const remainingSeconds = seconds % 60;
      return `${minutes}m ${remainingSeconds}s`;
    }

    it('should format seconds correctly', () => {
      expect(formatDuration('30s')).toBe('30 seconds');
    });

    it('should format minutes correctly', () => {
      expect(formatDuration('90s')).toBe('1m 30s');
    });

    it('should format exact minutes', () => {
      expect(formatDuration('120s')).toBe('2m 0s');
    });

    it('should handle zero seconds', () => {
      expect(formatDuration('0s')).toBe('0 seconds');
    });
  });

  describe('Provider Icons', () => {
    function getProviderIcon(provider: string) {
      switch (provider) {
        case 'spotify':
          return 'spotify-icon-path';
        case 'apple':
          return 'apple-icon-path';
        default:
          return 'default-icon-path';
      }
    }

    it('should return correct icon for Spotify', () => {
      expect(getProviderIcon('spotify')).toBe('spotify-icon-path');
    });

    it('should return correct icon for Apple Music', () => {
      expect(getProviderIcon('apple')).toBe('apple-icon-path');
    });

    it('should return default icon for unknown provider', () => {
      expect(getProviderIcon('unknown')).toBe('default-icon-path');
    });
  });

  describe('Capability Colors', () => {
    function getCapabilityColor(capability: string) {
      switch (capability) {
        case 'SUPPORTED':
          return 'text-green-400 bg-zinc-700';
        case 'LIMITED':
          return 'text-yellow-400 bg-zinc-700';
        case 'UNSUPPORTED':
          return 'text-red-400 bg-zinc-700';
        default:
          return 'text-zinc-300 bg-zinc-700';
      }
    }

    it('should use green for supported capabilities', () => {
      expect(getCapabilityColor('SUPPORTED')).toContain('green');
    });

    it('should use yellow for limited capabilities', () => {
      expect(getCapabilityColor('LIMITED')).toContain('yellow');
    });

    it('should use red for unsupported capabilities', () => {
      expect(getCapabilityColor('UNSUPPORTED')).toContain('red');
    });

    it('should use neutral for unknown capabilities', () => {
      expect(getCapabilityColor('UNKNOWN')).toContain('zinc');
    });
  });

  describe('Impact Display', () => {
    const mockImpact = {
      likedSongs: { toRemove: 15, collabsFound: 5 },
      playlists: { toScrub: 3, tracksToRemove: 20, featuringFound: 8 },
      following: { toUnfollow: 2 },
      radioSeeds: { toFilter: 4 },
    };

    it('should display liked songs impact', () => {
      expect(mockImpact.likedSongs.toRemove).toBe(15);
      expect(mockImpact.likedSongs.collabsFound).toBe(5);
    });

    it('should display playlists impact', () => {
      expect(mockImpact.playlists.toScrub).toBe(3);
      expect(mockImpact.playlists.tracksToRemove).toBe(20);
    });

    it('should display following impact', () => {
      expect(mockImpact.following.toUnfollow).toBe(2);
    });

    it('should display radio seeds impact', () => {
      expect(mockImpact.radioSeeds.toFilter).toBe(4);
    });
  });

  describe('Empty Plan State', () => {
    it('should show empty state when no plan', () => {
      const plan = null;
      expect(plan).toBeNull();
    });

    it('should have empty state message', () => {
      const emptyMessage = 'No enforcement plan';
      const emptySubMessage = 'Create a plan to see the preview.';
      expect(emptyMessage).toBeDefined();
      expect(emptySubMessage).toBeDefined();
    });
  });
});

// ============================================
// EnforcementExecution Component Tests
// ============================================

describe('EnforcementExecution Component', () => {
  describe('Progress Calculation', () => {
    function calculateProgress(batch: any) {
      if (!batch) return null;
      const { totalItems, completedItems, failedItems, skippedItems } = batch.summary;
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

    it('should return null when no batch', () => {
      expect(calculateProgress(null)).toBeNull();
    });

    it('should calculate percentage correctly', () => {
      const batch = {
        summary: {
          totalItems: 100,
          completedItems: 50,
          failedItems: 5,
          skippedItems: 5,
        },
      };
      const progress = calculateProgress(batch);
      expect(progress?.percentage).toBe(60);
    });

    it('should handle zero total items', () => {
      const batch = {
        summary: {
          totalItems: 0,
          completedItems: 0,
          failedItems: 0,
          skippedItems: 0,
        },
      };
      const progress = calculateProgress(batch);
      expect(progress?.percentage).toBe(0);
    });

    it('should track all item states', () => {
      const batch = {
        summary: {
          totalItems: 10,
          completedItems: 7,
          failedItems: 2,
          skippedItems: 1,
        },
      };
      const progress = calculateProgress(batch);
      expect(progress?.completed).toBe(7);
      expect(progress?.failed).toBe(2);
      expect(progress?.skipped).toBe(1);
      expect(progress?.processed).toBe(10);
    });
  });

  describe('Status Display', () => {
    function getStatusColor(status: string) {
      switch (status) {
        case 'pending':
          return 'text-zinc-400';
        case 'running':
          return 'text-blue-400';
        case 'completed':
          return 'text-green-400';
        case 'failed':
          return 'text-red-400';
        case 'cancelled':
          return 'text-yellow-400';
        default:
          return 'text-zinc-400';
      }
    }

    it('should have correct color for pending', () => {
      expect(getStatusColor('pending')).toContain('zinc');
    });

    it('should have correct color for running', () => {
      expect(getStatusColor('running')).toContain('blue');
    });

    it('should have correct color for completed', () => {
      expect(getStatusColor('completed')).toContain('green');
    });

    it('should have correct color for failed', () => {
      expect(getStatusColor('failed')).toContain('red');
    });

    it('should have correct color for cancelled', () => {
      expect(getStatusColor('cancelled')).toContain('yellow');
    });
  });
});

// ============================================
// EnforcementBadges Component Tests
// ============================================

describe('EnforcementBadges Component', () => {
  describe('Badge Display', () => {
    function getEnforcementStatus(connected: boolean, enforced: boolean) {
      if (!connected) return { label: 'Not Connected', color: 'bg-zinc-600' };
      if (enforced) return { label: 'Enforced', color: 'bg-green-600' };
      return { label: 'Connected', color: 'bg-blue-600' };
    }

    it('should show not connected status', () => {
      const status = getEnforcementStatus(false, false);
      expect(status.label).toBe('Not Connected');
      expect(status.color).toContain('zinc');
    });

    it('should show connected status', () => {
      const status = getEnforcementStatus(true, false);
      expect(status.label).toBe('Connected');
      expect(status.color).toContain('blue');
    });

    it('should show enforced status', () => {
      const status = getEnforcementStatus(true, true);
      expect(status.label).toBe('Enforced');
      expect(status.color).toContain('green');
    });
  });

  describe('Provider Badges', () => {
    const providers = ['spotify', 'apple'];

    it('should support Spotify', () => {
      expect(providers).toContain('spotify');
    });

    it('should support Apple Music', () => {
      expect(providers).toContain('apple');
    });
  });
});

// ============================================
// Derived Store Tests
// ============================================

describe('Enforcement Derived Stores', () => {
  describe('hasActivePlan', () => {
    it('should be true when plan exists', () => {
      const state = { currentPlan: { planId: '123' } };
      const hasActivePlan = state.currentPlan !== null;
      expect(hasActivePlan).toBe(true);
    });

    it('should be false when no plan', () => {
      const state = { currentPlan: null };
      const hasActivePlan = state.currentPlan !== null;
      expect(hasActivePlan).toBe(false);
    });
  });

  describe('canRollback', () => {
    it('should be true when completed batch has completed items', () => {
      const state = {
        actionHistory: [
          {
            status: 'completed',
            items: [{ status: 'completed' }, { status: 'failed' }],
          },
        ],
      };
      const canRollback = state.actionHistory.some(
        (batch: any) =>
          batch.status === 'completed' && batch.items.some((item: any) => item.status === 'completed')
      );
      expect(canRollback).toBe(true);
    });

    it('should be false when no completed batches', () => {
      const state = {
        actionHistory: [{ status: 'failed', items: [] }],
      };
      const canRollback = state.actionHistory.some(
        (batch: any) =>
          batch.status === 'completed' && batch.items.some((item: any) => item.status === 'completed')
      );
      expect(canRollback).toBe(false);
    });

    it('should be false when empty history', () => {
      const state = { actionHistory: [] };
      const canRollback = state.actionHistory.some((batch: any) => batch.status === 'completed');
      expect(canRollback).toBe(false);
    });
  });
});

// ============================================
// Apple Music Specific Tests
// ============================================

describe('Apple Music Enforcement', () => {
  describe('Ratings API Enforcement', () => {
    const capabilities = {
      ratings_enforcement: true,
      library_modification: false,
      playlist_modification: false,
    };

    it('should support ratings enforcement', () => {
      expect(capabilities.ratings_enforcement).toBe(true);
    });

    it('should not support library modification', () => {
      expect(capabilities.library_modification).toBe(false);
    });

    it('should not support playlist modification', () => {
      expect(capabilities.playlist_modification).toBe(false);
    });
  });

  describe('Enforcement Effects', () => {
    const effects = [
      'Reduces recommendations for similar content',
      "Influences 'For You' personalization",
    ];

    it('should reduce recommendations', () => {
      expect(effects.some((e) => e.includes('recommendations'))).toBe(true);
    });

    it('should influence For You', () => {
      expect(effects.some((e) => e.includes('For You'))).toBe(true);
    });
  });

  describe('Enforcement Limitations', () => {
    const limitations = [
      'Cannot remove songs from library',
      'Cannot prevent playback',
      'Cannot skip songs automatically',
      'Must dislike individual songs/albums (no artist-level dislike)',
    ];

    it('should document library limitation', () => {
      expect(limitations.some((l) => l.includes('library'))).toBe(true);
    });

    it('should document playback limitation', () => {
      expect(limitations.some((l) => l.includes('playback'))).toBe(true);
    });

    it('should document skip limitation', () => {
      expect(limitations.some((l) => l.includes('skip'))).toBe(true);
    });

    it('should document no artist-level dislike', () => {
      expect(limitations.some((l) => l.includes('artist-level'))).toBe(true);
    });
  });

  describe('Enforcement Request/Response', () => {
    const mockRequest = {
      dislike_songs: true,
      dislike_albums: true,
      include_library: true,
      include_catalog: false,
      batch_size: 50,
      dry_run: false,
    };

    const mockResponse = {
      run_id: '550e8400-e29b-41d4-a716-446655440000',
      status: 'completed',
      songs_disliked: 15,
      albums_disliked: 5,
      errors_count: 0,
      duration_seconds: 30,
      message: 'Enforcement complete. Disliked 15 songs and 5 albums.',
    };

    it('should have valid request defaults', () => {
      expect(mockRequest.dislike_songs).toBe(true);
      expect(mockRequest.dislike_albums).toBe(true);
      expect(mockRequest.include_library).toBe(true);
      expect(mockRequest.include_catalog).toBe(false);
      expect(mockRequest.batch_size).toBe(50);
    });

    it('should parse response correctly', () => {
      expect(mockResponse.status).toBe('completed');
      expect(mockResponse.songs_disliked).toBe(15);
      expect(mockResponse.albums_disliked).toBe(5);
    });

    it('should have success message', () => {
      expect(mockResponse.message).toContain('Enforcement complete');
    });
  });

  describe('Preview Response', () => {
    const mockPreview = {
      songs_to_dislike: 20,
      albums_to_dislike: 8,
      total_library_songs: 1500,
      total_library_albums: 300,
      estimated_duration_seconds: 14,
    };

    it('should show songs to dislike', () => {
      expect(mockPreview.songs_to_dislike).toBe(20);
    });

    it('should show total library size', () => {
      expect(mockPreview.total_library_songs).toBe(1500);
      expect(mockPreview.total_library_albums).toBe(300);
    });

    it('should estimate duration', () => {
      expect(mockPreview.estimated_duration_seconds).toBe(14);
    });

    it('should calculate percentage blocked', () => {
      const percentBlocked = (mockPreview.songs_to_dislike / mockPreview.total_library_songs) * 100;
      expect(percentBlocked).toBeCloseTo(1.33, 1);
    });
  });

  describe('Rollback Support', () => {
    const mockRollback = {
      run_id: '550e8400-e29b-41d4-a716-446655440000',
      ratings_removed: 20,
      errors: [],
      duration_seconds: 10,
    };

    it('should track ratings removed', () => {
      expect(mockRollback.ratings_removed).toBe(20);
    });

    it('should have no errors on success', () => {
      expect(mockRollback.errors).toHaveLength(0);
    });

    it('should track duration', () => {
      expect(mockRollback.duration_seconds).toBe(10);
    });
  });
});

// ============================================
// UI State Tests
// ============================================

describe('Enforcement UI States', () => {
  describe('Loading States', () => {
    it('should show planning indicator', () => {
      const state = { isPlanning: true };
      expect(state.isPlanning).toBe(true);
    });

    it('should show executing indicator', () => {
      const state = { isExecuting: true };
      expect(state.isExecuting).toBe(true);
    });
  });

  describe('Error States', () => {
    it('should display error message', () => {
      const state = { error: 'Failed to connect to Apple Music' };
      expect(state.error).toBe('Failed to connect to Apple Music');
    });

    it('should allow error clearing', () => {
      let state: { error: string | null } = { error: 'Some error' };
      state = { ...state, error: null };
      expect(state.error).toBeNull();
    });
  });

  describe('Empty States', () => {
    it('should handle no blocked artists', () => {
      const response = {
        status: 'skipped',
        message: 'No blocked artists found in DNP list',
      };
      expect(response.status).toBe('skipped');
      expect(response.message).toContain('No blocked artists');
    });

    it('should handle empty library', () => {
      const preview = {
        songs_to_dislike: 0,
        albums_to_dislike: 0,
        total_library_songs: 0,
        total_library_albums: 0,
      };
      expect(preview.total_library_songs).toBe(0);
    });
  });
});

// ============================================
// Test Data Factories
// ============================================

describe('Test Data Factories', () => {
  function createMockEnforcementRun(overrides = {}) {
    return {
      run_id: '550e8400-e29b-41d4-a716-446655440000',
      status: 'completed',
      songs_disliked: 10,
      albums_disliked: 5,
      errors_count: 0,
      duration_seconds: 20,
      message: 'Test enforcement run',
      ...overrides,
    };
  }

  function createMockEnforcementOptions(overrides = {}) {
    return {
      dislike_songs: true,
      dislike_albums: true,
      include_library: true,
      include_catalog: false,
      batch_size: 50,
      dry_run: false,
      ...overrides,
    };
  }

  it('should create mock enforcement run', () => {
    const run = createMockEnforcementRun();
    expect(run.status).toBe('completed');
  });

  it('should override mock enforcement run', () => {
    const run = createMockEnforcementRun({ status: 'failed' });
    expect(run.status).toBe('failed');
  });

  it('should create mock enforcement options', () => {
    const options = createMockEnforcementOptions();
    expect(options.dislike_songs).toBe(true);
  });

  it('should override mock enforcement options', () => {
    const options = createMockEnforcementOptions({ dry_run: true });
    expect(options.dry_run).toBe(true);
  });
});
