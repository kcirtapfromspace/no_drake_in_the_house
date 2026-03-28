<script lang="ts">
  import { onMount } from 'svelte';
  import {
    analyticsStore,
    analyticsActions,
    risingArtists,
    fallingArtists
  } from '../stores/analytics';
  import type { ReportRequest, TroubleTier } from '../stores/analytics';
  import { syncStore, syncActions, isAnySyncRunning } from '../stores/sync';
  import { navigateToArtist } from '../utils/simple-router';
  import CategoryRevenueBreakdown from './CategoryRevenueBreakdown.svelte';
  import { Skeleton } from './ui';
  import { timeAgo } from '../utils/time-ago';

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
    { id: 'deezer', name: 'Deezer', abbr: 'DZ', alwaysAvailable: true },
    { id: 'spotify', name: 'Spotify', abbr: 'SP', alwaysAvailable: false },
    { id: 'apple_music', name: 'Apple Music', abbr: 'AM', alwaysAvailable: false },
    { id: 'tidal', name: 'Tidal', abbr: 'TI', alwaysAvailable: false },
    { id: 'youtube_music', name: 'YouTube Music', abbr: 'YT', alwaysAvailable: false },
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

  function formatNumber(num: number | undefined | null): string {
    if (typeof num !== 'number' || !Number.isFinite(num)) return '0';
    if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`;
    if (num >= 1000) return `${(num / 1000).toFixed(1)}K`;
    return num.toString();
  }

  function formatCurrency(value: string | undefined): string {
    if (!value) return '$0.00';
    const num = parseFloat(value);
    if (!Number.isFinite(num)) return '$0.00';
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 2,
      maximumFractionDigits: 4,
    }).format(num);
  }

  function formatTrendPercent(value: number | undefined): string {
    if (typeof value !== 'number' || !Number.isFinite(value)) return '0.0';
    return Math.abs(value).toFixed(1);
  }

  function getTrendBarHeight(
    value: number | undefined,
    points: Array<{ value: number }>
  ): number {
    const finiteValues = points
      .map((point) => point.value)
      .filter((point) => Number.isFinite(point));
    const maxValue = finiteValues.length > 0 ? Math.max(...finiteValues) : 0;

    if (!Number.isFinite(value) || maxValue <= 0) {
      return 0;
    }

    return (value! / maxValue) * 100;
  }

  function getTrendIcon(trend: 'up' | 'down' | 'stable' | 'rising' | 'falling'): string {
    switch (trend) {
      case 'up': case 'rising': return '\u2197';
      case 'down': case 'falling': return '\u2198';
      default: return '\u2192';
    }
  }

  function getTrendColor(trend: 'up' | 'down' | 'stable' | 'rising' | 'falling'): string {
    switch (trend) {
      case 'up': case 'rising': return 'text-green-600';
      case 'down': case 'falling': return 'text-red-600';
      default: return 'text-zinc-300';
    }
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
      case 'running': return '\u21BB';
      case 'completed': return '\u2713';
      case 'error': case 'failed': return '\u2717';
      default: return '\u23F8';
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
  $: trendSummary = $analyticsStore.trends.summary;
  $: distribution = $analyticsStore.revenueDistribution;
  $: problematicArtists = $analyticsStore.problematicArtistRevenue;
  $: tierDist = $analyticsStore.tierDistribution;
  $: rates = $analyticsStore.payoutRates;
</script>

<div class="brand-page surface-page">
  <div class="brand-page__inner brand-page__stack">
    <section class="brand-hero">
      <div class="brand-hero__header">
        <div class="brand-hero__copy">
          <div class="brand-kickers">
            <span class="brand-kicker">Evidence Operations</span>
            <span class="brand-kicker brand-kicker--accent">Analytics + Revenue</span>
          </div>
          <h1 class="brand-title brand-title--compact">Track the fallout, not just the filters.</h1>
          <p class="brand-subtitle">
            Monitor system health, quantify revenue exposure, and keep sync activity visible from the same operating surface.
          </p>
        </div>

        <div class="brand-hero__aside">
          <div class="brand-stat-grid brand-stat-grid--compact" aria-label="Analytics overview">
            <div class="brand-stat">
              <span class="brand-stat__value">{formatNumber(userStats?.blocked_artists)}</span>
              <span class="brand-stat__label">Your blocked artists</span>
            </div>
            <div class="brand-stat">
              <span class="brand-stat__value">{formatNumber(userStats?.subscriptions)}</span>
              <span class="brand-stat__label">Subscriptions</span>
            </div>
            <div class="brand-stat">
              <span class="brand-stat__value">{formatNumber(userStats?.manual_blocks)}</span>
              <span class="brand-stat__label">Manual blocks</span>
            </div>
            <div class="brand-stat">
              <span class="brand-stat__value">{$isAnySyncRunning ? 'Live' : 'Idle'}</span>
              <span class="brand-stat__label">Sync status</span>
            </div>
          </div>

          <div class="brand-actions analytics-toolbar">
            {#if activeTab === 'overview'}
              <select
                bind:value={selectedTimeRange}
                on:change={handleTimeRangeChange}
                class="surface-panel-thin px-4 py-2 rounded-xl text-white analytics-toolbar__select"
              >
                {#each timeRanges as range}
                  <option value={range.value} style="background: #1a1a2e;">{range.label}</option>
                {/each}
              </select>
              <button
                type="button"
                on:click={openReportModal}
                class="brand-button brand-button--primary analytics-toolbar__button"
              >
                <svg class="analytics-toolbar__icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/><polyline points="10 9 9 9 8 9"/></svg>
                Generate Report
              </button>
            {:else if activeTab === 'revenue'}
              <select
                bind:value={revenueDays}
                on:change={handleRevenuePeriodChange}
                class="surface-panel-thin px-4 py-2 rounded-xl text-white analytics-toolbar__select"
              >
                {#each periodOptions as option}
                  <option value={option.value} style="background: #1a1a2e;">{option.label}</option>
                {/each}
              </select>
            {/if}
          </div>
        </div>
      </div>

      <div class="analytics-tabs">
        <button
          type="button"
          on:click={() => handleTabChange('overview')}
          class="analytics-tab"
          class:analytics-tab--active={activeTab === 'overview'}
        >
          <svg class="analytics-tab__icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/></svg>
          Overview
        </button>
        <button
          type="button"
          on:click={() => handleTabChange('revenue')}
          class="analytics-tab"
          class:analytics-tab--active={activeTab === 'revenue'}
        >
          <svg class="analytics-tab__icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="1" x2="12" y2="23"/><path d="M17 5H9.5a3.5 3.5 0 000 7h5a3.5 3.5 0 010 7H6"/></svg>
          Revenue
        </button>
        <button
          type="button"
          on:click={() => handleTabChange('sync')}
          class="analytics-tab"
          class:analytics-tab--active={activeTab === 'sync'}
        >
          <svg class="analytics-tab__icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/><path d="M3.51 9a9 9 0 0114.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0020.49 15"/></svg>
          Sync
          {#if $isAnySyncRunning}
            <span class="analytics-tab__badge">Running</span>
          {/if}
        </button>
      </div>
    </section>

    <div class="analytics-content">
    <!-- Error display -->
    {#if $analyticsStore.error}
      <div class="brand-alert brand-alert--error mb-6">
        <div class="flex items-center gap-2">
          <svg class="w-4 h-4 flex-shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>
          <span>{$analyticsStore.error}</span>
          <button type="button" on:click={analyticsActions.clearError} class="brand-alert__dismiss">
            Dismiss
          </button>
        </div>
      </div>
    {/if}

    {#if $syncStore.error && activeTab === 'sync'}
      <div class="brand-alert brand-alert--error mb-6">
        <div class="flex items-center gap-2">
          <svg class="w-4 h-4 flex-shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>
          <span>{$syncStore.error}</span>
          <button type="button" on:click={syncActions.clearError} class="brand-alert__dismiss">
            Dismiss
          </button>
        </div>
      </div>
    {/if}

    <!-- Loading state -->
    {#if $analyticsStore.isLoading || $syncStore.isLoading}
      <div role="status" aria-label="Loading analytics data">
        <!-- Stats skeleton -->
        <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mb-8">
          {#each Array(4) as _}
            <Skeleton variant="rectangular" height="100px" />
          {/each}
        </div>
        <!-- Content skeleton -->
        <div class="space-y-6">
          <Skeleton variant="rectangular" height="200px" />
          <Skeleton variant="rectangular" height="150px" />
          <div class="grid md:grid-cols-2 gap-6">
            <Skeleton variant="rectangular" height="250px" />
            <Skeleton variant="rectangular" height="250px" />
          </div>
        </div>
        <span class="sr-only">Loading analytics data...</span>
      </div>
    {:else}
      <!-- ==================== OVERVIEW TAB ==================== -->
      {#if activeTab === 'overview'}
        <!-- Quick Stats Cards -->
        <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mb-8">
          <div class="rounded-xl p-5 bg-zinc-800 border border-zinc-600">
            <div class="flex items-center gap-3">
              <div class="w-10 h-10 rounded-full flex items-center justify-center bg-blue-900/50">
                <svg class="w-5 h-5 text-blue-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 21v-2a4 4 0 00-4-4H5a4 4 0 00-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 00-3-3.87"/><path d="M16 3.13a4 4 0 010 7.75"/></svg>
              </div>
              <div>
                <div class="text-2xl font-bold text-white">{formatNumber(dashboard?.total_users)}</div>
                <div class="text-sm text-zinc-400">Total Users</div>
              </div>
            </div>
          </div>

          <div class="rounded-xl p-5 bg-zinc-800 border border-zinc-600">
            <div class="flex items-center gap-3">
              <div class="w-10 h-10 rounded-full flex items-center justify-center bg-green-900/50">
                <svg class="w-5 h-5 text-green-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 11.08V12a10 10 0 11-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
              </div>
              <div>
                <div class="text-2xl font-bold text-white">{formatNumber(dashboard?.active_users_today)}</div>
                <div class="text-sm text-zinc-400">Active Today</div>
              </div>
            </div>
          </div>

          <div class="rounded-xl p-5 bg-zinc-800 border border-zinc-600">
            <div class="flex items-center gap-3">
              <div class="w-10 h-10 rounded-full flex items-center justify-center bg-red-900/50">
                <svg class="w-5 h-5 text-red-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="4.93" y1="4.93" x2="19.07" y2="19.07"/></svg>
              </div>
              <div>
                <div class="text-2xl font-bold text-white">{formatNumber(dashboard?.total_blocked_artists)}</div>
                <div class="text-sm text-zinc-400">Blocked Artists</div>
              </div>
            </div>
          </div>

          <div class="rounded-xl p-5 bg-zinc-800 border border-zinc-600">
            <div class="flex items-center gap-3">
              <div class="w-10 h-10 rounded-full flex items-center justify-center bg-purple-900/50">
                <svg class="w-5 h-5 text-purple-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 8A6 6 0 006 8c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 01-3.46 0"/></svg>
              </div>
              <div>
                <div class="text-2xl font-bold text-white">{formatNumber(dashboard?.offense_detections_today)}</div>
                <div class="text-sm text-zinc-400">Offenses Today</div>
              </div>
            </div>
          </div>
        </div>


        <!-- System Health - Hidden from users (admin-only feature) -->
        <!-- TODO: Move to admin dashboard when implemented -->

        <!-- Trend Summary -->
        {#if trendSummary}
          <div class="bg-zinc-800 rounded-xl p-6 border border-zinc-600 shadow-sm mb-8">
            <div class="flex items-center justify-between mb-4">
              <h2 class="text-lg font-semibold text-white">Trend Summary</h2>
              <span class={getTrendColor(trendSummary.trend)}>
                {getTrendIcon(trendSummary.trend)} {formatTrendPercent(trendSummary.change_percent)}%
              </span>
            </div>
            <div class="text-sm text-zinc-400 mb-4">Period: {trendSummary.period || 'Current selection'}</div>
            {#if trendSummary.data_points?.length > 0}
              <div class="h-32 flex items-end gap-1">
                {#each trendSummary.data_points as point}
                  {@const height = getTrendBarHeight(point.value, trendSummary.data_points)}
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
              <svg class="w-5 h-5 text-green-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="23 6 13.5 15.5 8.5 10.5 1 18"/><polyline points="17 6 23 6 23 12"/></svg>
              Rising Artists
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
              <svg class="w-5 h-5 text-red-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="23 18 13.5 8.5 8.5 13.5 1 6"/><polyline points="17 18 23 18 23 12"/></svg>
              Falling Artists
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
              <div class="w-12 h-12 bg-blue-900/50 rounded-full flex items-center justify-center">
                <svg class="w-6 h-6 text-blue-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 18V5l12-3v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="15" r="3"/></svg>
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
              <div class="w-12 h-12 bg-green-900/50 rounded-full flex items-center justify-center">
                <svg class="w-6 h-6 text-green-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="1" x2="12" y2="23"/><path d="M17 5H9.5a3.5 3.5 0 000 7h5a3.5 3.5 0 010 7H6"/></svg>
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
              <div class="w-12 h-12 bg-red-900/50 rounded-full flex items-center justify-center">
                <svg class="w-6 h-6 text-red-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
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
              <div class="w-12 h-12 bg-purple-900/50 rounded-full flex items-center justify-center">
                <svg class="w-6 h-6 text-purple-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="20" x2="18" y2="10"/><line x1="12" y1="20" x2="12" y2="4"/><line x1="6" y1="20" x2="6" y2="14"/></svg>
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
            onArtistClick={(artistId) => navigateToArtist(artistId)}
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
                <div class="text-2xl mb-2">{platform.abbr}</div>
                <div class="font-medium text-white">{platform.name}</div>
                <div class="text-xs text-zinc-400 mt-1">
                  {#if status?.status === 'running'}
                    <span class="text-blue-400">Syncing...</span>
                  {:else if status?.artists_count}
                    {formatNumber(status.artists_count)} artists
                  {:else if platform.alwaysAvailable}
                    <span class="text-green-400">Ready</span>
                  {:else}
                    <span class="text-zinc-400">No credentials</span>
                  {/if}
                </div>
                {#if status?.last_sync}
                  <div class="text-xs text-zinc-400 mt-1">
                    Last: {timeAgo(status.last_sync)}
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
                  {platform.abbr} {platform.name}
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
              Sync in Progress...
            {:else}
              Start Catalog Sync
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
                        {run.sync_type} sync • {timeAgo(run.started_at)}
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
</div>

<!-- Report Modal -->
{#if showReportModal}
  <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
  <div class="analytics-modal-backdrop fixed inset-0 bg-black/70 flex items-center justify-center p-4 z-50" on:click={closeReportModal} role="dialog" aria-modal="true">
    <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
    <div class="analytics-modal bg-zinc-800 rounded-2xl max-w-lg w-full p-6 shadow-xl border border-zinc-600" on:click|stopPropagation role="document">
      <div class="flex items-center mb-6">
        <div class="w-14 h-14 bg-purple-900/50 rounded-full flex items-center justify-center mr-4">
          <svg class="analytics-modal-icon w-7 h-7 text-purple-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/><polyline points="10 9 9 9 8 9"/></svg>
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

<style>
  .analytics-content {
    display: grid;
    gap: 1.5rem;
  }

  .analytics-content > * {
    min-width: 0;
  }

  .analytics-tabs {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
    margin-top: 1.5rem;
  }

  .analytics-toolbar {
    align-items: center;
    justify-content: flex-end;
  }

  .analytics-toolbar__select {
    min-height: 3rem;
    min-width: 9.5rem;
    border: 1px solid rgba(82, 93, 114, 0.5);
    background: linear-gradient(152deg, rgba(25, 28, 40, 0.94), rgba(14, 18, 28, 0.92));
  }

  .analytics-toolbar__button {
    min-height: 3rem;
    padding: 0.8rem 1rem;
    white-space: nowrap;
    box-shadow: 0 16px 28px rgba(225, 29, 72, 0.22);
  }

  .analytics-toolbar__icon {
    width: 1rem;
    height: 1rem;
    flex: 0 0 1rem;
  }

  .analytics-tab {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.5rem 1rem;
    border-radius: 9999px;
    font-size: 0.875rem;
    font-weight: 500;
    transition: all 150ms ease;
    background: none;
    border: none;
    cursor: pointer;
    color: #9CA3AF;
  }

  .analytics-tab:hover {
    color: #d1d5db;
    background: rgba(255, 255, 255, 0.05);
  }

  .analytics-tab--active {
    background: var(--color-brand-primary);
    color: white;
  }

  .analytics-tab--active:hover {
    background: var(--color-brand-primary-hover);
    color: white;
  }

  .analytics-tab__icon {
    width: 1rem;
    height: 1rem;
    flex-shrink: 0;
  }

  .analytics-tab__badge {
    display: inline-flex;
    align-items: center;
    padding: 0.125rem 0.5rem;
    border-radius: 9999px;
    font-size: 0.75rem;
    font-weight: 500;
    background: rgba(34, 197, 94, 0.2);
    color: #4ade80;
  }

  .analytics-modal-backdrop {
    backdrop-filter: blur(6px);
  }

  .analytics-modal {
    background: linear-gradient(155deg, rgba(26, 18, 31, 0.96), rgba(13, 15, 27, 0.96));
    border-color: rgba(148, 163, 184, 0.28);
  }

  .analytics-modal-icon {
    width: 1.75rem;
    height: 1.75rem;
  }

  /* Local utility fallbacks to keep this page stable even when utility CSS is absent. */
  .grid { display: grid; }
  .flex { display: flex; }
  .block { display: block; }
  .inline-flex { display: inline-flex; }
  .items-center { align-items: center; }
  .items-end { align-items: flex-end; }
  .justify-between { justify-content: space-between; }
  .justify-center { justify-content: center; }
  .text-center { text-align: center; }
  .text-right { text-align: right; }
  .flex-1 { flex: 1 1 0%; }
  .flex-shrink-0 { flex-shrink: 0; }
  .flex-wrap { flex-wrap: wrap; }
  .overflow-hidden { overflow: hidden; }
  .overflow-x-auto { overflow-x: auto; }
  .relative { position: relative; }
  .w-full { width: 100%; }
  .w-40 { width: 10rem; }
  .h-40 { height: 10rem; }
  .w-32 { width: 8rem; }
  .h-32 { height: 8rem; }
  .h-2 { height: 0.5rem; }
  .w-14 { width: 3.5rem; }
  .h-14 { height: 3.5rem; }
  .w-12 { width: 3rem; }
  .h-12 { height: 3rem; }
  .w-8 { width: 2rem; }
  .h-8 { height: 2rem; }
  .w-7 { width: 1.75rem; }
  .h-7 { height: 1.75rem; }
  .w-6 { width: 1.5rem; }
  .h-6 { height: 1.5rem; }
  .w-5 { width: 1.25rem; }
  .h-5 { height: 1.25rem; }
  .w-4 { width: 1rem; }
  .h-4 { height: 1rem; }
  .gap-1 { gap: 0.25rem; }
  .gap-2 { gap: 0.5rem; }
  .gap-3 { gap: 0.75rem; }
  .gap-4 { gap: 1rem; }
  .gap-6 { gap: 1.5rem; }
  .space-y-3 > * + * { margin-top: 0.75rem; }
  .space-y-4 > * + * { margin-top: 1rem; }
  .space-y-6 > * + * { margin-top: 1.5rem; }
  .p-3 { padding: 0.75rem; }
  .p-4 { padding: 1rem; }
  .p-5 { padding: 1.25rem; }
  .p-6 { padding: 1.5rem; }
  .px-3 { padding-left: 0.75rem; padding-right: 0.75rem; }
  .px-4 { padding-left: 1rem; padding-right: 1rem; }
  .px-6 { padding-left: 1.5rem; padding-right: 1.5rem; }
  .py-1 { padding-top: 0.25rem; padding-bottom: 0.25rem; }
  .py-2 { padding-top: 0.5rem; padding-bottom: 0.5rem; }
  .py-3 { padding-top: 0.75rem; padding-bottom: 0.75rem; }
  .py-8 { padding-top: 2rem; padding-bottom: 2rem; }
  .mt-1 { margin-top: 0.25rem; }
  .mt-2 { margin-top: 0.5rem; }
  .mt-8 { margin-top: 2rem; }
  .mb-2 { margin-bottom: 0.5rem; }
  .mb-4 { margin-bottom: 1rem; }
  .mb-6 { margin-bottom: 1.5rem; }
  .mb-8 { margin-bottom: 2rem; }
  .mr-4 { margin-right: 1rem; }
  .rounded { border-radius: 0.25rem; }
  .rounded-lg { border-radius: 0.75rem; }
  .rounded-xl { border-radius: 1rem; }
  .rounded-2xl { border-radius: 1.25rem; }
  .rounded-full { border-radius: 9999px; }
  .border { border-width: 1px; border-style: solid; }
  .border-2 { border-width: 2px; border-style: solid; }
  .border-zinc-600 { border-color: rgba(82, 93, 114, 0.62); }
  .text-xs { font-size: 0.75rem; line-height: 1rem; }
  .text-sm { font-size: 0.875rem; line-height: 1.25rem; }
  .text-lg { font-size: 1.125rem; line-height: 1.5rem; }
  .text-xl { font-size: 1.25rem; line-height: 1.75rem; }
  .text-2xl { font-size: 1.5rem; line-height: 2rem; }
  .font-medium { font-weight: 500; }
  .font-semibold { font-weight: 600; }
  .font-bold { font-weight: 700; }
  .capitalize { text-transform: capitalize; }
  .shadow-sm { box-shadow: 0 14px 34px rgba(2, 6, 23, 0.28); }
  .shadow-xl { box-shadow: 0 24px 50px rgba(2, 6, 23, 0.48); }
  .transform { transform: translateZ(0); }
  .-rotate-90 { transform: rotate(-90deg); }
  .transition-all { transition: all 180ms ease; }
  .transition-colors { transition: color 180ms ease, background-color 180ms ease, border-color 180ms ease; }
  .animate-spin { animation: analytics-spin 900ms linear infinite; }
  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    margin: -1px;
    padding: 0;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    border: 0;
    white-space: nowrap;
  }

  .bg-zinc-800,
  .bg-zinc-700 {
    background: linear-gradient(152deg, rgba(25, 28, 40, 0.94), rgba(14, 18, 28, 0.92));
  }
  .bg-zinc-700 { background: rgba(22, 26, 37, 0.92); }
  .bg-zinc-600 { background: rgba(66, 73, 90, 0.9); }
  .bg-indigo-600 { background: linear-gradient(135deg, #f43f5e, #e11d48); }
  .bg-indigo-700 { background: linear-gradient(135deg, #fb7185, #e11d48); }
  .bg-blue-900\/50 { background: rgba(37, 99, 235, 0.2); }
  .bg-green-900\/50 { background: rgba(22, 163, 74, 0.22); }
  .bg-red-900\/50 { background: rgba(220, 38, 38, 0.2); }
  .bg-purple-900\/50 { background: rgba(147, 51, 234, 0.2); }
  .bg-zinc-900\/30 { background: rgba(24, 27, 37, 0.68); }
  .bg-zinc-900\/50 { background: rgba(17, 24, 39, 0.82); }
  .bg-green-900\/30 { background: rgba(22, 163, 74, 0.16); }
  .bg-red-900\/30 { background: rgba(220, 38, 38, 0.16); }
  .bg-yellow-900\/30 { background: rgba(202, 138, 4, 0.16); }
  .bg-orange-900\/30 { background: rgba(234, 88, 12, 0.16); }
  .bg-indigo-900\/50 { background: rgba(99, 102, 241, 0.22); }
  .border-indigo-600 { border-color: rgba(129, 140, 248, 0.7); }
  .border-blue-700 { border-color: rgba(59, 130, 246, 0.52); }
  .border-green-700 { border-color: rgba(34, 197, 94, 0.44); }
  .border-red-700 { border-color: rgba(248, 113, 113, 0.42); }
  .border-yellow-700 { border-color: rgba(250, 204, 21, 0.42); }
  .border-orange-700 { border-color: rgba(251, 146, 60, 0.42); }
  .text-white { color: rgba(248, 250, 252, 0.96); }
  .text-zinc-300 { color: rgba(216, 227, 241, 0.9); }
  .text-zinc-400 { color: rgba(168, 184, 206, 0.75); }
  .text-blue-400 { color: #60a5fa; }
  .text-green-400 { color: #4ade80; }
  .text-green-500 { color: #22c55e; }
  .text-red-400 { color: #f87171; }
  .text-purple-400 { color: #a78bfa; }
  .text-indigo-400 { color: #fb7185; }
  .text-indigo-300 { color: #fda4af; }
  .text-indigo-600 { color: #f43f5e; }
  .text-red-600 { color: #f87171; }
  .text-green-600 { color: #22c55e; }
  .text-yellow-400 { color: #facc15; }
  .text-yellow-500 { color: #eab308; }
  .text-orange-400 { color: #fb923c; }
  .text-orange-500 { color: #f97316; }
  .text-red-500 { color: #ef4444; }

  .bg-green-500 { background-color: #22c55e; }
  .bg-red-500 { background-color: #ef4444; }

  .grid-cols-2 { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  .grid-cols-4 { grid-template-columns: repeat(4, minmax(0, 1fr)); }
  .grid-cols-5 { grid-template-columns: repeat(5, minmax(0, 1fr)); }

  .hover\:bg-indigo-500:hover { background: #fb7185; }
  .hover\:bg-indigo-700:hover { background: #e11d48; }
  .hover\:bg-zinc-600:hover { background: rgba(46, 53, 67, 0.94); }
  .hover\:bg-zinc-700:hover { background: rgba(34, 40, 52, 0.94); }

  .focus\:border-indigo-500:focus,
  .focus\:border-indigo-500:focus-visible {
    border-color: #fb7185;
    outline: none;
  }

  .focus\:ring-2:focus,
  .focus\:ring-2:focus-visible {
    box-shadow: 0 0 0 2px rgba(244, 63, 94, 0.22);
  }

  .disabled\:opacity-50:disabled { opacity: 0.5; }
  .disabled\:cursor-not-allowed:disabled { cursor: not-allowed; }

  @media (max-width: 900px) {
    .analytics-tabs {
      gap: 0.5rem;
    }

    .analytics-tab {
      flex: 1 1 calc(50% - 0.5rem);
      justify-content: center;
      min-width: 8.25rem;
    }

    .analytics-toolbar {
      width: 100%;
      justify-content: flex-start;
    }
  }

  @media (max-width: 768px) {
    .md\:grid-cols-2 {
      grid-template-columns: 1fr;
    }

    .md\:grid-cols-4 {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }

    .md\:grid-cols-5 {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }

  @media (min-width: 768px) {
    .md\:grid-cols-2 {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }

    .md\:grid-cols-4 {
      grid-template-columns: repeat(4, minmax(0, 1fr));
    }

    .md\:grid-cols-5 {
      grid-template-columns: repeat(5, minmax(0, 1fr));
    }
  }

  @media (min-width: 1024px) {
    .lg\:grid-cols-5 {
      grid-template-columns: repeat(5, minmax(0, 1fr));
    }
  }

  @keyframes analytics-spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
