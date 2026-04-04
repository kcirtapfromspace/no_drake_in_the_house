import { describe, it, expect } from 'vitest';
import { render, screen, fireEvent, waitFor, within } from '@testing-library/svelte';
import ArtistDiscographyRevenue from '../ArtistDiscographyRevenue.svelte';

const renderComponent = () =>
  render(ArtistDiscographyRevenue, {
    artistId: 'artist-123',
    artistName: 'Drake',
  });

describe('ArtistDiscographyRevenue', () => {
  it('renders the current summary metrics from the simulated catalog', () => {
    renderComponent();

    expect(screen.getByTestId('monthly-revenue')).toHaveTextContent('$8,164,833');
    expect(screen.getByTestId('yearly-revenue')).toHaveTextContent('$97,977,996');
    expect(screen.getByTestId('monthly-streams')).toHaveTextContent('2B');
    expect(screen.getByTestId('discography-count')).toHaveTextContent('108');
    expect(screen.getAllByText('13 albums').length).toBeGreaterThan(0);
  });

  it('renders the album revenue breakdown using the current Drake catalog', () => {
    renderComponent();

    expect(screen.getByText('Album Revenue')).toBeInTheDocument();
    expect(screen.getByText('For All The Dogs')).toBeInTheDocument();
    expect(screen.getByText('Est. $4.00 / 1K streams')).toBeInTheDocument();

    const albumsSection = screen.getByTestId('albums-section');
    expect(within(albumsSection).getAllByRole('button')).toHaveLength(13);
  });

  it('expands an album to show the current track list and totals', async () => {
    renderComponent();

    await fireEvent.click(screen.getByRole('button', { name: /for all the dogs/i }));

    await waitFor(() => {
      expect(screen.getByText('Virginia Beach')).toBeInTheDocument();
      expect(screen.getByText('$180,000')).toBeInTheDocument();
    });
  });

  it('does not render the old offenses section in the simulated catalog view', () => {
    renderComponent();

    expect(screen.queryByTestId('offenses-section')).not.toBeInTheDocument();
    expect(screen.queryByTestId('offense-item')).not.toBeInTheDocument();
  });

  it('renders the current revenue context and simulation note', () => {
    renderComponent();

    expect(screen.getByTestId('revenue-context')).toHaveTextContent(
      'Revenue Methodology'
    );
    expect(screen.getByTestId('simulation-note')).toHaveTextContent(
      'Connect streaming accounts for real analytics'
    );
  });
});
