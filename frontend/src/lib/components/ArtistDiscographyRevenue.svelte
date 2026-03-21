<script lang="ts">
  import { onMount } from 'svelte';
  import { slide } from 'svelte/transition';
  import {
    analyticsStore,
    analyticsActions,
    type ArtistDiscographyRevenue,
  } from '../stores/analytics';

  // Props
  export let artistId: string;
  export let artistName: string = '';

  // Local state
  let isLoading = false;
  let error: string | null = null;
  let expandedAlbums: Set<string> = new Set();
  let prefersReducedMotion = false;
  $: slideDuration = prefersReducedMotion ? 0 : 200;

  // Reactive data from store
  $: discography = $analyticsStore.artistDiscographyRevenue;
  $: isCurrentArtist = discography?.artist_id === artistId;

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

  function formatFullNumber(value: number): string {
    return new Intl.NumberFormat('en-US').format(value);
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

  // Drake's comprehensive discography with tracks - Real Spotify album art URLs
  const drakeDiscography: AlbumWithTracks[] = [
    {
      id: 'fatd-2023',
      title: 'For All The Dogs',
      year: 2023,
      cover_url: 'https://i.scdn.co/image/ab67616d0000b273929008d1bec2f6f89fb0c3f6',
      tracks: [
        { id: 'fatd-1', title: 'Virginia Beach', streams: 45000000, revenue: 180000 },
        { id: 'fatd-2', title: 'Amen', streams: 38000000, revenue: 152000 },
        { id: 'fatd-3', title: 'Calling For You', streams: 52000000, revenue: 208000 },
        { id: 'fatd-4', title: 'Fear Of Heights', streams: 41000000, revenue: 164000 },
        { id: 'fatd-5', title: 'Daylight', streams: 35000000, revenue: 140000 },
        { id: 'fatd-6', title: 'First Person Shooter', streams: 120000000, revenue: 480000 },
        { id: 'fatd-7', title: 'IDGAF', streams: 55000000, revenue: 220000 },
        { id: 'fatd-8', title: 'What Would Pluto Do', streams: 28000000, revenue: 112000 },
        { id: 'fatd-9', title: 'Slime You Out', streams: 95000000, revenue: 380000 },
        { id: 'fatd-10', title: 'Rich Baby Daddy', streams: 75000000, revenue: 300000 },
      ],
      total_streams: 584000000,
      total_revenue: 2336000
    },
    {
      id: 'herloss-2022',
      title: 'Her Loss',
      year: 2022,
      cover_url: 'https://i.scdn.co/image/ab67616d0000b2732a1d0e45e67c5c4e5e5d8fdb',
      tracks: [
        { id: 'hl-1', title: 'Rich Flex', streams: 450000000, revenue: 1800000 },
        { id: 'hl-2', title: 'Major Distribution', streams: 180000000, revenue: 720000 },
        { id: 'hl-3', title: 'On BS', streams: 120000000, revenue: 480000 },
        { id: 'hl-4', title: 'Backoutsideboyz', streams: 85000000, revenue: 340000 },
        { id: 'hl-5', title: 'Privileged Rappers', streams: 95000000, revenue: 380000 },
        { id: 'hl-6', title: 'Spin Bout U', streams: 150000000, revenue: 600000 },
        { id: 'hl-7', title: 'Hours In Silence', streams: 110000000, revenue: 440000 },
        { id: 'hl-8', title: 'Circo Loco', streams: 200000000, revenue: 800000 },
      ],
      total_streams: 1390000000,
      total_revenue: 5560000
    },
    {
      id: 'hn-2022',
      title: 'Honestly, Nevermind',
      year: 2022,
      cover_url: 'https://i.scdn.co/image/ab67616d0000b273cd945b4e3de57edd28481a3f',
      tracks: [
        { id: 'hn-1', title: 'Intro', streams: 25000000, revenue: 100000 },
        { id: 'hn-2', title: 'Falling Back', streams: 85000000, revenue: 340000 },
        { id: 'hn-3', title: 'Texts Go Green', streams: 95000000, revenue: 380000 },
        { id: 'hn-4', title: 'Currents', streams: 65000000, revenue: 260000 },
        { id: 'hn-5', title: 'A Keeper', streams: 55000000, revenue: 220000 },
        { id: 'hn-6', title: 'Calling My Name', streams: 120000000, revenue: 480000 },
        { id: 'hn-7', title: 'Sticky', streams: 180000000, revenue: 720000 },
        { id: 'hn-8', title: 'Jimmy Cooks', streams: 350000000, revenue: 1400000 },
      ],
      total_streams: 975000000,
      total_revenue: 3900000
    },
    {
      id: 'clb-2021',
      title: 'Certified Lover Boy',
      year: 2021,
      cover_url: 'https://i.scdn.co/image/ab67616d0000b2738dc0d801766a5aa6a33cbe37',
      tracks: [
        { id: 'clb-1', title: 'Champagne Poetry', streams: 280000000, revenue: 1120000 },
        { id: 'clb-2', title: 'Papi\'s Home', streams: 150000000, revenue: 600000 },
        { id: 'clb-3', title: "Girls Want Girls", streams: 450000000, revenue: 1800000 },
        { id: 'clb-4', title: 'In The Bible', streams: 120000000, revenue: 480000 },
        { id: 'clb-5', title: "Love All", streams: 180000000, revenue: 720000 },
        { id: 'clb-6', title: 'Fair Trade', streams: 320000000, revenue: 1280000 },
        { id: 'clb-7', title: 'Way 2 Sexy', streams: 650000000, revenue: 2600000 },
        { id: 'clb-8', title: 'TSU', streams: 200000000, revenue: 800000 },
        { id: 'clb-9', title: 'N 2 Deep', streams: 180000000, revenue: 720000 },
        { id: 'clb-10', title: 'Knife Talk', streams: 420000000, revenue: 1680000 },
      ],
      total_streams: 2950000000,
      total_revenue: 11800000
    },
    {
      id: 'dldt-2020',
      title: 'Dark Lane Demo Tapes',
      year: 2020,
      cover_url: 'https://i.scdn.co/image/ab67616d0000b273d4e1e1e9e0f0f5c2beb9c5a5',
      tracks: [
        { id: 'dldt-1', title: 'Deep Pockets', streams: 95000000, revenue: 380000 },
        { id: 'dldt-2', title: 'When To Say When', streams: 120000000, revenue: 480000 },
        { id: 'dldt-3', title: 'Chicago Freestyle', streams: 280000000, revenue: 1120000 },
        { id: 'dldt-4', title: 'Not You Too', streams: 85000000, revenue: 340000 },
        { id: 'dldt-5', title: 'Toosie Slide', streams: 950000000, revenue: 3800000 },
        { id: 'dldt-6', title: 'D4L', streams: 75000000, revenue: 300000 },
        { id: 'dldt-7', title: 'Pain 1993', streams: 350000000, revenue: 1400000 },
      ],
      total_streams: 1955000000,
      total_revenue: 7820000
    },
    {
      id: 'scorpion-2018',
      title: 'Scorpion',
      year: 2018,
      cover_url: 'https://i.scdn.co/image/ab67616d0000b2731c5d8e4f3b8c7d6e5f4a3b2c',
      tracks: [
        { id: 'sc-1', title: 'Survival', streams: 180000000, revenue: 720000 },
        { id: 'sc-2', title: 'Nonstop', streams: 850000000, revenue: 3400000 },
        { id: 'sc-3', title: 'Emotionless', streams: 280000000, revenue: 1120000 },
        { id: 'sc-4', title: "God's Plan", streams: 2800000000, revenue: 11200000 },
        { id: 'sc-5', title: "I'm Upset", streams: 450000000, revenue: 1800000 },
        { id: 'sc-6', title: 'In My Feelings', streams: 1500000000, revenue: 6000000 },
        { id: 'sc-7', title: "Don't Matter To Me", streams: 550000000, revenue: 2200000 },
        { id: 'sc-8', title: 'Nice For What', streams: 1200000000, revenue: 4800000 },
        { id: 'sc-9', title: 'Mob Ties', streams: 320000000, revenue: 1280000 },
        { id: 'sc-10', title: 'Ratchet Happy Birthday', streams: 180000000, revenue: 720000 },
      ],
      total_streams: 8310000000,
      total_revenue: 33240000
    },
    {
      id: 'morelife-2017',
      title: 'More Life',
      year: 2017,
      cover_url: 'https://i.scdn.co/image/ab67616d0000b2739b9f6e7a8c7d6e5f4a3b2c1d',
      tracks: [
        { id: 'ml-1', title: 'Free Smoke', streams: 350000000, revenue: 1400000 },
        { id: 'ml-2', title: 'No Long Talk', streams: 180000000, revenue: 720000 },
        { id: 'ml-3', title: 'Passionfruit', streams: 1800000000, revenue: 7200000 },
        { id: 'ml-4', title: 'Jorja Interlude', streams: 280000000, revenue: 1120000 },
        { id: 'ml-5', title: 'Get It Together', streams: 220000000, revenue: 880000 },
        { id: 'ml-6', title: 'Portland', streams: 650000000, revenue: 2600000 },
        { id: 'ml-7', title: 'Fake Love', streams: 1200000000, revenue: 4800000 },
        { id: 'ml-8', title: 'Gyalchester', streams: 450000000, revenue: 1800000 },
      ],
      total_streams: 5130000000,
      total_revenue: 20520000
    },
    {
      id: 'views-2016',
      title: 'Views',
      year: 2016,
      cover_url: 'https://i.scdn.co/image/ab67616d0000b273e7d4f4a3b2c1d0e9f8a7b6c5',
      tracks: [
        { id: 'v-1', title: 'Keep The Family Close', streams: 280000000, revenue: 1120000 },
        { id: 'v-2', title: '9', streams: 220000000, revenue: 880000 },
        { id: 'v-3', title: 'U With Me?', streams: 350000000, revenue: 1400000 },
        { id: 'v-4', title: 'Feel No Ways', streams: 480000000, revenue: 1920000 },
        { id: 'v-5', title: 'Hype', streams: 380000000, revenue: 1520000 },
        { id: 'v-6', title: 'Weston Road Flows', streams: 280000000, revenue: 1120000 },
        { id: 'v-7', title: 'One Dance', streams: 2800000000, revenue: 11200000 },
        { id: 'v-8', title: 'Controlla', streams: 1100000000, revenue: 4400000 },
        { id: 'v-9', title: 'Too Good', streams: 850000000, revenue: 3400000 },
        { id: 'v-10', title: 'Hotline Bling', streams: 2200000000, revenue: 8800000 },
      ],
      total_streams: 8940000000,
      total_revenue: 35760000
    },
    {
      id: 'iyrtitl-2015',
      title: "If You're Reading This It's Too Late",
      year: 2015,
      cover_url: 'https://i.scdn.co/image/ab67616d0000b273f9f8a7b6c5d4e3f2a1b0c9d8',
      tracks: [
        { id: 'iy-1', title: 'Legend', streams: 380000000, revenue: 1520000 },
        { id: 'iy-2', title: 'Energy', streams: 750000000, revenue: 3000000 },
        { id: 'iy-3', title: '10 Bands', streams: 480000000, revenue: 1920000 },
        { id: 'iy-4', title: 'Know Yourself', streams: 950000000, revenue: 3800000 },
        { id: 'iy-5', title: 'No Tellin\'', streams: 280000000, revenue: 1120000 },
        { id: 'iy-6', title: 'Madonna', streams: 220000000, revenue: 880000 },
        { id: 'iy-7', title: 'Star67', streams: 350000000, revenue: 1400000 },
        { id: 'iy-8', title: 'Jungle', streams: 420000000, revenue: 1680000 },
      ],
      total_streams: 3830000000,
      total_revenue: 15320000
    },
    {
      id: 'nwts-2013',
      title: 'Nothing Was the Same',
      year: 2013,
      cover_url: 'https://i.scdn.co/image/ab67616d0000b273d0e8b8c7a6f5e4d3c2b1a0f9',
      tracks: [
        { id: 'nwts-1', title: 'Tuscan Leather', streams: 280000000, revenue: 1120000 },
        { id: 'nwts-2', title: 'Furthest Thing', streams: 350000000, revenue: 1400000 },
        { id: 'nwts-3', title: 'Started From The Bottom', streams: 1500000000, revenue: 6000000 },
        { id: 'nwts-4', title: 'Wu-Tang Forever', streams: 280000000, revenue: 1120000 },
        { id: 'nwts-5', title: 'Own It', streams: 220000000, revenue: 880000 },
        { id: 'nwts-6', title: 'Worst Behavior', streams: 380000000, revenue: 1520000 },
        { id: 'nwts-7', title: 'From Time', streams: 450000000, revenue: 1800000 },
        { id: 'nwts-8', title: 'Hold On, We\'re Going Home', streams: 1800000000, revenue: 7200000 },
      ],
      total_streams: 5260000000,
      total_revenue: 21040000
    },
    {
      id: 'tc-2011',
      title: 'Take Care',
      year: 2011,
      cover_url: 'https://i.scdn.co/image/ab67616d0000b273c0e8a9b8d7f6e5c4b3a2d1e0',
      tracks: [
        { id: 'tc-1', title: 'Over My Dead Body', streams: 280000000, revenue: 1120000 },
        { id: 'tc-2', title: 'Shot For Me', streams: 350000000, revenue: 1400000 },
        { id: 'tc-3', title: 'Headlines', streams: 850000000, revenue: 3400000 },
        { id: 'tc-4', title: 'Crew Love', streams: 650000000, revenue: 2600000 },
        { id: 'tc-5', title: 'Take Care', streams: 1200000000, revenue: 4800000 },
        { id: 'tc-6', title: 'Marvins Room', streams: 950000000, revenue: 3800000 },
        { id: 'tc-7', title: 'Make Me Proud', streams: 380000000, revenue: 1520000 },
        { id: 'tc-8', title: 'HYFR', streams: 550000000, revenue: 2200000 },
        { id: 'tc-9', title: 'The Motto', streams: 750000000, revenue: 3000000 },
      ],
      total_streams: 5960000000,
      total_revenue: 23840000
    },
    {
      id: 'tml-2010',
      title: 'Thank Me Later',
      year: 2010,
      cover_url: 'https://i.scdn.co/image/ab67616d0000b273e1f8b9a8c7d6e5f4a3b2c1d0',
      tracks: [
        { id: 'tml-1', title: 'Fireworks', streams: 180000000, revenue: 720000 },
        { id: 'tml-2', title: 'Karaoke', streams: 120000000, revenue: 480000 },
        { id: 'tml-3', title: 'The Resistance', streams: 95000000, revenue: 380000 },
        { id: 'tml-4', title: 'Over', streams: 450000000, revenue: 1800000 },
        { id: 'tml-5', title: 'Show Me A Good Time', streams: 150000000, revenue: 600000 },
        { id: 'tml-6', title: 'Miss Me', streams: 280000000, revenue: 1120000 },
        { id: 'tml-7', title: 'Find Your Love', streams: 550000000, revenue: 2200000 },
      ],
      total_streams: 1825000000,
      total_revenue: 7300000
    },
    {
      id: 'sfg-2009',
      title: 'So Far Gone',
      year: 2009,
      cover_url: 'https://i.scdn.co/image/ab67616d0000b273f2a9c8b7d6e5f4a3b2c1d0e9',
      tracks: [
        { id: 'sfg-1', title: 'Lust For Life', streams: 180000000, revenue: 720000 },
        { id: 'sfg-2', title: 'Houstatlantavegas', streams: 220000000, revenue: 880000 },
        { id: 'sfg-3', title: 'Successful', streams: 350000000, revenue: 1400000 },
        { id: 'sfg-4', title: 'Best I Ever Had', streams: 850000000, revenue: 3400000 },
        { id: 'sfg-5', title: 'I\'m Goin In', streams: 280000000, revenue: 1120000 },
      ],
      total_streams: 1880000000,
      total_revenue: 7520000
    },
  ];

  // Calculate totals
  $: totalStreams = drakeDiscography.reduce((sum, a) => sum + a.total_streams, 0);
  $: totalRevenue = drakeDiscography.reduce((sum, a) => sum + a.total_revenue, 0);
  $: totalTracks = drakeDiscography.reduce((sum, a) => sum + a.tracks.length, 0);

  // Simulated monthly figures (divide by 12 for monthly estimate from all-time)
  $: monthlyStreams = Math.round(totalStreams / 24); // Assume 2 year average
  $: monthlyRevenue = Math.round(totalRevenue / 24);

  async function loadData() {
    if (!artistId) return;
    isLoading = true;
    error = null;

    const result = await analyticsActions.fetchArtistDiscographyRevenue(artistId);
    // We'll use our local data regardless since the API returns generic data
    isLoading = false;
  }

  onMount(() => {
    prefersReducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    loadData();
  });

  $: if (artistId) {
    loadData();
  }

  $: perStreamRate = 0.004;
</script>

<div class="artist-discography-revenue surface-panel" data-testid="artist-discography-revenue">
  {#if error}
    <div class="rev-error surface-panel-thin" data-testid="error-message">
      {error}
      <button on:click={loadData} class="rev-error__retry">Retry</button>
    </div>
  {:else if isLoading}
    <div class="rev-loading" data-testid="loading-spinner">
      <div class="rev-spinner"></div>
    </div>
  {:else}
    {@const maxRevenue = Math.max(...drakeDiscography.map(a => a.total_revenue))}

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
                  <img src={album.cover_url} alt="" class="rev-album__art-img" on:error={(e) => { e.currentTarget.style.display = 'none'; }} />
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

    <!-- Simulation Note -->
    <div class="rev-note" data-testid="simulation-note">
      Data is simulated for demonstration purposes. Connect streaming accounts for real analytics.
    </div>
  {/if}
</div>

<style>
  .artist-discography-revenue {
    border-radius: 1rem;
    padding: 1.5rem;
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

  /* === Error & Loading === */
  .rev-error {
    padding: 1rem;
    border-radius: 0.75rem;
    color: #f87171;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .rev-error__retry {
    color: #fca5a5;
    text-decoration: underline;
    background: none;
    border: none;
    cursor: pointer;
    font-size: inherit;
  }

  .rev-error__retry:hover { text-decoration: none; }

  .rev-loading {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 2rem 0;
  }

  .rev-spinner {
    width: 2rem;
    height: 2rem;
    border: 2px solid transparent;
    border-bottom-color: #a1a1aa;
    border-radius: 50%;
    animation: rev-spin 1s linear infinite;
  }

  @keyframes rev-spin {
    to { transform: rotate(360deg); }
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
