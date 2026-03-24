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

<div class="picker" class:picker--has-selection={selectedId && selectedId !== 'skip'} class:picker--skipped={selectedId === 'skip'}>
  <div class="picker__original">
    <div class="picker__label">
      <span class="picker__label-dot picker__label-dot--red"></span>
      Blocked Track
    </div>
    <div class="picker__track-info">
      <span class="picker__track-name">{suggestion.original_track_name}</span>
      <span class="picker__artist-name">{suggestion.original_artist_name}</span>
    </div>
  </div>

  <div class="picker__arrow">
    <div class="picker__arrow-line"></div>
    <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
      <path d="M4 10h12m0 0l-4-4m4 4l-4 4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>
  </div>

  <div class="picker__candidates">
    <div class="picker__label">
      <span class="picker__label-dot picker__label-dot--green"></span>
      Replacements
    </div>
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
          <div class="picker__candidate-radio">
            <div class="picker__candidate-radio-inner"></div>
          </div>
          <div class="picker__candidate-info">
            <span class="picker__candidate-name">{candidate.track_name}</span>
            <span class="picker__candidate-artist">{candidate.artist_name} &middot; {candidate.album_name}</span>
            <span class="picker__candidate-meta">
              {formatDuration(candidate.duration_ms)}
              {#if candidate.popularity > 0}
                <span class="picker__popularity">
                  <span class="picker__popularity-bar" style="width: {candidate.popularity}%;"></span>
                </span>
              {/if}
            </span>
          </div>
          {#if candidate.preview_url}
            <button
              type="button"
              class="picker__preview-btn"
              class:picker__preview-btn--playing={playingPreviewId === candidate.track_id}
              on:click|stopPropagation={() => togglePreview(candidate.preview_url, candidate.track_id)}
              title="Preview 30s"
            >
              {#if playingPreviewId === candidate.track_id}
                <div class="picker__eq">
                  <span></span><span></span><span></span>
                </div>
              {:else}
                <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
                  <path d="M3.5 1.5l9 5.5-9 5.5V1.5z"/>
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
      <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><path d="M2 2l10 10M12 2L2 12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>
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
    padding: 1.125rem;
    border: 1px solid var(--color-border-subtle, #1f1f22);
    border-radius: 0.75rem;
    background: var(--color-bg-elevated, #111113);
    transition: border-color 0.2s, box-shadow 0.2s;
  }

  .picker--has-selection {
    border-color: rgba(34,197,94,0.2);
    box-shadow: 0 0 0 1px rgba(34,197,94,0.05);
  }

  .picker--skipped {
    border-color: rgba(245,158,11,0.2);
    opacity: 0.7;
  }

  .picker__label {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    font-size: 0.6875rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--color-text-tertiary, #666);
    margin-bottom: 0.5rem;
    font-weight: 600;
  }

  .picker__label-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .picker__label-dot--red {
    background: #ef4444;
    box-shadow: 0 0 6px rgba(239,68,68,0.4);
  }

  .picker__label-dot--green {
    background: #22c55e;
    box-shadow: 0 0 6px rgba(34,197,94,0.4);
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
    font-size: 0.8125rem;
    color: #fb7185;
  }

  .picker__arrow {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.25rem;
    padding-top: 1.75rem;
    color: var(--color-text-muted, #52525b);
  }

  .picker__arrow-line {
    width: 1px;
    height: 0.75rem;
    background: var(--color-border-subtle);
    display: none;
  }

  .picker__candidates {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
    min-width: 0;
  }

  .picker__empty {
    font-size: 0.8125rem;
    color: var(--color-text-tertiary, #666);
    font-style: italic;
    padding: 0.5rem 0;
  }

  .picker__candidate {
    display: flex;
    align-items: center;
    gap: 0.625rem;
    padding: 0.5rem 0.625rem;
    border: 1px solid var(--color-border-subtle, #1f1f22);
    border-radius: 0.5rem;
    background: transparent;
    cursor: pointer;
    text-align: left;
    color: inherit;
    transition: border-color 0.15s, background 0.15s, box-shadow 0.15s;
    font-family: inherit;
    font-size: inherit;
    width: 100%;
  }

  .picker__candidate:hover {
    border-color: var(--color-border-hover, #3f3f46);
    background: rgba(255, 255, 255, 0.02);
  }

  .picker__candidate--selected {
    border-color: rgba(244, 63, 94, 0.35);
    background: rgba(244, 63, 94, 0.05);
    box-shadow: 0 0 12px rgba(244, 63, 94, 0.06);
  }

  .picker__candidate--selected:hover {
    border-color: rgba(244, 63, 94, 0.45);
    background: rgba(244, 63, 94, 0.07);
  }

  /* Radio dot */
  .picker__candidate-radio {
    width: 1rem;
    height: 1rem;
    border-radius: 50%;
    border: 2px solid var(--color-border-default, #27272a);
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: border-color 0.15s;
  }

  .picker__candidate--selected .picker__candidate-radio {
    border-color: var(--color-brand-primary, #f43f5e);
  }

  .picker__candidate-radio-inner {
    width: 0.375rem;
    height: 0.375rem;
    border-radius: 50%;
    background: transparent;
    transition: background 0.15s, transform 0.15s;
    transform: scale(0);
  }

  .picker__candidate--selected .picker__candidate-radio-inner {
    background: var(--color-brand-primary, #f43f5e);
    transform: scale(1);
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
    color: var(--color-text-secondary, #a1a1aa);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .picker__candidate-meta {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.6875rem;
    color: var(--color-text-tertiary, #666);
    font-variant-numeric: tabular-nums;
  }

  .picker__popularity {
    width: 2.5rem;
    height: 3px;
    border-radius: 2px;
    background: var(--color-border-subtle);
    overflow: hidden;
  }

  .picker__popularity-bar {
    height: 100%;
    background: var(--color-brand-primary);
    border-radius: 2px;
    opacity: 0.6;
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
    background: var(--color-bg-interactive, #18181b);
    color: var(--color-text-secondary, #a1a1aa);
    cursor: pointer;
    transition: background 0.15s, color 0.15s, box-shadow 0.15s;
  }

  .picker__preview-btn:hover {
    background: var(--color-brand-primary, #f43f5e);
    color: white;
    box-shadow: 0 0 12px rgba(244,63,94,0.25);
  }

  .picker__preview-btn--playing {
    background: var(--color-brand-primary, #f43f5e);
    color: white;
    box-shadow: 0 0 12px rgba(244,63,94,0.3);
  }

  /* Equalizer animation */
  .picker__eq {
    display: flex;
    align-items: flex-end;
    gap: 2px;
    height: 12px;
  }

  .picker__eq span {
    width: 2px;
    background: currentColor;
    border-radius: 1px;
    animation: eqBounce 0.8s ease-in-out infinite;
  }

  .picker__eq span:nth-child(1) { height: 40%; animation-delay: 0s; }
  .picker__eq span:nth-child(2) { height: 70%; animation-delay: 0.15s; }
  .picker__eq span:nth-child(3) { height: 50%; animation-delay: 0.3s; }

  @keyframes eqBounce {
    0%, 100% { transform: scaleY(0.4); }
    50% { transform: scaleY(1); }
  }

  .picker__check {
    flex-shrink: 0;
    color: var(--color-brand-primary, #f43f5e);
  }

  .picker__skip-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.375rem 0.75rem;
    border: 1px dashed var(--color-border-subtle, #1f1f22);
    border-radius: 0.5rem;
    background: transparent;
    color: var(--color-text-tertiary, #666);
    cursor: pointer;
    font-size: 0.8125rem;
    transition: border-color 0.15s, color 0.15s, background 0.15s;
    font-family: inherit;
  }

  .picker__skip-btn svg { width: 14px; height: 14px; flex-shrink: 0; }

  .picker__skip-btn:hover {
    border-color: rgba(245,158,11,0.4);
    color: #f59e0b;
    background: rgba(245,158,11,0.04);
  }

  .picker__skip-btn--active {
    border-color: rgba(245,158,11,0.4);
    color: #f59e0b;
    background: rgba(245,158,11,0.06);
    border-style: solid;
  }

  @media (max-width: 640px) {
    .picker {
      grid-template-columns: 1fr;
    }

    .picker__arrow {
      justify-content: center;
      padding-top: 0;
      flex-direction: row;
    }

    .picker__arrow svg {
      transform: rotate(90deg);
    }

    .picker__arrow-line { display: none; }
  }
</style>
