<script lang="ts">
  import { onMount } from 'svelte';
  import {
    analyticsStore,
    analyticsActions,
    isSystemHealthy,
    risingArtists,
    fallingArtists
  } from '../stores/analytics';
  import type { ReportRequest } from '../stores/analytics';
  import { navigateTo } from '../utils/simple-router';

  let selectedTimeRange = 'last7days';
  let selectedPeriodDays = 7;
  let showReportModal = false;
  let reportType = 'user_activity';
  let reportFormat: 'json' | 'csv' | 'parquet' | 'html' = 'json';
  let includeDetails = true;

  const timeRanges = [
    { value: 'last24h', label: 'Last 24 Hours', days: 1 },
    { value: 'last7days', label: 'Last 7 Days', days: 7 },
    { value: 'last30days', label: 'Last 30 Days', days: 30 },
    { value: 'last90days', label: 'Last 90 Days', days: 90 },
  ];

  onMount(async () => {
    await Promise.all([
      analyticsActions.fetchDashboard(selectedTimeRange),
      analyticsActions.fetchUserStats(),
      analyticsActions.fetchSystemHealth(),
      analyticsActions.fetchTrendSummary(selectedPeriodDays),
      analyticsActions.fetchRisingArtists(),
      analyticsActions.fetchFallingArtists(),
      analyticsActions.fetchReportTypes(),
    ]);
  });

  async function handleTimeRangeChange() {
    const range = timeRanges.find(r => r.value === selectedTimeRange);
    if (range) {
      selectedPeriodDays = range.days;
      await Promise.all([
        analyticsActions.fetchDashboard(selectedTimeRange),
        analyticsActions.fetchTrendSummary(selectedPeriodDays),
      ]);
    }
  }

  function openReportModal() {
    showReportModal = true;
  }

  function closeReportModal() {
    showReportModal = false;
  }

  async function handleGenerateReport() {
    const request: ReportRequest = {
      report_type: reportType,
      format: reportFormat,
      time_range: selectedTimeRange,
      include_details: includeDetails,
    };

    const result = await analyticsActions.generateReport(request);
    if (result.success && result.reportId) {
      // Start polling for report status
      pollReportStatus(result.reportId);
      closeReportModal();
    }
  }

  async function pollReportStatus(reportId: string) {
    const result = await analyticsActions.fetchReportStatus(reportId);
    if (result.success && result.report) {
      if (result.report.status === 'pending' || result.report.status === 'processing') {
        setTimeout(() => pollReportStatus(reportId), 2000);
      }
    }
  }

  function formatNumber(num: number | undefined): string {
    if (num === undefined) return '0';
    if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`;
    if (num >= 1000) return `${(num / 1000).toFixed(1)}K`;
    return num.toString();
  }

  function getTrendIcon(trend: 'up' | 'down' | 'stable' | 'rising' | 'falling'): string {
    switch (trend) {
      case 'up': case 'rising': return '📈';
      case 'down': case 'falling': return '📉';
      default: return '➡️';
    }
  }

  function getTrendColor(trend: 'up' | 'down' | 'stable' | 'rising' | 'falling'): string {
    switch (trend) {
      case 'up': case 'rising': return 'text-green-600';
      case 'down': case 'falling': return 'text-red-600';
      default: return 'text-gray-600';
    }
  }

  function getHealthIcon(healthy: boolean): string {
    return healthy ? '✅' : '❌';
  }

  function getHealthColor(healthy: boolean): string {
    return healthy ? 'text-green-600' : 'text-red-600';
  }

  $: dashboard = $analyticsStore.dashboard;
  $: userStats = $analyticsStore.userStats;
  $: systemHealth = $analyticsStore.systemHealth;
  $: trendSummary = $analyticsStore.trends.summary;
</script>

<div class="min-h-screen bg-gradient-to-b from-purple-50 to-white">
  <!-- Header -->
  <div class="bg-white border-b border-gray-100">
    <div class="max-w-6xl mx-auto px-4 py-8 sm:px-6 lg:px-8">
      <button
        type="button"
        on:click={() => navigateTo('home')}
        class="flex items-center gap-2 text-gray-500 hover:text-gray-900 transition-colors mb-4"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
        </svg>
        Back to Home
      </button>
      <div class="flex items-center justify-between">
        <div>
          <h1 class="text-3xl font-bold text-gray-900 mb-2">
            Analytics Dashboard
          </h1>
          <p class="text-lg text-gray-600">
            Monitor system health, trends, and generate reports.
          </p>
        </div>
        <div class="flex items-center gap-4">
          <select
            bind:value={selectedTimeRange}
            on:change={handleTimeRangeChange}
            class="px-4 py-2 border border-gray-200 rounded-lg focus:border-indigo-500 focus:ring-2 focus:ring-indigo-200"
          >
            {#each timeRanges as range}
              <option value={range.value}>{range.label}</option>
            {/each}
          </select>
          <button
            type="button"
            on:click={openReportModal}
            class="px-6 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 font-medium transition-colors flex items-center gap-2"
          >
            <span>📊</span> Generate Report
          </button>
        </div>
      </div>
    </div>
  </div>

  <div class="max-w-6xl mx-auto px-4 py-6 sm:px-6 lg:px-8">
    <!-- Error display -->
    {#if $analyticsStore.error}
      <div class="bg-red-50 border border-red-200 rounded-xl p-4 mb-6">
        <div class="flex items-center gap-2 text-red-700">
          <span>❌</span>
          <span>{$analyticsStore.error}</span>
          <button type="button" on:click={analyticsActions.clearError} class="ml-auto text-red-500 hover:text-red-700">
            Dismiss
          </button>
        </div>
      </div>
    {/if}

    <!-- Loading state -->
    {#if $analyticsStore.isLoading}
      <div class="flex justify-center py-12">
        <div class="w-8 h-8 border-4 border-indigo-500 border-t-transparent rounded-full animate-spin"></div>
      </div>
    {:else}
      <!-- Quick Stats Cards -->
      <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mb-8">
        <div class="bg-white rounded-xl p-5 border border-gray-100 shadow-sm">
          <div class="flex items-center gap-3">
            <div class="w-12 h-12 bg-blue-100 rounded-full flex items-center justify-center text-xl">
              👥
            </div>
            <div>
              <div class="text-2xl font-bold text-gray-900">{formatNumber(dashboard?.total_users)}</div>
              <div class="text-sm text-gray-500">Total Users</div>
            </div>
          </div>
        </div>

        <div class="bg-white rounded-xl p-5 border border-gray-100 shadow-sm">
          <div class="flex items-center gap-3">
            <div class="w-12 h-12 bg-green-100 rounded-full flex items-center justify-center text-xl">
              🟢
            </div>
            <div>
              <div class="text-2xl font-bold text-gray-900">{formatNumber(dashboard?.active_users_today)}</div>
              <div class="text-sm text-gray-500">Active Today</div>
            </div>
          </div>
        </div>

        <div class="bg-white rounded-xl p-5 border border-gray-100 shadow-sm">
          <div class="flex items-center gap-3">
            <div class="w-12 h-12 bg-red-100 rounded-full flex items-center justify-center text-xl">
              🚫
            </div>
            <div>
              <div class="text-2xl font-bold text-gray-900">{formatNumber(dashboard?.total_blocked_artists)}</div>
              <div class="text-sm text-gray-500">Blocked Artists</div>
            </div>
          </div>
        </div>

        <div class="bg-white rounded-xl p-5 border border-gray-100 shadow-sm">
          <div class="flex items-center gap-3">
            <div class="w-12 h-12 bg-purple-100 rounded-full flex items-center justify-center text-xl">
              🔔
            </div>
            <div>
              <div class="text-2xl font-bold text-gray-900">{formatNumber(dashboard?.offense_detections_today)}</div>
              <div class="text-sm text-gray-500">Offenses Today</div>
            </div>
          </div>
        </div>
      </div>

      <!-- User Quick Stats -->
      {#if userStats}
        <div class="bg-white rounded-xl p-6 border border-gray-100 shadow-sm mb-8">
          <h2 class="text-lg font-semibold text-gray-900 mb-4">Your Quick Stats</h2>
          <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div class="text-center">
              <div class="text-2xl font-bold text-indigo-600">{userStats.blocked_artists}</div>
              <div class="text-sm text-gray-500">Blocked Artists</div>
            </div>
            <div class="text-center">
              <div class="text-2xl font-bold text-indigo-600">{userStats.subscriptions}</div>
              <div class="text-sm text-gray-500">Subscriptions</div>
            </div>
            <div class="text-center">
              <div class="text-2xl font-bold text-indigo-600">{userStats.manual_blocks}</div>
              <div class="text-sm text-gray-500">Manual Blocks</div>
            </div>
            <div class="text-center">
              <div class="text-sm font-medium text-gray-600">{userStats.last_sync ?? 'Never'}</div>
              <div class="text-sm text-gray-500">Last Sync</div>
            </div>
          </div>
        </div>
      {/if}

      <!-- System Health -->
      {#if systemHealth}
        <div class="bg-white rounded-xl p-6 border border-gray-100 shadow-sm mb-8">
          <div class="flex items-center justify-between mb-4">
            <h2 class="text-lg font-semibold text-gray-900">System Health</h2>
            <span class="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium {
              $isSystemHealthy ? 'bg-green-100 text-green-700' : 'bg-red-100 text-red-700'
            }">
              {systemHealth.overall === 'healthy' ? '💚 Healthy' :
               systemHealth.overall === 'degraded' ? '💛 Degraded' : '❤️ Unhealthy'}
            </span>
          </div>
          <div class="grid grid-cols-2 md:grid-cols-5 gap-4">
            <div class="text-center p-3 rounded-lg bg-gray-50">
              <div class={getHealthColor(systemHealth.databases.postgres)}>
                {getHealthIcon(systemHealth.databases.postgres)}
              </div>
              <div class="text-sm font-medium mt-1">PostgreSQL</div>
              {#if systemHealth.latencies_ms?.postgres}
                <div class="text-xs text-gray-500">{systemHealth.latencies_ms.postgres}ms</div>
              {/if}
            </div>
            <div class="text-center p-3 rounded-lg bg-gray-50">
              <div class={getHealthColor(systemHealth.databases.redis)}>
                {getHealthIcon(systemHealth.databases.redis)}
              </div>
              <div class="text-sm font-medium mt-1">Redis</div>
              {#if systemHealth.latencies_ms?.redis}
                <div class="text-xs text-gray-500">{systemHealth.latencies_ms.redis}ms</div>
              {/if}
            </div>
            <div class="text-center p-3 rounded-lg bg-gray-50">
              <div class={getHealthColor(systemHealth.databases.duckdb)}>
                {getHealthIcon(systemHealth.databases.duckdb)}
              </div>
              <div class="text-sm font-medium mt-1">DuckDB</div>
              {#if systemHealth.latencies_ms?.duckdb}
                <div class="text-xs text-gray-500">{systemHealth.latencies_ms.duckdb}ms</div>
              {/if}
            </div>
            <div class="text-center p-3 rounded-lg bg-gray-50">
              <div class={getHealthColor(systemHealth.databases.kuzu)}>
                {getHealthIcon(systemHealth.databases.kuzu)}
              </div>
              <div class="text-sm font-medium mt-1">Kuzu</div>
              {#if systemHealth.latencies_ms?.kuzu}
                <div class="text-xs text-gray-500">{systemHealth.latencies_ms.kuzu}ms</div>
              {/if}
            </div>
            <div class="text-center p-3 rounded-lg bg-gray-50">
              <div class={getHealthColor(systemHealth.databases.lancedb)}>
                {getHealthIcon(systemHealth.databases.lancedb)}
              </div>
              <div class="text-sm font-medium mt-1">LanceDB</div>
              {#if systemHealth.latencies_ms?.lancedb}
                <div class="text-xs text-gray-500">{systemHealth.latencies_ms.lancedb}ms</div>
              {/if}
            </div>
          </div>
        </div>
      {/if}

      <!-- Trend Summary -->
      {#if trendSummary}
        <div class="bg-white rounded-xl p-6 border border-gray-100 shadow-sm mb-8">
          <div class="flex items-center justify-between mb-4">
            <h2 class="text-lg font-semibold text-gray-900">Trend Summary</h2>
            <span class={getTrendColor(trendSummary.trend)}>
              {getTrendIcon(trendSummary.trend)} {Math.abs(trendSummary.change_percent).toFixed(1)}%
            </span>
          </div>
          <div class="text-sm text-gray-500 mb-4">Period: {trendSummary.period}</div>
          {#if trendSummary.data_points.length > 0}
            <div class="h-32 flex items-end gap-1">
              {#each trendSummary.data_points as point}
                {@const maxValue = Math.max(...trendSummary.data_points.map(p => p.value))}
                {@const height = maxValue > 0 ? (point.value / maxValue) * 100 : 0}
                <div
                  class="flex-1 bg-indigo-200 hover:bg-indigo-400 transition-colors rounded-t"
                  style="height: {height}%"
                  title="{point.date}: {point.value}"
                ></div>
              {/each}
            </div>
          {/if}
        </div>
      {/if}

      <!-- Artist Trends -->
      <div class="grid md:grid-cols-2 gap-6 mb-8">
        <!-- Rising Artists -->
        <div class="bg-white rounded-xl p-6 border border-gray-100 shadow-sm">
          <h2 class="text-lg font-semibold text-gray-900 mb-4 flex items-center gap-2">
            <span>📈</span> Rising Artists
          </h2>
          {#if $risingArtists.length === 0}
            <p class="text-gray-500 text-sm">No rising artists detected.</p>
          {:else}
            <div class="space-y-3">
              {#each $risingArtists.slice(0, 5) as artist}
                <div class="flex items-center justify-between p-3 bg-green-50 rounded-lg">
                  <div>
                    <div class="font-medium text-gray-900">{artist.artist_name}</div>
                    <div class="text-xs text-gray-500">{artist.mentions} mentions</div>
                  </div>
                  <div class="text-right">
                    <div class="text-green-600 font-medium">+{artist.offense_count}</div>
                    <div class="text-xs text-gray-500">offenses</div>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>

        <!-- Falling Artists -->
        <div class="bg-white rounded-xl p-6 border border-gray-100 shadow-sm">
          <h2 class="text-lg font-semibold text-gray-900 mb-4 flex items-center gap-2">
            <span>📉</span> Falling Artists
          </h2>
          {#if $fallingArtists.length === 0}
            <p class="text-gray-500 text-sm">No falling artists detected.</p>
          {:else}
            <div class="space-y-3">
              {#each $fallingArtists.slice(0, 5) as artist}
                <div class="flex items-center justify-between p-3 bg-red-50 rounded-lg">
                  <div>
                    <div class="font-medium text-gray-900">{artist.artist_name}</div>
                    <div class="text-xs text-gray-500">{artist.mentions} mentions</div>
                  </div>
                  <div class="text-right">
                    <div class="text-red-600 font-medium">{artist.offense_count}</div>
                    <div class="text-xs text-gray-500">offenses</div>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      </div>

      <!-- Report Status -->
      {#if $analyticsStore.currentReport}
        <div class="bg-white rounded-xl p-6 border border-gray-100 shadow-sm">
          <h2 class="text-lg font-semibold text-gray-900 mb-4">Current Report</h2>
          <div class="flex items-center justify-between">
            <div>
              <div class="font-medium">{$analyticsStore.currentReport.id}</div>
              <div class="text-sm text-gray-500">
                Status: <span class="capitalize">{$analyticsStore.currentReport.status}</span>
              </div>
            </div>
            {#if $analyticsStore.currentReport.status === 'processing'}
              <div class="flex items-center gap-2">
                <div class="w-32 h-2 bg-gray-200 rounded-full overflow-hidden">
                  <div
                    class="h-full bg-indigo-600 transition-all"
                    style="width: {$analyticsStore.currentReport.progress_percent ?? 0}%"
                  ></div>
                </div>
                <span class="text-sm text-gray-600">{$analyticsStore.currentReport.progress_percent ?? 0}%</span>
              </div>
            {:else if $analyticsStore.currentReport.status === 'completed' && $analyticsStore.currentReport.download_url}
              <a
                href={$analyticsStore.currentReport.download_url}
                class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 font-medium transition-colors"
              >
                Download
              </a>
            {:else if $analyticsStore.currentReport.status === 'failed'}
              <span class="text-red-600">{$analyticsStore.currentReport.error ?? 'Unknown error'}</span>
            {/if}
          </div>
        </div>
      {/if}
    {/if}
  </div>
</div>

<!-- Report Modal -->
{#if showReportModal}
  <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center p-4 z-50" on:click={closeReportModal} role="dialog" aria-modal="true">
    <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
    <div class="bg-white rounded-2xl max-w-lg w-full p-6 shadow-xl" on:click|stopPropagation role="document">
      <div class="flex items-center mb-6">
        <div class="w-14 h-14 bg-purple-100 rounded-full flex items-center justify-center text-2xl mr-4">
          📊
        </div>
        <div>
          <h3 class="text-xl font-bold text-gray-900">Generate Report</h3>
          <p class="text-gray-600">Configure and generate an analytics report</p>
        </div>
      </div>

      <!-- Report Type -->
      <div class="mb-4">
        <label for="report-type" class="block text-sm font-medium text-gray-700 mb-2">Report Type</label>
        <select
          id="report-type"
          bind:value={reportType}
          class="w-full px-4 py-3 border border-gray-200 rounded-xl focus:border-indigo-500 focus:ring-2 focus:ring-indigo-200"
        >
          {#each $analyticsStore.reportTypes as rt}
            <option value={rt.id}>{rt.name}</option>
          {/each}
          {#if $analyticsStore.reportTypes.length === 0}
            <option value="user_activity">User Activity</option>
            <option value="offense_summary">Offense Summary</option>
            <option value="platform_stats">Platform Statistics</option>
          {/if}
        </select>
      </div>

      <!-- Format -->
      <div class="mb-4">
        <label for="report-format" class="block text-sm font-medium text-gray-700 mb-2">Format</label>
        <select
          id="report-format"
          bind:value={reportFormat}
          class="w-full px-4 py-3 border border-gray-200 rounded-xl focus:border-indigo-500 focus:ring-2 focus:ring-indigo-200"
        >
          <option value="json">JSON</option>
          <option value="csv">CSV</option>
          <option value="parquet">Parquet</option>
          <option value="html">HTML</option>
        </select>
      </div>

      <!-- Include Details -->
      <div class="mb-6">
        <label class="flex items-center gap-2">
          <input
            type="checkbox"
            bind:checked={includeDetails}
            class="w-4 h-4 text-indigo-600 border-gray-300 rounded focus:ring-indigo-500"
          />
          <span class="text-sm text-gray-700">Include detailed breakdown</span>
        </label>
      </div>

      <!-- Actions -->
      <div class="flex gap-3">
        <button
          type="button"
          on:click={closeReportModal}
          class="flex-1 px-4 py-3 border border-gray-200 text-gray-700 rounded-xl hover:bg-gray-50 font-medium transition-colors"
        >
          Cancel
        </button>
        <button
          type="button"
          on:click={handleGenerateReport}
          class="flex-1 px-4 py-3 bg-indigo-600 text-white rounded-xl hover:bg-indigo-700 font-medium transition-colors"
        >
          Generate
        </button>
      </div>
    </div>
  </div>
{/if}
