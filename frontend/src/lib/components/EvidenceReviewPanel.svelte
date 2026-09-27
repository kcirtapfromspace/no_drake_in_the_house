<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { apiClient } from '../utils/api-client';
  import { isConvexEnabled } from '../convex/client';
  export let artistId: string;
  type Job = { _id: string; sourceUrl: string; sourceTitle?: string; sourceText?: string; status: string; model: string;
    decision?: { role: string; proceduralState: string; categories: string[]; passage?: { text: string } | null; reason: string } };
  let jobs: Job[] = [];
  let selected = '';
  let category = '';
  let severity = 'moderate';
  let reason = '';
  let excerpt = '';
  let proceduralState = 'not_stated';
  const proceduralStates = ['alleged', 'charged', 'convicted', 'acquitted', 'dismissed', 'settled', 'not_stated'];
  let error = '';
  let busy = false;
  const dispatch = createEventDispatcher();
  const categories = ['sexual_misconduct', 'domestic_violence', 'hate_speech', 'racism', 'antisemitism', 'homophobia', 'child_abuse', 'animal_cruelty', 'financial_crimes', 'drug_offenses', 'violent_crimes', 'harassment', 'plagiarism', 'certified_creeper'];
  async function load() {
    if (!isConvexEnabled()) return;
    try {
      const response = await apiClient.get<Job[]>(`/api/v1/artists/${artistId}/evaluations`);
      if (!response.success || !response.data) throw new Error('Queue unavailable');
      jobs = response.data.filter(j => ['needs_review', 'no_support', 'failed'].includes(j.status));
    } catch { error = 'Unable to load the review queue.'; }
  }
  function select(job: Job) { selected = job._id; category = job.decision?.categories[0] || ''; reason = ''; excerpt = job.decision?.passage?.text || ''; proceduralState = job.decision?.proceduralState || 'not_stated'; error = ''; }
  async function review(job: Job, approve: boolean) {
    busy = true; error = '';
    try {
      const response = await apiClient.post(`/api/v1/evaluations/${job._id}/review`, { approve, reason, category, severity, proceduralState, excerpt });
      if (!response.success) throw new Error(response.message || 'Review could not be saved.');
      selected = ''; await load(); dispatch('reviewed');
    } catch (e) { error = e instanceof Error ? e.message : 'Review could not be saved.'; }
    finally { busy = false; }
  }
  async function retry(job: Job) {
    busy = true; error = '';
    try {
      const response = await apiClient.post(`/api/v1/evaluations/${job._id}/retry`, {});
      if (!response.success) throw new Error(response.message || 'Retry could not be queued.');
      await load();
    } catch (e) { error = e instanceof Error ? e.message : 'Retry could not be queued.'; }
    finally { busy = false; }
  }
  onMount(() => { void load(); });
</script>

{#if jobs.length}
  <section class="review-panel" aria-label="Evidence review queue">
    <h2>Evidence awaiting review</h2>
    <p class="muted">Check the source, artist identity, and procedural outcome. Model assessment is advisory; approval records your independent review.</p>
    {#each jobs as job (job._id)}
      <article>
        <a href={job.sourceUrl} target="_blank" rel="noopener noreferrer">{job.sourceTitle || 'Read source'}</a>
        <p class="muted">{job.status.replace(/_/g, ' ')} · {job.model}</p>
        {#if job.decision}
          <p>Reported role: {job.decision.role.replace(/_/g, ' ')}. Procedural state: {job.decision.proceduralState.replace(/_/g, ' ')}.</p>
          {#if job.decision.passage}<blockquote>{job.decision.passage.text}</blockquote>{/if}
          <p class="muted">{job.decision.reason}</p>
        {/if}
        {#if job.status === 'failed'}
          <button type="button" disabled={busy} on:click={() => retry(job)}>Retry assessment</button>
        {:else if selected !== job._id}
          <button type="button" on:click={() => select(job)}>Review source</button>
        {:else}
          <div class="review-fields">
            <label for="review-category">Category</label>
            <select id="review-category" bind:value={category}><option value="">Select a category</option>{#each categories as c}<option value={c}>{c.replace(/_/g, ' ')}</option>{/each}</select>
            <label for="review-severity">Severity</label>
            <select id="review-severity" bind:value={severity}>{#each ['minor', 'moderate', 'severe', 'egregious'] as s}<option value={s}>{s}</option>{/each}</select>
            <label for="review-procedure">Procedural state reported by the source</label>
            <select id="review-procedure" bind:value={proceduralState}>{#each proceduralStates as state}<option value={state}>{state.replace(/_/g, ' ')}</option>{/each}</select>
            {#if job.sourceText}<details><summary>Read retained source</summary><pre>{job.sourceText}</pre></details>{/if}
            <label for="review-excerpt">Supporting passage (exact text from the retained source)</label>
            <textarea id="review-excerpt" bind:value={excerpt} minlength="20" maxlength="1200" rows="4" />
            <label for="review-reason">Review reason</label>
            <textarea id="review-reason" bind:value={reason} minlength="10" maxlength="2000" rows="3" placeholder="What supports your decision, including any uncertainty?" />
            <div class="actions">
              <button type="button" disabled={busy || reason.trim().length < 10 || !category || excerpt.trim().length < 20 || !job.sourceText?.includes(excerpt)} on:click={() => review(job, true)}>Approve evidence</button>
              <button type="button" disabled={busy || reason.trim().length < 10} on:click={() => review(job, false)}>Reject evidence</button>
            </div>
          </div>
        {/if}
      </article>
    {/each}
  </section>
{/if}
{#if error}<p role="alert">{error}</p>{/if}

<style>
  .review-panel { margin-bottom: 1.5rem; padding: 1.25rem; border: 1px solid #3f3f46; border-radius: 1rem; background: #18181b; color: #e4e4e7; }
  h2 { font-size: 1rem; font-weight: 700; margin: 0 0 .5rem; }
  p { font-size: .875rem; line-height: 1.6; margin: .5rem 0; }
  .muted { color: #a1a1aa; } article { border-top: 1px solid #3f3f46; padding-top: 1rem; margin-top: 1rem; }
  a { color: #fda4af; text-decoration: underline; } blockquote { border-left: 2px solid #71717a; padding-left: 1rem; margin: 1rem 0; font-size: .875rem; }
  pre { white-space: pre-wrap; overflow-wrap: anywhere; max-height: 20rem; overflow-y: auto; font: inherit; font-size: .875rem; }
  .review-fields { display: grid; gap: .5rem; margin-top: 1rem; } label { font-size: .8rem; }
  select, textarea { width: 100%; padding: .65rem; border-radius: .5rem; border: 1px solid #52525b; background: #09090b; color: #fafafa; }
  button { border: 1px solid #71717a; border-radius: .5rem; padding: .5rem .75rem; background: #27272a; font-size: .875rem; }
  button:disabled { opacity: .45; cursor: not-allowed; } .actions { display: flex; flex-wrap: wrap; gap: .75rem; margin-top: .5rem; }
  button:focus-visible, a:focus-visible, select:focus-visible, textarea:focus-visible { outline: 2px solid #fb7185; outline-offset: 3px; }
</style>
