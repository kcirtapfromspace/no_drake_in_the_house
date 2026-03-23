<script lang="ts">
  import { onMount } from 'svelte';
  import { sanitizerStore, sanitizerActions, hasGrade, hasPlan, allReplacementsSelected } from '../stores/sanitizer';
  import { playlistBrowserStore, filteredPlaylists, playlistBrowserActions, type PlaylistSummary } from '../stores/playlist-browser';
  import PlaylistCard from './PlaylistCard.svelte';
  import PlaylistTracklist from './PlaylistTracklist.svelte';
  import PlaylistGradeGauge from './PlaylistGradeGauge.svelte';
  import ReplacementPicker from './ReplacementPicker.svelte';

  let topView: 'browse' | 'sanitize' = 'browse';

  onMount(() => {
    playlistBrowserActions.fetchPlaylists();
  });

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
    sanitizerActions.reset();
  }

  async function handlePublish() {
    await sanitizerActions.confirmAndPublish();
  }

  function handleReset() {
    sanitizerActions.reset();
    topView = 'browse';
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
</script>

<div class="sanitizer">
  {#if topView === 'browse'}
    <!-- ======================== BROWSE MODE ======================== -->

    {#if $playlistBrowserStore.view === 'grid'}
      <!-- Grid View -->
      <header class="sanitizer__header">
        <h1 class="sanitizer__title">Your Playlists</h1>
        <p class="sanitizer__subtitle">Browse, grade, and sanitize your synced playlists</p>
      </header>

      {#if $playlistBrowserStore.error}
        <div class="sanitizer__error">
          <span>{$playlistBrowserStore.error}</span>
          <button type="button" on:click={playlistBrowserActions.clearError} class="sanitizer__error-dismiss">&times;</button>
        </div>
      {/if}

      <!-- Filter bar -->
      <div class="browser__filters">
        <input
          type="text"
          class="sanitizer__input browser__search"
          placeholder="Search playlists..."
          value={$playlistBrowserStore.searchQuery}
          on:input={(e) => playlistBrowserActions.setSearchQuery(e.currentTarget.value)}
        />
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
          >Spotify</button>
          <button
            type="button"
            class="browser__pill browser__pill--apple"
            class:browser__pill--active={$playlistBrowserStore.providerFilter === 'apple_music'}
            on:click={() => handleProviderFilter('apple_music')}
          >Apple Music</button>
        </div>
      </div>

      {#if $playlistBrowserStore.isLoadingPlaylists}
        <div class="browser__loading">
          <div class="brand-button__spinner browser__spinner"></div>
          <p>Loading playlists...</p>
        </div>
      {:else if $filteredPlaylists.length === 0}
        <p class="sanitizer__empty">
          {#if $playlistBrowserStore.playlists.length === 0}
            No synced playlists found. Sync your library from Spotify or Apple Music first.
          {:else}
            No playlists match your search.
          {/if}
        </p>
      {:else}
        <div class="browser__grid">
          {#each $filteredPlaylists as playlist (playlist.provider + '::' + playlist.playlist_name)}
            <PlaylistCard {playlist} on:select={handleSelectPlaylist} />
          {/each}
        </div>
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

    <button type="button" class="tracklist__back" on:click={handleBackToPlaylists}>
      &larr; Back to Playlists
    </button>

    <header class="sanitizer__header">
      <h1 class="sanitizer__title">Playlist Sanitizer</h1>
      <p class="sanitizer__subtitle">Grade, replace, and publish a clean version of your playlist</p>
    </header>

    <!-- Error banner -->
    {#if $sanitizerStore.error}
      <div class="sanitizer__error">
        <span>{$sanitizerStore.error}</span>
        <button type="button" on:click={sanitizerActions.clearError} class="sanitizer__error-dismiss">&times;</button>
      </div>
    {/if}

    <!-- Step tabs -->
    <nav class="sanitizer__tabs">
      <button
        type="button"
        class="sanitizer__tab"
        class:sanitizer__tab--active={$sanitizerStore.step === 'grade'}
        on:click={() => sanitizerActions.goToStep('grade')}
      >
        <span class="sanitizer__tab-num">1</span> Grade
      </button>
      <button
        type="button"
        class="sanitizer__tab"
        class:sanitizer__tab--active={$sanitizerStore.step === 'replace'}
        class:sanitizer__tab--disabled={!$hasGrade}
        disabled={!$hasGrade}
        on:click={() => sanitizerActions.goToStep('replace')}
      >
        <span class="sanitizer__tab-num">2</span> Replace
      </button>
      <button
        type="button"
        class="sanitizer__tab"
        class:sanitizer__tab--active={$sanitizerStore.step === 'publish'}
        class:sanitizer__tab--disabled={!$hasPlan}
        disabled={!$hasPlan}
        on:click={() => sanitizerActions.goToStep('publish')}
      >
        <span class="sanitizer__tab-num">3</span> Publish
      </button>
    </nav>

    <!-- Step 1: Grade -->
    {#if $sanitizerStore.step === 'grade'}
      <section class="sanitizer__section">
        {#if $sanitizerStore.isGrading || $sanitizerStore.isSuggesting}
          <div class="browser__loading">
            <div class="brand-button__spinner browser__spinner"></div>
            <p>Analyzing playlist...</p>
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
              <h3 class="section-heading">Blocked Artists Found</h3>
              <div class="blocked-artists__grid">
                {#each g.artist_breakdown as artist}
                  <div class="blocked-artist-card">
                    <span class="blocked-artist-card__name">{artist.artist_name}</span>
                    <span class="blocked-artist-card__count">{artist.track_count} track{artist.track_count !== 1 ? 's' : ''}</span>
                    <span class="blocked-artist-card__reason">{artist.block_reason}</span>
                  </div>
                {/each}
              </div>
            </div>
          {/if}

          {#if g.blocked_track_details.length > 0}
            <div class="blocked-tracks">
              <h3 class="section-heading">Blocked Tracks</h3>
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
          <p class="sanitizer__empty">No blocked tracks to replace. The playlist is clean!</p>
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
                Publishing...
              {:else}
                Create Sanitized Playlist
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
            <div class="brand-button__spinner publish-status__spinner"></div>
            <p>Creating your sanitized playlist...</p>
          </div>
        {:else if $sanitizerStore.publishResult}
          {@const r = $sanitizerStore.publishResult}
          <div class="publish-success">
            <div class="publish-success__icon">
              <svg width="48" height="48" viewBox="0 0 48 48" fill="none">
                <circle cx="24" cy="24" r="24" fill="var(--color-brand-success, #22c55e)" opacity="0.15"/>
                <path d="M14 24l7 7 13-13" stroke="var(--color-brand-success, #22c55e)" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
            </div>
            <h2 class="publish-success__title">Playlist Created!</h2>
            <div class="publish-success__stats">
              <div class="publish-success__stat">
                <span class="publish-success__stat-val">{r.tracks_kept}</span>
                <span class="publish-success__stat-label">Kept</span>
              </div>
              <div class="publish-success__stat">
                <span class="publish-success__stat-val">{r.tracks_replaced}</span>
                <span class="publish-success__stat-label">Replaced</span>
              </div>
              <div class="publish-success__stat">
                <span class="publish-success__stat-val">{r.tracks_removed}</span>
                <span class="publish-success__stat-label">Removed</span>
              </div>
              <div class="publish-success__stat">
                <span class="publish-success__stat-val">{r.total_tracks}</span>
                <span class="publish-success__stat-label">Total</span>
              </div>
            </div>
            <div class="sanitizer__actions">
              <a
                href={r.new_playlist_url}
                target="_blank"
                rel="noopener noreferrer"
                class="brand-button brand-button--primary"
              >
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
          <p class="sanitizer__empty">Confirm replacements in the previous step to create your playlist.</p>
        {/if}
      </section>
    {/if}
  {/if}
</div>

<style>
  .sanitizer {
    max-width: 56rem;
    margin: 0 auto;
    padding: 1.5rem;
  }

  .sanitizer__header {
    margin-bottom: 1.5rem;
  }

  .sanitizer__title {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--color-text-primary, #fff);
    margin: 0 0 0.25rem;
  }

  .sanitizer__subtitle {
    color: var(--color-text-secondary, #999);
    margin: 0;
    font-size: 0.9375rem;
  }

  .sanitizer__error {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1rem;
    border-radius: 0.5rem;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    color: var(--color-brand-danger, #ef4444);
    margin-bottom: 1rem;
    font-size: 0.875rem;
  }

  .sanitizer__error-dismiss {
    background: none;
    border: none;
    color: inherit;
    cursor: pointer;
    font-size: 1.25rem;
    padding: 0 0.25rem;
  }

  /* ---- Browser ---- */

  .browser__filters {
    display: flex;
    gap: 0.75rem;
    margin-bottom: 1.5rem;
    flex-wrap: wrap;
  }

  .browser__search {
    flex: 1;
    min-width: 200px;
  }

  .browser__pills {
    display: flex;
    gap: 0.375rem;
  }

  .browser__pill {
    padding: 0.375rem 0.75rem;
    border: 1px solid var(--color-border-subtle, #333);
    border-radius: 9999px;
    background: transparent;
    color: var(--color-text-secondary, #999);
    font-size: 0.8125rem;
    font-family: inherit;
    cursor: pointer;
    transition: all 0.15s;
  }

  .browser__pill:hover {
    border-color: var(--color-border-hover, #555);
    color: var(--color-text-primary, #fff);
  }

  .browser__pill--active {
    background: var(--color-brand-primary, #8b5cf6);
    border-color: var(--color-brand-primary, #8b5cf6);
    color: #fff;
  }

  .browser__pill--active:hover {
    background: var(--color-brand-primary, #8b5cf6);
    border-color: var(--color-brand-primary, #8b5cf6);
    color: #fff;
  }

  .browser__grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 0.875rem;
  }

  .browser__loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.75rem;
    padding: 3rem 1rem;
    color: var(--color-text-secondary, #999);
  }

  .browser__spinner {
    width: 1.5rem;
    height: 1.5rem;
  }

  /* ---- Sanitizer tabs & steps (preserved) ---- */

  .sanitizer__tabs {
    display: flex;
    gap: 0.25rem;
    margin-bottom: 1.5rem;
    border-bottom: 1px solid var(--color-border-subtle, #333);
    padding-bottom: -1px;
  }

  .sanitizer__tab {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1.25rem;
    border: none;
    background: none;
    color: var(--color-text-tertiary, #666);
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    border-bottom: 2px solid transparent;
    transition: color 0.15s, border-color 0.15s;
    font-family: inherit;
  }

  .sanitizer__tab:hover:not(:disabled) {
    color: var(--color-text-secondary, #999);
  }

  .sanitizer__tab--active {
    color: var(--color-text-primary, #fff);
    border-bottom-color: var(--color-brand-primary, #8b5cf6);
  }

  .sanitizer__tab--disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .sanitizer__tab-num {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 1.375rem;
    height: 1.375rem;
    border-radius: 50%;
    background: var(--color-surface-tertiary, #2a2a3e);
    font-size: 0.75rem;
    font-weight: 700;
  }

  .sanitizer__tab--active .sanitizer__tab-num {
    background: var(--color-brand-primary, #8b5cf6);
    color: white;
  }

  .sanitizer__section {
    animation: fadeIn 0.2s ease-out;
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(4px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .sanitizer__input {
    flex: 1;
    padding: 0.625rem 0.875rem;
    border: 1px solid var(--color-border-subtle, #333);
    border-radius: 0.5rem;
    background: var(--color-surface-secondary, #1a1a2e);
    color: var(--color-text-primary, #fff);
    font-size: 0.9375rem;
    font-family: inherit;
    outline: none;
    transition: border-color 0.15s;
  }

  .sanitizer__input:focus {
    border-color: var(--color-brand-primary, #8b5cf6);
  }

  .sanitizer__input::placeholder {
    color: var(--color-text-tertiary, #666);
  }

  .sanitizer__actions {
    display: flex;
    gap: 0.75rem;
    justify-content: flex-end;
    margin-top: 1.5rem;
  }

  .sanitizer__empty {
    text-align: center;
    color: var(--color-text-tertiary, #666);
    padding: 3rem 1rem;
    font-size: 0.9375rem;
  }

  /* Grade result */
  .grade-result {
    display: flex;
    gap: 2rem;
    align-items: center;
    padding: 1.5rem;
    border: 1px solid var(--color-border-subtle, #333);
    border-radius: 0.75rem;
    background: var(--color-surface-secondary, #1a1a2e);
    margin-bottom: 1.5rem;
  }

  .grade-result__info {
    flex: 1;
  }

  .grade-result__name {
    font-size: 1.125rem;
    font-weight: 600;
    margin: 0 0 0.75rem;
    color: var(--color-text-primary, #fff);
  }

  .grade-result__stats {
    display: flex;
    gap: 1.5rem;
  }

  .grade-result__stat {
    display: flex;
    flex-direction: column;
  }

  .grade-result__stat-val {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--color-text-primary, #fff);
  }

  .grade-result__stat--clean .grade-result__stat-val {
    color: var(--color-brand-success, #22c55e);
  }

  .grade-result__stat--blocked .grade-result__stat-val {
    color: var(--color-brand-danger, #ef4444);
  }

  .grade-result__stat-label {
    font-size: 0.75rem;
    color: var(--color-text-tertiary, #666);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  /* Blocked artists */
  .section-heading {
    font-size: 0.9375rem;
    font-weight: 600;
    margin: 0 0 0.75rem;
    color: var(--color-text-primary, #fff);
  }

  .blocked-artists {
    margin-bottom: 1.5rem;
  }

  .blocked-artists__grid {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .blocked-artist-card {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    border: 1px solid rgba(239, 68, 68, 0.2);
    border-radius: 0.375rem;
    background: rgba(239, 68, 68, 0.05);
  }

  .blocked-artist-card__name {
    font-weight: 600;
    font-size: 0.875rem;
    color: var(--color-text-primary, #fff);
  }

  .blocked-artist-card__count {
    font-size: 0.75rem;
    color: var(--color-brand-danger, #ef4444);
  }

  .blocked-artist-card__reason {
    font-size: 0.6875rem;
    color: var(--color-text-tertiary, #666);
    text-transform: uppercase;
  }

  /* Blocked tracks list */
  .blocked-tracks {
    margin-bottom: 1rem;
  }

  .blocked-tracks__list {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .blocked-track-row {
    display: grid;
    grid-template-columns: 2.5rem 1fr 1fr auto;
    gap: 0.75rem;
    align-items: center;
    padding: 0.5rem 0.75rem;
    border-radius: 0.375rem;
    font-size: 0.875rem;
  }

  .blocked-track-row:nth-child(odd) {
    background: rgba(255, 255, 255, 0.02);
  }

  .blocked-track-row__pos {
    color: var(--color-text-tertiary, #666);
    font-size: 0.75rem;
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
  }

  /* Replace step */
  .replace-header {
    margin-bottom: 1rem;
  }

  .replace-header__sub {
    color: var(--color-text-secondary, #999);
    font-size: 0.875rem;
    margin: 0;
  }

  .replacement-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    margin-bottom: 1.5rem;
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

  /* Publish step */
  .publish-status {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    padding: 3rem 1rem;
    color: var(--color-text-secondary, #999);
  }

  .publish-status__spinner {
    width: 2rem;
    height: 2rem;
  }

  .publish-success {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    padding: 2rem 1rem;
  }

  .publish-success__icon {
    margin-bottom: 1rem;
  }

  .publish-success__title {
    font-size: 1.375rem;
    font-weight: 700;
    color: var(--color-text-primary, #fff);
    margin: 0 0 1.5rem;
  }

  .publish-success__stats {
    display: flex;
    gap: 2rem;
    margin-bottom: 1.5rem;
  }

  .publish-success__stat {
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .publish-success__stat-val {
    font-size: 1.75rem;
    font-weight: 700;
    color: var(--color-text-primary, #fff);
  }

  .publish-success__stat-label {
    font-size: 0.75rem;
    color: var(--color-text-tertiary, #666);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  /* Back button (reused from tracklist) */
  .tracklist__back {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
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

  .tracklist__back:hover {
    color: var(--color-text-primary, #fff);
  }

  @media (max-width: 640px) {
    .browser__filters {
      flex-direction: column;
    }

    .browser__search {
      width: 100%;
    }

    .browser__grid {
      grid-template-columns: 1fr;
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
  }
</style>
