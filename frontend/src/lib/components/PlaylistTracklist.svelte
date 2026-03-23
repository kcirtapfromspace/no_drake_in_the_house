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

  $: providerColor = playlist.provider === 'spotify' ? '#1DB954' : '#fc3c44';
  $: providerLabel = playlist.provider === 'spotify' ? 'Spotify' : 'Apple Music';
  $: stats = $trackStats;
</script>

<div class="tracklist">
  <button type="button" class="tracklist__back" on:click={() => dispatch('back')}>
    &larr; All Playlists
  </button>

  <header class="tracklist__header">
    <div class="tracklist__header-info">
      <div class="tracklist__title-row">
        <h2 class="tracklist__title">{playlist.playlist_name}</h2>
        <span class="tracklist__provider-badge" style="background: {providerColor};">{providerLabel}</span>
      </div>

      <div class="tracklist__stat-bar">
        <div class="tracklist__stat">
          <span class="tracklist__stat-val">{stats.total}</span>
          <span class="tracklist__stat-label">Total</span>
        </div>
        <div class="tracklist__stat tracklist__stat--clean">
          <span class="tracklist__stat-val">{stats.clean}</span>
          <span class="tracklist__stat-label">Clean</span>
        </div>
        <div class="tracklist__stat tracklist__stat--flagged">
          <span class="tracklist__stat-val">{stats.flagged}</span>
          <span class="tracklist__stat-label">Flagged</span>
        </div>
        <div class="tracklist__stat tracklist__stat--blocked">
          <span class="tracklist__stat-val">{stats.blocked}</span>
          <span class="tracklist__stat-label">Blocked</span>
        </div>
      </div>
    </div>

    <div class="tracklist__gauge">
      <PlaylistGradeGauge score={Math.round(playlist.clean_ratio * 100)} grade={playlist.grade} size={120} />
    </div>
  </header>

  {#if isLoading}
    <div class="tracklist__loading">
      <div class="brand-button__spinner tracklist__spinner"></div>
      <p>Loading tracks...</p>
    </div>
  {:else if tracks.length === 0}
    <p class="tracklist__empty">No tracks found in this playlist.</p>
  {:else}
    <div class="tracklist__list surface-panel-thin">
      {#each tracks as track}
        <div class="tracklist__row">
          <span class="tracklist__pos">{track.position}</span>
          <span class="tracklist__dot tracklist__dot--{track.status}" title={track.status}></span>
          <span class="tracklist__track">{track.track_name}</span>
          <span class="tracklist__artist tracklist__artist--{track.status}">{track.artist_name}</span>
          {#if track.album_name}
            <span class="tracklist__album">{track.album_name}</span>
          {/if}
        </div>
      {/each}
    </div>
  {/if}

  {#if playlist.flagged_tracks > 0}
    <div class="tracklist__actions">
      <button
        type="button"
        class="brand-button brand-button--primary"
        on:click={() => dispatch('sanitize', { provider: playlist.provider, playlistName: playlist.playlist_name })}
      >
        Sanitize This Playlist
      </button>
    </div>
  {/if}
</div>

<style>
  .tracklist {
    max-width: 56rem;
    margin: 0 auto;
  }

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

  .tracklist__header {
    display: flex;
    gap: 1.5rem;
    align-items: center;
    margin-bottom: 1.5rem;
    padding: 1.25rem;
    border: 1px solid var(--color-border-subtle, #333);
    border-radius: 0.75rem;
    background: var(--color-surface-secondary, #1a1a2e);
  }

  .tracklist__header-info {
    flex: 1;
  }

  .tracklist__title-row {
    display: flex;
    align-items: center;
    gap: 0.625rem;
    margin-bottom: 0.75rem;
    flex-wrap: wrap;
  }

  .tracklist__title {
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--color-text-primary, #fff);
    margin: 0;
  }

  .tracklist__provider-badge {
    display: inline-flex;
    align-items: center;
    padding: 0.125rem 0.5rem;
    border-radius: 9999px;
    font-size: 0.6875rem;
    font-weight: 600;
    color: #fff;
  }

  .tracklist__stat-bar {
    display: flex;
    gap: 1.25rem;
  }

  .tracklist__stat {
    display: flex;
    flex-direction: column;
  }

  .tracklist__stat-val {
    font-size: 1.25rem;
    font-weight: 700;
    color: var(--color-text-primary, #fff);
  }

  .tracklist__stat--clean .tracklist__stat-val {
    color: var(--color-brand-success, #22c55e);
  }

  .tracklist__stat--flagged .tracklist__stat-val {
    color: var(--color-brand-danger, #ef4444);
  }

  .tracklist__stat--blocked .tracklist__stat-val {
    color: var(--color-brand-caution, #f97316);
  }

  .tracklist__stat-label {
    font-size: 0.6875rem;
    color: var(--color-text-tertiary, #666);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .tracklist__gauge {
    flex-shrink: 0;
  }

  .tracklist__loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.75rem;
    padding: 3rem 1rem;
    color: var(--color-text-secondary, #999);
  }

  .tracklist__spinner {
    width: 1.5rem;
    height: 1.5rem;
  }

  .tracklist__empty {
    text-align: center;
    color: var(--color-text-tertiary, #666);
    padding: 3rem 1rem;
    font-size: 0.9375rem;
  }

  .tracklist__list {
    border: 1px solid var(--color-border-subtle, #333);
    border-radius: 0.75rem;
    overflow: hidden;
  }

  .tracklist__row {
    display: grid;
    grid-template-columns: 2.5rem 1rem 1fr 1fr auto;
    gap: 0.75rem;
    align-items: center;
    padding: 0.5rem 0.875rem;
    font-size: 0.875rem;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
  }

  .tracklist__row:first-child {
    border-top: none;
  }

  .tracklist__pos {
    color: var(--color-text-tertiary, #666);
    font-size: 0.75rem;
    font-variant-numeric: tabular-nums;
    text-align: right;
  }

  .tracklist__dot {
    width: 0.5rem;
    height: 0.5rem;
    border-radius: 50%;
  }

  .tracklist__dot--clean {
    background: var(--color-brand-success, #22c55e);
  }

  .tracklist__dot--flagged {
    background: var(--color-brand-danger, #ef4444);
  }

  .tracklist__dot--blocked {
    background: var(--color-brand-caution, #f97316);
  }

  .tracklist__track {
    font-weight: 500;
    color: var(--color-text-primary, #fff);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tracklist__artist {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tracklist__artist--clean {
    color: var(--color-text-secondary, #999);
  }

  .tracklist__artist--flagged {
    color: #fb7185;
  }

  .tracklist__artist--blocked {
    color: #fbbf24;
  }

  .tracklist__album {
    color: var(--color-text-tertiary, #666);
    font-size: 0.8125rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tracklist__actions {
    display: flex;
    justify-content: flex-end;
    margin-top: 1.5rem;
  }

  @media (max-width: 640px) {
    .tracklist__header {
      flex-direction: column;
      text-align: center;
    }

    .tracklist__title-row {
      justify-content: center;
    }

    .tracklist__stat-bar {
      justify-content: center;
    }

    .tracklist__row {
      grid-template-columns: 2rem 0.75rem 1fr auto;
    }

    .tracklist__album {
      display: none;
    }

    .tracklist__artist {
      display: none;
    }
  }
</style>
