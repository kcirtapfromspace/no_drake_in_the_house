<script lang="ts">
  import { onMount } from 'svelte';
  import { slide } from 'svelte/transition';
  import {
    getStatusColor,
    getStatusLabel,
    getConfidenceLabel,
    getCategoryColor,
    getSourceTierLabel,
    getProceduralStateLabel,
    getEvidenceStrengthLabel,
    type ArtistProfile,
    type ArtistStatus,
    type ConfidenceLevel,
    type Offense,
    type SourceTier
  } from '../stores/artist';
  import { navigateTo, navigateToArtist } from '../utils/simple-router';
  import { apiClient } from '../utils/api-client';
  import { validateLinks, type LinkCheckResult } from '../utils/link-validator';
  import ArtistDiscographyRevenue from './ArtistDiscographyRevenue.svelte';

  export let artistId: string;

  let profile: ArtistProfile | null = null;
  let isLoading = true;
  let error: string | null = null;
  type ProfileTab = 'evidence' | 'catalog' | 'discography' | 'credits' | 'connections';
  let activeTab: ProfileTab = 'evidence';
  const profileTabs: { key: ProfileTab; label: string }[] = [
    { key: 'evidence', label: 'Evidence' },
    { key: 'catalog', label: 'Full Catalog' },
    { key: 'discography', label: 'Revenue' },
    { key: 'credits', label: 'Credits' },
    { key: 'connections', label: 'Connections' },
  ];
  function setTab(key: string) { activeTab = key as ProfileTab; }

  // Catalog data for tracking all artist appearances
  interface CatalogTrack {
    id: string;
    title: string;
    album?: string;
    role: 'main' | 'featured' | 'producer' | 'writer';
    year?: number;
    isBlocked: boolean;
    collaborators?: string[];
    duration?: string;
  }

  let catalog: CatalogTrack[] = [];
  let catalogFilter: 'all' | 'blocked' | 'unblocked' = 'all';
  let catalogSubTab: 'main' | 'featured' | 'behind' = 'main';
  let featuredShowCount = 20;
  let behindShowCount = 20;

  // Link validation state
  let linkStatuses: Map<string, LinkCheckResult> = new Map();
  let linksValidating = false;

  async function runLinkValidation() {
    if (!profile?.offenses?.length) return;
    linksValidating = true;
    const urls: { url: string; archivedUrl?: string }[] = [];
    for (const offense of profile.offenses) {
      for (const ev of offense.evidence) {
        if (ev.source.url) {
          urls.push({ url: ev.source.url, archivedUrl: ev.source.archived_url });
        }
      }
    }
    if (urls.length > 0) {
      linkStatuses = await validateLinks(urls);
    }
    linksValidating = false;
  }

  // Connections tab state
  let connectionsLoading = false;
  let blockedNetworkArtists: Set<string> = new Set();

  // Expandable albums state for catalog view
  let expandedCatalogAlbums: Set<string> = new Set();

  // Album cover modal state
  let showAlbumCoverModal = false;
  let selectedAlbumCover: { url: string; name: string } | null = null;

  // Blocking options dropdown state
  let showBlockingOptions = false;

  // Reduced motion preference
  let prefersReducedMotion = false;
  $: slideDuration = prefersReducedMotion ? 0 : 200;

  function hideImgOnError(e: Event) { (e.currentTarget as HTMLImageElement).style.display = 'none'; }
  function handleImgError(e: Event) {
    const img = e.currentTarget as HTMLImageElement;
    img.alt = 'Image unavailable';
    img.style.minHeight = '200px';
    img.style.background = 'var(--color-bg-interactive)';
  }

  function hexToRgba(hex: string, alpha: number): string {
    const r = parseInt(hex.slice(1, 3), 16);
    const g = parseInt(hex.slice(3, 5), 16);
    const b = parseInt(hex.slice(5, 7), 16);
    return `rgba(${r}, ${g}, ${b}, ${alpha})`;
  }

  function closeAlbumCoverModal() {
    showAlbumCoverModal = false;
    selectedAlbumCover = null;
  }

  function toggleCatalogAlbum(albumName: string) {
    if (expandedCatalogAlbums.has(albumName)) {
      expandedCatalogAlbums.delete(albumName);
    } else {
      expandedCatalogAlbums.add(albumName);
    }
    expandedCatalogAlbums = new Set(expandedCatalogAlbums);
  }

  // Album cover art mapping - Real Spotify album art URLs for Drake
  const albumCovers: Record<string, string> = {
    'For All The Dogs': 'https://i.scdn.co/image/ab67616d0000b273929008d1bec2f6f89fb0c3f6',
    'Her Loss': 'https://i.scdn.co/image/ab67616d0000b2732a1d0e45e67c5c4e5e5d8fdb',
    'Honestly, Nevermind': 'https://i.scdn.co/image/ab67616d0000b273cd945b4e3de57edd28481a3f',
    'Certified Lover Boy': 'https://i.scdn.co/image/ab67616d0000b2738dc0d801766a5aa6a33cbe37',
    'Dark Lane Demo Tapes': 'https://i.scdn.co/image/ab67616d0000b273d4e1e1e9e0f0f5c2beb9c5a5',
    'Scorpion': 'https://i.scdn.co/image/ab67616d0000b273f907de96b9a4fbc04accc0d5',
    'More Life': 'https://i.scdn.co/image/ab67616d0000b2734f0fd9dad63977146e685700',
    'Views': 'https://i.scdn.co/image/ab67616d0000b2739416ed64daf84936d89e671c',
    "If You're Reading This It's Too Late": 'https://i.scdn.co/image/ab67616d0000b27347b3bd2069e49f7f141a8aec',
    'Nothing Was the Same': 'https://i.scdn.co/image/ab67616d0000b2731ad8609a32946f90cf87eb71',
    'Take Care': 'https://i.scdn.co/image/ab67616d0000b27318afb1aae2b7197d7f0d768c',
    'Thank Me Later': 'https://i.scdn.co/image/ab67616d0000b273fc4f17340773c6c3579c0c7a',
    'So Far Gone': 'https://i.scdn.co/image/ab67616d0000b273fda5e89768a2630f7bc4d3ae',
    'What a Time to Be Alive': 'https://i.scdn.co/image/ab67616d0000b273c58c6a71a8bdce52a4399d1c',
  };

  function getAlbumCover(albumName: string | undefined): string {
    if (!albumName) return '';
    return albumCovers[albumName] || '';
  }

  // Group catalog tracks by album
  interface CatalogAlbum {
    name: string;
    year: number;
    cover: string;
    tracks: CatalogTrack[];
    blockedCount: number;
    totalCount: number;
  }

  $: catalogAlbums = (() => {
    const albumMap = new Map<string, CatalogAlbum>();
    const mainTracks = filteredCatalog.filter(t => t.role === 'main');

    for (const track of mainTracks) {
      const albumName = track.album || 'Singles & Loosies';
      if (!albumMap.has(albumName)) {
        albumMap.set(albumName, {
          name: albumName,
          year: track.year || 0,
          cover: getAlbumCover(track.album),
          tracks: [],
          blockedCount: 0,
          totalCount: 0
        });
      }
      const album = albumMap.get(albumName)!;
      album.tracks.push(track);
      album.totalCount++;
      if (track.isBlocked) album.blockedCount++;
    }

    // Sort by year descending
    return Array.from(albumMap.values()).sort((a, b) => b.year - a.year);
  })();

  // Toggle all tracks in an album (local state, with optional backend sync)
  function toggleAlbumBlocking(albumName: string, block: boolean) {
    const albumTracks = catalog.filter(t => t.album === albumName && t.role === 'main');
    const tracksToUpdate = albumTracks.filter(t => t.isBlocked !== block);

    if (tracksToUpdate.length === 0) return;

    // Update UI immediately
    catalog = catalog.map(track => {
      if (track.album === albumName && track.role === 'main') {
        return { ...track, isBlocked: block };
      }
      return track;
    });

    // Try to persist to backend (don't revert on failure)
    const trackIds = tracksToUpdate.map(t => t.id);
    apiClient.post('/api/v1/dnp/tracks/batch', {
      artist_id: artistId,
      track_ids: trackIds,
      action: block ? 'block' : 'unblock',
      role: 'main'
    }).catch(err => console.log('Backend sync skipped:', err));
  }

  // Toggle individual track blocking (local state, with optional backend sync)
  function toggleTrackBlock(trackId: string) {
    const track = catalog.find(t => t.id === trackId);
    if (!track) return;

    const newBlockedState = !track.isBlocked;

    // Update UI immediately
    catalog = catalog.map(t => {
      if (t.id === trackId) {
        return { ...t, isBlocked: newBlockedState };
      }
      return t;
    });

    // Try to persist to backend (don't revert on failure - local state is source of truth for demo)
    const endpoint = newBlockedState
      ? '/api/v1/dnp/tracks'
      : `/api/v1/dnp/tracks/${trackId}`;

    const apiCall = newBlockedState
      ? apiClient.post(endpoint, {
          artist_id: artistId,
          track_id: trackId,
          track_title: track.title,
          track_role: track.role
        })
      : apiClient.delete(endpoint);

    apiCall.catch(err => console.log('Backend sync skipped:', err));
  }

  // Block/unblock all tracks of a certain role (local state, with optional backend sync)
  function toggleRoleBlocking(role: 'main' | 'featured' | 'producer' | 'writer', block: boolean) {
    const tracksToUpdate = catalog.filter(t => t.role === role && t.isBlocked !== block);

    // Update UI immediately
    catalog = catalog.map(track => {
      if (track.role === role) {
        return { ...track, isBlocked: block };
      }
      return track;
    });

    // Try to persist to backend (don't revert on failure)
    const trackIds = tracksToUpdate.map(t => t.id);
    apiClient.post('/api/v1/dnp/tracks/batch', {
      artist_id: artistId,
      track_ids: trackIds,
      action: block ? 'block' : 'unblock',
      role: role
    }).catch(err => console.log('Backend sync skipped:', err));
  }

  // Get filtered catalog
  $: filteredCatalog = catalogFilter === 'all'
    ? catalog
    : catalogFilter === 'blocked'
      ? catalog.filter(t => t.isBlocked)
      : catalog.filter(t => !t.isBlocked);

  $: catalogMainCount = catalog.filter(t => t.role === 'main').length;
  $: catalogFeaturedCount = catalog.filter(t => t.role === 'featured').length;
  $: catalogBehindCount = catalog.filter(t => t.role === 'producer' || t.role === 'writer').length;

  let expandedOffenseId: string | null = null;
  let showReportModal = false;
  let reportDescription = '';
  let reportCategory = 'factual_error';

  // DNP state
  let isBlocked = false;
  let isBlockingInProgress = false;
  let dnpList: Set<string> = new Set();

  function normalizeDnpArtistIds(value: unknown): string[] {
    if (Array.isArray(value)) {
      return value
        .map((item) => (item as { artist_id?: string; id?: string }).artist_id || (item as { artist_id?: string; id?: string }).id || '')
        .filter(Boolean);
    }

    if (value && typeof value === 'object') {
      for (const key of ['entries', 'artists', 'items', 'data']) {
        const nested = (value as Record<string, unknown>)[key];
        if (Array.isArray(nested)) {
          return nested
            .map((item) => (item as { artist_id?: string; id?: string }).artist_id || (item as { artist_id?: string; id?: string }).id || '')
            .filter(Boolean);
        }
      }
    }

    return [];
  }

  onMount(async () => {
    prefersReducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    await loadArtist();
    await loadDnpStatus();
  });

  async function loadArtist() {
    isLoading = true;
    error = null;

    try {
      // Fetch full profile with offenses
      const result = await apiClient.get<any>(`/api/v1/offenses/query?artist_id=${artistId}`);

      if (result.success && result.data) {
        // Transform the response to match our profile structure
        profile = transformToProfile(result.data);
      } else {
        error = result.message || 'Failed to load artist';
      }

      // Also fetch streaming metrics if available
      const metricsResult = await apiClient.get<any>(`/api/v1/artists/${artistId}/analytics`);
      if (metricsResult.success && metricsResult.data && profile) {
        profile.streaming_metrics = metricsResult.data;
      }

      // Fetch collaborators from real API
      connectionsLoading = true;
      try {
        const [collabResult, blockedResult] = await Promise.all([
          apiClient.get<any>(`/api/v1/graph/artists/${artistId}/collaborators`),
          apiClient.get<any>(`/api/v1/graph/blocked-network`).catch(() => null),
        ]);

        // Build set of blocked/flagged artist IDs from the blocked-network endpoint
        blockedNetworkArtists = new Set();
        if (blockedResult?.success && blockedResult.data) {
          const clusters = blockedResult.data.blocked_clusters || [];
          for (const cluster of clusters) {
            for (const a of cluster.artists || []) {
              if (a.id) blockedNetworkArtists.add(a.id);
            }
          }
          const atRisk = blockedResult.data.at_risk_artists || [];
          for (const entry of atRisk) {
            if (entry.artist?.id && entry.artist?.is_blocked) {
              blockedNetworkArtists.add(entry.artist.id);
            }
          }
        }

        if (collabResult.success && collabResult.data && profile) {
          const raw = collabResult.data.collaborators || collabResult.data || [];
          // Map API response to Collaborator shape, merging blocked-network info
          profile.collaborators = raw.map((c: any) => ({
            id: c.artist_id || c.id || '',
            name: c.artist_name || c.name || 'Unknown',
            collaboration_type: c.collab_type || c.collaboration_type || 'featured',
            collaboration_count: c.track_count || c.collaboration_count || 1,
            is_flagged: c.is_flagged ?? blockedNetworkArtists.has(c.artist_id || c.id || ''),
            status: c.status || (blockedNetworkArtists.has(c.artist_id || c.id || '') ? 'flagged' : 'clean'),
            image_url: c.image_url || null,
            recent_tracks: c.track_title ? [c.track_title] : [],
          }));
          // De-duplicate by artist id (API may return multiple rows per collaborator)
          const seen = new Map<string, any>();
          for (const collab of profile.collaborators) {
            const existing = seen.get(collab.id);
            if (existing) {
              existing.collaboration_count = Math.max(existing.collaboration_count, collab.collaboration_count);
              if (collab.recent_tracks?.length) {
                existing.recent_tracks = [...new Set([...(existing.recent_tracks || []), ...collab.recent_tracks])];
              }
            } else {
              seen.set(collab.id, collab);
            }
          }
          profile.collaborators = Array.from(seen.values());
          // Sort by collaboration count descending
          profile.collaborators.sort((a: any, b: any) => b.collaboration_count - a.collaboration_count);
        }
      } catch (collabErr) {
        console.warn('Collaborators API failed:', collabErr);
      } finally {
        connectionsLoading = false;
      }

      // If still no collaborators, initialize to empty array (no mock data)
      if (profile && !profile.collaborators) {
        profile.collaborators = [];
      }

      // Add credits and catalog data for Drake showcase
      if (profile && profile.canonical_name === 'Drake') {
        // Add writers/producers credits for Drake
        profile.credits = {
          writers: [
            { id: 'writer-1', name: 'Noah "40" Shebib', role: 'writer', track_count: 120, is_flagged: false, image_url: 'https://i.scdn.co/image/ab6761610000e5eb9e3c7c7c7c7c7c7c7c7c7c7c' },
            { id: 'writer-2', name: 'Quentin Miller', role: 'writer', track_count: 15, is_flagged: false, note: 'Reference track controversy', image_url: null },
            { id: 'writer-3', name: 'PartyNextDoor', role: 'writer', track_count: 25, is_flagged: false, image_url: 'https://i.scdn.co/image/ab6761610000e5eb9997e163236a0dbbe8e7fdf7' },
            { id: 'writer-4', name: 'Nickelus F', role: 'writer', track_count: 8, is_flagged: false, image_url: null },
            { id: 'writer-5', name: 'Kenza Samir', role: 'writer', track_count: 12, is_flagged: false, image_url: null },
            { id: 'writer-6', name: 'Majid Jordan', role: 'writer', track_count: 18, is_flagged: false, image_url: 'https://i.scdn.co/image/ab6761610000e5eb2d2d2d2d2d2d2d2d2d2d2d2d' },
            { id: 'writer-7', name: 'OVO Noel', role: 'writer', track_count: 10, is_flagged: false, image_url: null },
          ],
          producers: [
            { id: 'prod-1', name: 'Noah "40" Shebib', role: 'producer', track_count: 150, is_flagged: false, image_url: 'https://i.scdn.co/image/ab6761610000e5eb9e3c7c7c7c7c7c7c7c7c7c7c' },
            { id: 'prod-2', name: 'Boi-1da', role: 'producer', track_count: 55, is_flagged: false, image_url: 'https://i.scdn.co/image/ab6761610000e5eb6a6a6a6a6a6a6a6a6a6a6a6a' },
            { id: 'prod-3', name: 'Metro Boomin', role: 'producer', track_count: 25, is_flagged: false, image_url: 'https://i.scdn.co/image/ab6761610000e5eb9d4b4b4b4b4b4b4b4b4b4b4b' },
            { id: 'prod-4', name: 'Tay Keith', role: 'producer', track_count: 15, is_flagged: false, image_url: 'https://i.scdn.co/image/ab6761610000e5eb8e8e8e8e8e8e8e8e8e8e8e8e' },
            { id: 'prod-5', name: 'Hit-Boy', role: 'producer', track_count: 12, is_flagged: false, image_url: 'https://i.scdn.co/image/ab6761610000e5eb7f7f7f7f7f7f7f7f7f7f7f7f' },
            { id: 'prod-6', name: 'Mike WiLL Made-It', role: 'producer', track_count: 8, is_flagged: false, image_url: 'https://i.scdn.co/image/ab6761610000e5eb5e5e5e5e5e5e5e5e5e5e5e5e' },
            { id: 'prod-7', name: 'Vinylz', role: 'producer', track_count: 20, is_flagged: false, image_url: null },
            { id: 'prod-8', name: 'T-Minus', role: 'producer', track_count: 18, is_flagged: false, image_url: null },
          ],
        };

        // Add comprehensive catalog for Drake - FULL discography across his career
        catalog = [
          // ========================================
          // MAIN ARTIST TRACKS (200+)
          // ========================================

          // --- FOR ALL THE DOGS (2023) - 23 tracks ---
          { id: 'fatd-1', title: 'Virginia Beach', album: 'For All The Dogs', role: 'main', year: 2023, isBlocked: true, duration: '3:24' },
          { id: 'fatd-2', title: 'Amen', album: 'For All The Dogs', role: 'main', year: 2023, isBlocked: true, collaborators: ['Teezo Touchdown'], duration: '2:55' },
          { id: 'fatd-3', title: 'Calling For You', album: 'For All The Dogs', role: 'main', year: 2023, isBlocked: true, collaborators: ['21 Savage'], duration: '3:41' },
          { id: 'fatd-4', title: 'Fear of Heights', album: 'For All The Dogs', role: 'main', year: 2023, isBlocked: true, duration: '5:34' },
          { id: 'fatd-5', title: 'Daylight', album: 'For All The Dogs', role: 'main', year: 2023, isBlocked: true, duration: '3:29' },
          { id: 'fatd-6', title: 'First Person Shooter', album: 'For All The Dogs', role: 'main', year: 2023, isBlocked: true, collaborators: ['J. Cole'], duration: '4:05' },
          { id: 'fatd-7', title: 'IDGAF', album: 'For All The Dogs', role: 'main', year: 2023, isBlocked: true, collaborators: ['Yeat'], duration: '3:14' },
          { id: 'fatd-8', title: 'Rich Baby Daddy', album: 'For All The Dogs', role: 'main', year: 2023, isBlocked: true, collaborators: ['Sexyy Red', 'SZA'], duration: '3:49' },
          { id: 'fatd-9', title: 'Slime You Out', album: 'For All The Dogs', role: 'main', year: 2023, isBlocked: true, collaborators: ['SZA'], duration: '4:14' },
          { id: 'fatd-10', title: 'Members Only', album: 'For All The Dogs', role: 'main', year: 2023, isBlocked: true, duration: '3:22' },
          { id: 'fatd-11', title: 'What Would Pluto Do', album: 'For All The Dogs', role: 'main', year: 2023, isBlocked: true, duration: '2:28' },
          { id: 'fatd-12', title: 'Tried Our Best', album: 'For All The Dogs', role: 'main', year: 2023, isBlocked: true, duration: '2:37' },
          { id: 'fatd-13', title: 'Screw The World', album: 'For All The Dogs', role: 'main', year: 2023, isBlocked: true, duration: '3:00' },
          { id: 'fatd-14', title: 'All The Parties', album: 'For All The Dogs', role: 'main', year: 2023, isBlocked: true, collaborators: ['Chief Keef'], duration: '3:57' },
          { id: 'fatd-15', title: '8am in Charlotte', album: 'For All The Dogs', role: 'main', year: 2023, isBlocked: true, duration: '4:56' },
          { id: 'fatd-16', title: 'Gently', album: 'For All The Dogs', role: 'main', year: 2023, isBlocked: true, collaborators: ['Bad Bunny'], duration: '3:55' },
          { id: 'fatd-17', title: 'Drew a Picasso', album: 'For All The Dogs', role: 'main', year: 2023, isBlocked: true, duration: '2:25' },
          { id: 'fatd-18', title: 'Bahamas Promises', album: 'For All The Dogs', role: 'main', year: 2023, isBlocked: true, duration: '2:45' },
          { id: 'fatd-19', title: 'Away From Home', album: 'For All The Dogs', role: 'main', year: 2023, isBlocked: true, duration: '3:44' },
          { id: 'fatd-20', title: 'Another Late Night', album: 'For All The Dogs', role: 'main', year: 2023, isBlocked: true, collaborators: ['Lil Yachty'], duration: '3:32' },
          { id: 'fatd-21', title: 'BBL', album: 'For All The Dogs', role: 'main', year: 2023, isBlocked: true, duration: '2:49' },
          { id: 'fatd-22', title: 'Polar Opposites', album: 'For All The Dogs', role: 'main', year: 2023, isBlocked: true, duration: '3:09' },
          { id: 'fatd-23', title: 'Stories About My Brother', album: 'For All The Dogs', role: 'main', year: 2023, isBlocked: true, duration: '3:56' },

          // --- HER LOSS (2022) - 16 tracks ---
          { id: 'hl-1', title: 'Rich Flex', album: 'Her Loss', role: 'main', year: 2022, isBlocked: true, collaborators: ['21 Savage'], duration: '3:59' },
          { id: 'hl-2', title: 'Major Distribution', album: 'Her Loss', role: 'main', year: 2022, isBlocked: true, collaborators: ['21 Savage'], duration: '2:46' },
          { id: 'hl-3', title: 'On BS', album: 'Her Loss', role: 'main', year: 2022, isBlocked: true, collaborators: ['21 Savage'], duration: '3:09' },
          { id: 'hl-4', title: 'BackOutsideBoyz', album: 'Her Loss', role: 'main', year: 2022, isBlocked: true, collaborators: ['21 Savage'], duration: '3:14' },
          { id: 'hl-5', title: 'Privileged Rappers', album: 'Her Loss', role: 'main', year: 2022, isBlocked: true, collaborators: ['21 Savage'], duration: '3:54' },
          { id: 'hl-6', title: 'Spin Bout U', album: 'Her Loss', role: 'main', year: 2022, isBlocked: true, collaborators: ['21 Savage'], duration: '3:27' },
          { id: 'hl-7', title: 'Hours In Silence', album: 'Her Loss', role: 'main', year: 2022, isBlocked: true, collaborators: ['21 Savage'], duration: '7:38' },
          { id: 'hl-8', title: 'Treacherous Twins', album: 'Her Loss', role: 'main', year: 2022, isBlocked: true, collaborators: ['21 Savage'], duration: '3:48' },
          { id: 'hl-9', title: 'Circo Loco', album: 'Her Loss', role: 'main', year: 2022, isBlocked: true, collaborators: ['21 Savage'], duration: '3:14' },
          { id: 'hl-10', title: 'Jumbotron Shit Poppin', album: 'Her Loss', role: 'main', year: 2022, isBlocked: true, collaborators: ['21 Savage'], duration: '2:19' },
          { id: 'hl-11', title: 'More M\'s', album: 'Her Loss', role: 'main', year: 2022, isBlocked: true, collaborators: ['21 Savage', 'Travis Scott'], duration: '3:43' },
          { id: 'hl-12', title: '3AM on Glenwood', album: 'Her Loss', role: 'main', year: 2022, isBlocked: true, collaborators: ['21 Savage'], duration: '2:52' },
          { id: 'hl-13', title: 'Middle of the Ocean', album: 'Her Loss', role: 'main', year: 2022, isBlocked: true, collaborators: ['21 Savage'], duration: '2:54' },
          { id: 'hl-14', title: 'Pussy & Millions', album: 'Her Loss', role: 'main', year: 2022, isBlocked: true, collaborators: ['21 Savage', 'Travis Scott'], duration: '3:20' },
          { id: 'hl-15', title: 'Broke Boys', album: 'Her Loss', role: 'main', year: 2022, isBlocked: true, collaborators: ['21 Savage'], duration: '3:39' },
          { id: 'hl-16', title: 'I Guess It\'s Fuck Me', album: 'Her Loss', role: 'main', year: 2022, isBlocked: true, collaborators: ['21 Savage'], duration: '4:04' },

          // --- HONESTLY, NEVERMIND (2022) - 14 tracks ---
          { id: 'hn-1', title: 'Intro', album: 'Honestly, Nevermind', role: 'main', year: 2022, isBlocked: true, duration: '1:36' },
          { id: 'hn-2', title: 'Falling Back', album: 'Honestly, Nevermind', role: 'main', year: 2022, isBlocked: true, duration: '3:01' },
          { id: 'hn-3', title: 'Texts Go Green', album: 'Honestly, Nevermind', role: 'main', year: 2022, isBlocked: true, duration: '2:13' },
          { id: 'hn-4', title: 'Currents', album: 'Honestly, Nevermind', role: 'main', year: 2022, isBlocked: true, duration: '2:53' },
          { id: 'hn-5', title: 'A Keeper', album: 'Honestly, Nevermind', role: 'main', year: 2022, isBlocked: true, duration: '2:40' },
          { id: 'hn-6', title: 'Calling My Name', album: 'Honestly, Nevermind', role: 'main', year: 2022, isBlocked: true, duration: '2:48' },
          { id: 'hn-7', title: 'Sticky', album: 'Honestly, Nevermind', role: 'main', year: 2022, isBlocked: true, duration: '2:50' },
          { id: 'hn-8', title: 'Massive', album: 'Honestly, Nevermind', role: 'main', year: 2022, isBlocked: true, duration: '3:17' },
          { id: 'hn-9', title: 'Flight\'s Booked', album: 'Honestly, Nevermind', role: 'main', year: 2022, isBlocked: true, duration: '2:25' },
          { id: 'hn-10', title: 'Overdrive', album: 'Honestly, Nevermind', role: 'main', year: 2022, isBlocked: true, duration: '3:08' },
          { id: 'hn-11', title: 'Down Hill', album: 'Honestly, Nevermind', role: 'main', year: 2022, isBlocked: true, duration: '2:42' },
          { id: 'hn-12', title: 'Tie That Binds', album: 'Honestly, Nevermind', role: 'main', year: 2022, isBlocked: true, duration: '2:29' },
          { id: 'hn-13', title: 'Liability', album: 'Honestly, Nevermind', role: 'main', year: 2022, isBlocked: true, duration: '2:37' },
          { id: 'hn-14', title: 'Jimmy Cooks', album: 'Honestly, Nevermind', role: 'main', year: 2022, isBlocked: true, collaborators: ['21 Savage'], duration: '3:39' },

          // --- CERTIFIED LOVER BOY (2021) - 21 tracks ---
          { id: 'clb-1', title: 'Champagne Poetry', album: 'Certified Lover Boy', role: 'main', year: 2021, isBlocked: true, duration: '6:07' },
          { id: 'clb-2', title: 'Papi\'s Home', album: 'Certified Lover Boy', role: 'main', year: 2021, isBlocked: true, duration: '2:58' },
          { id: 'clb-3', title: 'Girls Want Girls', album: 'Certified Lover Boy', role: 'main', year: 2021, isBlocked: true, collaborators: ['Lil Baby'], duration: '3:41' },
          { id: 'clb-4', title: 'In The Bible', album: 'Certified Lover Boy', role: 'main', year: 2021, isBlocked: true, collaborators: ['Lil Durk', 'Giveon'], duration: '4:33' },
          { id: 'clb-5', title: 'Love All', album: 'Certified Lover Boy', role: 'main', year: 2021, isBlocked: true, collaborators: ['Jay-Z'], duration: '3:36' },
          { id: 'clb-6', title: 'Fair Trade', album: 'Certified Lover Boy', role: 'main', year: 2021, isBlocked: true, collaborators: ['Travis Scott'], duration: '4:51' },
          { id: 'clb-7', title: 'Way 2 Sexy', album: 'Certified Lover Boy', role: 'main', year: 2021, isBlocked: true, collaborators: ['Future', 'Young Thug'], duration: '4:17' },
          { id: 'clb-8', title: 'TSU', album: 'Certified Lover Boy', role: 'main', year: 2021, isBlocked: true, duration: '5:08' },
          { id: 'clb-9', title: 'N 2 Deep', album: 'Certified Lover Boy', role: 'main', year: 2021, isBlocked: true, collaborators: ['Future'], duration: '4:30' },
          { id: 'clb-10', title: 'Pipe Down', album: 'Certified Lover Boy', role: 'main', year: 2021, isBlocked: true, duration: '3:58' },
          { id: 'clb-11', title: 'Yebba\'s Heartbreak', album: 'Certified Lover Boy', role: 'main', year: 2021, isBlocked: true, collaborators: ['Yebba'], duration: '2:17' },
          { id: 'clb-12', title: 'No Friends In The Industry', album: 'Certified Lover Boy', role: 'main', year: 2021, isBlocked: true, duration: '3:47' },
          { id: 'clb-13', title: 'Knife Talk', album: 'Certified Lover Boy', role: 'main', year: 2021, isBlocked: true, collaborators: ['21 Savage', 'Project Pat'], duration: '4:03' },
          { id: 'clb-14', title: '7am on Bridle Path', album: 'Certified Lover Boy', role: 'main', year: 2021, isBlocked: true, duration: '5:18' },
          { id: 'clb-15', title: 'Race My Mind', album: 'Certified Lover Boy', role: 'main', year: 2021, isBlocked: true, duration: '3:58' },
          { id: 'clb-16', title: 'Fountains', album: 'Certified Lover Boy', role: 'main', year: 2021, isBlocked: true, collaborators: ['Tems'], duration: '3:18' },
          { id: 'clb-17', title: 'Get Along Better', album: 'Certified Lover Boy', role: 'main', year: 2021, isBlocked: true, collaborators: ['Ty Dolla $ign'], duration: '3:16' },
          { id: 'clb-18', title: 'You Only Live Twice', album: 'Certified Lover Boy', role: 'main', year: 2021, isBlocked: true, collaborators: ['Lil Wayne', 'Rick Ross'], duration: '4:57' },
          { id: 'clb-19', title: 'IMY2', album: 'Certified Lover Boy', role: 'main', year: 2021, isBlocked: true, collaborators: ['Kid Cudi'], duration: '3:18' },
          { id: 'clb-20', title: 'F**king Fans', album: 'Certified Lover Boy', role: 'main', year: 2021, isBlocked: true, duration: '6:38' },
          { id: 'clb-21', title: 'The Remorse', album: 'Certified Lover Boy', role: 'main', year: 2021, isBlocked: true, duration: '4:41' },

          // --- DARK LANE DEMO TAPES (2020) - 14 tracks ---
          { id: 'dldt-1', title: 'Deep Pockets', album: 'Dark Lane Demo Tapes', role: 'main', year: 2020, isBlocked: true, duration: '2:55' },
          { id: 'dldt-2', title: 'When To Say When', album: 'Dark Lane Demo Tapes', role: 'main', year: 2020, isBlocked: true, duration: '5:28' },
          { id: 'dldt-3', title: 'Chicago Freestyle', album: 'Dark Lane Demo Tapes', role: 'main', year: 2020, isBlocked: true, collaborators: ['Giveon'], duration: '4:37' },
          { id: 'dldt-4', title: 'Not You Too', album: 'Dark Lane Demo Tapes', role: 'main', year: 2020, isBlocked: true, collaborators: ['Chris Brown'], duration: '4:15' },
          { id: 'dldt-5', title: 'Toosie Slide', album: 'Dark Lane Demo Tapes', role: 'main', year: 2020, isBlocked: true, duration: '4:07' },
          { id: 'dldt-6', title: 'Desires', album: 'Dark Lane Demo Tapes', role: 'main', year: 2020, isBlocked: true, collaborators: ['Future'], duration: '3:55' },
          { id: 'dldt-7', title: 'Time Flies', album: 'Dark Lane Demo Tapes', role: 'main', year: 2020, isBlocked: true, duration: '3:01' },
          { id: 'dldt-8', title: 'Landed', album: 'Dark Lane Demo Tapes', role: 'main', year: 2020, isBlocked: true, duration: '2:50' },
          { id: 'dldt-9', title: 'D4L', album: 'Dark Lane Demo Tapes', role: 'main', year: 2020, isBlocked: true, collaborators: ['Future', 'Young Thug'], duration: '3:27' },
          { id: 'dldt-10', title: 'Pain 1993', album: 'Dark Lane Demo Tapes', role: 'main', year: 2020, isBlocked: true, collaborators: ['Playboi Carti'], duration: '2:32' },
          { id: 'dldt-11', title: 'Losses', album: 'Dark Lane Demo Tapes', role: 'main', year: 2020, isBlocked: true, duration: '4:16' },
          { id: 'dldt-12', title: 'From Florida With Love', album: 'Dark Lane Demo Tapes', role: 'main', year: 2020, isBlocked: true, duration: '3:22' },
          { id: 'dldt-13', title: 'Demons', album: 'Dark Lane Demo Tapes', role: 'main', year: 2020, isBlocked: true, collaborators: ['Fivio Foreign', 'Sosa Geek'], duration: '3:58' },
          { id: 'dldt-14', title: 'War', album: 'Dark Lane Demo Tapes', role: 'main', year: 2020, isBlocked: true, duration: '3:25' },

          // --- SCORPION (2018) - 25 tracks ---
          { id: 'scorp-1', title: 'Survival', album: 'Scorpion', role: 'main', year: 2018, isBlocked: true, duration: '2:36' },
          { id: 'scorp-2', title: 'Nonstop', album: 'Scorpion', role: 'main', year: 2018, isBlocked: true, duration: '3:58' },
          { id: 'scorp-3', title: 'Elevate', album: 'Scorpion', role: 'main', year: 2018, isBlocked: true, duration: '3:04' },
          { id: 'scorp-4', title: 'Emotionless', album: 'Scorpion', role: 'main', year: 2018, isBlocked: true, collaborators: ['Mariah Carey'], duration: '5:02' },
          { id: 'scorp-5', title: 'God\'s Plan', album: 'Scorpion', role: 'main', year: 2018, isBlocked: true, duration: '3:19' },
          { id: 'scorp-6', title: 'I\'m Upset', album: 'Scorpion', role: 'main', year: 2018, isBlocked: true, duration: '3:34' },
          { id: 'scorp-7', title: '8 Out of 10', album: 'Scorpion', role: 'main', year: 2018, isBlocked: true, duration: '3:21' },
          { id: 'scorp-8', title: 'Mob Ties', album: 'Scorpion', role: 'main', year: 2018, isBlocked: true, duration: '3:29' },
          { id: 'scorp-9', title: 'Can\'t Take a Joke', album: 'Scorpion', role: 'main', year: 2018, isBlocked: true, duration: '3:20' },
          { id: 'scorp-10', title: 'Sandra\'s Rose', album: 'Scorpion', role: 'main', year: 2018, isBlocked: true, duration: '4:23' },
          { id: 'scorp-11', title: 'Talk Up', album: 'Scorpion', role: 'main', year: 2018, isBlocked: true, collaborators: ['Jay-Z'], duration: '3:17' },
          { id: 'scorp-12', title: 'Is There More', album: 'Scorpion', role: 'main', year: 2018, isBlocked: true, duration: '4:33' },
          { id: 'scorp-13', title: 'Peak', album: 'Scorpion', role: 'main', year: 2018, isBlocked: true, duration: '2:56' },
          { id: 'scorp-14', title: 'Summer Games', album: 'Scorpion', role: 'main', year: 2018, isBlocked: true, duration: '3:30' },
          { id: 'scorp-15', title: 'Jaded', album: 'Scorpion', role: 'main', year: 2018, isBlocked: true, duration: '4:14' },
          { id: 'scorp-16', title: 'Nice For What', album: 'Scorpion', role: 'main', year: 2018, isBlocked: true, duration: '3:31' },
          { id: 'scorp-17', title: 'Finesse', album: 'Scorpion', role: 'main', year: 2018, isBlocked: true, duration: '3:10' },
          { id: 'scorp-18', title: 'Ratchet Happy Birthday', album: 'Scorpion', role: 'main', year: 2018, isBlocked: true, duration: '2:34' },
          { id: 'scorp-19', title: 'That\'s How You Feel', album: 'Scorpion', role: 'main', year: 2018, isBlocked: true, duration: '3:36' },
          { id: 'scorp-20', title: 'Blue Tint', album: 'Scorpion', role: 'main', year: 2018, isBlocked: true, duration: '3:25' },
          { id: 'scorp-21', title: 'In My Feelings', album: 'Scorpion', role: 'main', year: 2018, isBlocked: true, duration: '3:37' },
          { id: 'scorp-22', title: 'Don\'t Matter to Me', album: 'Scorpion', role: 'main', year: 2018, isBlocked: true, collaborators: ['Michael Jackson'], duration: '4:00' },
          { id: 'scorp-23', title: 'After Dark', album: 'Scorpion', role: 'main', year: 2018, isBlocked: true, collaborators: ['Ty Dolla $ign', 'Static Major'], duration: '4:32' },
          { id: 'scorp-24', title: 'Final Fantasy', album: 'Scorpion', role: 'main', year: 2018, isBlocked: true, duration: '3:14' },
          { id: 'scorp-25', title: 'March 14', album: 'Scorpion', role: 'main', year: 2018, isBlocked: true, duration: '5:09' },

          // --- MORE LIFE (2017) - 22 tracks ---
          { id: 'ml-1', title: 'Free Smoke', album: 'More Life', role: 'main', year: 2017, isBlocked: true, duration: '3:36' },
          { id: 'ml-2', title: 'No Long Talk', album: 'More Life', role: 'main', year: 2017, isBlocked: true, collaborators: ['Giggs'], duration: '2:33' },
          { id: 'ml-3', title: 'Passionfruit', album: 'More Life', role: 'main', year: 2017, isBlocked: true, duration: '4:58' },
          { id: 'ml-4', title: 'Jorja Interlude', album: 'More Life', role: 'main', year: 2017, isBlocked: true, duration: '1:49' },
          { id: 'ml-5', title: 'Get It Together', album: 'More Life', role: 'main', year: 2017, isBlocked: true, collaborators: ['Jorja Smith', 'Black Coffee'], duration: '5:07' },
          { id: 'ml-6', title: 'Madiba Riddim', album: 'More Life', role: 'main', year: 2017, isBlocked: true, duration: '3:13' },
          { id: 'ml-7', title: 'Blem', album: 'More Life', role: 'main', year: 2017, isBlocked: true, duration: '3:26' },
          { id: 'ml-8', title: '4422', album: 'More Life', role: 'main', year: 2017, isBlocked: true, collaborators: ['Sampha'], duration: '4:27' },
          { id: 'ml-9', title: 'Gyalchester', album: 'More Life', role: 'main', year: 2017, isBlocked: true, duration: '3:09' },
          { id: 'ml-10', title: 'Skepta Interlude', album: 'More Life', role: 'main', year: 2017, isBlocked: true, duration: '2:23' },
          { id: 'ml-11', title: 'Portland', album: 'More Life', role: 'main', year: 2017, isBlocked: true, collaborators: ['Quavo', 'Travis Scott'], duration: '3:57' },
          { id: 'ml-12', title: 'Sacrifices', album: 'More Life', role: 'main', year: 2017, isBlocked: true, collaborators: ['2 Chainz', 'Young Thug'], duration: '5:41' },
          { id: 'ml-13', title: 'Nothings Into Somethings', album: 'More Life', role: 'main', year: 2017, isBlocked: true, duration: '4:15' },
          { id: 'ml-14', title: 'Teenage Fever', album: 'More Life', role: 'main', year: 2017, isBlocked: true, duration: '3:40' },
          { id: 'ml-15', title: 'KMT', album: 'More Life', role: 'main', year: 2017, isBlocked: true, collaborators: ['Giggs'], duration: '3:06' },
          { id: 'ml-16', title: 'Lose You', album: 'More Life', role: 'main', year: 2017, isBlocked: true, duration: '3:23' },
          { id: 'ml-17', title: 'Can\'t Have Everything', album: 'More Life', role: 'main', year: 2017, isBlocked: true, duration: '4:15' },
          { id: 'ml-18', title: 'Glow', album: 'More Life', role: 'main', year: 2017, isBlocked: true, collaborators: ['Kanye West'], duration: '3:21' },
          { id: 'ml-19', title: 'Since Way Back', album: 'More Life', role: 'main', year: 2017, isBlocked: true, collaborators: ['PARTYNEXTDOOR'], duration: '4:29' },
          { id: 'ml-20', title: 'Fake Love', album: 'More Life', role: 'main', year: 2017, isBlocked: true, duration: '3:27' },
          { id: 'ml-21', title: 'Ice Melts', album: 'More Life', role: 'main', year: 2017, isBlocked: true, collaborators: ['Young Thug'], duration: '3:35' },
          { id: 'ml-22', title: 'Do Not Disturb', album: 'More Life', role: 'main', year: 2017, isBlocked: true, duration: '5:07' },

          // --- VIEWS (2016) - 20 tracks ---
          { id: 'views-1', title: 'Keep the Family Close', album: 'Views', role: 'main', year: 2016, isBlocked: true, duration: '6:25' },
          { id: 'views-2', title: '9', album: 'Views', role: 'main', year: 2016, isBlocked: true, duration: '4:04' },
          { id: 'views-3', title: 'U With Me?', album: 'Views', role: 'main', year: 2016, isBlocked: true, duration: '6:06' },
          { id: 'views-4', title: 'Feel No Ways', album: 'Views', role: 'main', year: 2016, isBlocked: true, duration: '4:34' },
          { id: 'views-5', title: 'Hype', album: 'Views', role: 'main', year: 2016, isBlocked: true, duration: '3:28' },
          { id: 'views-6', title: 'Weston Road Flows', album: 'Views', role: 'main', year: 2016, isBlocked: true, duration: '4:47' },
          { id: 'views-7', title: 'Redemption', album: 'Views', role: 'main', year: 2016, isBlocked: true, duration: '5:32' },
          { id: 'views-8', title: 'With You', album: 'Views', role: 'main', year: 2016, isBlocked: true, collaborators: ['PARTYNEXTDOOR'], duration: '4:32' },
          { id: 'views-9', title: 'Faithful', album: 'Views', role: 'main', year: 2016, isBlocked: true, collaborators: ['Pimp C', 'dvsn'], duration: '4:52' },
          { id: 'views-10', title: 'Still Here', album: 'Views', role: 'main', year: 2016, isBlocked: true, duration: '3:42' },
          { id: 'views-11', title: 'Controlla', album: 'Views', role: 'main', year: 2016, isBlocked: true, duration: '4:04' },
          { id: 'views-12', title: 'One Dance', album: 'Views', role: 'main', year: 2016, isBlocked: true, collaborators: ['Wizkid', 'Kyla'], duration: '2:54' },
          { id: 'views-13', title: 'Grammys', album: 'Views', role: 'main', year: 2016, isBlocked: true, collaborators: ['Future'], duration: '4:11' },
          { id: 'views-14', title: 'Child\'s Play', album: 'Views', role: 'main', year: 2016, isBlocked: true, duration: '3:56' },
          { id: 'views-15', title: 'Pop Style', album: 'Views', role: 'main', year: 2016, isBlocked: true, duration: '3:27' },
          { id: 'views-16', title: 'Too Good', album: 'Views', role: 'main', year: 2016, isBlocked: true, collaborators: ['Rihanna'], duration: '4:23' },
          { id: 'views-17', title: 'Summers Over Interlude', album: 'Views', role: 'main', year: 2016, isBlocked: true, collaborators: ['Majid Jordan'], duration: '1:58' },
          { id: 'views-18', title: 'Fire & Desire', album: 'Views', role: 'main', year: 2016, isBlocked: true, duration: '4:55' },
          { id: 'views-19', title: 'Views', album: 'Views', role: 'main', year: 2016, isBlocked: true, duration: '4:46' },
          { id: 'views-20', title: 'Hotline Bling', album: 'Views', role: 'main', year: 2016, isBlocked: true, duration: '4:27' },

          // --- IF YOU'RE READING THIS IT'S TOO LATE (2015) - 17 tracks ---
          { id: 'iyrtitl-1', title: 'Legend', album: "If You're Reading This It's Too Late", role: 'main', year: 2015, isBlocked: true, duration: '3:17' },
          { id: 'iyrtitl-2', title: 'Energy', album: "If You're Reading This It's Too Late", role: 'main', year: 2015, isBlocked: true, duration: '3:03' },
          { id: 'iyrtitl-3', title: '10 Bands', album: "If You're Reading This It's Too Late", role: 'main', year: 2015, isBlocked: true, duration: '2:58' },
          { id: 'iyrtitl-4', title: 'Know Yourself', album: "If You're Reading This It's Too Late", role: 'main', year: 2015, isBlocked: true, duration: '4:27' },
          { id: 'iyrtitl-5', title: 'No Tellin\'', album: "If You're Reading This It's Too Late", role: 'main', year: 2015, isBlocked: true, duration: '4:05' },
          { id: 'iyrtitl-6', title: 'Madonna', album: "If You're Reading This It's Too Late", role: 'main', year: 2015, isBlocked: true, duration: '4:00' },
          { id: 'iyrtitl-7', title: '6 God', album: "If You're Reading This It's Too Late", role: 'main', year: 2015, isBlocked: true, duration: '2:57' },
          { id: 'iyrtitl-8', title: 'Star67', album: "If You're Reading This It's Too Late", role: 'main', year: 2015, isBlocked: true, duration: '4:27' },
          { id: 'iyrtitl-9', title: 'Preach', album: "If You're Reading This It's Too Late", role: 'main', year: 2015, isBlocked: true, collaborators: ['PARTYNEXTDOOR'], duration: '3:28' },
          { id: 'iyrtitl-10', title: 'Wednesday Night Interlude', album: "If You're Reading This It's Too Late", role: 'main', year: 2015, isBlocked: true, collaborators: ['PARTYNEXTDOOR'], duration: '3:47' },
          { id: 'iyrtitl-11', title: 'Used To', album: "If You're Reading This It's Too Late", role: 'main', year: 2015, isBlocked: true, collaborators: ['Lil Wayne'], duration: '4:06' },
          { id: 'iyrtitl-12', title: '6 Man', album: "If You're Reading This It's Too Late", role: 'main', year: 2015, isBlocked: true, duration: '3:04' },
          { id: 'iyrtitl-13', title: 'Now & Forever', album: "If You're Reading This It's Too Late", role: 'main', year: 2015, isBlocked: true, duration: '3:33' },
          { id: 'iyrtitl-14', title: 'Company', album: "If You're Reading This It's Too Late", role: 'main', year: 2015, isBlocked: true, collaborators: ['Travis Scott'], duration: '4:07' },
          { id: 'iyrtitl-15', title: 'You & The 6', album: "If You're Reading This It's Too Late", role: 'main', year: 2015, isBlocked: true, duration: '5:04' },
          { id: 'iyrtitl-16', title: 'Jungle', album: "If You're Reading This It's Too Late", role: 'main', year: 2015, isBlocked: true, duration: '5:49' },
          { id: 'iyrtitl-17', title: '6PM in New York', album: "If You're Reading This It's Too Late", role: 'main', year: 2015, isBlocked: true, duration: '4:49' },

          // --- NOTHING WAS THE SAME (2013) - 15 tracks ---
          { id: 'nwts-1', title: 'Tuscan Leather', album: 'Nothing Was the Same', role: 'main', year: 2013, isBlocked: true, duration: '6:06' },
          { id: 'nwts-2', title: 'Furthest Thing', album: 'Nothing Was the Same', role: 'main', year: 2013, isBlocked: true, duration: '5:34' },
          { id: 'nwts-3', title: 'Started From the Bottom', album: 'Nothing Was the Same', role: 'main', year: 2013, isBlocked: true, duration: '2:59' },
          { id: 'nwts-4', title: 'Wu-Tang Forever', album: 'Nothing Was the Same', role: 'main', year: 2013, isBlocked: true, duration: '4:35' },
          { id: 'nwts-5', title: 'Own It', album: 'Nothing Was the Same', role: 'main', year: 2013, isBlocked: true, duration: '3:54' },
          { id: 'nwts-6', title: 'Worst Behavior', album: 'Nothing Was the Same', role: 'main', year: 2013, isBlocked: true, duration: '4:30' },
          { id: 'nwts-7', title: 'From Time', album: 'Nothing Was the Same', role: 'main', year: 2013, isBlocked: true, collaborators: ['Jhené Aiko'], duration: '5:23' },
          { id: 'nwts-8', title: 'Hold On, We\'re Going Home', album: 'Nothing Was the Same', role: 'main', year: 2013, isBlocked: true, collaborators: ['Majid Jordan'], duration: '3:47' },
          { id: 'nwts-9', title: 'Connect', album: 'Nothing Was the Same', role: 'main', year: 2013, isBlocked: true, duration: '4:30' },
          { id: 'nwts-10', title: 'The Language', album: 'Nothing Was the Same', role: 'main', year: 2013, isBlocked: true, duration: '5:07' },
          { id: 'nwts-11', title: '305 To My City', album: 'Nothing Was the Same', role: 'main', year: 2013, isBlocked: true, collaborators: ['Detail'], duration: '2:54' },
          { id: 'nwts-12', title: 'Too Much', album: 'Nothing Was the Same', role: 'main', year: 2013, isBlocked: true, collaborators: ['Sampha'], duration: '5:28' },
          { id: 'nwts-13', title: 'Pound Cake / Paris Morton Music 2', album: 'Nothing Was the Same', role: 'main', year: 2013, isBlocked: true, collaborators: ['Jay-Z'], duration: '8:11' },
          { id: 'nwts-14', title: 'Come Thru', album: 'Nothing Was the Same', role: 'main', year: 2013, isBlocked: true, duration: '4:20' },
          { id: 'nwts-15', title: 'All Me', album: 'Nothing Was the Same', role: 'main', year: 2013, isBlocked: true, collaborators: ['2 Chainz', 'Big Sean'], duration: '5:06' },

          // --- TAKE CARE (2011) - 20 tracks ---
          { id: 'tc-1', title: 'Over My Dead Body', album: 'Take Care', role: 'main', year: 2011, isBlocked: true, duration: '4:46' },
          { id: 'tc-2', title: 'Shot For Me', album: 'Take Care', role: 'main', year: 2011, isBlocked: true, duration: '4:22' },
          { id: 'tc-3', title: 'Headlines', album: 'Take Care', role: 'main', year: 2011, isBlocked: true, duration: '3:56' },
          { id: 'tc-4', title: 'Crew Love', album: 'Take Care', role: 'main', year: 2011, isBlocked: true, collaborators: ['The Weeknd'], duration: '3:28' },
          { id: 'tc-5', title: 'Take Care', album: 'Take Care', role: 'main', year: 2011, isBlocked: true, collaborators: ['Rihanna'], duration: '4:37' },
          { id: 'tc-6', title: 'Marvin\'s Room', album: 'Take Care', role: 'main', year: 2011, isBlocked: true, duration: '5:47' },
          { id: 'tc-7', title: 'Buried Alive Interlude', album: 'Take Care', role: 'main', year: 2011, isBlocked: true, collaborators: ['Kendrick Lamar'], duration: '2:37' },
          { id: 'tc-8', title: 'Under Ground Kings', album: 'Take Care', role: 'main', year: 2011, isBlocked: true, duration: '5:33' },
          { id: 'tc-9', title: 'We\'ll Be Fine', album: 'Take Care', role: 'main', year: 2011, isBlocked: true, collaborators: ['Birdman'], duration: '3:49' },
          { id: 'tc-10', title: 'Make Me Proud', album: 'Take Care', role: 'main', year: 2011, isBlocked: true, collaborators: ['Nicki Minaj'], duration: '4:09' },
          { id: 'tc-11', title: 'Lord Knows', album: 'Take Care', role: 'main', year: 2011, isBlocked: true, collaborators: ['Rick Ross'], duration: '4:47' },
          { id: 'tc-12', title: 'Cameras / Good Ones Go Interlude', album: 'Take Care', role: 'main', year: 2011, isBlocked: true, duration: '6:06' },
          { id: 'tc-13', title: 'Doing It Wrong', album: 'Take Care', role: 'main', year: 2011, isBlocked: true, collaborators: ['Stevie Wonder'], duration: '4:17' },
          { id: 'tc-14', title: 'The Real Her', album: 'Take Care', role: 'main', year: 2011, isBlocked: true, collaborators: ['Lil Wayne', 'Andre 3000'], duration: '5:16' },
          { id: 'tc-15', title: 'Look What You\'ve Done', album: 'Take Care', role: 'main', year: 2011, isBlocked: true, duration: '5:55' },
          { id: 'tc-16', title: 'HYFR (Hell Ya Fucking Right)', album: 'Take Care', role: 'main', year: 2011, isBlocked: true, collaborators: ['Lil Wayne'], duration: '3:28' },
          { id: 'tc-17', title: 'Practice', album: 'Take Care', role: 'main', year: 2011, isBlocked: true, duration: '2:55' },
          { id: 'tc-18', title: 'The Ride', album: 'Take Care', role: 'main', year: 2011, isBlocked: true, collaborators: ['The Weeknd'], duration: '4:25' },
          { id: 'tc-19', title: 'The Motto', album: 'Take Care', role: 'main', year: 2011, isBlocked: true, collaborators: ['Lil Wayne'], duration: '3:00' },
          { id: 'tc-20', title: 'Hate Sleeping Alone', album: 'Take Care', role: 'main', year: 2011, isBlocked: true, duration: '4:08' },

          // --- THANK ME LATER (2010) - 14 tracks ---
          { id: 'tml-1', title: 'Fireworks', album: 'Thank Me Later', role: 'main', year: 2010, isBlocked: true, collaborators: ['Alicia Keys'], duration: '4:57' },
          { id: 'tml-2', title: 'Karaoke', album: 'Thank Me Later', role: 'main', year: 2010, isBlocked: true, duration: '4:15' },
          { id: 'tml-3', title: 'The Resistance', album: 'Thank Me Later', role: 'main', year: 2010, isBlocked: true, duration: '3:53' },
          { id: 'tml-4', title: 'Over', album: 'Thank Me Later', role: 'main', year: 2010, isBlocked: true, duration: '4:14' },
          { id: 'tml-5', title: 'Show Me a Good Time', album: 'Thank Me Later', role: 'main', year: 2010, isBlocked: true, duration: '4:06' },
          { id: 'tml-6', title: 'Up All Night', album: 'Thank Me Later', role: 'main', year: 2010, isBlocked: true, collaborators: ['Nicki Minaj'], duration: '4:25' },
          { id: 'tml-7', title: 'Fancy', album: 'Thank Me Later', role: 'main', year: 2010, isBlocked: true, collaborators: ['T.I.', 'Swizz Beatz'], duration: '4:55' },
          { id: 'tml-8', title: 'Shut It Down', album: 'Thank Me Later', role: 'main', year: 2010, isBlocked: true, collaborators: ['The-Dream'], duration: '4:29' },
          { id: 'tml-9', title: 'Unforgettable', album: 'Thank Me Later', role: 'main', year: 2010, isBlocked: true, collaborators: ['Young Jeezy'], duration: '4:02' },
          { id: 'tml-10', title: 'Light Up', album: 'Thank Me Later', role: 'main', year: 2010, isBlocked: true, collaborators: ['Jay-Z'], duration: '5:01' },
          { id: 'tml-11', title: 'Miss Me', album: 'Thank Me Later', role: 'main', year: 2010, isBlocked: true, collaborators: ['Lil Wayne'], duration: '5:06' },
          { id: 'tml-12', title: 'Cece\'s Interlude', album: 'Thank Me Later', role: 'main', year: 2010, isBlocked: true, duration: '1:40' },
          { id: 'tml-13', title: 'Find Your Love', album: 'Thank Me Later', role: 'main', year: 2010, isBlocked: true, duration: '4:06' },
          { id: 'tml-14', title: 'Thank Me Now', album: 'Thank Me Later', role: 'main', year: 2010, isBlocked: true, duration: '6:02' },

          // --- SO FAR GONE (2009) - 7 tracks (EP version) ---
          { id: 'sfg-1', title: 'Lust For Life', album: 'So Far Gone', role: 'main', year: 2009, isBlocked: true, duration: '4:12' },
          { id: 'sfg-2', title: 'Houstatlantavegas', album: 'So Far Gone', role: 'main', year: 2009, isBlocked: true, duration: '4:22' },
          { id: 'sfg-3', title: 'Successful', album: 'So Far Gone', role: 'main', year: 2009, isBlocked: true, collaborators: ['Trey Songz', 'Lil Wayne'], duration: '4:55' },
          { id: 'sfg-4', title: 'Best I Ever Had', album: 'So Far Gone', role: 'main', year: 2009, isBlocked: true, duration: '4:19' },
          { id: 'sfg-5', title: 'I\'m Goin\' In', album: 'So Far Gone', role: 'main', year: 2009, isBlocked: true, collaborators: ['Lil Wayne', 'Young Jeezy'], duration: '4:34' },
          { id: 'sfg-6', title: 'Ignorant Shit', album: 'So Far Gone', role: 'main', year: 2009, isBlocked: true, collaborators: ['Lil Wayne'], duration: '3:12' },
          { id: 'sfg-7', title: 'Brand New', album: 'So Far Gone', role: 'main', year: 2009, isBlocked: true, duration: '4:08' },

          // --- WHAT A TIME TO BE ALIVE (2015) with Future - 11 tracks ---
          { id: 'wattba-1', title: 'Digital Dash', album: 'What a Time to Be Alive', role: 'main', year: 2015, isBlocked: true, collaborators: ['Future'], duration: '3:23' },
          { id: 'wattba-2', title: 'Big Rings', album: 'What a Time to Be Alive', role: 'main', year: 2015, isBlocked: true, collaborators: ['Future'], duration: '4:08' },
          { id: 'wattba-3', title: 'Live from the Gutter', album: 'What a Time to Be Alive', role: 'main', year: 2015, isBlocked: true, collaborators: ['Future'], duration: '3:49' },
          { id: 'wattba-4', title: 'Diamonds Dancing', album: 'What a Time to Be Alive', role: 'main', year: 2015, isBlocked: true, collaborators: ['Future'], duration: '5:36' },
          { id: 'wattba-5', title: 'Scholarships', album: 'What a Time to Be Alive', role: 'main', year: 2015, isBlocked: true, collaborators: ['Future'], duration: '4:03' },
          { id: 'wattba-6', title: 'Plastic Bag', album: 'What a Time to Be Alive', role: 'main', year: 2015, isBlocked: true, collaborators: ['Future'], duration: '3:47' },
          { id: 'wattba-7', title: 'I\'m the Plug', album: 'What a Time to Be Alive', role: 'main', year: 2015, isBlocked: true, collaborators: ['Future'], duration: '3:17' },
          { id: 'wattba-8', title: 'Change Locations', album: 'What a Time to Be Alive', role: 'main', year: 2015, isBlocked: true, collaborators: ['Future'], duration: '3:49' },
          { id: 'wattba-9', title: 'Jumpman', album: 'What a Time to Be Alive', role: 'main', year: 2015, isBlocked: true, collaborators: ['Future'], duration: '3:18' },
          { id: 'wattba-10', title: 'Jersey', album: 'What a Time to Be Alive', role: 'main', year: 2015, isBlocked: true, collaborators: ['Future'], duration: '3:31' },
          { id: 'wattba-11', title: '30 for 30 Freestyle', album: 'What a Time to Be Alive', role: 'main', year: 2015, isBlocked: true, collaborators: ['Future'], duration: '4:47' },

          // ========================================
          // FEATURED APPEARANCES (50+)
          // ========================================
          { id: 'feat-1', title: 'Sicko Mode', album: 'ASTROWORLD', role: 'featured', year: 2018, isBlocked: true, collaborators: ['Travis Scott'], duration: '5:12' },
          { id: 'feat-2', title: 'Work', album: 'ANTI', role: 'featured', year: 2016, isBlocked: true, collaborators: ['Rihanna'], duration: '3:39' },
          { id: 'feat-3', title: 'Poetic Justice', album: 'good kid, m.A.A.d city', role: 'featured', year: 2012, isBlocked: true, collaborators: ['Kendrick Lamar'], duration: '5:00' },
          { id: 'feat-4', title: 'No Guidance', album: 'Indigo', role: 'featured', year: 2019, isBlocked: true, collaborators: ['Chris Brown'], duration: '4:22' },
          { id: 'feat-5', title: 'Forever', album: 'More Than a Game OST', role: 'featured', year: 2009, isBlocked: true, collaborators: ['Kanye West', 'Lil Wayne', 'Eminem'], duration: '5:27' },
          { id: 'feat-6', title: 'Versace (Remix)', album: 'Single', role: 'featured', year: 2013, isBlocked: true, collaborators: ['Migos'], duration: '5:24' },
          { id: 'feat-7', title: 'Only', album: 'The Pinkprint', role: 'featured', year: 2014, isBlocked: true, collaborators: ['Nicki Minaj', 'Lil Wayne', 'Chris Brown'], duration: '5:28' },
          { id: 'feat-8', title: 'POPSTAR', album: 'KHALED KHALED', role: 'featured', year: 2020, isBlocked: true, collaborators: ['DJ Khaled'], duration: '3:29' },
          { id: 'feat-9', title: 'GREECE', album: 'KHALED KHALED', role: 'featured', year: 2020, isBlocked: true, collaborators: ['DJ Khaled'], duration: '3:07' },
          { id: 'feat-10', title: 'Life Is Good', album: 'High Off Life', role: 'featured', year: 2020, isBlocked: true, collaborators: ['Future'], duration: '3:57' },
          { id: 'feat-11', title: 'Stay Schemin\'', album: 'Rich Forever', role: 'featured', year: 2012, isBlocked: true, collaborators: ['Rick Ross', 'French Montana'], duration: '4:49' },
          { id: 'feat-12', title: 'Walk It Talk It', album: 'Culture II', role: 'featured', year: 2018, isBlocked: true, collaborators: ['Migos'], duration: '4:30' },
          { id: 'feat-13', title: 'I Got the Keys', album: 'Major Key', role: 'featured', year: 2016, isBlocked: true, collaborators: ['DJ Khaled', 'Jay-Z', 'Future'], duration: '4:39' },
          { id: 'feat-14', title: 'For Free', album: 'Major Key', role: 'featured', year: 2016, isBlocked: true, collaborators: ['DJ Khaled'], duration: '2:33' },
          { id: 'feat-15', title: 'No New Friends', album: 'Suffering from Success', role: 'featured', year: 2013, isBlocked: true, collaborators: ['DJ Khaled', 'Rick Ross', 'Lil Wayne'], duration: '4:48' },
          { id: 'feat-16', title: 'Aston Martin Music', album: 'Teflon Don', role: 'featured', year: 2010, isBlocked: true, collaborators: ['Rick Ross', 'Chrisette Michele'], duration: '5:44' },
          { id: 'feat-17', title: 'Money in the Grave', album: 'The Best in the World Pack', role: 'featured', year: 2019, isBlocked: true, collaborators: ['Rick Ross'], duration: '3:47' },
          { id: 'feat-18', title: 'Going Bad', album: 'Championships', role: 'featured', year: 2018, isBlocked: true, collaborators: ['Meek Mill'], duration: '2:51' },
          { id: 'feat-19', title: 'Lemon (Remix)', album: 'No One Ever Really Dies', role: 'featured', year: 2018, isBlocked: true, collaborators: ['N.E.R.D', 'Rihanna'], duration: '4:18' },
          { id: 'feat-20', title: 'Both', album: 'Droptopwop', role: 'featured', year: 2017, isBlocked: true, collaborators: ['Gucci Mane'], duration: '3:46' },
          { id: 'feat-21', title: 'My Way (Remix)', album: 'Single', role: 'featured', year: 2016, isBlocked: true, collaborators: ['Fetty Wap'], duration: '4:07' },
          { id: 'feat-22', title: 'Believe Me', album: 'Single', role: 'featured', year: 2014, isBlocked: true, collaborators: ['Lil Wayne'], duration: '4:59' },
          { id: 'feat-23', title: 'Tuesday', album: 'Days Before Rodeo', role: 'featured', year: 2014, isBlocked: true, collaborators: ['ILoveMakonnen'], duration: '4:40' },
          { id: 'feat-24', title: 'Draft Day', album: 'Single', role: 'featured', year: 2014, isBlocked: true, duration: '5:35' },
          { id: 'feat-25', title: 'Trophies', album: 'Single', role: 'featured', year: 2014, isBlocked: true, duration: '3:37' },
          { id: 'feat-26', title: '100', album: 'Single', role: 'featured', year: 2014, isBlocked: true, duration: '3:33' },
          { id: 'feat-27', title: '0 to 100 / The Catch Up', album: 'Single', role: 'featured', year: 2014, isBlocked: true, duration: '5:18' },
          { id: 'feat-28', title: 'We Made It', album: 'Single', role: 'featured', year: 2014, isBlocked: true, collaborators: ['Soulja Boy'], duration: '3:59' },
          { id: 'feat-29', title: 'The Language', album: 'Single', role: 'featured', year: 2013, isBlocked: true, duration: '5:07' },
          { id: 'feat-30', title: '5AM in Toronto', album: 'Single', role: 'featured', year: 2013, isBlocked: true, duration: '3:43' },
          { id: 'feat-31', title: 'Girls Love Beyoncé', album: 'Single', role: 'featured', year: 2013, isBlocked: true, collaborators: ['James Fauntleroy'], duration: '4:47' },
          { id: 'feat-32', title: 'Worst Behavior', album: 'Single', role: 'featured', year: 2013, isBlocked: true, duration: '4:30' },
          { id: 'feat-33', title: 'The Motion', album: 'Single', role: 'featured', year: 2013, isBlocked: true, collaborators: ['Sampha'], duration: '5:15' },
          { id: 'feat-34', title: 'All Me', album: 'Single', role: 'featured', year: 2013, isBlocked: true, collaborators: ['2 Chainz', 'Big Sean'], duration: '5:06' },
          { id: 'feat-35', title: 'Right Above It', album: 'I Am Not a Human Being', role: 'featured', year: 2010, isBlocked: true, collaborators: ['Lil Wayne'], duration: '3:48' },
          { id: 'feat-36', title: 'Every Girl', album: 'We Are Young Money', role: 'featured', year: 2009, isBlocked: true, collaborators: ['Young Money'], duration: '5:00' },
          { id: 'feat-37', title: 'BedRock', album: 'We Are Young Money', role: 'featured', year: 2009, isBlocked: true, collaborators: ['Young Money'], duration: '4:44' },
          { id: 'feat-38', title: 'What\'s My Name?', album: 'Loud', role: 'featured', year: 2010, isBlocked: true, collaborators: ['Rihanna'], duration: '4:23' },
          { id: 'feat-39', title: 'Moment 4 Life', album: 'Pink Friday', role: 'featured', year: 2010, isBlocked: true, collaborators: ['Nicki Minaj'], duration: '4:39' },
          { id: 'feat-40', title: 'Up All Night', album: 'Pink Friday', role: 'featured', year: 2010, isBlocked: true, collaborators: ['Nicki Minaj'], duration: '4:25' },
          { id: 'feat-41', title: 'Make Me Proud', album: 'Single', role: 'featured', year: 2011, isBlocked: true, collaborators: ['Nicki Minaj'], duration: '4:09' },
          { id: 'feat-42', title: 'Amen', album: 'God Forgives, I Don\'t', role: 'featured', year: 2012, isBlocked: true, collaborators: ['Meek Mill'], duration: '5:20' },
          { id: 'feat-43', title: 'Tony Montana', album: 'Single', role: 'featured', year: 2011, isBlocked: true, collaborators: ['Future'], duration: '4:40' },
          { id: 'feat-44', title: 'Fuckin\' Problems', album: 'Long. Live. ASAP', role: 'featured', year: 2012, isBlocked: true, collaborators: ['A$AP Rocky', '2 Chainz', 'Kendrick Lamar'], duration: '4:03' },
          { id: 'feat-45', title: 'The Motto', album: 'Single', role: 'featured', year: 2011, isBlocked: true, collaborators: ['Lil Wayne', 'Tyga'], duration: '4:31' },

          // ========================================
          // PRODUCER CREDITS (20+)
          // ========================================
          { id: 'prod-1', title: 'Come Thru', album: 'PartyNextDoor Two', role: 'producer', year: 2014, isBlocked: false, collaborators: ['PartyNextDoor'], duration: '4:24' },
          { id: 'prod-2', title: 'Wednesday Night Interlude', album: 'Views', role: 'producer', year: 2016, isBlocked: true, collaborators: ['PartyNextDoor'], duration: '3:47' },
          { id: 'prod-3', title: 'Own It', album: 'Thank Me Later', role: 'producer', year: 2010, isBlocked: true, duration: '3:54' },
          { id: 'prod-4', title: 'Over My Dead Body', album: 'Take Care', role: 'producer', year: 2011, isBlocked: true, duration: '4:46' },
          { id: 'prod-5', title: 'Cameras / Good Ones Go', album: 'Take Care', role: 'producer', year: 2011, isBlocked: true, duration: '6:06' },
          { id: 'prod-6', title: 'Doing It Wrong', album: 'Take Care', role: 'producer', year: 2011, isBlocked: true, collaborators: ['Stevie Wonder'], duration: '4:17' },
          { id: 'prod-7', title: 'Practice', album: 'Take Care', role: 'producer', year: 2011, isBlocked: true, duration: '2:55' },
          { id: 'prod-8', title: 'Shot for Me', album: 'Take Care', role: 'producer', year: 2011, isBlocked: true, duration: '4:22' },
          { id: 'prod-9', title: 'Tuscan Leather', album: 'Nothing Was the Same', role: 'producer', year: 2013, isBlocked: true, duration: '6:06' },
          { id: 'prod-10', title: 'Wu-Tang Forever', album: 'Nothing Was the Same', role: 'producer', year: 2013, isBlocked: true, duration: '4:35' },
          { id: 'prod-11', title: 'Jungle', album: "If You're Reading This It's Too Late", role: 'producer', year: 2015, isBlocked: true, duration: '5:49' },
          { id: 'prod-12', title: 'Preach', album: "If You're Reading This It's Too Late", role: 'producer', year: 2015, isBlocked: true, collaborators: ['PARTYNEXTDOOR'], duration: '3:28' },
          { id: 'prod-13', title: 'U With Me?', album: 'Views', role: 'producer', year: 2016, isBlocked: true, duration: '6:06' },
          { id: 'prod-14', title: '9', album: 'Views', role: 'producer', year: 2016, isBlocked: true, duration: '4:04' },
          { id: 'prod-15', title: 'Recognize', album: 'PARTYNEXTDOOR', role: 'producer', year: 2013, isBlocked: false, collaborators: ['PARTYNEXTDOOR'], duration: '4:17' },

          // ========================================
          // WRITER CREDITS (25+)
          // ========================================
          { id: 'write-1', title: 'Drunk in Love (Remix)', album: 'Beyoncé', role: 'writer', year: 2013, isBlocked: false, collaborators: ['Beyoncé'], duration: '5:23' },
          { id: 'write-2', title: 'R.I.C.O.', album: 'Barter 6', role: 'writer', year: 2015, isBlocked: false, collaborators: ['Meek Mill', 'Young Thug'], duration: '5:23' },
          { id: 'write-3', title: 'R.I.P.', album: 'ORA', role: 'writer', year: 2012, isBlocked: false, collaborators: ['Rita Ora'], duration: '3:58' },
          { id: 'write-4', title: 'Fall for Your Type', album: 'Best Night of My Life', role: 'writer', year: 2010, isBlocked: false, collaborators: ['Jamie Foxx'], duration: '3:45' },
          { id: 'write-5', title: 'Unthinkable (I\'m Ready)', album: 'The Element of Freedom', role: 'writer', year: 2010, isBlocked: false, collaborators: ['Alicia Keys'], duration: '4:08' },
          { id: 'write-6', title: 'Invented Sex', album: 'Ready', role: 'writer', year: 2009, isBlocked: false, collaborators: ['Trey Songz'], duration: '4:26' },
          { id: 'write-7', title: 'Mr. Wrong', album: 'My Life II', role: 'writer', year: 2011, isBlocked: false, collaborators: ['Mary J. Blige'], duration: '4:21' },
          { id: 'write-8', title: 'Moment 4 Life', album: 'Pink Friday', role: 'writer', year: 2010, isBlocked: false, collaborators: ['Nicki Minaj'], duration: '4:39' },
          { id: 'write-9', title: '2 On', album: 'Aquarius', role: 'writer', year: 2014, isBlocked: false, collaborators: ['Tinashe', 'ScHoolboy Q'], duration: '3:28' },
          { id: 'write-10', title: 'Recognize', album: 'PARTYNEXTDOOR', role: 'writer', year: 2013, isBlocked: false, collaborators: ['PARTYNEXTDOOR'], duration: '4:17' },
          { id: 'write-11', title: 'Timmy\'s Prayer', album: 'Process', role: 'writer', year: 2017, isBlocked: false, collaborators: ['Sampha'], duration: '4:51' },
          { id: 'write-12', title: 'Her', album: 'Majid Jordan', role: 'writer', year: 2016, isBlocked: false, collaborators: ['Majid Jordan'], duration: '5:01' },
          { id: 'write-13', title: 'Hallucinations', album: 'Sept. 5th', role: 'writer', year: 2016, isBlocked: false, collaborators: ['dvsn'], duration: '4:36' },
          { id: 'write-14', title: 'Drama', album: 'Say Less', role: 'writer', year: 2017, isBlocked: false, collaborators: ['Roy Woods'], duration: '3:33' },
          { id: 'write-15', title: 'Own It', album: 'barter 6', role: 'writer', year: 2013, isBlocked: false, collaborators: ['Young Thug'], duration: '3:22' },
          { id: 'write-16', title: 'You Could Be', album: 'What For?', role: 'writer', year: 2015, isBlocked: false, collaborators: ['Toro y Moi'], duration: '3:22' },
          { id: 'write-17', title: 'Come and See Me', album: 'PARTYNEXTDOOR 3', role: 'writer', year: 2016, isBlocked: false, collaborators: ['PARTYNEXTDOOR'], duration: '4:53' },
          { id: 'write-18', title: 'My House', album: 'My House', role: 'writer', year: 2018, isBlocked: false, collaborators: ['Beyoncé'], duration: '4:15' },
          { id: 'write-19', title: 'Never Recover', album: 'Drip Harder', role: 'writer', year: 2018, isBlocked: false, collaborators: ['Lil Baby', 'Gunna'], duration: '3:36' },
          { id: 'write-20', title: 'Love Galore', album: 'Ctrl', role: 'writer', year: 2017, isBlocked: false, collaborators: ['SZA', 'Travis Scott'], duration: '4:38' },
        ];
      }

      // Derive connections from credits when the collaborations API returns empty
      if (profile && profile.collaborators.length === 0 && profile.credits) {
        const creditsCollabs: typeof profile.collaborators = [];
        const seen = new Set<string>();

        for (const writer of profile.credits.writers || []) {
          if (!seen.has(writer.id)) {
            seen.add(writer.id);
            creditsCollabs.push({
              id: writer.id,
              name: writer.name,
              image_url: writer.image_url || undefined,
              collaboration_count: writer.track_count,
              is_flagged: writer.is_flagged,
              status: writer.is_flagged ? 'flagged' : 'clean',
              collaboration_type: 'writer',
              recent_tracks: [],
            });
          }
        }

        for (const producer of profile.credits.producers || []) {
          if (!seen.has(producer.id)) {
            seen.add(producer.id);
            creditsCollabs.push({
              id: producer.id,
              name: producer.name,
              image_url: producer.image_url || undefined,
              collaboration_count: producer.track_count,
              is_flagged: producer.is_flagged,
              status: producer.is_flagged ? 'flagged' : 'clean',
              collaboration_type: 'producer',
              recent_tracks: [],
            });
          } else {
            // Already added as writer — bump count
            const existing = creditsCollabs.find(c => c.id === producer.id);
            if (existing) {
              existing.collaboration_count += producer.track_count;
            }
          }
        }

        // Also extract unique collaborators from catalog tracks
        const catalogCollabs = new Map<string, { name: string; count: number }>();
        for (const track of catalog) {
          for (const name of track.collaborators || []) {
            const entry = catalogCollabs.get(name);
            if (entry) {
              entry.count++;
            } else {
              catalogCollabs.set(name, { name, count: 1 });
            }
          }
        }
        for (const [name, entry] of catalogCollabs) {
          if (!creditsCollabs.some(c => c.name === name)) {
            creditsCollabs.push({
              id: `catalog-${name.toLowerCase().replace(/\s+/g, '-')}`,
              name: entry.name,
              collaboration_count: entry.count,
              is_flagged: false,
              status: 'clean',
              collaboration_type: 'featured',
              recent_tracks: [],
            });
          }
        }

        creditsCollabs.sort((a, b) => b.collaboration_count - a.collaboration_count);
        profile.collaborators = creditsCollabs;
      }

    } catch (e: any) {
      error = e.message || 'Failed to load artist';
    } finally {
      isLoading = false;
      // Validate evidence links in the background
      runLinkValidation();
    }
  }

  async function loadDnpStatus() {
    try {
      const result = await apiClient.get<unknown>('/api/v1/dnp/list');
      if (result.success && result.data) {
        dnpList = new Set(normalizeDnpArtistIds(result.data));
        isBlocked = dnpList.has(artistId);
      }
    } catch (e) {
      console.error('Failed to load DNP status:', e);
    }
  }

  // Drake's Spotify image URL for showcase
  const DRAKE_IMAGE = 'https://i.scdn.co/image/ab6761610000e5eb4293385d324db8558179afd9';

  function transformToProfile(data: any): ArtistProfile {
    // Determine status based on offenses
    const hasOffenses = data.offenses && data.offenses.length > 0;
    const hasConvictions = data.offenses?.some((o: any) =>
      o.status === 'convicted' || o.procedural_state === 'convicted'
    );

    let status: ArtistStatus = 'clean';
    if (hasConvictions) status = 'flagged';
    else if (hasOffenses) status = 'certified_creeper';

    // Determine confidence based on evidence quality
    let confidence: ConfidenceLevel = 'low';
    if (data.offenses?.length > 0) {
      const tierACount = data.offenses.reduce((count: number, o: any) => {
        return count + (o.evidence?.filter((e: any) =>
          e.source_type === 'court_record' || e.tier === 'tier_a'
        ).length || 0);
      }, 0);

      if (tierACount >= 2) confidence = 'high';
      else if (tierACount >= 1) confidence = 'medium';
    }

    // Extract image from metadata or use fallback for Drake
    let imageUrl = data.image_url || data.metadata?.image || data.primary_image?.url;
    const artistName = data.canonical_name || data.name || 'Unknown Artist';
    if (!imageUrl && artistName === 'Drake') {
      imageUrl = DRAKE_IMAGE;
    }

    return {
      id: data.id || artistId,
      canonical_name: artistName,
      aliases: data.aliases || [],
      external_ids: data.external_ids || {},
      status,
      confidence,
      images: data.images || [],
      primary_image: imageUrl ? { url: imageUrl, source: 'Spotify' } : data.primary_image || data.images?.[0],
      genres: data.genres || [],
      offenses: (data.offenses || []).map((o: any) => transformOffense(o)),
      streaming_metrics: data.streaming_metrics,
      collaborators: data.collaborators || [],
      label: data.label,
      last_reviewed: data.last_reviewed || data.updated_at,
      created_at: data.created_at || new Date().toISOString(),
      updated_at: data.updated_at || new Date().toISOString(),
    };
  }

  function transformOffense(data: any): Offense {
    return {
      id: data.id,
      artist_id: artistId,
      category: {
        id: data.category || 'unknown',
        name: getCategoryColor(data.category).name,
        description: '',
        color: getCategoryColor(data.category).icon,
        icon: 'alert',
      },
      tags: data.tags || [],
      title: data.title,
      description: data.description,
      incident_date: data.incident_date,
      procedural_state: data.procedural_state || data.status || 'alleged',
      evidence: (data.evidence || []).map((e: any) => ({
        id: e._id || e.id,
        offense_id: data.id,
        source: {
          id: e._id || e.id,
          url: e.url || e.source_url,
          title: e.title || e.sourceName || e.source_name,
          source_name: e.sourceName || e.source_name,
          source_type: e.sourceType || e.source_type || 'news',
          tier: determineSourceTier(e),
          published_date: e.publishedDate || e.published_date,
          excerpt: e.excerpt,
          archived_url: e.archivedUrl || e.archived_url,
          credibility_score: e.credibilityScore || e.credibility_score,
        },
        date_added: e.createdAt || e.date_added || new Date().toISOString(),
        verified: e.verified || false,
      })),
      evidence_strength: determineEvidenceStrength(data.evidence || []),
      last_updated: data.updated_at || new Date().toISOString(),
      created_at: data.created_at || new Date().toISOString(),
    };
  }

  function determineSourceTier(evidence: any): SourceTier {
    if (evidence.source_type === 'court_record' || evidence.tier === 'tier_a') return 'tier_a';
    if (evidence.source_type === 'news' || evidence.tier === 'tier_b') return 'tier_b';
    if (evidence.tier === 'tier_c') return 'tier_c';
    return 'tier_d';
  }

  function determineEvidenceStrength(evidence: any[]): 'strong' | 'moderate' | 'weak' {
    if (!evidence?.length) return 'weak';
    const tierACount = evidence.filter(e =>
      e.source_type === 'court_record' || e.tier === 'tier_a'
    ).length;
    if (tierACount >= 2) return 'strong';
    if (tierACount >= 1 || evidence.length >= 3) return 'moderate';
    return 'weak';
  }

  async function toggleBlock() {
    if (!profile) return;

    isBlockingInProgress = true;
    try {
      if (isBlocked) {
        await apiClient.delete(`/api/v1/dnp/list/${artistId}`);
        isBlocked = false;
        dnpList.delete(artistId);
      } else {
        await apiClient.post('/api/v1/dnp/list', { artist_id: artistId });
        isBlocked = true;
        dnpList.add(artistId);
      }
    } catch (e) {
      console.error('Failed to toggle block:', e);
    } finally {
      isBlockingInProgress = false;
    }
  }

  async function submitReport() {
    if (!reportDescription.trim()) return;

    try {
      await apiClient.post('/api/v1/offenses/report-error', {
        artist_id: artistId,
        description: reportDescription,
        category: reportCategory,
      });
      showReportModal = false;
      reportDescription = '';
    } catch (e) {
      console.error('Failed to submit report:', e);
    }
  }

  function formatDate(dateString?: string): string {
    if (!dateString) return 'Unknown';
    const date = new Date(dateString);
    return date.toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric'
    });
  }

  function formatNumber(num?: number): string {
    if (!num) return '0';
    if (num >= 1_000_000_000) return `${(num / 1_000_000_000).toFixed(1)}B`;
    if (num >= 1_000_000) return `${(num / 1_000_000).toFixed(1)}M`;
    if (num >= 1_000) return `${(num / 1_000).toFixed(1)}K`;
    return num.toString();
  }

  $: statusColor = profile ? getStatusColor(profile.status) : null;
  $: statusLabel = profile ? getStatusLabel(profile.status) : '';
  $: confidenceLabel = profile ? getConfidenceLabel(profile.confidence) : '';
</script>

<div class="profile brand-page surface-page">
  {#if isLoading}
    <div class="profile__loading">
      <div class="profile__loading-inner surface-panel-thin">
        <div class="profile__spinner"></div>
        <p class="profile__loading-text">Loading artist profile...</p>
      </div>
    </div>
  {:else if error}
    <div class="profile__loading">
      <div class="profile__error-card">
        <div class="profile__error-bang">!</div>
        <h2 class="profile__error-title">Error Loading Profile</h2>
        <p class="profile__error-msg">{error}</p>
        <button type="button"
          on:click={() => navigateTo('home')}
          class="brand-button brand-button--danger profile__error-btn"
        >
          Go Back
        </button>
      </div>
    </div>
  {:else if profile}
    <!-- Clean Header Section -->
    <section class="profile__header-section brand-hero">
      <div class="profile__container">
        <button
          type="button"
          on:click={() => navigateTo('home')}
          class="brand-back profile__back-link"
        >
          <svg class="brand-back__icon profile__back-icon" width="20" height="20" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
          </svg>
          Back to Home
        </button>

        <div class="profile__header-row">
          <!-- Artist Photo -->
          <div class="profile__photo-wrap">
            <div
              class="profile__photo"
              style="border-color: {statusColor?.border || 'var(--color-border-default)'}; overflow: hidden; position: relative;"
            >
              {#if profile.primary_image?.url}
                <img
                  src={profile.primary_image.url}
                  alt=""
                  class="profile__photo-img"
                  style="position: absolute; inset: 0;"
                  on:error={hideImgOnError}
                />
              {/if}
              <div class="profile__photo-placeholder">
                {profile.canonical_name.charAt(0)}
              </div>
            </div>
          </div>

          <!-- Artist Info -->
          <div class="profile__info">
            <div class="brand-kickers profile__kickers">
              <span class="brand-kicker">Artist Case File</span>
              <span class="brand-kicker brand-kicker--accent">
                {statusLabel} {profile.offenses.length > 0 ? `· ${profile.offenses.length} incidents` : '· No incidents'}
              </span>
            </div>

            <div class="profile__badges">
              <!-- Status Badge -->
              <span
                class="profile__status-badge"
                style="background: {statusColor?.bg}; color: {statusColor?.text};"
              >
                {statusLabel}
              </span>

              <!-- Confidence Level (only show for medium/high) -->
              {#if profile.confidence !== 'low'}
                <div class="profile__confidence">
                  <div class="profile__confidence-bars">
                    {#each [0, 1, 2] as i}
                      <div
                        class="profile__confidence-bar"
                        style="background: {
                          (profile.confidence === 'high') ||
                          (profile.confidence === 'medium' && i < 2)
                            ? 'var(--color-success)' : 'var(--color-bg-hover)'
                        };"
                      ></div>
                    {/each}
                  </div>
                  <span class="profile__confidence-label">{confidenceLabel}</span>
                </div>
              {/if}
            </div>

            <h1 class="profile__name brand-title brand-title--compact">{profile.canonical_name}</h1>

            {#if profile.genres.length > 0}
              <p class="profile__genres brand-subtitle">{profile.genres.join(' • ')}</p>
            {:else}
              <p class="profile__genres brand-subtitle">
                Evidence summary, catalog exposure, and blocking controls in one view.
              </p>
            {/if}

            <div class="brand-meta profile__meta-row">
              {#if profile.last_reviewed}
                <span class="brand-meta__item">Last reviewed {formatDate(profile.last_reviewed)}</span>
              {/if}
              <span class="brand-meta__item">Confidence rating: {confidenceLabel}</span>
              <span class="brand-meta__item">{profile.offenses.length} documented incidents</span>
            </div>
          </div>

            <!-- Action Buttons -->
            <div class="profile__actions">
              <!-- Primary Block Button with Dropdown -->
              <div class="profile__block-group">
                <div class="profile__block-split">
                  <button
                    type="button"
                    on:click={toggleBlock}
                    disabled={isBlockingInProgress}
                    class="profile__block-btn {isBlocked ? 'profile__block-btn--blocked' : 'profile__block-btn--unblocked'}"
                  >
                    {#if isBlockingInProgress}
                      <div class="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin"></div>
                    {:else if isBlocked}
                      <svg class="w-4 h-4" width="16" height="16" fill="currentColor" viewBox="0 0 24 24">
                        <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
                      </svg>
                      Blocked
                    {:else}
                      <svg class="w-4 h-4" width="16" height="16" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
                      </svg>
                      Block
                    {/if}
                  </button>
                  <button
                    type="button"
                    on:click={() => showBlockingOptions = !showBlockingOptions}
                    class="profile__block-dropdown {isBlocked ? 'profile__block-dropdown--blocked' : 'profile__block-dropdown--unblocked'}"
                  >
                    <svg class="w-3 h-3 transition-transform {showBlockingOptions ? 'rotate-180' : ''}" width="12" height="12" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M19 9l-7 7-7-7" />
                    </svg>
                  </button>
                </div>

                <!-- Blocking Options Dropdown -->
                {#if showBlockingOptions}
                  <div class="profile__dropdown-menu">
                    <div class="profile__dropdown-inner">
                      <div class="profile__dropdown-label">Blocking Options</div>
                      <button
                        type="button"
                        on:click={() => { toggleRoleBlocking('main', true); showBlockingOptions = false; }}
                        class="profile__dropdown-item"
                      >
                        <svg class="profile__dropdown-icon profile__dropdown-icon--rose" width="16" height="16" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3" />
                        </svg>
                        Block All Main Tracks
                      </button>
                      <button
                        type="button"
                        on:click={() => { toggleRoleBlocking('featured', true); showBlockingOptions = false; }}
                        class="profile__dropdown-item"
                      >
                        <svg class="profile__dropdown-icon profile__dropdown-icon--orange" width="16" height="16" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                        </svg>
                        Block Collaborations
                      </button>
                      <button
                        type="button"
                        on:click={() => { toggleRoleBlocking('producer', true); showBlockingOptions = false; }}
                        class="profile__dropdown-item"
                      >
                        <svg class="profile__dropdown-icon profile__dropdown-icon--purple" width="16" height="16" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z" />
                        </svg>
                        Block Producer Credits
                      </button>
                      <button
                        type="button"
                        on:click={() => { toggleRoleBlocking('writer', true); showBlockingOptions = false; }}
                        class="profile__dropdown-item"
                      >
                        <svg class="profile__dropdown-icon profile__dropdown-icon--blue" width="16" height="16" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
                        </svg>
                        Block Writer Credits
                      </button>
                      <div class="profile__dropdown-divider"></div>
                      <button
                        type="button"
                        on:click={() => {
                          toggleRoleBlocking('main', true);
                          toggleRoleBlocking('featured', true);
                          toggleRoleBlocking('producer', true);
                          toggleRoleBlocking('writer', true);
                          showBlockingOptions = false;
                        }}
                        class="profile__dropdown-item profile__dropdown-item--danger"
                      >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
                        </svg>
                        Block Everything
                      </button>
                    </div>
                  </div>
                {/if}
              </div>

              <!-- Secondary Actions -->
              <div class="profile__secondary-actions">
                <button
                  type="button"
                  on:click={() => activeTab = 'evidence'}
                  class="profile__action-btn profile__action-btn--primary"
                >
                  <svg class="profile__action-icon" width="14" height="14" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                  </svg>
                  Evidence
                </button>
                <button
                  type="button"
                  on:click={() => showReportModal = true}
                  class="profile__action-btn profile__action-btn--secondary"
                >
                  <svg class="profile__action-icon" width="14" height="14" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                  </svg>
                  Report
                </button>
              </div>
          </div>
        </div>
      </div>
    </section>

    <div class="profile__content-card">
    <!-- Tab Navigation -->
    <nav class="profile__tab-bar" aria-label="Artist profile sections">
      {#each profileTabs as tab}
        <button
          type="button"
          on:click={() => setTab(tab.key)}
          class="profile__tab"
          class:profile__tab--active={activeTab === tab.key}
          role="tab"
          aria-selected={activeTab === tab.key}
        >
          {tab.label}
        </button>
      {/each}
    </nav>

    <!-- Main Content -->
    <main class="profile__main" role="tabpanel">
      {#if activeTab === 'evidence'}
        <!-- Evidence Timeline — Full-width feed -->
        <div class="ev-header">
          <h2 class="ev-title">
            Evidence Timeline
            {#if profile.offenses.length > 0}
              <span class="ev-count">({profile.offenses.length})</span>
            {/if}
          </h2>
        </div>

        {#if profile.offenses.length === 0}
          <div class="ev-empty surface-panel-thin">
            <p class="text-zinc-400 text-sm">No documented incidents found for this artist.</p>
          </div>
        {:else}
          <div class="ev-feed surface-panel-thin">
            {#each profile.offenses.sort((a, b) => new Date(b.incident_date || b.created_at).getTime() - new Date(a.incident_date || a.created_at).getTime()) as offense, index}
              {@const catColor = getCategoryColor(offense.category.id)}
              {@const evidenceStrength = getEvidenceStrengthLabel(offense.evidence_strength)}
              {@const isExpanded = expandedOffenseId === offense.id}

              <div class="ev-card {index > 0 ? 'ev-card--bordered' : ''}">
                <button
                  type="button"
                  on:click={() => expandedOffenseId = isExpanded ? null : offense.id}
                  class="ev-card__btn"
                >
                  <div class="ev-card__badges">
                    <span class="ev-pill" style="background: {hexToRgba(catColor.icon, 0.18)}; color: {catColor.icon};">{offense.category.name}</span>
                    <span class="ev-pill ev-pill--muted">{getProceduralStateLabel(offense.procedural_state)}</span>
                    <span class="ev-pill" style="background: {hexToRgba(evidenceStrength.color, 0.22)}; color: {evidenceStrength.color};">{evidenceStrength.label} Evidence</span>
                    {#if offense.incident_date}
                      <span class="ev-card__date">{formatDate(offense.incident_date)}</span>
                    {/if}
                  </div>

                  <h3 class="ev-card__title">{offense.title}</h3>
                  <p class="ev-card__desc">{offense.description}</p>

                  <div class="ev-card__footer">
                    <span class="ev-card__sources">{offense.evidence.length} source{offense.evidence.length !== 1 ? 's' : ''}</span>
                    <svg class="ev-card__chevron {isExpanded ? 'ev-card__chevron--open' : ''}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                    </svg>
                  </div>
                </button>

                {#if isExpanded}
                  <div class="ev-sources" transition:slide={{ duration: slideDuration }}>
                    <h4 class="ev-sources__heading">Sources ({offense.evidence.length})</h4>
                    {#if offense.evidence.length === 0}
                      <p class="ev-sources__empty">No sources available</p>
                    {:else}
                      {#each offense.evidence as evidence}
                        {@const tierInfo = getSourceTierLabel(evidence.source.tier)}
                        {@const linkCheck = linkStatuses.get(evidence.source.url)}
                        {@const resolvedUrl = linkCheck?.resolvedUrl || evidence.source.archived_url || evidence.source.url}
                        {@const linkBroken = linkCheck?.status === 'broken'}
                        {@const linkArchived = linkCheck?.status === 'archived'}
                        <a
                          href={resolvedUrl}
                          target="_blank"
                          rel="noopener noreferrer"
                          class="ev-source {linkBroken ? 'ev-source--broken' : ''} {linkArchived ? 'ev-source--archived' : ''}"
                          title={linkArchived ? 'Original link unavailable — viewing archived version via Wayback Machine' : linkBroken ? 'This link appears to be broken — no archived version found' : ''}
                        >
                          <div class="ev-source__tier" style="background: {hexToRgba(tierInfo.color, 0.2)}; color: {tierInfo.color};">
                            {tierInfo.label.replace('Tier ', '')}
                          </div>
                          <div class="ev-source__info">
                            <p class="ev-source__title">
                              {evidence.source.title || evidence.source.source_name}
                              {#if linkArchived}
                                <span class="ev-source__badge ev-source__badge--archived" title="Archived via Wayback Machine">archived</span>
                              {:else if linkBroken}
                                <span class="ev-source__badge ev-source__badge--broken" title="Link broken, no archive found">broken link</span>
                              {:else if linksValidating}
                                <span class="ev-source__badge ev-source__badge--checking">checking...</span>
                              {/if}
                            </p>
                            <p class="ev-source__meta">
                              {evidence.source.source_name}{#if evidence.source.published_date} · {formatDate(evidence.source.published_date)}{/if}
                              {#if linkArchived} · <span style="color: #F59E0B;">Wayback Machine</span>{/if}
                            </p>
                            {#if evidence.source.excerpt}
                              <p class="ev-source__excerpt">"{evidence.source.excerpt}"</p>
                            {/if}
                          </div>
                        </a>
                      {/each}
                    {/if}
                  </div>
                {/if}
              </div>
            {/each}
          </div>

          <!-- Summary Strip -->
          <div class="ev-summary">
            <div class="ev-summary__cell">
              <span class="ev-summary__value">{profile.offenses.length}</span>
              <span class="ev-summary__label">Total Incidents</span>
            </div>
            <div class="ev-summary__cell">
              <span class="ev-summary__value">{profile.offenses.reduce((count, o) => count + o.evidence.filter(e => e.source.tier === 'tier_a').length, 0)}</span>
              <span class="ev-summary__label">Primary Sources (Tier A)</span>
            </div>
            <div class="ev-summary__cell">
              <span class="ev-summary__value" style="color: {statusColor?.text};">{confidenceLabel}</span>
              <span class="ev-summary__label">Confidence Level</span>
            </div>
          </div>
        {/if}

      {:else if activeTab === 'catalog'}
        <div>
          <!-- Filter + Stats -->
          <div class="cat-header">
            <div class="brand-segmented">
              <button type="button" on:click={() => catalogFilter = 'all'} class="brand-segmented__item" class:brand-segmented__item--active={catalogFilter === 'all'}>All</button>
              <button type="button" on:click={() => catalogFilter = 'blocked'} class="brand-segmented__item" class:brand-segmented__item--active={catalogFilter === 'blocked'}>Blocked</button>
              <button type="button" on:click={() => catalogFilter = 'unblocked'} class="brand-segmented__item" class:brand-segmented__item--active={catalogFilter === 'unblocked'}>Allowed</button>
            </div>
            <div class="cat-indicator">
              <div class="cat-indicator__bar">
                <div class="cat-indicator__fill" style="width: {catalog.length > 0 ? (catalog.filter(t => t.isBlocked).length / catalog.length * 100) : 0}%;"></div>
              </div>
              <span class="cat-indicator__text" aria-live="polite"><span class="cat-indicator__count">{catalog.filter(t => t.isBlocked).length}</span> / {catalog.length} blocked</span>
            </div>
          </div>

          <!-- Sub-tabs -->
          <div class="flex border-b border-white/[0.06] mb-5">
            <button type="button" on:click={() => catalogSubTab = 'main'} class="catalog-subtab" class:catalog-subtab--active={catalogSubTab === 'main'}>
              Main Artist <span class="text-zinc-400 ml-1">{catalogMainCount}</span>
            </button>
            <button type="button" on:click={() => catalogSubTab = 'featured'} class="catalog-subtab" class:catalog-subtab--active={catalogSubTab === 'featured'}>
              Featured <span class="text-zinc-400 ml-1">{catalogFeaturedCount}</span>
            </button>
            <button type="button" on:click={() => catalogSubTab = 'behind'} class="catalog-subtab" class:catalog-subtab--active={catalogSubTab === 'behind'}>
              Writing & Production <span class="text-zinc-400 ml-1">{catalogBehindCount}</span>
            </button>
          </div>

          <!-- Main Artist Albums -->
          {#if catalogSubTab === 'main'}
            {#if catalogAlbums.length > 0}
              <div class="cat-albums surface-panel-thin">
                {#each catalogAlbums as album, albumIdx}
                  <div class="cat-album {albumIdx > 0 ? 'cat-album--bordered' : ''}">
                    <button
                      type="button"
                      class="cat-album__header"
                      on:click={() => toggleCatalogAlbum(album.name)}
                    >
                      <div class="cat-album__art">
                        {#if album.cover && !album.cover.includes('data:image')}
                          <img src={album.cover} alt="" class="cat-album__art-img" on:error={hideImgOnError} />
                        {/if}
                        <div class="cat-album__art-ph">
                          <svg fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3" /></svg>
                        </div>
                      </div>
                      <div class="cat-album__info">
                        <div class="cat-album__title">{album.name}</div>
                        <div class="cat-album__meta">{album.year} · {album.totalCount} tracks</div>
                      </div>
                      {#if album.blockedCount === album.totalCount && album.totalCount > 0}
                        <span class="cat-album__status cat-album__status--all">All blocked</span>
                      {:else if album.blockedCount > 0}
                        <span class="cat-album__status">{album.blockedCount}/{album.totalCount}</span>
                      {/if}
                      <svg class="cat-chevron {expandedCatalogAlbums.has(album.name) ? 'cat-chevron--open' : ''}" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" /></svg>
                    </button>

                    {#if expandedCatalogAlbums.has(album.name)}
                      <div class="cat-tracks" transition:slide={{ duration: slideDuration }}>
                        <div class="cat-batch">
                          <span class="cat-batch__count">{album.blockedCount} of {album.totalCount} blocked</span>
                          <button type="button" on:click|stopPropagation={() => toggleAlbumBlocking(album.name, album.blockedCount < album.totalCount)} class="cat-batch__btn {album.blockedCount === album.totalCount ? '' : 'cat-batch__btn--danger'}">
                            {album.blockedCount === album.totalCount ? 'Allow all' : 'Block all'}
                          </button>
                        </div>
                        {#each album.tracks as track, idx}
                          <div class="cat-track">
                            <button type="button" on:click|stopPropagation={() => toggleTrackBlock(track.id)} class="cat-toggle {track.isBlocked ? 'cat-toggle--active' : ''}" title={track.isBlocked ? 'Allow this track' : 'Block this track'} aria-label={track.isBlocked ? `Allow ${track.title}` : `Block ${track.title}`}>
                              <span class="cat-toggle__thumb"></span>
                            </button>
                            <span class="cat-track__num">{idx + 1}</span>
                            <div class="cat-track__info">
                              <span class="cat-track__title {track.isBlocked ? 'cat-track__title--blocked' : ''}">{track.title}</span>
                              {#if track.collaborators}<span class="cat-track__feat">feat. {track.collaborators.join(', ')}</span>{/if}
                            </div>
                            <span class="cat-track__duration">{track.duration || ''}</span>
                          </div>
                        {/each}
                      </div>
                    {/if}
                  </div>
                {/each}
              </div>
            {:else}
              <div class="cat-empty">No main artist tracks found.</div>
            {/if}

          {:else if catalogSubTab === 'featured'}
            {@const featuredTracks = filteredCatalog.filter(t => t.role === 'featured')}
            {#if featuredTracks.length > 0}
              <div class="cat-list surface-panel-thin">
                {#each featuredTracks.slice(0, featuredShowCount) as track, fi}
                  <div class="cat-list-row {fi > 0 ? 'cat-list-row--bordered' : ''}">
                    <button type="button" on:click|stopPropagation={() => toggleTrackBlock(track.id)} class="cat-toggle {track.isBlocked ? 'cat-toggle--active' : ''}" title={track.isBlocked ? 'Allow' : 'Block'} aria-label={track.isBlocked ? `Allow ${track.title}` : `Block ${track.title}`}>
                      <span class="cat-toggle__thumb"></span>
                    </button>
                    <div class="cat-list-row__info">
                      <div class="cat-list-row__title {track.isBlocked ? 'cat-track__title--blocked' : ''}">{track.title}</div>
                      <div class="cat-list-row__meta">{track.collaborators?.join(', ') || '—'}{#if track.album} · {track.album}{/if}</div>
                    </div>
                    <span class="cat-list-row__year">{track.year || ''}</span>
                  </div>
                {/each}
              </div>
              {#if featuredTracks.length > featuredShowCount}
                <button type="button" on:click={() => featuredShowCount += 20} class="profile__show-more">
                  Show more ({featuredTracks.length - featuredShowCount} remaining)
                </button>
              {/if}
            {:else}
              <div class="cat-empty surface-panel-thin"><p class="text-zinc-500 text-sm">No featured appearances found.</p></div>
            {/if}

          {:else if catalogSubTab === 'behind'}
            {@const behindTracks = filteredCatalog.filter(t => t.role === 'producer' || t.role === 'writer')}
            {#if behindTracks.length > 0}
              <div class="cat-list surface-panel-thin">
                {#each behindTracks.slice(0, behindShowCount) as track, bi}
                  <div class="cat-list-row {bi > 0 ? 'cat-list-row--bordered' : ''}">
                    <button type="button" on:click|stopPropagation={() => toggleTrackBlock(track.id)} class="cat-toggle {track.isBlocked ? 'cat-toggle--active' : ''}" title={track.isBlocked ? 'Allow' : 'Block'} aria-label={track.isBlocked ? `Allow ${track.title}` : `Block ${track.title}`}>
                      <span class="cat-toggle__thumb"></span>
                    </button>
                    <div class="cat-list-row__info">
                      <div class="cat-list-row__title {track.isBlocked ? 'cat-track__title--blocked' : ''}">{track.title}</div>
                      <div class="cat-list-row__meta">{track.collaborators?.join(', ') || '—'}</div>
                    </div>
                    <span class="cat-list-row__role">{track.role}</span>
                    <span class="cat-list-row__year">{track.year || ''}</span>
                  </div>
                {/each}
              </div>
              {#if behindTracks.length > behindShowCount}
                <button type="button" on:click={() => behindShowCount += 20} class="profile__show-more">
                  Show more ({behindTracks.length - behindShowCount} remaining)
                </button>
              {/if}
            {:else}
              <div class="cat-empty surface-panel-thin"><p class="text-zinc-500 text-sm">No writing or production credits found.</p></div>
            {/if}
          {/if}

          <p class="cat-disclaimer">
            Catalog data aggregated from Spotify, Apple Music, and MusicBrainz. Some entries may be incomplete.
          </p>
        </div>

      {:else if activeTab === 'discography'}
        <!-- Discography Impact Panel -->
        <div class="space-y-8">
          <div data-testid="discography-revenue-section">
            <ArtistDiscographyRevenue
              artistId={artistId}
              artistName={profile.canonical_name}
            />
          </div>

          {#if (profile.streaming_metrics?.platform_breakdown?.length ?? 0) > 0}
            <div class="rounded-xl p-5 surface-panel-thin">
              <h3 class="text-lg font-semibold text-zinc-100 mb-6">Platform Distribution</h3>
              <div class="space-y-5">
                {#each profile.streaming_metrics?.platform_breakdown ?? [] as platform}
                  <div>
                    <div class="flex justify-between text-sm mb-2">
                      <span class="text-zinc-200 capitalize font-medium">{platform.platform}</span>
                      <span class="text-zinc-300">{formatNumber(platform.streams)} ({platform.percentage}%)</span>
                    </div>
                    <div class="h-3 rounded-full overflow-hidden" style="background: var(--color-bg-inset);">
                      <div
                        class="h-full rounded-full transition-all"
                        style="width: {platform.percentage}%; background: {
                          platform.platform === 'spotify' ? '#1DB954' :
                          platform.platform === 'apple' ? '#FC3C44' :
                          platform.platform === 'youtube' ? '#FF0000' :
                          '#f43f5e'
                        };"
                      ></div>
                    </div>
                  </div>
                {/each}
              </div>
            </div>
          {/if}

          {#if (profile.streaming_metrics?.top_tracks?.length ?? 0) > 0}
            <div class="rounded-xl p-5 surface-panel-thin">
              <h3 class="text-lg font-semibold text-zinc-100 mb-6">Top Tracks</h3>
              <div class="space-y-3">
                {#each (profile.streaming_metrics?.top_tracks ?? []).slice(0, 5) as track, index}
                  <div class="flex items-center gap-4 p-4 rounded-xl hover:bg-white/[0.04] transition-colors surface-panel-thin">
                    <span class="text-2xl font-bold text-zinc-400 w-8 text-center">{index + 1}</span>
                    <div class="flex-1 min-w-0">
                      <p class="text-zinc-100 font-medium truncate">{track.title}</p>
                      <p class="text-sm text-zinc-400 mt-0.5">{formatNumber(track.streams)} streams</p>
                    </div>
                    {#if track.revenue_estimate}
                      <span class="text-emerald-400 font-semibold">${formatNumber(track.revenue_estimate)}</span>
                    {/if}
                  </div>
                {/each}
              </div>
            </div>
          {/if}

          <div class="text-center py-4">
            <p class="text-xs text-zinc-500 max-w-xl mx-auto leading-relaxed">
              Revenue figures are <strong class="text-zinc-400">simulated</strong> based on average streaming payouts.
              Actual earnings vary by platform, region, and contract terms.
              {#if profile.label}
                <span class="block mt-2">Label: <span class="text-zinc-400">{profile.label}</span></span>
              {/if}
            </p>
          </div>
        </div>

      {:else if activeTab === 'credits'}
        <!-- Songwriters Section -->
        <div class="cred-section">
          <h3 class="cred-heading">
            Songwriters
            {#if profile.credits?.writers?.length}<span class="cred-heading__count">({profile.credits.writers.length})</span>{/if}
          </h3>
          {#if !profile.credits?.writers?.length}
            <div class="cred-empty surface-panel-thin">No writing credits found</div>
          {:else}
            {@const maxWriterTracks = Math.max(...profile.credits.writers.map(w => w.track_count))}
            <div class="cred-list surface-panel-thin">
              {#each profile.credits.writers as writer}
                <div class="cred-row">
                  <div class="cred-avatar">
                    {#if writer.image_url}<img src={writer.image_url} alt="" class="cred-avatar__img" on:error={hideImgOnError} />{/if}
                    <div class="cred-avatar__ph">{writer.name.charAt(0)}</div>
                  </div>
                  <div class="cred-info">
                    <div class="cred-name">
                      {writer.name}
                      {#if writer.is_flagged}<span class="cred-flag">flagged</span>{/if}
                    </div>
                    {#if writer.note}<p class="cred-note">{writer.note}</p>{/if}
                  </div>
                  <div class="cred-bar-wrap">
                    <div class="cred-bar" style="width: {maxWriterTracks > 0 ? (writer.track_count / maxWriterTracks * 100) : 0}%;"></div>
                  </div>
                  <span class="cred-tracks">{writer.track_count} tracks</span>
                </div>
              {/each}
            </div>
          {/if}
        </div>

        <!-- Producers Section -->
        <div class="cred-section">
          <h3 class="cred-heading">
            Producers
            {#if profile.credits?.producers?.length}<span class="cred-heading__count">({profile.credits.producers.length})</span>{/if}
          </h3>
          {#if !profile.credits?.producers?.length}
            <div class="cred-empty surface-panel-thin">No production credits found</div>
          {:else}
            {@const maxProducerTracks = Math.max(...profile.credits.producers.map(p => p.track_count))}
            <div class="cred-list surface-panel-thin">
              {#each profile.credits.producers as producer}
                <div class="cred-row">
                  <div class="cred-avatar">
                    {#if producer.image_url}<img src={producer.image_url} alt="" class="cred-avatar__img" on:error={hideImgOnError} />{/if}
                    <div class="cred-avatar__ph">{producer.name.charAt(0)}</div>
                  </div>
                  <div class="cred-info">
                    <div class="cred-name">
                      {producer.name}
                      {#if producer.is_flagged}<span class="cred-flag">flagged</span>{/if}
                    </div>
                  </div>
                  <div class="cred-bar-wrap">
                    <div class="cred-bar" style="width: {maxProducerTracks > 0 ? (producer.track_count / maxProducerTracks * 100) : 0}%;"></div>
                  </div>
                  <span class="cred-tracks">{producer.track_count} tracks</span>
                </div>
              {/each}
            </div>
          {/if}
        </div>

        <p class="cred-disclaimer">
          <strong>Flagged collaborators</strong> have documented offenses in our database.
        </p>

      {:else if activeTab === 'connections'}
        <!-- Connections — Dense list -->
        <div class="conn-header">
          <h2 class="conn-title">
            Connections
            {#if profile.collaborators.length > 0}
              <span class="conn-count">({profile.collaborators.length})</span>
            {/if}
          </h2>
          {#if profile.collaborators.length > 0}
            {@const flaggedCount = profile.collaborators.filter(c => c.is_flagged).length}
            {#if flaggedCount > 0}
              <span class="conn-flagged-summary">{flaggedCount} flagged/blocked</span>
            {/if}
          {/if}
        </div>

        {#if connectionsLoading}
          <div class="conn-empty surface-panel-thin">
            <div class="conn-empty__icon">
              <svg width="20" height="20" fill="none" stroke="currentColor" viewBox="0 0 24 24" class="conn-spinner">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
            </div>
            <p class="conn-empty__title">Loading connections...</p>
            <p class="conn-empty__sub">Fetching collaborator and network data</p>
          </div>
        {:else if profile.collaborators.length === 0}
          <div class="conn-empty surface-panel-thin">
            <div class="conn-empty__icon">
              <svg width="20" height="20" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
              </svg>
            </div>
            <p class="conn-empty__title">No collaborations found</p>
            <p class="conn-empty__sub">Collaboration data is being populated</p>
          </div>
        {:else}
          <div class="conn-list surface-panel-thin">
            {#each profile.collaborators as collab}
              <button
                type="button"
                class="conn-row {collab.is_flagged ? 'conn-row--flagged' : ''}"
                on:click={() => navigateToArtist(collab.id)}
              >
                <div class="conn-avatar">
                  {#if collab.image_url}
                    <img src={collab.image_url} alt="" class="conn-avatar__img" on:error={hideImgOnError} />
                  {/if}
                  <div class="conn-avatar__ph">{collab.name.charAt(0)}</div>
                </div>
                <div class="conn-info">
                  <span class="conn-name">{collab.name}</span>
                  {#if collab.is_flagged}
                    <span class="conn-dot conn-dot--flagged" title="Flagged or blocked artist"></span>
                  {:else if blockedNetworkArtists.has(collab.id)}
                    <span class="conn-dot conn-dot--blocked" title="In your blocked network"></span>
                  {:else}
                    <span class="conn-dot conn-dot--clean"></span>
                  {/if}
                </div>
                <span class="conn-collabs">{collab.collaboration_count} collab{collab.collaboration_count !== 1 ? 's' : ''}</span>
                <span class="conn-type">{collab.collaboration_type}</span>
                <svg class="conn-chevron" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                </svg>
              </button>
            {/each}
          </div>
        {/if}

        <!-- Warning Card -->
        <div class="conn-warning surface-panel-thin">
          <p>
            Connections shown represent professional collaborations only.
            <strong>Guilt is never transferred</strong> across connections.
            A collaboration with a flagged artist does not imply involvement in their misconduct.
          </p>
        </div>

      {/if}
    </main>
    </div><!-- end profile__content-card -->

    <!-- Report Error Modal -->
    {#if showReportModal}
      <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/85">
        <div
          class="w-full max-w-md rounded-2xl p-6 surface-panel-thin"
         
        >
          <div class="flex items-center justify-between mb-6">
            <h3 class="text-xl font-bold text-white">Report an Error</h3>
            <button
              type="button"
              on:click={() => showReportModal = false}
              class="p-2 rounded-lg text-zinc-400 hover:text-white hover:bg-zinc-700"
            >
              <svg class="w-5 h-5" width="20" height="20" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          <div class="space-y-4">
            <div>
              <label for="error-type" class="block text-sm font-medium text-zinc-300 mb-2">Error Type</label>
              <select id="error-type" bind:value={reportCategory} class="w-full px-4 py-3 rounded-lg text-white surface-panel-thin" >
                <option value="factual_error">Factual Error</option>
                <option value="wrong_artist">Wrong Artist</option>
                <option value="outdated_info">Outdated Information</option>
                <option value="missing_context">Missing Context</option>
                <option value="source_issue">Source Issue</option>
                <option value="other">Other</option>
              </select>
            </div>

            <div>
              <label for="error-description" class="block text-sm font-medium text-zinc-300 mb-2">Description</label>
              <textarea id="error-description" bind:value={reportDescription} rows="4" placeholder="Describe the error and provide any supporting information..." class="w-full px-4 py-3 rounded-lg text-white placeholder-zinc-500 resize-none surface-panel-thin" ></textarea>
            </div>

            <div class="flex gap-3 pt-2">
              <button
                type="button"
                on:click={() => showReportModal = false}
                class="flex-1 px-4 py-3 rounded-lg font-medium transition-colors hover:bg-zinc-700 surface-panel-thin text-white"
               
              >
                Cancel
              </button>
              <button
                type="button"
                on:click={submitReport}
                disabled={!reportDescription.trim()}
                class="flex-1 px-4 py-3 rounded-lg font-medium transition-colors disabled:opacity-50 hover:bg-rose-700 bg-rose-600 text-white"
               
              >
                Submit Report
              </button>
            </div>
          </div>
        </div>
      </div>
    {/if}

    <!-- Album Cover Modal -->
    {#if showAlbumCoverModal && selectedAlbumCover}
      <div
        class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/90"
       
        on:click={closeAlbumCoverModal}
        on:keydown={(e) => e.key === 'Escape' && closeAlbumCoverModal()}
        role="dialog"
        aria-modal="true"
        aria-label="Album cover preview"
        tabindex="-1"
      >
        <div class="relative max-w-2xl max-h-[80vh]" on:click|stopPropagation>
          <button
            type="button"
            on:click={closeAlbumCoverModal}
            class="absolute -top-12 right-0 p-2 rounded-lg text-zinc-400 hover:text-white hover:bg-zinc-800 transition-colors"
            aria-label="Close"
          >
            <svg class="w-6 h-6" width="24" height="24" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
          <img
            src={selectedAlbumCover.url}
            alt={selectedAlbumCover.name}
            class="max-w-full max-h-[70vh] rounded-xl shadow-2xl object-contain"
            on:error={handleImgError}
          />
          <div class="text-center mt-4">
            <p class="text-white font-semibold text-lg">{selectedAlbumCover.name}</p>
          </div>
        </div>
      </div>
    {/if}
  {/if}
</div>

<style>
  /* ===== Root ===== */
  .profile {
    min-height: 100%;
    color: var(--color-text-primary);
  }

  /* ===== Loading / Error States ===== */
  .profile__loading {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: min(70vh, 38rem);
    padding: 1.5rem 0;
  }

  .profile__loading-inner {
    text-align: center;
    width: min(100%, 22rem);
    padding: 2rem;
    border-radius: 1.5rem;
  }

  .profile__spinner {
    width: 3rem;
    height: 3rem;
    border: 4px solid var(--color-brand-primary);
    border-top-color: transparent;
    border-radius: var(--radius-full);
    animation: profile-spin 1s linear infinite;
    margin: 0 auto;
  }

  .profile__loading-text {
    margin-top: 1rem;
    color: var(--color-text-secondary);
  }

  .profile__error-card {
    text-align: center;
    max-width: 28rem;
    padding: 2rem;
    border-radius: 1.5rem;
    background:
      linear-gradient(180deg, rgba(255, 255, 255, 0.05), transparent 18%),
      rgba(9, 9, 11, 0.86);
    border: 1px solid rgba(255, 255, 255, 0.08);
    box-shadow: 0 28px 80px rgba(0, 0, 0, 0.42);
    backdrop-filter: blur(18px);
  }

  .profile__error-bang {
    font-size: 3.75rem;
    color: var(--color-brand-primary);
    margin-bottom: 1rem;
  }

  .profile__error-title {
    font-size: var(--text-2xl);
    font-weight: 700;
    color: var(--color-text-primary);
    margin: 0 0 0.5rem;
  }

  .profile__error-msg {
    color: var(--color-text-secondary);
    margin: 0 0 1.5rem;
  }

  .profile__error-btn {
    min-width: 8.5rem;
  }
  .profile__error-btn:hover {
    transform: none;
    filter: none;
  }

  /* ===== Container ===== */
  .profile__container {
    max-width: 72rem;
    margin: 0 auto;
    padding-left: 1.5rem;
    padding-right: 1.5rem;
  }

  /* ===== Header Section ===== */
  .profile__header-section {
    max-width: 72rem;
    margin: 0 auto;
    padding: clamp(1.2rem, 3vw, 1.8rem);
  }

  .profile__header-section .profile__container {
    max-width: none;
    padding-left: 0;
    padding-right: 0;
  }

  .profile__back-icon {
    width: 1.25rem;
    height: 1.25rem;
  }

  .profile__back-link {
    margin-bottom: 1rem;
  }

  .profile__header-row {
    display: flex;
    align-items: flex-start;
    gap: 1.5rem;
  }

  /* ===== Photo ===== */
  .profile__photo-wrap {
    flex-shrink: 0;
  }

  .profile__photo {
    width: 8rem;
    height: 8rem;
    border-radius: 1.4rem;
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.2);
    overflow: hidden;
    border: 2px solid var(--color-border-default);
  }

  .profile__photo-img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .profile__photo-placeholder {
    width: 8rem;
    height: 8rem;
    border-radius: 1.4rem;
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.2);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 2.25rem;
    font-weight: 700;
    background:
      linear-gradient(180deg, rgba(255, 255, 255, 0.045), rgba(255, 255, 255, 0.02)),
      rgba(24, 24, 27, 0.95);
    border: 2px solid rgba(255, 255, 255, 0.08);
    color: var(--color-brand-primary);
  }

  /* ===== Artist Info ===== */
  .profile__info {
    flex: 1;
    min-width: 0;
  }

  .profile__kickers {
    margin-bottom: 0.75rem;
  }

  .profile__badges {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 0.75rem;
  }

  .profile__status-badge {
    padding: 0.5rem 1rem;
    border-radius: var(--radius-full);
    font-size: var(--text-sm);
    font-weight: 800;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    box-shadow: var(--shadow-lg);
    text-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
  }

  .profile__confidence {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.25rem 0.5rem;
    border-radius: var(--radius-full);
    background-color: var(--color-bg-elevated);
    border: 1px solid var(--color-border-subtle);
  }

  .profile__confidence-bars {
    display: flex;
    gap: 2px;
  }

  .profile__confidence-bar {
    width: 0.375rem;
    height: 0.75rem;
    border-radius: 1px;
  }

  .profile__confidence-label {
    font-size: var(--text-xs);
    color: var(--color-text-secondary);
  }

  .profile__name {
    font-size: clamp(2rem, 5vw, 3rem);
    font-weight: 700;
    color: var(--color-text-primary);
    margin: 0 0 0.5rem;
    letter-spacing: -0.04em;
    line-height: 1.05;
  }

  .profile__genres {
    font-size: var(--text-sm);
    color: var(--color-text-tertiary);
    margin: 0 0 0.75rem;
  }

  .profile__meta-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.65rem;
  }

  /* ===== Actions ===== */
  .profile__actions {
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    align-self: flex-start;
    margin-top: 0.5rem;
    min-width: 0;
    max-width: 12rem;
  }

  .profile__block-group {
    position: relative;
  }

  .profile__block-split {
    display: flex;
    width: 100%;
  }

  .profile__block-btn {
    padding: 0.5rem 1rem;
    border-radius: var(--radius-full) 0 0 var(--radius-full);
    font-weight: 700;
    font-size: var(--text-sm);
    transition: opacity var(--transition-fast);
    display: flex;
    align-items: center;
    gap: 0.375rem;
    cursor: pointer;
    border: none;
    white-space: nowrap;
    flex: 1;
    justify-content: center;
  }
  .profile__block-btn:disabled { opacity: 0.5; }
  .profile__block-btn:hover:not(:disabled) { opacity: 0.9; }

  .profile__block-btn--unblocked {
    background: var(--color-brand-gradient);
    color: var(--color-text-on-brand);
  }

  .profile__block-btn--blocked {
    background: var(--color-blocked-bg);
    color: var(--color-blocked-text);
    border: 2px solid var(--color-blocked-border);
    border-right: none;
  }

  .profile__block-dropdown {
    padding: 0.5rem 0.625rem;
    border-radius: 0 var(--radius-full) var(--radius-full) 0;
    font-weight: 700;
    transition: opacity var(--transition-fast);
    display: flex;
    align-items: center;
    cursor: pointer;
    border: none;
    flex-shrink: 0;
  }
  .profile__block-dropdown:hover { opacity: 0.9; }

  .profile__block-dropdown--unblocked {
    background: var(--color-brand-gradient);
    color: var(--color-text-on-brand);
    border-left: 1px solid var(--color-overlay-subtle);
  }

  .profile__block-dropdown--blocked {
    background: var(--color-blocked-bg);
    color: var(--color-blocked-text);
    border: 2px solid var(--color-blocked-border);
    border-left: 1px solid var(--color-blocked-border-accent);
  }

  /* ===== Dropdown Menu ===== */
  .profile__dropdown-menu {
    position: absolute;
    right: 0;
    margin-top: 0.5rem;
    width: 16rem;
    border-radius: 1.15rem;
    box-shadow: 0 24px 48px rgba(0, 0, 0, 0.3);
    z-index: 50;
    overflow: hidden;
    background:
      linear-gradient(180deg, rgba(255, 255, 255, 0.04), rgba(255, 255, 255, 0.018)),
      rgba(17, 17, 19, 0.96);
    border: 1px solid rgba(255, 255, 255, 0.08);
    backdrop-filter: blur(16px);
  }

  .profile__dropdown-inner {
    padding: 0.5rem;
  }

  .profile__dropdown-label {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    padding: 0.5rem 0.75rem;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .profile__dropdown-item {
    width: 100%;
    padding: 0.5rem 0.75rem;
    text-align: left;
    font-size: var(--text-sm);
    color: var(--color-text-secondary);
    border-radius: var(--radius-lg);
    display: flex;
    align-items: center;
    gap: 0.75rem;
    background: none;
    border: none;
    cursor: pointer;
    transition: background-color var(--transition-fast);
  }
  .profile__dropdown-item:hover {
    background-color: var(--color-bg-interactive);
  }

  .profile__dropdown-item--danger {
    color: var(--color-error);
    font-weight: 500;
  }
  .profile__dropdown-item--danger:hover {
    background-color: rgba(244, 63, 94, 0.1);
  }

  .profile__dropdown-icon {
    width: 1rem;
    height: 1rem;
  }
  .profile__dropdown-icon--rose { color: #fb7185; }
  .profile__dropdown-icon--orange { color: #fb923c; }
  .profile__dropdown-icon--purple { color: #c084fc; }
  .profile__dropdown-icon--blue { color: #60a5fa; }

  .profile__dropdown-divider {
    margin: 0.5rem 0;
    border-top: 1px solid var(--color-border-subtle);
  }

  /* ===== Secondary Actions ===== */
  .profile__secondary-actions {
    display: flex;
    gap: 0.375rem;
    width: 100%;
  }

  .profile__action-btn {
    padding: 0.375rem 0.625rem;
    border-radius: 0.75rem;
    font-size: var(--text-xs);
    font-weight: 600;
    transition: background-color var(--transition-fast), border-color var(--transition-fast), transform var(--transition-fast);
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.25rem;
    border: none;
    cursor: pointer;
    white-space: nowrap;
    flex: 1;
    min-width: 0;
  }

  .profile__action-btn--primary {
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.08);
    color: var(--color-text-primary);
  }
  .profile__action-btn--primary:hover {
    transform: translateY(-1px);
    border-color: rgba(244, 63, 94, 0.18);
    background: rgba(255, 255, 255, 0.08);
  }

  .profile__action-btn--secondary {
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.08);
    color: var(--color-text-tertiary);
  }
  .profile__action-btn--secondary:hover {
    transform: translateY(-1px);
    border-color: rgba(244, 63, 94, 0.18);
    background: rgba(255, 255, 255, 0.08);
  }

  .profile__action-icon {
    width: 0.875rem;
    height: 0.875rem;
    flex-shrink: 0;
  }

  /* ===== Content Card (wraps tabs + main) ===== */
  .profile__content-card {
    max-width: 72rem;
    margin: 0.75rem auto 0;
    border-radius: 1.75rem;
    background:
      linear-gradient(180deg, rgba(255, 255, 255, 0.025), rgba(255, 255, 255, 0.008)),
      rgba(17, 17, 19, 0.92);
    border: 1px solid rgba(255, 255, 255, 0.08);
    overflow: hidden;
  }

  /* ===== Tab Navigation ===== */
  .profile__tab-bar {
    display: flex;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  }

  .profile__tab {
    flex: 1 1 0;
    padding: 0.875rem 1rem;
    text-align: center;
    color: var(--color-text-secondary);
    background: transparent;
    border: none;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    position: relative;
    white-space: nowrap;
    transition: color 0.15s;
  }

  .profile__tab:hover {
    color: var(--color-text-primary);
    background: rgba(255, 255, 255, 0.02);
  }

  .profile__tab--active {
    color: #fff;
  }

  .profile__tab--active::after {
    content: '';
    position: absolute;
    bottom: 0;
    left: 0.75rem;
    right: 0.75rem;
    height: 2px;
    background: var(--color-brand-primary);
    border-radius: 1px 1px 0 0;
  }

  /* ===== Main Content ===== */
  .profile__main {
    padding: 1.5rem;
  }

  /* ===== Catalog Sub-tabs ===== */
  .catalog-subtab {
    flex: 1 1 0;
    padding: 0.625rem 0.75rem;
    text-align: center;
    color: var(--color-text-tertiary);
    background: transparent;
    border: none;
    font-size: 0.8125rem;
    font-weight: 500;
    cursor: pointer;
    position: relative;
    white-space: nowrap;
    transition: color 0.15s;
  }

  .catalog-subtab:hover {
    color: var(--color-text-primary);
  }

  .catalog-subtab--active {
    color: #fff;
  }

  .catalog-subtab--active::after {
    content: '';
    position: absolute;
    bottom: 0;
    left: 0.75rem;
    right: 0.75rem;
    height: 2px;
    background: var(--color-brand-primary);
    border-radius: 1px 1px 0 0;
  }

  @media (max-width: 900px) {
    .profile__header-row {
      flex-direction: column;
      align-items: flex-start;
    }

    .profile__actions {
      flex-direction: row;
      align-items: center;
      width: 100%;
      max-width: 100%;
      gap: 0.75rem;
    }

    .profile__block-group {
      flex-shrink: 0;
    }

    .profile__secondary-actions {
      flex: 1;
    }

    .profile__tab {
      min-width: max-content;
    }
  }

  /* ===== Utility ===== */
  .line-clamp-2 {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  /* ===== Show More Button ===== */
  .profile__show-more {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    width: 100%;
    padding: var(--space-3);
    margin-top: var(--space-2);
    border-radius: var(--radius-lg);
    border: 1px dashed var(--color-border-subtle);
    background: transparent;
    color: var(--color-text-tertiary);
    font-size: var(--text-xs);
    font-weight: 500;
    cursor: pointer;
    transition: all var(--transition-fast);
  }
  .profile__show-more:hover {
    color: var(--color-text-primary);
    border-color: var(--color-brand-primary-muted);
    background: rgba(255, 255, 255, 0.02);
  }

  /* ===== Press Feedback ===== */
  .profile__block-btn:active:not(:disabled),
  .profile__action-btn:active {
    transform: scale(0.97);
  }

  @keyframes profile-spin {
    to { transform: rotate(360deg); }
  }

  /* ===== Evidence Tab — News Feed ===== */
  .ev-header { margin-bottom: 1rem; }

  .ev-title {
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
  }

  .ev-count {
    color: var(--color-text-muted);
    font-weight: 400;
    margin-left: 0.25rem;
  }

  .ev-empty {
    padding: 2rem;
    border-radius: 0.75rem;
    text-align: center;
  }

  .ev-feed {
    border-radius: 0.75rem;
    overflow: hidden;
  }

  .ev-card--bordered {
    border-top: 1px solid rgba(255, 255, 255, 0.06);
  }

  .ev-card__btn {
    width: 100%;
    padding: 1.25rem;
    text-align: left;
    background: transparent;
    border: none;
    cursor: pointer;
    color: inherit;
    font: inherit;
    transition: background-color 0.15s;
    display: block;
  }

  .ev-card__btn:hover {
    background: rgba(255, 255, 255, 0.03);
  }

  .ev-card__badges {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.375rem;
    margin-bottom: 0.625rem;
  }

  .ev-pill {
    padding: 0.1875rem 0.5rem;
    font-size: 0.6875rem;
    font-weight: 600;
    border-radius: 9999px;
    white-space: nowrap;
  }

  .ev-pill--muted {
    background: rgba(255, 255, 255, 0.08);
    color: var(--color-text-secondary);
  }

  .ev-card__date {
    margin-left: auto;
    font-size: 0.75rem;
    color: var(--color-text-muted);
  }

  .ev-card__title {
    font-size: 0.9375rem;
    font-weight: 600;
    color: #fff;
    margin: 0 0 0.375rem;
    line-height: 1.4;
  }

  .ev-card__desc {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
    margin: 0;
    display: -webkit-box;
    -webkit-line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
    line-height: 1.5;
  }

  .ev-card__footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: 0.75rem;
  }

  .ev-card__sources {
    font-size: 0.75rem;
    color: var(--color-text-tertiary);
  }

  .ev-card__chevron {
    width: 1rem;
    height: 1rem;
    color: var(--color-text-muted);
    transition: transform 0.2s;
    flex-shrink: 0;
  }

  .ev-card__chevron--open {
    transform: rotate(180deg);
  }

  /* Evidence expanded sources */
  .ev-sources {
    padding: 0 1.25rem 1.25rem;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
  }

  .ev-sources__heading {
    font-size: 0.6875rem;
    font-weight: 500;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    padding-top: 1rem;
    margin: 0 0 0.75rem;
  }

  .ev-sources__empty {
    font-size: 0.8125rem;
    color: var(--color-text-muted);
    margin: 0;
  }

  .ev-source {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    padding: 0.625rem 0.75rem;
    border-radius: 0.5rem;
    transition: background-color 0.15s;
    text-decoration: none;
    color: inherit;
  }

  .ev-source:hover {
    background: rgba(255, 255, 255, 0.04);
  }

  .ev-source__tier {
    width: 1.75rem;
    height: 1.75rem;
    border-radius: 0.25rem;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.625rem;
    font-weight: 700;
    flex-shrink: 0;
  }

  .ev-source__info { flex: 1; min-width: 0; }
  .ev-source__title { font-size: 0.8125rem; font-weight: 500; color: #d4d4d8; margin: 0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; display: flex; align-items: center; gap: 0.375rem; }
  .ev-source__meta { font-size: 0.6875rem; color: #52525b; margin: 0.125rem 0 0; }
  .ev-source__excerpt { font-size: 0.6875rem; color: #52525b; margin: 0.25rem 0 0; font-style: italic; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }

  .ev-source--broken { opacity: 0.55; }
  .ev-source--broken .ev-source__title { text-decoration: line-through; text-decoration-color: rgba(239, 68, 68, 0.5); }
  .ev-source--archived { border-left: 2px solid rgba(245, 158, 11, 0.5); }

  .ev-source__badge {
    flex-shrink: 0;
    font-size: 0.5625rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    padding: 0.1rem 0.375rem;
    border-radius: 0.25rem;
    line-height: 1.2;
  }
  .ev-source__badge--archived { background: rgba(245, 158, 11, 0.2); color: #F59E0B; }
  .ev-source__badge--broken { background: rgba(239, 68, 68, 0.2); color: #EF4444; }
  .ev-source__badge--checking { background: rgba(161, 161, 170, 0.15); color: #71717a; }

  /* Evidence summary strip */
  .ev-summary {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1px;
    background: rgba(255, 255, 255, 0.06);
    border-radius: 0.75rem;
    overflow: hidden;
    margin-top: 1rem;
  }

  .ev-summary__cell {
    padding: 0.875rem 1rem;
    background: var(--color-bg-elevated, rgba(24, 24, 27, 0.95));
    text-align: center;
  }

  .ev-summary__value {
    display: block;
    font-size: 1.25rem;
    font-weight: 700;
    color: var(--color-text-primary);
    font-variant-numeric: tabular-nums;
  }

  .ev-summary__label {
    display: block;
    font-size: 0.6875rem;
    color: var(--color-text-tertiary);
    margin-top: 0.25rem;
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  /* ===== Catalog Tab ===== */
  .cat-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 1.25rem;
  }

  .cat-indicator {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .cat-indicator__bar {
    height: 0.375rem;
    width: 7rem;
    border-radius: 9999px;
    overflow: hidden;
    background: rgba(255, 255, 255, 0.04);
  }

  .cat-indicator__fill {
    height: 100%;
    border-radius: 9999px;
    background: var(--color-brand-primary);
    transition: width 0.3s;
  }

  .cat-indicator__text {
    font-size: 0.75rem;
    color: var(--color-text-tertiary);
  }

  .cat-indicator__count {
    color: #fb7185;
    font-weight: 500;
  }

  /* Album list */
  .cat-albums { border-radius: 0.75rem; overflow: hidden; }
  .cat-album--bordered { border-top: 1px solid rgba(255, 255, 255, 0.06); }

  .cat-album__header {
    width: 100%;
    padding: 0.75rem 1.25rem;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    text-align: left;
    background: transparent;
    border: none;
    cursor: pointer;
    color: inherit;
    font: inherit;
    transition: background-color 0.15s;
  }

  .cat-album__header:hover { background: rgba(255, 255, 255, 0.04); }

  .cat-album__art {
    width: 3rem;
    height: 3rem;
    border-radius: 0.5rem;
    overflow: hidden;
    background: #27272a;
    flex-shrink: 0;
    position: relative;
  }

  .cat-album__art-img { width: 100%; height: 100%; object-fit: cover; position: absolute; inset: 0; }
  .cat-album__art-ph { width: 100%; height: 100%; display: flex; align-items: center; justify-content: center; }
  .cat-album__art-ph svg { width: 1rem; height: 1rem; color: #52525b; }

  .cat-album__info { flex: 1; min-width: 0; }
  .cat-album__title { font-size: 0.9375rem; font-weight: 600; color: #fff; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .cat-album__meta { font-size: 0.75rem; color: #71717a; margin-top: 0.125rem; }

  .cat-album__status { font-size: 0.6875rem; color: #52525b; flex-shrink: 0; }
  .cat-album__status--all { color: #fb7185; }

  .cat-chevron { width: 1rem; height: 1rem; color: #52525b; flex-shrink: 0; transition: transform 0.2s; }
  .cat-chevron--open { transform: rotate(180deg); }

  /* Expanded track table */
  .cat-tracks { border-top: 1px solid rgba(255, 255, 255, 0.06); }

  .cat-batch {
    padding: 0.5rem 1.25rem;
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: rgba(0, 0, 0, 0.2);
  }

  .cat-batch__count { font-size: 0.75rem; color: #52525b; }

  .cat-batch__btn {
    font-size: 0.75rem;
    font-weight: 500;
    padding: 0.25rem 0.625rem;
    border-radius: 9999px;
    background: none;
    border: none;
    cursor: pointer;
    color: #71717a;
    transition: all 0.15s;
  }

  .cat-batch__btn:hover { color: #fff; }
  .cat-batch__btn--danger { color: #fb7185; background: rgba(244, 63, 94, 0.1); }
  .cat-batch__btn--danger:hover { background: rgba(244, 63, 94, 0.15); }

  /* Toggle switch */
  .cat-toggle {
    position: relative;
    width: 2rem;
    height: 1.125rem;
    border-radius: 9999px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.1);
    cursor: pointer;
    flex-shrink: 0;
    padding: 0;
    transition: background-color 0.2s, border-color 0.2s;
  }

  .cat-toggle--active {
    background: rgba(244, 63, 94, 0.25);
    border-color: #fb7185;
  }

  .cat-toggle__thumb {
    position: absolute;
    top: 0.125rem;
    left: 0.125rem;
    width: 0.75rem;
    height: 0.75rem;
    border-radius: 50%;
    background: #71717a;
    transition: transform 0.2s, background-color 0.2s;
  }

  .cat-toggle--active .cat-toggle__thumb {
    transform: translateX(0.875rem);
    background: #fb7185;
  }

  /* Track rows */
  .cat-track {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.5rem 1.25rem;
    border-top: 1px solid rgba(255, 255, 255, 0.04);
    transition: background-color 0.1s;
  }

  .cat-track:hover { background: rgba(255, 255, 255, 0.03); }

  .cat-track__num {
    width: 1.5rem;
    text-align: right;
    font-size: 0.75rem;
    color: #52525b;
    font-variant-numeric: tabular-nums;
    flex-shrink: 0;
  }

  .cat-track__info { flex: 1; min-width: 0; display: flex; align-items: baseline; gap: 0.375rem; }
  .cat-track__title { font-size: 0.8125rem; color: #d4d4d8; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .cat-track__title--blocked { text-decoration: line-through; color: rgba(251, 113, 133, 0.6); }
  .cat-track__feat { font-size: 0.6875rem; color: #52525b; white-space: nowrap; flex-shrink: 0; }

  .cat-track__duration {
    font-size: 0.75rem;
    color: #52525b;
    width: 3rem;
    text-align: right;
    font-variant-numeric: tabular-nums;
    flex-shrink: 0;
  }

  /* Featured / Behind list rows */
  .cat-list { border-radius: 0.75rem; overflow: hidden; }
  .cat-list-row { display: flex; align-items: center; gap: 0.75rem; padding: 0.625rem 1.25rem; transition: background-color 0.1s; }
  .cat-list-row:hover { background: rgba(255, 255, 255, 0.03); }
  .cat-list-row--bordered { border-top: 1px solid rgba(255, 255, 255, 0.06); }

  .cat-list-row__info { flex: 1; min-width: 0; }
  .cat-list-row__title { font-size: 0.8125rem; color: #d4d4d8; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .cat-list-row__meta { font-size: 0.6875rem; color: #52525b; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; margin-top: 0.125rem; }
  .cat-list-row__role { font-size: 0.6875rem; color: #52525b; flex-shrink: 0; text-transform: capitalize; }
  .cat-list-row__year { font-size: 0.6875rem; color: #52525b; flex-shrink: 0; }

  .cat-empty { text-align: center; padding: 3rem 0; font-size: 0.8125rem; color: #52525b; }

  .cat-disclaimer {
    font-size: 0.75rem;
    color: #52525b;
    text-align: center;
    padding-top: 0.5rem;
    margin: 0;
  }

  /* ===== Credits Tab ===== */
  .cred-section {
    margin-bottom: 1.5rem;
  }

  .cred-heading {
    font-size: 1.125rem;
    font-weight: 600;
    color: #fff;
    margin: 0 0 0.75rem;
  }

  .cred-heading__count {
    color: #71717a;
    font-weight: 400;
    font-size: 0.875rem;
  }

  .cred-empty {
    padding: 1.5rem;
    border-radius: 0.75rem;
    font-size: 0.8125rem;
    color: #52525b;
  }

  .cred-list {
    border-radius: 0.75rem;
    overflow: hidden;
  }

  .cred-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.625rem 1.25rem;
    border-top: 1px solid rgba(255, 255, 255, 0.04);
    transition: background-color 0.1s;
  }

  .cred-row:first-child { border-top: none; }
  .cred-row:hover { background: rgba(255, 255, 255, 0.02); }

  .cred-avatar {
    width: 2.5rem;
    height: 2.5rem;
    border-radius: 50%;
    overflow: hidden;
    flex-shrink: 0;
    position: relative;
  }

  .cred-avatar__img {
    width: 2.5rem;
    height: 2.5rem;
    border-radius: 50%;
    object-fit: cover;
    position: absolute;
    inset: 0;
  }

  .cred-avatar__ph {
    width: 2.5rem;
    height: 2.5rem;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.75rem;
    font-weight: 600;
    background: #27272a;
    color: #71717a;
  }

  .cred-info { flex: 1; min-width: 0; }
  .cred-name { font-size: 0.8125rem; color: #d4d4d8; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .cred-flag {
    display: inline-block;
    margin-left: 0.375rem;
    font-size: 0.625rem;
    font-weight: 600;
    padding: 0.0625rem 0.375rem;
    border-radius: 9999px;
    background: rgba(244, 63, 94, 0.15);
    color: #fb7185;
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  .cred-note {
    font-size: 0.6875rem;
    color: #d97706;
    margin: 0.125rem 0 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .cred-bar-wrap {
    width: 5rem;
    height: 0.375rem;
    border-radius: 0.25rem;
    background: rgba(255, 255, 255, 0.04);
    overflow: hidden;
    flex-shrink: 0;
  }

  .cred-bar {
    height: 100%;
    border-radius: 0.25rem;
    background: linear-gradient(90deg, #6366f1, #818cf8);
    transition: width 0.3s;
  }

  .cred-tracks {
    font-size: 0.75rem;
    color: #52525b;
    flex-shrink: 0;
    min-width: 4rem;
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  .cred-disclaimer {
    font-size: 0.75rem;
    color: #71717a;
    text-align: center;
    margin: 0;
  }

  .cred-disclaimer strong { color: #d4d4d8; }

  /* ===== Connections Tab ===== */
  .conn-header { margin-bottom: 1rem; display: flex; align-items: baseline; gap: 0.5rem; }

  .conn-title {
    font-size: 1.125rem;
    font-weight: 600;
    color: #fff;
    margin: 0;
  }

  .conn-count {
    color: #71717a;
    font-weight: 400;
  }

  .conn-flagged-summary {
    font-size: 0.75rem;
    color: #fb7185;
    font-weight: 500;
    margin-left: auto;
  }

  .conn-spinner {
    animation: conn-spin 1.2s linear infinite;
  }

  @keyframes conn-spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .conn-empty {
    padding: 3rem;
    border-radius: 0.75rem;
    text-align: center;
  }

  .conn-empty__icon {
    width: 2.5rem;
    height: 2.5rem;
    margin: 0 auto 0.75rem;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(99, 102, 241, 0.12);
    color: #818cf8;
  }

  .conn-empty__title { font-size: 0.875rem; font-weight: 500; color: #d4d4d8; margin: 0; }
  .conn-empty__sub { font-size: 0.75rem; color: #52525b; margin: 0.25rem 0 0; }

  .conn-list {
    border-radius: 0.75rem;
    overflow: hidden;
  }

  .conn-row {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.625rem 1.25rem;
    background: transparent;
    border: none;
    border-top: 1px solid rgba(255, 255, 255, 0.04);
    cursor: pointer;
    color: inherit;
    font: inherit;
    text-align: left;
    transition: background-color 0.15s;
  }

  .conn-row:first-child { border-top: none; }
  .conn-row:hover { background: rgba(255, 255, 255, 0.04); }
  .conn-row--flagged { background: rgba(244, 63, 94, 0.04); }
  .conn-row--flagged:hover { background: rgba(244, 63, 94, 0.07); }

  .conn-avatar {
    width: 3rem;
    height: 3rem;
    border-radius: 0.75rem;
    overflow: hidden;
    flex-shrink: 0;
    position: relative;
    background: #27272a;
  }

  .conn-avatar__img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    position: absolute;
    inset: 0;
  }

  .conn-avatar__ph {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1rem;
    font-weight: 600;
    color: #52525b;
  }

  .conn-info {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .conn-name {
    font-size: 0.875rem;
    font-weight: 500;
    color: #fafafa;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .conn-dot {
    width: 0.5rem;
    height: 0.5rem;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .conn-dot--clean { background: #22c55e; }
  .conn-dot--flagged { background: #fb7185; }
  .conn-dot--blocked { background: #f59e0b; }

  .conn-collabs {
    font-size: 0.75rem;
    color: #71717a;
    flex-shrink: 0;
    font-variant-numeric: tabular-nums;
  }

  .conn-type {
    font-size: 0.6875rem;
    padding: 0.125rem 0.5rem;
    border-radius: 9999px;
    background: rgba(255, 255, 255, 0.06);
    color: #a1a1aa;
    text-transform: capitalize;
    flex-shrink: 0;
    white-space: nowrap;
  }

  .conn-chevron {
    width: 1rem;
    height: 1rem;
    color: #52525b;
    flex-shrink: 0;
  }

  .conn-warning {
    margin-top: 1rem;
    padding: 1rem 1.25rem;
    border-radius: 0.75rem;
  }

  .conn-warning p {
    font-size: 0.8125rem;
    color: #a1a1aa;
    margin: 0;
    line-height: 1.5;
  }

  .conn-warning strong { color: #fafafa; }
</style>
