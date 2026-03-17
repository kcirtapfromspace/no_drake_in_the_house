<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import {
    artistStore,
    artistActions,
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
    type Evidence,
    type SourceTier
  } from '../stores/artist';
  import { navigateTo, navigateToArtist } from '../utils/simple-router';
  import { apiClient } from '../utils/api-client';
  import ArtistDiscographyRevenue from './ArtistDiscographyRevenue.svelte';

  export let artistId: string;

  let profile: ArtistProfile | null = null;
  let isLoading = true;
  let error: string | null = null;
  let activeTab: 'evidence' | 'catalog' | 'discography' | 'credits' | 'connections' = 'evidence';

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

  // Track blocking in progress
  let trackBlockingInProgress: Set<string> = new Set();

  // Expandable albums state for catalog view
  let expandedCatalogAlbums: Set<string> = new Set();

  // Album cover modal state
  let showAlbumCoverModal = false;
  let selectedAlbumCover: { url: string; name: string } | null = null;

  // Blocking options dropdown state
  let showBlockingOptions = false;

  function openAlbumCover(url: string, name: string) {
    selectedAlbumCover = { url, name };
    showAlbumCoverModal = true;
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

      // Fetch collaborators
      try {
        const collabResult = await apiClient.get<any>(`/api/v1/graph/artists/${artistId}/collaborators`);
        if (collabResult.success && collabResult.data && profile) {
          profile.collaborators = collabResult.data.collaborators || collabResult.data || [];
        }
      } catch (collabErr) {
        console.log('Collaborators API failed, will use fallback');
      }

      // If no collaborators and this is Drake, add simulated ones for showcase
      if (profile && (!profile.collaborators || profile.collaborators.length === 0) && profile.canonical_name === 'Drake') {
        profile.collaborators = [
          // Featured Artists
          { id: 'collab-1', name: 'Future', collaboration_type: 'featured', collaboration_count: 25, is_flagged: false, status: 'clean', image_url: 'https://i.scdn.co/image/ab6761610000e5ebdf9a1555f53a20087b8c5a5c' },
          { id: 'collab-2', name: '21 Savage', collaboration_type: 'featured', collaboration_count: 18, is_flagged: false, status: 'clean', image_url: 'https://i.scdn.co/image/ab6761610000e5eb50defaf9fc059a1efc541f4c' },
          { id: 'collab-3', name: 'Lil Wayne', collaboration_type: 'featured', collaboration_count: 28, is_flagged: false, status: 'clean', image_url: 'https://i.scdn.co/image/ab6761610000e5eb749114d7b3fc16d80a0a9572' },
          { id: 'collab-4', name: 'Rihanna', collaboration_type: 'featured', collaboration_count: 12, is_flagged: false, status: 'clean', image_url: 'https://i.scdn.co/image/ab6761610000e5eb99e4fca7c0b7cb166d915789' },
          { id: 'collab-5', name: 'Travis Scott', collaboration_type: 'featured', collaboration_count: 8, is_flagged: true, status: 'flagged', image_url: 'https://i.scdn.co/image/ab6761610000e5eb19c2790744c792d05570bb71' },
          { id: 'collab-6', name: 'Chris Brown', collaboration_type: 'featured', collaboration_count: 7, is_flagged: true, status: 'flagged', image_url: 'https://i.scdn.co/image/ab6761610000e5eb6be070445b03e0b63147c2c1' },
          { id: 'collab-7', name: 'Nicki Minaj', collaboration_type: 'featured', collaboration_count: 15, is_flagged: false, status: 'clean', image_url: 'https://i.scdn.co/image/ab6761610000e5bbd0251a2d22a026181f0c5cc5' },
          { id: 'collab-8', name: 'The Weeknd', collaboration_type: 'featured', collaboration_count: 9, is_flagged: false, status: 'clean', image_url: 'https://i.scdn.co/image/ab6761610000e5eb214f3cf1cbe7139c1e26ffbb' },
          { id: 'collab-9', name: 'DJ Khaled', collaboration_type: 'featured', collaboration_count: 20, is_flagged: false, status: 'clean', image_url: 'https://i.scdn.co/image/ab6761610000e5eb8945f4d4f9ceccf7a44e5e2a' },
          { id: 'collab-10', name: 'Rick Ross', collaboration_type: 'featured', collaboration_count: 12, is_flagged: false, status: 'clean', image_url: 'https://i.scdn.co/image/ab6761610000e5eb85c03c0b19a5a34d76f88de1' },
          { id: 'collab-11', name: 'J. Cole', collaboration_type: 'featured', collaboration_count: 6, is_flagged: false, status: 'clean', image_url: 'https://i.scdn.co/image/ab6761610000e5ebadd503b411a712e277895c8a' },
          { id: 'collab-12', name: 'Kanye West', collaboration_type: 'featured', collaboration_count: 5, is_flagged: true, status: 'flagged', image_url: 'https://i.scdn.co/image/ab6761610000e5eb6e835a500e791bf9c27a519a' },
          { id: 'collab-13', name: 'Kendrick Lamar', collaboration_type: 'featured', collaboration_count: 4, is_flagged: false, status: 'clean', image_url: 'https://i.scdn.co/image/ab6761610000e5eb437b9e2a82505b3d93ff1022' },
          { id: 'collab-14', name: 'Migos', collaboration_type: 'featured', collaboration_count: 8, is_flagged: false, status: 'certified_creeper', image_url: 'https://i.scdn.co/image/ab6761610000e5eb1a0c6c6c6c6c6c6c6c6c6c6c' },
          { id: 'collab-15', name: 'Young Thug', collaboration_type: 'featured', collaboration_count: 7, is_flagged: true, status: 'flagged', image_url: 'https://i.scdn.co/image/ab6761610000e5eb4b4b4b4b4b4b4b4b4b4b4b4b' },
          { id: 'collab-16', name: 'Partynextdoor', collaboration_type: 'featured', collaboration_count: 14, is_flagged: false, status: 'clean', image_url: 'https://i.scdn.co/image/ab6761610000e5eb9997e163236a0dbbe8e7fdf7' },
          { id: 'collab-17', name: 'Tory Lanez', collaboration_type: 'featured', collaboration_count: 6, is_flagged: true, status: 'flagged', image_url: 'https://i.scdn.co/image/ab6761610000e5eb4f4f4f4f4f4f4f4f4f4f4f4f' },
          { id: 'collab-18', name: 'Quavo', collaboration_type: 'featured', collaboration_count: 5, is_flagged: false, status: 'clean', image_url: 'https://i.scdn.co/image/ab6761610000e5ebf83e7d4a4e4a4e4a4e4a4e4a' },
          // Producers
          { id: 'collab-p1', name: 'Noah "40" Shebib', collaboration_type: 'producer', collaboration_count: 150, is_flagged: false, status: 'clean', image_url: 'https://i.scdn.co/image/ab6761610000e5eb9e3c7c7c7c7c7c7c7c7c7c7c' },
          { id: 'collab-p2', name: 'Boi-1da', collaboration_type: 'producer', collaboration_count: 55, is_flagged: false, status: 'clean', image_url: 'https://i.scdn.co/image/ab6761610000e5eb6a6a6a6a6a6a6a6a6a6a6a6a' },
          { id: 'collab-p3', name: 'Metro Boomin', collaboration_type: 'producer', collaboration_count: 25, is_flagged: false, status: 'clean', image_url: 'https://i.scdn.co/image/ab6761610000e5eb9d4b4b4b4b4b4b4b4b4b4b4b' },
          { id: 'collab-p4', name: 'Tay Keith', collaboration_type: 'producer', collaboration_count: 15, is_flagged: false, status: 'clean', image_url: 'https://i.scdn.co/image/ab6761610000e5eb8e8e8e8e8e8e8e8e8e8e8e8e' },
          { id: 'collab-p5', name: 'Hit-Boy', collaboration_type: 'producer', collaboration_count: 12, is_flagged: false, status: 'clean', image_url: 'https://i.scdn.co/image/ab6761610000e5eb7f7f7f7f7f7f7f7f7f7f7f7f' },
        ];

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

    } catch (e: any) {
      error = e.message || 'Failed to load artist';
    } finally {
      isLoading = false;
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
        id: e.id,
        offense_id: data.id,
        source: {
          id: e.id,
          url: e.source_url,
          title: e.title || e.source_name,
          source_name: e.source_name,
          source_type: e.source_type || 'news',
          tier: determineSourceTier(e),
          published_date: e.published_date,
          excerpt: e.excerpt,
          credibility_score: e.credibility_score,
        },
        date_added: e.date_added || new Date().toISOString(),
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
          <svg class="brand-back__icon profile__back-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
          </svg>
          Back to Home
        </button>

        <div class="profile__header-row">
          <!-- Artist Photo -->
          {#if profile.primary_image?.url}
            <div class="profile__photo-wrap">
              <div
                class="profile__photo"
                style="border-color: {statusColor?.border || 'var(--color-border-default)'};"
              >
                <img
                  src={profile.primary_image.url}
                  alt={profile.canonical_name}
                  class="profile__photo-img"
                />
              </div>
            </div>
          {:else}
            <div class="profile__photo-wrap">
              <div class="profile__photo-placeholder">
                {profile.canonical_name.charAt(0)}
              </div>
            </div>
          {/if}

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
                      <div class="w-5 h-5 border-2 border-current border-t-transparent rounded-full animate-spin"></div>
                    {:else if isBlocked}
                      <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                        <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
                      </svg>
                      Blocked
                    {:else}
                      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
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
                    <svg class="w-4 h-4 transition-transform {showBlockingOptions ? 'rotate-180' : ''}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
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
                        <svg class="profile__dropdown-icon profile__dropdown-icon--rose" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3" />
                        </svg>
                        Block All Main Tracks
                      </button>
                      <button
                        type="button"
                        on:click={() => { toggleRoleBlocking('featured', true); showBlockingOptions = false; }}
                        class="profile__dropdown-item"
                      >
                        <svg class="profile__dropdown-icon profile__dropdown-icon--orange" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                        </svg>
                        Block Collaborations
                      </button>
                      <button
                        type="button"
                        on:click={() => { toggleRoleBlocking('producer', true); showBlockingOptions = false; }}
                        class="profile__dropdown-item"
                      >
                        <svg class="profile__dropdown-icon profile__dropdown-icon--purple" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z" />
                        </svg>
                        Block Producer Credits
                      </button>
                      <button
                        type="button"
                        on:click={() => { toggleRoleBlocking('writer', true); showBlockingOptions = false; }}
                        class="profile__dropdown-item"
                      >
                        <svg class="profile__dropdown-icon profile__dropdown-icon--blue" fill="none" stroke="currentColor" viewBox="0 0 24 24">
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
                  <svg class="profile__action-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                  </svg>
                  Evidence
                </button>
                <button
                  type="button"
                  on:click={() => showReportModal = true}
                  class="profile__action-btn profile__action-btn--secondary"
                >
                  <svg class="profile__action-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                  </svg>
                  Report
                </button>
              </div>
          </div>
        </div>
      </div>
    </section>

    <!-- Tab Navigation -->
    <div class="profile__tab-bar">
      <div class="profile__container profile__tab-bar-inner">
        <div class="brand-segmented profile__tab-group" role="tablist" aria-label="Artist profile sections">
          {#each [
            { key: 'evidence', label: 'Evidence' },
            { key: 'catalog', label: 'Full Catalog' },
            { key: 'discography', label: 'Revenue' },
            { key: 'credits', label: 'Credits' },
            { key: 'connections', label: 'Connections' },
          ] as tab}
            <button
              type="button"
              on:click={() => activeTab = tab.key}
              class="brand-segmented__item profile__tab"
              class:brand-segmented__item--active={activeTab === tab.key}
              aria-pressed={activeTab === tab.key}
            >
              {tab.label}
            </button>
          {/each}
        </div>
      </div>
    </div>

    <!-- Main Content -->
    <main class="profile__main">
      {#if activeTab === 'evidence'}
        <!-- Evidence Timeline -->
        <div class="grid lg:grid-cols-3 gap-8">
          <!-- Main Timeline -->
          <div class="lg:col-span-2">
            <h2 class="text-2xl font-bold text-white mb-6">
              Documented Incidents
              {#if profile.offenses.length > 0}
                <span class="text-zinc-400 font-normal ml-2">({profile.offenses.length})</span>
              {/if}
            </h2>

            {#if profile.offenses.length === 0}
              <div class="text-center py-16 rounded-2xl bg-zinc-900 border border-zinc-900">
                <div class="w-20 h-20 mx-auto mb-4 rounded-full flex items-center justify-center bg-green-500/20">
                  <svg class="w-10 h-10 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                </div>
                <p class="text-zinc-200 text-lg">No documented incidents</p>
                <p class="text-zinc-400 text-sm mt-2">No offense records found for this artist</p>
              </div>
            {:else}
              <!-- Timeline -->
              <div class="space-y-4">
                {#each profile.offenses.sort((a, b) => new Date(b.incident_date || b.created_at).getTime() - new Date(a.incident_date || a.created_at).getTime()) as offense, index}
                  {@const catColor = getCategoryColor(offense.category.id)}
                  {@const evidenceStrength = getEvidenceStrengthLabel(offense.evidence_strength)}
                  {@const isExpanded = expandedOffenseId === offense.id}

                  <div
                    class="rounded-2xl overflow-hidden transition-all relative group/card"
                    style="background: {catColor.cardBg}; border: 2px solid {catColor.icon}; box-shadow: 0 0 40px {catColor.bg};"
                  >
                    <!-- Category accent stripe with glow -->
                    <div class="absolute left-0 top-0 bottom-0 w-2 rounded-l-2xl" style="background: {catColor.icon};"></div>
                    <!-- Offense Header -->
                    <button
                      type="button"
                      on:click={() => expandedOffenseId = isExpanded ? null : offense.id}
                      class="w-full p-6 pl-8 text-left transition-all hover:bg-white/[0.02] bg-transparent border-none"
                    >
                      <div class="flex items-start gap-4">
                        <!-- Timeline Indicator with glow -->
                        <div class="flex-shrink-0 flex flex-col items-center">
                          <div
                            class="w-12 h-12 rounded-xl flex items-center justify-center relative"
                            style="background: {catColor.bg}; box-shadow: 0 0 20px {catColor.icon}40, 0 0 40px {catColor.icon}20;"
                          >
                            <svg class="w-6 h-6" style="color: {catColor.icon};" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                            </svg>
                          </div>
                          {#if index < profile.offenses.length - 1}
                            <div class="w-0.5 h-full min-h-[20px] mt-2" style="background: linear-gradient(180deg, {catColor.icon}60, transparent);"></div>
                          {/if}
                        </div>

                        <!-- Offense Content -->
                        <div class="flex-1 min-w-0">
                          <div class="flex items-center flex-wrap gap-2 mb-2">
                            <!-- Category Badge with glow -->
                            <span
                              class="px-3 py-1 text-xs font-semibold rounded-full border"
                              style="background: {catColor.bg}; color: {catColor.icon}; border-color: {catColor.icon}50; box-shadow: 0 0 12px {catColor.icon}30;"
                            >
                              {offense.category.name}
                            </span>

                            <!-- Procedural State Badge -->
                            <span class="px-3 py-1 text-xs font-medium rounded-full text-zinc-200 bg-zinc-800" >
                              {getProceduralStateLabel(offense.procedural_state)}
                            </span>

                            <!-- Evidence Strength with glow -->
                            <span
                              class="px-3 py-1 text-xs font-semibold rounded-full border"
                              style="background: rgba({evidenceStrength.color === '#10B981' ? '16, 185, 129' : evidenceStrength.color === '#F59E0B' ? '245, 158, 11' : '239, 68, 68'}, 0.15); color: {evidenceStrength.color}; border-color: {evidenceStrength.color}50; box-shadow: 0 0 12px {evidenceStrength.color}30;"
                            >
                              {evidenceStrength.label} Evidence
                            </span>
                          </div>

                          <h3 class="text-xl font-semibold text-white mb-3">{offense.title}</h3>
                          <p class="text-zinc-300 leading-relaxed line-clamp-2">{offense.description}</p>

                          {#if offense.incident_date}
                            <p class="text-sm text-zinc-300 mt-3 flex items-center gap-2">
                              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                              </svg>
                              {formatDate(offense.incident_date)}
                            </p>
                          {/if}
                        </div>

                        <!-- Expand Icon -->
                        <div class="flex-shrink-0">
                          <svg
                            class="w-6 h-6 text-zinc-400 transition-transform"
                            style="transform: rotate({isExpanded ? '180deg' : '0deg'});"
                            fill="none" stroke="currentColor" viewBox="0 0 24 24"
                          >
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                          </svg>
                        </div>
                      </div>
                    </button>

                    <!-- Expanded Evidence Section -->
                    {#if isExpanded}
                      <div
                        class="px-6 pb-6"
                        style="background: linear-gradient(180deg, {catColor.icon}20 0%, {catColor.icon}08 40%, transparent 100%); box-shadow: inset 0 1px 0 {catColor.icon}40;"
                      >
                        <div class="pt-6">
                          <h4 class="text-sm font-medium text-zinc-300 uppercase tracking-wider mb-4">
                            Sources ({offense.evidence.length})
                          </h4>

                          {#if offense.evidence.length === 0}
                            <p class="text-zinc-400 text-sm">No sources available</p>
                          {:else}
                            <div class="space-y-3">
                              {#each offense.evidence as evidence}
                                {@const tierInfo = getSourceTierLabel(evidence.source.tier)}
                                <a href={evidence.source.url} target="_blank" rel="noopener noreferrer" class="flex items-start gap-4 p-4 rounded-lg transition-all hover:bg-white/5 bg-black/20" >
                                  <!-- Tier Badge -->
                                  <div
                                    class="flex-shrink-0 w-10 h-10 rounded-lg flex items-center justify-center text-xs font-bold"
                                    style="background: {tierInfo.color}30; color: {tierInfo.color};"
                                    title={tierInfo.description}
                                  >
                                    {tierInfo.label.replace('Tier ', '')}
                                  </div>

                                  <!-- Source Info -->
                                  <div class="flex-1 min-w-0">
                                    <p class="font-medium text-white truncate">
                                      {evidence.source.title || evidence.source.source_name}
                                    </p>
                                    <p class="text-sm text-zinc-400 flex items-center gap-2">
                                      <span>{evidence.source.source_name}</span>
                                      {#if evidence.source.published_date}
                                        <span>|</span>
                                        <span>{formatDate(evidence.source.published_date)}</span>
                                      {/if}
                                    </p>
                                    {#if evidence.source.excerpt}
                                      <p class="text-sm text-zinc-300 mt-2 line-clamp-2 italic">
                                        "{evidence.source.excerpt}"
                                      </p>
                                    {/if}
                                  </div>

                                  <!-- External Link Icon -->
                                  <svg class="w-5 h-5 text-zinc-400 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                                  </svg>
                                </a>
                              {/each}
                            </div>
                          {/if}
                        </div>
                      </div>
                    {/if}
                  </div>
                {/each}
              </div>
            {/if}
          </div>

          <!-- Sidebar -->
          <div class="lg:col-span-1 space-y-6">
            <!-- Source Reliability Legend -->
            <div class="rounded-3xl p-6 bg-gradient-deep shadow-deep">
              <h3 class="text-lg font-semibold mb-5 text-stone-50">Source Reliability</h3>
              <div class="space-y-3">
                {#each ['tier_a', 'tier_b', 'tier_c', 'tier_d'] as tier}
                  {@const tierInfo = getSourceTierLabel(tier)}
                  <div class="flex items-start gap-3">
                    <div
                      class="w-8 h-8 rounded-lg flex items-center justify-center text-xs font-bold flex-shrink-0"
                      style="background: {tierInfo.color}30; color: {tierInfo.color};"
                    >
                      {tierInfo.label.replace('Tier ', '')}
                    </div>
                    <div class="text-sm">
                      <p class="text-white font-medium">{tierInfo.label}</p>
                      <p class="text-zinc-400 text-xs">{tierInfo.description}</p>
                    </div>
                  </div>
                {/each}
              </div>
            </div>

            <!-- Quick Stats -->
            <div class="rounded-3xl p-6 bg-gradient-deep shadow-deep">
              <h3 class="text-lg font-semibold mb-5 text-stone-50">Profile Summary</h3>
              <div class="space-y-4">
                <div class="flex justify-between">
                  <span class="text-zinc-300">Total Incidents</span>
                  <span class="text-white font-medium">{profile.offenses.length}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-zinc-300">Primary Sources</span>
                  <span class="text-white font-medium">
                    {profile.offenses.reduce((count, o) =>
                      count + o.evidence.filter(e => e.source.tier === 'tier_a').length, 0
                    )}
                  </span>
                </div>
                <div class="flex justify-between">
                  <span class="text-zinc-300">Status</span>
                  <span
                    class="px-2 py-0.5 rounded-md text-xs font-medium"
                    style="background: {statusColor?.bg}; color: {statusColor?.text};"
                  >
                    {statusLabel}
                  </span>
                </div>
                <div class="flex justify-between">
                  <span class="text-zinc-300">Confidence</span>
                  <span class="text-white font-medium">{confidenceLabel}</span>
                </div>
              </div>
            </div>

          </div>
        </div>

      {:else if activeTab === 'catalog'}
        <!-- Full Artist Catalog - All Appearances -->
        <div class="space-y-6">
          <!-- Catalog Summary -->
          <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
            {#each [
              { role: 'main', label: 'Main Artist' },
              { role: 'featured', label: 'Featured On' },
              { role: 'producer', label: 'Producer' },
              { role: 'writer', label: 'Writer' }
            ] as cat}
              {@const total = catalog.filter(t => t.role === cat.role).length}
              {@const blocked = catalog.filter(t => t.role === cat.role && t.isBlocked).length}
              {@const allBlocked = blocked === total && total > 0}
              <button
                type="button"
                on:click={() => toggleRoleBlocking(cat.role, !allBlocked)}
                class="p-4 rounded-xl text-left transition-all hover:scale-[1.02] border {allBlocked ? 'bg-rose-500/10 border-rose-500/30' : 'bg-zinc-900 border-zinc-800/50'}"
              >
                <div class="flex items-center justify-between">
                  <div class="text-2xl font-bold text-white">{total}</div>
                  {#if allBlocked}
                    <div class="flex items-center gap-1 px-2 py-0.5 rounded bg-rose-500/20 border border-rose-500/40">
                      <svg class="w-3 h-3 text-rose-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
                        <path d="M18 6L6 18M6 6l12 12" stroke-linecap="round" />
                      </svg>
                      <span class="text-xs font-semibold text-rose-400">ALL</span>
                    </div>
                  {:else if blocked > 0}
                    <div class="px-2 py-0.5 rounded bg-amber-500/20 border border-amber-500/30">
                      <span class="text-xs font-medium text-amber-400">{blocked}/{total}</span>
                    </div>
                  {:else}
                    <div class="px-2 py-0.5 rounded bg-zinc-800/50 border border-zinc-700/30">
                      <span class="text-xs font-medium text-zinc-500">None</span>
                    </div>
                  {/if}
                </div>
                <div class="text-sm text-zinc-400 mt-1">{cat.label}</div>
                <div class="text-xs mt-1 {blocked > 0 ? 'text-rose-400' : 'text-zinc-500'}">
                  {blocked > 0 ? `${blocked} blocked` : 'Click to block all'}
                </div>
              </button>
            {/each}
          </div>

          <!-- Blocking Progress & Filter -->
          <div class="p-4 rounded-xl bg-gradient-panel border border-zinc-900">
            <div class="flex items-center justify-between mb-3">
              <div class="flex items-center gap-3">
                <span class="text-sm font-medium text-zinc-300">Catalog Blocking Status</span>
                <div class="flex gap-1">
                  <button
                    type="button"
                    on:click={() => catalogFilter = 'all'}
                    class="px-2.5 py-1 text-xs rounded-md transition-all {catalogFilter === 'all' ? 'bg-zinc-700 text-white' : 'bg-zinc-900 text-zinc-400 hover:bg-zinc-800'}"
                  >All</button>
                  <button
                    type="button"
                    on:click={() => catalogFilter = 'blocked'}
                    class="px-2.5 py-1 text-xs rounded-md transition-all flex items-center gap-1 {catalogFilter === 'blocked' ? 'bg-rose-500/20 text-rose-400 border border-rose-500/40' : 'bg-zinc-900 text-zinc-400 hover:bg-zinc-800'}"
                  >
                    <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                      <path d="M18 6L6 18M6 6l12 12" stroke-linecap="round" />
                    </svg>
                    Blocked
                  </button>
                  <button
                    type="button"
                    on:click={() => catalogFilter = 'unblocked'}
                    class="px-2.5 py-1 text-xs rounded-md transition-all flex items-center gap-1 {catalogFilter === 'unblocked' ? 'bg-zinc-500/20 text-zinc-300 border border-zinc-500/50' : 'bg-zinc-900 text-zinc-400 hover:bg-zinc-800'}"
                  >
                    <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <circle cx="12" cy="12" r="9" />
                    </svg>
                    Allowed
                  </button>
                </div>
              </div>
              <!-- Blocked counter with clear semantics -->
              <div class="flex items-center gap-2">
                <span class="text-sm text-rose-400 font-medium">{catalog.filter(t => t.isBlocked).length}</span>
                <span class="text-sm text-zinc-500">of {catalog.length} blocked</span>
              </div>
            </div>
            <!-- Progress bar: red for blocked, gray for allowed -->
            <div class="h-2 rounded-full overflow-hidden bg-zinc-800/50">
              <div
                class="h-full rounded-full transition-all"
                style="width: {(catalog.filter(t => t.isBlocked).length / catalog.length * 100)}%; background: linear-gradient(90deg, #f43f5e, #e11d48);"
              ></div>
            </div>
            <p class="text-xs text-zinc-500 mt-2">
              <span class="text-rose-400">●</span> Blocked tracks won't play • <span class="text-zinc-400">○</span> Allowed tracks play normally
            </p>
          </div>

          <!-- Main Artist Albums (Expandable) -->
          {#if catalogAlbums.length > 0}
          <div>
            <h3 class="text-lg font-semibold text-white mb-4 flex items-center gap-2">
              <svg class="w-5 h-5 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3" />
              </svg>
              Main Artist ({filteredCatalog.filter(t => t.role === 'main').length} tracks across {catalogAlbums.length} albums)
            </h3>

            <div class="space-y-3">
              {#each catalogAlbums as album}
                <div class="rounded-xl overflow-hidden bg-zinc-900 border border-zinc-900">
                  <!-- Album Header (clickable) -->
                  <button
                    type="button"
                    class="w-full p-4 flex items-center gap-4 hover:bg-zinc-800/50 transition-colors text-left"
                    on:click={() => toggleCatalogAlbum(album.name)}
                  >
                    <!-- Block Toggle (left side) - Crystal clear state indication -->
                    <button
                      type="button"
                      on:click|stopPropagation={() => toggleAlbumBlocking(album.name, album.blockedCount < album.totalCount)}
                      class="flex-shrink-0 transition-all duration-200 hover:scale-105 active:scale-95 group relative"
                      title={album.blockedCount === album.totalCount ? 'Click to ALLOW all tracks' : 'Click to BLOCK all tracks'}
                    >
                      {#if album.blockedCount === album.totalCount}
                        <!-- BLOCKED STATE: Vibrant rose on dark -->
                        <div class="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg transition-all bg-rose-500/20 border border-rose-500/40 group-hover:bg-rose-500/30">
                          <svg class="w-4 h-4 text-rose-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                            <circle cx="12" cy="12" r="9" />
                            <path d="M5.5 5.5l13 13" />
                          </svg>
                          <span class="text-xs font-semibold uppercase tracking-wide text-rose-400">Blocked</span>
                        </div>
                      {:else if album.blockedCount > 0}
                        <!-- PARTIAL STATE: Vibrant amber -->
                        <div class="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg transition-all bg-amber-500/20 border border-amber-500/40 group-hover:bg-amber-500/30">
                          <svg class="w-4 h-4 text-amber-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <circle cx="12" cy="12" r="9" />
                            <path d="M12 8v4m0 4h.01" stroke-linecap="round" />
                          </svg>
                          <span class="text-xs font-medium text-amber-400">{album.blockedCount}/{album.totalCount}</span>
                        </div>
                      {:else}
                        <!-- ALLOWED STATE: Subtle warm gray -->
                        <div class="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg bg-zinc-800/30 border border-zinc-700/30 group-hover:bg-zinc-700/40 group-hover:border-zinc-500/50 transition-all">
                          <svg class="w-4 h-4 text-zinc-400 group-hover:text-zinc-300" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <circle cx="12" cy="12" r="9" />
                            <path d="M8 12l2 2 4-4" stroke-linecap="round" stroke-linejoin="round" />
                          </svg>
                          <span class="text-xs font-medium text-zinc-400 group-hover:text-zinc-300">Allowed</span>
                        </div>
                      {/if}
                    </button>

                    <!-- Album Art (thumbnail) - Clickable for larger view -->
                    {#if album.cover && !album.cover.includes('data:image')}
                    <button
                      type="button"
                      class="flex-shrink-0 group relative"
                      on:click|stopPropagation={() => openAlbumCover(album.cover, album.name)}
                      title="Click to enlarge"
                    >
                      <img
                        src={album.cover}
                        alt={album.name}
                        class="w-10 h-10 rounded object-cover bg-zinc-800 transition-all group-hover:ring-2 group-hover:ring-white/30"
                        on:error={(e) => { e.currentTarget.parentElement.style.display = 'none'; }}
                      />
                      <div class="absolute inset-0 flex items-center justify-center bg-black/50 opacity-0 group-hover:opacity-100 transition-opacity rounded">
                        <svg class="w-4 h-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0zM10 7v3m0 0v3m0-3h3m-3 0H7" />
                        </svg>
                      </div>
                    </button>
                    {/if}

                    <!-- Album Info -->
                    <div class="flex-1 min-w-0">
                      <div class="font-semibold text-stone-100 truncate">{album.name}</div>
                      <div class="text-sm text-zinc-400">{album.year} · {album.totalCount} tracks</div>
                      <div class="flex items-center gap-2 mt-1">
                        <div class="h-1.5 flex-1 max-w-32 rounded-full overflow-hidden bg-zinc-800/50">
                          <div
                            class="h-full rounded-full transition-all"
                            style="width: {(album.blockedCount / album.totalCount * 100)}%; background: {album.blockedCount > 0 ? 'linear-gradient(90deg, #f43f5e, #e11d48)' : 'transparent'};"
                          ></div>
                        </div>
                        <span class="text-xs {album.blockedCount > 0 ? 'text-rose-400' : 'text-zinc-500'}">{album.blockedCount}/{album.totalCount} blocked</span>
                      </div>
                    </div>

                    <!-- Expand/Collapse Icon (plus/minus) -->
                    <div class="w-6 h-6 rounded-full border border-zinc-700 flex items-center justify-center flex-shrink-0">
                      {#if expandedCatalogAlbums.has(album.name)}
                        <svg class="w-3.5 h-3.5 text-zinc-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4" />
                        </svg>
                      {:else}
                        <svg class="w-3.5 h-3.5 text-zinc-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
                        </svg>
                      {/if}
                    </div>
                  </button>

                  <!-- Expanded Track List -->
                  {#if expandedCatalogAlbums.has(album.name)}
                    <div class="border-t border-zinc-800">
                      <table class="w-full text-sm">
                        <thead>
                          <tr class="text-left text-zinc-500 bg-zinc-900/50">
                            <th class="py-2 px-4 font-medium w-12"></th>
                            <th class="py-2 px-4 font-medium w-10">#</th>
                            <th class="py-2 px-4 font-medium">Title</th>
                            <th class="py-2 px-4 font-medium text-right">Duration</th>
                          </tr>
                        </thead>
                        <tbody>
                          {#each album.tracks as track, idx}
                            <tr class="border-t border-zinc-900/50 hover:bg-zinc-900/30">
                              <td class="py-2 px-4">
                                <button
                                  type="button"
                                  on:click|stopPropagation={() => toggleTrackBlock(track.id)}
                                  class="transition-all duration-200 hover:scale-105 active:scale-95 group"
                                  title={track.isBlocked ? 'Click to ALLOW this track' : 'Click to BLOCK this track'}
                                >
                                  {#if track.isBlocked}
                                    <!-- BLOCKED: Vibrant rose circle with X -->
                                    <div class="w-7 h-7 rounded-full flex items-center justify-center transition-all bg-rose-500/20 border-2 border-rose-500 group-hover:bg-rose-500/30">
                                      <svg class="w-3.5 h-3.5 text-rose-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
                                        <path d="M18 6L6 18M6 6l12 12" stroke-linecap="round" />
                                      </svg>
                                    </div>
                                  {:else}
                                    <!-- ALLOWED: Empty circle - shows X on hover -->
                                    <div class="w-7 h-7 rounded-full border-2 border-zinc-600 flex items-center justify-center group-hover:border-rose-400/50 group-hover:bg-rose-500/10 transition-all">
                                      <svg class="w-3.5 h-3.5 text-zinc-700 group-hover:text-rose-400 transition-colors opacity-0 group-hover:opacity-100" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
                                        <path d="M18 6L6 18M6 6l12 12" stroke-linecap="round" />
                                      </svg>
                                    </div>
                                  {/if}
                                </button>
                              </td>
                              <td class="py-2 px-4 text-zinc-500">{idx + 1}</td>
                              <td class="py-2 px-4">
                                <div class="font-medium {track.isBlocked ? 'line-through text-rose-300/70 decoration-rose-500/50' : 'text-zinc-200'}">{track.title}</div>
                                {#if track.collaborators}
                                  <div class="text-xs text-zinc-500">feat. {track.collaborators.join(', ')}</div>
                                {/if}
                              </td>
                              <td class="py-2 px-4 text-right text-zinc-500">{track.duration || '—'}</td>
                            </tr>
                          {/each}
                        </tbody>
                      </table>
                    </div>
                  {/if}
                </div>
              {/each}
            </div>
          </div>
          {/if}

          <!-- Featured Appearances -->
          {#if filteredCatalog.filter(t => t.role === 'featured').length > 0}
          <div>
            <h3 class="text-lg font-semibold text-white mb-4 flex items-center gap-2">
              <svg class="w-5 h-5 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
              </svg>
              Featured Appearances ({catalog.filter(t => t.role === 'featured').length})
            </h3>
            <div class="overflow-x-auto">
              <table class="w-full text-sm">
                <thead>
                  <tr class="text-left text-zinc-500 border-b border-zinc-900">
                    <th class="pb-2 font-medium w-12"></th>
                    <th class="pb-2 font-medium">Title</th>
                    <th class="pb-2 font-medium">Main Artist</th>
                    <th class="pb-2 font-medium">Album</th>
                    <th class="pb-2 font-medium text-center">Year</th>
                  </tr>
                </thead>
                <tbody>
                  {#each filteredCatalog.filter(t => t.role === 'featured') as track}
                    <tr class="border-b border-zinc-900/50 hover:bg-zinc-900/30">
                      <td class="py-3">
                        <button
                          type="button"
                          on:click|stopPropagation={() => toggleTrackBlock(track.id)}
                          class="transition-all duration-200 hover:scale-105 active:scale-95 group"
                          title={track.isBlocked ? 'Click to ALLOW this track' : 'Click to BLOCK this track'}
                        >
                          {#if track.isBlocked}
                            <!-- BLOCKED: Filled red circle with X -->
                            <div class="w-6 h-6 rounded-full bg-rose-500/15 border-2 border-rose-500 flex items-center justify-center group-hover:bg-rose-500/20 transition-all">
                              <svg class="w-3 h-3 text-rose-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
                                <path d="M18 6L6 18M6 6l12 12" stroke-linecap="round" />
                              </svg>
                            </div>
                          {:else}
                            <!-- ALLOWED: Empty circle - click to block -->
                            <div class="w-6 h-6 rounded-full border-2 border-zinc-700 flex items-center justify-center group-hover:border-zinc-400 group-hover:bg-rose-500/10 transition-all">
                              <svg class="w-3 h-3 text-zinc-700 group-hover:text-rose-400 transition-colors opacity-0 group-hover:opacity-100" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
                                <path d="M18 6L6 18M6 6l12 12" stroke-linecap="round" />
                              </svg>
                            </div>
                          {/if}
                        </button>
                      </td>
                      <td class="py-3">
                        <div class="font-medium {track.isBlocked ? 'line-through text-rose-300/70 decoration-rose-500/50' : 'text-zinc-200'}">{track.title}</div>
                      </td>
                      <td class="py-3 text-zinc-400">{track.collaborators?.join(', ') || '—'}</td>
                      <td class="py-3 text-zinc-500">{track.album || '—'}</td>
                      <td class="py-3 text-center text-zinc-500">{track.year || '—'}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </div>
          {/if}

          <!-- Producer/Writer Credits -->
          {#if filteredCatalog.filter(t => t.role === 'producer' || t.role === 'writer').length > 0}
            <div>
              <h3 class="text-lg font-semibold text-white mb-4 flex items-center gap-2">
                <svg class="w-5 h-5 text-amber-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
                </svg>
                Behind the Scenes ({catalog.filter(t => t.role === 'producer' || t.role === 'writer').length})
              </h3>
              <p class="text-sm text-zinc-500 mb-4">
                Songs where {profile.canonical_name} contributed as producer or writer. These may require additional review as blocking preferences vary.
              </p>
              <div class="overflow-x-auto">
                <table class="w-full text-sm">
                  <thead>
                    <tr class="text-left text-zinc-500 border-b border-zinc-900">
                      <th class="pb-2 font-medium w-12"></th>
                      <th class="pb-2 font-medium">Title</th>
                      <th class="pb-2 font-medium">Role</th>
                      <th class="pb-2 font-medium">Artist</th>
                      <th class="pb-2 font-medium text-center">Year</th>
                    </tr>
                  </thead>
                  <tbody>
                    {#each filteredCatalog.filter(t => t.role === 'producer' || t.role === 'writer') as track}
                      <tr class="border-b border-zinc-900/50 hover:bg-zinc-900/30">
                        <td class="py-3">
                          <button
                            type="button"
                            on:click|stopPropagation={() => toggleTrackBlock(track.id)}
                            class="transition-all duration-200 hover:scale-105 active:scale-95 group"
                            title={track.isBlocked ? 'Click to ALLOW this track' : 'Click to BLOCK this track'}
                          >
                            {#if track.isBlocked}
                              <!-- BLOCKED: Filled red circle with X -->
                              <div class="w-6 h-6 rounded-full bg-rose-500/15 border-2 border-rose-500 flex items-center justify-center group-hover:bg-rose-500/20 transition-all">
                                <svg class="w-3 h-3 text-rose-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
                                  <path d="M18 6L6 18M6 6l12 12" stroke-linecap="round" />
                                </svg>
                              </div>
                            {:else}
                              <!-- ALLOWED: Empty circle - click to block -->
                              <div class="w-6 h-6 rounded-full border-2 border-zinc-700 flex items-center justify-center group-hover:border-zinc-400 group-hover:bg-rose-500/10 transition-all">
                                <svg class="w-3 h-3 text-zinc-700 group-hover:text-rose-400 transition-colors opacity-0 group-hover:opacity-100" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
                                  <path d="M18 6L6 18M6 6l12 12" stroke-linecap="round" />
                                </svg>
                              </div>
                            {/if}
                          </button>
                        </td>
                        <td class="py-3">
                          <div class="font-medium {track.isBlocked ? 'line-through text-rose-300/70 decoration-rose-500/50' : 'text-zinc-200'}">{track.title}</div>
                        </td>
                        <td class="py-3">
                          <span class="px-2 py-0.5 rounded text-xs font-medium capitalize {track.role === 'producer' ? 'bg-purple-900/30 text-purple-400' : 'bg-blue-900/30 text-blue-400'}">
                            {track.role}
                          </span>
                        </td>
                        <td class="py-3 text-zinc-400">{track.collaborators?.join(', ') || '—'}</td>
                        <td class="py-3 text-center text-zinc-500">{track.year || '—'}</td>
                      </tr>
                    {/each}
                  </tbody>
                </table>
              </div>
            </div>
          {/if}

          <!-- Catalog Info -->
          <div class="p-4 rounded-xl bg-zinc-900 border border-zinc-900">
            <div class="flex items-start gap-3">
              <svg class="w-5 h-5 text-zinc-500 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              <div>
                <p class="text-sm text-zinc-400">
                  This catalog tracks all known appearances by {profile.canonical_name}. When enforcement is active,
                  the system will scan your connected streaming services and block/skip matching tracks.
                </p>
                <p class="text-xs text-zinc-500 mt-2">
                  Catalog data is aggregated from Spotify, Apple Music, and MusicBrainz. Some entries may be incomplete.
                </p>
              </div>
            </div>
          </div>
        </div>

      {:else if activeTab === 'discography'}
        <!-- Discography Impact Panel - Simplified Layout -->
        <div class="space-y-8">
          <!-- Primary: Discography Revenue (always shown, uses simulated data) -->
          <div data-testid="discography-revenue-section">
            <ArtistDiscographyRevenue
              artistId={artistId}
              artistName={profile.canonical_name}
            />
          </div>

          <!-- Secondary: Platform-specific data (only if available and non-zero) -->
          {#if profile.streaming_metrics?.platform_breakdown?.length > 0}
            <div class="bg-zinc-900 rounded-2xl p-6">
              <h3 class="text-lg font-semibold text-stone-100 mb-6">Platform Distribution</h3>
              <div class="space-y-5">
                {#each profile.streaming_metrics.platform_breakdown as platform}
                  <div>
                    <div class="flex justify-between text-sm mb-2">
                      <span class="text-zinc-200 capitalize font-medium">{platform.platform}</span>
                      <span class="text-zinc-300">{formatNumber(platform.streams)} ({platform.percentage}%)</span>
                    </div>
                    <div class="h-3 rounded-full overflow-hidden bg-zinc-900">
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

          <!-- Tertiary: Top Tracks (only if available) -->
          {#if profile.streaming_metrics?.top_tracks?.length > 0}
            <div class="bg-zinc-900 rounded-2xl p-6">
              <h3 class="text-lg font-semibold text-stone-100 mb-6">Top Tracks</h3>
              <div class="space-y-3">
                {#each profile.streaming_metrics.top_tracks.slice(0, 5) as track, index}
                  <div class="flex items-center gap-4 p-4 rounded-xl bg-zinc-900/30 hover:bg-zinc-900/60 transition-colors">
                    <span class="text-2xl font-bold text-zinc-500 w-8 text-center">{index + 1}</span>
                    <div class="flex-1 min-w-0">
                      <p class="text-stone-100 font-medium truncate">{track.title}</p>
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

          <!-- Data Disclaimer -->
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
        <!-- Writer & Producer Credits -->
        <div class="grid lg:grid-cols-2 gap-8">
          <!-- Writers Section -->
          <div>
            <h2 class="text-2xl font-bold text-white mb-6 flex items-center gap-3">
              <svg class="w-6 h-6 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
              </svg>
              Songwriters
              {#if profile.credits?.writers?.length}
                <span class="text-zinc-400 font-normal text-lg">({profile.credits.writers.length})</span>
              {/if}
            </h2>

            {#if !profile.credits?.writers?.length}
              <div class="text-center py-12 rounded-2xl bg-zinc-900 border border-zinc-900">
                <div class="w-16 h-16 mx-auto mb-4 rounded-full flex items-center justify-center bg-zinc-900" >
                  <svg class="w-8 h-8 text-zinc-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
                  </svg>
                </div>
                <p class="text-zinc-400">No writing credits found</p>
              </div>
            {:else}
              <div class="space-y-3">
                {#each profile.credits.writers as writer}
                  <div
                    class="p-4 rounded-xl transition-all hover:bg-zinc-800"
                    style="background: #0a0a0c; border: 1px solid {writer.is_flagged ? '#ef4444' : '#3f3f46'};"
                  >
                    <div class="flex items-center justify-between">
                      <div class="flex items-center gap-3">
                        <div class="w-10 h-10 rounded-full flex items-center justify-center text-lg font-bold bg-zinc-800 text-zinc-400">
                          {writer.name.charAt(0)}
                        </div>
                        <div>
                          <div class="flex items-center gap-2">
                            <span class="font-medium text-white">{writer.name}</span>
                            {#if writer.is_flagged}
                              <span class="px-2 py-0.5 text-xs rounded-full bg-rose-900 text-rose-300">Flagged</span>
                            {/if}
                          </div>
                          {#if writer.note}
                            <p class="text-xs text-amber-400 mt-0.5">{writer.note}</p>
                          {/if}
                        </div>
                      </div>
                      <div class="text-right">
                        <span class="text-lg font-semibold text-zinc-200">{writer.track_count}</span>
                        <span class="text-sm text-zinc-400 ml-1">tracks</span>
                      </div>
                    </div>
                  </div>
                {/each}
              </div>
            {/if}
          </div>

          <!-- Producers Section -->
          <div>
            <h2 class="text-2xl font-bold text-white mb-6 flex items-center gap-3">
              <svg class="w-6 h-6 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3" />
              </svg>
              Producers
              {#if profile.credits?.producers?.length}
                <span class="text-zinc-400 font-normal text-lg">({profile.credits.producers.length})</span>
              {/if}
            </h2>

            {#if !profile.credits?.producers?.length}
              <div class="text-center py-12 rounded-2xl bg-zinc-900 border border-zinc-900">
                <div class="w-16 h-16 mx-auto mb-4 rounded-full flex items-center justify-center bg-zinc-900" >
                  <svg class="w-8 h-8 text-zinc-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3" />
                  </svg>
                </div>
                <p class="text-zinc-400">No production credits found</p>
              </div>
            {:else}
              <div class="space-y-3">
                {#each profile.credits.producers as producer}
                  <div
                    class="p-4 rounded-xl transition-all hover:bg-zinc-800"
                    style="background: #0a0a0c; border: 1px solid {producer.is_flagged ? '#ef4444' : '#3f3f46'};"
                  >
                    <div class="flex items-center justify-between">
                      <div class="flex items-center gap-3">
                        {#if producer.image_url}
                          <img src={producer.image_url} alt={producer.name} class="w-10 h-10 rounded-full object-cover" />
                        {:else}
                          <div class="w-10 h-10 rounded-full flex items-center justify-center text-lg font-bold" style="background: linear-gradient(135deg, #7C3AED, #A855F7); color: white;">
                            {producer.name.charAt(0)}
                          </div>
                        {/if}
                        <div>
                          <div class="flex items-center gap-2">
                            <span class="font-medium text-white">{producer.name}</span>
                            {#if producer.is_flagged}
                              <span class="px-2 py-0.5 text-xs rounded-full bg-rose-900 text-rose-300">Flagged</span>
                            {/if}
                          </div>
                          <p class="text-xs text-zinc-500 capitalize">{producer.role}</p>
                        </div>
                      </div>
                      <div class="text-right">
                        <span class="text-lg font-semibold text-zinc-200">{producer.track_count}</span>
                        <span class="text-sm text-zinc-400 ml-1">tracks</span>
                      </div>
                    </div>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        </div>

        <!-- Credits Info Box -->
        <div class="mt-8 p-6 rounded-2xl bg-gradient-deep border border-zinc-900">
          <h3 class="text-lg font-semibold text-white mb-3 flex items-center gap-2">
            <svg class="w-5 h-5 text-zinc-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            About Credits
          </h3>
          <p class="text-sm text-zinc-400 leading-relaxed">
            Credits show the songwriters and producers who have worked on this artist's music.
            <strong class="text-zinc-300">Flagged collaborators</strong> have their own documented offenses in our database.
            Working with a flagged collaborator does not imply the artist endorses or is aware of that person's misconduct.
          </p>
        </div>

      {:else if activeTab === 'connections'}
        <!-- Connections / Collaborators -->
        <div class="grid lg:grid-cols-3 gap-8">
          <div class="lg:col-span-2">
            <h2 class="text-2xl font-bold text-white mb-6">
              Collaborators & Connections
              {#if profile.collaborators.length > 0}
                <span class="text-zinc-400 font-normal ml-2">({profile.collaborators.length})</span>
              {/if}
            </h2>

            {#if profile.collaborators.length === 0}
              <div class="text-center py-16 rounded-2xl bg-zinc-900 border border-zinc-900">
                <div class="w-20 h-20 mx-auto mb-4 rounded-full flex items-center justify-center bg-zinc-900" >
                  <svg class="w-10 h-10 text-zinc-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                  </svg>
                </div>
                <p class="text-zinc-200 text-lg">No collaborations found</p>
                <p class="text-zinc-400 text-sm mt-2">Collaboration data is being populated</p>
              </div>
            {:else}
              <div class="grid grid-cols-2 sm:grid-cols-3 gap-4">
                {#each profile.collaborators as collab}
                  {@const collabStatus = getStatusColor(collab.status)}
                  <button
                    type="button"
                    class="group p-4 rounded-2xl text-left transition-all hover:scale-[1.02]"
                    style="background: #0a0a0c; border: 1px solid {collab.is_flagged ? '#ef4444' : '#3f3f46'};"
                    on:click={() => navigateToArtist(collab.id)}
                  >
                    <div class="relative aspect-square mb-3 rounded-xl overflow-hidden bg-gradient-panel-soft">
                      {#if collab.image_url}
                        <img src={collab.image_url} alt={collab.name} class="w-full h-full object-cover" />
                      {:else}
                        <div class="w-full h-full flex items-center justify-center text-3xl font-bold text-zinc-400">
                          {collab.name.charAt(0)}
                        </div>
                      {/if}

                      {#if collab.is_flagged}
                        <div class="absolute top-2 right-2 w-6 h-6 rounded-full flex items-center justify-center bg-rose-500">
                          <svg class="w-3 h-3 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                          </svg>
                        </div>
                      {/if}
                    </div>

                    <h4 class="font-medium text-white truncate">{collab.name}</h4>
                    <p class="text-xs text-zinc-400 capitalize">{collab.collaboration_type}</p>
                    <p class="text-xs text-zinc-400">{collab.collaboration_count} collaboration{collab.collaboration_count !== 1 ? 's' : ''}</p>
                  </button>
                {/each}
              </div>
            {/if}
          </div>

          <!-- Sidebar -->
          <div class="lg:col-span-1 space-y-4">
            <div class="rounded-3xl p-6 bg-gradient-deep shadow-deep">
              <h3 class="text-lg font-semibold mb-4 text-stone-50">Connection Warning</h3>
              <p class="text-sm leading-relaxed text-zinc-300">
                Connections shown represent professional collaborations only.
                <strong class="text-stone-50">Guilt is never transferred</strong> across connections.
                A collaboration with a flagged artist does not imply involvement in their misconduct.
              </p>
            </div>

            <button
              type="button"
              on:click={() => navigateTo('graph')}
              class="w-full px-4 py-3 rounded-2xl font-medium transition-all flex items-center justify-center gap-2 hover:bg-indigo-700 bg-indigo-600 text-white"
             
            >
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
              </svg>
              Explore Full Network
            </button>
          </div>
        </div>
      {/if}
    </main>

    <!-- Report Error Modal -->
    {#if showReportModal}
      <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/85">
        <div
          class="w-full max-w-md rounded-2xl p-6 bg-zinc-900 border border-zinc-900"
         
        >
          <div class="flex items-center justify-between mb-6">
            <h3 class="text-xl font-bold text-white">Report an Error</h3>
            <button
              type="button"
              on:click={() => showReportModal = false}
              class="p-2 rounded-lg text-zinc-400 hover:text-white hover:bg-zinc-700"
            >
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
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
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
          <img
            src={selectedAlbumCover.url}
            alt={selectedAlbumCover.name}
            class="max-w-full max-h-[70vh] rounded-xl shadow-2xl object-contain"
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
    color: #fda4af;
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
    padding-bottom: 0.5rem;
    align-self: stretch;
    justify-content: flex-end;
  }

  .profile__block-group {
    position: relative;
  }

  .profile__block-split {
    display: flex;
  }

  .profile__block-btn {
    padding: 0.75rem 1.5rem;
    border-radius: var(--radius-full) 0 0 var(--radius-full);
    font-weight: 700;
    font-size: var(--text-lg);
    transition: opacity var(--transition-fast);
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
    border: none;
  }
  .profile__block-btn:disabled { opacity: 0.5; }
  .profile__block-btn:hover:not(:disabled) { opacity: 0.9; }

  .profile__block-btn--unblocked {
    background: linear-gradient(135deg, #fb7185, #e11d48);
    color: white;
  }

  .profile__block-btn--blocked {
    background: rgba(20, 83, 45, 0.34);
    color: #bbf7d0;
    border: 2px solid rgba(74, 222, 128, 0.2);
    border-right: none;
  }

  .profile__block-dropdown {
    padding: 0.75rem;
    border-radius: var(--radius-full);
    font-weight: 700;
    transition: opacity var(--transition-fast);
    display: flex;
    align-items: center;
    cursor: pointer;
    border: none;
  }
  .profile__block-dropdown:hover { opacity: 0.9; }

  .profile__block-dropdown--unblocked {
    background: linear-gradient(135deg, #fb7185, #e11d48);
    color: white;
    border-left: 1px solid rgba(255, 255, 255, 0.2);
  }

  .profile__block-dropdown--blocked {
    background: rgba(20, 83, 45, 0.34);
    color: #bbf7d0;
    border: 2px solid rgba(74, 222, 128, 0.2);
    border-left: 1px solid rgba(74, 222, 128, 0.18);
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
    gap: 0.5rem;
  }

  .profile__action-btn {
    padding: 0.5rem 1rem;
    border-radius: 1rem;
    font-size: var(--text-sm);
    font-weight: 600;
    transition: background-color var(--transition-fast), border-color var(--transition-fast), transform var(--transition-fast);
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.25rem;
    border: none;
    cursor: pointer;
  }

  .profile__action-btn--primary {
    flex: 1;
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
    width: 1rem;
    height: 1rem;
  }

  /* ===== Tab Navigation ===== */
  .profile__tab-bar {
    max-width: 72rem;
    margin: 0 auto;
  }

  .profile__tab-bar-inner {
    padding-top: 0.75rem;
    padding-bottom: 0;
  }

  .profile__tab-group {
    width: 100%;
  }

  .profile__tab {
    flex: 1 1 0;
    white-space: nowrap;
  }

  /* ===== Main Content ===== */
  .profile__main {
    max-width: 72rem;
    margin: 0 auto;
    padding: 1.5rem;
  }

  @media (max-width: 900px) {
    .profile__header-row {
      flex-direction: column;
      align-items: flex-start;
    }

    .profile__actions {
      width: 100%;
    }

    .profile__secondary-actions {
      width: 100%;
      flex-wrap: wrap;
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

  @keyframes profile-spin {
    to { transform: rotate(360deg); }
  }
</style>
