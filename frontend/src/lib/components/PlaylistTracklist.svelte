<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { PlaylistSummary, PlaylistTrack } from '../stores/playlist-browser';
  import { trackStats } from '../stores/playlist-browser';
  import { hashString } from '../utils/playlist-helpers';
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
  $: images = playlist.cover_images || [];
  $: heroImage = images[0] || '';
  $: hue1 = hashString(playlist.playlist_name) % 360;
  $: hue2 = (hue1 + 40 + (hashString(playlist.provider) % 60)) % 360;
</script>

<div class="tl">
  <!-- Hero header -->
  <header class="tl__hero">
    {#if heroImage}
      <img class="tl__hero-bg" src={heroImage} alt="" />
    {:else}
      <div class="tl__hero-bg tl__hero-bg--gradient" style="background: linear-gradient(135deg, hsl({hue1},55%,22%) 0%, hsl({hue2},40%,12%) 100%);"></div>
    {/if}
    <div class="tl__hero-grain"></div>
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
              <span class="tl__chip-val">{$trackStats.clean}</span>
              <span class="tl__chip-label">Clean</span>
            </div>
            {#if $trackStats.flagged > 0}
              <div class="tl__chip tl__chip--flagged">
                <span class="tl__chip-val">{$trackStats.flagged}</span>
                <span class="tl__chip-label">Flagged</span>
              </div>
            {/if}
            {#if $trackStats.blocked > 0}
              <div class="tl__chip tl__chip--blocked">
                <span class="tl__chip-val">{$trackStats.blocked}</span>
                <span class="tl__chip-label">Blocked</span>
              </div>
            {/if}
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
      <div class="tl__loading-bars">
        <span></span><span></span><span></span><span></span>
      </div>
      <p>Loading tracks...</p>
    </div>
  {:else if tracks.length === 0}
    {#if playlist.tracks_out_of_sync}
      <div class="tl__empty tl__empty--warning">
        <p>This playlist needs a re-sync before its tracks can be shown.</p>
        <p>
          The provider reports
          {playlist.provider_track_count ?? playlist.total_tracks}
          track{(playlist.provider_track_count ?? playlist.total_tracks) === 1 ? '' : 's'}, but the imported detail rows are still empty.
        </p>
      </div>
    {:else}
      <p class="tl__empty">No tracks found in this playlist.</p>
    {/if}
  {:else}
    <!-- Column header -->
    <div class="tl__col-header">
      <span class="tl__col-num">#</span>
      <span class="tl__col-title">Title</span>
      <span class="tl__col-artist">Artist</span>
      <span class="tl__col-album">Album</span>
      <span class="tl__col-status">Status</span>
    </div>

    <div class="tl__rows">
      {#each tracks as track, i}
        <div
          class="tl__row"
          class:tl__row--flagged={track.status === 'flagged'}
          class:tl__row--blocked={track.status === 'blocked'}
          style="--ri: {i}; animation-delay: {Math.min(i * 20, 600)}ms;"
        >
          <span class="tl__num">{track.position + 1}</span>

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
          {:else}
            <span class="tl__badge tl__badge--clean">
              <svg width="12" height="12" viewBox="0 0 12 12" fill="none"><path d="M2.5 6l2.5 2.5 4.5-4.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>
            </span>
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
        on:click={() => dispatch('sanitize', { provider: playlist.provider, playlistName: playlist.provider_playlist_id || playlist.playlist_name })}
      >
        <svg width="18" height="18" viewBox="0 0 18 18" fill="none">
          <path d="M9 2v5M9 11v5M2 9h5M11 9h5" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        </svg>
        Sanitize This Playlist
      </button>
    </div>
  {/if}
</div>

<style>
  .tl {
    max-width: 64rem;
    margin: 0 auto;
    animation: tlIn 0.35s ease-out both;
  }

  @keyframes tlIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  /* ---- Hero ---- */
  .tl__hero {
    position: relative;
    border-radius: 0.875rem;
    overflow: hidden;
    margin-bottom: 1.5rem;
    min-height: 230px;
  }

  .tl__hero-bg {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    max-width: none;
    max-height: none;
    filter: blur(24px) saturate(1.3) brightness(0.9);
    transform: scale(1.2);
  }

  .tl__hero-bg--gradient { filter: none; transform: none; }

  .tl__hero-grain {
    position: absolute;
    inset: 0;
    opacity: 0.04;
    mix-blend-mode: overlay;
    pointer-events: none;
    background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 256 256' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.85' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)'/%3E%3C/svg%3E");
    background-size: 128px 128px;
  }

  .tl__hero-scrim {
    position: absolute;
    inset: 0;
    background: linear-gradient(to bottom,
      rgba(9,9,11,0.25) 0%,
      rgba(9,9,11,0.65) 50%,
      rgba(9,9,11,0.95) 100%
    );
  }

  .tl__hero-content {
    position: relative;
    padding: 1.5rem 1.75rem 1.75rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
    z-index: 1;
  }

  .tl__back {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    background: none;
    border: none;
    color: var(--color-text-secondary);
    font-size: 0.8125rem;
    cursor: pointer;
    padding: 0.25rem 0;
    font-family: inherit;
    transition: color 0.15s, gap 0.2s;
    align-self: flex-start;
  }

  .tl__back svg { width: 16px; height: 16px; max-width: none; max-height: none; transition: transform 0.2s; }
  .tl__back:hover { color: var(--color-text-primary); }
  .tl__back:hover svg { transform: translateX(-2px); }

  .tl__hero-main {
    display: flex;
    align-items: flex-end;
    gap: 1.75rem;
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
    font-size: 1.75rem;
    font-weight: 700;
    color: var(--color-text-primary);
    margin: 0 0 0.75rem;
    line-height: 1.15;
    letter-spacing: -0.025em;
    animation: titleIn 0.5s cubic-bezier(.22,1,.36,1) 0.15s both;
  }

  @keyframes titleIn {
    from { opacity: 0; transform: translateY(8px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .tl__stat-chips {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .tl__chip {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    padding: 0.25rem 0.625rem;
    border-radius: 9999px;
    font-size: 0.75rem;
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    border: 1px solid rgba(255,255,255,0.06);
    transition: transform 0.15s;
  }

  .tl__chip:hover { transform: scale(1.04); }

  .tl__chip--clean { background: rgba(34,197,94,0.12); }
  .tl__chip--flagged { background: rgba(239,68,68,0.12); }
  .tl__chip--blocked { background: rgba(245,158,11,0.12); }

  .tl__chip-val { font-weight: 700; font-variant-numeric: tabular-nums; }
  .tl__chip--clean .tl__chip-val { color: var(--color-success); }
  .tl__chip--flagged .tl__chip-val { color: var(--color-error); }
  .tl__chip--blocked .tl__chip-val { color: var(--color-warning); }

  .tl__chip-label { color: var(--color-text-tertiary); font-weight: 500; }

  .tl__hero-gauge {
    flex-shrink: 0;
    animation: gaugeIn 0.5s cubic-bezier(.22,1,.36,1) 0.25s both;
  }

  @keyframes gaugeIn {
    from { opacity: 0; transform: scale(0.85); }
    to { opacity: 1; transform: scale(1); }
  }

  /* ---- Column header ---- */
  .tl__col-header {
    display: grid;
    grid-template-columns: 2.5rem 1fr 1fr auto;
    gap: 0.75rem;
    padding: 0 1rem 0.625rem;
    font-size: 0.6875rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--color-text-muted);
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .tl__col-album, .tl__col-status { display: none; }

  @media (min-width: 768px) {
    .tl__col-header { grid-template-columns: 2.5rem 1fr 1fr 1fr 4.5rem; }
    .tl__col-album, .tl__col-status { display: block; }
    .tl__col-status { text-align: center; }
  }

  /* ---- Loading ---- */
  .tl__loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    padding: 3.5rem 1rem;
    color: var(--color-text-secondary);
  }

  .tl__loading-bars {
    display: flex;
    align-items: flex-end;
    gap: 3px;
    height: 1.5rem;
  }

  .tl__loading-bars span {
    width: 3px;
    background: var(--color-brand-primary);
    border-radius: 1px;
    animation: barPulse 1.2s ease-in-out infinite;
  }

  .tl__loading-bars span:nth-child(1) { height: 40%; animation-delay: 0s; }
  .tl__loading-bars span:nth-child(2) { height: 70%; animation-delay: 0.15s; }
  .tl__loading-bars span:nth-child(3) { height: 50%; animation-delay: 0.3s; }
  .tl__loading-bars span:nth-child(4) { height: 80%; animation-delay: 0.45s; }

  @keyframes barPulse {
    0%, 100% { transform: scaleY(0.4); opacity: 0.5; }
    50% { transform: scaleY(1); opacity: 1; }
  }

  .tl__empty {
    text-align: center;
    color: var(--color-text-tertiary);
    padding: 3rem 1rem;
  }

  .tl__empty--warning {
    max-width: 34rem;
    margin: 0 auto;
    border: 1px solid color-mix(in srgb, var(--color-warning, #f59e0b) 30%, transparent);
    border-radius: 0.875rem;
    background: color-mix(in srgb, var(--color-warning, #f59e0b) 10%, transparent);
  }

  .tl__empty--warning p {
    margin: 0.35rem 0;
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
    border-radius: 0.5rem;
    transition: background 0.15s, transform 0.1s;
    animation: rowIn 0.3s ease-out both;
  }

  @keyframes rowIn {
    from { opacity: 0; transform: translateX(-6px); }
    to { opacity: 1; transform: translateX(0); }
  }

  @media (min-width: 768px) {
    .tl__row { grid-template-columns: 2.5rem 1fr 1fr 1fr 4.5rem; }
  }

  .tl__row:hover {
    background: var(--color-bg-hover);
  }

  .tl__row--flagged {
    background: rgba(239,68,68,0.03);
    border-left: 2px solid rgba(239,68,68,0.3);
  }

  .tl__row--flagged:hover {
    background: rgba(239,68,68,0.06);
  }

  .tl__row--blocked {
    background: rgba(245,158,11,0.03);
    border-left: 2px solid rgba(245,158,11,0.3);
  }

  .tl__row--blocked:hover {
    background: rgba(245,158,11,0.06);
  }

  .tl__num {
    font-variant-numeric: tabular-nums;
    text-align: right;
    color: var(--color-text-muted);
    font-size: 0.75rem;
    transition: color 0.15s;
  }

  .tl__row:hover .tl__num {
    color: var(--color-text-secondary);
  }

  .tl__title-cell {
    display: flex;
    align-items: center;
    gap: 0.625rem;
    min-width: 0;
  }

  .tl__avatar {
    width: 2.125rem;
    height: 2.125rem;
    border-radius: 0.3rem;
    object-fit: cover;
    flex-shrink: 0;
    max-width: none;
    max-height: none;
    background: var(--color-bg-inset);
    transition: transform 0.15s;
  }

  .tl__row:hover .tl__avatar {
    transform: scale(1.06);
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
  .tl__status-dot--flagged { background: var(--color-error); box-shadow: 0 0 6px rgba(239,68,68,0.4); }
  .tl__status-dot--blocked { background: var(--color-warning); box-shadow: 0 0 6px rgba(245,158,11,0.4); }

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
    padding: 0.15rem 0.5rem;
    border-radius: 9999px;
    text-align: center;
    display: none;
  }

  @media (min-width: 768px) {
    .tl__badge { display: inline-flex; align-items: center; justify-content: center; }
  }

  .tl__badge--clean {
    color: var(--color-success);
    opacity: 0.5;
  }

  .tl__badge--flagged {
    color: var(--color-error);
    background: var(--color-error-muted);
    box-shadow: 0 0 8px rgba(239,68,68,0.1);
  }

  .tl__badge--blocked {
    color: var(--color-warning);
    background: var(--color-warning-muted);
    box-shadow: 0 0 8px rgba(245,158,11,0.1);
  }

  /* ---- Footer ---- */
  .tl__footer {
    display: flex;
    justify-content: center;
    padding: 2rem 0 0.5rem;
  }

  .tl__sanitize-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1.75rem;
    border-radius: 9999px;
    border: none;
    background: linear-gradient(135deg, #fb7185, #e11d48);
    color: #fff;
    font-size: 0.9375rem;
    font-weight: 600;
    font-family: inherit;
    cursor: pointer;
    transition: transform 0.2s, box-shadow 0.2s;
    box-shadow: 0 4px 20px rgba(244,63,94,0.25);
  }

  .tl__sanitize-btn svg { width: 18px; height: 18px; max-width: none; max-height: none; }

  .tl__sanitize-btn:hover {
    transform: translateY(-2px) scale(1.02);
    box-shadow: 0 8px 30px rgba(244,63,94,0.35);
  }

  .tl__sanitize-btn:active {
    transform: translateY(0) scale(0.98);
    transition-duration: 0.1s;
  }

  /* ---- Mobile ---- */
  @media (max-width: 640px) {
    .tl__hero-main { flex-direction: column; align-items: flex-start; gap: 1rem; }
    .tl__title { font-size: 1.375rem; }
    .tl__row { grid-template-columns: 2rem 1fr auto; gap: 0.5rem; }
    .tl__artist { display: none; }
    .tl__avatar { width: 1.75rem; height: 1.75rem; }
    .tl__hero-content { padding: 1.25rem; }
  }
</style>
