<script lang="ts">
  import { onMount } from 'svelte';
  import {
    analyticsStore,
    analyticsActions,
    isSystemHealthy,
    risingArtists,
    fallingArtists
  } from '../stores/analytics';
  import type { ReportRequest, TroubleTier } from '../stores/analytics';
  import { syncStore, syncActions, isAnySyncRunning } from '../stores/sync';
  import { navigateTo } from '../utils/simple-router';
  import CategoryRevenueBreakdown from './CategoryRevenueBreakdown.svelte';

  // Tab state
  type Tab = 'overview' | 'revenue' | 'sync';
  let activeTab: Tab = 'overview';

  // Overview state
  let selectedTimeRange = 'last7days';
  let selectedPeriodDays = 7;
  let showReportModal = false;
  let reportType = 'user_activity';
  let reportFormat: 'json' | 'csv' | 'parquet' | 'html' = 'json';
  let includeDetails = true;

  // Revenue state
  let revenueDays = 30;
  let selectedMinTier: TroubleTier = 'moderate';

  // Sync state
  let syncPlatforms: string[] = ['deezer'];
  let syncType: 'full' | 'incremental' = 'incremental';
  let syncPriority: 'low' | 'normal' | 'high' | 'critical' = 'normal';

  const timeRanges = [
    { value: 'last24h', label: 'Last 24 Hours', days: 1 },
    { value: 'last7days', label: 'Last 7 Days', days: 7 },
    { value: 'last30days', label: 'Last 30 Days', days: 30 },
    { value: 'last90days', label: 'Last 90 Days', days: 90 },
  ];

  const periodOptions = [
    { value: 7, label: 'Last 7 Days' },
    { value: 30, label: 'Last 30 Days' },
    { value: 90, label: 'Last 90 Days' },
    { value: 365, label: 'Last Year' },
  ];

  const tierOptions: { value: TroubleTier; label: string }[] = [
    { value: 'moderate', label: 'Moderate+' },
    { value: 'high', label: 'High+' },
    { value: 'critical', label: 'Critical Only' },
  ];

  const platformList = [
    { id: 'deezer', name: 'Deezer', icon: '🎵', alwaysAvailable: true },
    { id: 'spotify', name: 'Spotify', icon: '🎧', alwaysAvailable: false },
    { id: 'apple_music', name: 'Apple Music', icon: '🍎', alwaysAvailable: false },
    { id: 'tidal', name: 'Tidal', icon: '🌊', alwaysAvailable: false },
    { id: 'youtube_music', name: 'YouTube Music', icon: '▶️', alwaysAvailable: false },
  ];

  onMount(async () => {
    // Load overview data
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

  async function loadRevenueData() {
    await Promise.all([
      analyticsActions.fetchRevenueDistribution(revenueDays),
      analyticsActions.fetchProblematicArtistRevenue(revenueDays, selectedMinTier, 10),
      analyticsActions.fetchTierDistribution(),
      analyticsActions.fetchPayoutRates(),
    ]);
  }

  async function loadSyncData() {
    await Promise.all([
      syncActions.fetchStatus(),
      syncActions.fetchRuns(),
      syncActions.fetchHealth(),
    ]);
  }

  async function handleTabChange(tab: Tab) {
    activeTab = tab;
    if (tab === 'revenue') {
      await loadRevenueData();
    } else if (tab === 'sync') {
      await loadSyncData();
    }
  }

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

  async function handleRevenuePeriodChange() {
    await analyticsActions.fetchRevenueDistribution(revenueDays);
    await analyticsActions.fetchProblematicArtistRevenue(revenueDays, selectedMinTier, 10);
  }

  async function handleTierChange() {
    await analyticsActions.fetchProblematicArtistRevenue(revenueDays, selectedMinTier, 10);
  }

  async function handleTriggerSync() {
    await syncActions.triggerSync({
      platforms: syncPlatforms,
      sync_type: syncType,
      priority: syncPriority,
    });
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

  function formatCurrency(value: string | undefined): string {
    if (!value) return '$0.00';
    const num = parseFloat(value);
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 2,
      maximumFractionDigits: 4,
    }).format(num);
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
      default: return 'text-zinc-300';
    }
  }

  function getHealthIcon(healthy: boolean): string {
    return healthy ? '✅' : '❌';
  }

  function getHealthColor(healthy: boolean): string {
    return healthy ? 'text-green-600' : 'text-red-600';
  }

  function getTierColor(tier: TroubleTier | undefined): string {
    switch (tier) {
      case 'critical': return 'bg-red-600';
      case 'high': return 'bg-orange-500';
      case 'moderate': return 'bg-yellow-500';
      case 'low': return 'bg-green-500';
      default: return 'bg-zinc-400';
    }
  }

  function getTierBgColor(tier: TroubleTier | undefined): string {
    switch (tier) {
      case 'critical': return 'bg-red-900/30 border-red-700';
      case 'high': return 'bg-orange-900/30 border-orange-700';
      case 'moderate': return 'bg-yellow-900/30 border-yellow-700';
      case 'low': return 'bg-green-900/30 border-green-700';
      default: return 'bg-zinc-800 border-zinc-600';
    }
  }

  function getTierLabel(tier: TroubleTier | undefined): string {
    if (!tier) return 'Unknown';
    return tier.charAt(0).toUpperCase() + tier.slice(1);
  }

  function getSyncStatusIcon(status: string): string {
    switch (status) {
      case 'running': return '🔄';
      case 'completed': return '✅';
      case 'error': case 'failed': return '❌';
      default: return '⏸️';
    }
  }

  function togglePlatform(platformId: string) {
    if (syncPlatforms.includes(platformId)) {
      syncPlatforms = syncPlatforms.filter(p => p !== platformId);
    } else {
      syncPlatforms = [...syncPlatforms, platformId];
    }
  }

  $: dashboard = $analyticsStore.dashboard;
  $: userStats = $analyticsStore.userStats;
  $: systemHealth = $analyticsStore.systemHealth;
  $: trendSummary = $analyticsStore.trends.summary;
  $: distribution = $analyticsStore.revenueDistribution;
  $: problematicArtists = $analyticsStore.problematicArtistRevenue;
  $: tierDist = $analyticsStore.tierDistribution;
  $: rates = $analyticsStore.payoutRates;
</script>

<div class="min-h-screen py-6">
  <!-- Header -->
  <div class="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8 mb-8">
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-3xl font-bold text-white mb-2">
          Analytics Dashboard
        </h1>
        <p class="text-lg text-zinc-400">
          Monitor system health, revenue impact, and catalog sync.
        </p>
      </div>
      <div>
        {#if activeTab === 'overview'}
          <div class="flex items-center gap-4">
            <select
              bind:value={selectedTimeRange}
              on:change={handleTimeRangeChange}
              class="px-4 py-2 rounded-lg text-white bg-zinc-700 border border-zinc-600"
            >
              {#each timeRanges as range}
                <option value={range.value} style="background: #1a1a2e;">{range.label}</option>
              {/each}
            </select>
            <button
              type="button"
              on:click={openReportModal}
              class="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 font-medium transition-colors flex items-center gap-2"
            >
              <span>📊</span> Generate Report
            </button>
          </div>
        {:else if activeTab === 'revenue'}
          <select
            bind:value={revenueDays}
            on:change={handleRevenuePeriodChange}
            class="px-4 py-2 rounded-lg text-white bg-zinc-700 border border-zinc-600"
          >
            {#each periodOptions as option}
              <option value={option.value} style="background: #1a1a2e;">{option.label}</option>
            {/each}
          </select>
        {/if}
      </div>

      <!-- Tab Navigation -->
      <div class="flex gap-1 mt-6">
        <button
          type="button"
          on:click={() => handleTabChange('overview')}
          class="px-4 py-2 rounded-full text-sm font-medium transition-all"
          style={activeTab === 'overview' ? 'background: #3B82F6; color: white;' : 'color: #9CA3AF;'}
        >
          📊 Overview
        </button>
        <button
          type="button"
          on:click={() => handleTabChange('revenue')}
          class="px-4 py-2 rounded-full text-sm font-medium transition-all"
          style={activeTab === 'revenue' ? 'background: #3B82F6; color: white;' : 'color: #9CA3AF;'}
        >
          💰 Revenue
        </button>
        <button
          type="button"
          on:click={() => handleTabChange('sync')}
          class="px-4 py-2 rounded-full text-sm font-medium transition-all flex items-center gap-2"
          style={activeTab === 'sync' ? 'background: #3B82F6; color: white;' : 'color: #9CA3AF;'}
        >
          🔄 Sync
          {#if $isAnySyncRunning}
            <span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-green-500/20 text-green-400">
              Running
            </span>
          {/if}
        </button>
      </div>
    </div>
  </div>

  <div class="max-w-6xl mx-auto px-4 py-6 sm:px-6 lg:px-8">
    <!-- Error display -->
    {#if $analyticsStore.error}
      <div class="rounded-xl p-4 mb-6" style="background: rgba(239, 68, 68, 0.15); border: 1px solid rgba(239, 68, 68, 0.3);">
        <div class="flex items-center gap-2 text-red-400">
          <span>❌</span>
          <span>{$analyticsStore.error}</span>
          <button type="button" on:click={analyticsActions.clearError} class="ml-auto text-red-400 hover:text-red-300">
            Dismiss
          </button>
        </div>
      </div>
    {/if}

    {#if $syncStore.error && activeTab === 'sync'}
      <div class="rounded-xl p-4 mb-6" style="background: rgba(239, 68, 68, 0.15); border: 1px solid rgba(239, 68, 68, 0.3);">
        <div class="flex items-center gap-2 text-red-400">
          <span>❌</span>
          <span>{$syncStore.error}</span>
          <button type="button" on:click={syncActions.clearError} class="ml-auto text-red-400 hover:text-red-300">
            Dismiss
          </button>
        </div>
      </div>
    {/if}

    <!-- Loading state -->
    {#if $analyticsStore.isLoading || $syncStore.isLoading}
      <div class="flex justify-center py-12">
        <div class="w-8 h-8 border-4 border-blue-500 border-t-transparent rounded-full animate-spin"></div>
      </div>
    {:else}
      <!-- ==================== OVERVIEW TAB ==================== -->
      {#if activeTab === 'overview'}
        <!-- Quick Stats Cards -->
        <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mb-8">
          <div class="rounded-xl p-5 bg-zinc-800 border border-zinc-600">
            <div class="flex items-center gap-3">
              <div class="w-12 h-12 rounded-full flex items-center justify-center text-xl bg-blue-900/50">
                👥
              </div>
              <div>
                <div class="text-2xl font-bold text-white">{formatNumber(dashboard?.total_users)}</div>
                <div class="text-sm text-zinc-400">Total Users</div>
              </div>
            </div>
          </div>

          <div class="rounded-xl p-5 bg-zinc-800 border border-zinc-600">
            <div class="flex items-center gap-3">
              <div class="w-12 h-12 rounded-full flex items-center justify-center text-xl bg-green-900/50">
                🟢
              </div>
              <div>
                <div class="text-2xl font-bold text-white">{formatNumber(dashboard?.active_users_today)}</div>
                <div class="text-sm text-zinc-400">Active Today</div>
              </div>
            </div>
          </div>

          <div class="rounded-xl p-5 bg-zinc-800 border border-zinc-600">
            <div class="flex items-center gap-3">
              <div class="w-12 h-12 rounded-full flex items-center justify-center text-xl bg-red-900/50">
                🚫
              </div>
              <div>
                <div class="text-2xl font-bold text-white">{formatNumber(dashboard?.total_blocked_artists)}</div>
                <div class="text-sm text-zinc-400">Blocked Artists</div>
              </div>
            </div>
          </div>

          <div class="rounded-xl p-5 bg-zinc-800 border border-zinc-600">
            <div class="flex items-center gap-3">
              <div class="w-12 h-12 rounded-full flex items-center justify-center text-xl bg-purple-900/50">
                🔔
              </div>
              <div>
                <div class="text-2xl font-bold text-white">{formatNumber(dashboard?.offense_detections_today)}</div>
                <div class="text-sm text-zinc-400">Offenses Today</div>
              </div>
            </div>
          </div>
        </div>

        <!-- User Quick Stats -->
        {#if userStats}
          <div class="bg-zinc-800 rounded-xl p-6 border border-zinc-600 shadow-sm mb-8">
            <h2 class="text-lg font-semibold text-white mb-4">Your Quick Stats</h2>
            <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
              <div class="text-center">
                <div class="text-2xl font-bold text-indigo-400">{userStats.blocked_artists}</div>
                <div class="text-sm text-zinc-400">Blocked Artists</div>
              </div>
              <div class="text-center">
                <div class="text-2xl font-bold text-indigo-400">{userStats.subscriptions}</div>
                <div class="text-sm text-zinc-400">Subscriptions</div>
              </div>
              <div class="text-center">
                <div class="text-2xl font-bold text-indigo-400">{userStats.manual_blocks}</div>
                <div class="text-sm text-zinc-400">Manual Blocks</div>
              </div>
              <div class="text-center">
                <div class="text-sm font-medium text-zinc-300">{userStats.last_sync ?? 'Never'}</div>
                <div class="text-sm text-zinc-400">Last Sync</div>
              </div>
            </div>
          </div>
        {/if}

        <!-- System Health -->
        {#if systemHealth}
          <div class="bg-zinc-800 rounded-xl p-6 border border-zinc-600 shadow-sm mb-8">
            <div class="flex items-center justify-between mb-4">
              <h2 class="text-lg font-semibold text-white">System Health</h2>
              <span class="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium {
                $isSystemHealthy ? 'bg-green-900/50 text-green-400' : 'bg-red-900/50 text-red-400'
              }">
                {systemHealth.overall === 'healthy' ? '💚 Healthy' :
                 systemHealth.overall === 'degraded' ? '💛 Degraded' : '❤️ Unhealthy'}
              </span>
            </div>
            <div class="grid grid-cols-2 md:grid-cols-5 gap-4">
              <div class="text-center p-3 rounded-lg bg-zinc-700">
                <div class={getHealthColor(systemHealth.databases.postgres)}>
                  {getHealthIcon(systemHealth.databases.postgres)}
                </div>
                <div class="text-sm font-medium mt-1 text-white">PostgreSQL</div>
                {#if systemHealth.latencies_ms?.postgres}
                  <div class="text-xs text-zinc-400">{systemHealth.latencies_ms.postgres}ms</div>
                {/if}
              </div>
              <div class="text-center p-3 rounded-lg bg-zinc-700">
                <div class={getHealthColor(systemHealth.databases.redis)}>
                  {getHealthIcon(systemHealth.databases.redis)}
                </div>
                <div class="text-sm font-medium mt-1 text-white">Redis</div>
                {#if systemHealth.latencies_ms?.redis}
                  <div class="text-xs text-zinc-400">{systemHealth.latencies_ms.redis}ms</div>
                {/if}
              </div>
              <div class="text-center p-3 rounded-lg bg-zinc-700">
                <div class={getHealthColor(systemHealth.databases.duckdb)}>
                  {getHealthIcon(systemHealth.databases.duckdb)}
                </div>
                <div class="text-sm font-medium mt-1 text-white">DuckDB</div>
                {#if systemHealth.latencies_ms?.duckdb}
                  <div class="text-xs text-zinc-400">{systemHealth.latencies_ms.duckdb}ms</div>
                {/if}
              </div>
              <div class="text-center p-3 rounded-lg bg-zinc-700">
                <div class={getHealthColor(systemHealth.databases.kuzu)}>
                  {getHealthIcon(systemHealth.databases.kuzu)}
                </div>
                <div class="text-sm font-medium mt-1 text-white">Kuzu</div>
                {#if systemHealth.latencies_ms?.kuzu}
                  <div class="text-xs text-zinc-400">{systemHealth.latencies_ms.kuzu}ms</div>
                {/if}
              </div>
              <div class="text-center p-3 rounded-lg bg-zinc-700">
                <div class={getHealthColor(systemHealth.databases.lancedb)}>
                  {getHealthIcon(systemHealth.databases.lancedb)}
                </div>
                <div class="text-sm font-medium mt-1 text-white">LanceDB</div>
                {#if systemHealth.latencies_ms?.lancedb}
                  <div class="text-xs text-zinc-400">{systemHealth.latencies_ms.lancedb}ms</div>
                {/if}
              </div>
            </div>
          </div>
        {/if}

        <!-- Trend Summary -->
        {#if trendSummary}
          <div class="bg-zinc-800 rounded-xl p-6 border border-zinc-600 shadow-sm mb-8">
            <div class="flex items-center justify-between mb-4">
              <h2 class="text-lg font-semibold text-white">Trend Summary</h2>
              <span class={getTrendColor(trendSummary.trend)}>
                {getTrendIcon(trendSummary.trend)} {Math.abs(trendSummary.change_percent).toFixed(1)}%
              </span>
            </div>
            <div class="text-sm text-zinc-400 mb-4">Period: {trendSummary.period}</div>
            {#if trendSummary.data_points.length > 0}
              <div class="h-32 flex items-end gap-1">
                {#each trendSummary.data_points as point}
                  {@const maxValue = Math.max(...trendSummary.data_points.map(p => p.value))}
                  {@const height = maxValue > 0 ? (point.value / maxValue) * 100 : 0}
                  <div
                    class="flex-1 bg-indigo-600 hover:bg-indigo-500 transition-colors rounded-t"
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
          <div class="bg-zinc-800 rounded-xl p-6 border border-zinc-600 shadow-sm">
            <h2 class="text-lg font-semibold text-white mb-4 flex items-center gap-2">
              <span>📈</span> Rising Artists
            </h2>
            {#if $risingArtists.length === 0}
              <p class="text-zinc-400 text-sm">No rising artists detected.</p>
            {:else}
              <div class="space-y-3">
                {#each $risingArtists.slice(0, 5) as artist}
                  <div class="flex items-center justify-between p-3 bg-green-900/30 rounded-lg border border-green-700">
                    <div>
                      <div class="font-medium text-white">{artist.artist_name}</div>
                      <div class="text-xs text-zinc-400">{artist.mentions} mentions</div>
                    </div>
                    <div class="text-right">
                      <div class="text-green-400 font-medium">+{artist.offense_count}</div>
                      <div class="text-xs text-zinc-400">offenses</div>
                    </div>
                  </div>
                {/each}
              </div>
            {/if}
          </div>

          <!-- Falling Artists -->
          <div class="bg-zinc-800 rounded-xl p-6 border border-zinc-600 shadow-sm">
            <h2 class="text-lg font-semibold text-white mb-4 flex items-center gap-2">
              <span>📉</span> Falling Artists
            </h2>
            {#if $fallingArtists.length === 0}
              <p class="text-zinc-400 text-sm">No falling artists detected.</p>
            {:else}
              <div class="space-y-3">
                {#each $fallingArtists.slice(0, 5) as artist}
                  <div class="flex items-center justify-between p-3 bg-red-900/30 rounded-lg border border-red-700">
                    <div>
                      <div class="font-medium text-white">{artist.artist_name}</div>
                      <div class="text-xs text-zinc-400">{artist.mentions} mentions</div>
                    </div>
                    <div class="text-right">
                      <div class="text-red-400 font-medium">{artist.offense_count}</div>
                      <div class="text-xs text-zinc-400">offenses</div>
                    </div>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        </div>

        <!-- Report Status -->
        {#if $analyticsStore.currentReport}
          <div class="bg-zinc-800 rounded-xl p-6 border border-zinc-600 shadow-sm">
            <h2 class="text-lg font-semibold text-white mb-4">Current Report</h2>
            <div class="flex items-center justify-between">
              <div>
                <div class="font-medium text-white">{$analyticsStore.currentReport.id}</div>
                <div class="text-sm text-zinc-400">
                  Status: <span class="capitalize">{$analyticsStore.currentReport.status}</span>
                </div>
              </div>
              {#if $analyticsStore.currentReport.status === 'processing'}
                <div class="flex items-center gap-2">
                  <div class="w-32 h-2 bg-zinc-700 rounded-full overflow-hidden">
                    <div
                      class="h-full bg-indigo-600 transition-all"
                      style="width: {$analyticsStore.currentReport.progress_percent ?? 0}%"
                    ></div>
                  </div>
                  <span class="text-sm text-zinc-300">{$analyticsStore.currentReport.progress_percent ?? 0}%</span>
                </div>
              {:else if $analyticsStore.currentReport.status === 'completed' && $analyticsStore.currentReport.download_url}
                <a
                  href={$analyticsStore.currentReport.download_url}
                  class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 font-medium transition-colors"
                >
                  Download
                </a>
              {:else if $analyticsStore.currentReport.status === 'failed'}
                <span class="text-red-400">{$analyticsStore.currentReport.error ?? 'Unknown error'}</span>
              {/if}
            </div>
          </div>
        {/if}

      <!-- ==================== REVENUE IMPACT TAB ==================== -->
      {:else if activeTab === 'revenue'}
        <!-- Summary Cards -->
        <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mb-8">
          <div class="bg-zinc-800 rounded-xl p-5 border border-zinc-600 shadow-sm">
            <div class="flex items-center gap-3">
              <div class="w-12 h-12 bg-blue-900/50 rounded-full flex items-center justify-center text-xl">
                🎧
              </div>
              <div>
                <div class="text-2xl font-bold text-white">
                  {formatNumber(distribution?.total_streams ?? 0)}
                </div>
                <div class="text-sm text-zinc-400">Total Streams</div>
              </div>
            </div>
          </div>

          <div class="bg-zinc-800 rounded-xl p-5 border border-zinc-600 shadow-sm">
            <div class="flex items-center gap-3">
              <div class="w-12 h-12 bg-green-900/50 rounded-full flex items-center justify-center text-xl">
                💰
              </div>
              <div>
                <div class="text-2xl font-bold text-white">
                  {formatCurrency(distribution?.total_revenue)}
                </div>
                <div class="text-sm text-zinc-400">Total Revenue</div>
              </div>
            </div>
          </div>

          <div class="bg-zinc-800 rounded-xl p-5 border border-zinc-600 shadow-sm">
            <div class="flex items-center gap-3">
              <div class="w-12 h-12 bg-red-900/50 rounded-full flex items-center justify-center text-xl">
                ⚠️
              </div>
              <div>
                <div class="text-2xl font-bold text-red-400">
                  {formatCurrency(distribution?.revenue_to_problematic_artists)}
                </div>
                <div class="text-sm text-zinc-400">To Problematic Artists</div>
              </div>
            </div>
          </div>

          <div class="bg-zinc-800 rounded-xl p-5 border border-zinc-600 shadow-sm">
            <div class="flex items-center gap-3">
              <div class="w-12 h-12 bg-purple-900/50 rounded-full flex items-center justify-center text-xl">
                📊
              </div>
              <div>
                <div class="text-2xl font-bold text-purple-400">
                  {(distribution?.problematic_percentage ?? 0).toFixed(1)}%
                </div>
                <div class="text-sm text-zinc-400">Problematic Share</div>
              </div>
            </div>
          </div>
        </div>

        <!-- Revenue Distribution Chart -->
        <div class="bg-zinc-800 rounded-xl p-6 border border-zinc-600 shadow-sm mb-8">
          <h2 class="text-lg font-semibold text-white mb-4">Revenue Distribution</h2>
          <div class="flex items-center gap-6">
            <!-- Pie chart visual representation -->
            <div class="relative w-40 h-40">
              <svg class="w-40 h-40 transform -rotate-90" viewBox="0 0 100 100">
                <!-- Clean portion (green) -->
                <circle
                  cx="50" cy="50" r="40"
                  fill="none"
                  stroke="#22c55e"
                  stroke-width="20"
                  stroke-dasharray="{(100 - (distribution?.problematic_percentage ?? 0)) * 2.51} 251"
                  stroke-dashoffset="0"
                />
                <!-- Problematic portion (red) -->
                <circle
                  cx="50" cy="50" r="40"
                  fill="none"
                  stroke="#ef4444"
                  stroke-width="20"
                  stroke-dasharray="{(distribution?.problematic_percentage ?? 0) * 2.51} 251"
                  stroke-dashoffset="{-((100 - (distribution?.problematic_percentage ?? 0)) * 2.51)}"
                />
              </svg>
            </div>
            <div class="flex-1 space-y-4">
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-2">
                  <div class="w-4 h-4 rounded bg-green-500"></div>
                  <span class="text-zinc-300">Clean Artists</span>
                </div>
                <div class="text-right">
                  <div class="font-semibold text-white">{formatCurrency(distribution?.revenue_to_clean_artists)}</div>
                  <div class="text-sm text-zinc-400">{(100 - (distribution?.problematic_percentage ?? 0)).toFixed(1)}%</div>
                </div>
              </div>
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-2">
                  <div class="w-4 h-4 rounded bg-red-500"></div>
                  <span class="text-zinc-300">Problematic Artists</span>
                </div>
                <div class="text-right">
                  <div class="font-semibold text-red-400">{formatCurrency(distribution?.revenue_to_problematic_artists)}</div>
                  <div class="text-sm text-zinc-400">{(distribution?.problematic_percentage ?? 0).toFixed(1)}%</div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Tier Distribution -->
        {#if tierDist}
          <div class="bg-zinc-800 rounded-xl p-6 border border-zinc-600 shadow-sm mb-8">
            <h2 class="text-lg font-semibold text-white mb-4">Artists by Trouble Tier</h2>
            <div class="grid grid-cols-4 gap-4">
              <div class="text-center p-4 bg-green-900/30 rounded-lg border border-green-700">
                <div class="text-2xl font-bold text-green-400">{tierDist.low}</div>
                <div class="text-sm text-green-500">Low</div>
              </div>
              <div class="text-center p-4 bg-yellow-900/30 rounded-lg border border-yellow-700">
                <div class="text-2xl font-bold text-yellow-400">{tierDist.moderate}</div>
                <div class="text-sm text-yellow-500">Moderate</div>
              </div>
              <div class="text-center p-4 bg-orange-900/30 rounded-lg border border-orange-700">
                <div class="text-2xl font-bold text-orange-400">{tierDist.high}</div>
                <div class="text-sm text-orange-500">High</div>
              </div>
              <div class="text-center p-4 bg-red-900/30 rounded-lg border border-red-700">
                <div class="text-2xl font-bold text-red-400">{tierDist.critical}</div>
                <div class="text-sm text-red-500">Critical</div>
              </div>
            </div>
          </div>
        {/if}

        <!-- Top Problematic Artists -->
        <div class="bg-zinc-800 rounded-xl p-6 border border-zinc-600 shadow-sm mb-8">
          <div class="flex items-center justify-between mb-4">
            <h2 class="text-lg font-semibold text-white">Top Problematic Artists by Your Revenue</h2>
            <select
              bind:value={selectedMinTier}
              on:change={handleTierChange}
              class="px-3 py-1 text-sm border border-zinc-600 rounded-lg focus:border-indigo-500 bg-zinc-700 text-white"
            >
              {#each tierOptions as option}
                <option value={option.value}>{option.label}</option>
              {/each}
            </select>
          </div>

          {#if problematicArtists.length === 0}
            <div class="text-center py-8 text-zinc-400">
              <p>No problematic artists found in your listening history.</p>
              <p class="text-sm mt-2">This is great news! Your streaming revenue is going to clean artists.</p>
            </div>
          {:else}
            <div class="space-y-3">
              {#each problematicArtists as artist, index}
                <div class="flex items-center justify-between p-4 rounded-lg border {getTierBgColor(artist.trouble_tier)}">
                  <div class="flex items-center gap-4">
                    <div class="w-8 h-8 rounded-full bg-zinc-600 flex items-center justify-center font-bold text-zinc-300">
                      {index + 1}
                    </div>
                    <div>
                      <div class="font-medium text-white">{artist.artist_name}</div>
                      <div class="text-xs text-zinc-400 flex items-center gap-2">
                        <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium {getTierColor(artist.trouble_tier)} text-white">
                          {getTierLabel(artist.trouble_tier)}
                        </span>
                        <span>Score: {(artist.trouble_score ?? 0).toFixed(2)}</span>
                      </div>
                    </div>
                  </div>
                  <div class="text-right">
                    <div class="font-semibold text-white">{formatCurrency(artist.total_revenue)}</div>
                    <div class="text-xs text-zinc-400">
                      {formatNumber(artist.total_streams)} streams ({artist.percentage_of_user_spend.toFixed(1)}%)
                    </div>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>

        <!-- Platform Payout Rates -->
        <div class="bg-zinc-800 rounded-xl p-6 border border-zinc-600 shadow-sm">
          <h2 class="text-lg font-semibold text-white mb-4">Platform Payout Rates</h2>
          <p class="text-sm text-zinc-400 mb-4">
            Average payout per stream by platform. Actual rates vary based on subscription tier, region, and other factors.
          </p>
          <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-4">
            {#each rates as rate}
              <div class="text-center p-4 bg-zinc-700 rounded-lg">
                <div class="text-lg font-bold text-indigo-400">
                  {formatCurrency(rate.rate_per_stream)}
                </div>
                <div class="text-sm font-medium text-zinc-300 capitalize">{rate.platform.replace('_', ' ')}</div>
                <div class="text-xs text-zinc-400">{rate.rate_tier}</div>
              </div>
            {/each}
          </div>
        </div>

        <!-- Category Revenue Breakdown (Simulated) -->
        <div class="mt-8">
          <CategoryRevenueBreakdown
            showDetails={true}
            maxCategories={12}
            onArtistClick={(artistId) => navigateTo(`/artist/${artistId}`)}
          />
        </div>

      <!-- ==================== CATALOG SYNC TAB ==================== -->
      {:else if activeTab === 'sync'}
        <!-- Platform Status Overview -->
        <div class="bg-zinc-800 rounded-xl p-6 border border-zinc-600 shadow-sm mb-8">
          <h2 class="text-lg font-semibold text-white mb-4">Platform Status</h2>
          <div class="grid grid-cols-2 md:grid-cols-5 gap-4">
            {#each platformList as platform}
              {@const status = $syncStore.status.find(s => s.platform === platform.id)}
              <div class="text-center p-4 rounded-lg border {
                status?.status === 'running' ? 'bg-blue-900/30 border-blue-700' :
                status?.status === 'completed' ? 'bg-green-900/30 border-green-700' :
                status?.status === 'error' ? 'bg-red-900/30 border-red-700' :
                'bg-zinc-700 border-zinc-600'
              }">
                <div class="text-2xl mb-2">{platform.icon}</div>
                <div class="font-medium text-white">{platform.name}</div>
                <div class="text-xs text-zinc-400 mt-1">
                  {#if status?.status === 'running'}
                    <span class="text-blue-400">🔄 Syncing...</span>
                  {:else if status?.artists_count}
                    {formatNumber(status.artists_count)} artists
                  {:else if platform.alwaysAvailable}
                    <span class="text-green-400">✅ Ready</span>
                  {:else}
                    <span class="text-zinc-400">No credentials</span>
                  {/if}
                </div>
                {#if status?.last_sync}
                  <div class="text-xs text-zinc-400 mt-1">
                    Last: {new Date(status.last_sync).toLocaleDateString()}
                  </div>
                {/if}
              </div>
            {/each}
          </div>
        </div>

        <!-- Trigger Sync Controls -->
        <div class="bg-zinc-800 rounded-xl p-6 border border-zinc-600 shadow-sm mb-8">
          <h2 class="text-lg font-semibold text-white mb-4">Trigger Catalog Sync</h2>
          <p class="text-sm text-zinc-400 mb-4">
            Sync artist catalogs from streaming platforms. Deezer is always available (no API key required).
            Add credentials to .env to enable other platforms.
          </p>

          <!-- Platform Selection -->
          <div class="mb-4">
            <label class="block text-sm font-medium text-zinc-300 mb-2">Select Platforms</label>
            <div class="flex flex-wrap gap-2">
              {#each platformList as platform}
                <button
                  type="button"
                  on:click={() => togglePlatform(platform.id)}
                  disabled={!platform.alwaysAvailable && !$syncStore.status.find(s => s.platform === platform.id)}
                  class="px-4 py-2 rounded-lg border text-sm font-medium transition-colors {
                    syncPlatforms.includes(platform.id)
                      ? 'bg-indigo-900/50 border-indigo-600 text-indigo-300'
                      : 'bg-zinc-700 border-zinc-600 text-zinc-400 hover:bg-zinc-600'
                  } {
                    !platform.alwaysAvailable && !$syncStore.status.find(s => s.platform === platform.id)
                      ? 'opacity-50 cursor-not-allowed'
                      : ''
                  }"
                >
                  {platform.icon} {platform.name}
                </button>
              {/each}
            </div>
          </div>

          <!-- Sync Type -->
          <div class="grid grid-cols-2 gap-4 mb-4">
            <div>
              <label for="sync-type" class="block text-sm font-medium text-zinc-300 mb-2">Sync Type</label>
              <select
                id="sync-type"
                bind:value={syncType}
                class="w-full px-4 py-2 border border-zinc-600 rounded-lg focus:border-indigo-500 focus:ring-2 focus:ring-indigo-900 bg-zinc-700 text-white"
              >
                <option value="incremental">Incremental (new artists only)</option>
                <option value="full">Full (resync all artists)</option>
              </select>
            </div>
            <div>
              <label for="sync-priority" class="block text-sm font-medium text-zinc-300 mb-2">Priority</label>
              <select
                id="sync-priority"
                bind:value={syncPriority}
                class="w-full px-4 py-2 border border-zinc-600 rounded-lg focus:border-indigo-500 focus:ring-2 focus:ring-indigo-900 bg-zinc-700 text-white"
              >
                <option value="low">Low</option>
                <option value="normal">Normal</option>
                <option value="high">High</option>
                <option value="critical">Critical</option>
              </select>
            </div>
          </div>

          <!-- Trigger Button -->
          <button
            type="button"
            on:click={handleTriggerSync}
            disabled={syncPlatforms.length === 0 || $syncStore.isTriggering || $isAnySyncRunning}
            class="w-full px-6 py-3 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
          >
            {#if $syncStore.isTriggering}
              <div class="w-5 h-5 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
              Triggering...
            {:else if $isAnySyncRunning}
              🔄 Sync in Progress...
            {:else}
              🚀 Start Catalog Sync
            {/if}
          </button>
        </div>

        <!-- Recent Sync Runs -->
        <div class="bg-zinc-800 rounded-xl p-6 border border-zinc-600 shadow-sm">
          <h2 class="text-lg font-semibold text-white mb-4">Recent Sync Runs</h2>
          {#if $syncStore.runs.length === 0}
            <div class="text-center py-8 text-zinc-400">
              <p>No sync runs yet. Trigger a sync to populate artist catalogs.</p>
            </div>
          {:else}
            <div class="space-y-3">
              {#each $syncStore.runs as run}
                <div class="flex items-center justify-between p-4 rounded-lg bg-zinc-700 border border-zinc-600">
                  <div class="flex items-center gap-4">
                    <div class="text-2xl">{getSyncStatusIcon(run.status)}</div>
                    <div>
                      <div class="font-medium text-white capitalize">{run.platform}</div>
                      <div class="text-xs text-zinc-400">
                        {run.sync_type} sync • {new Date(run.started_at).toLocaleString()}
                      </div>
                    </div>
                  </div>
                  <div class="text-right">
                    <div class="font-semibold text-white">{formatNumber(run.artists_processed)} artists</div>
                    <div class="text-xs text-zinc-400">
                      {#if run.status === 'running'}
                        <span class="text-blue-400">In progress...</span>
                      {:else if run.status === 'completed'}
                        <span class="text-green-400">Completed</span>
                      {:else if run.status === 'failed'}
                        <span class="text-red-400">{run.errors_count} errors</span>
                      {:else}
                        <span class="capitalize">{run.status}</span>
                      {/if}
                    </div>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {/if}
    {/if}
  </div>
</div>

<!-- Report Modal -->
{#if showReportModal}
  <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
  <div class="fixed inset-0 bg-black/70 flex items-center justify-center p-4 z-50" on:click={closeReportModal} role="dialog" aria-modal="true">
    <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
    <div class="bg-zinc-800 rounded-2xl max-w-lg w-full p-6 shadow-xl border border-zinc-600" on:click|stopPropagation role="document">
      <div class="flex items-center mb-6">
        <div class="w-14 h-14 bg-purple-900/50 rounded-full flex items-center justify-center text-2xl mr-4">
          📊
        </div>
        <div>
          <h3 class="text-xl font-bold text-white">Generate Report</h3>
          <p class="text-zinc-400">Configure and generate an analytics report</p>
        </div>
      </div>

      <!-- Report Type -->
      <div class="mb-4">
        <label for="report-type" class="block text-sm font-medium text-zinc-300 mb-2">Report Type</label>
        <select
          id="report-type"
          bind:value={reportType}
          class="w-full px-4 py-3 border border-zinc-600 rounded-xl focus:border-indigo-500 focus:ring-2 focus:ring-indigo-900 bg-zinc-700 text-white"
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
        <label for="report-format" class="block text-sm font-medium text-zinc-300 mb-2">Format</label>
        <select
          id="report-format"
          bind:value={reportFormat}
          class="w-full px-4 py-3 border border-zinc-600 rounded-xl focus:border-indigo-500 focus:ring-2 focus:ring-indigo-900 bg-zinc-700 text-white"
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
            class="w-4 h-4 text-indigo-600 border-zinc-600 rounded focus:ring-indigo-500 bg-zinc-700"
          />
          <span class="text-sm text-zinc-300">Include detailed breakdown</span>
        </label>
      </div>

      <!-- Actions -->
      <div class="flex gap-3">
        <button
          type="button"
          on:click={closeReportModal}
          class="flex-1 px-4 py-3 border border-zinc-600 text-zinc-300 rounded-xl hover:bg-zinc-700 font-medium transition-colors"
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
