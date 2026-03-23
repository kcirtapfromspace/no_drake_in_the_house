<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { PlaylistSummary } from '../stores/playlist-browser';

  export let playlist: PlaylistSummary;
  export let index: number = 0;

  const dispatch = createEventDispatcher<{ select: PlaylistSummary }>();

  $: providerColor = playlist.provider === 'spotify' ? 'var(--color-spotify)' : 'var(--color-apple)';
  $: providerLabel = playlist.provider === 'spotify' ? 'Spotify' : 'Apple';
  $: gradeColor = getGradeColor(playlist.grade);
  $: images = playlist.cover_images || [];
  $: cleanPct = Math.round(playlist.clean_ratio * 100);

  // Deterministic gradient from playlist name when no images
  $: gradientHue = hashString(playlist.playlist_name) % 360;
  $: gradientHue2 = (gradientHue + 40 + (hashString(playlist.provider) % 60)) % 360;

  function hashString(s: string): number {
    let h = 0;
    for (let i = 0; i < s.length; i++) {
      h = ((h << 5) - h + s.charCodeAt(i)) | 0;
    }
    return Math.abs(h);
  }

  function getGradeColor(g: string): string {
    switch (g) {
      case 'A+': case 'A': return 'var(--color-success)';
      case 'B': return 'var(--color-info)';
      case 'C': return 'var(--color-warning)';
      case 'D': return '#f97316';
      default: return 'var(--color-error)';
    }
  }

  $: flaggedPreview = playlist.flagged_artists.slice(0, 2);
  $: flaggedMore = playlist.flagged_artists.length - 2;
</script>

<button
  type="button"
  class="pc"
  style="--stagger: {index * 50}ms"
  on:click={() => dispatch('select', playlist)}
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
      <div class="pc__gradient" style="background: linear-gradient(135deg, hsl({gradientHue}, 60%, 25%) 0%, hsl({gradientHue2}, 45%, 15%) 100%);"></div>
    {/if}

    <!-- Overlay with grade -->
    <div class="pc__cover-overlay">
      <span class="pc__grade" style="--gc: {gradeColor};">{playlist.grade}</span>
    </div>

    <!-- Provider pill -->
    <span class="pc__provider" style="--pc: {providerColor};">{providerLabel}</span>
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

    <!-- Clean ratio bar -->
    <div class="pc__bar-track">
      <div class="pc__bar-fill" style="width: {cleanPct}%; background: {gradeColor};"></div>
    </div>

    {#if flaggedPreview.length > 0}
      <p class="pc__flagged-names">
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
    border-radius: 0.625rem;
    background: var(--color-bg-elevated);
    cursor: pointer;
    text-align: left;
    font-family: inherit;
    color: inherit;
    overflow: hidden;
    transition: transform 0.22s cubic-bezier(.4,0,.2,1),
                border-color 0.22s,
                box-shadow 0.22s;
    width: 100%;
    padding: 0;
    animation: cardIn 0.4s cubic-bezier(.4,0,.2,1) both;
    animation-delay: var(--stagger, 0ms);
  }

  @keyframes cardIn {
    from { opacity: 0; transform: translateY(12px) scale(0.97); }
    to { opacity: 1; transform: translateY(0) scale(1); }
  }

  .pc:hover {
    transform: translateY(-3px) scale(1.01);
    border-color: var(--color-border-hover);
    box-shadow: var(--shadow-lg);
  }

  .pc:active {
    transform: translateY(0) scale(0.99);
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
  }

  .pc__hero-img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    max-width: none;
    max-height: none;
  }

  .pc__gradient {
    width: 100%;
    height: 100%;
  }

  .pc__cover-overlay {
    position: absolute;
    inset: 0;
    background: linear-gradient(to top, rgba(0,0,0,0.7) 0%, transparent 50%);
    display: flex;
    align-items: flex-end;
    justify-content: flex-end;
    padding: 0.625rem;
  }

  .pc__grade {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 2rem;
    padding: 0.2rem 0.5rem;
    border-radius: 0.375rem;
    font-size: 0.8125rem;
    font-weight: 800;
    letter-spacing: 0.02em;
    color: var(--gc);
    background: rgba(0,0,0,0.6);
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
    border: 1px solid rgba(255,255,255,0.08);
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
    box-shadow: 0 2px 6px rgba(0,0,0,0.4);
  }

  /* ---- Body ---- */
  .pc__body {
    padding: 0.75rem 0.75rem 0.875rem;
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

  .pc__bar-track {
    height: 3px;
    border-radius: 2px;
    background: var(--color-border-subtle);
    overflow: hidden;
  }

  .pc__bar-fill {
    height: 100%;
    border-radius: 2px;
    transition: width 0.6s cubic-bezier(.4,0,.2,1);
  }

  .pc__flagged-names {
    font-size: 0.6875rem;
    color: var(--color-text-muted);
    margin: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    line-height: 1.4;
  }

  .pc__more {
    color: var(--color-text-tertiary);
  }
</style>
