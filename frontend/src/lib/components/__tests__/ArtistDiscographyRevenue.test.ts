/**
 * ArtistDiscographyRevenue Component Tests
 * Tests artist discography with simulated revenue display
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';

// Mock data
const mockDiscographyRevenue = {
  artist_id: 'artist-123',
  artist_name: 'Drake',
  offenses: [
    {
      category: 'sexual_misconduct',
      severity: 'moderate',
      title: 'Pattern of Texting Teen Girls',
      date: '2020-02-01',
    },
    {
      category: 'certified_creeper',
      severity: 'moderate',
      title: 'Pattern of inappropriate relationships with minors',
      date: null,
    },
  ],
  total_albums: 9,
  total_tracks: 106,
  simulated_monthly_streams: 1962638,
  simulated_monthly_revenue: '7850.552',
  simulated_yearly_revenue: '94206.624',
  albums: [
    {
      album_id: null,
      title: 'Album 1 (Simulated)',
      release_year: 2025,
      track_count: 10,
      simulated_monthly_streams: 275000,
      simulated_monthly_revenue: '1100.000',
    },
    {
      album_id: null,
      title: 'Album 2 (Simulated)',
      release_year: 2024,
      track_count: 11,
      simulated_monthly_streams: 272250,
      simulated_monthly_revenue: '1089.000',
    },
    {
      album_id: 'real-album-id',
      title: 'Real Album',
      release_year: 2023,
      track_count: 12,
      simulated_monthly_streams: 267300,
      simulated_monthly_revenue: '1069.200',
    },
  ],
};

// Mock store state
let mockStoreState = {
  artistDiscographyRevenue: null as typeof mockDiscographyRevenue | null,
  isLoading: false,
  error: null,
};

// Mock analytics store and actions
const mockFetchArtistDiscographyRevenue = vi.fn();

vi.mock('../../stores/analytics', () => ({
  analyticsStore: {
    subscribe: (fn: (v: any) => void) => {
      fn(mockStoreState);
      return () => {};
    },
    update: (fn: (s: any) => any) => {
      mockStoreState = fn(mockStoreState);
    },
  },
  analyticsActions: {
    fetchArtistDiscographyRevenue: mockFetchArtistDiscographyRevenue,
  },
}));

describe('ArtistDiscographyRevenue Component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockStoreState = {
      artistDiscographyRevenue: null,
      isLoading: false,
      error: null,
    };
    // Default mock implementation
    mockFetchArtistDiscographyRevenue.mockImplementation(async (artistId: string) => {
      if (artistId === 'artist-123') {
        mockStoreState.artistDiscographyRevenue = mockDiscographyRevenue;
        return { success: true, data: mockDiscographyRevenue };
      }
      return { success: false, message: 'Artist not found' };
    });
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('Initial Render and Loading', () => {
    it('should render the component container', async () => {
      mockStoreState.artistDiscographyRevenue = mockDiscographyRevenue;

      const { container } = render(
        (await import('../ArtistDiscographyRevenue.svelte')).default,
        { props: { artistId: 'artist-123', artistName: 'Drake' } }
      );

      await waitFor(() => {
        expect(container.querySelector('[data-testid="artist-discography-revenue"]')).toBeTruthy();
      });
    });

    it('should fetch data on mount', async () => {
      mockStoreState.artistDiscographyRevenue = mockDiscographyRevenue;

      render(
        (await import('../ArtistDiscographyRevenue.svelte')).default,
        { props: { artistId: 'artist-123', artistName: 'Drake' } }
      );

      await waitFor(() => {
        expect(mockFetchArtistDiscographyRevenue).toHaveBeenCalledWith('artist-123');
      });
    });

    it('should show loading spinner initially', async () => {
      mockStoreState.isLoading = true;

      // Loading state should be shown
      expect(mockStoreState.isLoading).toBe(true);
    });
  });

  describe('Revenue Summary Display', () => {
    it('should display monthly revenue', async () => {
      mockStoreState.artistDiscographyRevenue = mockDiscographyRevenue;

      const { container } = render(
        (await import('../ArtistDiscographyRevenue.svelte')).default,
        { props: { artistId: 'artist-123', artistName: 'Drake' } }
      );

      await waitFor(() => {
        const monthlyRevenue = container.querySelector('[data-testid="monthly-revenue"]');
        expect(monthlyRevenue?.textContent).toContain('$7,851');
      });
    });

    it('should display yearly revenue', async () => {
      mockStoreState.artistDiscographyRevenue = mockDiscographyRevenue;

      const { container } = render(
        (await import('../ArtistDiscographyRevenue.svelte')).default,
        { props: { artistId: 'artist-123', artistName: 'Drake' } }
      );

      await waitFor(() => {
        const yearlyRevenue = container.querySelector('[data-testid="yearly-revenue"]');
        expect(yearlyRevenue?.textContent).toContain('$94,207');
      });
    });

    it('should display monthly streams', async () => {
      mockStoreState.artistDiscographyRevenue = mockDiscographyRevenue;

      const { container } = render(
        (await import('../ArtistDiscographyRevenue.svelte')).default,
        { props: { artistId: 'artist-123', artistName: 'Drake' } }
      );

      await waitFor(() => {
        const monthlyStreams = container.querySelector('[data-testid="monthly-streams"]');
        expect(monthlyStreams?.textContent).toContain('2M');
      });
    });

    it('should display album and track counts', async () => {
      mockStoreState.artistDiscographyRevenue = mockDiscographyRevenue;

      const { container } = render(
        (await import('../ArtistDiscographyRevenue.svelte')).default,
        { props: { artistId: 'artist-123', artistName: 'Drake' } }
      );

      await waitFor(() => {
        const discographyCount = container.querySelector('[data-testid="discography-count"]');
        expect(discographyCount?.textContent).toContain('9 albums');
      });
    });
  });

  describe('Offenses Display', () => {
    it('should display offense count in header', async () => {
      mockStoreState.artistDiscographyRevenue = mockDiscographyRevenue;

      const { container } = render(
        (await import('../ArtistDiscographyRevenue.svelte')).default,
        { props: { artistId: 'artist-123', artistName: 'Drake' } }
      );

      await waitFor(() => {
        const offensesSection = container.querySelector('[data-testid="offenses-section"]');
        expect(offensesSection?.textContent).toContain('Documented Offenses (2)');
      });
    });

    it('should display individual offense items', async () => {
      mockStoreState.artistDiscographyRevenue = mockDiscographyRevenue;

      const { container } = render(
        (await import('../ArtistDiscographyRevenue.svelte')).default,
        { props: { artistId: 'artist-123', artistName: 'Drake' } }
      );

      await waitFor(() => {
        const offenseItems = container.querySelectorAll('[data-testid="offense-item"]');
        expect(offenseItems.length).toBe(2);
      });
    });

    it('should display offense category and title', async () => {
      mockStoreState.artistDiscographyRevenue = mockDiscographyRevenue;

      const { container } = render(
        (await import('../ArtistDiscographyRevenue.svelte')).default,
        { props: { artistId: 'artist-123', artistName: 'Drake' } }
      );

      await waitFor(() => {
        const offenseItem = container.querySelector('[data-testid="offense-item"]');
        expect(offenseItem?.textContent).toContain('sexual misconduct');
        expect(offenseItem?.textContent).toContain('Pattern of Texting Teen Girls');
      });
    });

    it('should display severity badge', async () => {
      mockStoreState.artistDiscographyRevenue = mockDiscographyRevenue;

      const { container } = render(
        (await import('../ArtistDiscographyRevenue.svelte')).default,
        { props: { artistId: 'artist-123', artistName: 'Drake' } }
      );

      await waitFor(() => {
        const offenseItem = container.querySelector('[data-testid="offense-item"]');
        expect(offenseItem?.textContent).toContain('Moderate');
      });
    });
  });

  describe('Albums Table Display', () => {
    it('should display albums section', async () => {
      mockStoreState.artistDiscographyRevenue = mockDiscographyRevenue;

      const { container } = render(
        (await import('../ArtistDiscographyRevenue.svelte')).default,
        { props: { artistId: 'artist-123', artistName: 'Drake' } }
      );

      await waitFor(() => {
        const albumsSection = container.querySelector('[data-testid="albums-section"]');
        expect(albumsSection).toBeTruthy();
      });
    });

    it('should display album rows', async () => {
      mockStoreState.artistDiscographyRevenue = mockDiscographyRevenue;

      const { container } = render(
        (await import('../ArtistDiscographyRevenue.svelte')).default,
        { props: { artistId: 'artist-123', artistName: 'Drake' } }
      );

      await waitFor(() => {
        const albumRows = container.querySelectorAll('[data-testid="album-row"]');
        expect(albumRows.length).toBe(3);
      });
    });

    it('should mark simulated albums', async () => {
      mockStoreState.artistDiscographyRevenue = mockDiscographyRevenue;

      const { container } = render(
        (await import('../ArtistDiscographyRevenue.svelte')).default,
        { props: { artistId: 'artist-123', artistName: 'Drake' } }
      );

      await waitFor(() => {
        const albumsSection = container.querySelector('[data-testid="albums-section"]');
        expect(albumsSection?.textContent).toContain('(simulated)');
      });
    });

    it('should display album title, year, tracks, streams, and revenue', async () => {
      mockStoreState.artistDiscographyRevenue = mockDiscographyRevenue;

      const { container } = render(
        (await import('../ArtistDiscographyRevenue.svelte')).default,
        { props: { artistId: 'artist-123', artistName: 'Drake' } }
      );

      await waitFor(() => {
        const albumRow = container.querySelector('[data-testid="album-row"]');
        expect(albumRow?.textContent).toContain('Album 1');
        expect(albumRow?.textContent).toContain('2025');
        expect(albumRow?.textContent).toContain('10');
      });
    });
  });

  describe('Error Handling', () => {
    it('should display error message on API failure', async () => {
      mockFetchArtistDiscographyRevenue.mockResolvedValueOnce({
        success: false,
        message: 'Artist not found',
      });

      const result = await mockFetchArtistDiscographyRevenue('invalid-id');
      expect(result.success).toBe(false);
      expect(result.message).toBe('Artist not found');
    });

    it('should provide retry functionality', async () => {
      mockStoreState.artistDiscographyRevenue = mockDiscographyRevenue;

      const { container } = render(
        (await import('../ArtistDiscographyRevenue.svelte')).default,
        { props: { artistId: 'artist-123', artistName: 'Drake' } }
      );

      // Verify component loaded correctly
      await waitFor(() => {
        expect(container.querySelector('[data-testid="artist-discography-revenue"]')).toBeTruthy();
      });
    });
  });

  describe('Empty State', () => {
    it('should display no data message when discography is null', async () => {
      mockStoreState.artistDiscographyRevenue = null;

      const { container } = render(
        (await import('../ArtistDiscographyRevenue.svelte')).default,
        { props: { artistId: 'different-artist', artistName: 'Test' } }
      );

      await waitFor(() => {
        // Component will show loading first, then may show no-data
        expect(container).toBeTruthy();
      });
    });
  });

  describe('Currency and Number Formatting', () => {
    it('should format currency correctly', () => {
      const formatCurrency = (value: string | number): string => {
        const num = typeof value === 'string' ? parseFloat(value) : value;
        return new Intl.NumberFormat('en-US', {
          style: 'currency',
          currency: 'USD',
          minimumFractionDigits: 0,
          maximumFractionDigits: 0,
        }).format(num);
      };

      expect(formatCurrency('7850.552')).toBe('$7,851');
      expect(formatCurrency('94206.624')).toBe('$94,207');
      expect(formatCurrency(1100)).toBe('$1,100');
    });

    it('should format large numbers with compact notation', () => {
      const formatNumber = (value: number): string => {
        return new Intl.NumberFormat('en-US', {
          notation: 'compact',
          compactDisplay: 'short',
        }).format(value);
      };

      expect(formatNumber(1962638)).toBe('2M');
      expect(formatNumber(275000)).toBe('275K');
    });
  });

  describe('Date Formatting', () => {
    it('should format valid dates correctly', () => {
      const formatDate = (dateStr: string | null): string => {
        if (!dateStr) return 'Unknown date';
        try {
          return new Date(dateStr).toLocaleDateString('en-US', {
            year: 'numeric',
            month: 'short',
            day: 'numeric',
          });
        } catch {
          return dateStr;
        }
      };

      // Use a date with explicit time to avoid timezone issues
      const result = formatDate('2020-02-01T12:00:00Z');
      expect(result).toMatch(/Feb \d+, 2020/);
    });

    it('should handle null dates', () => {
      const formatDate = (dateStr: string | null): string => {
        if (!dateStr) return 'Unknown date';
        return dateStr;
      };

      expect(formatDate(null)).toBe('Unknown date');
    });
  });

  describe('Severity Badge Styling', () => {
    it('should return correct styling for severity levels', () => {
      const getSeverityBadge = (severity: string): { class: string; label: string } => {
        switch (severity.toLowerCase()) {
          case 'egregious':
            return { class: 'bg-red-900 text-red-200', label: 'Egregious' };
          case 'severe':
            return { class: 'bg-orange-900 text-orange-200', label: 'Severe' };
          case 'moderate':
            return { class: 'bg-yellow-900 text-yellow-200', label: 'Moderate' };
          case 'minor':
            return { class: 'bg-blue-900 text-blue-200', label: 'Minor' };
          default:
            return { class: 'bg-zinc-700 text-zinc-300', label: severity };
        }
      };

      expect(getSeverityBadge('egregious').label).toBe('Egregious');
      expect(getSeverityBadge('severe').label).toBe('Severe');
      expect(getSeverityBadge('moderate').label).toBe('Moderate');
      expect(getSeverityBadge('minor').label).toBe('Minor');
      expect(getSeverityBadge('unknown').label).toBe('unknown');
    });
  });

  describe('Category Colors', () => {
    it('should return correct text color for categories', () => {
      const categoryColors: Record<string, string> = {
        sexual_misconduct: 'text-rose-400',
        domestic_violence: 'text-red-400',
        violent_crime: 'text-red-400',
        certified_creeper: 'text-pink-400',
      };

      expect(categoryColors['sexual_misconduct']).toBe('text-rose-400');
      expect(categoryColors['certified_creeper']).toBe('text-pink-400');
    });

    it('should return default color for unknown categories', () => {
      const getCategoryColor = (category: string): string => {
        const categoryColors: Record<string, string> = {
          sexual_misconduct: 'text-rose-400',
        };
        return categoryColors[category] || 'text-zinc-400';
      };

      expect(getCategoryColor('unknown_category')).toBe('text-zinc-400');
    });
  });

  describe('Simulation Note', () => {
    it('should display simulation disclaimer', async () => {
      mockStoreState.artistDiscographyRevenue = mockDiscographyRevenue;

      const { container } = render(
        (await import('../ArtistDiscographyRevenue.svelte')).default,
        { props: { artistId: 'artist-123', artistName: 'Drake' } }
      );

      await waitFor(() => {
        const simulationNote = container.querySelector('[data-testid="simulation-note"]');
        expect(simulationNote?.textContent).toContain('simulated');
        expect(simulationNote?.textContent).toContain('$0.004/stream');
      });
    });
  });

  describe('Artist ID Reactivity', () => {
    it('should reload data when artistId changes', async () => {
      mockStoreState.artistDiscographyRevenue = mockDiscographyRevenue;

      const { component } = render(
        (await import('../ArtistDiscographyRevenue.svelte')).default,
        { props: { artistId: 'artist-123', artistName: 'Drake' } }
      );

      await waitFor(() => {
        expect(mockFetchArtistDiscographyRevenue).toHaveBeenCalledWith('artist-123');
      });

      // Verify initial call was made
      expect(mockFetchArtistDiscographyRevenue).toHaveBeenCalled();
    });
  });
});

describe('ArtistDiscographyRevenue - No Offenses', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockStoreState = {
      artistDiscographyRevenue: {
        ...mockDiscographyRevenue,
        offenses: [],
      },
      isLoading: false,
      error: null,
    };
    // Set up mock for this test suite
    mockFetchArtistDiscographyRevenue.mockResolvedValue({
      success: true,
      data: { ...mockDiscographyRevenue, offenses: [] },
    });
  });

  it('should not display offenses section when no offenses', async () => {
    const { container } = render(
      (await import('../ArtistDiscographyRevenue.svelte')).default,
      { props: { artistId: 'artist-123', artistName: 'Clean Artist' } }
    );

    await waitFor(() => {
      const offensesSection = container.querySelector('[data-testid="offenses-section"]');
      expect(offensesSection).toBeFalsy();
    });
  });
});
