<script lang="ts">
  import { onMount } from 'svelte';
  import { syncStore, syncActions, isAnySyncRunning, platformsStatus, recentRuns } from '../stores/sync';
  import type { TriggerSyncRequest } from '../stores/sync';
  import { navigateTo } from '../utils/simple-router';

  let selectedPlatforms: string[] = [];
  let syncType: 'full' | 'incremental' = 'incremental';
  let priority: 'low' | 'normal' | 'high' | 'critical' = 'normal';
  let showTriggerModal = false;

  const platforms = [
    { id: 'spotify', name: 'Spotify', icon: '🎵' },
    { id: 'apple', name: 'Apple Music', icon: '🍎' },
    { id: 'tidal', name: 'Tidal', icon: '🌊' },
    { id: 'youtube', name: 'YouTube Music', icon: '▶️' },
    { id: 'deezer', name: 'Deezer', icon: '🎧' },
  ];

  onMount(async () => {
    await Promise.all([
      syncActions.fetchStatus(),
      syncActions.fetchRuns(),
      syncActions.fetchHealth(),
    ]);
  });

  function getStatusColor(status: string): string {
    switch (status) {
      case 'running': return 'bg-blue-500/20 text-blue-400';
      case 'completed': return 'bg-green-500/20 text-green-400';
      case 'error': case 'failed': return 'bg-red-500/20 text-red-400';
      case 'cancelled': return 'bg-zinc-500/20 text-zinc-300';
      default: return 'bg-zinc-500/20 text-zinc-300';
    }
  }

  function getStatusIcon(status: string): string {
    switch (status) {
      case 'running': return '🔄';
      case 'completed': return '✅';
      case 'error': case 'failed': return '❌';
      case 'cancelled': return '⏹️';
      default: return '⏳';
    }
  }

  function formatDuration(ms?: number): string {
    if (!ms) return '-';
    if (ms < 1000) return `${ms}ms`;
    if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
    return `${(ms / 60000).toFixed(1)}m`;
  }

  function formatDate(dateStr?: string): string {
    if (!dateStr) return 'Never';
    const date = new Date(dateStr);
    const now = new Date();
    const diff = now.getTime() - date.getTime();

    if (diff < 60000) return 'Just now';
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
    return date.toLocaleDateString();
  }

  function openTriggerModal() {
    selectedPlatforms = [];
    syncType = 'incremental';
    priority = 'normal';
    showTriggerModal = true;
  }

  function closeTriggerModal() {
    showTriggerModal = false;
  }

  function togglePlatform(platformId: string) {
    if (selectedPlatforms.includes(platformId)) {
      selectedPlatforms = selectedPlatforms.filter(p => p !== platformId);
    } else {
      selectedPlatforms = [...selectedPlatforms, platformId];
    }
  }

  async function handleTriggerSync() {
    if (selectedPlatforms.length === 0) return;

    const request: TriggerSyncRequest = {
      platforms: selectedPlatforms,
      sync_type: syncType,
      priority,
    };

    const result = await syncActions.triggerSync(request);
    if (result.success) {
      closeTriggerModal();
    }
  }

  async function handleCancelRun(runId: string) {
    await syncActions.cancelRun(runId);
  }

  $: healthStatus = $syncStore.health?.overall_status ?? 'unknown';
  $: healthColor = healthStatus === 'healthy' ? 'text-green-600' :
                   healthStatus === 'degraded' ? 'text-yellow-600' : 'text-red-600';
</script>

<div class="min-h-screen" style="background: linear-gradient(to bottom, #27272a, #18181b);">
  <!-- Header -->
  <div class="bg-zinc-900" style="border-bottom: 1px solid #52525b;">
    <div class="max-w-6xl mx-auto px-4 py-8 sm:px-6 lg:px-8">
      <button
        type="button"
        on:click={() => navigateTo('home')}
        class="flex items-center gap-2 text-zinc-400 hover:text-white transition-colors mb-4"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
        </svg>
        Back to Home
      </button>
      <div class="flex items-center justify-between">
        <div>
          <h1 class="text-3xl font-bold text-white mb-2">
            Catalog Sync
          </h1>
          <p class="text-lg text-zinc-400">
            Synchronize artist catalogs across streaming platforms.
          </p>
        </div>
        <button
          type="button"
          on:click={openTriggerModal}
          disabled={$isAnySyncRunning || $syncStore.isTriggering}
          class="px-6 py-3 bg-indigo-600 text-white rounded-xl hover:bg-indigo-700 font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
        >
          {#if $syncStore.isTriggering}
            <span class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></span>
            Starting...
          {:else}
            <span>🔄</span> Trigger Sync
          {/if}
        </button>
      </div>
    </div>
  </div>

  <div class="max-w-6xl mx-auto px-4 py-6 sm:px-6 lg:px-8">
    <!-- Health Status -->
    {#if $syncStore.health}
      <div class="bg-zinc-900 rounded-xl p-4 shadow-sm mb-6" style="border: 1px solid #52525b;">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <span class="text-2xl">
              {healthStatus === 'healthy' ? '💚' : healthStatus === 'degraded' ? '💛' : '❤️'}
            </span>
            <div>
              <span class="font-medium text-white">Overall Health:</span>
              <span class="{healthColor} font-semibold ml-2 capitalize">{healthStatus}</span>
            </div>
          </div>
          <div class="flex items-center gap-4 text-sm text-zinc-400">
            {#each $syncStore.health.platforms as platform}
              <div class="flex items-center gap-1">
                <span class={platform.is_healthy ? 'text-green-500' : 'text-red-500'}>
                  {platform.is_healthy ? '●' : '○'}
                </span>
                <span class="capitalize">{platform.platform}</span>
              </div>
            {/each}
          </div>
        </div>
      </div>
    {/if}

    <!-- Error display -->
    {#if $syncStore.error}
      <div class="bg-red-50 border border-red-200 rounded-xl p-4 mb-6">
        <div class="flex items-center gap-2 text-red-700">
          <span>❌</span>
          <span>{$syncStore.error}</span>
          <button type="button" on:click={syncActions.clearError} class="ml-auto text-red-500 hover:text-red-700">
            Dismiss
          </button>
        </div>
      </div>
    {/if}

    <!-- Platform Status Grid -->
    <div class="mb-8">
      <h2 class="text-xl font-semibold text-white mb-4">Platform Status</h2>
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {#each platforms as platform}
          {@const status = $platformsStatus.find(s => s.platform === platform.id)}
          <div class="bg-zinc-900 rounded-xl p-5 shadow-sm" style="border: 1px solid #52525b;">
            <div class="flex items-start justify-between mb-3">
              <div class="flex items-center gap-3">
                <span class="text-2xl">{platform.icon}</span>
                <div>
                  <div class="font-medium text-white">{platform.name}</div>
                  {#if status}
                    <span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium {getStatusColor(status.status)}">
                      {getStatusIcon(status.status)} {status.status}
                    </span>
                  {:else}
                    <span class="text-xs text-zinc-300">Not synced</span>
                  {/if}
                </div>
              </div>
            </div>
            {#if status}
              <div class="space-y-1 text-sm text-zinc-400">
                <div class="flex justify-between">
                  <span>Artists:</span>
                  <span class="font-medium">{status.artists_count?.toLocaleString() ?? 0}</span>
                </div>
                <div class="flex justify-between">
                  <span>Last sync:</span>
                  <span class="font-medium">{formatDate(status.last_sync)}</span>
                </div>
              </div>
              {#if status.error_message}
                <div class="mt-2 text-xs text-red-600 bg-red-50 rounded p-2">
                  {status.error_message}
                </div>
              {/if}
            {/if}
          </div>
        {/each}
      </div>
    </div>

    <!-- Recent Sync Runs -->
    <div>
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-xl font-semibold text-white">Recent Sync Runs</h2>
        <button
          type="button"
          on:click={() => syncActions.fetchRuns()}
          class="text-indigo-600 hover:text-indigo-700 text-sm font-medium flex items-center gap-1"
        >
          <span>🔄</span> Refresh
        </button>
      </div>

      {#if $syncStore.isLoading}
        <div class="flex justify-center py-12">
          <div class="w-8 h-8 border-4 border-indigo-500 border-t-transparent rounded-full animate-spin"></div>
        </div>
      {:else if $recentRuns.length === 0}
        <div class="rounded-xl p-8 text-center" style="background: #3f3f46;">
          <span class="text-4xl mb-3 block">📭</span>
          <p class="text-zinc-300">No sync runs yet. Trigger your first sync above.</p>
        </div>
      {:else}
        <div class="bg-zinc-900 rounded-xl shadow-sm overflow-hidden" style="border: 1px solid #52525b;">
          <table class="w-full">
            <thead style="background: #3f3f46; border-bottom: 1px solid #52525b;">
              <tr>
                <th class="px-4 py-3 text-left text-xs font-medium text-zinc-400 uppercase">Platform</th>
                <th class="px-4 py-3 text-left text-xs font-medium text-zinc-400 uppercase">Type</th>
                <th class="px-4 py-3 text-left text-xs font-medium text-zinc-400 uppercase">Status</th>
                <th class="px-4 py-3 text-left text-xs font-medium text-zinc-400 uppercase">Artists</th>
                <th class="px-4 py-3 text-left text-xs font-medium text-zinc-400 uppercase">Duration</th>
                <th class="px-4 py-3 text-left text-xs font-medium text-zinc-400 uppercase">Started</th>
                <th class="px-4 py-3 text-left text-xs font-medium text-zinc-400 uppercase">Actions</th>
              </tr>
            </thead>
            <tbody style="border-top: 1px solid #52525b;">
              {#each $recentRuns as run}
                <tr class="hover:bg-zinc-800">
                  <td class="px-4 py-3">
                    <span class="capitalize font-medium">{run.platform}</span>
                  </td>
                  <td class="px-4 py-3">
                    <span class="px-2 py-0.5 rounded text-xs {run.sync_type === 'full' ? 'bg-purple-100 text-purple-700' : 'bg-blue-100 text-blue-700'}">
                      {run.sync_type}
                    </span>
                  </td>
                  <td class="px-4 py-3">
                    <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium {getStatusColor(run.status)}">
                      {getStatusIcon(run.status)} {run.status}
                    </span>
                  </td>
                  <td class="px-4 py-3 text-zinc-300">
                    {run.artists_processed.toLocaleString()}
                    {#if run.errors_count > 0}
                      <span class="text-red-500 ml-1">({run.errors_count} errors)</span>
                    {/if}
                  </td>
                  <td class="px-4 py-3 text-zinc-300">
                    {formatDuration(run.duration_ms)}
                  </td>
                  <td class="px-4 py-3 text-zinc-300">
                    {formatDate(run.started_at)}
                  </td>
                  <td class="px-4 py-3">
                    {#if run.status === 'running' || run.status === 'pending'}
                      <button
                        type="button"
                        on:click={() => handleCancelRun(run.id)}
                        class="text-red-600 hover:text-red-700 text-sm font-medium"
                      >
                        Cancel
                      </button>
                    {:else}
                      <button
                        type="button"
                        on:click={() => syncActions.fetchRun(run.id)}
                        class="text-indigo-600 hover:text-indigo-700 text-sm font-medium"
                      >
                        Details
                      </button>
                    {/if}
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </div>
  </div>
</div>

<!-- Trigger Sync Modal -->
{#if showTriggerModal}
  <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center p-4 z-50" on:click={closeTriggerModal} role="dialog" aria-modal="true">
    <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
    <div class="bg-zinc-900 rounded-2xl max-w-lg w-full p-6 shadow-xl" on:click|stopPropagation role="document">
      <div class="flex items-center mb-6">
        <div class="w-14 h-14 bg-blue-100 rounded-full flex items-center justify-center text-2xl mr-4">
          🔄
        </div>
        <div>
          <h3 class="text-xl font-bold text-white">Trigger Catalog Sync</h3>
          <p class="text-zinc-400">Select platforms and sync options</p>
        </div>
      </div>

      <!-- Platform Selection -->
      <div class="mb-6">
        <label class="block text-sm font-medium text-white mb-3">Platforms</label>
        <div class="grid grid-cols-2 gap-2">
          {#each platforms as platform}
            <button
              type="button"
              on:click={() => togglePlatform(platform.id)}
              class="p-3 rounded-xl border-2 transition-all text-left flex items-center gap-2 text-zinc-300 {
                selectedPlatforms.includes(platform.id)
                  ? 'border-indigo-500 bg-indigo-900'
                  : 'border-zinc-600 hover:border-zinc-500'
              }"
            >
              <span class="text-xl">{platform.icon}</span>
              <span class="font-medium">{platform.name}</span>
            </button>
          {/each}
        </div>
      </div>

      <!-- Sync Type -->
      <div class="mb-6">
        <label class="block text-sm font-medium text-white mb-3">Sync Type</label>
        <div class="grid grid-cols-2 gap-2">
          <button
            type="button"
            on:click={() => syncType = 'incremental'}
            class="p-3 rounded-xl border-2 transition-all text-left text-zinc-300 {
              syncType === 'incremental' ? 'border-indigo-500 bg-indigo-900' : 'border-zinc-600 hover:border-zinc-500'
            }"
          >
            <div class="font-medium">Incremental</div>
            <div class="text-xs text-zinc-400">Only new/changed artists</div>
          </button>
          <button
            type="button"
            on:click={() => syncType = 'full'}
            class="p-3 rounded-xl border-2 transition-all text-left text-zinc-300 {
              syncType === 'full' ? 'border-indigo-500 bg-indigo-900' : 'border-zinc-600 hover:border-zinc-500'
            }"
          >
            <div class="font-medium">Full</div>
            <div class="text-xs text-zinc-400">Complete catalog refresh</div>
          </button>
        </div>
      </div>

      <!-- Priority -->
      <div class="mb-6">
        <label for="priority" class="block text-sm font-medium text-white mb-2">Priority</label>
        <select
          id="priority"
          bind:value={priority}
          class="w-full px-4 py-3 rounded-xl focus:border-indigo-500 focus:ring-2 focus:ring-indigo-200 text-zinc-300 bg-zinc-800"
          style="border: 1px solid #52525b;"
        >
          <option value="low">Low</option>
          <option value="normal">Normal</option>
          <option value="high">High</option>
          <option value="critical">Critical</option>
        </select>
      </div>

      <!-- Actions -->
      <div class="flex gap-3">
        <button
          type="button"
          on:click={closeTriggerModal}
          class="flex-1 px-4 py-3 text-white rounded-xl hover:bg-zinc-700 font-medium transition-colors"
          style="border: 1px solid #52525b;"
        >
          Cancel
        </button>
        <button
          type="button"
          on:click={handleTriggerSync}
          disabled={selectedPlatforms.length === 0 || $syncStore.isTriggering}
          class="flex-1 px-4 py-3 bg-indigo-600 text-white rounded-xl hover:bg-indigo-700 font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {#if $syncStore.isTriggering}
            Starting...
          {:else}
            Start Sync
          {/if}
        </button>
      </div>
    </div>
  </div>
{/if}
