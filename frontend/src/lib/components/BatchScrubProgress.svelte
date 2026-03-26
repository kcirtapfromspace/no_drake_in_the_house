<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { sanitizerStore, sanitizerActions } from '../stores/sanitizer';

  const dispatch = createEventDispatcher<{ back: void }>();

  $: batch = $sanitizerStore.batchScrub;
  $: jobs = batch?.jobs ?? [];
  $: completedCount = jobs.filter((j) => j.status === 'completed').length;
  $: failedCount = jobs.filter((j) => j.status === 'failed').length;
  $: totalCount = jobs.length;
  $: progressPct = totalCount > 0 ? Math.round(((completedCount + failedCount) / totalCount) * 100) : 0;
  $: isDone = batch?.status === 'completed' || batch?.status === 'cancelled';

  function handleCancel() {
    sanitizerActions.cancelBatch();
  }

  function handleBack() {
    sanitizerActions.clearBatch();
    dispatch('back');
  }

</script>

<div class="batch">
  <header class="batch__header">
    <h2 class="batch__title">
      {#if isDone}
        Batch Scrub Complete
      {:else}
        Scrubbing Playlists...
      {/if}
    </h2>
    <p class="batch__subtitle">
      {completedCount} of {totalCount} playlists processed
      {#if failedCount > 0}
        <span class="batch__fail-note">({failedCount} failed)</span>
      {/if}
    </p>
  </header>

  <!-- Progress bar -->
  <div class="batch__progress">
    <div class="batch__progress-track">
      <div
        class="batch__progress-fill"
        class:batch__progress-fill--done={isDone}
        style="width: {progressPct}%;"
      ></div>
    </div>
    <span class="batch__progress-pct">{progressPct}%</span>
  </div>

  <!-- Job list -->
  <div class="batch__jobs">
    {#each jobs as job, i (job.playlistId)}
      <div
        class="batch__job"
        class:batch__job--active={batch?.currentIndex === i && !isDone}
        class:batch__job--completed={job.status === 'completed'}
        class:batch__job--failed={job.status === 'failed'}
        class:batch__job--skipped={job.status === 'skipped'}
      >
        <div class="batch__job-status">
          {#if job.status === 'grading'}
            <div class="batch__spinner"></div>
          {:else if job.status === 'completed'}
            <svg class="batch__icon batch__icon--success" width="16" height="16" viewBox="0 0 16 16" fill="none">
              <path d="M4 8l3 3 5-5" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
          {:else if job.status === 'failed'}
            <svg class="batch__icon batch__icon--error" width="16" height="16" viewBox="0 0 16 16" fill="none">
              <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
            </svg>
          {:else if job.status === 'skipped'}
            <svg class="batch__icon batch__icon--muted" width="16" height="16" viewBox="0 0 16 16" fill="none">
              <path d="M4 8h8" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
            </svg>
          {:else}
            <div class="batch__dot"></div>
          {/if}
        </div>

        <div class="batch__job-info">
          <span class="batch__job-name">{job.playlistName}</span>
          <span class="batch__job-provider">{job.provider}</span>
        </div>

        {#if job.gradeLetter}
          <span class="batch__job-grade">{job.gradeLetter}</span>
        {/if}

        {#if job.error}
          <span class="batch__job-error" title={job.error}>{job.error}</span>
        {/if}
      </div>
    {/each}
  </div>

  <!-- Actions -->
  <div class="batch__actions">
    {#if isDone}
      <button type="button" class="batch__btn batch__btn--primary" on:click={handleBack}>
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none"><path d="M10 12L6 8l4-4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>
        Back to Playlists
      </button>
    {:else}
      <button type="button" class="batch__btn batch__btn--cancel" on:click={handleCancel}>
        Cancel
      </button>
    {/if}
  </div>
</div>

<style>
  .batch {
    max-width: 640px;
    margin: 0 auto;
  }

  .batch__header {
    margin-bottom: 1.5rem;
  }

  .batch__title {
    font-size: 1.375rem;
    font-weight: 700;
    color: var(--color-text-primary);
    margin: 0 0 0.25rem;
  }

  .batch__subtitle {
    font-size: 0.875rem;
    color: var(--color-text-tertiary);
    margin: 0;
  }

  .batch__fail-note {
    color: var(--color-error);
  }

  /* Progress */
  .batch__progress {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 1.5rem;
  }

  .batch__progress-track {
    flex: 1;
    height: 6px;
    border-radius: 3px;
    background: var(--color-border-subtle);
    overflow: hidden;
  }

  .batch__progress-fill {
    height: 100%;
    border-radius: 3px;
    background: linear-gradient(90deg, #e11d48, #f43f5e);
    transition: width 0.5s cubic-bezier(.4,0,.2,1);
  }

  .batch__progress-fill--done {
    background: linear-gradient(90deg, #22c55e, #4ade80);
  }

  .batch__progress-pct {
    font-size: 0.75rem;
    font-weight: 700;
    color: var(--color-text-secondary);
    min-width: 2.5rem;
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  /* Job list */
  .batch__jobs {
    display: flex;
    flex-direction: column;
    gap: 2px;
    margin-bottom: 1.5rem;
    max-height: 420px;
    overflow-y: auto;
    border-radius: 0.75rem;
    border: 1px solid var(--color-border-subtle);
  }

  .batch__job {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.625rem 0.875rem;
    background: var(--color-bg-elevated);
    transition: background 0.2s;
  }

  .batch__job--active {
    background: color-mix(in srgb, var(--color-accent-rose, #e11d48) 8%, var(--color-bg-elevated));
  }

  .batch__job--failed {
    background: color-mix(in srgb, var(--color-error) 6%, var(--color-bg-elevated));
  }

  .batch__job--skipped {
    opacity: 0.5;
  }

  /* Status icons */
  .batch__job-status {
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .batch__spinner {
    width: 16px;
    height: 16px;
    border: 2px solid var(--color-border-subtle);
    border-top-color: #e11d48;
    border-radius: 50%;
    animation: batchSpin 0.7s linear infinite;
  }

  @keyframes batchSpin {
    to { transform: rotate(360deg); }
  }

  .batch__icon { color: var(--color-text-muted); }
  .batch__icon--success { color: #22c55e; }
  .batch__icon--error { color: var(--color-error); }
  .batch__icon--muted { color: var(--color-text-tertiary); }

  .batch__dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--color-border-subtle);
  }

  /* Job info */
  .batch__job-info {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .batch__job-name {
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--color-text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .batch__job-provider {
    font-size: 0.625rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-text-tertiary);
    flex-shrink: 0;
  }

  .batch__job-grade {
    font-size: 0.75rem;
    font-weight: 800;
    color: #22c55e;
    flex-shrink: 0;
  }

  .batch__job-error {
    font-size: 0.6875rem;
    color: var(--color-error);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 140px;
  }

  /* Actions */
  .batch__actions {
    display: flex;
    justify-content: center;
    gap: 0.75rem;
  }

  .batch__btn {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.625rem 1.25rem;
    border-radius: 0.5rem;
    font-size: 0.875rem;
    font-weight: 600;
    cursor: pointer;
    border: none;
    color: #fff;
    font-family: inherit;
    transition: transform 0.15s, opacity 0.15s;
  }

  .batch__btn:hover { transform: translateY(-1px); }
  .batch__btn:active { transform: translateY(0) scale(0.98); }

  .batch__btn--primary {
    background: linear-gradient(135deg, #e11d48, #be123c);
  }

  .batch__btn--cancel {
    background: var(--color-bg-inset);
    color: var(--color-text-secondary);
    border: 1px solid var(--color-border-subtle);
  }
</style>
