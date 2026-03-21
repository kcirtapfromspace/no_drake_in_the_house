import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor, within } from '@testing-library/svelte';
import ArtistDiscographyRevenue from '../ArtistDiscographyRevenue.svelte';

const { mockFetchArtistDiscographyRevenue } = vi.hoisted(() => ({
  mockFetchArtistDiscographyRevenue: vi.fn(),
}));

let mockStoreState = {
  artistDiscographyRevenue: {
    artist_id: 'artist-123',
  },
};

vi.mock('../../stores/analytics', () => ({
  analyticsStore: {
    subscribe: (fn: (value: unknown) => void) => {
      fn(mockStoreState);
      return () => {};
    },
  },
  analyticsActions: {
    fetchArtistDiscographyRevenue: mockFetchArtistDiscographyRevenue,
  },
}));

const renderComponent = () =>
  render(ArtistDiscographyRevenue, {
    artistId: 'artist-123',
    artistName: 'Drake',
  });

const waitForCatalog = async () => {
  await waitFor(() => {
    expect(screen.queryByTestId('loading-spinner')).not.toBeInTheDocument();
  });
};

describe('ArtistDiscographyRevenue', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockStoreState = {
      artistDiscographyRevenue: {
        artist_id: 'artist-123',
      },
    };

    mockFetchArtistDiscographyRevenue.mockResolvedValue({
      success: true,
      data: mockStoreState.artistDiscographyRevenue,
    });
  });

  it('fetches artist discography revenue on mount', async () => {
    renderComponent();

    await waitFor(() => {
      expect(mockFetchArtistDiscographyRevenue).toHaveBeenCalledWith('artist-123');
    });
  });

  it('renders the current summary metrics from the simulated catalog', () => {
    renderComponent();
    
    return waitForCatalog().then(() => {
      expect(screen.getByTestId('monthly-revenue')).toHaveTextContent('$8,164,833');
      expect(screen.getByTestId('yearly-revenue')).toHaveTextContent('$97,977,996');
      expect(screen.getByTestId('monthly-streams')).toHaveTextContent('2B');
      expect(screen.getByTestId('discography-count')).toHaveTextContent('108');
      expect(screen.getAllByText('13 albums').length).toBeGreaterThan(0);
    });
  });

  it('renders the album revenue breakdown using the current Drake catalog', async () => {
    renderComponent();
    await waitForCatalog();

    expect(screen.getByText('Album Revenue')).toBeInTheDocument();
    expect(screen.getByText('For All The Dogs')).toBeInTheDocument();
    expect(screen.getByText('Est. $4.00 / 1K streams')).toBeInTheDocument();

    const albumsSection = screen.getByTestId('albums-section');
    expect(within(albumsSection).getAllByRole('button')).toHaveLength(13);
  });

  it('expands an album to show the current track list and totals', async () => {
    renderComponent();
    await waitForCatalog();

    await fireEvent.click(screen.getByRole('button', { name: /for all the dogs/i }));

    await waitFor(() => {
      expect(screen.getByText('Virginia Beach')).toBeInTheDocument();
      expect(screen.getByText('$180,000')).toBeInTheDocument();
    });
  });

  it('does not render the old offenses section in the simulated catalog view', async () => {
    renderComponent();
    await waitForCatalog();

    expect(screen.queryByTestId('offenses-section')).not.toBeInTheDocument();
    expect(screen.queryByTestId('offense-item')).not.toBeInTheDocument();
  });

  it('renders the current revenue context and simulation note', async () => {
    renderComponent();
    await waitForCatalog();

    expect(screen.getByTestId('revenue-context')).toHaveTextContent(
      'Revenue Methodology'
    );
    expect(screen.getByTestId('simulation-note')).toHaveTextContent(
      'Connect streaming accounts for real analytics'
    );
  });
});
