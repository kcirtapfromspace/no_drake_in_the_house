<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { PlaylistSummary } from '../stores/playlist-browser';

  export let playlist: PlaylistSummary;

  const dispatch = createEventDispatcher<{ select: PlaylistSummary }>();

  $: providerColor = playlist.provider === 'spotify' ? '#1DB954' : '#fc3c44';
  $: gradeColor = getGradeColor(playlist.grade);

  function getGradeColor(g: string): string {
    switch (g) {
      case 'A+': return '#22c55e';
      case 'A': return '#22c55e';
      case 'B': return '#3b82f6';
      case 'C': return '#eab308';
      case 'D': return '#f97316';
      default: return '#ef4444';
    }
  }

  $: flaggedPreview = playlist.flagged_artists.slice(0, 2);
  $: flaggedMore = playlist.flagged_artists.length - 2;
</script>

<button
  type="button"
  class="playlist-card surface-panel-thin"
  on:click={() => dispatch('select', playlist)}
>
  <span class="playlist-card__provider" style="background: {providerColor};" title={playlist.provider}></span>

  <h3 class="playlist-card__name">{playlist.playlist_name}</h3>

  <span class="playlist-card__grade" style="background: {gradeColor}20; color: {gradeColor};">
    {playlist.grade}
  </span>

  <div class="playlist-card__stats">
    <span>{playlist.total_tracks} tracks</span>
    <span class="playlist-card__sep">&middot;</span>
    <span class="playlist-card__flagged-count">{playlist.flagged_tracks} flagged</span>
  </div>

  {#if flaggedPreview.length > 0}
    <p class="playlist-card__flagged">
      {flaggedPreview.join(', ')}{#if flaggedMore > 0} +{flaggedMore} more{/if}
    </p>
  {/if}
</button>

<style>
  .playlist-card {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 1rem 1.125rem;
    border: 1px solid var(--color-border-subtle, #333);
    border-radius: 0.75rem;
    background: var(--color-surface-secondary, #1a1a2e);
    cursor: pointer;
    text-align: left;
    font-family: inherit;
    color: inherit;
    transition: transform 0.15s, border-color 0.15s, box-shadow 0.15s;
    width: 100%;
  }

  .playlist-card:hover {
    transform: translateY(-2px);
    border-color: var(--color-border-hover, #555);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
  }

  .playlist-card__provider {
    position: absolute;
    top: 0.75rem;
    right: 0.75rem;
    width: 0.5rem;
    height: 0.5rem;
    border-radius: 50%;
  }

  .playlist-card__name {
    font-size: 0.9375rem;
    font-weight: 600;
    color: var(--color-text-primary, #fff);
    margin: 0;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
    padding-right: 1.25rem;
    line-height: 1.35;
  }

  .playlist-card__grade {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0.125rem 0.5rem;
    border-radius: 9999px;
    font-size: 0.75rem;
    font-weight: 700;
    width: fit-content;
  }

  .playlist-card__stats {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    font-size: 0.8125rem;
    color: var(--color-text-secondary, #999);
  }

  .playlist-card__sep {
    opacity: 0.5;
  }

  .playlist-card__flagged-count {
    color: var(--color-brand-danger, #ef4444);
  }

  .playlist-card__flagged {
    font-size: 0.75rem;
    color: var(--color-text-tertiary, #666);
    margin: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
