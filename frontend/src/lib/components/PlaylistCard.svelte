<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { PlaylistSummary } from '../stores/playlist-browser';
  import { getGradeColor, getGradeGlow, hashString } from '../utils/playlist-helpers';

  export let playlist: PlaylistSummary;
  export let index: number = 0;
  export let selectionMode: boolean = false;
  export let selected: boolean = false;

  const dispatch = createEventDispatcher<{
    select: PlaylistSummary;
    toggleSelect: PlaylistSummary;
    quickScrub: PlaylistSummary;
  }>();

  function handleClick() {
    dispatch(selectionMode ? 'toggleSelect' : 'select', playlist);
  }

  const PROVIDER_COLORS: Record<string, string> = {
    spotify: 'var(--color-spotify)',
    apple_music: 'var(--color-apple)',
    tidal: '#00FFFF',
    youtube: '#FF0000',
  };
  const PROVIDER_LABELS: Record<string, string> = {
    spotify: 'Spotify',
    apple_music: 'Apple Music',
    tidal: 'Tidal',
    youtube: 'YouTube Music',
  };
  $: providerColor = PROVIDER_COLORS[playlist.provider] ?? 'var(--color-apple)';
  $: providerLabel = PROVIDER_LABELS[playlist.provider] ?? playlist.provider;
  $: gradeColor = getGradeColor(playlist.grade);
  $: gradeGlow = getGradeGlow(playlist.grade);
  $: images = playlist.cover_images || [];
  $: cleanPct = Math.round(playlist.clean_ratio * 100);
  $: gradientHue = hashString(playlist.playlist_name) % 360;
  $: gradientHue2 = (gradientHue + 40 + (hashString(playlist.provider) % 60)) % 360;
  $: flaggedPreview = playlist.flagged_artists.slice(0, 2);
  $: flaggedMore = playlist.flagged_artists.length - 2;
</script>

<button
  type="button"
  class="pc"
  class:pc--selected={selected}
  class:pc--selection-mode={selectionMode}
  style="--stagger: {index * 60}ms; --grade-color: {gradeColor}; --grade-glow: {gradeGlow};"
  on:click={handleClick}
>
  <!-- Cover art area -->
  <div class="pc__cover">
    {#if images.length >= 4}
      <div class="pc__mosaic">
        {#each images.slice(0, 4) as src}
          <img class="pc__mosaic-img" {src} alt="" loading="lazy" />
        {/each}
      </div>
    {:else if images.length >= 1}
      <img class="pc__hero-img" src={images[0]} alt="" loading="lazy" />
    {:else}
      <div class="pc__gradient" style="background: linear-gradient(135deg, hsl({gradientHue}, 60%, 25%) 0%, hsl({gradientHue2}, 45%, 12%) 100%);"></div>
    {/if}

    <!-- Film grain noise overlay -->
    <div class="pc__grain"></div>

    <!-- Overlay with grade -->
    <div class="pc__cover-overlay">
      <span class="pc__grade">{playlist.grade}</span>
    </div>

    <!-- Provider pill -->
    <span class="pc__provider" style="--pc: {providerColor};">{providerLabel}</span>

    <!-- Selection checkbox (visible in selection mode) -->
    {#if selectionMode}
      <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-noninteractive-element-interactions -->
      <label class="pc__checkbox" on:click|stopPropagation>
        <input
          type="checkbox"
          checked={selected}
          on:change|stopPropagation={() => dispatch('toggleSelect', playlist)}
          class="pc__checkbox-input"
        />
        <span class="pc__checkbox-box" class:pc__checkbox-box--checked={selected}>
          {#if selected}
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none"><path d="M2.5 6l2.5 2.5 4.5-4.5" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>
          {/if}
        </span>
      </label>
    {/if}

    <!-- Quick scrub button (visible on hover, hidden in selection mode) -->
    {#if !selectionMode && playlist.flagged_tracks > 0}
      <button
        type="button"
        class="pc__quick-scrub"
        title="Scrub this playlist"
        on:click|stopPropagation={() => dispatch('quickScrub', playlist)}
      >
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
          <path d="M13.5 2.5l-1.2 1.2M8.5 7.5l-3.3 3.3a1.5 1.5 0 0 1-2.1-2.1l3.3-3.3m2.1 2.1l3.8-3.8m-3.8 3.8L6.4 5.4m5.9-2.9l.7 2.2-2.2.8m-6.6 5.4L2 13l2.1-2.1" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
        Scrub
      </button>
    {/if}

    <!-- Hover vinyl peek -->
    <div class="pc__vinyl-peek">
      <div class="pc__vinyl-disc"></div>
    </div>
  </div>

  <!-- Info area -->
  <div class="pc__body">
    <h3 class="pc__name">{playlist.playlist_name}</h3>

    <div class="pc__meta">
      <span class="pc__track-count">{playlist.total_tracks} tracks</span>
      {#if playlist.flagged_tracks > 0}
        <span class="pc__dot">&middot;</span>
        <span class="pc__flagged-count">{playlist.flagged_tracks} flagged</span>
      {/if}
    </div>

    {#if playlist.tracks_out_of_sync}
      <p class="pc__resync-note">Track details missing. Run Library Sync to refresh this playlist.</p>
    {/if}

    <!-- Clean ratio bar -->
    <div class="pc__bar-wrap">
      <div class="pc__bar-track">
        <div class="pc__bar-fill" style="width: {cleanPct}%;"></div>
      </div>
      <span class="pc__bar-pct">{cleanPct}%</span>
    </div>

    {#if flaggedPreview.length > 0}
      <p class="pc__flagged-names">
        <span class="pc__flagged-dot"></span>
        {flaggedPreview.join(', ')}{#if flaggedMore > 0}<span class="pc__more"> +{flaggedMore}</span>{/if}
      </p>
    {/if}
  </div>
</button>

<style>
  .pc {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--color-border-subtle);
    border-radius: 0.75rem;
    background: var(--color-bg-elevated);
    cursor: pointer;
    text-align: left;
    font-family: inherit;
    color: inherit;
    overflow: hidden;
    transition: transform 0.28s cubic-bezier(.4,0,.2,1),
                border-color 0.28s,
                box-shadow 0.28s;
    width: 100%;
    padding: 0;
    animation: cardIn 0.45s cubic-bezier(.22,1,.36,1) both;
    animation-delay: var(--stagger, 0ms);
    position: relative;
  }

  @keyframes cardIn {
    from { opacity: 0; transform: translateY(16px) scale(0.96); }
    to { opacity: 1; transform: translateY(0) scale(1); }
  }

  .pc:hover {
    transform: translateY(-4px) scale(1.015);
    border-color: color-mix(in srgb, var(--grade-color) 30%, var(--color-border-hover));
    box-shadow: var(--shadow-lg), 0 8px 32px -8px color-mix(in srgb, var(--grade-color) 20%, transparent);
  }

  .pc:active {
    transform: translateY(-1px) scale(1.005);
    transition-duration: 0.1s;
  }

  /* ---- Cover ---- */
  .pc__cover {
    position: relative;
    width: 100%;
    aspect-ratio: 1 / 1;
    overflow: hidden;
    background: var(--color-bg-inset);
  }

  .pc__mosaic {
    display: grid;
    grid-template-columns: 1fr 1fr;
    grid-template-rows: 1fr 1fr;
    width: 100%;
    height: 100%;
  }

  .pc__mosaic-img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    max-width: none;
    max-height: none;
    transition: transform 0.4s cubic-bezier(.4,0,.2,1);
  }

  .pc:hover .pc__mosaic-img {
    transform: scale(1.06);
  }

  .pc__hero-img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    max-width: none;
    max-height: none;
    transition: transform 0.5s cubic-bezier(.4,0,.2,1);
  }

  .pc:hover .pc__hero-img {
    transform: scale(1.08);
  }

  .pc__gradient {
    width: 100%;
    height: 100%;
    transition: transform 0.5s cubic-bezier(.4,0,.2,1);
  }

  .pc:hover .pc__gradient {
    transform: scale(1.08);
  }

  /* Film grain noise */
  .pc__grain {
    position: absolute;
    inset: 0;
    opacity: 0.06;
    mix-blend-mode: overlay;
    pointer-events: none;
    background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 256 256' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)'/%3E%3C/svg%3E");
    background-size: 128px 128px;
  }

  .pc__cover-overlay {
    position: absolute;
    inset: 0;
    background: linear-gradient(to top, rgba(0,0,0,0.75) 0%, rgba(0,0,0,0.1) 40%, transparent 60%);
    display: flex;
    align-items: flex-end;
    justify-content: flex-end;
    padding: 0.625rem;
  }

  .pc__grade {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 2.125rem;
    padding: 0.2rem 0.5rem;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    font-weight: 800;
    letter-spacing: 0.02em;
    color: var(--grade-color);
    background: rgba(0,0,0,0.65);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    border: 1px solid rgba(255,255,255,0.06);
    box-shadow: var(--grade-glow);
    transition: box-shadow 0.3s, transform 0.3s;
  }

  .pc:hover .pc__grade {
    transform: scale(1.08);
    box-shadow: var(--grade-glow), 0 0 20px color-mix(in srgb, var(--grade-color) 30%, transparent);
  }

  .pc__provider {
    position: absolute;
    top: 0.5rem;
    left: 0.5rem;
    padding: 0.15rem 0.5rem;
    border-radius: 9999px;
    font-size: 0.625rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: #fff;
    background: var(--pc);
    box-shadow: 0 2px 8px rgba(0,0,0,0.5);
  }

  /* Vinyl disc peek on hover */
  .pc__vinyl-peek {
    position: absolute;
    top: 50%;
    right: -30%;
    transform: translateY(-50%);
    width: 65%;
    aspect-ratio: 1;
    transition: right 0.45s cubic-bezier(.34,1.56,.64,1);
    pointer-events: none;
    opacity: 0;
  }

  .pc:hover .pc__vinyl-peek {
    right: -22%;
    opacity: 1;
    transition: right 0.45s cubic-bezier(.34,1.56,.64,1), opacity 0.2s ease;
  }

  .pc__vinyl-disc {
    width: 100%;
    height: 100%;
    border-radius: 50%;
    background:
      radial-gradient(circle at center, #1a1a1a 18%, transparent 19%),
      radial-gradient(circle at center, #111 20%, transparent 21%),
      radial-gradient(circle at center, transparent 44%, rgba(40,40,40,0.5) 45%, rgba(40,40,40,0.5) 46%, transparent 47%),
      radial-gradient(circle at center, transparent 58%, rgba(40,40,40,0.3) 59%, rgba(40,40,40,0.3) 60%, transparent 61%),
      conic-gradient(from 0deg, #1a1a1a, #252525, #1a1a1a, #222, #1a1a1a, #252525, #1a1a1a, #222, #1a1a1a);
    box-shadow: -2px 0 12px rgba(0,0,0,0.6);
    animation: vinylSpin 4s linear infinite;
    animation-play-state: paused;
  }

  .pc:hover .pc__vinyl-disc {
    animation-play-state: running;
  }

  @keyframes vinylSpin {
    to { transform: rotate(360deg); }
  }

  /* ---- Body ---- */
  .pc__body {
    padding: 0.75rem 0.875rem 0.875rem;
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
  }

  .pc__name {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
    line-height: 1.3;
    transition: color 0.2s;
  }

  .pc:hover .pc__name {
    color: var(--grade-color);
  }

  .pc__meta {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    font-size: 0.75rem;
    color: var(--color-text-tertiary);
  }

  .pc__dot { opacity: 0.4; }

  .pc__flagged-count {
    color: var(--color-error);
  }

  .pc__resync-note {
    margin: 0;
    font-size: 0.6875rem;
    line-height: 1.4;
    color: color-mix(in srgb, var(--color-warning, #f59e0b) 78%, white);
  }

  .pc__bar-wrap {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .pc__bar-track {
    flex: 1;
    height: 3px;
    border-radius: 2px;
    background: var(--color-border-subtle);
    overflow: hidden;
  }

  .pc__bar-fill {
    height: 100%;
    border-radius: 2px;
    background: linear-gradient(90deg, var(--grade-color), color-mix(in srgb, var(--grade-color) 70%, white));
    transition: width 0.8s cubic-bezier(.4,0,.2,1);
    box-shadow: 0 0 6px color-mix(in srgb, var(--grade-color) 30%, transparent);
  }

  .pc__bar-pct {
    font-size: 0.625rem;
    font-weight: 700;
    color: var(--grade-color);
    font-variant-numeric: tabular-nums;
    min-width: 2rem;
    text-align: right;
    opacity: 0.8;
  }

  .pc__flagged-names {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    font-size: 0.6875rem;
    color: var(--color-text-muted);
    margin: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    line-height: 1.4;
  }

  .pc__flagged-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--color-error);
    flex-shrink: 0;
    box-shadow: 0 0 6px rgba(239,68,68,0.4);
  }

  .pc__more {
    color: var(--color-text-tertiary);
  }

  /* ---- Selection mode ---- */
  .pc--selected {
    border-color: #e11d48;
    box-shadow: 0 0 0 1px #e11d48, 0 0 16px rgba(225,29,72,0.15);
  }

  .pc--selected::after {
    content: '';
    position: absolute;
    inset: 0;
    background: rgba(225,29,72,0.06);
    pointer-events: none;
    border-radius: inherit;
    z-index: 1;
  }

  .pc--selection-mode {
    cursor: pointer;
  }

  .pc--selection-mode:hover {
    border-color: rgba(225,29,72,0.5);
  }

  /* Checkbox */
  .pc__checkbox {
    position: absolute;
    top: 0.5rem;
    right: 0.5rem;
    z-index: 5;
    cursor: pointer;
    display: flex;
  }

  .pc__checkbox-input {
    position: absolute;
    opacity: 0;
    width: 0;
    height: 0;
  }

  .pc__checkbox-box {
    width: 22px;
    height: 22px;
    border-radius: 6px;
    border: 2px solid rgba(255,255,255,0.3);
    background: rgba(0,0,0,0.5);
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
    display: flex;
    align-items: center;
    justify-content: center;
    color: #fff;
    transition: background 0.15s, border-color 0.15s, transform 0.15s;
  }

  .pc__checkbox-box--checked {
    background: #e11d48;
    border-color: #e11d48;
    transform: scale(1.05);
  }

  /* Quick scrub button */
  .pc__quick-scrub {
    position: absolute;
    bottom: 0.5rem;
    right: 0.5rem;
    z-index: 4;
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    padding: 0.3rem 0.6rem;
    border: none;
    border-radius: 0.375rem;
    background: linear-gradient(135deg, #e11d48, #be123c);
    color: #fff;
    font-size: 0.6875rem;
    font-weight: 700;
    font-family: inherit;
    cursor: pointer;
    opacity: 0;
    transform: translateY(4px);
    transition: opacity 0.2s, transform 0.2s;
    box-shadow: 0 2px 8px rgba(0,0,0,0.4);
  }

  .pc:hover .pc__quick-scrub {
    opacity: 1;
    transform: translateY(0);
  }

  .pc__quick-scrub:hover {
    filter: brightness(1.1);
  }

  .pc__quick-scrub:active {
    transform: scale(0.96);
  }
</style>
