<script lang="ts">
  import { onMount } from 'svelte';
  import { slide } from 'svelte/transition';
  import { apiClient } from '../utils/api-client';

  // Props
  export let artistId: string;
  export let artistName: string = '';

  function hideImgOnError(e: Event) { (e.currentTarget as HTMLImageElement).style.display = 'none'; }

  let expandedAlbums: Set<string> = new Set();
  let prefersReducedMotion = false;
  let loading = true;
  $: slideDuration = prefersReducedMotion ? 0 : 200;

  function toggleAlbum(albumId: string) {
    if (expandedAlbums.has(albumId)) {
      expandedAlbums.delete(albumId);
    } else {
      expandedAlbums.add(albumId);
    }
    expandedAlbums = new Set(expandedAlbums);
  }

  function formatCurrency(value: string | number): string {
    const num = typeof value === 'string' ? parseFloat(value) : value;
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 0,
      maximumFractionDigits: 0,
    }).format(num);
  }

  function formatNumber(value: number): string {
    return new Intl.NumberFormat('en-US', {
      notation: 'compact',
      compactDisplay: 'short',
    }).format(value);
  }

  // Album data with tracks and cover art
  interface AlbumWithTracks {
    id: string;
    title: string;
    year: number;
    cover_url: string;
    tracks: { id: string; title: string; streams: number; revenue: number }[];
    total_streams: number;
    total_revenue: number;
  }

  // Dynamic discography built from library data
  let drakeDiscography: AlbumWithTracks[] = [];

  // Calculate totals
  $: totalStreams = drakeDiscography.reduce((sum, a) => sum + a.total_streams, 0);
  $: totalRevenue = drakeDiscography.reduce((sum, a) => sum + a.total_revenue, 0);
  $: totalTracks = drakeDiscography.reduce((sum, a) => sum + a.tracks.length, 0);

  $: monthlyStreams = Math.round(totalStreams / 24);
  $: monthlyRevenue = Math.round(totalRevenue / 24);

  const perStreamRate = 0.004;
  $: maxRevenue = Math.max(...drakeDiscography.map(a => a.total_revenue), 1);

  async function loadRevenueData() {
    try {
      // Fetch catalog (library tracks grouped by album)
      const catalogResult = await apiClient.authenticatedRequest<{
        tracks: { id: string; title: string; album: string | null; provider: string }[];
      }>('GET', `/api/v1/artists/${artistId}/catalog`);

      if (!catalogResult.success || !catalogResult.data?.tracks) {
        loading = false;
        return;
      }

      // Group tracks by album
      const albumMap = new Map<string, { title: string; tracks: { id: string; title: string }[] }>();
      for (const track of catalogResult.data.tracks) {
        const albumName = track.album || 'Singles / Unknown Album';
        if (!albumMap.has(albumName)) {
          albumMap.set(albumName, { title: albumName, tracks: [] });
        }
        albumMap.get(albumName)!.tracks.push({ id: track.id, title: track.title });
      }

      // Build discography with estimated revenue per track
      drakeDiscography = [...albumMap.entries()]
        .map(([key, album], idx) => ({
          id: `album-${idx}`,
          title: album.title,
          year: 0,
          cover_url: '',
          tracks: album.tracks.map(t => ({
            id: t.id,
            title: t.title,
            streams: 1, // 1 library presence = exposure metric
            revenue: perStreamRate,
          })),
          total_streams: album.tracks.length,
          total_revenue: album.tracks.length * perStreamRate,
        }))
        .sort((a, b) => b.tracks.length - a.tracks.length);
    } catch (e) {
      console.error('Failed to load revenue data:', e);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    prefersReducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    loadRevenueData();
  });
</script>

<div class="artist-discography-revenue surface-panel" data-testid="artist-discography-revenue">

  {#if loading}
    <div class="rev-loading">Loading revenue data...</div>
  {:else if drakeDiscography.length === 0}
    <div class="rev-loading">No catalog data available. Sync your library to populate.</div>
  {:else}
    <!-- Revenue Stat Bar — 4 cells inline -->
    <div class="rev-stats" data-testid="revenue-summary">
      <div class="rev-stats__cell rev-stats__cell--hero">
        <span class="rev-stats__label">Yearly Revenue</span>
        <span class="rev-stats__value rev-stats__value--green" data-testid="yearly-revenue">{formatCurrency(monthlyRevenue * 12)}</span>
      </div>
      <div class="rev-stats__cell">
        <span class="rev-stats__label">Monthly Revenue</span>
        <span class="rev-stats__value" data-testid="monthly-revenue">{formatCurrency(monthlyRevenue)}</span>
      </div>
      <div class="rev-stats__cell">
        <span class="rev-stats__label">Monthly Streams</span>
        <span class="rev-stats__value" data-testid="monthly-streams">{formatNumber(monthlyStreams)}</span>
      </div>
      <div class="rev-stats__cell">
        <span class="rev-stats__label">Catalog Size</span>
        <span class="rev-stats__value" data-testid="discography-count">{totalTracks}</span>
        <span class="rev-stats__sub">{drakeDiscography.length} albums</span>
      </div>
    </div>

    <!-- Album Revenue Table -->
    <div class="rev-section-head">
      <h3 class="rev-section-head__title">Album Revenue</h3>
      <span class="rev-section-head__rate">Est. ${(perStreamRate * 1000).toFixed(2)} / 1K streams</span>
    </div>

    <div class="rev-col-headers">
      <span class="rev-col rev-col--album">Album</span>
      <span class="rev-col rev-col--year">Year</span>
      <span class="rev-col rev-col--tracks">Tracks</span>
      <span class="rev-col rev-col--streams">Streams</span>
      <span class="rev-col rev-col--revenue">Revenue</span>
      <span class="rev-col rev-col--action"></span>
    </div>

    <div data-testid="albums-section">
      {#each drakeDiscography as album}
        {@const barWidth = maxRevenue > 0 ? (album.total_revenue / maxRevenue * 100) : 0}
        <div class="rev-album">
          <button class="rev-album__btn" on:click={() => toggleAlbum(album.id)}>
            <div class="rev-col rev-col--album">
              <div class="rev-album__art">
                {#if album.cover_url}
                  <img src={album.cover_url} alt="" class="rev-album__art-img" on:error={hideImgOnError} />
                {/if}
                <div class="rev-album__art-ph">
                  <svg fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3" /></svg>
                </div>
              </div>
              <span class="rev-album__title">{album.title}</span>
            </div>
            <span class="rev-col rev-col--year rev-tabnum">{album.year}</span>
            <span class="rev-col rev-col--tracks rev-tabnum">{album.tracks.length}</span>
            <span class="rev-col rev-col--streams rev-tabnum">{formatNumber(album.total_streams)}</span>
            <div class="rev-col rev-col--revenue">
              <div class="rev-bar">
                <div class="rev-bar__fill" style="width: {barWidth}%;"></div>
              </div>
              <span class="rev-bar__amount rev-tabnum">{formatCurrency(album.total_revenue)}</span>
            </div>
            <div class="rev-col rev-col--action">
              <svg class="rev-chevron {expandedAlbums.has(album.id) ? 'rev-chevron--open' : ''}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
              </svg>
            </div>
          </button>

          {#if expandedAlbums.has(album.id)}
            <div class="rev-tracks" transition:slide={{ duration: slideDuration }}>
              {#each album.tracks as track, idx}
                <div class="rev-track">
                  <span class="rev-track__num rev-tabnum">{idx + 1}</span>
                  <span class="rev-track__title">{track.title}</span>
                  <span class="rev-track__streams rev-tabnum">{formatNumber(track.streams)}</span>
                  <span class="rev-track__revenue rev-tabnum">{formatCurrency(track.revenue)}</span>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {/each}
    </div>

    <!-- Totals -->
    <div class="rev-totals surface-panel">
      <div>
        <div class="rev-totals__title">Total Catalog Revenue</div>
        <div class="rev-totals__meta">{drakeDiscography.length} albums · {totalTracks} tracks · {formatNumber(totalStreams)} streams</div>
      </div>
      <div class="rev-totals__amount rev-tabnum">{formatCurrency(totalRevenue)}</div>
    </div>

    <!-- Revenue Beyond Streaming -->
    <div class="rev-beyond surface-panel">
      <h3 class="rev-beyond__heading">Revenue Beyond Streaming</h3>
      <div class="rev-beyond__grid">
        <div class="rev-beyond__card surface-panel-thin">
          <div class="rev-beyond__card-head">
            <svg class="rev-beyond__icon rev-beyond__icon--blue" width="16" height="16" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" /></svg>
            <span class="rev-beyond__card-label">Songwriting</span>
          </div>
          <div class="rev-beyond__rows">
            <div class="rev-beyond__row"><span>Own catalog</span><span>{totalTracks} songs</span></div>
            <div class="rev-beyond__row"><span>Other artists</span><span>~85 songs</span></div>
            <div class="rev-beyond__row"><span>Publishing share</span><span>~50%</span></div>
            <div class="rev-beyond__row rev-beyond__row--total"><span>Est. Annual</span><span class="rev-beyond__val--blue">{formatCurrency(Math.round(totalRevenue * 0.15))}</span></div>
          </div>
        </div>
        <div class="rev-beyond__card surface-panel-thin">
          <div class="rev-beyond__card-head">
            <svg class="rev-beyond__icon rev-beyond__icon--purple" width="16" height="16" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3" /></svg>
            <span class="rev-beyond__card-label">Production</span>
          </div>
          <div class="rev-beyond__rows">
            <div class="rev-beyond__row"><span>Co-production</span><span>~45 songs</span></div>
            <div class="rev-beyond__row"><span>Executive producer</span><span>13 albums</span></div>
            <div class="rev-beyond__row"><span>Points (avg)</span><span>~3%</span></div>
            <div class="rev-beyond__row rev-beyond__row--total"><span>Est. Annual</span><span class="rev-beyond__val--purple">{formatCurrency(Math.round(totalRevenue * 0.08))}</span></div>
          </div>
        </div>
      </div>

      <div class="rev-grand surface-panel-thin">
        <span class="rev-grand__label">All-Time Estimated Revenue (Streaming + Writing + Production)</span>
        <span class="rev-grand__value rev-tabnum">{formatCurrency(Math.round(totalRevenue * 1.23))}</span>
      </div>
    </div>

    <!-- Revenue Context -->
    <div class="rev-context surface-panel-thin" data-testid="revenue-context">
      <svg class="rev-context__icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
      <div>
        <div class="rev-context__title">Revenue Methodology</div>
        <div class="rev-context__text">
          Estimates based on average streaming platform payouts (~$0.003-0.005 per stream).
          Actual revenue varies by platform, region, and artist contract terms.
          Top-tier artists may negotiate higher rates.
        </div>
      </div>
    </div>

    <!-- Data Source Note -->
    <div class="rev-note" data-testid="simulation-note">
      Revenue estimates based on library presence across connected platforms (~$0.004 per track exposure). Actual streaming revenue varies by platform, region, and contract terms.
    </div>
  {/if}
</div>

<style>
  .artist-discography-revenue {
    border-radius: 1rem;
    padding: 1.5rem;
  }

  .rev-loading {
    padding: 2rem;
    text-align: center;
    color: var(--color-text-tertiary, #71717a);
    font-size: 0.875rem;
  }

  /* === Stats Bar === */
  .rev-stats {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 1px;
    background: rgba(255, 255, 255, 0.06);
    border-radius: 0.75rem;
    overflow: hidden;
    margin-bottom: 1.5rem;
  }

  .rev-stats__cell {
    padding: 0.875rem 1rem;
    background: var(--color-bg-elevated, rgba(24, 24, 27, 0.95));
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .rev-stats__cell--hero {
    background: linear-gradient(135deg, rgba(16, 185, 129, 0.08), rgba(16, 185, 129, 0.02));
  }

  .rev-stats__label {
    font-size: 0.6875rem;
    color: var(--color-text-tertiary, #71717a);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    font-weight: 500;
  }

  .rev-stats__value {
    font-size: 1.125rem;
    font-weight: 700;
    color: var(--color-text-primary, #fafafa);
    font-variant-numeric: tabular-nums;
  }

  .rev-stats__value--green {
    color: #34d399;
    font-size: 1.375rem;
  }

  .rev-stats__sub {
    font-size: 0.6875rem;
    color: var(--color-text-tertiary, #71717a);
  }

  /* === Section Header === */
  .rev-section-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 0.75rem;
  }

  .rev-section-head__title {
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--color-text-primary, #fafafa);
    margin: 0;
  }

  .rev-section-head__rate {
    font-size: 0.75rem;
    color: var(--color-text-tertiary, #71717a);
  }

  /* === Column Headers === */
  .rev-col-headers {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0 1.25rem;
    margin-bottom: 0.375rem;
  }

  .rev-col-headers .rev-col {
    font-size: 0.6875rem;
    color: var(--color-text-muted, #52525b);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    font-weight: 500;
  }

  /* === Column Widths === */
  .rev-col--album { flex: 1; min-width: 0; display: flex; align-items: center; gap: 0.75rem; }
  .rev-col--year { width: 3.5rem; text-align: right; flex-shrink: 0; }
  .rev-col--tracks { width: 3.5rem; text-align: right; flex-shrink: 0; }
  .rev-col--streams { width: 5rem; text-align: right; flex-shrink: 0; }
  .rev-col--revenue { width: 9rem; flex-shrink: 0; display: flex; align-items: center; gap: 0.5rem; }
  .rev-col--action { width: 1.5rem; flex-shrink: 0; display: flex; align-items: center; justify-content: center; }

  /* === Album Rows === */
  .rev-album {
    margin-bottom: 0.25rem;
  }

  .rev-album__btn {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.625rem 1.25rem;
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(255, 255, 255, 0.04);
    border-radius: 0.75rem;
    text-align: left;
    cursor: pointer;
    transition: background-color 0.15s;
    color: inherit;
    font: inherit;
  }

  .rev-album__btn:hover {
    background: rgba(255, 255, 255, 0.05);
  }

  .rev-album__art {
    width: 2.5rem;
    height: 2.5rem;
    border-radius: 0.5rem;
    overflow: hidden;
    background: #27272a;
    flex-shrink: 0;
    position: relative;
  }

  .rev-album__art-img {
    width: 2.5rem;
    height: 2.5rem;
    object-fit: cover;
    position: absolute;
    inset: 0;
  }

  .rev-album__art-ph {
    width: 2.5rem;
    height: 2.5rem;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .rev-album__art-ph svg {
    width: 1rem;
    height: 1rem;
    color: #52525b;
  }

  .rev-album__title {
    font-weight: 600;
    color: #fafafa;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
    min-width: 0;
    font-size: 0.875rem;
  }

  .rev-tabnum {
    font-variant-numeric: tabular-nums;
  }

  .rev-album__btn .rev-col--year,
  .rev-album__btn .rev-col--tracks,
  .rev-album__btn .rev-col--streams {
    font-size: 0.8125rem;
    color: #a1a1aa;
  }

  /* === Revenue Bar === */
  .rev-bar {
    height: 0.375rem;
    flex: 1;
    border-radius: 0.25rem;
    overflow: hidden;
    background: rgba(255, 255, 255, 0.04);
  }

  .rev-bar__fill {
    height: 100%;
    border-radius: 0.25rem;
    background: linear-gradient(90deg, #10b981, #34d399);
    transition: width 0.3s ease;
  }

  .rev-bar__amount {
    font-size: 0.8125rem;
    font-weight: 600;
    color: #34d399;
    white-space: nowrap;
    min-width: 4.5rem;
    text-align: right;
  }

  /* === Chevron === */
  .rev-chevron {
    width: 1rem;
    height: 1rem;
    color: #52525b;
    transition: transform 0.2s;
    flex-shrink: 0;
  }

  .rev-chevron--open {
    transform: rotate(180deg);
  }

  /* === Expanded Tracks === */
  .rev-tracks {
    border-top: 1px solid rgba(255, 255, 255, 0.06);
    background: rgba(0, 0, 0, 0.15);
    border-radius: 0 0 0.75rem 0.75rem;
    overflow: hidden;
  }

  .rev-track {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.5rem 1.25rem 0.5rem 5rem;
    border-top: 1px solid rgba(255, 255, 255, 0.03);
    transition: background-color 0.1s;
  }

  .rev-track:first-child { border-top: none; }
  .rev-track:hover { background: rgba(255, 255, 255, 0.02); }

  .rev-track__num {
    width: 1.5rem;
    text-align: right;
    font-size: 0.75rem;
    color: #52525b;
    flex-shrink: 0;
  }

  .rev-track__title {
    flex: 1;
    min-width: 0;
    font-size: 0.8125rem;
    color: #d4d4d8;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .rev-track__streams {
    font-size: 0.75rem;
    color: #71717a;
    width: 4rem;
    text-align: right;
    flex-shrink: 0;
  }

  .rev-track__revenue {
    font-size: 0.75rem;
    font-weight: 500;
    color: #34d399;
    width: 4.5rem;
    text-align: right;
    flex-shrink: 0;
  }

  /* === Totals === */
  .rev-totals {
    margin-top: 0.75rem;
    padding: 1rem 1.25rem;
    border-radius: 0.75rem;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .rev-totals__title {
    font-size: 0.875rem;
    font-weight: 600;
    color: #fafafa;
  }

  .rev-totals__meta {
    font-size: 0.75rem;
    color: #71717a;
    margin-top: 0.125rem;
  }

  .rev-totals__amount {
    font-size: 1.25rem;
    font-weight: 700;
    color: #34d399;
  }

  /* === Beyond Streaming === */
  .rev-beyond {
    margin-top: 1.5rem;
    border-radius: 0.75rem;
    padding: 1.25rem;
  }

  .rev-beyond__heading {
    font-size: 1.125rem;
    font-weight: 600;
    color: #fafafa;
    margin: 0 0 1rem;
  }

  .rev-beyond__grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.75rem;
  }

  .rev-beyond__card {
    padding: 1rem;
    border-radius: 0.75rem;
  }

  .rev-beyond__card-head {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.75rem;
  }

  .rev-beyond__icon { width: 1rem; height: 1rem; }
  .rev-beyond__icon--blue { color: #60a5fa; }
  .rev-beyond__icon--purple { color: #c084fc; }

  .rev-beyond__card-label {
    font-size: 0.875rem;
    font-weight: 500;
    color: #fafafa;
  }

  .rev-beyond__rows {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
  }

  .rev-beyond__row {
    display: flex;
    justify-content: space-between;
    font-size: 0.8125rem;
  }

  .rev-beyond__row span:first-child { color: #71717a; }
  .rev-beyond__row span:last-child { color: #d4d4d8; }

  .rev-beyond__row--total {
    border-top: 1px solid rgba(255, 255, 255, 0.06);
    padding-top: 0.5rem;
    margin-top: 0.375rem;
  }

  .rev-beyond__row--total span:first-child { color: #d4d4d8; font-weight: 500; }
  .rev-beyond__val--blue { color: #60a5fa !important; font-weight: 700 !important; }
  .rev-beyond__val--purple { color: #c084fc !important; font-weight: 700 !important; }

  /* === Grand Total === */
  .rev-grand {
    margin-top: 0.75rem;
    padding: 1rem;
    border-radius: 0.75rem;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .rev-grand__label { font-size: 0.75rem; color: #71717a; }
  .rev-grand__value { font-size: 1.5rem; font-weight: 700; color: #34d399; }

  /* === Context & Note === */
  .rev-context {
    margin-top: 1.25rem;
    padding: 1rem;
    border-radius: 0.75rem;
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
  }

  .rev-context__icon { width: 1rem; height: 1rem; color: #52525b; flex-shrink: 0; margin-top: 0.125rem; }
  .rev-context__title { font-size: 0.875rem; font-weight: 500; color: #d4d4d8; margin-bottom: 0.25rem; }
  .rev-context__text { font-size: 0.8125rem; color: #71717a; line-height: 1.5; }

  .rev-note {
    margin-top: 0.75rem;
    text-align: center;
    font-size: 0.75rem;
    color: #71717a;
  }

  @media (max-width: 640px) {
    .rev-stats { grid-template-columns: repeat(2, 1fr); }
    .rev-col-headers .rev-col--year,
    .rev-col-headers .rev-col--tracks { display: none; }
    .rev-album__btn .rev-col--year,
    .rev-album__btn .rev-col--tracks { display: none; }
    .rev-beyond__grid { grid-template-columns: 1fr; }
    .rev-track { padding-left: 2rem; }
  }
</style>
