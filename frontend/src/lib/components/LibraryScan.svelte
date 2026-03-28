<script lang="ts">
  import { onMount } from 'svelte';
  import { slide } from 'svelte/transition';
  import { navigateTo } from '../utils/simple-router';
  import {
    libraryStore,
    libraryActions,
    flaggedArtists,
    scanStats,
    severityConfig,
    categoryLabels,
    type FlaggedArtist,
  } from '../stores/library';
  import { dnpActions } from '../stores/dnp';

  // Reactive state from store
  $: isScanning = $libraryStore.isScanning;
  $: isLoadingCached = $libraryStore.isLoadingCached;
  $: scanProgress = $libraryStore.scanProgress;
  $: scanComplete = $libraryStore.scanResult !== null;
  $: scannedAt = $libraryStore.scanResult?.scanned_at || null;
  $: error = $libraryStore.error;

  // Expandable rows state
  let expandedArtists = new Set<string>();
  let slideDuration = 200;

  // Auto-load cached scan results on mount
  onMount(() => {
    if (!$libraryStore.scanResult && !$libraryStore.isScanning) {
      libraryActions.fetchCachedScan();
    }

    const mql = window.matchMedia('(prefers-reduced-motion: reduce)');
    if (mql.matches) slideDuration = 0;
  });

  function toggleArtist(id: string) {
    if (expandedArtists.has(id)) {
      expandedArtists.delete(id);
    } else {
      expandedArtists.add(id);
    }
    expandedArtists = expandedArtists; // trigger reactivity
  }

  function formatRelativeTime(dateStr: string): string {
    const date = new Date(dateStr);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    if (diffMins < 1) return 'just now';
    if (diffMins < 60) return `${diffMins}m ago`;
    const diffHours = Math.floor(diffMins / 60);
    if (diffHours < 24) return `${diffHours}h ago`;
    const diffDays = Math.floor(diffHours / 24);
    if (diffDays < 30) return `${diffDays}d ago`;
    return date.toLocaleDateString();
  }

  async function startScan() {
    await libraryActions.scanLibrary();
  }

  function getSeverityCount(severity: string): number {
    return $flaggedArtists.filter(a => a.severity === severity).length;
  }

  async function blockArtist(artist: FlaggedArtist) {
    const tags = artist.offenses.map(o => o.category);
    const note = artist.offenses.map(o => `${categoryLabels[o.category]}: ${o.title} (${o.date})`).join('\n');

    await dnpActions.addArtist(artist.id, tags, note);

    libraryStore.update(s => ({
      ...s,
      scanResult: s.scanResult ? {
        ...s.scanResult,
        flagged_artists: s.scanResult.flagged_artists.filter(a => a.id !== artist.id)
      } : null
    }));
  }

  async function blockAll() {
    for (const artist of $flaggedArtists) {
      await blockArtist(artist);
    }
  }

  /** Map severity keys to CSS modifier classes */
  const severityModifiers: Record<string, string> = {
    critical: 'scan-severity--critical',
    high: 'scan-severity--high',
    medium: 'scan-severity--medium',
    low: 'scan-severity--low',
  };
</script>

<div class="scan brand-page surface-page">
  <div class="scan__container brand-page__inner brand-page__stack">
    <section class="brand-hero scan__hero">
      <div class="brand-hero__header">
        <div class="brand-hero__copy">
          <button
            type="button"
            on:click={() => navigateTo('sync')}
            class="brand-back"
          >
            <svg fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
            </svg>
            Back to Library Control
          </button>
          <div class="brand-kickers">
            <span class="brand-kicker">Library Audit</span>
            <span class="brand-kicker brand-kicker--accent">Evidence Scan</span>
          </div>
          <h1 class="brand-title brand-title--compact">Scan the collection, not just the latest sync.</h1>
          <p class="brand-subtitle">
            Cross-reference imported tracks against documented offenses, keep the severity breakdown readable, and move into blocking without dropping into a different UI language.
          </p>
        </div>

        <div class="brand-hero__aside">
          <div class="brand-stat-grid brand-stat-grid--compact" aria-label="Scan overview">
            <div class="brand-stat">
              <span class="brand-stat__value">{scanComplete && $scanStats ? $scanStats.flaggedArtists : '--'}</span>
              <span class="brand-stat__label">Flagged artists</span>
            </div>
            <div class="brand-stat">
              <span class="brand-stat__value">{scanComplete && $scanStats ? $scanStats.totalTracks.toLocaleString() : '--'}</span>
              <span class="brand-stat__label">Tracks scanned</span>
            </div>
          </div>
        </div>
      </div>
    </section>

    <div class="scan__content">
    {#if error}
      <div class="scan__error" role="alert">
        <svg class="scan__error-icon" fill="currentColor" viewBox="0 0 20 20">
          <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
        </svg>
        <div class="scan__error-body">
          <p>{error}</p>
          <button
            type="button"
            on:click={libraryActions.clearError}
            class="scan__error-dismiss"
          >
            Dismiss
          </button>
        </div>
      </div>
    {/if}

    {#if isLoadingCached}
      <!-- Loading cached results -->
      <div class="scan__prompt">
        <div class="scan__prompt-icon scan__prompt-icon--pulse">
          <svg class="scan__prompt-svg" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
        </div>
        <h2 class="scan__prompt-title">Loading Previous Results...</h2>
        <p class="scan__prompt-desc">
          Checking for cached scan results
        </p>
      </div>

    {:else if !scanComplete && !isScanning}
      <!-- Start scan prompt -->
      <div class="scan__prompt">
        <div class="scan__prompt-icon">
          <svg class="scan__prompt-svg" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
        </div>
        <h2 class="scan__prompt-title">Check Your Library</h2>
        <p class="scan__prompt-desc">
          We'll cross-reference your imported music library against our evidence database
          of artists with documented harmful behavior.
        </p>

        <div class="scan__steps">
          <h3 class="scan__steps-title">Before you scan:</h3>
          <ul class="scan__steps-list">
            <li class="scan__step">
              <span class="scan__step-num">1.</span>
              <span>Import your library from <button type="button" on:click={() => navigateTo('connections')} class="scan__link">Connections</button></span>
            </li>
            <li class="scan__step">
              <span class="scan__step-num">2.</span>
              <span>Upload your Spotify or Apple Music export file</span>
            </li>
            <li class="scan__step">
              <span class="scan__step-num">3.</span>
              <span>Click scan to check against our database</span>
            </li>
          </ul>
        </div>

        <button
          type="button"
          on:click={startScan}
          class="scan__start-btn"
        >
          Start Library Scan
        </button>
      </div>

    {:else if isScanning}
      <!-- Scanning in progress -->
      <div class="scan__prompt">
        <div class="scan__prompt-icon scan__prompt-icon--pulse">
          <svg class="scan__prompt-svg" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
        </div>
        <h2 class="scan__prompt-title">Scanning Your Library...</h2>
        <p class="scan__prompt-desc">
          Cross-referencing your music against our evidence database
        </p>

        <div class="scan__progress">
          <div class="scan__progress-track">
            <div
              class="scan__progress-fill"
              style="width: {scanProgress}%"
            ></div>
          </div>
          <p class="scan__progress-text">{scanProgress}% complete</p>
        </div>
      </div>

    {:else}
      <!-- Scan results -->
      <div class="scan__results">
        <!-- Summary card -->
        <div class="scan__summary">
          <div class="scan__summary-header">
            <div>
              <h2 class="scan__summary-title">Scan Results</h2>
              {#if scannedAt}
                <p class="scan__summary-time">Last scanned: {formatRelativeTime(scannedAt)}</p>
              {/if}
            </div>
            <button
              type="button"
              on:click={startScan}
              class="scan__rescan-btn"
            >
              Scan Again
            </button>
          </div>

          {#if $scanStats}
            <div class="scan__stats-grid">
              <div class="scan__stat">
                <div class="scan__stat-value">{$scanStats.totalTracks.toLocaleString()}</div>
                <div class="scan__stat-label">Total Tracks</div>
              </div>
              <div class="scan__stat">
                <div class="scan__stat-value">{$scanStats.totalArtists}</div>
                <div class="scan__stat-label">Artists</div>
              </div>
              <div class="scan__stat scan__stat--flagged">
                <div class="scan__stat-value">{$scanStats.flaggedArtists}</div>
                <div class="scan__stat-label">Flagged Artists</div>
              </div>
              <div class="scan__stat scan__stat--flagged">
                <div class="scan__stat-value">{$scanStats.flaggedTracks}</div>
                <div class="scan__stat-label">Flagged Tracks</div>
              </div>
            </div>
          {/if}

          <!-- Severity breakdown -->
          {#if $flaggedArtists.length > 0}
            <div class="scan__severity-bar">
              <span class="scan__severity-label">By severity:</span>
              {#each Object.entries(severityConfig) as [key, config]}
                {@const count = getSeverityCount(key)}
                {#if count > 0}
                  <span class="scan__severity-item">
                    <span class="scan__severity-dot {severityModifiers[key] || ''}"></span>
                    {count} {config.label}
                  </span>
                {/if}
              {/each}
            </div>
          {/if}
        </div>

        <!-- Sanitizer card -->
        <div class="scan__sanitizer-card surface-panel-thin">
          <div class="scan__sanitizer-icon">
            <svg width="24" height="24" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3" />
            </svg>
          </div>
          <div class="scan__sanitizer-body">
            <h3 class="scan__sanitizer-title">Playlist Sanitizer</h3>
            <p class="scan__sanitizer-desc">Grade, clean, and republish your playlists without flagged artists</p>
            {#if $scanStats && $scanStats.flaggedTracks > 0}
              <p class="scan__sanitizer-preview">{$scanStats.flaggedTracks} flagged track{$scanStats.flaggedTracks !== 1 ? 's' : ''} found across your library</p>
            {/if}
          </div>
          <button
            type="button"
            class="scan__sanitizer-cta"
            on:click={() => navigateTo('playlist-sanitizer')}
          >
            Open Sanitizer
            <svg width="16" height="16" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
            </svg>
          </button>
        </div>

        <!-- Flagged artists list -->
        {#if $flaggedArtists.length > 0}
          <div class="scan__flagged-header">
            <h3 class="scan__flagged-title">Flagged Artists</h3>
            <button
              type="button"
              on:click={blockAll}
              class="scan__block-all-btn"
            >
              Block All Flagged
            </button>
          </div>

          <div class="scan__list surface-panel-thin">
            {#each $flaggedArtists as artist (artist.id)}
              <button
                type="button"
                class="scan__row"
                on:click={() => toggleArtist(artist.id)}
              >
                <div class="scan__row-avatar">
                  <svg fill="currentColor" viewBox="0 0 20 20" width="16" height="16">
                    <path fill-rule="evenodd" d="M10 9a3 3 0 100-6 3 3 0 000 6zm-7 9a7 7 0 1114 0H3z" clip-rule="evenodd" />
                  </svg>
                </div>
                <div class="scan__row-info">
                  <span class="scan__row-name">{artist.name}</span>
                  <span class="scan__row-meta">{artist.track_count} track{artist.track_count !== 1 ? 's' : ''}</span>
                </div>
                <span class="scan__severity-badge {severityModifiers[artist.severity] || ''}">
                  {severityConfig[artist.severity].label}
                </span>
                <span class="scan__row-count">{artist.offenses.length}</span>
                <svg
                  class="scan__row-chevron {expandedArtists.has(artist.id) ? 'scan__row-chevron--open' : ''}"
                  fill="none" stroke="currentColor" viewBox="0 0 24 24"
                  width="18" height="18"
                >
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                </svg>
              </button>

              {#if expandedArtists.has(artist.id)}
                <div class="scan__row-detail" transition:slide={{ duration: slideDuration }}>
                  <div class="scan__row-offenses">
                    {#each artist.offenses as offense}
                      <div class="scan__row-offense">
                        <span class="scan__row-offense-cat">{categoryLabels[offense.category]}</span>
                        <h5 class="scan__row-offense-title">{offense.title}</h5>
                        <p class="scan__row-offense-meta">{offense.date} &middot; {offense.evidence_count} source{offense.evidence_count !== 1 ? 's' : ''}</p>
                      </div>
                    {/each}
                  </div>
                  <div class="scan__row-actions">
                    <button
                      type="button"
                      class="scan__row-action scan__row-action--block"
                      on:click|stopPropagation={() => blockArtist(artist)}
                    >
                      Block Artist
                    </button>
                    <button
                      type="button"
                      class="scan__row-action scan__row-action--ignore"
                      on:click|stopPropagation={() => {
                        libraryStore.update(s => ({
                          ...s,
                          scanResult: s.scanResult ? {
                            ...s.scanResult,
                            flagged_artists: s.scanResult.flagged_artists.filter(a => a.id !== artist.id)
                          } : null
                        }));
                      }}
                    >
                      Ignore
                    </button>
                    <button
                      type="button"
                      class="scan__row-action scan__row-action--evidence"
                      on:click|stopPropagation={() => navigateTo('offense-database')}
                    >
                      View Evidence
                    </button>
                  </div>
                </div>
              {/if}
            {/each}
          </div>
        {:else}
          <!-- Clean library -->
          <div class="scan__clean">
            <div class="scan__clean-icon">
              <svg class="scan__clean-svg" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
            </div>
            <h3 class="scan__clean-title">Your library looks clean!</h3>
            <p class="scan__clean-desc">
              We didn't find any artists with documented misconduct in your imported music library.
            </p>
          </div>
        {/if}
      </div>
    {/if}

    <!-- Import prompt -->
    {#if !scanComplete}
      <div class="scan__import-prompt">
        <div class="scan__import-icon">
          <svg class="scan__import-svg" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
          </svg>
        </div>
        <div>
          <h4 class="scan__import-title">Import Your Music Library</h4>
          <p class="scan__import-desc">
            To scan your library, first import your music data from your streaming service.
            Go to Connections to upload your Spotify or Apple Music export.
          </p>
          <button
            type="button"
            on:click={() => navigateTo('connections')}
            class="scan__import-link"
          >
            Go to Connections
          </button>
        </div>
      </div>
    {/if}

    <!-- Evidence database info -->
    <div class="scan__info">
      <div class="scan__info-icon">
        <svg class="scan__info-svg" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
        </svg>
      </div>
      <div>
        <h4 class="scan__info-title">About Our Evidence Database</h4>
        <p class="scan__info-desc">
          Our database contains documented cases from court records, major news outlets,
          and verified reports. Each entry is reviewed for accuracy and includes links
          to primary sources.
        </p>
        <button
          type="button"
          on:click={() => navigateTo('community')}
          class="scan__info-link"
        >
          Learn more about our verification process
        </button>
      </div>
    </div>
    </div>
  </div>
</div>

<style>
  /* ===== Root ===== */
  .scan {
    min-height: calc(100vh - 4.5rem);
  }

  .scan__container {
    width: 100%;
  }

  .scan__back-btn {
    display: flex;
    align-items: center;
    color: var(--color-text-tertiary);
    font-size: var(--text-sm);
    margin-bottom: 1rem;
    background: none;
    border: none;
    cursor: pointer;
    transition: color var(--transition-fast);
  }
  .scan__back-btn:hover { color: var(--color-text-primary); }

  .scan__back-icon {
    width: 1rem;
    height: 1rem;
    margin-right: 0.25rem;
  }

  .scan__title {
    font-size: var(--text-3xl);
    font-weight: 700;
    color: var(--color-text-primary);
    margin: 0 0 0.5rem;
  }

  .scan__subtitle {
    font-size: var(--text-lg);
    color: var(--color-text-tertiary);
    margin: 0;
  }

  /* ===== Content ===== */
  .scan__content {
    padding: 0;
  }

  /* ===== Error (CRITICAL FIX: dark theme) ===== */
  .scan__error {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    padding: 1rem;
    margin-bottom: 1.5rem;
    border-radius: var(--radius-xl);
    background-color: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
  }

  .scan__error-icon {
    width: 1.25rem;
    height: 1.25rem;
    color: var(--color-error);
    flex-shrink: 0;
    margin-top: 0.125rem;
  }

  .scan__error-body {
    flex: 1;
  }
  .scan__error-body p {
    margin: 0;
    color: var(--color-error);
  }

  .scan__error-dismiss {
    background: none;
    border: none;
    color: var(--color-error);
    text-decoration: underline;
    font-size: var(--text-sm);
    margin-top: 0.25rem;
    cursor: pointer;
    padding: 0;
  }

  /* ===== Prompt Card (pre-scan & scanning) ===== */
  .scan__prompt {
    background-color: var(--color-bg-elevated);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-2xl);
    box-shadow: var(--shadow-lg);
    padding: 2rem;
    text-align: center;
  }

  .scan__prompt-icon {
    width: 5rem;
    height: 5rem;
    background-color: var(--color-brand-primary-muted);
    border-radius: var(--radius-full);
    display: flex;
    align-items: center;
    justify-content: center;
    margin: 0 auto 1.5rem;
  }
  .scan__prompt-icon--pulse {
    animation: scan-pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
  }

  .scan__prompt-svg {
    width: 2.5rem;
    height: 2.5rem;
    color: var(--color-brand-primary);
  }

  .scan__prompt-title {
    font-size: var(--text-2xl);
    font-weight: 700;
    color: var(--color-text-primary);
    margin: 0 0 1rem;
  }

  .scan__prompt-desc {
    color: var(--color-text-tertiary);
    max-width: 32rem;
    margin: 0 auto 2rem;
  }

  /* ===== Steps ===== */
  .scan__steps {
    background-color: var(--color-bg-interactive);
    border-radius: var(--radius-xl);
    padding: 1.5rem;
    margin-bottom: 2rem;
    text-align: left;
    max-width: 28rem;
    margin-left: auto;
    margin-right: auto;
  }

  .scan__steps-title {
    font-weight: 500;
    color: var(--color-text-primary);
    margin: 0 0 0.75rem;
  }

  .scan__steps-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .scan__step {
    display: flex;
    align-items: flex-start;
    font-size: var(--text-sm);
    color: var(--color-text-secondary);
  }

  .scan__step-num {
    color: var(--color-brand-primary);
    margin-right: 0.5rem;
  }

  .scan__link {
    color: var(--color-brand-primary);
    text-decoration: underline;
    background: none;
    border: none;
    font: inherit;
    cursor: pointer;
    padding: 0;
  }

  /* ===== Start Scan Button ===== */
  .scan__start-btn {
    padding: 1rem 2rem;
    background-color: var(--color-brand-primary);
    color: var(--color-text-on-brand);
    font-size: var(--text-lg);
    font-weight: 500;
    border-radius: var(--radius-xl);
    border: none;
    cursor: pointer;
    transition: background-color var(--transition-fast);
  }
  .scan__start-btn:hover {
    background-color: var(--color-brand-primary-hover);
  }

  /* ===== Progress Bar ===== */
  .scan__progress {
    max-width: 28rem;
    margin: 0 auto;
  }

  .scan__progress-track {
    height: 1rem;
    border-radius: var(--radius-full);
    overflow: hidden;
    background-color: var(--color-bg-interactive);
  }

  .scan__progress-fill {
    height: 100%;
    background-color: var(--color-brand-primary);
    transition: width 300ms ease;
  }

  .scan__progress-text {
    font-size: var(--text-sm);
    color: var(--color-text-tertiary);
    margin-top: 0.5rem;
  }

  /* ===== Results ===== */
  .scan__results {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .scan__summary {
    background-color: var(--color-bg-elevated);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-2xl);
    box-shadow: var(--shadow-lg);
    padding: 1.5rem;
  }

  .scan__summary-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 1rem;
  }

  .scan__summary-title {
    font-size: var(--text-xl);
    font-weight: 700;
    color: var(--color-text-primary);
    margin: 0;
  }

  .scan__summary-time {
    font-size: var(--text-sm);
    color: var(--color-text-tertiary);
    margin: 0.25rem 0 0;
  }

  .scan__rescan-btn {
    font-size: var(--text-sm);
    color: var(--color-text-tertiary);
    background: none;
    border: none;
    cursor: pointer;
    transition: color var(--transition-fast);
  }
  .scan__rescan-btn:hover { color: var(--color-text-primary); }

  /* ===== Stats Grid ===== */
  .scan__stats-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 1rem;
    margin-bottom: 1.5rem;
  }
  @media (min-width: 640px) {
    .scan__stats-grid { grid-template-columns: repeat(4, 1fr); }
  }

  .scan__stat {
    background-color: var(--color-bg-interactive);
    border-radius: var(--radius-xl);
    padding: 1rem;
    text-align: center;
  }

  .scan__stat-value {
    font-size: var(--text-2xl);
    font-weight: 700;
    color: var(--color-text-primary);
  }

  .scan__stat-label {
    font-size: var(--text-sm);
    color: var(--color-text-tertiary);
  }

  .scan__stat--flagged {
    background-color: rgba(239, 68, 68, 0.1);
  }
  .scan__stat--flagged .scan__stat-value {
    color: var(--color-error);
  }
  .scan__stat--flagged .scan__stat-label {
    color: var(--color-error);
  }

  /* ===== Severity ===== */
  .scan__severity-bar {
    display: flex;
    align-items: center;
    gap: 1rem;
    font-size: var(--text-sm);
    flex-wrap: wrap;
  }

  .scan__severity-label {
    color: var(--color-text-tertiary);
  }

  .scan__severity-item {
    display: flex;
    align-items: center;
    color: var(--color-text-secondary);
  }

  .scan__severity-dot {
    width: 0.75rem;
    height: 0.75rem;
    border-radius: var(--radius-full);
    margin-right: 0.25rem;
    background-color: var(--color-text-muted);
  }
  .scan-severity--critical { background-color: #dc2626; }
  .scan-severity--high { background-color: #ea580c; }
  .scan-severity--medium { background-color: #d97706; }
  .scan-severity--low { background-color: #65a30d; }

  /* ===== Severity Badge ===== */
  .scan__severity-badge {
    padding: 0.25rem 0.75rem;
    color: var(--color-text-on-brand);
    font-size: var(--text-xs);
    font-weight: 500;
    border-radius: var(--radius-full);
    background-color: var(--color-text-muted);
    flex-shrink: 0;
  }
  .scan__severity-badge.scan-severity--critical { background-color: #dc2626; }
  .scan__severity-badge.scan-severity--high { background-color: #ea580c; }
  .scan__severity-badge.scan-severity--medium { background-color: #d97706; }
  .scan__severity-badge.scan-severity--low { background-color: #65a30d; }

  /* ===== Sanitizer Card ===== */
  .scan__sanitizer-card {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1.25rem 1.5rem;
    border-radius: var(--radius-2xl);
  }

  .scan__sanitizer-icon {
    width: 3rem;
    height: 3rem;
    background-color: rgba(139, 92, 246, 0.15);
    border-radius: var(--radius-full);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    color: #a78bfa;
  }

  .scan__sanitizer-body {
    flex: 1;
    min-width: 0;
  }

  .scan__sanitizer-title {
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
    font-size: var(--text-base);
  }

  .scan__sanitizer-desc {
    font-size: var(--text-sm);
    color: var(--color-text-tertiary);
    margin: 0.125rem 0 0;
  }

  .scan__sanitizer-preview {
    font-size: var(--text-xs);
    color: #a78bfa;
    margin: 0.25rem 0 0;
  }

  .scan__sanitizer-cta {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.5rem 1rem;
    background-color: #4f46e5;
    color: var(--color-text-on-brand);
    font-size: var(--text-sm);
    font-weight: 500;
    border-radius: var(--radius-lg);
    border: none;
    cursor: pointer;
    white-space: nowrap;
    flex-shrink: 0;
    transition: background-color var(--transition-fast);
  }
  .scan__sanitizer-cta:hover {
    background-color: #4338ca;
  }

  @media (max-width: 639px) {
    .scan__sanitizer-card {
      flex-wrap: wrap;
    }
    .scan__sanitizer-cta {
      width: 100%;
      justify-content: center;
    }
  }

  /* ===== Flagged Header ===== */
  .scan__flagged-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .scan__flagged-title {
    font-size: var(--text-lg);
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
  }

  .scan__block-all-btn {
    padding: 0.5rem 1rem;
    background-color: var(--color-error);
    color: var(--color-text-on-brand);
    font-size: var(--text-sm);
    font-weight: 500;
    border-radius: var(--radius-lg);
    border: none;
    cursor: pointer;
    transition: opacity var(--transition-fast);
  }
  .scan__block-all-btn:hover { opacity: 0.9; }

  /* ===== Dense Row List ===== */
  .scan__list {
    border-radius: var(--radius-2xl);
    overflow: hidden;
  }

  .scan__row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.625rem 1.25rem;
    width: 100%;
    background: none;
    border: none;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
    cursor: pointer;
    text-align: left;
    color: inherit;
    font: inherit;
    transition: background-color var(--transition-fast);
  }
  .scan__row:first-child {
    border-top: none;
  }
  .scan__row:hover {
    background: rgba(255, 255, 255, 0.04);
  }

  .scan__row-avatar {
    width: 2.5rem;
    height: 2.5rem;
    border-radius: var(--radius-full);
    background-color: rgba(63, 63, 70, 1);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    color: var(--color-text-tertiary);
  }

  .scan__row-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .scan__row-name {
    font-weight: 600;
    color: var(--color-text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .scan__row-meta {
    font-size: var(--text-xs);
    color: var(--color-text-tertiary);
  }

  .scan__row-count {
    font-size: var(--text-sm);
    color: var(--color-text-tertiary);
    font-variant-numeric: tabular-nums;
    flex-shrink: 0;
    min-width: 1.5rem;
    text-align: right;
  }

  .scan__row-chevron {
    color: var(--color-text-tertiary);
    flex-shrink: 0;
    transition: transform 0.2s;
  }
  .scan__row-chevron--open {
    transform: rotate(180deg);
  }

  /* ===== Expanded Detail ===== */
  .scan__row-detail {
    padding: 0.75rem 1.25rem 1rem;
    padding-left: 4.5rem;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
  }

  .scan__row-offenses {
    display: flex;
    flex-direction: column;
    gap: 0.625rem;
    margin-bottom: 0.75rem;
  }

  .scan__row-offense {
    padding: 0.625rem 0.75rem;
    background: rgba(239, 68, 68, 0.06);
    border-radius: var(--radius-lg);
  }

  .scan__row-offense-cat {
    font-size: 0.625rem;
    font-weight: 600;
    color: #f472b6;
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  .scan__row-offense-title {
    font-weight: 500;
    color: var(--color-text-primary);
    margin: 0.125rem 0 0;
    font-size: var(--text-sm);
  }

  .scan__row-offense-meta {
    font-size: var(--text-xs);
    color: var(--color-text-tertiary);
    margin: 0.125rem 0 0;
  }

  .scan__row-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .scan__row-action {
    padding: 0.375rem 0.75rem;
    font-size: var(--text-sm);
    font-weight: 500;
    border-radius: var(--radius-lg);
    border: none;
    cursor: pointer;
    transition: opacity var(--transition-fast), background-color var(--transition-fast);
  }

  .scan__row-action--block {
    background-color: var(--color-error);
    color: var(--color-text-on-brand);
  }
  .scan__row-action--block:hover { opacity: 0.9; }

  .scan__row-action--ignore {
    background-color: var(--color-bg-interactive);
    border: 1px solid var(--color-border-default);
    color: var(--color-text-secondary);
  }
  .scan__row-action--ignore:hover {
    background-color: var(--color-bg-hover);
  }

  .scan__row-action--evidence {
    background: none;
    color: #818cf8;
    padding: 0.375rem 0.5rem;
  }
  .scan__row-action--evidence:hover {
    color: #a5b4fc;
  }

  @media (max-width: 639px) {
    .scan__row-count {
      display: none;
    }
    .scan__row {
      padding: 0.5rem 1rem;
    }
    .scan__row-detail {
      padding-left: 1rem;
    }
    .scan__row-actions {
      flex-wrap: wrap;
    }
  }

  /* ===== Clean Library ===== */
  .scan__clean {
    background-color: rgba(34, 197, 94, 0.1);
    border-radius: var(--radius-2xl);
    padding: 2rem;
    text-align: center;
  }

  .scan__clean-icon {
    width: 4rem;
    height: 4rem;
    background-color: rgba(34, 197, 94, 0.15);
    border-radius: var(--radius-full);
    display: flex;
    align-items: center;
    justify-content: center;
    margin: 0 auto 1rem;
  }

  .scan__clean-svg {
    width: 2rem;
    height: 2rem;
    color: var(--color-success);
  }

  .scan__clean-title {
    font-size: var(--text-xl);
    font-weight: 600;
    color: var(--color-success);
    margin: 0 0 0.5rem;
  }

  .scan__clean-desc {
    color: #86efac;
    margin: 0;
  }

  /* ===== Import Prompt ===== */
  .scan__import-prompt {
    display: flex;
    align-items: flex-start;
    gap: 1rem;
    margin-top: 2rem;
    background-color: rgba(59, 130, 246, 0.1);
    border-radius: var(--radius-2xl);
    padding: 1.5rem;
  }

  .scan__import-icon {
    width: 3rem;
    height: 3rem;
    background-color: rgba(59, 130, 246, 0.15);
    border-radius: var(--radius-full);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .scan__import-svg {
    width: 1.5rem;
    height: 1.5rem;
    color: #60a5fa;
  }

  .scan__import-title {
    font-weight: 500;
    color: #93c5fd;
    margin: 0 0 0.25rem;
  }

  .scan__import-desc {
    font-size: var(--text-sm);
    color: #bfdbfe;
    margin: 0 0 0.75rem;
  }

  .scan__import-link {
    font-size: var(--text-sm);
    color: #60a5fa;
    font-weight: 500;
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
    transition: color var(--transition-fast);
  }
  .scan__import-link:hover { color: #93c5fd; }

  /* ===== Info Box ===== */
  .scan__info {
    display: flex;
    align-items: flex-start;
    gap: 1rem;
    margin-top: 2rem;
    background-color: var(--color-bg-interactive);
    border-radius: var(--radius-2xl);
    padding: 1.5rem;
  }

  .scan__info-icon {
    width: 3rem;
    height: 3rem;
    background-color: var(--color-bg-hover);
    border-radius: var(--radius-full);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .scan__info-svg {
    width: 1.5rem;
    height: 1.5rem;
    color: var(--color-text-secondary);
  }

  .scan__info-title {
    font-weight: 500;
    color: var(--color-text-primary);
    margin: 0 0 0.25rem;
  }

  .scan__info-desc {
    font-size: var(--text-sm);
    color: var(--color-text-tertiary);
    margin: 0 0 0.75rem;
  }

  .scan__info-link {
    font-size: var(--text-sm);
    color: var(--color-brand-primary);
    font-weight: 500;
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
    transition: color var(--transition-fast);
  }
  .scan__info-link:hover { color: var(--color-brand-primary-hover); }

  /* ===== Animation ===== */
  @keyframes scan-pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }
</style>
