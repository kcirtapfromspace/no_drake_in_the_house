<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { PlaylistSummary, PlaylistTrack } from '../stores/playlist-browser';
  import { trackStats } from '../stores/playlist-browser';
  import PlaylistGradeGauge from './PlaylistGradeGauge.svelte';

  export let playlist: PlaylistSummary;
  export let tracks: PlaylistTrack[] = [];
  export let isLoading: boolean = false;

  const dispatch = createEventDispatcher<{
    back: void;
    sanitize: { provider: string; playlistName: string };
  }>();

  $: providerColor = playlist.provider === 'spotify' ? 'var(--color-spotify)' : 'var(--color-apple)';
  $: providerLabel = playlist.provider === 'spotify' ? 'Spotify' : 'Apple Music';
  $: stats = $trackStats;
  $: images = playlist.cover_images || [];
  $: heroImage = images[0] || '';

  // Gradient fallback
  function hashStr(s: string): number {
    let h = 0;
    for (let i = 0; i < s.length; i++) h = ((h << 5) - h + s.charCodeAt(i)) | 0;
    return Math.abs(h);
  }
  $: hue1 = hashStr(playlist.playlist_name) % 360;
  $: hue2 = (hue1 + 40 + (hashStr(playlist.provider) % 60)) % 360;
</script>

<div class="tl">
  <!-- Hero header -->
  <header class="tl__hero">
    {#if heroImage}
      <img class="tl__hero-bg" src={heroImage} alt="" />
    {:else}
      <div class="tl__hero-bg tl__hero-bg--gradient" style="background: linear-gradient(135deg, hsl({hue1},55%,22%) 0%, hsl({hue2},40%,12%) 100%);"></div>
    {/if}
    <div class="tl__hero-scrim"></div>

    <div class="tl__hero-content">
      <button type="button" class="tl__back" on:click={() => dispatch('back')}>
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none"><path d="M10 12L6 8l4-4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>
        All Playlists
      </button>

      <div class="tl__hero-main">
        <div class="tl__hero-info">
          <div class="tl__hero-badge-row">
            <span class="tl__provider-pill" style="background: {providerColor};">{providerLabel}</span>
            <span class="tl__track-count">{playlist.total_tracks} tracks</span>
          </div>
          <h1 class="tl__title">{playlist.playlist_name}</h1>

          <div class="tl__stat-chips">
            <div class="tl__chip tl__chip--clean">
              <span class="tl__chip-val">{stats.clean}</span>
              <span class="tl__chip-label">Clean</span>
            </div>
            <div class="tl__chip tl__chip--flagged">
              <span class="tl__chip-val">{stats.flagged}</span>
              <span class="tl__chip-label">Flagged</span>
            </div>
            <div class="tl__chip tl__chip--blocked">
              <span class="tl__chip-val">{stats.blocked}</span>
              <span class="tl__chip-label">Blocked</span>
            </div>
          </div>
        </div>

        <div class="tl__hero-gauge">
          <PlaylistGradeGauge score={Math.round(playlist.clean_ratio * 100)} grade={playlist.grade} size={110} />
        </div>
      </div>
    </div>
  </header>

  <!-- Track list -->
  {#if isLoading}
    <div class="tl__loading">
      <div class="brand-button__spinner tl__spinner"></div>
      <p>Loading tracks...</p>
    </div>
  {:else if tracks.length === 0}
    <p class="tl__empty">No tracks found in this playlist.</p>
  {:else}
    <!-- Column header -->
    <div class="tl__col-header">
      <span class="tl__col-num">#</span>
      <span class="tl__col-title">Title</span>
      <span class="tl__col-artist">Artist</span>
      <span class="tl__col-album">Album</span>
    </div>

    <div class="tl__rows">
      {#each tracks as track, i}
        <div class="tl__row" class:tl__row--flagged={track.status === 'flagged'} class:tl__row--blocked={track.status === 'blocked'} style="--ri: {i}">
          <span class="tl__num">{track.position}</span>

          <div class="tl__title-cell">
            {#if track.artist_image_url}
              <img class="tl__avatar" src={track.artist_image_url} alt="" loading="lazy" />
            {:else}
              <div class="tl__avatar tl__avatar--placeholder">
                <span class="tl__status-dot tl__status-dot--{track.status}"></span>
              </div>
            {/if}
            <div class="tl__title-text">
              <span class="tl__track-name">{track.track_name}</span>
            </div>
          </div>

          <span class="tl__artist tl__artist--{track.status}">{track.artist_name}</span>

          <span class="tl__album">{track.album_name || ''}</span>

          {#if track.status !== 'clean'}
            <span class="tl__badge tl__badge--{track.status}">{track.status}</span>
          {/if}
        </div>
      {/each}
    </div>
  {/if}

  {#if playlist.flagged_tracks > 0 && !isLoading}
    <div class="tl__footer">
      <button
        type="button"
        class="tl__sanitize-btn"
        on:click={() => dispatch('sanitize', { provider: playlist.provider, playlistName: playlist.playlist_name })}
      >
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none"><path d="M8 2v4M8 10v4M2 8h4M10 8h4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>
        Sanitize This Playlist
      </button>
    </div>
  {/if}
</div>

<style>
  .tl { max-width: 64rem; margin: 0 auto; }

  /* ---- Hero ---- */
  .tl__hero {
    position: relative;
    border-radius: 0.75rem;
    overflow: hidden;
    margin-bottom: 1.25rem;
    min-height: 220px;
  }

  .tl__hero-bg {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    max-width: none;
    max-height: none;
    filter: blur(20px) saturate(1.2);
    transform: scale(1.15);
  }

  .tl__hero-bg--gradient { filter: none; transform: none; }

  .tl__hero-scrim {
    position: absolute;
    inset: 0;
    background: linear-gradient(to bottom,
      rgba(9,9,11,0.3) 0%,
      rgba(9,9,11,0.75) 60%,
      rgba(9,9,11,0.95) 100%
    );
  }

  .tl__hero-content {
    position: relative;
    padding: 1.25rem 1.5rem 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
    z-index: 1;
  }

  .tl__back {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    background: none;
    border: none;
    color: var(--color-text-secondary);
    font-size: 0.8125rem;
    cursor: pointer;
    padding: 0;
    font-family: inherit;
    transition: color 0.15s;
    align-self: flex-start;
  }

  .tl__back svg { width: 16px; height: 16px; max-width: none; max-height: none; }
  .tl__back:hover { color: var(--color-text-primary); }

  .tl__hero-main {
    display: flex;
    align-items: flex-end;
    gap: 1.5rem;
  }

  .tl__hero-info { flex: 1; }

  .tl__hero-badge-row {
    display: flex;
    align-items: center;
    gap: 0.625rem;
    margin-bottom: 0.5rem;
  }

  .tl__provider-pill {
    padding: 0.15rem 0.5rem;
    border-radius: 9999px;
    font-size: 0.625rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: #fff;
  }

  .tl__track-count {
    font-size: 0.75rem;
    color: var(--color-text-tertiary);
  }

  .tl__title {
    font-size: 1.625rem;
    font-weight: 700;
    color: var(--color-text-primary);
    margin: 0 0 0.75rem;
    line-height: 1.2;
    letter-spacing: -0.02em;
  }

  .tl__stat-chips {
    display: flex;
    gap: 0.5rem;
  }

  .tl__chip {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    padding: 0.25rem 0.625rem;
    border-radius: 9999px;
    font-size: 0.75rem;
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
    border: 1px solid rgba(255,255,255,0.06);
  }

  .tl__chip--clean { background: rgba(34,197,94,0.12); }
  .tl__chip--flagged { background: rgba(239,68,68,0.12); }
  .tl__chip--blocked { background: rgba(245,158,11,0.12); }

  .tl__chip-val { font-weight: 700; }
  .tl__chip--clean .tl__chip-val { color: var(--color-success); }
  .tl__chip--flagged .tl__chip-val { color: var(--color-error); }
  .tl__chip--blocked .tl__chip-val { color: var(--color-warning); }

  .tl__chip-label { color: var(--color-text-tertiary); font-weight: 500; }

  .tl__hero-gauge { flex-shrink: 0; }

  /* ---- Column header ---- */
  .tl__col-header {
    display: grid;
    grid-template-columns: 2.5rem 1fr 1fr auto;
    gap: 0.75rem;
    padding: 0 1rem 0.5rem;
    font-size: 0.6875rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--color-text-muted);
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .tl__col-album { display: none; }

  @media (min-width: 768px) {
    .tl__col-header { grid-template-columns: 2.5rem 1fr 1fr 1fr auto; }
    .tl__col-album { display: block; }
  }

  /* ---- Loading / empty ---- */
  .tl__loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.75rem;
    padding: 3rem 1rem;
    color: var(--color-text-secondary);
  }

  .tl__spinner { width: 1.5rem; height: 1.5rem; }

  .tl__empty {
    text-align: center;
    color: var(--color-text-tertiary);
    padding: 3rem 1rem;
  }

  /* ---- Track rows ---- */
  .tl__rows {
    display: flex;
    flex-direction: column;
  }

  .tl__row {
    display: grid;
    grid-template-columns: 2.5rem 1fr 1fr auto;
    gap: 0.75rem;
    align-items: center;
    padding: 0.5rem 1rem;
    font-size: 0.8125rem;
    border-radius: 0.375rem;
    transition: background 0.12s;
  }

  @media (min-width: 768px) {
    .tl__row { grid-template-columns: 2.5rem 1fr 1fr 1fr auto; }
  }

  .tl__row:hover {
    background: var(--color-bg-hover);
  }

  .tl__row--flagged { background: rgba(239,68,68,0.03); }
  .tl__row--blocked { background: rgba(245,158,11,0.03); }

  .tl__num {
    font-variant-numeric: tabular-nums;
    text-align: right;
    color: var(--color-text-muted);
    font-size: 0.75rem;
  }

  .tl__title-cell {
    display: flex;
    align-items: center;
    gap: 0.625rem;
    min-width: 0;
  }

  .tl__avatar {
    width: 2rem;
    height: 2rem;
    border-radius: 0.25rem;
    object-fit: cover;
    flex-shrink: 0;
    max-width: none;
    max-height: none;
    background: var(--color-bg-inset);
  }

  .tl__avatar--placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .tl__status-dot {
    width: 0.375rem;
    height: 0.375rem;
    border-radius: 50%;
  }

  .tl__status-dot--clean { background: var(--color-success); }
  .tl__status-dot--flagged { background: var(--color-error); }
  .tl__status-dot--blocked { background: var(--color-warning); }

  .tl__title-text {
    min-width: 0;
  }

  .tl__track-name {
    font-weight: 500;
    color: var(--color-text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    display: block;
  }

  .tl__artist {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tl__artist--clean { color: var(--color-text-secondary); }
  .tl__artist--flagged { color: #fb7185; }
  .tl__artist--blocked { color: #fbbf24; }

  .tl__album {
    color: var(--color-text-muted);
    font-size: 0.75rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    display: none;
  }

  @media (min-width: 768px) {
    .tl__album { display: block; }
  }

  .tl__badge {
    font-size: 0.5625rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    padding: 0.15rem 0.4rem;
    border-radius: 9999px;
  }

  .tl__badge--flagged {
    color: var(--color-error);
    background: var(--color-error-muted);
  }

  .tl__badge--blocked {
    color: var(--color-warning);
    background: var(--color-warning-muted);
  }

  /* ---- Footer ---- */
  .tl__footer {
    display: flex;
    justify-content: center;
    padding: 1.5rem 0 0.5rem;
  }

  .tl__sanitize-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.625rem 1.5rem;
    border-radius: 9999px;
    border: none;
    background: var(--color-brand-primary);
    color: #fff;
    font-size: 0.875rem;
    font-weight: 600;
    font-family: inherit;
    cursor: pointer;
    transition: background 0.15s, transform 0.15s, box-shadow 0.15s;
    box-shadow: 0 0 0 0 rgba(244,63,94,0);
  }

  .tl__sanitize-btn svg { width: 16px; height: 16px; max-width: none; max-height: none; }

  .tl__sanitize-btn:hover {
    background: var(--color-brand-primary-hover);
    transform: translateY(-1px);
    box-shadow: 0 4px 16px rgba(244,63,94,0.3);
  }

  .tl__sanitize-btn:active { transform: translateY(0); }

  /* ---- Mobile ---- */
  @media (max-width: 640px) {
    .tl__hero-main { flex-direction: column; align-items: flex-start; }
    .tl__title { font-size: 1.25rem; }
    .tl__row { grid-template-columns: 2rem 1fr auto; gap: 0.5rem; }
    .tl__artist { display: none; }
    .tl__avatar { width: 1.75rem; height: 1.75rem; }
  }
</style>
