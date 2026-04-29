import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor, within } from '@testing-library/svelte';

const mockCatalogResponse = {
  success: true,
  data: {
    tracks: [
      {
        id: 'track-1',
        title: 'Virginia Beach',
        album: 'For All The Dogs',
        albumCover: '',
        year: 2023,
        duration: '4:12',
      },
      {
        id: 'track-2',
        title: 'First Person Shooter',
        album: 'For All The Dogs',
        albumCover: '',
        year: 2023,
        duration: '4:07',
      },
      {
        id: 'track-3',
        title: 'Jimmy Cooks',
        album: 'Honestly, Nevermind',
        albumCover: '',
        year: 2022,
        duration: '3:38',
      },
    ],
  },
};

const renderComponent = async () => {
  const { default: ArtistDiscographyRevenue } = await import('../ArtistDiscographyRevenue.svelte');
  const rendered = render(ArtistDiscographyRevenue, {
    artistId: 'artist-123',
    artistName: 'Drake',
  });
  await (rendered.component as { refreshRevenueData: () => Promise<void> }).refreshRevenueData();
  return rendered;
};

describe('ArtistDiscographyRevenue', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Remove random variance in revenue projections for deterministic assertions.
    vi.spyOn(Math, 'random').mockReturnValue(0);
    vi.mocked(globalThis.fetch).mockImplementation(async () =>
      new Response(JSON.stringify(mockCatalogResponse), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    );
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('renders the current summary metrics from the simulated catalog', async () => {
    await renderComponent();

    await waitFor(() => {
      expect(screen.getByTestId('monthly-revenue')).toHaveTextContent('$');
    });

    expect(screen.getByTestId('yearly-revenue')).toHaveTextContent('$');
    expect(screen.getByTestId('monthly-streams')).not.toHaveTextContent('0');
    expect(screen.getByTestId('discography-count')).toHaveTextContent('3');
    expect(screen.getByText('2 albums')).toBeInTheDocument();
  });

  it('renders the album revenue breakdown using the current Drake catalog', async () => {
    await renderComponent();

    await waitFor(() => {
      expect(screen.getByText('Album Revenue')).toBeInTheDocument();
    });
    expect(screen.getByText('For All The Dogs')).toBeInTheDocument();
    expect(screen.getByText('Est. $4.00 / 1K streams')).toBeInTheDocument();

    const albumsSection = screen.getByTestId('albums-section');
    expect(within(albumsSection).getAllByRole('button')).toHaveLength(2);
  });

  it('expands an album to show the current track list and totals', async () => {
    await renderComponent();

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /for all the dogs/i })).toBeInTheDocument();
    });

    await fireEvent.click(screen.getByRole('button', { name: /for all the dogs/i }));

    await waitFor(() => {
      expect(screen.getByText('Virginia Beach')).toBeInTheDocument();
      expect(screen.getByText('First Person Shooter')).toBeInTheDocument();
    });
  });

  it('does not render the old offenses section in the simulated catalog view', async () => {
    await renderComponent();

    await waitFor(() => {
      expect(screen.getByTestId('revenue-summary')).toBeInTheDocument();
    });

    expect(screen.queryByTestId('offenses-section')).not.toBeInTheDocument();
    expect(screen.queryByTestId('offense-item')).not.toBeInTheDocument();
  });

  it('renders the current revenue context and simulation note', async () => {
    await renderComponent();

    await waitFor(() => {
      expect(screen.getByTestId('revenue-context')).toBeInTheDocument();
    });

    expect(screen.getByTestId('revenue-context')).toHaveTextContent(
      'Revenue Methodology'
    );
    expect(screen.getByTestId('simulation-note')).toHaveTextContent(
      'Revenue estimates based on library presence across connected platforms'
    );
  });
});
