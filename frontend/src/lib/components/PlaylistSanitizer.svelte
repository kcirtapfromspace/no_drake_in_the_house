<script lang="ts">
  import { onMount } from 'svelte';
  import { sanitizerStore, sanitizerActions, hasGrade, hasPlan, allReplacementsSelected } from '../stores/sanitizer';
  import { playlistBrowserStore, filteredPlaylists, selectedPlaylistCount, allFilteredSelected, selectedPlaylists, playlistBrowserActions, type PlaylistSummary } from '../stores/playlist-browser';
  import PlaylistCard from './PlaylistCard.svelte';
  import PlaylistTracklist from './PlaylistTracklist.svelte';
  import PlaylistGradeGauge from './PlaylistGradeGauge.svelte';
  import ReplacementPicker from './ReplacementPicker.svelte';
  import BatchScrubProgress from './BatchScrubProgress.svelte';

  let topView: 'browse' | 'sanitize' = 'browse';
  let showConfetti = false;
  let urlInput = '';
  let urlError = '';
  let isExternalPlaylist = false;

  onMount(() => {
    playlistBrowserActions.fetchPlaylists();
  });

  function isValidSpotifyUrl(input: string): boolean {
    return (
      input.includes('open.spotify.com/playlist/') ||
      input.startsWith('spotify:playlist:') ||
      /^[a-zA-Z0-9]{22}$/.test(input.trim())
    );
  }

  function handleUrlSubmit() {
    const trimmed = urlInput.trim();
    if (!trimmed) return;

    if (!isValidSpotifyUrl(trimmed)) {
      urlError = 'Paste a Spotify playlist URL, URI, or ID';
      return;
    }

    urlError = '';
    isExternalPlaylist = true;
    topView = 'sanitize';
    sanitizerActions.reset();
    sanitizerActions.suggestReplacements(trimmed, 'spotify');
  }

  function handleUrlKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') handleUrlSubmit();
  }

  function handleUrlInput(e: Event) {
    urlInput = (e.target as HTMLInputElement).value;
    if (urlError) urlError = '';
  }

  function handleSelectPlaylist(e: CustomEvent<PlaylistSummary>) {
    playlistBrowserActions.selectPlaylist(e.detail);
  }

  function handleBack() {
    playlistBrowserActions.backToGrid();
  }

  function handleSanitize(e: CustomEvent<{ provider: string; playlistName: string }>) {
    topView = 'sanitize';
    sanitizerActions.reset();
    sanitizerActions.suggestReplacements(e.detail.playlistName, e.detail.provider);
  }

  function handleBackToPlaylists() {
    topView = 'browse';
    isExternalPlaylist = false;
    urlInput = '';
    urlError = '';
    sanitizerActions.reset();
  }

  async function handlePublish() {
    await sanitizerActions.confirmAndPublish();
    // Trigger celebration
    showConfetti = true;
    setTimeout(() => { showConfetti = false; }, 3000);
  }

  function handleReset() {
    sanitizerActions.reset();
    topView = 'browse';
    isExternalPlaylist = false;
    urlInput = '';
    urlError = '';
    playlistBrowserActions.backToGrid();
  }

  function formatDuration(ms: number): string {
    const minutes = Math.floor(ms / 60000);
    const seconds = Math.floor((ms % 60000) / 1000);
    return `${minutes}:${seconds.toString().padStart(2, '0')}`;
  }

  function handleProviderFilter(provider: string) {
    playlistBrowserActions.setProviderFilter(
      $playlistBrowserStore.providerFilter === provider ? '' : provider
    );
  }

  function handleToggleSelect(e: CustomEvent<PlaylistSummary>) {
    playlistBrowserActions.togglePlaylistSelection(e.detail.id);
  }

  function handleQuickScrubSingle(e: CustomEvent<PlaylistSummary>) {
    const p = e.detail;
    topView = 'sanitize';
    isExternalPlaylist = false;
    sanitizerActions.reset();
    sanitizerActions.suggestReplacements(
      p.provider_playlist_id || p.playlist_name || p.name,
      p.provider
    );
  }

  function handleScrubSelected() {
    const playlists = $selectedPlaylists;
    if (playlists.length === 0) return;
    playlistBrowserActions.exitSelectionMode();
    topView = 'sanitize';
    isExternalPlaylist = false;
    sanitizerActions.reset();
    sanitizerActions.startBatchScrub(playlists);
  }

  function handleScrubAll() {
    const playlists = $filteredPlaylists;
    if (playlists.length === 0) return;
    playlistBrowserActions.exitSelectionMode();
    topView = 'sanitize';
    isExternalPlaylist = false;
    sanitizerActions.reset();
    sanitizerActions.startBatchScrub(playlists);
  }

  function handleToggleSelectionMode() {
    if ($playlistBrowserStore.selectionMode) {
      playlistBrowserActions.exitSelectionMode();
    } else {
      playlistBrowserActions.enterSelectionMode();
    }
  }

  function handleSelectAll() {
    if ($allFilteredSelected) {
      playlistBrowserActions.deselectAll();
    } else {
      playlistBrowserActions.selectAllFiltered($filteredPlaylists.map((p) => p.id));
    }
  }

  function handleBatchBack() {
    sanitizerActions.clearBatch();
    topView = 'browse';
  }

  // Detect playlists that need a re-sync: metadata says tracks exist but we haven't fetched them yet.
  // Since provider_track_count isn't in the summary, heuristic: some playlists have 0 tracks
  // while others do have tracks, indicating an incomplete sync.
  $: needsResyncPlaylists = (() => {
    const all = $playlistBrowserStore.playlists;
    const hasPopulated = all.some(p => p.total_tracks > 0);
    const hasEmpty = all.some(p => p.total_tracks === 0);
    return hasPopulated && hasEmpty;
  })();

  $: stepIndex = $sanitizerStore.step === 'grade' ? 0 : $sanitizerStore.step === 'replace' ? 1 : 2;
</script>

<div class="sanitizer">
  {#if topView === 'browse'}
    <!-- ======================== BROWSE MODE ======================== -->

    {#if $playlistBrowserStore.view === 'grid'}
      <!-- Grid View -->
      <header class="sanitizer__header">
        <h1 class="sanitizer__title">Your Playlists</h1>
        <p class="sanitizer__subtitle">Browse your synced playlists, or paste any Spotify playlist URL to scrub &amp; clone</p>
      </header>

      <!-- Paste URL bar -->
      <div class="url-bar">
        <div class="url-bar__icon">
          <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
            <path d="M8.5 11.5l3-3m-1.1-2.4L12.3 4.2a2.83 2.83 0 1 1 4 4l-1.9 1.9m-4.8 0L7.7 12a2.83 2.83 0 1 1-4-4l1.9-1.9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </div>
        <input
          type="text"
          class="url-bar__input"
          placeholder="Paste a Spotify playlist URL to scrub & clone..."
          value={urlInput}
          on:input={handleUrlInput}
          on:keydown={handleUrlKeydown}
          on:paste={() => setTimeout(handleUrlSubmit, 0)}
        />
        <button
          type="button"
          class="url-bar__btn"
          disabled={!urlInput.trim()}
          on:click={handleUrlSubmit}
        >
          Scrub It
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><path d="M3 7h8m0 0L8 4m3 3L8 10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>
        </button>
      </div>
      {#if urlError}
        <p class="url-bar__error">{urlError}</p>
      {/if}

      <div class="url-bar__divider">
        <span class="url-bar__divider-line"></span>
        <span class="url-bar__divider-text">or browse your library</span>
        <span class="url-bar__divider-line"></span>
      </div>

      {#if $playlistBrowserStore.error}
        <div class="sanitizer__error">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none"><circle cx="8" cy="8" r="7" stroke="currentColor" stroke-width="1.5"/><path d="M8 5v3M8 10.5v.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>
          <span>{$playlistBrowserStore.error}</span>
          <button type="button" on:click={playlistBrowserActions.clearError} class="sanitizer__error-dismiss">&times;</button>
        </div>
      {/if}

      {#if needsResyncPlaylists}
        <div class="sanitizer__resync-banner">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none"><path d="M8 1a7 7 0 1 0 0 14A7 7 0 0 0 8 1zm0 10.5a.75.75 0 1 1 0-1.5.75.75 0 0 1 0 1.5zM8.75 7.5a.75.75 0 0 1-1.5 0v-3a.75.75 0 0 1 1.5 0v3z" fill="currentColor"/></svg>
          <span>Some playlists need a re-sync to load tracks. Go to Library &gt; Sync to update.</span>
        </div>
      {/if}

      <!-- Filter bar -->
      <div class="browser__filters">
        <div class="browser__search-wrap">
          <svg class="browser__search-icon" width="16" height="16" viewBox="0 0 16 16" fill="none"><circle cx="7" cy="7" r="5" stroke="currentColor" stroke-width="1.5"/><path d="M11 11l3 3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>
          <input
            type="text"
            class="browser__search"
            placeholder="Search playlists..."
            value={$playlistBrowserStore.searchQuery}
            on:input={(e) => playlistBrowserActions.setSearchQuery(e.currentTarget.value)}
          />
        </div>
        <div class="browser__pills">
          <button
            type="button"
            class="browser__pill"
            class:browser__pill--active={$playlistBrowserStore.providerFilter === ''}
            on:click={() => playlistBrowserActions.setProviderFilter('')}
          >All</button>
          <button
            type="button"
            class="browser__pill browser__pill--spotify"
            class:browser__pill--active={$playlistBrowserStore.providerFilter === 'spotify'}
            on:click={() => handleProviderFilter('spotify')}
          >
            <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor"><path d="M7 0a7 7 0 1 0 0 14A7 7 0 0 0 7 0zm3.2 10.1a.43.43 0 0 1-.6.15c-1.63-.99-3.68-1.22-6.1-.67a.44.44 0 0 1-.2-.85c2.64-.6 4.9-.34 6.73.78.2.13.27.4.14.6zm.86-1.9a.55.55 0 0 1-.75.18c-1.86-1.15-4.7-1.48-6.9-.81a.55.55 0 0 1-.32-1.05c2.51-.76 5.63-.39 7.78.93.25.16.33.5.18.75zm.07-1.98C9.06 4.94 5.26 4.82 3.07 5.48a.66.66 0 0 1-.38-1.26c2.51-.76 6.68-.61 9.32 1.02a.66.66 0 0 1-.66 1.14z"/></svg>
            Spotify
          </button>
          <button
            type="button"
            class="browser__pill browser__pill--apple"
            class:browser__pill--active={$playlistBrowserStore.providerFilter === 'apple_music'}
            on:click={() => handleProviderFilter('apple_music')}
          >Apple Music</button>
        </div>
      </div>

      <!-- Toolbar: selection controls + Scrub All -->
      {#if !$playlistBrowserStore.isLoadingPlaylists && $filteredPlaylists.length > 0}
        <div class="browser__toolbar">
          <div class="browser__toolbar-left">
            <button
              type="button"
              class="browser__select-toggle"
              class:browser__select-toggle--active={$playlistBrowserStore.selectionMode}
              on:click={handleToggleSelectionMode}
            >
              <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                <rect x="2" y="2" width="5" height="5" rx="1" stroke="currentColor" stroke-width="1.3"/>
                <rect x="9" y="2" width="5" height="5" rx="1" stroke="currentColor" stroke-width="1.3"/>
                <rect x="2" y="9" width="5" height="5" rx="1" stroke="currentColor" stroke-width="1.3"/>
                <rect x="9" y="9" width="5" height="5" rx="1" stroke="currentColor" stroke-width="1.3"/>
              </svg>
              {$playlistBrowserStore.selectionMode ? 'Cancel' : 'Select'}
            </button>
            {#if $playlistBrowserStore.selectionMode}
              <button
                type="button"
                class="browser__select-all"
                on:click={handleSelectAll}
              >
                {$allFilteredSelected ? 'Deselect All' : 'Select All'}
              </button>
            {/if}
          </div>
          <button
            type="button"
            class="browser__scrub-all"
            on:click={handleScrubAll}
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
              <path d="M13.5 2.5l-1.2 1.2M8.5 7.5l-3.3 3.3a1.5 1.5 0 0 1-2.1-2.1l3.3-3.3m2.1 2.1l3.8-3.8m-3.8 3.8L6.4 5.4m5.9-2.9l.7 2.2-2.2.8m-6.6 5.4L2 13l2.1-2.1" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
            Scrub All
          </button>
        </div>
      {/if}

      {#if $playlistBrowserStore.isLoadingPlaylists}
        <div class="browser__loading">
          <div class="browser__loading-bars">
            <span></span><span></span><span></span><span></span>
          </div>
          <p>Loading playlists...</p>
        </div>
      {:else if $filteredPlaylists.length === 0}
        <div class="sanitizer__empty-state">
          {#if $playlistBrowserStore.playlists.length === 0}
            <svg width="48" height="48" viewBox="0 0 48 48" fill="none"><circle cx="24" cy="24" r="20" stroke="var(--color-text-muted)" stroke-width="1.5" stroke-dasharray="4 3"/><path d="M18 24h12M24 18v12" stroke="var(--color-text-muted)" stroke-width="1.5" stroke-linecap="round"/></svg>
            <p>No synced playlists found.</p>
            <span class="sanitizer__empty-hint">Sync your library from Spotify or Apple Music first.</span>
          {:else}
            <svg width="48" height="48" viewBox="0 0 48 48" fill="none"><circle cx="20" cy="20" r="12" stroke="var(--color-text-muted)" stroke-width="1.5"/><path d="M29 29l10 10" stroke="var(--color-text-muted)" stroke-width="1.5" stroke-linecap="round"/></svg>
            <p>No playlists match your search.</p>
          {/if}
        </div>
      {:else}
        <div class="browser__grid">
          {#each $filteredPlaylists as playlist, i (playlist.id || (playlist.provider + '::' + playlist.playlist_name))}
            <PlaylistCard
              {playlist}
              index={i}
              selectionMode={$playlistBrowserStore.selectionMode}
              selected={$playlistBrowserStore.selectedPlaylistIds.has(playlist.id)}
              on:select={handleSelectPlaylist}
              on:toggleSelect={handleToggleSelect}
              on:quickScrub={handleQuickScrubSingle}
            />
          {/each}
        </div>

        <!-- Sticky action bar when playlists are selected -->
        {#if $selectedPlaylistCount > 0}
          <div class="browser__sticky-bar">
            <div class="browser__sticky-inner">
              <span class="browser__sticky-count">
                {$selectedPlaylistCount} playlist{$selectedPlaylistCount !== 1 ? 's' : ''} selected
              </span>
              <div class="browser__sticky-actions">
                <button
                  type="button"
                  class="browser__sticky-clear"
                  on:click={() => playlistBrowserActions.deselectAll()}
                >Clear</button>
                <button
                  type="button"
                  class="browser__sticky-scrub"
                  on:click={handleScrubSelected}
                >
                  <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                    <path d="M13.5 2.5l-1.2 1.2M8.5 7.5l-3.3 3.3a1.5 1.5 0 0 1-2.1-2.1l3.3-3.3m2.1 2.1l3.8-3.8m-3.8 3.8L6.4 5.4m5.9-2.9l.7 2.2-2.2.8m-6.6 5.4L2 13l2.1-2.1" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"/>
                  </svg>
                  Scrub Selected
                </button>
              </div>
            </div>
          </div>
        {/if}
      {/if}

    {:else}
      <!-- Detail View -->
      {#if $playlistBrowserStore.selectedPlaylist}
        <PlaylistTracklist
          playlist={$playlistBrowserStore.selectedPlaylist}
          tracks={$playlistBrowserStore.tracks}
          isLoading={$playlistBrowserStore.isLoadingTracks}
          on:back={handleBack}
          on:sanitize={handleSanitize}
        />
      {/if}
    {/if}

  {:else}
    <!-- ======================== SANITIZE MODE ======================== -->

    {#if $sanitizerStore.batchScrub}
      <!-- Batch scrub progress view -->
      <button type="button" class="sanitizer__back-btn" on:click={handleBatchBack}>
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none"><path d="M10 12L6 8l4-4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>
        Back to Playlists
      </button>
      <BatchScrubProgress on:back={handleBatchBack} />
    {:else}

    <button type="button" class="sanitizer__back-btn" on:click={handleBackToPlaylists}>
      <svg width="16" height="16" viewBox="0 0 16 16" fill="none"><path d="M10 12L6 8l4-4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>
      Back to Playlists
    </button>

    <header class="sanitizer__header">
      <h1 class="sanitizer__title">{isExternalPlaylist ? 'Scrub & Clone' : 'Playlist Sanitizer'}</h1>
      <p class="sanitizer__subtitle">
        {isExternalPlaylist
          ? 'Scrub blocked artists and clone a clean version to your account'
          : 'Grade, replace, and publish a clean version of your playlist'}
      </p>
    </header>

    <!-- Error banner -->
    {#if $sanitizerStore.error}
      <div class="sanitizer__error">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none"><circle cx="8" cy="8" r="7" stroke="currentColor" stroke-width="1.5"/><path d="M8 5v3M8 10.5v.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>
        <span>{$sanitizerStore.error}</span>
        <button type="button" on:click={sanitizerActions.clearError} class="sanitizer__error-dismiss">&times;</button>
      </div>
    {/if}

    <!-- Step tabs — redesigned with connector line -->
    <nav class="stepper">
      <div class="stepper__track">
        <div class="stepper__track-fill" style="width: {stepIndex * 50}%;"></div>
      </div>

      <button
        type="button"
        class="stepper__step"
        class:stepper__step--active={$sanitizerStore.step === 'grade'}
        class:stepper__step--done={stepIndex > 0}
        on:click={() => sanitizerActions.goToStep('grade')}
      >
        <span class="stepper__num">
          {#if stepIndex > 0}
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><path d="M3 7l3 3 5-5" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>
          {:else}1{/if}
        </span>
        <span class="stepper__label">Grade</span>
      </button>

      <button
        type="button"
        class="stepper__step"
        class:stepper__step--active={$sanitizerStore.step === 'replace'}
        class:stepper__step--done={stepIndex > 1}
        class:stepper__step--disabled={!$hasGrade}
        disabled={!$hasGrade}
        on:click={() => sanitizerActions.goToStep('replace')}
      >
        <span class="stepper__num">
          {#if stepIndex > 1}
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><path d="M3 7l3 3 5-5" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>
          {:else}2{/if}
        </span>
        <span class="stepper__label">Replace</span>
      </button>

      <button
        type="button"
        class="stepper__step"
        class:stepper__step--active={$sanitizerStore.step === 'publish'}
        class:stepper__step--disabled={!$hasPlan}
        disabled={!$hasPlan}
        on:click={() => sanitizerActions.goToStep('publish')}
      >
        <span class="stepper__num">3</span>
        <span class="stepper__label">Publish</span>
      </button>
    </nav>

    <!-- Step 1: Grade -->
    {#if $sanitizerStore.step === 'grade'}
      <section class="sanitizer__section">
        {#if $sanitizerStore.isGrading || $sanitizerStore.isSuggesting}
          <div class="analyzing">
            <div class="analyzing__pulse"></div>
            <div class="analyzing__bars">
              <span></span><span></span><span></span><span></span><span></span>
            </div>
            <p class="analyzing__text">Analyzing playlist...</p>
            <p class="analyzing__sub">Checking tracks against your blocklist</p>
          </div>
        {:else if $sanitizerStore.currentGrade}
          {@const g = $sanitizerStore.currentGrade}
          <div class="grade-result">
            <div class="grade-result__gauge">
              <PlaylistGradeGauge score={g.cleanliness_score} grade={g.grade_letter} />
            </div>

            <div class="grade-result__info">
              <h2 class="grade-result__name">{g.playlist_name}</h2>
              <div class="grade-result__stats">
                <div class="grade-result__stat">
                  <span class="grade-result__stat-val">{g.total_tracks}</span>
                  <span class="grade-result__stat-label">Total</span>
                </div>
                <div class="grade-result__stat grade-result__stat--clean">
                  <span class="grade-result__stat-val">{g.clean_tracks}</span>
                  <span class="grade-result__stat-label">Clean</span>
                </div>
                <div class="grade-result__stat grade-result__stat--blocked">
                  <span class="grade-result__stat-val">{g.blocked_tracks}</span>
                  <span class="grade-result__stat-label">Blocked</span>
                </div>
              </div>
            </div>
          </div>

          {#if g.artist_breakdown.length > 0}
            <div class="blocked-artists">
              <h3 class="section-heading">
                <span class="section-heading__dot section-heading__dot--red"></span>
                Blocked Artists Found
              </h3>
              <div class="blocked-artists__grid">
                {#each g.artist_breakdown as artist, i}
                  <div class="blocked-artist-card" style="animation-delay: {i * 60}ms;">
                    <div class="blocked-artist-card__header">
                      <span class="blocked-artist-card__name">{artist.artist_name}</span>
                      <span class="blocked-artist-card__count">{artist.track_count} track{artist.track_count !== 1 ? 's' : ''}</span>
                    </div>
                    <span class="blocked-artist-card__reason">{artist.block_reason}</span>
                  </div>
                {/each}
              </div>
            </div>
          {/if}

          {#if g.blocked_track_details.length > 0}
            <div class="blocked-tracks">
              <h3 class="section-heading">
                <span class="section-heading__dot section-heading__dot--amber"></span>
                Blocked Tracks
              </h3>
              <div class="blocked-tracks__list">
                {#each g.blocked_track_details as track}
                  <div class="blocked-track-row">
                    <span class="blocked-track-row__pos">#{track.position + 1}</span>
                    <span class="blocked-track-row__name">{track.track_name}</span>
                    <span class="blocked-track-row__artist">{track.all_artist_names.join(', ')}</span>
                    <span class="blocked-track-row__dur">{formatDuration(track.duration_ms)}</span>
                  </div>
                {/each}
              </div>
            </div>

            {#if g.blocked_tracks > 0}
              <div class="sanitizer__actions">
                <button
                  type="button"
                  class="brand-button brand-button--primary"
                  on:click={() => sanitizerActions.goToStep('replace')}
                >
                  Choose Replacements
                  <svg width="16" height="16" viewBox="0 0 16 16" fill="none"><path d="M6 4l4 4-4 4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>
                </button>
              </div>
            {/if}
          {/if}
        {/if}
      </section>
    {/if}

    <!-- Step 2: Replace -->
    {#if $sanitizerStore.step === 'replace'}
      <section class="sanitizer__section">
        {#if $sanitizerStore.replacements.length === 0}
          <div class="sanitizer__empty-state">
            <svg width="48" height="48" viewBox="0 0 48 48" fill="none"><circle cx="24" cy="24" r="20" fill="rgba(34,197,94,0.1)"/><path d="M16 24l6 6 10-10" stroke="#22c55e" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/></svg>
            <p>No blocked tracks to replace. The playlist is clean!</p>
          </div>
        {:else}
          <div class="replace-header">
            <h2 class="section-heading">Choose Replacements</h2>
            <p class="replace-header__sub">
              Pick a replacement for each blocked track, or skip to remove it from the sanitized playlist.
            </p>
          </div>

          <div class="replacement-list">
            {#each $sanitizerStore.replacements as suggestion}
              <ReplacementPicker
                {suggestion}
                selectedId={$sanitizerStore.selectedReplacements[suggestion.original_track_id] || ''}
              />
            {/each}
          </div>

          <div class="target-name">
            <label class="target-name__label" for="target-name-input">Sanitized Playlist Name</label>
            <input
              id="target-name-input"
              type="text"
              class="sanitizer__input"
              bind:value={$sanitizerStore.targetPlaylistName}
              on:input={(e) => sanitizerActions.setTargetName(e.currentTarget.value)}
            />
          </div>

          <div class="sanitizer__actions">
            <button
              type="button"
              class="brand-button brand-button--secondary"
              on:click={() => sanitizerActions.goToStep('grade')}
            >
              <svg width="16" height="16" viewBox="0 0 16 16" fill="none"><path d="M10 12L6 8l4-4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>
              Back
            </button>
            <button
              type="button"
              class="brand-button brand-button--primary"
              on:click={handlePublish}
              disabled={!$allReplacementsSelected || $sanitizerStore.isPublishing}
            >
              {#if $sanitizerStore.isPublishing}
                <span class="brand-button__spinner"></span>
                {isExternalPlaylist ? 'Cloning...' : 'Publishing...'}
              {:else}
                {isExternalPlaylist ? 'Scrub & Clone to My Account' : 'Create Sanitized Playlist'}
                <svg width="16" height="16" viewBox="0 0 16 16" fill="none"><path d="M6 4l4 4-4 4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>
              {/if}
            </button>
          </div>
        {/if}
      </section>
    {/if}

    <!-- Step 3: Publish -->
    {#if $sanitizerStore.step === 'publish'}
      <section class="sanitizer__section">
        {#if $sanitizerStore.isPublishing}
          <div class="publish-status">
            <div class="publish-status__vinyl">
              <div class="publish-status__disc"></div>
            </div>
            <p class="publish-status__text">
              {isExternalPlaylist ? 'Cloning scrubbed playlist...' : 'Creating your sanitized playlist...'}
            </p>
            <p class="publish-status__sub">Adding tracks to Spotify</p>
          </div>
        {:else if $sanitizerStore.publishResult}
          {@const r = $sanitizerStore.publishResult}

          <!-- Confetti -->
          {#if showConfetti}
            <div class="confetti" aria-hidden="true">
              {#each Array(20) as _, i}
                <div
                  class="confetti__piece"
                  style="
                    left: {10 + Math.random() * 80}%;
                    animation-delay: {Math.random() * 0.5}s;
                    --hue: {Math.random() * 360};
                    --drift: {-30 + Math.random() * 60}px;
                  "
                ></div>
              {/each}
            </div>
          {/if}

          <div class="publish-success">
            <div class="publish-success__icon">
              <svg width="56" height="56" viewBox="0 0 56 56" fill="none">
                <circle cx="28" cy="28" r="28" fill="rgba(34,197,94,0.12)"/>
                <circle cx="28" cy="28" r="20" fill="rgba(34,197,94,0.08)"/>
                <path d="M18 28l7 7 13-13" stroke="#22c55e" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" class="publish-success__check"/>
              </svg>
            </div>

            <h2 class="publish-success__title">{isExternalPlaylist ? 'Playlist Cloned!' : 'Playlist Created!'}</h2>
            <p class="publish-success__subtitle">
              {isExternalPlaylist
                ? 'The scrubbed playlist is now in your Spotify library'
                : 'Your sanitized playlist is live on Spotify'}
            </p>

            <div class="publish-success__stats">
              <div class="publish-success__stat">
                <span class="publish-success__stat-val">{r.tracks_kept}</span>
                <span class="publish-success__stat-label">Kept</span>
              </div>
              <div class="publish-success__stat publish-success__stat--replaced">
                <span class="publish-success__stat-val">{r.tracks_replaced}</span>
                <span class="publish-success__stat-label">Replaced</span>
              </div>
              <div class="publish-success__stat publish-success__stat--removed">
                <span class="publish-success__stat-val">{r.tracks_removed}</span>
                <span class="publish-success__stat-label">Removed</span>
              </div>
              <div class="publish-success__stat publish-success__stat--total">
                <span class="publish-success__stat-val">{r.total_tracks}</span>
                <span class="publish-success__stat-label">Total</span>
              </div>
            </div>

            <div class="sanitizer__actions sanitizer__actions--center">
              <a
                href={r.new_playlist_url}
                target="_blank"
                rel="noopener noreferrer"
                class="brand-button brand-button--primary"
              >
                <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor"><path d="M8 0a8 8 0 1 0 0 16A8 8 0 0 0 8 0zm3.66 11.54a.5.5 0 0 1-.68.17c-1.87-1.14-4.23-1.4-7-.77a.5.5 0 0 1-.22-.97c3.04-.7 5.65-.4 7.72.88a.5.5 0 0 1 .17.69zm.98-2.18a.62.62 0 0 1-.86.21c-2.14-1.32-5.4-1.7-7.93-.93a.62.62 0 0 1-.37-1.19c2.89-.88 6.48-.45 8.95 1.06.28.18.37.55.2.85zm.08-2.27C10.16 5.56 5.9 5.42 3.43 6.16a.74.74 0 1 1-.43-1.42c2.83-.86 7.53-.69 10.5 1.15a.74.74 0 0 1-.78 1.26z"/></svg>
                Open in Spotify
              </a>
              <button
                type="button"
                class="brand-button brand-button--secondary"
                on:click={handleReset}
              >
                Back to Playlists
              </button>
            </div>
          </div>
        {:else}
          <div class="sanitizer__empty-state">
            <p>Confirm replacements in the previous step to create your playlist.</p>
          </div>
        {/if}
      </section>
    {/if}
    {/if}
  {/if}
</div>

<style>
  .sanitizer {
    max-width: 68rem;
    margin: 0 auto;
    padding: 1.25rem 1.5rem;
  }

  .sanitizer__header {
    margin-bottom: 1.5rem;
  }

  .sanitizer__title {
    font-size: 1.875rem;
    font-weight: 700;
    color: var(--color-text-primary, #fff);
    margin: 0 0 0.25rem;
    letter-spacing: -0.03em;
    line-height: 1.15;
  }

  .sanitizer__subtitle {
    color: var(--color-text-tertiary, #71717a);
    margin: 0;
    font-size: 0.9375rem;
  }

  .sanitizer__error {
    display: flex;
    align-items: center;
    gap: 0.625rem;
    padding: 0.75rem 1rem;
    border-radius: 0.625rem;
    background: rgba(239, 68, 68, 0.08);
    border: 1px solid rgba(239, 68, 68, 0.2);
    color: var(--color-error, #ef4444);
    margin-bottom: 1rem;
    font-size: 0.875rem;
    animation: slideDown 0.25s ease-out;
  }

  .sanitizer__error svg { flex-shrink: 0; }

  @keyframes slideDown {
    from { opacity: 0; transform: translateY(-8px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .sanitizer__error-dismiss {
    background: none;
    border: none;
    color: inherit;
    cursor: pointer;
    font-size: 1.25rem;
    padding: 0 0.25rem;
    margin-left: auto;
    opacity: 0.6;
    transition: opacity 0.15s;
  }

  .sanitizer__error-dismiss:hover { opacity: 1; }

  .sanitizer__resync-banner {
    display: flex;
    align-items: center;
    gap: 0.625rem;
    padding: 0.75rem 1rem;
    border-radius: 0.625rem;
    background: rgba(234, 179, 8, 0.08);
    border: 1px solid rgba(234, 179, 8, 0.25);
    color: #eab308;
    margin-bottom: 1rem;
    font-size: 0.875rem;
    animation: slideDown 0.25s ease-out;
  }

  .sanitizer__resync-banner svg { flex-shrink: 0; }

  /* ---- Back button ---- */
  .sanitizer__back-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    background: none;
    border: none;
    color: var(--color-text-secondary, #999);
    font-size: 0.875rem;
    cursor: pointer;
    padding: 0.25rem 0;
    margin-bottom: 1rem;
    font-family: inherit;
    transition: color 0.15s;
  }

  .sanitizer__back-btn svg { transition: transform 0.2s; }
  .sanitizer__back-btn:hover { color: var(--color-text-primary, #fff); }
  .sanitizer__back-btn:hover svg { transform: translateX(-2px); }

  /* ---- Browser ---- */

  .browser__filters {
    display: flex;
    gap: 0.75rem;
    margin-bottom: 1.5rem;
    flex-wrap: wrap;
    align-items: center;
  }

  .browser__search-wrap {
    position: relative;
    flex: 1;
    min-width: 200px;
  }

  .browser__search-icon {
    position: absolute;
    left: 0.75rem;
    top: 50%;
    transform: translateY(-50%);
    color: var(--color-text-muted);
    pointer-events: none;
  }

  .browser__search {
    width: 100%;
    padding: 0.5rem 0.875rem 0.5rem 2.25rem;
    border: 1px solid var(--color-border-default, #27272a);
    border-radius: 0.625rem;
    background: var(--input-bg, #1c1c22);
    color: var(--color-text-primary, #fff);
    font-size: 0.8125rem;
    font-family: inherit;
    outline: none;
    transition: border-color 0.15s, box-shadow 0.15s;
  }

  .browser__search:focus {
    border-color: var(--color-brand-primary, #f43f5e);
    box-shadow: 0 0 0 3px rgba(244,63,94,0.08);
  }

  .browser__search::placeholder {
    color: var(--color-text-muted, #52525b);
  }

  .browser__pills {
    display: flex;
    gap: 0.375rem;
  }

  .browser__pill {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.375rem 0.75rem;
    border: 1px solid var(--color-border-subtle, #1f1f22);
    border-radius: 9999px;
    background: transparent;
    color: var(--color-text-secondary, #999);
    font-size: 0.8125rem;
    font-family: inherit;
    cursor: pointer;
    transition: all 0.15s;
  }

  .browser__pill svg { width: 14px; height: 14px; }

  .browser__pill:hover {
    border-color: var(--color-border-hover, #3f3f46);
    color: var(--color-text-primary, #fff);
    background: var(--color-bg-interactive);
  }

  .browser__pill--active {
    background: var(--color-brand-primary, #f43f5e);
    border-color: var(--color-brand-primary, #f43f5e);
    color: #fff;
  }

  .browser__pill--active:hover {
    background: var(--color-brand-primary-hover, #e11d48);
    border-color: var(--color-brand-primary-hover, #e11d48);
    color: #fff;
  }

  .browser__grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(210px, 1fr));
    gap: 1rem;
  }

  .browser__loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    padding: 4rem 1rem;
    color: var(--color-text-secondary, #999);
  }

  .browser__loading-bars {
    display: flex;
    align-items: flex-end;
    gap: 3px;
    height: 1.5rem;
  }

  .browser__loading-bars span {
    width: 3px;
    background: var(--color-brand-primary);
    border-radius: 1px;
    animation: barPulse 1.2s ease-in-out infinite;
  }

  .browser__loading-bars span:nth-child(1) { height: 40%; animation-delay: 0s; }
  .browser__loading-bars span:nth-child(2) { height: 70%; animation-delay: 0.15s; }
  .browser__loading-bars span:nth-child(3) { height: 50%; animation-delay: 0.3s; }
  .browser__loading-bars span:nth-child(4) { height: 80%; animation-delay: 0.45s; }

  @keyframes barPulse {
    0%, 100% { transform: scaleY(0.4); opacity: 0.5; }
    50% { transform: scaleY(1); opacity: 1; }
  }

  /* ---- Stepper ---- */

  .stepper {
    display: flex;
    align-items: center;
    justify-content: space-between;
    position: relative;
    margin-bottom: 2rem;
    padding: 0 1rem;
  }

  .stepper__track {
    position: absolute;
    top: 50%;
    left: 3rem;
    right: 3rem;
    height: 2px;
    background: var(--color-border-subtle, #1f1f22);
    transform: translateY(-50%);
    border-radius: 1px;
    z-index: 0;
  }

  .stepper__track-fill {
    height: 100%;
    background: var(--color-brand-primary, #f43f5e);
    border-radius: 1px;
    transition: width 0.4s cubic-bezier(.4,0,.2,1);
    box-shadow: 0 0 8px rgba(244,63,94,0.3);
  }

  .stepper__step {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    background: none;
    border: none;
    cursor: pointer;
    font-family: inherit;
    color: var(--color-text-tertiary, #666);
    z-index: 1;
    transition: color 0.2s;
    padding: 0;
  }

  .stepper__step:hover:not(:disabled) {
    color: var(--color-text-secondary, #999);
  }

  .stepper__step--active {
    color: var(--color-text-primary, #fff);
  }

  .stepper__step--disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  .stepper__num {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 2.25rem;
    height: 2.25rem;
    border-radius: 50%;
    background: var(--color-bg-interactive, #18181b);
    border: 2px solid var(--color-border-subtle, #1f1f22);
    font-size: 0.8125rem;
    font-weight: 700;
    transition: all 0.25s;
  }

  .stepper__step--active .stepper__num {
    background: var(--color-brand-primary, #f43f5e);
    border-color: var(--color-brand-primary, #f43f5e);
    color: white;
    box-shadow: 0 0 16px rgba(244,63,94,0.35);
  }

  .stepper__step--done .stepper__num {
    background: rgba(34,197,94,0.15);
    border-color: rgba(34,197,94,0.4);
    color: #22c55e;
  }

  .stepper__label {
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  /* ---- Sections ---- */

  .sanitizer__section {
    animation: sectionIn 0.3s ease-out;
  }

  @keyframes sectionIn {
    from { opacity: 0; transform: translateY(6px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .sanitizer__input {
    width: 100%;
    padding: 0.5rem 0.875rem;
    border: 1px solid var(--color-border-default, #27272a);
    border-radius: 0.625rem;
    background: var(--input-bg, #1c1c22);
    color: var(--color-text-primary, #fff);
    font-size: 0.875rem;
    font-family: inherit;
    outline: none;
    transition: border-color 0.15s, box-shadow 0.15s;
  }

  .sanitizer__input:focus {
    border-color: var(--color-brand-primary, #f43f5e);
    box-shadow: 0 0 0 3px rgba(244,63,94,0.08);
  }

  .sanitizer__input::placeholder {
    color: var(--color-text-muted, #52525b);
  }

  .sanitizer__actions {
    display: flex;
    gap: 0.75rem;
    justify-content: flex-end;
    margin-top: 1.75rem;
  }

  .sanitizer__actions--center {
    justify-content: center;
  }

  .sanitizer__empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    padding: 3.5rem 1rem;
    gap: 0.75rem;
    color: var(--color-text-tertiary, #666);
    font-size: 0.9375rem;
  }

  .sanitizer__empty-state p { margin: 0; }

  .sanitizer__empty-hint {
    font-size: 0.8125rem;
    color: var(--color-text-muted);
  }

  /* ---- URL bar ---- */

  .url-bar {
    display: flex;
    align-items: center;
    gap: 0;
    border: 1px solid var(--color-border-default, #27272a);
    border-radius: 0.75rem;
    background: var(--color-bg-elevated, #111113);
    overflow: hidden;
    transition: border-color 0.2s, box-shadow 0.2s;
    margin-bottom: 0.5rem;
  }

  .url-bar:focus-within {
    border-color: var(--color-brand-primary, #f43f5e);
    box-shadow: 0 0 0 3px rgba(244,63,94,0.08);
  }

  .url-bar__icon {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0 0.25rem 0 0.875rem;
    color: var(--color-text-muted, #52525b);
    flex-shrink: 0;
  }

  .url-bar__input {
    flex: 1;
    padding: 0.75rem 0.75rem;
    border: none;
    background: transparent;
    color: var(--color-text-primary, #fff);
    font-size: 0.9375rem;
    font-family: inherit;
    outline: none;
    min-width: 0;
  }

  .url-bar__input::placeholder {
    color: var(--color-text-muted, #52525b);
  }

  .url-bar__btn {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.5rem 1.125rem;
    margin: 0.375rem;
    border: none;
    border-radius: 0.5rem;
    background: var(--color-brand-primary, #f43f5e);
    color: white;
    font-size: 0.8125rem;
    font-weight: 600;
    font-family: inherit;
    cursor: pointer;
    white-space: nowrap;
    transition: background 0.15s, opacity 0.15s, box-shadow 0.15s;
  }

  .url-bar__btn:hover:not(:disabled) {
    background: var(--color-brand-primary-hover, #e11d48);
    box-shadow: 0 0 16px rgba(244,63,94,0.25);
  }

  .url-bar__btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  .url-bar__error {
    color: var(--color-error, #ef4444);
    font-size: 0.8125rem;
    margin: 0.25rem 0 0;
    padding-left: 0.25rem;
  }

  .url-bar__divider {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin: 1.25rem 0 1.5rem;
  }

  .url-bar__divider-line {
    flex: 1;
    height: 1px;
    background: var(--color-border-subtle, #1f1f22);
  }

  .url-bar__divider-text {
    font-size: 0.75rem;
    color: var(--color-text-muted, #52525b);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    font-weight: 600;
    white-space: nowrap;
  }

  /* ---- Analyzing animation ---- */

  .analyzing {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    padding: 3.5rem 1rem;
    position: relative;
  }

  .analyzing__pulse {
    position: absolute;
    width: 120px;
    height: 120px;
    border-radius: 50%;
    background: radial-gradient(circle, rgba(244,63,94,0.08) 0%, transparent 70%);
    animation: pulse 2s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { transform: scale(0.8); opacity: 0.5; }
    50% { transform: scale(1.2); opacity: 1; }
  }

  .analyzing__bars {
    display: flex;
    align-items: flex-end;
    gap: 4px;
    height: 2rem;
    z-index: 1;
  }

  .analyzing__bars span {
    width: 4px;
    background: var(--color-brand-primary);
    border-radius: 2px;
    animation: barPulse 1.2s ease-in-out infinite;
  }

  .analyzing__bars span:nth-child(1) { height: 40%; animation-delay: 0s; }
  .analyzing__bars span:nth-child(2) { height: 65%; animation-delay: 0.1s; }
  .analyzing__bars span:nth-child(3) { height: 85%; animation-delay: 0.2s; }
  .analyzing__bars span:nth-child(4) { height: 55%; animation-delay: 0.3s; }
  .analyzing__bars span:nth-child(5) { height: 70%; animation-delay: 0.4s; }

  .analyzing__text {
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
  }

  .analyzing__sub {
    font-size: 0.8125rem;
    color: var(--color-text-tertiary);
    margin: 0;
  }

  /* ---- Grade result ---- */

  .grade-result {
    display: flex;
    gap: 2rem;
    align-items: center;
    padding: 1.75rem;
    border: 1px solid var(--color-border-subtle, #1f1f22);
    border-radius: 0.875rem;
    background: var(--color-bg-elevated, #111113);
    margin-bottom: 1.75rem;
    animation: gradeIn 0.4s cubic-bezier(.22,1,.36,1) both;
  }

  @keyframes gradeIn {
    from { opacity: 0; transform: scale(0.97); }
    to { opacity: 1; transform: scale(1); }
  }

  .grade-result__info {
    flex: 1;
  }

  .grade-result__name {
    font-size: 1.125rem;
    font-weight: 600;
    margin: 0 0 0.875rem;
    color: var(--color-text-primary, #fff);
  }

  .grade-result__stats {
    display: flex;
    gap: 1.75rem;
  }

  .grade-result__stat {
    display: flex;
    flex-direction: column;
  }

  .grade-result__stat-val {
    font-size: 1.625rem;
    font-weight: 700;
    color: var(--color-text-primary, #fff);
    font-variant-numeric: tabular-nums;
    line-height: 1.1;
  }

  .grade-result__stat--clean .grade-result__stat-val {
    color: #22c55e;
  }

  .grade-result__stat--blocked .grade-result__stat-val {
    color: #ef4444;
  }

  .grade-result__stat-label {
    font-size: 0.6875rem;
    color: var(--color-text-tertiary, #666);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    font-weight: 600;
    margin-top: 0.125rem;
  }

  /* ---- Section headings ---- */

  .section-heading {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.9375rem;
    font-weight: 600;
    margin: 0 0 0.875rem;
    color: var(--color-text-primary, #fff);
  }

  .section-heading__dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .section-heading__dot--red {
    background: #ef4444;
    box-shadow: 0 0 8px rgba(239,68,68,0.4);
  }

  .section-heading__dot--amber {
    background: #f59e0b;
    box-shadow: 0 0 8px rgba(245,158,11,0.4);
  }

  /* ---- Blocked artists ---- */

  .blocked-artists {
    margin-bottom: 1.75rem;
  }

  .blocked-artists__grid {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .blocked-artist-card {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    padding: 0.625rem 0.875rem;
    border: 1px solid rgba(239, 68, 68, 0.15);
    border-radius: 0.5rem;
    background: rgba(239, 68, 68, 0.04);
    transition: border-color 0.15s, background 0.15s;
    animation: cardFadeIn 0.35s ease-out both;
  }

  @keyframes cardFadeIn {
    from { opacity: 0; transform: translateY(4px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .blocked-artist-card:hover {
    border-color: rgba(239, 68, 68, 0.3);
    background: rgba(239, 68, 68, 0.06);
  }

  .blocked-artist-card__header {
    display: flex;
    align-items: center;
    gap: 0.625rem;
  }

  .blocked-artist-card__name {
    font-weight: 600;
    font-size: 0.875rem;
    color: var(--color-text-primary, #fff);
  }

  .blocked-artist-card__count {
    font-size: 0.6875rem;
    color: #ef4444;
    font-weight: 600;
  }

  .blocked-artist-card__reason {
    font-size: 0.6875rem;
    color: var(--color-text-tertiary, #666);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  /* ---- Blocked tracks ---- */

  .blocked-tracks {
    margin-bottom: 1.25rem;
  }

  .blocked-tracks__list {
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
  }

  .blocked-track-row {
    display: grid;
    grid-template-columns: 2.5rem 1fr 1fr auto;
    gap: 0.75rem;
    align-items: center;
    padding: 0.5rem 0.75rem;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    transition: background 0.12s;
  }

  .blocked-track-row:hover {
    background: rgba(255,255,255,0.02);
  }

  .blocked-track-row:nth-child(odd) {
    background: rgba(255, 255, 255, 0.015);
  }

  .blocked-track-row__pos {
    color: var(--color-text-tertiary, #666);
    font-size: 0.75rem;
    font-variant-numeric: tabular-nums;
  }

  .blocked-track-row__name {
    font-weight: 500;
    color: var(--color-text-primary, #fff);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .blocked-track-row__artist {
    color: var(--color-text-secondary, #999);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .blocked-track-row__dur {
    color: var(--color-text-tertiary, #666);
    font-size: 0.75rem;
    font-variant-numeric: tabular-nums;
  }

  /* ---- Replace step ---- */

  .replace-header {
    margin-bottom: 1.25rem;
  }

  .replace-header__sub {
    color: var(--color-text-secondary, #999);
    font-size: 0.875rem;
    margin: 0.25rem 0 0;
  }

  .replacement-list {
    display: flex;
    flex-direction: column;
    gap: 0.875rem;
    margin-bottom: 1.75rem;
  }

  .target-name {
    margin-bottom: 0.5rem;
  }

  .target-name__label {
    display: block;
    font-size: 0.8125rem;
    font-weight: 600;
    color: var(--color-text-secondary, #999);
    margin-bottom: 0.375rem;
  }

  /* ---- Publish step ---- */

  .publish-status {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    padding: 3.5rem 1rem;
  }

  .publish-status__vinyl {
    width: 4rem;
    height: 4rem;
    position: relative;
  }

  .publish-status__disc {
    width: 100%;
    height: 100%;
    border-radius: 50%;
    background:
      radial-gradient(circle at center, #1a1a1a 20%, transparent 21%),
      conic-gradient(from 0deg, #1a1a1a, #2a2a2a, #1a1a1a, #252525, #1a1a1a);
    animation: spin 2s linear infinite;
    box-shadow: 0 0 20px rgba(244,63,94,0.15);
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .publish-status__text {
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
  }

  .publish-status__sub {
    font-size: 0.8125rem;
    color: var(--color-text-tertiary);
    margin: 0;
  }

  /* Success */

  .publish-success {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    padding: 2.5rem 1rem;
    animation: successIn 0.5s cubic-bezier(.22,1,.36,1) both;
    position: relative;
  }

  @keyframes successIn {
    from { opacity: 0; transform: scale(0.95) translateY(8px); }
    to { opacity: 1; transform: scale(1) translateY(0); }
  }

  .publish-success__icon {
    margin-bottom: 1.25rem;
  }

  .publish-success__check {
    stroke-dasharray: 40;
    stroke-dashoffset: 40;
    animation: drawCheck 0.6s ease-out 0.3s forwards;
  }

  @keyframes drawCheck {
    to { stroke-dashoffset: 0; }
  }

  .publish-success__title {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--color-text-primary, #fff);
    margin: 0 0 0.375rem;
    letter-spacing: -0.02em;
  }

  .publish-success__subtitle {
    color: var(--color-text-tertiary);
    font-size: 0.875rem;
    margin: 0 0 1.75rem;
  }

  .publish-success__stats {
    display: flex;
    gap: 2.25rem;
    margin-bottom: 2rem;
  }

  .publish-success__stat {
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .publish-success__stat-val {
    font-size: 2rem;
    font-weight: 700;
    color: var(--color-text-primary, #fff);
    font-variant-numeric: tabular-nums;
    line-height: 1.1;
  }

  .publish-success__stat--replaced .publish-success__stat-val { color: #3b82f6; }
  .publish-success__stat--removed .publish-success__stat-val { color: #f59e0b; }
  .publish-success__stat--total .publish-success__stat-val { color: #22c55e; }

  .publish-success__stat-label {
    font-size: 0.6875rem;
    color: var(--color-text-tertiary, #666);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    font-weight: 600;
    margin-top: 0.125rem;
  }

  /* ---- Confetti ---- */

  .confetti {
    position: fixed;
    inset: 0;
    pointer-events: none;
    z-index: 100;
    overflow: hidden;
  }

  .confetti__piece {
    position: absolute;
    top: -10px;
    width: 8px;
    height: 8px;
    background: hsl(var(--hue), 80%, 60%);
    border-radius: 1px;
    animation: confettiFall 2.5s ease-out forwards;
  }

  @keyframes confettiFall {
    0% {
      transform: translateY(0) translateX(0) rotate(0deg);
      opacity: 1;
    }
    100% {
      transform: translateY(100vh) translateX(var(--drift)) rotate(720deg);
      opacity: 0;
    }
  }

  /* ---- Mobile ---- */

  @media (max-width: 640px) {
    .sanitizer { padding: 1rem; }

    .url-bar__btn span { display: none; }
    .url-bar__input { font-size: 0.8125rem; padding: 0.625rem 0.5rem; }

    .browser__filters {
      flex-direction: column;
    }

    .browser__search-wrap {
      width: 100%;
    }

    .browser__grid {
      grid-template-columns: 1fr 1fr;
      gap: 0.75rem;
    }

    .grade-result {
      flex-direction: column;
      text-align: center;
    }

    .grade-result__stats {
      justify-content: center;
    }

    .blocked-track-row {
      grid-template-columns: 2rem 1fr auto;
    }

    .blocked-track-row__artist {
      display: none;
    }

    .stepper { padding: 0; }
    .stepper__track { left: 2rem; right: 2rem; }

    .publish-success__stats { gap: 1.25rem; }
    .publish-success__stat-val { font-size: 1.5rem; }

    .browser__toolbar { flex-direction: column; gap: 0.5rem; }
    .browser__toolbar-left { width: 100%; }
    .browser__scrub-all { width: 100%; justify-content: center; }
  }

  /* ---- Toolbar ---- */
  .browser__toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    margin-bottom: 1rem;
  }

  .browser__toolbar-left {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .browser__select-toggle {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.4rem 0.75rem;
    border-radius: 0.5rem;
    border: 1px solid var(--color-border-subtle);
    background: var(--color-bg-elevated);
    color: var(--color-text-secondary);
    font-size: 0.8125rem;
    font-weight: 600;
    font-family: inherit;
    cursor: pointer;
    transition: background 0.15s, border-color 0.15s, color 0.15s;
  }

  .browser__select-toggle:hover {
    border-color: var(--color-border-hover);
    color: var(--color-text-primary);
  }

  .browser__select-toggle--active {
    background: rgba(225,29,72,0.1);
    border-color: rgba(225,29,72,0.3);
    color: #e11d48;
  }

  .browser__select-all {
    padding: 0.4rem 0.75rem;
    border-radius: 0.5rem;
    border: none;
    background: transparent;
    color: var(--color-text-tertiary);
    font-size: 0.8125rem;
    font-weight: 500;
    font-family: inherit;
    cursor: pointer;
    transition: color 0.15s;
  }

  .browser__select-all:hover {
    color: var(--color-text-primary);
  }

  .browser__scrub-all {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.4rem 0.875rem;
    border-radius: 0.5rem;
    border: none;
    background: linear-gradient(135deg, #e11d48, #be123c);
    color: #fff;
    font-size: 0.8125rem;
    font-weight: 700;
    font-family: inherit;
    cursor: pointer;
    transition: transform 0.15s, filter 0.15s;
    box-shadow: 0 2px 8px rgba(225,29,72,0.25);
  }

  .browser__scrub-all:hover {
    transform: translateY(-1px);
    filter: brightness(1.1);
  }

  .browser__scrub-all:active {
    transform: translateY(0) scale(0.98);
  }

  /* ---- Sticky action bar ---- */
  .browser__sticky-bar {
    position: sticky;
    bottom: 0;
    z-index: 20;
    padding: 0.75rem 0;
    margin-top: 1rem;
  }

  .browser__sticky-bar::before {
    content: '';
    position: absolute;
    inset: -1.5rem 0 0;
    background: linear-gradient(to top, var(--color-bg-base) 60%, transparent);
    pointer-events: none;
  }

  .browser__sticky-inner {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.75rem 1rem;
    border-radius: 0.75rem;
    background: rgba(39,39,42,0.85);
    backdrop-filter: blur(16px);
    -webkit-backdrop-filter: blur(16px);
    border: 1px solid rgba(225,29,72,0.2);
    box-shadow: 0 -4px 24px rgba(0,0,0,0.3);
    animation: slideUp 0.25s cubic-bezier(.22,1,.36,1);
  }

  @keyframes slideUp {
    from { opacity: 0; transform: translateY(12px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .browser__sticky-count {
    font-size: 0.8125rem;
    font-weight: 600;
    color: var(--color-text-secondary);
  }

  .browser__sticky-actions {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .browser__sticky-clear {
    padding: 0.375rem 0.75rem;
    border: none;
    background: transparent;
    color: var(--color-text-tertiary);
    font-size: 0.8125rem;
    font-weight: 500;
    font-family: inherit;
    cursor: pointer;
    transition: color 0.15s;
  }

  .browser__sticky-clear:hover {
    color: var(--color-text-primary);
  }

  .browser__sticky-scrub {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.5rem 1rem;
    border-radius: 0.5rem;
    border: none;
    background: linear-gradient(135deg, #e11d48, #be123c);
    color: #fff;
    font-size: 0.8125rem;
    font-weight: 700;
    font-family: inherit;
    cursor: pointer;
    transition: transform 0.15s, filter 0.15s;
    box-shadow: 0 2px 8px rgba(225,29,72,0.3);
  }

  .browser__sticky-scrub:hover {
    transform: translateY(-1px);
    filter: brightness(1.1);
  }

  .browser__sticky-scrub:active {
    transform: translateY(0) scale(0.98);
  }
</style>
