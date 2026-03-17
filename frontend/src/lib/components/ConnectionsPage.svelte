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

    const headers = lines[0].split(',').map(h => h.trim().toLowerCase().replace(/"/g, ''));
    const trackNameIdx = headers.findIndex(h => h.includes('song') || h.includes('track') || h.includes('title'));
    const artistIdx = headers.findIndex(h => h.includes('artist'));
    const albumIdx = headers.findIndex(h => h.includes('album'));
    const dateIdx = headers.findIndex(h => h.includes('date') || h.includes('added'));

    const tracks: ImportTrack[] = [];

    for (let i = 1; i < lines.length; i++) {
      const line = lines[i].trim();
      if (!line) continue;

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

  async function parseFile() {
    if (!importFile) return;

    parseStep = 'parsing';
    importError = '';

    try {
      const fileName = importFile.name.toLowerCase();
      const fileText = await importFile.text();

      if (fileName.endsWith('.json')) {
        const data = JSON.parse(fileText);

        if (Array.isArray(data)) {
          if (data[0]?.endTime || data[0]?.ts || data[0]?.artistName || data[0]?.master_metadata_album_artist_name) {
            detectedProvider = 'spotify';
            parsedTracks = parseSpotifyStreamingHistory(data);
          }
        } else if (data.tracks || data.albums || data.artists) {
          detectedProvider = 'spotify';
          parsedTracks = parseSpotifyLibrary(data);
        } else {
          throw new Error('Unrecognized JSON format. Please upload a Spotify export file.');
        }

      } else if (fileName.endsWith('.csv')) {
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

<div class="connections brand-page surface-page">
  <div class="connections__container brand-page__inner brand-page__stack">
    <section class="brand-hero connections__hero">
      <div class="brand-hero__header">
        <div class="brand-hero__copy">
          <button
            type="button"
            on:click={() => navigateTo('home')}
            class="brand-back"
          >
            <svg fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
            </svg>
            Back to Home
          </button>
          <div class="brand-kickers">
            <span class="brand-kicker">Music Access</span>
            <span class="brand-kicker brand-kicker--accent">Connections + Imports</span>
          </div>
          <h1 class="brand-title brand-title--compact">Bring every listening surface into the same filter layer.</h1>
          <p class="brand-subtitle">
            Connect live services, import historical exports, and keep the extension story on the same branded system as the rest of the app.
          </p>
        </div>

        <div class="brand-hero__aside">
          <div class="brand-stat-grid brand-stat-grid--compact" aria-label="Connection overview">
            <div class="brand-stat">
              <span class="brand-stat__value">{connections.lastfm.connected ? '1' : '0'}</span>
              <span class="brand-stat__label">Live sources</span>
            </div>
            <div class="brand-stat">
              <span class="brand-stat__value">3</span>
              <span class="brand-stat__label">Import paths</span>
            </div>
          </div>
        </div>
      </div>
    </section>

    <div class="connections__body">
    <!-- Tabs -->
    <div class="tabs">
      <button
        type="button"
        on:click={() => activeTab = 'services'}
        class="tabs__btn"
        class:tabs__btn--active={activeTab === 'services'}
      >
        Streaming Services
      </button>
      <button
        type="button"
        on:click={() => activeTab = 'import'}
        class="tabs__btn"
        class:tabs__btn--active={activeTab === 'import'}
      >
        Import Data
      </button>
      <button
        type="button"
        on:click={() => activeTab = 'extension'}
        class="tabs__btn"
        class:tabs__btn--active={activeTab === 'extension'}
      >
        Browser Extension
      </button>
    </div>

    {#if activeTab === 'services'}
      <div class="services">
        <!-- Last.fm -->
        <div class="service-card service-card--featured">
          <span class="service-card__badge">Recommended</span>
          <div class="service-card__row">
            <div class="service-card__icon service-card__icon--lastfm">
              <span>Last</span>
            </div>
            <div class="service-card__content">
              <div class="service-card__name-row">
                <h3 class="service-card__name">Last.fm</h3>
                {#if connections.lastfm.connected}
                  <span class="service-card__status">Connected</span>
                {/if}
              </div>
              <p class="service-card__desc">
                {#if connections.lastfm.connected}
                  Connected as @{connections.lastfm.username}. We can see your complete listening history.
                {:else}
                  Last.fm tracks your listening across ALL services (Spotify, Apple Music, etc.).
                  Connect to scan your complete listening history.
                {/if}
              </p>
              <div class="service-card__actions">
                {#if connections.lastfm.connected}
                  <button
                    type="button"
                    on:click={() => navigateTo('library-scan')}
                    class="btn btn--primary"
                  >
                    Scan Library
                  </button>
                  <button
                    type="button"
                    on:click={() => connections.lastfm = { connected: false, username: '' }}
                    class="btn btn--secondary"
                  >
                    Disconnect
                  </button>
                {:else}
                  <button
                    type="button"
                    on:click={connectLastFm}
                    class="btn btn--lastfm"
                  >
                    Connect Last.fm
                  </button>
                {/if}
              </div>
            </div>
          </div>
          <div class="service-card__footer">
            <p>
              <strong>Don't have Last.fm?</strong> It's free!
              <a href="https://www.last.fm/join" target="_blank" rel="noopener noreferrer">Create an account</a>
              and install the
              <a href="https://www.last.fm/about/trackmymusic" target="_blank" rel="noopener noreferrer">scrobbler</a>
              to automatically track what you listen to.
            </p>
          </div>
        </div>

        <!-- YouTube Music -->
        <div class="service-card">
          <div class="service-card__row">
            <div class="service-card__icon service-card__icon--youtube">
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z"/>
              </svg>
            </div>
            <div class="service-card__content">
              <h3 class="service-card__name">YouTube Music</h3>
              <p class="service-card__desc">
                Connect with your Google account to scan your YouTube Music library.
              </p>
              <button type="button" on:click={connectYouTube} class="btn btn--secondary btn--disabled">
                Coming Soon
              </button>
            </div>
          </div>
        </div>

        <!-- Spotify notice -->
        <div class="notice notice--warning">
          <svg class="notice__icon" fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
            <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
          </svg>
          <div>
            <h4 class="notice__title">About Spotify</h4>
            <p class="notice__text">
              Spotify is currently not accepting new developer applications.
              Use Last.fm (works with Spotify!) or import your Spotify data export instead.
            </p>
          </div>
        </div>
      </div>

    {:else if activeTab === 'import'}
      <div class="import-section">
        <h2 class="import-section__title">Import Your Music Data</h2>
        <p class="import-section__desc">
          Export your data from streaming services and upload it here. We'll scan it for
          artists with documented misconduct.
        </p>

        <!-- Supported formats -->
        <div class="format-grid">
          <div class="format-card">
            <div class="format-card__header">
              <div class="format-card__icon format-card__icon--spotify">S</div>
              <h4 class="format-card__name">Spotify</h4>
            </div>
            <p class="format-card__desc">Upload StreamingHistory.json or YourLibrary.json</p>
            <a href="https://www.spotify.com/account/privacy/" target="_blank" rel="noopener noreferrer" class="format-card__link format-card__link--spotify">
              Request your data
            </a>
          </div>
          <div class="format-card">
            <div class="format-card__header">
              <div class="format-card__icon format-card__icon--apple">A</div>
              <h4 class="format-card__name">Apple Music</h4>
            </div>
            <p class="format-card__desc">Upload Apple Music Library Tracks.csv</p>
            <a href="https://privacy.apple.com/" target="_blank" rel="noopener noreferrer" class="format-card__link format-card__link--apple">
              Request your data
            </a>
          </div>
        </div>

        <!-- Upload area -->
        <div class="upload-zone">
          {#if parseStep === 'done'}
            <div class="upload-zone__center">
              <div class="upload-zone__icon-circle upload-zone__icon-circle--success">
                <svg fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                </svg>
              </div>
              <h3 class="upload-zone__heading">Import Complete!</h3>
              <p class="upload-zone__text">
                Successfully imported {importedCount.toLocaleString()} tracks from {detectedProvider === 'spotify' ? 'Spotify' : 'Apple Music'}.
              </p>
              <div class="upload-zone__actions">
                <button type="button" on:click={() => navigateTo('library-scan')} class="btn btn--primary">Scan Library</button>
                <button type="button" on:click={resetImport} class="btn btn--secondary">Import More</button>
              </div>
            </div>

          {:else if parseStep === 'importing' || parseStep === 'parsing'}
            <div class="upload-zone__center">
              <div class="upload-zone__spinner"></div>
              <p class="upload-zone__text">
                {parseStep === 'importing' ? `Uploading ${parsedTracks.length.toLocaleString()} tracks...` : 'Parsing file...'}
              </p>
            </div>

          {:else if parseStep === 'ready'}
            <div class="upload-zone__center">
              <div class="upload-zone__icon-circle" class:upload-zone__icon-circle--spotify={detectedProvider === 'spotify'} class:upload-zone__icon-circle--apple={detectedProvider !== 'spotify'}>
                <span class="upload-zone__emoji">{detectedProvider === 'spotify' ? '🎵' : '🍎'}</span>
              </div>
              <h3 class="upload-zone__heading">Ready to Import</h3>
              <p class="upload-zone__text">
                Found <strong>{parsedTracks.length.toLocaleString()}</strong> tracks from <strong>{detectedProvider === 'spotify' ? 'Spotify' : 'Apple Music'}</strong>
              </p>
              <p class="upload-zone__meta">File: {importFile?.name}</p>
              <div class="upload-zone__actions">
                <button type="button" on:click={importLibrary} class="btn btn--primary">
                  Import {parsedTracks.length.toLocaleString()} Tracks
                </button>
                <button type="button" on:click={resetImport} class="btn btn--secondary">Cancel</button>
              </div>
            </div>

          {:else}
            <div class="upload-zone__center">
              <input type="file" accept=".json,.csv" on:change={handleFileSelect} class="upload-zone__input" id="file-upload" />
              <label for="file-upload" class="upload-zone__label">
                <div class="upload-zone__icon-circle upload-zone__icon-circle--neutral">
                  <svg fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
                  </svg>
                </div>
                <h3 class="upload-zone__heading">Drop your export file here</h3>
                <p class="upload-zone__meta">JSON (Spotify) or CSV (Apple Music)</p>
                <span class="btn btn--primary">Choose File</span>
              </label>
            </div>
          {/if}

          {#if importError}
            <div class="upload-zone__error">
              <svg fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
                <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
              </svg>
              <div>
                <p>{importError}</p>
                <button type="button" on:click={resetImport} class="upload-zone__error-retry">Try again</button>
              </div>
            </div>
          {/if}
        </div>

        <!-- Instructions -->
        <div class="info-box">
          <h4 class="info-box__title">How to get your data:</h4>
          <div class="info-box__content">
            <div>
              <strong>Spotify:</strong>
              <ol class="info-box__list">
                <li>Go to your <a href="https://www.spotify.com/account/privacy/" target="_blank" rel="noopener noreferrer">Privacy Settings</a></li>
                <li>Click "Request" under "Download your data"</li>
                <li>Wait for email (usually 1-5 days)</li>
                <li>Extract the ZIP and upload StreamingHistory.json or YourLibrary.json</li>
              </ol>
            </div>
            <div>
              <strong>Apple Music:</strong>
              <ol class="info-box__list">
                <li>Go to <a href="https://privacy.apple.com/" target="_blank" rel="noopener noreferrer">privacy.apple.com</a></li>
                <li>Sign in and select "Request a copy of your data"</li>
                <li>Select "Apple Media Services information"</li>
                <li>Upload the Apple Music Library Tracks.csv file</li>
              </ol>
            </div>
          </div>
        </div>
      </div>

    {:else if activeTab === 'extension'}
      <div class="extension-section">
        <div class="extension-section__hero">
          <div class="extension-section__icon">
            <svg fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14 10l-2 1m0 0l-2-1m2 1v2.5M20 7l-2 1m2-1l-2-1m2 1v2.5M14 4l-2-1-2 1M4 7l2-1M4 7l2 1M4 7v2.5M12 21l-2-1m2 1l2-1m-2 1v-2.5M6 18l-2-1v-2.5M18 18l2-1v-2.5" />
            </svg>
          </div>
          <h2 class="extension-section__title">Browser Extension</h2>
          <p class="extension-section__desc">
            Our browser extension works with any web-based music player and blocks
            artists in real-time as you browse.
          </p>
        </div>

        <div class="browser-grid">
          <div class="browser-card">
            <div class="browser-card__icon browser-card__icon--chrome">
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 0C8.21 0 4.831 1.757 2.632 4.501l3.953 6.848A5.454 5.454 0 0 1 12 6.545h10.691A12 12 0 0 0 12 0zM1.931 5.47A11.943 11.943 0 0 0 0 12c0 6.627 5.373 12 12 12 1.118 0 2.201-.153 3.229-.439l-4.311-7.469a5.455 5.455 0 0 1-5.016-2.895L1.931 5.47zM21.5 8.182H12a5.455 5.455 0 0 1-.019 10.909l4.316 7.479A12 12 0 0 0 21.5 8.182z"/>
              </svg>
            </div>
            <h4 class="browser-card__name">Chrome</h4>
            <span class="browser-card__badge">Coming Soon</span>
          </div>
          <div class="browser-card">
            <div class="browser-card__icon browser-card__icon--firefox">
              <svg viewBox="0 0 24 24" fill="currentColor">
                <path d="M23.442 10.037c-.097-.104-.224-.185-.366-.235l-9.197-3.283a.89.89 0 0 0-.576 0l-9.197 3.283c-.142.05-.269.131-.366.235-.097.104-.168.231-.205.371-.037.14-.04.288-.008.43.032.141.098.273.191.385l5.55 6.673c.117.141.278.243.457.29a.89.89 0 0 0 .529-.044l3.193-1.285v4.68c0 .248.098.486.272.661a.93.93 0 0 0 .66.274.93.93 0 0 0 .66-.274.934.934 0 0 0 .273-.661v-4.68l3.193 1.285a.89.89 0 0 0 .529.044c.179-.047.34-.149.457-.29l5.55-6.673c.093-.112.159-.244.191-.385a.867.867 0 0 0-.008-.43.862.862 0 0 0-.205-.371z"/>
              </svg>
            </div>
            <h4 class="browser-card__name">Firefox</h4>
            <span class="browser-card__badge">Coming Soon</span>
          </div>
        </div>

        <div class="info-box">
          <h4 class="info-box__title">How the extension works:</h4>
          <ul class="info-box__checklist">
            <li>
              <svg fill="currentColor" viewBox="0 0 20 20" aria-hidden="true"><path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" /></svg>
              Runs on Spotify Web Player, YouTube Music, SoundCloud, etc.
            </li>
            <li>
              <svg fill="currentColor" viewBox="0 0 20 20" aria-hidden="true"><path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" /></svg>
              Highlights or hides tracks from blocked artists
            </li>
            <li>
              <svg fill="currentColor" viewBox="0 0 20 20" aria-hidden="true"><path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" /></svg>
              Shows warnings before playing flagged content
            </li>
            <li>
              <svg fill="currentColor" viewBox="0 0 20 20" aria-hidden="true"><path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" /></svg>
              Syncs with your blocklist in real-time
            </li>
          </ul>
        </div>
      </div>
    {/if}

    <!-- Help text -->
    <div class="help-card">
      <div class="help-card__icon">
        <svg fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
      </div>
      <div>
        <h4 class="help-card__title">Why connect your music?</h4>
        <p class="help-card__text">
          When you connect your music data, we can scan your library and identify artists
          with documented harmful behavior. You'll see exactly how much of your music
          is affected and can choose to block them.
        </p>
      </div>
    </div>
    </div>
  </div>
</div>

<style>
  .connections {
    min-height: calc(100vh - 4.5rem);
  }

  .connections__container {
    width: 100%;
  }

  .connections__body {
    padding: 0;
  }

  /* ===== TABS ===== */
  .tabs {
    display: flex;
    gap: 0.25rem;
    padding: 0.1875rem;
    background-color: var(--color-bg-interactive);
    border-radius: var(--radius-xl);
    margin-bottom: 2rem;
  }

  .tabs__btn {
    flex: 1;
    padding: 0.625rem 1rem;
    font-family: var(--font-family-sans);
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--color-text-tertiary);
    background: none;
    border: none;
    border-radius: var(--radius-lg);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .tabs__btn:hover:not(.tabs__btn--active) {
    color: var(--color-text-secondary);
  }

  .tabs__btn--active {
    background-color: var(--color-bg-elevated);
    color: var(--color-text-primary);
    box-shadow: var(--shadow-sm);
  }

  /* ===== SERVICE CARDS ===== */
  .services {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .service-card {
    background-color: var(--color-bg-elevated);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-2xl);
    padding: 1.5rem;
  }

  .service-card--featured {
    border-color: var(--color-brand-primary);
  }

  .service-card__badge {
    display: inline-block;
    padding: 0.125rem 0.625rem;
    font-size: var(--text-xs);
    font-weight: 500;
    color: var(--color-success);
    background-color: rgba(16, 185, 129, 0.15);
    border-radius: var(--radius-full);
    margin-bottom: 0.75rem;
  }

  .service-card__row {
    display: flex;
    align-items: flex-start;
    gap: 1.25rem;
  }

  .service-card__icon {
    width: 3.5rem;
    height: 3.5rem;
    border-radius: var(--radius-2xl);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    color: white;
  }

  .service-card__icon svg {
    width: 2rem;
    height: 2rem;
    max-width: none;
    max-height: none;
  }

  .service-card__icon--lastfm {
    background-color: #dc2626;
    font-weight: 700;
    font-size: var(--text-lg);
  }

  .service-card__icon--youtube {
    background-color: #dc2626;
  }

  .service-card__content {
    flex: 1;
    min-width: 0;
  }

  .service-card__name-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 0.5rem;
  }

  .service-card__name {
    font-size: var(--text-xl);
    font-weight: 700;
    color: var(--color-text-primary);
  }

  .service-card__status {
    padding: 0.1875rem 0.75rem;
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--color-success);
    background-color: rgba(16, 185, 129, 0.15);
    border-radius: var(--radius-full);
  }

  .service-card__desc {
    font-size: var(--text-sm);
    color: var(--color-text-tertiary);
    margin-bottom: 1rem;
    line-height: 1.5;
  }

  .service-card__actions {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .service-card__footer {
    margin-top: 1.25rem;
    padding-top: 1.25rem;
    border-top: 1px solid var(--color-border-subtle);
    font-size: var(--text-sm);
    color: var(--color-text-tertiary);
    line-height: 1.5;
  }

  .service-card__footer strong {
    color: var(--color-text-secondary);
  }

  .service-card__footer a {
    color: var(--color-brand-primary);
    text-decoration: none;
  }

  .service-card__footer a:hover {
    text-decoration: underline;
  }

  /* ===== BUTTONS ===== */
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0.625rem 1.25rem;
    font-family: var(--font-family-sans);
    font-size: var(--text-sm);
    font-weight: 600;
    border-radius: var(--radius-lg);
    border: none;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .btn--primary {
    background-color: var(--color-brand-primary);
    color: white;
  }

  .btn--primary:hover {
    background-color: var(--color-brand-primary-hover);
  }

  .btn--secondary {
    background-color: var(--color-bg-interactive);
    color: var(--color-text-secondary);
    border: 1px solid var(--color-border-default);
  }

  .btn--secondary:hover {
    background-color: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .btn--lastfm {
    background-color: #dc2626;
    color: white;
  }

  .btn--lastfm:hover {
    background-color: #b91c1c;
  }

  .btn--disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* ===== NOTICE ===== */
  .notice {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    padding: 1.25rem;
    border-radius: var(--radius-2xl);
  }

  .notice--warning {
    background-color: var(--color-warning-muted, rgba(245, 158, 11, 0.15));
    border: 1px solid var(--color-warning, #f59e0b);
  }

  .notice__icon {
    width: 1.5rem;
    height: 1.5rem;
    flex-shrink: 0;
    color: var(--color-warning, #f59e0b);
    max-width: none;
    max-height: none;
  }

  .notice__title {
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--color-warning, #fbbf24);
    margin-bottom: 0.25rem;
  }

  .notice__text {
    font-size: var(--text-sm);
    color: var(--color-text-secondary);
    line-height: 1.5;
  }

  /* ===== IMPORT SECTION ===== */
  .import-section {
    background-color: var(--color-bg-elevated);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-2xl);
    padding: 1.5rem;
  }

  .import-section__title {
    font-size: var(--text-xl);
    font-weight: 700;
    color: var(--color-text-primary);
    margin-bottom: 0.5rem;
  }

  .import-section__desc {
    font-size: var(--text-sm);
    color: var(--color-text-tertiary);
    margin-bottom: 1.5rem;
    line-height: 1.5;
  }

  /* ===== FORMAT CARDS ===== */
  .format-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(14rem, 1fr));
    gap: 1rem;
    margin-bottom: 1.5rem;
  }

  .format-card {
    padding: 1rem;
    border-radius: var(--radius-xl);
    border: 1px solid var(--color-border-default);
    background-color: var(--color-bg-interactive);
  }

  .format-card__header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 0.5rem;
  }

  .format-card__icon {
    width: 2rem;
    height: 2rem;
    border-radius: var(--radius-lg);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: var(--text-sm);
    font-weight: 700;
    color: white;
    flex-shrink: 0;
  }

  .format-card__icon--spotify {
    background-color: #1DB954;
  }

  .format-card__icon--apple {
    background: linear-gradient(135deg, #FA2D48, #A833B9);
  }

  .format-card__name {
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .format-card__desc {
    font-size: var(--text-sm);
    color: var(--color-text-tertiary);
    margin-bottom: 0.75rem;
    line-height: 1.5;
  }

  .format-card__link {
    font-size: var(--text-sm);
    text-decoration: none;
  }

  .format-card__link:hover {
    text-decoration: underline;
  }

  .format-card__link--spotify {
    color: #1DB954;
  }

  .format-card__link--apple {
    color: #ec4899;
  }

  /* ===== UPLOAD ZONE ===== */
  .upload-zone {
    border: 2px dashed var(--color-border-default);
    border-radius: var(--radius-xl);
    padding: 2rem;
  }

  .upload-zone__center {
    text-align: center;
  }

  .upload-zone__input {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip: rect(0 0 0 0);
  }

  .upload-zone__label {
    cursor: pointer;
    display: block;
  }

  .upload-zone__icon-circle {
    width: 4rem;
    height: 4rem;
    border-radius: var(--radius-full);
    display: flex;
    align-items: center;
    justify-content: center;
    margin: 0 auto 1rem;
  }

  .upload-zone__icon-circle svg {
    width: 2rem;
    height: 2rem;
    max-width: none;
    max-height: none;
  }

  .upload-zone__icon-circle--neutral {
    background-color: var(--color-bg-interactive);
    color: var(--color-text-tertiary);
  }

  .upload-zone__icon-circle--success {
    background-color: rgba(16, 185, 129, 0.15);
    color: var(--color-success);
  }

  .upload-zone__icon-circle--spotify {
    background-color: rgba(16, 185, 129, 0.15);
  }

  .upload-zone__icon-circle--apple {
    background-color: rgba(236, 72, 153, 0.15);
  }

  .upload-zone__emoji {
    font-size: var(--text-2xl);
  }

  .upload-zone__heading {
    font-size: var(--text-lg);
    font-weight: 600;
    color: var(--color-text-primary);
    margin-bottom: 0.5rem;
  }

  .upload-zone__text {
    font-size: var(--text-sm);
    color: var(--color-text-tertiary);
    margin-bottom: 0.5rem;
    line-height: 1.5;
  }

  .upload-zone__text strong {
    color: var(--color-text-secondary);
  }

  .upload-zone__meta {
    font-size: var(--text-sm);
    color: var(--color-text-muted);
    margin-bottom: 1rem;
  }

  .upload-zone__actions {
    display: flex;
    justify-content: center;
    gap: 0.75rem;
    margin-top: 1rem;
  }

  .upload-zone__spinner {
    width: 2.5rem;
    height: 2.5rem;
    border: 4px solid var(--color-border-default);
    border-top-color: var(--color-brand-primary);
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin: 0 auto 1rem;
  }

  .upload-zone__error {
    margin-top: 1rem;
    padding: 1rem;
    border-radius: var(--radius-xl);
    background-color: var(--color-error-muted);
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
  }

  .upload-zone__error svg {
    width: 1.25rem;
    height: 1.25rem;
    color: var(--color-error);
    flex-shrink: 0;
    margin-top: 0.125rem;
    max-width: none;
    max-height: none;
  }

  .upload-zone__error p {
    font-size: var(--text-sm);
    color: var(--color-error);
  }

  .upload-zone__error-retry {
    font-size: var(--text-sm);
    color: var(--color-error);
    text-decoration: underline;
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
    margin-top: 0.25rem;
    font-family: var(--font-family-sans);
  }

  /* ===== INFO BOX ===== */
  .info-box {
    margin-top: 1.5rem;
    padding: 1.25rem;
    border-radius: var(--radius-xl);
    background-color: var(--color-bg-interactive);
    border: 1px solid var(--color-border-default);
  }

  .info-box__title {
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--color-text-primary);
    margin-bottom: 0.75rem;
  }

  .info-box__content {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    font-size: var(--text-sm);
    color: var(--color-text-tertiary);
    line-height: 1.5;
  }

  .info-box__content strong {
    color: var(--color-text-secondary);
  }

  .info-box__content a {
    color: var(--color-brand-primary);
    text-decoration: underline;
  }

  .info-box__list {
    list-style: decimal;
    list-style-position: inside;
    margin-top: 0.25rem;
    margin-left: 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .info-box__checklist {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    font-size: var(--text-sm);
    color: var(--color-text-secondary);
  }

  .info-box__checklist li {
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
  }

  .info-box__checklist svg {
    width: 1.25rem;
    height: 1.25rem;
    color: var(--color-success);
    flex-shrink: 0;
    max-width: none;
    max-height: none;
  }

  /* ===== EXTENSION SECTION ===== */
  .extension-section {
    background-color: var(--color-bg-elevated);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-2xl);
    padding: 1.5rem;
  }

  .extension-section__hero {
    text-align: center;
    margin-bottom: 2rem;
  }

  .extension-section__icon {
    width: 5rem;
    height: 5rem;
    border-radius: var(--radius-full);
    display: flex;
    align-items: center;
    justify-content: center;
    margin: 0 auto 1rem;
    background-color: var(--color-brand-primary-muted);
    color: var(--color-brand-primary);
  }

  .extension-section__icon svg {
    width: 2.5rem;
    height: 2.5rem;
    max-width: none;
    max-height: none;
  }

  .extension-section__title {
    font-size: var(--text-2xl);
    font-weight: 700;
    color: var(--color-text-primary);
    margin-bottom: 0.5rem;
  }

  .extension-section__desc {
    font-size: var(--text-sm);
    color: var(--color-text-tertiary);
    max-width: 32rem;
    margin: 0 auto;
    line-height: 1.5;
  }

  /* ===== BROWSER CARDS ===== */
  .browser-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(10rem, 1fr));
    gap: 1rem;
    margin-bottom: 1.5rem;
  }

  .browser-card {
    text-align: center;
    padding: 1.25rem;
    border-radius: var(--radius-xl);
    border: 1px solid var(--color-border-default);
    background-color: var(--color-bg-interactive);
  }

  .browser-card__icon {
    width: 3rem;
    height: 3rem;
    border-radius: var(--radius-full);
    display: flex;
    align-items: center;
    justify-content: center;
    margin: 0 auto 0.75rem;
  }

  .browser-card__icon svg {
    width: 1.5rem;
    height: 1.5rem;
    max-width: none;
    max-height: none;
  }

  .browser-card__icon--chrome {
    background-color: rgba(59, 130, 246, 0.15);
    color: #60a5fa;
  }

  .browser-card__icon--firefox {
    background-color: rgba(249, 115, 22, 0.15);
    color: #fb923c;
  }

  .browser-card__name {
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--color-text-primary);
    margin-bottom: 0.5rem;
  }

  .browser-card__badge {
    display: inline-block;
    padding: 0.375rem 1rem;
    font-size: var(--text-sm);
    color: var(--color-text-tertiary);
    background-color: var(--color-bg-elevated);
    border-radius: var(--radius-lg);
  }

  /* ===== HELP CARD ===== */
  .help-card {
    margin-top: 2rem;
    display: flex;
    align-items: flex-start;
    gap: 1rem;
    padding: 1.5rem;
    border-radius: var(--radius-xl);
    background-color: var(--color-bg-interactive);
    border: 1px solid var(--color-border-default);
  }

  .help-card__icon {
    width: 3rem;
    height: 3rem;
    border-radius: var(--radius-full);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    background-color: var(--color-bg-elevated);
    color: var(--color-text-secondary);
  }

  .help-card__icon svg {
    width: 1.5rem;
    height: 1.5rem;
    max-width: none;
    max-height: none;
  }

  .help-card__title {
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--color-text-primary);
    margin-bottom: 0.25rem;
  }

  .help-card__text {
    font-size: var(--text-sm);
    color: var(--color-text-tertiary);
    line-height: 1.5;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
