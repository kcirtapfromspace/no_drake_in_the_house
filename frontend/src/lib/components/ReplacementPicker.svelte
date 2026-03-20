<script lang="ts">
  import type { ReplacementSuggestion } from '../stores/sanitizer';
  import { sanitizerActions } from '../stores/sanitizer';

  export let suggestion: ReplacementSuggestion;
  export let selectedId: string = '';

  let previewAudio: HTMLAudioElement | null = null;
  let playingPreviewId: string | null = null;

  function formatDuration(ms: number): string {
    const minutes = Math.floor(ms / 60000);
    const seconds = Math.floor((ms % 60000) / 1000);
    return `${minutes}:${seconds.toString().padStart(2, '0')}`;
  }

  function selectCandidate(trackId: string) {
    sanitizerActions.selectReplacement(suggestion.original_track_id, trackId);
  }

  function skipTrack() {
    sanitizerActions.skipTrack(suggestion.original_track_id);
  }

  function togglePreview(previewUrl: string | null, trackId: string) {
    if (!previewUrl) return;

    if (playingPreviewId === trackId) {
      previewAudio?.pause();
      playingPreviewId = null;
      return;
    }

    if (previewAudio) {
      previewAudio.pause();
    }
    previewAudio = new Audio(previewUrl);
    previewAudio.volume = 0.5;
    previewAudio.play();
    playingPreviewId = trackId;
    previewAudio.onended = () => { playingPreviewId = null; };
  }
</script>

<div class="picker">
  <div class="picker__original">
    <div class="picker__label">Blocked Track</div>
    <div class="picker__track-info">
      <span class="picker__track-name">{suggestion.original_track_name}</span>
      <span class="picker__artist-name">{suggestion.original_artist_name}</span>
    </div>
  </div>

  <div class="picker__arrow">
    <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
      <path d="M4 10h12m0 0l-4-4m4 4l-4 4" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>
  </div>

  <div class="picker__candidates">
    <div class="picker__label">Replacements</div>
    {#if suggestion.candidates.length === 0}
      <div class="picker__empty">No replacements found</div>
    {:else}
      {#each suggestion.candidates as candidate}
        <div
          class="picker__candidate"
          class:picker__candidate--selected={selectedId === candidate.track_id}
          role="button"
          tabindex="0"
          on:click={() => selectCandidate(candidate.track_id)}
          on:keydown={(e) => e.key === 'Enter' && selectCandidate(candidate.track_id)}
        >
          <div class="picker__candidate-info">
            <span class="picker__candidate-name">{candidate.track_name}</span>
            <span class="picker__candidate-artist">{candidate.artist_name} &middot; {candidate.album_name}</span>
            <span class="picker__candidate-meta">
              {formatDuration(candidate.duration_ms)} &middot; Pop: {candidate.popularity}
            </span>
          </div>
          {#if candidate.preview_url}
            <button
              type="button"
              class="picker__preview-btn"
              on:click|stopPropagation={() => togglePreview(candidate.preview_url, candidate.track_id)}
              title="Preview 30s"
            >
              {#if playingPreviewId === candidate.track_id}
                <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                  <rect x="3" y="2" width="4" height="12" rx="1"/>
                  <rect x="9" y="2" width="4" height="12" rx="1"/>
                </svg>
              {:else}
                <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                  <path d="M4 2l10 6-10 6V2z"/>
                </svg>
              {/if}
            </button>
          {/if}
          {#if selectedId === candidate.track_id}
            <span class="picker__check">
              <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                <path d="M3 8l3 3 7-7" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
            </span>
          {/if}
        </div>
      {/each}
    {/if}
    <button
      type="button"
      class="picker__skip-btn"
      class:picker__skip-btn--active={selectedId === 'skip'}
      on:click={skipTrack}
    >
      Skip (remove track)
    </button>
  </div>
</div>

<style>
  .picker {
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    gap: 1rem;
    align-items: start;
    padding: 1rem;
    border: 1px solid var(--color-border-subtle, #333);
    border-radius: 0.5rem;
    background: var(--color-surface-secondary, #1a1a2e);
  }

  .picker__label {
    font-size: 0.6875rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-text-tertiary, #666);
    margin-bottom: 0.5rem;
    font-weight: 600;
  }

  .picker__original {
    min-width: 0;
  }

  .picker__track-info {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .picker__track-name {
    font-weight: 600;
    color: var(--color-text-primary, #fff);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .picker__artist-name {
    font-size: 0.875rem;
    color: var(--color-brand-danger, #ef4444);
  }

  .picker__arrow {
    display: flex;
    align-items: center;
    padding-top: 1.5rem;
    color: var(--color-text-tertiary, #666);
  }

  .picker__candidates {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
    min-width: 0;
  }

  .picker__empty {
    font-size: 0.875rem;
    color: var(--color-text-tertiary, #666);
    font-style: italic;
  }

  .picker__candidate {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem;
    border: 1px solid var(--color-border-subtle, #333);
    border-radius: 0.375rem;
    background: transparent;
    cursor: pointer;
    text-align: left;
    color: inherit;
    transition: border-color 0.15s, background 0.15s;
    font-family: inherit;
    font-size: inherit;
    width: 100%;
  }

  .picker__candidate:hover {
    border-color: var(--color-border-hover, #555);
    background: var(--color-surface-hover, rgba(255, 255, 255, 0.03));
  }

  .picker__candidate--selected {
    border-color: var(--color-brand-primary, #8b5cf6);
    background: rgba(139, 92, 246, 0.08);
  }

  .picker__candidate-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
    min-width: 0;
  }

  .picker__candidate-name {
    font-weight: 500;
    font-size: 0.875rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .picker__candidate-artist {
    font-size: 0.75rem;
    color: var(--color-text-secondary, #999);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .picker__candidate-meta {
    font-size: 0.6875rem;
    color: var(--color-text-tertiary, #666);
  }

  .picker__preview-btn {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 2rem;
    height: 2rem;
    border: none;
    border-radius: 50%;
    background: var(--color-surface-tertiary, #2a2a3e);
    color: var(--color-text-secondary, #999);
    cursor: pointer;
    transition: background 0.15s, color 0.15s;
  }

  .picker__preview-btn:hover {
    background: var(--color-brand-primary, #8b5cf6);
    color: white;
  }

  .picker__check {
    flex-shrink: 0;
    color: var(--color-brand-primary, #8b5cf6);
  }

  .picker__skip-btn {
    padding: 0.375rem 0.75rem;
    border: 1px dashed var(--color-border-subtle, #333);
    border-radius: 0.375rem;
    background: transparent;
    color: var(--color-text-tertiary, #666);
    cursor: pointer;
    font-size: 0.8125rem;
    transition: border-color 0.15s, color 0.15s;
    font-family: inherit;
  }

  .picker__skip-btn:hover {
    border-color: var(--color-brand-danger, #ef4444);
    color: var(--color-brand-danger, #ef4444);
  }

  .picker__skip-btn--active {
    border-color: var(--color-brand-danger, #ef4444);
    color: var(--color-brand-danger, #ef4444);
    background: rgba(239, 68, 68, 0.08);
  }

  @media (max-width: 640px) {
    .picker {
      grid-template-columns: 1fr;
    }

    .picker__arrow {
      justify-content: center;
      padding-top: 0;
      transform: rotate(90deg);
    }
  }
</style>
