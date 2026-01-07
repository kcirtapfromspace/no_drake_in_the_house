<script lang="ts">
  import { onMount } from 'svelte';
  import { navigateTo } from '../utils/simple-router';
  import { libraryActions, type ImportTrack } from '../stores/library';

  let activeTab: 'services' | 'import' | 'extension' = 'services';
  let importFile: File | null = null;
  let isImporting = false;
  let importSuccess = false;
  let importError = '';
  let importedCount = 0;
  let detectedProvider = '';
  let parsedTracks: ImportTrack[] = [];
  let parseStep: 'idle' | 'parsing' | 'ready' | 'importing' | 'done' = 'idle';

  // Connection states
  let connections = {
    lastfm: { connected: false, username: '' },
    youtube: { connected: false, email: '' },
    deezer: { connected: false, username: '' }
  };

  async function connectLastFm() {
    const username = prompt('Enter your Last.fm username:');
    if (username) {
      connections.lastfm = { connected: true, username };
    }
  }

  async function connectYouTube() {
    alert('YouTube Music connection coming soon! We use the YouTube Data API.');
  }

  async function connectDeezer() {
    alert('Deezer connection coming soon!');
  }

  function handleFileSelect(event: Event) {
    const target = event.target as HTMLInputElement;
    if (target.files?.length) {
      importFile = target.files[0];
      parseStep = 'idle';
      importError = '';
      parsedTracks = [];
      detectedProvider = '';
    }
  }

  // Parse Spotify streaming history JSON
  function parseSpotifyStreamingHistory(data: any[]): ImportTrack[] {
    const trackMap = new Map<string, ImportTrack>();

    for (const item of data) {
      const key = `${item.artistName || item.master_metadata_album_artist_name}-${item.trackName || item.master_metadata_track_name}`;
      if (!trackMap.has(key)) {
        trackMap.set(key, {
          provider_track_id: key,
          track_name: item.trackName || item.master_metadata_track_name || 'Unknown',
          album_name: item.master_metadata_album_album_name,
          artist_name: item.artistName || item.master_metadata_album_artist_name || 'Unknown',
          source_type: 'streaming_history',
          added_at: item.endTime || item.ts
        });
      }
    }

    return Array.from(trackMap.values());
  }

  // Parse Spotify library JSON (YourLibrary.json format)
  function parseSpotifyLibrary(data: any): ImportTrack[] {
    const tracks: ImportTrack[] = [];

    // Handle tracks array
    if (data.tracks) {
      for (const track of data.tracks) {
        tracks.push({
          provider_track_id: track.track?.uri || `${track.artist}-${track.track}`,
          track_name: track.track || 'Unknown',
          album_name: track.album,
          artist_name: track.artist || 'Unknown',
          source_type: 'saved_tracks'
        });
      }
    }

    // Handle albums array
    if (data.albums) {
      for (const album of data.albums) {
        tracks.push({
          provider_track_id: album.uri || `album-${album.artist}-${album.album}`,
          track_name: album.album || 'Unknown Album',
          album_name: album.album,
          artist_name: album.artist || 'Unknown',
          source_type: 'saved_albums'
        });
      }
    }

    // Handle artists array (treat as followed artists)
    if (data.artists) {
      for (const artist of data.artists) {
        tracks.push({
          provider_track_id: artist.uri || `artist-${artist.name}`,
          track_name: `[Artist: ${artist.name}]`,
          artist_name: artist.name || 'Unknown',
          source_type: 'followed_artists'
        });
      }
    }

    return tracks;
  }

  // Parse Apple Music CSV
  function parseAppleMusicCSV(csvText: string): ImportTrack[] {
    const lines = csvText.split('\n');
    if (lines.length < 2) return [];

    // Find headers
    const headers = lines[0].split(',').map(h => h.trim().toLowerCase().replace(/"/g, ''));
    const trackNameIdx = headers.findIndex(h => h.includes('song') || h.includes('track') || h.includes('title'));
    const artistIdx = headers.findIndex(h => h.includes('artist'));
    const albumIdx = headers.findIndex(h => h.includes('album'));
    const dateIdx = headers.findIndex(h => h.includes('date') || h.includes('added'));

    const tracks: ImportTrack[] = [];

    for (let i = 1; i < lines.length; i++) {
      const line = lines[i].trim();
      if (!line) continue;

      // Simple CSV parsing (handles quoted fields)
      const values = parseCSVLine(line);

      const trackName = values[trackNameIdx] || 'Unknown';
      const artistName = values[artistIdx] || 'Unknown';
      const albumName = values[albumIdx];
      const addedAt = values[dateIdx];

      if (trackName && artistName) {
        tracks.push({
          provider_track_id: `apple-${artistName}-${trackName}`.replace(/\s+/g, '-').toLowerCase(),
          track_name: trackName,
          album_name: albumName,
          artist_name: artistName,
          source_type: 'library',
          added_at: addedAt
        });
      }
    }

    return tracks;
  }

  // Helper to parse CSV line with quoted fields
  function parseCSVLine(line: string): string[] {
    const result: string[] = [];
    let current = '';
    let inQuotes = false;

    for (let i = 0; i < line.length; i++) {
      const char = line[i];
      if (char === '"') {
        inQuotes = !inQuotes;
      } else if (char === ',' && !inQuotes) {
        result.push(current.trim());
        current = '';
      } else {
        current += char;
      }
    }
    result.push(current.trim());

    return result.map(v => v.replace(/^"|"$/g, ''));
  }

  // Detect file type and parse
  async function parseFile() {
    if (!importFile) return;

    parseStep = 'parsing';
    importError = '';

    try {
      const fileName = importFile.name.toLowerCase();
      const fileText = await importFile.text();

      if (fileName.endsWith('.json')) {
        const data = JSON.parse(fileText);

        // Detect Spotify format
        if (Array.isArray(data)) {
          // StreamingHistory format
          if (data[0]?.endTime || data[0]?.ts || data[0]?.artistName || data[0]?.master_metadata_album_artist_name) {
            detectedProvider = 'spotify';
            parsedTracks = parseSpotifyStreamingHistory(data);
          }
        } else if (data.tracks || data.albums || data.artists) {
          // YourLibrary.json format
          detectedProvider = 'spotify';
          parsedTracks = parseSpotifyLibrary(data);
        } else {
          throw new Error('Unrecognized JSON format. Please upload a Spotify export file.');
        }

      } else if (fileName.endsWith('.csv')) {
        // Assume Apple Music CSV
        detectedProvider = 'apple_music';
        parsedTracks = parseAppleMusicCSV(fileText);

      } else if (fileName.endsWith('.zip')) {
        importError = 'Please extract the ZIP file first and upload the JSON or CSV file inside.';
        parseStep = 'idle';
        return;

      } else {
        throw new Error('Unsupported file format. Please upload a JSON or CSV file.');
      }

      if (parsedTracks.length === 0) {
        throw new Error('No tracks found in the file. Please check the file format.');
      }

      parseStep = 'ready';

    } catch (e: any) {
      importError = e.message || 'Failed to parse file';
      parseStep = 'idle';
    }
  }

  async function importLibrary() {
    if (parsedTracks.length === 0) return;

    parseStep = 'importing';
    importError = '';

    try {
      const result = await libraryActions.importLibrary(detectedProvider, parsedTracks);

      if (result) {
        importedCount = result.imported;
        parseStep = 'done';
        importSuccess = true;
      } else {
        throw new Error('Failed to import tracks to server');
      }
    } catch (e: any) {
      importError = e.message || 'Failed to import library';
      parseStep = 'ready';
    }
  }

  function resetImport() {
    importFile = null;
    parseStep = 'idle';
    importError = '';
    parsedTracks = [];
    detectedProvider = '';
    importSuccess = false;
    importedCount = 0;
  }

  // Auto-parse when file is selected
  $: if (importFile && parseStep === 'idle') {
    parseFile();
  }
</script>

<div class="min-h-screen bg-gradient-to-b from-green-50 to-white">
  <!-- Header -->
  <div class="bg-white border-b border-gray-100">
    <div class="max-w-4xl mx-auto px-4 py-8 sm:px-6 lg:px-8">
      <button
        type="button"
        on:click={() => navigateTo('dashboard')}
        class="text-gray-500 hover:text-gray-700 mb-4 flex items-center text-sm"
      >
        <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
        </svg>
        Back to Dashboard
      </button>
      <h1 class="text-3xl font-bold text-gray-900 mb-2">
        Connect Your Music
      </h1>
      <p class="text-lg text-gray-600">
        Link your music data so we can scan for problematic artists.
      </p>
    </div>
  </div>

  <div class="max-w-4xl mx-auto px-4 py-8 sm:px-6 lg:px-8">
    <!-- Tabs -->
    <div class="flex space-x-1 bg-gray-100 rounded-xl p-1 mb-8">
      <button
        type="button"
        on:click={() => activeTab = 'services'}
        class="flex-1 py-2.5 px-4 rounded-lg text-sm font-medium transition-all {
          activeTab === 'services'
            ? 'bg-white text-gray-900 shadow-sm'
            : 'text-gray-600 hover:text-gray-900'
        }"
      >
        Streaming Services
      </button>
      <button
        type="button"
        on:click={() => activeTab = 'import'}
        class="flex-1 py-2.5 px-4 rounded-lg text-sm font-medium transition-all {
          activeTab === 'import'
            ? 'bg-white text-gray-900 shadow-sm'
            : 'text-gray-600 hover:text-gray-900'
        }"
      >
        Import Data
      </button>
      <button
        type="button"
        on:click={() => activeTab = 'extension'}
        class="flex-1 py-2.5 px-4 rounded-lg text-sm font-medium transition-all {
          activeTab === 'extension'
            ? 'bg-white text-gray-900 shadow-sm'
            : 'text-gray-600 hover:text-gray-900'
        }"
      >
        Browser Extension
      </button>
    </div>

    {#if activeTab === 'services'}
      <!-- Streaming Services -->
      <div class="space-y-4">
        <!-- Last.fm - Best option -->
        <div class="bg-white rounded-2xl shadow-lg border-2 border-red-200 p-6">
          <div class="flex items-center mb-2">
            <span class="px-2 py-0.5 bg-green-100 text-green-700 text-xs font-medium rounded-full">
              Recommended
            </span>
          </div>
          <div class="flex items-start">
            <div class="w-14 h-14 bg-red-600 rounded-2xl flex items-center justify-center mr-5 flex-shrink-0">
              <span class="text-white font-bold text-lg">Last</span>
            </div>
            <div class="flex-1">
              <div class="flex items-center mb-2">
                <h3 class="text-xl font-bold text-gray-900">Last.fm</h3>
                {#if connections.lastfm.connected}
                  <span class="ml-3 px-3 py-1 bg-green-100 text-green-800 text-sm font-medium rounded-full">
                    Connected
                  </span>
                {/if}
              </div>
              <p class="text-gray-600 mb-4">
                {#if connections.lastfm.connected}
                  Connected as @{connections.lastfm.username}. We can see your complete listening history.
                {:else}
                  Last.fm tracks your listening across ALL services (Spotify, Apple Music, etc.).
                  Connect to scan your complete listening history.
                {/if}
              </p>

              <div class="flex items-center gap-3">
                {#if connections.lastfm.connected}
                  <button
                    type="button"
                    on:click={() => navigateTo('library-scan')}
                    class="px-5 py-2.5 bg-purple-600 text-white rounded-xl hover:bg-purple-700 font-medium transition-colors"
                  >
                    Scan Library
                  </button>
                  <button
                    type="button"
                    on:click={() => connections.lastfm = { connected: false, username: '' }}
                    class="px-5 py-2.5 border border-gray-200 text-gray-700 rounded-xl hover:bg-gray-50 font-medium transition-colors"
                  >
                    Disconnect
                  </button>
                {:else}
                  <button
                    type="button"
                    on:click={connectLastFm}
                    class="px-6 py-3 bg-red-600 text-white rounded-xl hover:bg-red-700 font-medium transition-colors"
                  >
                    Connect Last.fm
                  </button>
                {/if}
              </div>
            </div>
          </div>

          <div class="mt-5 pt-5 border-t border-gray-100">
            <p class="text-sm text-gray-500">
              <strong>Don't have Last.fm?</strong> It's free!
              <a href="https://www.last.fm/join" target="_blank" rel="noopener" class="text-red-600 hover:underline">
                Create an account
              </a> and install the
              <a href="https://www.last.fm/about/trackmymusic" target="_blank" rel="noopener" class="text-red-600 hover:underline">
                scrobbler
              </a> to automatically track what you listen to.
            </p>
          </div>
        </div>

        <!-- YouTube Music -->
        <div class="bg-white rounded-2xl border border-gray-100 p-6">
          <div class="flex items-start">
            <div class="w-14 h-14 bg-red-600 rounded-2xl flex items-center justify-center mr-5 flex-shrink-0">
              <svg class="w-8 h-8 text-white" viewBox="0 0 24 24" fill="currentColor">
                <path d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z"/>
              </svg>
            </div>
            <div class="flex-1">
              <h3 class="text-xl font-bold text-gray-900 mb-2">YouTube Music</h3>
              <p class="text-gray-600 mb-4">
                Connect with your Google account to scan your YouTube Music library.
              </p>
              <button
                type="button"
                on:click={connectYouTube}
                class="px-6 py-3 bg-gray-100 text-gray-700 rounded-xl hover:bg-gray-200 font-medium transition-colors"
              >
                Coming Soon
              </button>
            </div>
          </div>
        </div>

        <!-- Note about Spotify -->
        <div class="bg-yellow-50 rounded-2xl p-5">
          <div class="flex items-start">
            <svg class="w-6 h-6 text-yellow-600 mr-3 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
            </svg>
            <div>
              <h4 class="font-medium text-yellow-900 mb-1">About Spotify</h4>
              <p class="text-sm text-yellow-800">
                Spotify is currently not accepting new developer applications.
                Use Last.fm (works with Spotify!) or import your Spotify data export instead.
              </p>
            </div>
          </div>
        </div>
      </div>

    {:else if activeTab === 'import'}
      <!-- Import Data -->
      <div class="bg-white rounded-2xl shadow-lg border border-gray-100 p-6">
        <h2 class="text-xl font-bold text-gray-900 mb-4">Import Your Music Data</h2>
        <p class="text-gray-600 mb-6">
          Export your data from streaming services and upload it here. We'll scan it for
          artists with documented misconduct.
        </p>

        <!-- Supported formats -->
        <div class="grid sm:grid-cols-2 gap-4 mb-6">
          <div class="border border-gray-200 rounded-xl p-4">
            <div class="flex items-center mb-2">
              <div class="w-8 h-8 bg-green-500 rounded-lg flex items-center justify-center mr-3">
                <span class="text-white text-sm font-bold">S</span>
              </div>
              <h4 class="font-medium text-gray-900">Spotify</h4>
            </div>
            <p class="text-sm text-gray-500 mb-3">
              Upload StreamingHistory.json or YourLibrary.json
            </p>
            <a
              href="https://www.spotify.com/account/privacy/"
              target="_blank"
              rel="noopener"
              class="text-sm text-green-600 hover:underline"
            >
              Request your data
            </a>
          </div>

          <div class="border border-gray-200 rounded-xl p-4">
            <div class="flex items-center mb-2">
              <div class="w-8 h-8 bg-gradient-to-br from-pink-500 to-red-500 rounded-lg flex items-center justify-center mr-3">
                <span class="text-white text-sm font-bold">A</span>
              </div>
              <h4 class="font-medium text-gray-900">Apple Music</h4>
            </div>
            <p class="text-sm text-gray-500 mb-3">
              Upload Apple Music Library Tracks.csv
            </p>
            <a
              href="https://privacy.apple.com/"
              target="_blank"
              rel="noopener"
              class="text-sm text-pink-600 hover:underline"
            >
              Request your data
            </a>
          </div>
        </div>

        <!-- Upload area -->
        <div class="border-2 border-dashed border-gray-300 rounded-xl p-8">
          {#if parseStep === 'done'}
            <div class="text-center">
              <div class="w-16 h-16 bg-green-100 rounded-full flex items-center justify-center mx-auto mb-4">
                <svg class="w-8 h-8 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                </svg>
              </div>
              <h3 class="font-medium text-lg text-gray-900 mb-2">Import Complete!</h3>
              <p class="text-gray-600 mb-4">
                Successfully imported {importedCount.toLocaleString()} tracks from {detectedProvider === 'spotify' ? 'Spotify' : 'Apple Music'}.
              </p>
              <div class="flex justify-center gap-3">
                <button
                  type="button"
                  on:click={() => navigateTo('library-scan')}
                  class="px-6 py-2.5 bg-purple-600 text-white rounded-xl hover:bg-purple-700 font-medium"
                >
                  Scan Library
                </button>
                <button
                  type="button"
                  on:click={resetImport}
                  class="px-6 py-2.5 border border-gray-200 text-gray-700 rounded-xl hover:bg-gray-50 font-medium"
                >
                  Import More
                </button>
              </div>
            </div>

          {:else if parseStep === 'importing'}
            <div class="text-center">
              <div class="w-10 h-10 border-4 border-indigo-600 border-t-transparent rounded-full animate-spin mx-auto mb-4"></div>
              <p class="text-gray-600">Uploading {parsedTracks.length.toLocaleString()} tracks...</p>
            </div>

          {:else if parseStep === 'parsing'}
            <div class="text-center">
              <div class="w-10 h-10 border-4 border-indigo-600 border-t-transparent rounded-full animate-spin mx-auto mb-4"></div>
              <p class="text-gray-600">Parsing file...</p>
            </div>

          {:else if parseStep === 'ready'}
            <div class="text-center">
              <div class="w-16 h-16 {detectedProvider === 'spotify' ? 'bg-green-100' : 'bg-pink-100'} rounded-full flex items-center justify-center mx-auto mb-4">
                <span class="text-2xl">{detectedProvider === 'spotify' ? '🎵' : '🍎'}</span>
              </div>
              <h3 class="font-medium text-lg text-gray-900 mb-2">
                Ready to Import
              </h3>
              <p class="text-gray-600 mb-2">
                Found <strong>{parsedTracks.length.toLocaleString()}</strong> tracks from <strong>{detectedProvider === 'spotify' ? 'Spotify' : 'Apple Music'}</strong>
              </p>
              <p class="text-sm text-gray-500 mb-4">
                File: {importFile?.name}
              </p>

              <div class="flex justify-center gap-3">
                <button
                  type="button"
                  on:click={importLibrary}
                  class="px-6 py-2.5 bg-indigo-600 text-white rounded-xl hover:bg-indigo-700 font-medium"
                >
                  Import {parsedTracks.length.toLocaleString()} Tracks
                </button>
                <button
                  type="button"
                  on:click={resetImport}
                  class="px-6 py-2.5 border border-gray-200 text-gray-700 rounded-xl hover:bg-gray-50 font-medium"
                >
                  Cancel
                </button>
              </div>
            </div>

          {:else}
            <div class="text-center">
              <input
                type="file"
                accept=".json,.csv"
                on:change={handleFileSelect}
                class="hidden"
                id="file-upload"
              />
              <label for="file-upload" class="cursor-pointer block">
                <div class="w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center mx-auto mb-4">
                  <svg class="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
                  </svg>
                </div>
                <h3 class="font-medium text-gray-900 mb-2">
                  Drop your export file here
                </h3>
                <p class="text-sm text-gray-500 mb-4">
                  JSON (Spotify) or CSV (Apple Music)
                </p>
                <span class="inline-block px-6 py-2.5 bg-indigo-100 text-indigo-700 rounded-xl font-medium hover:bg-indigo-200 transition-colors">
                  Choose File
                </span>
              </label>
            </div>
          {/if}

          {#if importError}
            <div class="mt-4 p-4 bg-red-50 rounded-xl">
              <div class="flex items-start">
                <svg class="w-5 h-5 text-red-500 mr-2 flex-shrink-0 mt-0.5" fill="currentColor" viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
                </svg>
                <div>
                  <p class="text-red-800">{importError}</p>
                  <button
                    type="button"
                    on:click={resetImport}
                    class="text-red-600 underline text-sm mt-1"
                  >
                    Try again
                  </button>
                </div>
              </div>
            </div>
          {/if}
        </div>

        <!-- Instructions -->
        <div class="mt-6 bg-blue-50 rounded-xl p-5">
          <h4 class="font-medium text-blue-900 mb-3">How to get your data:</h4>
          <div class="space-y-4 text-sm text-blue-800">
            <div>
              <strong class="text-blue-900">Spotify:</strong>
              <ol class="list-decimal list-inside ml-2 mt-1 space-y-1">
                <li>Go to your <a href="https://www.spotify.com/account/privacy/" target="_blank" class="underline">Privacy Settings</a></li>
                <li>Click "Request" under "Download your data"</li>
                <li>Wait for email (usually 1-5 days)</li>
                <li>Extract the ZIP and upload StreamingHistory.json or YourLibrary.json</li>
              </ol>
            </div>
            <div>
              <strong class="text-blue-900">Apple Music:</strong>
              <ol class="list-decimal list-inside ml-2 mt-1 space-y-1">
                <li>Go to <a href="https://privacy.apple.com/" target="_blank" class="underline">privacy.apple.com</a></li>
                <li>Sign in and select "Request a copy of your data"</li>
                <li>Select "Apple Media Services information"</li>
                <li>Upload the Apple Music Library Tracks.csv file</li>
              </ol>
            </div>
          </div>
        </div>
      </div>

    {:else if activeTab === 'extension'}
      <!-- Browser Extension -->
      <div class="bg-white rounded-2xl shadow-lg border border-gray-100 p-6">
        <div class="text-center mb-8">
          <div class="w-20 h-20 bg-indigo-100 rounded-full flex items-center justify-center mx-auto mb-4">
            <svg class="w-10 h-10 text-indigo-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14 10l-2 1m0 0l-2-1m2 1v2.5M20 7l-2 1m2-1l-2-1m2 1v2.5M14 4l-2-1-2 1M4 7l2-1M4 7l2 1M4 7v2.5M12 21l-2-1m2 1l2-1m-2 1v-2.5M6 18l-2-1v-2.5M18 18l2-1v-2.5" />
            </svg>
          </div>
          <h2 class="text-2xl font-bold text-gray-900 mb-2">Browser Extension</h2>
          <p class="text-gray-600 max-w-lg mx-auto">
            Our browser extension works with any web-based music player and blocks
            artists in real-time as you browse.
          </p>
        </div>

        <div class="grid sm:grid-cols-2 gap-4 mb-8">
          <div class="border border-gray-200 rounded-xl p-5 text-center">
            <div class="w-12 h-12 bg-blue-100 rounded-full flex items-center justify-center mx-auto mb-3">
              <svg class="w-6 h-6 text-blue-600" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 0C8.21 0 4.831 1.757 2.632 4.501l3.953 6.848A5.454 5.454 0 0 1 12 6.545h10.691A12 12 0 0 0 12 0zM1.931 5.47A11.943 11.943 0 0 0 0 12c0 6.627 5.373 12 12 12 1.118 0 2.201-.153 3.229-.439l-4.311-7.469a5.455 5.455 0 0 1-5.016-2.895L1.931 5.47zM21.5 8.182H12a5.455 5.455 0 0 1-.019 10.909l4.316 7.479A12 12 0 0 0 21.5 8.182z"/>
              </svg>
            </div>
            <h4 class="font-medium text-gray-900 mb-2">Chrome</h4>
            <span class="px-4 py-2 bg-gray-100 text-gray-500 rounded-lg text-sm inline-block">
              Coming Soon
            </span>
          </div>
          <div class="border border-gray-200 rounded-xl p-5 text-center">
            <div class="w-12 h-12 bg-orange-100 rounded-full flex items-center justify-center mx-auto mb-3">
              <svg class="w-6 h-6 text-orange-600" viewBox="0 0 24 24" fill="currentColor">
                <path d="M23.442 10.037c-.097-.104-.224-.185-.366-.235l-9.197-3.283a.89.89 0 0 0-.576 0l-9.197 3.283c-.142.05-.269.131-.366.235-.097.104-.168.231-.205.371-.037.14-.04.288-.008.43.032.141.098.273.191.385l5.55 6.673c.117.141.278.243.457.29a.89.89 0 0 0 .529-.044l3.193-1.285v4.68c0 .248.098.486.272.661a.93.93 0 0 0 .66.274.93.93 0 0 0 .66-.274.934.934 0 0 0 .273-.661v-4.68l3.193 1.285a.89.89 0 0 0 .529.044c.179-.047.34-.149.457-.29l5.55-6.673c.093-.112.159-.244.191-.385a.867.867 0 0 0-.008-.43.862.862 0 0 0-.205-.371z"/>
              </svg>
            </div>
            <h4 class="font-medium text-gray-900 mb-2">Firefox</h4>
            <span class="px-4 py-2 bg-gray-100 text-gray-500 rounded-lg text-sm inline-block">
              Coming Soon
            </span>
          </div>
        </div>

        <div class="bg-indigo-50 rounded-xl p-5">
          <h4 class="font-medium text-indigo-900 mb-2">How the extension works:</h4>
          <ul class="text-sm text-indigo-800 space-y-2">
            <li class="flex items-start">
              <svg class="w-5 h-5 text-indigo-500 mr-2 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
              </svg>
              Runs on Spotify Web Player, YouTube Music, SoundCloud, etc.
            </li>
            <li class="flex items-start">
              <svg class="w-5 h-5 text-indigo-500 mr-2 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
              </svg>
              Highlights or hides tracks from blocked artists
            </li>
            <li class="flex items-start">
              <svg class="w-5 h-5 text-indigo-500 mr-2 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
              </svg>
              Shows warnings before playing flagged content
            </li>
            <li class="flex items-start">
              <svg class="w-5 h-5 text-indigo-500 mr-2 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
              </svg>
              Syncs with your blocklist in real-time
            </li>
          </ul>
        </div>
      </div>
    {/if}

    <!-- Help text -->
    <div class="mt-8 bg-gray-50 rounded-2xl p-6">
      <div class="flex items-start">
        <div class="w-12 h-12 bg-gray-100 rounded-full flex items-center justify-center mr-4 flex-shrink-0">
          <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        </div>
        <div>
          <h4 class="font-medium text-gray-900 mb-1">Why connect your music?</h4>
          <p class="text-sm text-gray-600">
            When you connect your music data, we can scan your library and identify artists
            with documented harmful behavior. You'll see exactly how much of your music
            is affected and can choose to block them.
          </p>
        </div>
      </div>
    </div>
  </div>
</div>
