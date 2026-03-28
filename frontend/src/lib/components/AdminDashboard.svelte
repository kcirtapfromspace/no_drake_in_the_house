<script lang="ts">
  import { onMount } from 'svelte';
  import { adminStore, adminActions } from '../stores/admin';
  import { navigateTo } from '../utils/simple-router';
  import { currentUser } from '../stores/auth';
  import { get } from 'svelte/store';

  let isLoading = true;
  let accessDenied = false;

  const CATEGORY_NAMES: Record<string, string> = {
    domestic_violence: 'Domestic Violence',
    sexual_misconduct: 'Sexual Misconduct',
    sexual_assault: 'Sexual Assault',
    child_abuse: 'Child Abuse',
    hate_speech: 'Hate Speech',
    racism: 'Racism',
    antisemitism: 'Antisemitism',
    violent_crime: 'Violent Crime',
    drug_trafficking: 'Drug Trafficking',
    fraud: 'Fraud',
    certified_creeper: 'Certified Creeper',
    homophobia: 'Homophobia',
    animal_abuse: 'Animal Abuse',
    other: 'Other',
  };

  const CRON_JOBS = [
    { name: 'Refresh extension snapshot', interval: 'Every 1 hour' },
    { name: 'Refresh OAuth tokens', interval: 'Every 30 min' },
    { name: 'Promote classifications', interval: 'Every 6 hours' },
    { name: 'Rebuild offending artist index', interval: 'Every 6 hours' },
    { name: 'Snapshot trouble scores', interval: 'Daily 4:00 UTC' },
    { name: 'Snapshot catalog metrics', interval: 'Daily 4:30 UTC' },
    { name: 'Sweep stale summaries', interval: 'Daily 5:00 UTC' },
    { name: 'Investigate library artists', interval: 'Daily 3:00 UTC' },
  ];

  onMount(async () => {
    // Check owner role client-side
    const user = get(currentUser);
    if (!user?.roles?.includes('owner')) {
      accessDenied = true;
      navigateTo('home');
      return;
    }

    await Promise.all([
      adminActions.fetchMetrics(),
      adminActions.fetchHistory(),
    ]);
    isLoading = false;
  });

  $: metrics = $adminStore.metrics;
  $: history = $adminStore.history;
  $: error = $adminStore.error;

  function formatDelta(delta: number | undefined | null): string {
    if (delta === undefined || delta === null) return '';
    if (delta === 0) return '';
    return delta > 0 ? `+${delta.toLocaleString()}` : delta.toLocaleString();
  }

  function deltaClass(delta: number | undefined | null): string {
    if (!delta) return '';
    return delta > 0 ? 'delta-up' : 'delta-down';
  }

  function pipelineProgressPct(): number {
    if (!metrics) return 0;
    const { investigated, never_investigated, stale } = metrics.pipeline;
    const total = investigated + never_investigated + stale;
    return total > 0 ? Math.round((investigated / total) * 100) : 0;
  }

  function estimateDocs(currentCount: number, dailyDelta: number, days: number): string {
    const est = currentCount + (dailyDelta * days);
    return est.toLocaleString();
  }

  function formatDate(iso: string): string {
    try {
      const d = new Date(iso);
      return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
    } catch {
      return iso.slice(0, 10);
    }
  }

  async function handleRefresh() {
    isLoading = true;
    await Promise.all([
      adminActions.fetchMetrics(),
      adminActions.fetchHistory(),
    ]);
    isLoading = false;
  }
</script>

{#if accessDenied}
  <div class="admin-dashboard brand-page surface-page">
    <div class="admin-dashboard__denied">
      <p>Owner access required.</p>
    </div>
  </div>
{:else}
<div class="admin-dashboard brand-page surface-page">
  <div class="admin-dashboard__container brand-page__inner brand-page__stack">

    <!-- Hero -->
    <section class="brand-hero">
      <div class="brand-hero__header">
        <div class="brand-hero__copy">
          <button type="button" on:click={() => navigateTo('home')} class="brand-back">
            <svg fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
            </svg>
            Home
          </button>
          <div class="brand-kickers">
            <span class="brand-kicker">Owner</span>
            <span class="brand-kicker brand-kicker--accent">Admin Dashboard</span>
          </div>
          <h1 class="brand-title brand-title--compact">Catalog health, pipeline progress, and cost fundamentals.</h1>
          <p class="brand-subtitle">
            Monitor your offense database growth, evidence coverage, and backfill pipeline to understand infrastructure costs.
          </p>
        </div>

        <div class="brand-hero__aside">
          <div class="brand-actions">
            <button
              type="button"
              on:click={handleRefresh}
              disabled={isLoading}
              class="brand-button brand-button--secondary"
            >
              {#if isLoading}
                <div class="brand-button__spinner"></div>
              {:else}
                <svg fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                </svg>
              {/if}
              <span>Refresh</span>
            </button>
          </div>
        </div>
      </div>
    </section>

    {#if error}
      <div class="admin-dashboard__error">{error}</div>
    {/if}

    {#if isLoading && !metrics}
      <div class="admin-dashboard__loading">Loading admin metrics...</div>
    {:else if metrics}

      <!-- Section A: Catalog Overview -->
      <section class="admin-section">
        <h2 class="admin-section__title">Catalog Overview</h2>
        <div class="admin-stats-grid">
          <div class="admin-stat-card">
            <span class="admin-stat-card__value">{metrics.catalog.total_artists.toLocaleString()}</span>
            <span class="admin-stat-card__label">Total Artists</span>
            {#if metrics.growth}
              <span class="admin-stat-card__delta {deltaClass(metrics.growth.artists_delta)}">
                {formatDelta(metrics.growth.artists_delta)} since last snapshot
              </span>
            {/if}
          </div>
          <div class="admin-stat-card">
            <span class="admin-stat-card__value">{metrics.catalog.total_offenses.toLocaleString()}</span>
            <span class="admin-stat-card__label">Total Offenses</span>
            {#if metrics.growth}
              <span class="admin-stat-card__delta {deltaClass(metrics.growth.offenses_delta)}">
                {formatDelta(metrics.growth.offenses_delta)} since last snapshot
              </span>
            {/if}
          </div>
          <div class="admin-stat-card">
            <span class="admin-stat-card__value">{metrics.catalog.total_evidence.toLocaleString()}</span>
            <span class="admin-stat-card__label">Evidence Records</span>
            {#if metrics.growth}
              <span class="admin-stat-card__delta {deltaClass(metrics.growth.evidence_delta)}">
                {formatDelta(metrics.growth.evidence_delta)} since last snapshot
              </span>
            {/if}
          </div>
          <div class="admin-stat-card">
            <span class="admin-stat-card__value">{metrics.catalog.total_classifications.toLocaleString()}</span>
            <span class="admin-stat-card__label">Classifications</span>
            <span class="admin-stat-card__sub">{metrics.catalog.pending_classifications.toLocaleString()} pending</span>
          </div>
        </div>

        <div class="admin-stats-grid admin-stats-grid--secondary">
          <div class="admin-stat-card admin-stat-card--small">
            <span class="admin-stat-card__value">{metrics.evidence_density.avg_per_offense}</span>
            <span class="admin-stat-card__label">Avg evidence / offense</span>
          </div>
          <div class="admin-stat-card admin-stat-card--small">
            <span class="admin-stat-card__value {metrics.evidence_density.zero_evidence_pct > 50 ? 'text-warning' : ''}">{metrics.evidence_density.zero_evidence_pct}%</span>
            <span class="admin-stat-card__label">Offenses with 0 evidence</span>
          </div>
          <div class="admin-stat-card admin-stat-card--small">
            <span class="admin-stat-card__value">{metrics.catalog.total_news_articles.toLocaleString()}</span>
            <span class="admin-stat-card__label">News Articles</span>
          </div>
        </div>
      </section>

      <!-- Section B: Category Health Matrix -->
      <section class="admin-section">
        <h2 class="admin-section__title">Category Coverage</h2>
        <div class="admin-table-wrap">
          <table class="admin-table">
            <thead>
              <tr>
                <th>Category</th>
                <th class="num">Artists</th>
                <th class="num">Offenses</th>
                <th class="num">Evidence %</th>
              </tr>
            </thead>
            <tbody>
              {#each metrics.category_coverage.sort((a, b) => b.offense_count - a.offense_count) as cat}
                <tr class:zero-row={cat.offense_count === 0}>
                  <td class="cat-name">{CATEGORY_NAMES[cat.category] ?? cat.category}</td>
                  <td class="num">{cat.unique_artist_count}</td>
                  <td class="num">{cat.offense_count}</td>
                  <td class="num">
                    <span class:low-coverage={cat.evidence_coverage_pct < 30 && cat.offense_count > 0}>
                      {cat.offense_count > 0 ? `${cat.evidence_coverage_pct}%` : '--'}
                    </span>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      </section>

      <!-- Section C: Backfill Pipeline Monitor -->
      <section class="admin-section">
        <h2 class="admin-section__title">Backfill Pipeline</h2>

        <div class="pipeline-progress">
          <div class="pipeline-progress__bar">
            <div class="pipeline-progress__fill" style="width: {pipelineProgressPct()}%"></div>
          </div>
          <span class="pipeline-progress__label">{pipelineProgressPct()}% investigated</span>
        </div>

        <div class="admin-stats-grid">
          <div class="admin-stat-card admin-stat-card--small">
            <span class="admin-stat-card__value text-green">{metrics.pipeline.investigated.toLocaleString()}</span>
            <span class="admin-stat-card__label">Investigated</span>
          </div>
          <div class="admin-stat-card admin-stat-card--small">
            <span class="admin-stat-card__value text-warning">{metrics.pipeline.never_investigated.toLocaleString()}</span>
            <span class="admin-stat-card__label">Never Investigated</span>
          </div>
          <div class="admin-stat-card admin-stat-card--small">
            <span class="admin-stat-card__value text-muted">{metrics.pipeline.stale.toLocaleString()}</span>
            <span class="admin-stat-card__label">Stale ({'>'}30 days)</span>
          </div>
        </div>

        <div class="pipeline-runs">
          <h3 class="pipeline-runs__title">Recent Runs</h3>
          <div class="admin-table-wrap">
            <table class="admin-table admin-table--compact">
              <thead>
                <tr>
                  <th>Window</th>
                  <th class="num">Total</th>
                  <th class="num">Success</th>
                  <th class="num">Failed</th>
                  <th class="num">Offenses Found</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td>Last 24h</td>
                  <td class="num">{metrics.pipeline.recent_runs_24h.total}</td>
                  <td class="num text-green">{metrics.pipeline.recent_runs_24h.success}</td>
                  <td class="num {metrics.pipeline.recent_runs_24h.failed > 0 ? 'text-error' : ''}">{metrics.pipeline.recent_runs_24h.failed}</td>
                  <td class="num">{metrics.pipeline.recent_runs_24h.offenses_found}</td>
                </tr>
                <tr>
                  <td>Last 7d</td>
                  <td class="num">{metrics.pipeline.recent_runs_7d.total}</td>
                  <td class="num text-green">{metrics.pipeline.recent_runs_7d.success}</td>
                  <td class="num {metrics.pipeline.recent_runs_7d.failed > 0 ? 'text-error' : ''}">{metrics.pipeline.recent_runs_7d.failed}</td>
                  <td class="num">{metrics.pipeline.recent_runs_7d.offenses_found}</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </section>

      <!-- Section D: Growth History -->
      <section class="admin-section">
        <h2 class="admin-section__title">Growth History</h2>
        {#if history.length === 0}
          <p class="admin-section__empty">No snapshots yet. The daily cron job at 4:30 AM UTC will begin recording catalog metrics.</p>
        {:else}
          <div class="admin-table-wrap">
            <table class="admin-table admin-table--compact">
              <thead>
                <tr>
                  <th>Date</th>
                  <th class="num">Artists</th>
                  <th class="num">Offenses</th>
                  <th class="num">Evidence</th>
                  <th class="num">Articles</th>
                  <th class="num">Classifications</th>
                </tr>
              </thead>
              <tbody>
                {#each history as snap, i}
                  {@const prev = i > 0 ? history[i - 1] : null}
                  <tr>
                    <td>{formatDate(snap.date)}</td>
                    <td class="num">
                      {snap.total_artists?.toLocaleString() ?? '--'}
                      {#if prev && snap.total_artists !== prev.total_artists}
                        <span class="inline-delta {deltaClass((snap.total_artists ?? 0) - (prev.total_artists ?? 0))}">
                          {formatDelta((snap.total_artists ?? 0) - (prev.total_artists ?? 0))}
                        </span>
                      {/if}
                    </td>
                    <td class="num">
                      {snap.total_offenses?.toLocaleString() ?? '--'}
                      {#if prev && snap.total_offenses !== prev.total_offenses}
                        <span class="inline-delta {deltaClass((snap.total_offenses ?? 0) - (prev.total_offenses ?? 0))}">
                          {formatDelta((snap.total_offenses ?? 0) - (prev.total_offenses ?? 0))}
                        </span>
                      {/if}
                    </td>
                    <td class="num">
                      {snap.total_evidence?.toLocaleString() ?? '--'}
                    </td>
                    <td class="num">
                      {snap.total_news_articles?.toLocaleString() ?? '--'}
                    </td>
                    <td class="num">
                      {snap.total_classifications?.toLocaleString() ?? '--'}
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      </section>

      <!-- Section E: Cost Indicators -->
      <section class="admin-section">
        <h2 class="admin-section__title">Cost Indicators</h2>

        <div class="cost-grid">
          <div class="cost-section">
            <h3 class="cost-section__title">Document Counts by Table</h3>
            <div class="admin-table-wrap">
              <table class="admin-table admin-table--compact">
                <thead>
                  <tr><th>Table</th><th class="num">Documents</th></tr>
                </thead>
                <tbody>
                  <tr><td>artists</td><td class="num">{metrics.catalog.total_artists.toLocaleString()}</td></tr>
                  <tr><td>artistOffenses</td><td class="num">{metrics.catalog.total_offenses.toLocaleString()}</td></tr>
                  <tr><td>offenseEvidence</td><td class="num">{metrics.catalog.total_evidence.toLocaleString()}</td></tr>
                  <tr><td>newsArticles</td><td class="num">{metrics.catalog.total_news_articles.toLocaleString()}</td></tr>
                  <tr><td>newsOffenseClassifications</td><td class="num">{metrics.catalog.total_classifications.toLocaleString()}</td></tr>
                </tbody>
              </table>
            </div>
          </div>

          <div class="cost-section">
            <h3 class="cost-section__title">Cron Execution Frequency</h3>
            <div class="admin-table-wrap">
              <table class="admin-table admin-table--compact">
                <thead>
                  <tr><th>Job</th><th>Schedule</th></tr>
                </thead>
                <tbody>
                  {#each CRON_JOBS as job}
                    <tr><td>{job.name}</td><td>{job.interval}</td></tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </div>
        </div>

        {#if metrics.growth}
          {@const dailyArtists = metrics.growth.artists_delta}
          {@const dailyOffenses = metrics.growth.offenses_delta}
          {@const dailyEvidence = metrics.growth.evidence_delta}
          <div class="cost-projections">
            <h3 class="cost-section__title">Growth Projections (at current daily rate)</h3>
            <div class="admin-table-wrap">
              <table class="admin-table admin-table--compact">
                <thead>
                  <tr><th>Metric</th><th class="num">Now</th><th class="num">Daily Rate</th><th class="num">+30 days</th><th class="num">+90 days</th></tr>
                </thead>
                <tbody>
                  <tr>
                    <td>Artists</td>
                    <td class="num">{metrics.catalog.total_artists.toLocaleString()}</td>
                    <td class="num">{formatDelta(dailyArtists)}</td>
                    <td class="num">{estimateDocs(metrics.catalog.total_artists, dailyArtists, 30)}</td>
                    <td class="num">{estimateDocs(metrics.catalog.total_artists, dailyArtists, 90)}</td>
                  </tr>
                  <tr>
                    <td>Offenses</td>
                    <td class="num">{metrics.catalog.total_offenses.toLocaleString()}</td>
                    <td class="num">{formatDelta(dailyOffenses)}</td>
                    <td class="num">{estimateDocs(metrics.catalog.total_offenses, dailyOffenses, 30)}</td>
                    <td class="num">{estimateDocs(metrics.catalog.total_offenses, dailyOffenses, 90)}</td>
                  </tr>
                  <tr>
                    <td>Evidence</td>
                    <td class="num">{metrics.catalog.total_evidence.toLocaleString()}</td>
                    <td class="num">{formatDelta(dailyEvidence)}</td>
                    <td class="num">{estimateDocs(metrics.catalog.total_evidence, dailyEvidence, 30)}</td>
                    <td class="num">{estimateDocs(metrics.catalog.total_evidence, dailyEvidence, 90)}</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        {/if}
      </section>

    {/if}
  </div>
</div>
{/if}

<style>
  .admin-dashboard {
    min-height: 100vh;
  }

  .admin-dashboard__container {
    width: 100%;
  }

  .admin-dashboard__error {
    padding: 0.75rem 1rem;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    border-radius: var(--radius-lg);
    color: #f87171;
    font-size: var(--text-sm);
    margin-bottom: 1rem;
  }

  .admin-dashboard__loading {
    text-align: center;
    padding: 3rem;
    color: var(--color-text-secondary);
    font-size: var(--text-sm);
  }

  .admin-dashboard__denied {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 50vh;
    color: var(--color-text-secondary);
    font-size: var(--text-base);
  }

  /* Sections */
  .admin-section {
    margin-bottom: 2rem;
  }

  .admin-section__title {
    font-size: var(--text-lg);
    font-weight: 600;
    color: var(--color-text-primary);
    margin-bottom: 1rem;
    letter-spacing: -0.01em;
  }

  .admin-section__empty {
    color: var(--color-text-secondary);
    font-size: var(--text-sm);
    padding: 1.5rem;
    background: rgba(17, 17, 19, 0.88);
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: var(--radius-lg);
  }

  /* Stat Cards */
  .admin-stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 0.75rem;
    margin-bottom: 1rem;
  }

  .admin-stats-grid--secondary {
    grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
  }

  .admin-stat-card {
    background:
      linear-gradient(180deg, rgba(255, 255, 255, 0.045), rgba(255, 255, 255, 0.02)),
      rgba(17, 17, 19, 0.88);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: var(--radius-xl);
    padding: 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    box-shadow: 0 20px 48px rgba(0, 0, 0, 0.18);
    backdrop-filter: blur(12px);
  }

  .admin-stat-card--small {
    padding: 1rem;
  }

  .admin-stat-card__value {
    font-size: 1.75rem;
    font-weight: 700;
    color: var(--color-text-primary);
    letter-spacing: -0.02em;
    line-height: 1.1;
  }

  .admin-stat-card--small .admin-stat-card__value {
    font-size: 1.25rem;
  }

  .admin-stat-card__label {
    font-size: var(--text-xs);
    font-weight: 500;
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .admin-stat-card__delta {
    font-size: var(--text-xs);
    color: var(--color-text-tertiary);
  }

  .admin-stat-card__sub {
    font-size: var(--text-xs);
    color: var(--color-text-tertiary);
  }

  /* Delta indicators */
  .delta-up {
    color: #4ade80;
  }

  .delta-down {
    color: #f87171;
  }

  .inline-delta {
    font-size: 0.7em;
    margin-left: 0.35em;
  }

  /* Color helpers */
  .text-green {
    color: #4ade80;
  }

  .text-warning {
    color: #fbbf24;
  }

  .text-error {
    color: #f87171;
  }

  .text-muted {
    color: var(--color-text-tertiary);
  }

  /* Tables */
  .admin-table-wrap {
    overflow-x: auto;
    border-radius: var(--radius-lg);
    border: 1px solid rgba(255, 255, 255, 0.06);
    background: rgba(17, 17, 19, 0.88);
  }

  .admin-table {
    width: 100%;
    border-collapse: collapse;
    font-size: var(--text-sm);
  }

  .admin-table--compact {
    font-size: var(--text-xs);
  }

  .admin-table thead th {
    padding: 0.625rem 1rem;
    font-weight: 600;
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    font-size: 0.65rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    text-align: left;
    white-space: nowrap;
  }

  .admin-table thead th.num {
    text-align: right;
  }

  .admin-table tbody td {
    padding: 0.5rem 1rem;
    color: var(--color-text-primary);
    border-bottom: 1px solid rgba(255, 255, 255, 0.03);
  }

  .admin-table tbody td.num {
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  .admin-table tbody td.cat-name {
    font-weight: 500;
  }

  .admin-table tbody tr.zero-row td {
    color: var(--color-text-tertiary);
  }

  .admin-table tbody tr.zero-row td.cat-name {
    color: #f87171;
  }

  .low-coverage {
    color: #fbbf24;
    font-weight: 600;
  }

  /* Pipeline progress */
  .pipeline-progress {
    margin-bottom: 1rem;
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .pipeline-progress__bar {
    flex: 1;
    height: 8px;
    background: rgba(255, 255, 255, 0.06);
    border-radius: 4px;
    overflow: hidden;
  }

  .pipeline-progress__fill {
    height: 100%;
    background: linear-gradient(90deg, #4ade80, #22c55e);
    border-radius: 4px;
    transition: width 0.5s ease;
  }

  .pipeline-progress__label {
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--color-text-secondary);
    white-space: nowrap;
  }

  .pipeline-runs {
    margin-top: 1.5rem;
  }

  .pipeline-runs__title {
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--color-text-secondary);
    margin-bottom: 0.5rem;
  }

  /* Cost grid */
  .cost-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
    margin-bottom: 1.5rem;
  }

  .cost-section__title {
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--color-text-secondary);
    margin-bottom: 0.5rem;
  }

  .cost-projections {
    margin-top: 0.5rem;
  }

  @media (max-width: 768px) {
    .admin-stats-grid {
      grid-template-columns: repeat(2, 1fr);
    }

    .cost-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
