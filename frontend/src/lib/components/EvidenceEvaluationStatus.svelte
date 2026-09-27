<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { apiClient } from '../utils/api-client';
  export let jobId: string;
  let status = 'queued';
  let error = '';
  let active = true;
  let timer: ReturnType<typeof setTimeout>;
  const labels: Record<string, string> = {
    queued: 'Evidence queued', running: 'Assessing the source', retry_wait: 'Waiting to retry the assessment',
    needs_review: 'Awaiting independent review', no_support: 'No supporting passage identified',
    failed: 'Assessment could not be completed', approved: 'Evidence approved by a reviewer', rejected: 'Evidence was not approved',
  };
  async function refresh() {
    clearTimeout(timer);
    try {
      const response = await apiClient.get<{ status: string }>(`/api/v1/evaluations/${jobId}`);
      if (!active) return;
      if (!response.success || !response.data) throw new Error('Unable to load assessment status.');
      status = response.data.status;
      error = '';
      if (['queued', 'running', 'retry_wait'].includes(status)) timer = setTimeout(refresh, 3000);
    } catch {
      if (active) error = 'Status is temporarily unavailable. Your submission is saved.';
    }
  }
  onMount(() => { void refresh(); });
  onDestroy(() => { active = false; clearTimeout(timer); });
</script>

<section class="evaluation-status" aria-live="polite" aria-atomic="true">
  <strong>{labels[status] || 'Evidence assessment'}</strong>
  {#if status === 'no_support'}
    <p>The source did not yield a supporting passage. This is an assessment of this source, not a conclusion about the artist.</p>
  {:else if status === 'failed'}
    <p>The source remains unverified. A reviewer can retry the assessment once the service is available.</p>
  {:else if !['approved', 'rejected'].includes(status)}
    <p>Submitting a source does not verify an allegation or change anyone's blocklist. Approval requires an independent reviewer.</p>
  {/if}
  {#if error}<p>{error}</p>{/if}
  {#if error || ['needs_review', 'no_support', 'failed'].includes(status)}<button type="button" on:click={refresh}>Refresh status</button>{/if}
</section>

<style>
  .evaluation-status { padding: 1rem; border: 1px solid #3f3f46; border-radius: .75rem; background: #18181b; color: #fafafa; }
  p { margin: .5rem 0 0; color: #a1a1aa; font-size: .875rem; line-height: 1.6; }
  button { margin-top: .75rem; color: #fda4af; text-decoration: underline; }
  button:focus-visible { outline: 2px solid #fb7185; outline-offset: 4px; }
</style>
