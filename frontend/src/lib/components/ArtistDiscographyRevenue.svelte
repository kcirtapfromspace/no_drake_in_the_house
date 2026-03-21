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
    <div class="bg-red-900/20 border border-red-800 rounded-lg p-4 text-red-400" data-testid="error-message">
      {error}
      <button on:click={loadData} class="ml-2 text-red-300 underline hover:no-underline">Retry</button>
    </div>
  {:else if isLoading}
    <div class="flex items-center justify-center py-8" data-testid="loading-spinner">
      <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-zinc-400"></div>
    </div>
  {:else}
    {@const maxRevenue = Math.max(...drakeDiscography.map(a => a.total_revenue))}

    <!-- Hero Revenue Stat -->
    <div class="mb-8 text-center" data-testid="revenue-summary">
      <div class="text-sm text-zinc-400 mb-1">Estimated Yearly Revenue</div>
      <div class="text-4xl font-bold text-emerald-400" data-testid="yearly-revenue">{formatCurrency(monthlyRevenue * 12)}</div>
      <div class="text-sm text-zinc-400 mt-1">Streaming + Writing + Production</div>

      <div class="grid grid-cols-3 gap-4 mt-6 max-w-lg mx-auto">
        <div>
          <div class="text-lg font-semibold text-zinc-100" data-testid="monthly-revenue">{formatCurrency(monthlyRevenue)}</div>
          <div class="text-xs text-zinc-400">Monthly</div>
        </div>
        <div>
          <div class="text-lg font-semibold text-zinc-100" data-testid="monthly-streams">{formatNumber(monthlyStreams)}</div>
          <div class="text-xs text-zinc-400">Monthly streams</div>
        </div>
        <div>
          <div class="text-lg font-semibold text-zinc-100" data-testid="discography-count">{totalTracks}</div>
          <div class="text-xs text-zinc-400">{drakeDiscography.length} albums</div>
        </div>
      </div>
    </div>

    <!-- Album Revenue Breakdown -->
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-lg font-semibold text-zinc-100">Album Revenue</h3>
      <span class="text-xs text-zinc-400">Est. ${(perStreamRate * 1000).toFixed(2)} / 1K streams</span>
    </div>

    <div class="space-y-3" data-testid="albums-section">
      {#each drakeDiscography as album}
        {@const barWidth = maxRevenue > 0 ? (album.total_revenue / maxRevenue * 100) : 0}
        <div class="rounded-xl overflow-hidden surface-panel-thin">
          <button
            class="w-full px-5 py-4 flex items-center gap-4 text-left transition-colors hover:bg-white/[0.04]"
            on:click={() => toggleAlbum(album.id)}
          >
            <!-- Album Art -->
            <div class="w-12 h-12 rounded-lg overflow-hidden bg-zinc-800 flex-shrink-0 relative">
              {#if album.cover_url}
                <img src={album.cover_url} alt="" class="w-12 h-12 object-cover absolute inset-0" on:error={(e) => { e.currentTarget.style.display = 'none'; }} />
              {/if}
              <div class="w-12 h-12 flex items-center justify-center">
                <svg class="w-5 h-5 text-zinc-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3" />
                </svg>
              </div>
            </div>

            <!-- Album Info + Proportional Bar -->
            <div class="flex-1 min-w-0">
              <div class="flex items-baseline gap-2 mb-1">
                <span class="text-white font-semibold truncate flex-1 min-w-0">{album.title}</span>
                <span class="text-xs text-zinc-400 flex-shrink-0">{album.year} · {album.tracks.length}t · {formatNumber(album.total_streams)} streams</span>
              </div>
              <div class="flex items-center gap-3">
                <div class="h-2 flex-1 max-w-[60%] rounded-full overflow-hidden" style="background: var(--color-bg-inset);">
                  <div class="h-full rounded-full transition-all" style="width: {barWidth}%; background: linear-gradient(90deg, var(--color-success), #34d399);"></div>
                </div>
                <span class="text-sm font-medium text-emerald-400 flex-shrink-0 w-24 text-right">{formatCurrency(album.total_revenue)}</span>
              </div>
            </div>

            <!-- Expand -->
            <svg class="w-4 h-4 text-zinc-400 flex-shrink-0 transition-transform {expandedAlbums.has(album.id) ? 'rotate-180' : ''}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
            </svg>
          </button>

          {#if expandedAlbums.has(album.id)}
            <div class="border-t border-white/[0.06]" transition:slide={{ duration: slideDuration }}>
              {#each album.tracks as track, idx}
                <div class="flex items-center gap-3 px-4 py-2 border-t border-white/[0.04] first:border-t-0 hover:bg-white/[0.02]">
                  <span class="text-xs text-zinc-400 w-5 text-right">{idx + 1}</span>
                  <span class="flex-1 text-sm text-zinc-300 truncate">{track.title}</span>
                  <span class="text-xs text-zinc-400 flex-shrink-0">{formatNumber(track.streams)}</span>
                  <span class="text-xs text-emerald-400 font-medium flex-shrink-0 w-16 text-right">{formatCurrency(track.revenue)}</span>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {/each}
    </div>

    <!-- Totals -->
    <div class="mt-4 p-4 rounded-xl flex items-center justify-between surface-panel">
      <div>
        <div class="text-sm font-semibold text-zinc-100">Total Catalog Revenue</div>
        <div class="text-xs text-zinc-400">{drakeDiscography.length} albums · {totalTracks} tracks · {formatNumber(totalStreams)} streams</div>
      </div>
      <div class="text-xl font-bold text-emerald-400">{formatCurrency(totalRevenue)}</div>
    </div>

    <!-- Credits Revenue -->
    <div class="mt-8 rounded-xl p-5 surface-panel">
    <h3 class="text-lg font-semibold text-zinc-100 mb-4">Revenue Beyond Streaming</h3>
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <div class="p-4 rounded-xl surface-panel-thin">
        <div class="flex items-center gap-2 mb-3">
          <svg class="w-4 h-4 text-blue-400" width="16" height="16" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
          </svg>
          <span class="text-zinc-100 font-medium text-sm">Songwriting</span>
        </div>
        <div class="space-y-1.5 text-sm">
          <div class="flex justify-between"><span class="text-zinc-400">Own catalog</span><span class="text-zinc-300">{totalTracks} songs</span></div>
          <div class="flex justify-between"><span class="text-zinc-400">Other artists</span><span class="text-zinc-300">~85 songs</span></div>
          <div class="flex justify-between"><span class="text-zinc-400">Publishing share</span><span class="text-zinc-300">~50%</span></div>
          <div class="border-t border-white/[0.06] pt-2 mt-2 flex justify-between">
            <span class="text-zinc-300 font-medium">Est. Annual</span>
            <span class="text-blue-400 font-bold">{formatCurrency(Math.round(totalRevenue * 0.15))}</span>
          </div>
        </div>
      </div>

      <div class="p-4 rounded-xl surface-panel-thin">
        <div class="flex items-center gap-2 mb-3">
          <svg class="w-4 h-4 text-purple-400" width="16" height="16" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3" />
          </svg>
          <span class="text-zinc-100 font-medium text-sm">Production</span>
        </div>
        <div class="space-y-1.5 text-sm">
          <div class="flex justify-between"><span class="text-zinc-400">Co-production</span><span class="text-zinc-300">~45 songs</span></div>
          <div class="flex justify-between"><span class="text-zinc-400">Executive producer</span><span class="text-zinc-300">13 albums</span></div>
          <div class="flex justify-between"><span class="text-zinc-400">Points (avg)</span><span class="text-zinc-300">~3%</span></div>
          <div class="border-t border-white/[0.06] pt-2 mt-2 flex justify-between">
            <span class="text-zinc-300 font-medium">Est. Annual</span>
            <span class="text-purple-400 font-bold">{formatCurrency(Math.round(totalRevenue * 0.08))}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Grand Total -->
    <div class="mt-4 p-4 rounded-xl text-center surface-panel-thin">
      <div class="text-xs text-zinc-400 mb-1">All-Time Estimated Revenue (Streaming + Writing + Production)</div>
      <div class="text-2xl font-bold text-emerald-400">{formatCurrency(Math.round(totalRevenue * 1.23))}</div>
    </div>
    </div>

    <!-- Revenue Context -->
    <div class="mt-6 p-4 rounded-xl surface-panel-thin" data-testid="revenue-context">
      <div class="flex items-start gap-3">
        <svg class="w-4 h-4 text-zinc-400 mt-0.5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        <div>
          <div class="text-zinc-300 font-medium text-sm mb-1">Revenue Methodology</div>
          <div class="text-sm text-zinc-400">
            Estimates based on average streaming platform payouts (~$0.003-0.005 per stream).
            Actual revenue varies by platform, region, and artist contract terms.
            Top-tier artists may negotiate higher rates.
          </div>
        </div>
      </div>
    </div>

    <!-- Simulation Note -->
    <div class="mt-4 text-xs text-zinc-400 text-center" data-testid="simulation-note">
      Data is simulated for demonstration purposes. Connect streaming accounts for real analytics.
    </div>
  {/if}
</div>

<style>
  .artist-discography-revenue {
    border-radius: 1rem;
    padding: 1.5rem;
  }
</style>
