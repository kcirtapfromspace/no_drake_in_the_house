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
  import { navigateTo } from '../utils/simple-router';
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
      const result = await apiClient.get<Array<{ artist_id: string }>>('/api/v1/dnp/list');
      if (result.success && result.data) {
        dnpList = new Set(result.data.map(item => item.artist_id));
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
    if (!evidence.length) return 'weak';
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

<div class="min-h-screen" style="background: #18181b;">
  {#if isLoading}
    <div class="flex items-center justify-center min-h-screen">
      <div class="text-center">
        <div class="w-12 h-12 border-4 border-rose-500 border-t-transparent rounded-full animate-spin mx-auto"></div>
        <p class="mt-4 text-zinc-300">Loading artist profile...</p>
      </div>
    </div>
  {:else if error}
    <div class="flex items-center justify-center min-h-screen">
      <div class="text-center max-w-md p-8 rounded-xl" style="background: #27272a; border: 1px solid #52525b;">
        <div class="text-6xl text-rose-500 mb-4">!</div>
        <h2 class="text-2xl font-bold text-white mb-2">Error Loading Profile</h2>
        <p class="text-zinc-300 mb-6">{error}</p>
        <button
          on:click={() => navigateTo('home')}
          class="px-6 py-3 bg-rose-600 hover:bg-rose-700 text-white rounded-lg font-medium transition-colors"
        >
          Go Back
        </button>
      </div>
    </div>
  {:else if profile}
    <!-- Clean Header Section -->
    <div class="pt-6 pb-8" style="background: #18181b;">
      <!-- Back Button -->
      <nav class="max-w-7xl mx-auto px-6 mb-6">
        <button
          type="button"
          on:click={() => navigateTo('home')}
          class="flex items-center gap-2 px-4 py-2 rounded-full text-zinc-300 hover:text-white transition-all hover:bg-zinc-800"
          style="border: 1px solid #3f3f46;"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
          </svg>
          Back
        </button>
      </nav>

      <!-- Artist Header -->
      <div class="max-w-7xl mx-auto px-6">
        <div class="flex items-start gap-6">
          <!-- Artist Photo -->
          {#if profile.primary_image?.url}
            <div class="flex-shrink-0">
              <div
                class="w-32 h-32 rounded-xl shadow-xl overflow-hidden border-2"
                style="border-color: {statusColor?.border || '#3f3f46'};"
              >
                <img
                  src={profile.primary_image.url}
                  alt={profile.canonical_name}
                  class="w-full h-full object-cover"
                />
              </div>
            </div>
          {:else}
            <div class="flex-shrink-0">
              <div
                class="w-32 h-32 rounded-xl shadow-xl flex items-center justify-center text-4xl font-bold"
                style="background: linear-gradient(135deg, #3f3f46, #27272a); color: #71717a; border: 2px solid #3f3f46;"
              >
                {profile.canonical_name.charAt(0)}
              </div>
            </div>
          {/if}

          <!-- Artist Info -->
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-3 mb-3">
              <!-- Status Badge - Made larger and more prominent -->
              <span
                class="px-4 py-2 rounded-full text-sm font-extrabold uppercase tracking-wider shadow-lg"
                style="background: {statusColor?.bg}; color: {statusColor?.text}; text-shadow: 0 1px 2px rgba(0,0,0,0.3);"
              >
                {statusLabel}
              </span>

              <!-- Confidence Level (only show for medium/high) -->
              {#if profile.confidence !== 'low'}
                <div class="flex items-center gap-2 px-2 py-1 rounded-full" style="background: #27272a; border: 1px solid #3f3f46;">
                  <div class="flex gap-0.5">
                    {#each [0, 1, 2] as i}
                      <div
                        class="w-1.5 h-3 rounded-sm"
                        style="background: {
                          (profile.confidence === 'high') ||
                          (profile.confidence === 'medium' && i < 2)
                            ? '#10B981' : '#52525b'
                        };"
                      ></div>
                    {/each}
                  </div>
                  <span class="text-xs text-zinc-300">{confidenceLabel}</span>
                </div>
              {/if}
            </div>

            <h1 class="text-3xl font-bold text-white mb-2">{profile.canonical_name}</h1>

            {#if profile.genres.length > 0}
              <p class="text-sm text-zinc-400 mb-2">{profile.genres.join(' • ')}</p>
            {/if}

            <!-- Social Media Links (for Drake showcase) -->
            {#if profile.canonical_name === 'Drake'}
            <div class="flex items-center gap-3 mb-3">
              <a href="https://instagram.com/champagnepapi" target="_blank" rel="noopener noreferrer"
                 class="w-8 h-8 rounded-full flex items-center justify-center transition-all hover:scale-110"
                 style="background: linear-gradient(45deg, #f09433, #e6683c, #dc2743, #cc2366, #bc1888);"
                 title="Instagram">
                <svg class="w-4 h-4 text-white" fill="currentColor" viewBox="0 0 24 24">
                  <path d="M12 2.163c3.204 0 3.584.012 4.85.07 3.252.148 4.771 1.691 4.919 4.919.058 1.265.069 1.645.069 4.849 0 3.205-.012 3.584-.069 4.849-.149 3.225-1.664 4.771-4.919 4.919-1.266.058-1.644.07-4.85.07-3.204 0-3.584-.012-4.849-.07-3.26-.149-4.771-1.699-4.919-4.92-.058-1.265-.07-1.644-.07-4.849 0-3.204.013-3.583.07-4.849.149-3.227 1.664-4.771 4.919-4.919 1.266-.057 1.645-.069 4.849-.069zm0-2.163c-3.259 0-3.667.014-4.947.072-4.358.2-6.78 2.618-6.98 6.98-.059 1.281-.073 1.689-.073 4.948 0 3.259.014 3.668.072 4.948.2 4.358 2.618 6.78 6.98 6.98 1.281.058 1.689.072 4.948.072 3.259 0 3.668-.014 4.948-.072 4.354-.2 6.782-2.618 6.979-6.98.059-1.28.073-1.689.073-4.948 0-3.259-.014-3.667-.072-4.947-.196-4.354-2.617-6.78-6.979-6.98-1.281-.059-1.69-.073-4.949-.073zm0 5.838c-3.403 0-6.162 2.759-6.162 6.162s2.759 6.163 6.162 6.163 6.162-2.759 6.162-6.163c0-3.403-2.759-6.162-6.162-6.162zm0 10.162c-2.209 0-4-1.79-4-4 0-2.209 1.791-4 4-4s4 1.791 4 4c0 2.21-1.791 4-4 4zm6.406-11.845c-.796 0-1.441.645-1.441 1.44s.645 1.44 1.441 1.44c.795 0 1.439-.645 1.439-1.44s-.644-1.44-1.439-1.44z"/>
                </svg>
              </a>
              <a href="https://twitter.com/Drake" target="_blank" rel="noopener noreferrer"
                 class="w-8 h-8 rounded-full flex items-center justify-center transition-all hover:scale-110"
                 style="background: #000000; border: 1px solid #333;"
                 title="X (Twitter)">
                <svg class="w-4 h-4 text-white" fill="currentColor" viewBox="0 0 24 24">
                  <path d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z"/>
                </svg>
              </a>
              <a href="https://open.spotify.com/artist/3TVXtAsR1Inumwj472S9r4" target="_blank" rel="noopener noreferrer"
                 class="w-8 h-8 rounded-full flex items-center justify-center transition-all hover:scale-110"
                 style="background: #1DB954;"
                 title="Spotify">
                <svg class="w-4 h-4 text-white" fill="currentColor" viewBox="0 0 24 24">
                  <path d="M12 0C5.4 0 0 5.4 0 12s5.4 12 12 12 12-5.4 12-12S18.66 0 12 0zm5.521 17.34c-.24.359-.66.48-1.021.24-2.82-1.74-6.36-2.101-10.561-1.141-.418.122-.779-.179-.899-.539-.12-.421.18-.78.54-.9 4.56-1.021 8.52-.6 11.64 1.32.42.18.479.659.301 1.02zm1.44-3.3c-.301.42-.841.6-1.262.3-3.239-1.98-8.159-2.58-11.939-1.38-.479.12-1.02-.12-1.14-.6-.12-.48.12-1.021.6-1.141C9.6 9.9 15 10.561 18.72 12.84c.361.181.54.78.241 1.2zm.12-3.36C15.24 8.4 8.82 8.16 5.16 9.301c-.6.179-1.2-.181-1.38-.721-.18-.601.18-1.2.72-1.381 4.26-1.26 11.28-1.02 15.721 1.621.539.3.719 1.02.419 1.56-.299.421-1.02.599-1.559.3z"/>
                </svg>
              </a>
              <a href="https://music.apple.com/artist/drake/271256" target="_blank" rel="noopener noreferrer"
                 class="w-8 h-8 rounded-full flex items-center justify-center transition-all hover:scale-110"
                 style="background: linear-gradient(180deg, #FA233B, #FB5C74);"
                 title="Apple Music">
                <svg class="w-4 h-4 text-white" fill="currentColor" viewBox="0 0 24 24">
                  <path d="M23.994 6.124a9.23 9.23 0 00-.24-2.19c-.317-1.31-1.062-2.31-2.18-3.043a5.022 5.022 0 00-1.877-.726 10.496 10.496 0 00-1.564-.15c-.04-.003-.083-.01-.124-.013H5.986c-.152.01-.303.017-.455.026-.747.043-1.49.123-2.193.4-1.336.53-2.3 1.452-2.865 2.78-.192.448-.292.925-.363 1.408-.056.392-.088.785-.1 1.18 0 .032-.007.062-.01.093v12.223c.01.14.017.283.027.424.05.815.154 1.624.497 2.373.65 1.42 1.738 2.353 3.234 2.8.42.127.856.187 1.293.228.555.053 1.11.06 1.667.06h11.03c.525 0 1.048-.034 1.57-.1.823-.106 1.597-.35 2.296-.81a5.046 5.046 0 001.88-2.207c.186-.42.293-.87.37-1.324.113-.675.138-1.358.137-2.04-.002-3.8 0-7.595-.003-11.393zm-6.423 3.99v5.712c0 .417-.058.827-.244 1.206-.29.59-.76.962-1.388 1.14-.35.1-.706.157-1.07.173-.95.042-1.785-.384-2.16-1.257-.238-.554-.255-1.134-.1-1.715.2-.752.676-1.25 1.4-1.53.317-.123.646-.21.98-.28.39-.083.784-.144 1.176-.22.227-.045.453-.1.673-.18.155-.056.253-.164.278-.327.014-.09.026-.18.026-.27v-5.162c0-.076-.016-.154-.04-.228-.02-.065-.065-.12-.132-.14-.06-.018-.123-.023-.188-.012-.16.027-.32.057-.477.097-.913.233-1.825.47-2.736.706l-3.47.896c-.357.093-.714.187-1.07.283-.19.052-.323.17-.37.364-.026.108-.038.22-.038.333-.002 2.255-.004 7.098-.006 9.353 0 .455-.053.904-.257 1.324-.27.556-.704.938-1.285 1.145-.42.15-.857.22-1.302.244-.58.03-1.14-.037-1.66-.305-.748-.385-1.13-1-1.16-1.838-.02-.57.09-1.1.403-1.574.376-.57.916-.912 1.575-1.07.377-.092.76-.16 1.145-.218.448-.067.897-.12 1.34-.206.247-.048.49-.116.7-.248.25-.156.357-.39.36-.687.002-.103.002-.207.002-.31V5.782c0-.127.014-.252.044-.376.068-.28.256-.468.522-.548.14-.042.286-.072.432-.1l1.61-.412c1.277-.327 2.555-.652 3.832-.98 1.09-.28 2.178-.56 3.268-.838.193-.05.39-.093.59-.108.132-.01.263.007.39.043.237.068.38.23.447.462.037.128.05.26.05.393v4.258z"/>
                </svg>
              </a>
              <span class="text-xs text-zinc-500 ml-2">87M followers</span>
            </div>
            {/if}

            {#if profile.last_reviewed}
              <p class="text-xs text-zinc-500">Last reviewed: {formatDate(profile.last_reviewed)}</p>
            {/if}
          </div>

            <!-- Action Buttons -->
            <div class="flex-shrink-0 flex flex-col gap-2 pb-2">
              <!-- Primary Block Button with Dropdown -->
              <div class="relative">
                <div class="flex">
                  <button
                    type="button"
                    on:click={toggleBlock}
                    disabled={isBlockingInProgress}
                    class="px-6 py-3 rounded-l-full font-bold text-lg transition-all disabled:opacity-50 hover:opacity-90 flex items-center gap-2"
                    style="{isBlocked
                      ? 'background: rgba(34, 197, 94, 0.2); color: #22C55E; border: 2px solid rgba(34, 197, 94, 0.4); border-right: none;'
                      : 'background: #DC2626; color: white; border-right: none;'}"
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
                    class="px-3 py-3 rounded-r-full font-bold transition-all hover:opacity-90 flex items-center"
                    style="{isBlocked
                      ? 'background: rgba(34, 197, 94, 0.2); color: #22C55E; border: 2px solid rgba(34, 197, 94, 0.4); border-left: 1px solid rgba(34, 197, 94, 0.3);'
                      : 'background: #DC2626; color: white; border-left: 1px solid rgba(255,255,255,0.2);'}"
                  >
                    <svg class="w-4 h-4 transition-transform {showBlockingOptions ? 'rotate-180' : ''}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                    </svg>
                  </button>
                </div>

                <!-- Blocking Options Dropdown -->
                {#if showBlockingOptions}
                  <div
                    class="absolute right-0 mt-2 w-64 rounded-xl shadow-xl z-50 overflow-hidden"
                    style="background: #27272a; border: 1px solid #3f3f46;"
                  >
                    <div class="p-2">
                      <div class="text-xs text-zinc-500 px-3 py-2 font-medium uppercase tracking-wider">Blocking Options</div>
                      <button
                        type="button"
                        on:click={() => { toggleRoleBlocking('main', true); showBlockingOptions = false; }}
                        class="w-full px-3 py-2 text-left text-sm text-zinc-200 hover:bg-zinc-700 rounded-lg flex items-center gap-3"
                      >
                        <svg class="w-4 h-4 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3" />
                        </svg>
                        Block All Main Tracks
                      </button>
                      <button
                        type="button"
                        on:click={() => { toggleRoleBlocking('featured', true); showBlockingOptions = false; }}
                        class="w-full px-3 py-2 text-left text-sm text-zinc-200 hover:bg-zinc-700 rounded-lg flex items-center gap-3"
                      >
                        <svg class="w-4 h-4 text-orange-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                        </svg>
                        Block Collaborations
                      </button>
                      <button
                        type="button"
                        on:click={() => { toggleRoleBlocking('producer', true); showBlockingOptions = false; }}
                        class="w-full px-3 py-2 text-left text-sm text-zinc-200 hover:bg-zinc-700 rounded-lg flex items-center gap-3"
                      >
                        <svg class="w-4 h-4 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z" />
                        </svg>
                        Block Producer Credits
                      </button>
                      <button
                        type="button"
                        on:click={() => { toggleRoleBlocking('writer', true); showBlockingOptions = false; }}
                        class="w-full px-3 py-2 text-left text-sm text-zinc-200 hover:bg-zinc-700 rounded-lg flex items-center gap-3"
                      >
                        <svg class="w-4 h-4 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
                        </svg>
                        Block Writer Credits
                      </button>
                      <div class="my-2 border-t border-zinc-700"></div>
                      <button
                        type="button"
                        on:click={() => {
                          toggleRoleBlocking('main', true);
                          toggleRoleBlocking('featured', true);
                          toggleRoleBlocking('producer', true);
                          toggleRoleBlocking('writer', true);
                          showBlockingOptions = false;
                        }}
                        class="w-full px-3 py-2 text-left text-sm text-red-400 hover:bg-red-900/20 rounded-lg flex items-center gap-3 font-medium"
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
              <div class="flex gap-2">
                <button
                  type="button"
                  on:click={() => activeTab = 'evidence'}
                  class="flex-1 px-4 py-2 rounded-lg text-sm font-medium transition-all flex items-center justify-center gap-1 hover:bg-zinc-600"
                  style="background: #3f3f46; color: white; border: 1px solid #52525b;"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                  </svg>
                  Evidence
                </button>
                <button
                  type="button"
                  on:click={() => showReportModal = true}
                  class="px-4 py-2 rounded-lg text-sm font-medium transition-all flex items-center justify-center gap-1 hover:bg-zinc-600"
                  style="background: #27272a; color: #a1a1aa; border: 1px solid #52525b;"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                  </svg>
                  Report
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

    <!-- Tab Navigation -->
    <div class="sticky top-0 z-40" style="background: #18181b;">
      <div class="max-w-7xl mx-auto px-6 py-3">
        <div class="inline-flex gap-2 p-1.5 rounded-xl" style="background: #27272a;">
          <button
            type="button"
            on:click={() => activeTab = 'evidence'}
            class="px-5 py-2.5 text-sm font-medium transition-all rounded-lg"
            style="background: {activeTab === 'evidence' ? '#3f3f46' : 'transparent'}; color: {activeTab === 'evidence' ? 'white' : '#a1a1aa'};"
          >
            Evidence
          </button>
          <button
            type="button"
            on:click={() => activeTab = 'catalog'}
            class="px-5 py-2.5 text-sm font-medium transition-all rounded-lg"
            style="background: {activeTab === 'catalog' ? '#3f3f46' : 'transparent'}; color: {activeTab === 'catalog' ? 'white' : '#a1a1aa'};"
          >
            Full Catalog
          </button>
          <button
            type="button"
            on:click={() => activeTab = 'discography'}
            class="px-5 py-2.5 text-sm font-medium transition-all rounded-lg"
            style="background: {activeTab === 'discography' ? '#3f3f46' : 'transparent'}; color: {activeTab === 'discography' ? 'white' : '#a1a1aa'};"
          >
            Revenue
          </button>
          <button
            type="button"
            on:click={() => activeTab = 'credits'}
            class="px-5 py-2.5 text-sm font-medium transition-all rounded-lg"
            style="background: {activeTab === 'credits' ? '#3f3f46' : 'transparent'}; color: {activeTab === 'credits' ? 'white' : '#a1a1aa'};"
          >
            Credits
          </button>
          <button
            type="button"
            on:click={() => activeTab = 'connections'}
            class="px-5 py-2.5 text-sm font-medium transition-all rounded-lg"
            style="background: {activeTab === 'connections' ? '#3f3f46' : 'transparent'}; color: {activeTab === 'connections' ? 'white' : '#a1a1aa'};"
          >
            Connections
          </button>
        </div>
      </div>
    </div>

    <!-- Main Content -->
    <main class="max-w-7xl mx-auto px-6 py-8">
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
              <div class="text-center py-16 rounded-2xl" style="background: #18181b; border: 1px solid #3f3f46;">
                <div class="w-20 h-20 mx-auto mb-4 rounded-full flex items-center justify-center" style="background: rgba(16, 185, 129, 0.2);">
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
                    class="rounded-2xl overflow-hidden transition-all"
                    style="background: #18181b; border: 1px solid {isExpanded ? catColor.icon : '#3f3f46'};"
                  >
                    <!-- Offense Header -->
                    <button
                      type="button"
                      on:click={() => expandedOffenseId = isExpanded ? null : offense.id}
                      class="w-full p-6 text-left"
                    >
                      <div class="flex items-start gap-4">
                        <!-- Timeline Indicator -->
                        <div class="flex-shrink-0 flex flex-col items-center">
                          <div
                            class="w-12 h-12 rounded-xl flex items-center justify-center"
                            style="background: {catColor.bg};"
                          >
                            <svg class="w-6 h-6" style="color: {catColor.icon};" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                            </svg>
                          </div>
                          {#if index < profile.offenses.length - 1}
                            <div class="w-0.5 h-full min-h-[20px] mt-2" style="background: #52525b;"></div>
                          {/if}
                        </div>

                        <!-- Offense Content -->
                        <div class="flex-1 min-w-0">
                          <div class="flex items-center flex-wrap gap-2 mb-2">
                            <!-- Category Badge -->
                            <span
                              class="px-3 py-1 text-xs font-medium rounded-full"
                              style="background: {catColor.bg}; color: {catColor.icon};"
                            >
                              {offense.category.name}
                            </span>

                            <!-- Procedural State Badge -->
                            <span class="px-3 py-1 text-xs font-medium rounded-full text-zinc-200" style="background: #3f3f46;">
                              {getProceduralStateLabel(offense.procedural_state)}
                            </span>

                            <!-- Evidence Strength -->
                            <span
                              class="px-3 py-1 text-xs font-medium rounded-full"
                              style="background: rgba({evidenceStrength.color === '#10B981' ? '16, 185, 129' : evidenceStrength.color === '#F59E0B' ? '245, 158, 11' : '239, 68, 68'}, 0.15); color: {evidenceStrength.color};"
                            >
                              {evidenceStrength.label} Evidence
                            </span>
                          </div>

                          <h3 class="text-xl font-semibold text-white mb-3">{offense.title}</h3>
                          <p class="text-zinc-300 leading-relaxed line-clamp-2">{offense.description}</p>

                          {#if offense.incident_date}
                            <p class="text-sm text-zinc-400 mt-3 flex items-center gap-2">
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
                      <div class="px-6 pb-6" style="border-top: 1px solid #52525b;">
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
                                <a
                                  href={evidence.source.url}
                                  target="_blank"
                                  rel="noopener noreferrer"
                                  class="flex items-start gap-4 p-4 rounded-lg transition-all hover:bg-zinc-600"
                                  style="background: #3f3f46; border: 1px solid #52525b;"
                                >
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
            <div class="rounded-3xl p-6" style="background: linear-gradient(145deg, #1f1f23, #18181b); border: none; box-shadow: 0 4px 20px rgba(0,0,0,0.3);">
              <h3 class="text-lg font-semibold mb-5" style="color: #fafafa;">Source Reliability</h3>
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
            <div class="rounded-3xl p-6" style="background: linear-gradient(145deg, #1f1f23, #18181b); border: none; box-shadow: 0 4px 20px rgba(0,0,0,0.3);">
              <h3 class="text-lg font-semibold mb-5" style="color: #fafafa;">Profile Summary</h3>
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

            <!-- Recent News -->
            {#if profile.canonical_name === 'Drake'}
            <div class="rounded-3xl p-6" style="background: linear-gradient(145deg, #1f1f23, #18181b); border: none; box-shadow: 0 4px 20px rgba(0,0,0,0.3);">
              <h3 class="text-lg font-semibold mb-5 flex items-center gap-2" style="color: #fafafa;">
                <svg class="w-5 h-5 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 20H5a2 2 0 01-2-2V6a2 2 0 012-2h10a2 2 0 012 2v1m2 13a2 2 0 01-2-2V7m2 13a2 2 0 002-2V9a2 2 0 00-2-2h-2m-4-3H9M7 16h6M7 8h6v4H7V8z" />
                </svg>
                Recent Headlines
              </h3>
              <div class="space-y-4">
                <a href="#" class="block group">
                  <p class="text-sm text-zinc-200 group-hover:text-white transition-colors line-clamp-2">Drake and Kendrick Lamar feud escalates with diss tracks</p>
                  <p class="text-xs text-zinc-500 mt-1">Rolling Stone - 2024</p>
                </a>
                <a href="#" class="block group">
                  <p class="text-sm text-zinc-200 group-hover:text-white transition-colors line-clamp-2">Drake sued for $10M over unauthorized sample</p>
                  <p class="text-xs text-zinc-500 mt-1">Billboard - 2024</p>
                </a>
                <a href="#" class="block group">
                  <p class="text-sm text-zinc-200 group-hover:text-white transition-colors line-clamp-2">Former associate alleges financial impropriety</p>
                  <p class="text-xs text-zinc-500 mt-1">Complex - 2023</p>
                </a>
                <a href="#" class="block group">
                  <p class="text-sm text-zinc-200 group-hover:text-white transition-colors line-clamp-2">Questions raised about relationship with underage artists</p>
                  <p class="text-xs text-zinc-500 mt-1">Pitchfork - 2022</p>
                </a>
              </div>
            </div>
            {/if}
          </div>
        </div>

      {:else if activeTab === 'catalog'}
        <!-- Full Artist Catalog - All Appearances -->
        <div class="space-y-6">
          <!-- Catalog Summary -->
          <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
            <button
              type="button"
              on:click={() => toggleRoleBlocking('main', catalog.filter(t => t.role === 'main' && t.isBlocked).length < catalog.filter(t => t.role === 'main').length)}
              class="p-4 rounded-xl text-left transition-all hover:scale-[1.02]"
              style="background: #27272a; border: 1px solid #3f3f46;"
            >
              <div class="flex items-center justify-between">
                <div class="text-2xl font-bold text-white">{catalog.filter(t => t.role === 'main').length}</div>
                <div class="w-8 h-5 rounded-full transition-all {catalog.filter(t => t.role === 'main' && t.isBlocked).length === catalog.filter(t => t.role === 'main').length ? 'bg-green-500' : 'bg-zinc-600'}">
                  <div class="w-4 h-4 mt-0.5 rounded-full bg-white shadow transition-all {catalog.filter(t => t.role === 'main' && t.isBlocked).length === catalog.filter(t => t.role === 'main').length ? 'ml-3.5' : 'ml-0.5'}"></div>
                </div>
              </div>
              <div class="text-sm text-zinc-400">Main Artist</div>
              <div class="text-xs text-green-400 mt-1">{catalog.filter(t => t.role === 'main' && t.isBlocked).length} blocked</div>
            </button>
            <button
              type="button"
              on:click={() => toggleRoleBlocking('featured', catalog.filter(t => t.role === 'featured' && t.isBlocked).length < catalog.filter(t => t.role === 'featured').length)}
              class="p-4 rounded-xl text-left transition-all hover:scale-[1.02]"
              style="background: #27272a; border: 1px solid #3f3f46;"
            >
              <div class="flex items-center justify-between">
                <div class="text-2xl font-bold text-white">{catalog.filter(t => t.role === 'featured').length}</div>
                <div class="w-8 h-5 rounded-full transition-all {catalog.filter(t => t.role === 'featured' && t.isBlocked).length === catalog.filter(t => t.role === 'featured').length ? 'bg-green-500' : 'bg-zinc-600'}">
                  <div class="w-4 h-4 mt-0.5 rounded-full bg-white shadow transition-all {catalog.filter(t => t.role === 'featured' && t.isBlocked).length === catalog.filter(t => t.role === 'featured').length ? 'ml-3.5' : 'ml-0.5'}"></div>
                </div>
              </div>
              <div class="text-sm text-zinc-400">Featured On</div>
              <div class="text-xs text-green-400 mt-1">{catalog.filter(t => t.role === 'featured' && t.isBlocked).length} blocked</div>
            </button>
            <button
              type="button"
              on:click={() => toggleRoleBlocking('producer', catalog.filter(t => t.role === 'producer' && t.isBlocked).length < catalog.filter(t => t.role === 'producer').length)}
              class="p-4 rounded-xl text-left transition-all hover:scale-[1.02]"
              style="background: #27272a; border: 1px solid #3f3f46;"
            >
              <div class="flex items-center justify-between">
                <div class="text-2xl font-bold text-white">{catalog.filter(t => t.role === 'producer').length}</div>
                <div class="w-8 h-5 rounded-full transition-all {catalog.filter(t => t.role === 'producer' && t.isBlocked).length === catalog.filter(t => t.role === 'producer').length ? 'bg-green-500' : 'bg-zinc-600'}">
                  <div class="w-4 h-4 mt-0.5 rounded-full bg-white shadow transition-all {catalog.filter(t => t.role === 'producer' && t.isBlocked).length === catalog.filter(t => t.role === 'producer').length ? 'ml-3.5' : 'ml-0.5'}"></div>
                </div>
              </div>
              <div class="text-sm text-zinc-400">Producer</div>
              <div class="text-xs text-amber-400 mt-1">{catalog.filter(t => t.role === 'producer' && t.isBlocked).length} blocked</div>
            </button>
            <button
              type="button"
              on:click={() => toggleRoleBlocking('writer', catalog.filter(t => t.role === 'writer' && t.isBlocked).length < catalog.filter(t => t.role === 'writer').length)}
              class="p-4 rounded-xl text-left transition-all hover:scale-[1.02]"
              style="background: #27272a; border: 1px solid #3f3f46;"
            >
              <div class="flex items-center justify-between">
                <div class="text-2xl font-bold text-white">{catalog.filter(t => t.role === 'writer').length}</div>
                <div class="w-8 h-5 rounded-full transition-all {catalog.filter(t => t.role === 'writer' && t.isBlocked).length === catalog.filter(t => t.role === 'writer').length ? 'bg-green-500' : 'bg-zinc-600'}">
                  <div class="w-4 h-4 mt-0.5 rounded-full bg-white shadow transition-all {catalog.filter(t => t.role === 'writer' && t.isBlocked).length === catalog.filter(t => t.role === 'writer').length ? 'ml-3.5' : 'ml-0.5'}"></div>
                </div>
              </div>
              <div class="text-sm text-zinc-400">Writer</div>
              <div class="text-xs text-amber-400 mt-1">{catalog.filter(t => t.role === 'writer' && t.isBlocked).length} blocked</div>
            </button>
          </div>

          <!-- Blocking Progress & Filter -->
          <div class="p-4 rounded-xl" style="background: linear-gradient(135deg, #27272a, #18181b); border: 1px solid #3f3f46;">
            <div class="flex items-center justify-between mb-3">
              <div class="flex items-center gap-3">
                <span class="text-sm font-medium text-zinc-300">Catalog Blocking Status</span>
                <div class="flex gap-1">
                  <button
                    type="button"
                    on:click={() => catalogFilter = 'all'}
                    class="px-2 py-1 text-xs rounded transition-all {catalogFilter === 'all' ? 'bg-zinc-600 text-white' : 'bg-zinc-800 text-zinc-400 hover:bg-zinc-700'}"
                  >All</button>
                  <button
                    type="button"
                    on:click={() => catalogFilter = 'blocked'}
                    class="px-2 py-1 text-xs rounded transition-all {catalogFilter === 'blocked' ? 'bg-green-600 text-white' : 'bg-zinc-800 text-zinc-400 hover:bg-zinc-700'}"
                  >Blocked</button>
                  <button
                    type="button"
                    on:click={() => catalogFilter = 'unblocked'}
                    class="px-2 py-1 text-xs rounded transition-all {catalogFilter === 'unblocked' ? 'bg-red-600 text-white' : 'bg-zinc-800 text-zinc-400 hover:bg-zinc-700'}"
                  >Unblocked</button>
                </div>
              </div>
              <span class="text-sm text-green-400">{catalog.filter(t => t.isBlocked).length} / {catalog.length} blocked</span>
            </div>
            <div class="h-2 rounded-full overflow-hidden" style="background: #3f3f46;">
              <div
                class="h-full rounded-full transition-all"
                style="width: {(catalog.filter(t => t.isBlocked).length / catalog.length * 100)}%; background: linear-gradient(90deg, #22C55E, #16A34A);"
              ></div>
            </div>
            <p class="text-xs text-zinc-500 mt-2">
              Click category cards above to toggle all tracks in that category. Click individual track toggles below for granular control.
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
                <div class="rounded-xl overflow-hidden" style="background: #27272a; border: 1px solid #3f3f46;">
                  <!-- Album Header (clickable) -->
                  <button
                    type="button"
                    class="w-full p-4 flex items-center gap-4 hover:bg-zinc-700/50 transition-colors text-left"
                    on:click={() => toggleCatalogAlbum(album.name)}
                  >
                    <!-- Block Icon (left side) - Prohibition Sign with clear color states -->
                    <button
                      type="button"
                      on:click|stopPropagation={() => toggleAlbumBlocking(album.name, album.blockedCount < album.totalCount)}
                      class="flex-shrink-0 transition-all hover:scale-110 p-1 rounded-full"
                      style="background: {album.blockedCount === album.totalCount ? 'rgba(239, 68, 68, 0.15)' : 'rgba(34, 197, 94, 0.15)'};"
                      title={album.blockedCount === album.totalCount ? 'Unblock all tracks' : 'Block all tracks'}
                    >
                      {#if album.blockedCount === album.totalCount}
                        <!-- Blocked state: Red prohibition sign -->
                        <svg class="w-6 h-6 text-red-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                          <circle cx="12" cy="12" r="9" />
                          <path d="M5.5 5.5l13 13" />
                        </svg>
                      {:else}
                        <!-- Unblocked state: Green checkmark circle -->
                        <svg class="w-6 h-6 text-green-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                          <circle cx="12" cy="12" r="9" />
                          <path d="M9 12l2 2 4-4" stroke-linecap="round" stroke-linejoin="round" />
                        </svg>
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
                        class="w-10 h-10 rounded object-cover bg-zinc-700 transition-all group-hover:ring-2 group-hover:ring-white/30"
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
                      <div class="font-semibold text-zinc-100 truncate">{album.name}</div>
                      <div class="text-sm text-zinc-400">{album.year} · {album.totalCount} tracks</div>
                      <div class="flex items-center gap-2 mt-1">
                        <div class="h-1.5 flex-1 max-w-32 rounded-full overflow-hidden bg-zinc-700">
                          <div
                            class="h-full rounded-full transition-all"
                            style="width: {(album.blockedCount / album.totalCount * 100)}%; background: linear-gradient(90deg, #22C55E, #16A34A);"
                          ></div>
                        </div>
                        <span class="text-xs text-green-400">{album.blockedCount}/{album.totalCount}</span>
                      </div>
                    </div>

                    <!-- Expand/Collapse Icon (plus/minus) -->
                    <div class="w-6 h-6 rounded-full border border-zinc-600 flex items-center justify-center flex-shrink-0">
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
                    <div class="border-t border-zinc-700">
                      <table class="w-full text-sm">
                        <thead>
                          <tr class="text-left text-zinc-500 bg-zinc-800/50">
                            <th class="py-2 px-4 font-medium w-12"></th>
                            <th class="py-2 px-4 font-medium w-10">#</th>
                            <th class="py-2 px-4 font-medium">Title</th>
                            <th class="py-2 px-4 font-medium text-right">Duration</th>
                          </tr>
                        </thead>
                        <tbody>
                          {#each album.tracks as track, idx}
                            <tr class="border-t border-zinc-800/50 hover:bg-zinc-800/30">
                              <td class="py-2 px-4">
                                <button
                                  type="button"
                                  on:click|stopPropagation={() => toggleTrackBlock(track.id)}
                                  class="transition-all hover:scale-110 p-0.5 rounded-full"
                                  style="background: {track.isBlocked ? 'rgba(239, 68, 68, 0.15)' : 'rgba(34, 197, 94, 0.15)'};"
                                  title={track.isBlocked ? 'Click to unblock' : 'Click to block'}
                                >
                                  {#if track.isBlocked}
                                    <!-- Blocked: Red prohibition sign -->
                                    <svg class="w-5 h-5 text-red-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                                      <circle cx="12" cy="12" r="9" />
                                      <path d="M5.5 5.5l13 13" />
                                    </svg>
                                  {:else}
                                    <!-- Unblocked: Green checkmark -->
                                    <svg class="w-5 h-5 text-green-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                                      <circle cx="12" cy="12" r="9" />
                                      <path d="M9 12l2 2 4-4" stroke-linecap="round" stroke-linejoin="round" />
                                    </svg>
                                  {/if}
                                </button>
                              </td>
                              <td class="py-2 px-4 text-zinc-500">{idx + 1}</td>
                              <td class="py-2 px-4">
                                <div class="font-medium {track.isBlocked ? 'text-zinc-200' : 'text-zinc-500'}">{track.title}</div>
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
                  <tr class="text-left text-zinc-500 border-b border-zinc-800">
                    <th class="pb-2 font-medium w-12"></th>
                    <th class="pb-2 font-medium">Title</th>
                    <th class="pb-2 font-medium">Main Artist</th>
                    <th class="pb-2 font-medium">Album</th>
                    <th class="pb-2 font-medium text-center">Year</th>
                  </tr>
                </thead>
                <tbody>
                  {#each filteredCatalog.filter(t => t.role === 'featured') as track}
                    <tr class="border-b border-zinc-800/50 hover:bg-zinc-800/30">
                      <td class="py-3">
                        <button
                          type="button"
                          on:click|stopPropagation={() => toggleTrackBlock(track.id)}
                          class="transition-all hover:scale-110"
                          title={track.isBlocked ? 'Click to unblock' : 'Click to block'}
                        >
                          <svg class="w-5 h-5 {track.isBlocked ? 'text-red-500' : 'text-zinc-600'}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <circle cx="12" cy="12" r="9" />
                            <path d="M5.5 5.5l13 13" />
                          </svg>
                        </button>
                      </td>
                      <td class="py-3">
                        <div class="font-medium {track.isBlocked ? 'text-zinc-200' : 'text-zinc-500'}">{track.title}</div>
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
                    <tr class="text-left text-zinc-500 border-b border-zinc-800">
                      <th class="pb-2 font-medium w-12"></th>
                      <th class="pb-2 font-medium">Title</th>
                      <th class="pb-2 font-medium">Role</th>
                      <th class="pb-2 font-medium">Artist</th>
                      <th class="pb-2 font-medium text-center">Year</th>
                    </tr>
                  </thead>
                  <tbody>
                    {#each filteredCatalog.filter(t => t.role === 'producer' || t.role === 'writer') as track}
                      <tr class="border-b border-zinc-800/50 hover:bg-zinc-800/30">
                        <td class="py-3">
                          <button
                            type="button"
                            on:click|stopPropagation={() => toggleTrackBlock(track.id)}
                            class="transition-all hover:scale-110"
                            title={track.isBlocked ? 'Click to unblock' : 'Click to block'}
                          >
                            <svg class="w-5 h-5 {track.isBlocked ? 'text-red-500' : 'text-zinc-600'}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                              <circle cx="12" cy="12" r="9" />
                              <path d="M5.5 5.5l13 13" />
                            </svg>
                          </button>
                        </td>
                        <td class="py-3">
                          <div class="font-medium {track.isBlocked ? 'text-zinc-200' : 'text-zinc-500'}">{track.title}</div>
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
          <div class="p-4 rounded-xl" style="background: #18181b; border: 1px solid #3f3f46;">
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
              <h3 class="text-lg font-semibold text-zinc-100 mb-6">Platform Distribution</h3>
              <div class="space-y-5">
                {#each profile.streaming_metrics.platform_breakdown as platform}
                  <div>
                    <div class="flex justify-between text-sm mb-2">
                      <span class="text-zinc-200 capitalize font-medium">{platform.platform}</span>
                      <span class="text-zinc-300">{formatNumber(platform.streams)} ({platform.percentage}%)</span>
                    </div>
                    <div class="h-3 rounded-full overflow-hidden bg-zinc-800">
                      <div
                        class="h-full rounded-full transition-all"
                        style="width: {platform.percentage}%; background: {
                          platform.platform === 'spotify' ? '#1DB954' :
                          platform.platform === 'apple' ? '#FC3C44' :
                          platform.platform === 'youtube' ? '#FF0000' :
                          '#3B82F6'
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
              <h3 class="text-lg font-semibold text-zinc-100 mb-6">Top Tracks</h3>
              <div class="space-y-3">
                {#each profile.streaming_metrics.top_tracks.slice(0, 5) as track, index}
                  <div class="flex items-center gap-4 p-4 rounded-xl bg-zinc-800/30 hover:bg-zinc-800/60 transition-colors">
                    <span class="text-2xl font-bold text-zinc-500 w-8 text-center">{index + 1}</span>
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
              <div class="text-center py-12 rounded-2xl" style="background: #18181b; border: 1px solid #3f3f46;">
                <div class="w-16 h-16 mx-auto mb-4 rounded-full flex items-center justify-center" style="background: #27272a;">
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
                    class="p-4 rounded-xl transition-all hover:bg-zinc-700"
                    style="background: #18181b; border: 1px solid {writer.is_flagged ? '#EF4444' : '#3f3f46'};"
                  >
                    <div class="flex items-center justify-between">
                      <div class="flex items-center gap-3">
                        <div class="w-10 h-10 rounded-full flex items-center justify-center text-lg font-bold" style="background: #3f3f46; color: #a1a1aa;">
                          {writer.name.charAt(0)}
                        </div>
                        <div>
                          <div class="flex items-center gap-2">
                            <span class="font-medium text-white">{writer.name}</span>
                            {#if writer.is_flagged}
                              <span class="px-2 py-0.5 text-xs rounded-full bg-red-900 text-red-300">Flagged</span>
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
              <div class="text-center py-12 rounded-2xl" style="background: #18181b; border: 1px solid #3f3f46;">
                <div class="w-16 h-16 mx-auto mb-4 rounded-full flex items-center justify-center" style="background: #27272a;">
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
                    class="p-4 rounded-xl transition-all hover:bg-zinc-700"
                    style="background: #18181b; border: 1px solid {producer.is_flagged ? '#EF4444' : '#3f3f46'};"
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
                              <span class="px-2 py-0.5 text-xs rounded-full bg-red-900 text-red-300">Flagged</span>
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
        <div class="mt-8 p-6 rounded-2xl" style="background: linear-gradient(145deg, #1f1f23, #18181b); border: 1px solid #3f3f46;">
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
              <div class="text-center py-16 rounded-2xl" style="background: #18181b; border: 1px solid #3f3f46;">
                <div class="w-20 h-20 mx-auto mb-4 rounded-full flex items-center justify-center" style="background: #27272a;">
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
                    style="background: #18181b; border: 1px solid {collab.is_flagged ? '#EF4444' : '#3f3f46'};"
                    on:click={() => {
                      // Navigate to collaborator profile
                      window.location.href = `#artist/${collab.id}`;
                      artistId = collab.id;
                      loadArtist();
                    }}
                  >
                    <div class="relative aspect-square mb-3 rounded-xl overflow-hidden" style="background: linear-gradient(135deg, #3f3f46, #27272a);">
                      {#if collab.image_url}
                        <img src={collab.image_url} alt={collab.name} class="w-full h-full object-cover" />
                      {:else}
                        <div class="w-full h-full flex items-center justify-center text-3xl font-bold text-zinc-400">
                          {collab.name.charAt(0)}
                        </div>
                      {/if}

                      {#if collab.is_flagged}
                        <div class="absolute top-2 right-2 w-6 h-6 rounded-full flex items-center justify-center bg-red-500">
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
            <div class="rounded-3xl p-6" style="background: linear-gradient(145deg, #1f1f23, #18181b); box-shadow: 0 4px 20px rgba(0,0,0,0.3);">
              <h3 class="text-lg font-semibold mb-4" style="color: #fafafa;">Connection Warning</h3>
              <p class="text-sm leading-relaxed" style="color: #d4d4d8;">
                Connections shown represent professional collaborations only.
                <strong style="color: #fafafa;">Guilt is never transferred</strong> across connections.
                A collaboration with a flagged artist does not imply involvement in their misconduct.
              </p>
            </div>

            <button
              type="button"
              on:click={() => navigateTo('graph')}
              class="w-full px-4 py-3 rounded-2xl font-medium transition-all flex items-center justify-center gap-2 hover:bg-blue-600"
              style="background: #3B82F6; color: white;"
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
      <div class="fixed inset-0 z-50 flex items-center justify-center p-4" style="background: rgba(0,0,0,0.85);">
        <div
          class="w-full max-w-md rounded-2xl p-6"
          style="background: #18181b; border: 1px solid #3f3f46;"
        >
          <div class="flex items-center justify-between mb-6">
            <h3 class="text-xl font-bold text-white">Report an Error</h3>
            <button
              type="button"
              on:click={() => showReportModal = false}
              class="p-2 rounded-lg text-zinc-400 hover:text-white hover:bg-zinc-600"
            >
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          <div class="space-y-4">
            <div>
              <label for="error-type" class="block text-sm font-medium text-zinc-300 mb-2">Error Type</label>
              <select
                id="error-type"
                bind:value={reportCategory}
                class="w-full px-4 py-3 rounded-lg text-white"
                style="background: #3f3f46; border: 1px solid #52525b;"
              >
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
              <textarea
                id="error-description"
                bind:value={reportDescription}
                rows="4"
                placeholder="Describe the error and provide any supporting information..."
                class="w-full px-4 py-3 rounded-lg text-white placeholder-zinc-500 resize-none"
                style="background: #3f3f46; border: 1px solid #52525b;"
              ></textarea>
            </div>

            <div class="flex gap-3 pt-2">
              <button
                type="button"
                on:click={() => showReportModal = false}
                class="flex-1 px-4 py-3 rounded-lg font-medium transition-colors hover:bg-zinc-600"
                style="background: #3f3f46; color: white; border: 1px solid #52525b;"
              >
                Cancel
              </button>
              <button
                type="button"
                on:click={submitReport}
                disabled={!reportDescription.trim()}
                class="flex-1 px-4 py-3 rounded-lg font-medium transition-colors disabled:opacity-50 hover:bg-rose-700"
                style="background: #e11d48; color: white;"
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
        class="fixed inset-0 z-50 flex items-center justify-center p-4"
        style="background: rgba(0,0,0,0.9);"
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
            class="absolute -top-12 right-0 p-2 rounded-lg text-zinc-400 hover:text-white hover:bg-zinc-700 transition-colors"
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
  .line-clamp-2 {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
</style>
