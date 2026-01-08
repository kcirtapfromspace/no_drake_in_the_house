<script lang="ts">
  import { onMount } from 'svelte';
  import {
    analyticsStore,
    analyticsActions,
  } from '../stores/analytics';
  import type { TroubleTier, ArtistRevenueBreakdown } from '../stores/analytics';
  import { navigateTo } from '../utils/simple-router';

  let selectedDays = 30;
  let selectedMinTier: TroubleTier = 'moderate';
  let isLoading = true;

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

  onMount(async () => {
    await fetchData();
    isLoading = false;
  });

  async function fetchData() {
    isLoading = true;
    await Promise.all([
      analyticsActions.fetchRevenueDistribution(selectedDays),
      analyticsActions.fetchProblematicArtistRevenue(selectedDays, selectedMinTier, 10),
      analyticsActions.fetchTierDistribution(),
      analyticsActions.fetchPayoutRates(),
    ]);
    isLoading = false;
  }

  async function handlePeriodChange() {
    await fetchData();
  }

  async function handleTierChange() {
    await analyticsActions.fetchProblematicArtistRevenue(selectedDays, selectedMinTier, 10);
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

  function formatNumber(num: number): string {
    return new Intl.NumberFormat('en-US').format(num);
  }

  function getTierColor(tier: TroubleTier | undefined): string {
    switch (tier) {
      case 'critical': return 'bg-red-600';
      case 'high': return 'bg-orange-500';
      case 'moderate': return 'bg-yellow-500';
      case 'low': return 'bg-green-500';
      default: return 'bg-gray-400';
    }
  }

  function getTierBgColor(tier: TroubleTier | undefined): string {
    switch (tier) {
      case 'critical': return 'bg-red-50 border-red-200';
      case 'high': return 'bg-orange-50 border-orange-200';
      case 'moderate': return 'bg-yellow-50 border-yellow-200';
      case 'low': return 'bg-green-50 border-green-200';
      default: return 'bg-gray-50 border-gray-200';
    }
  }

  function getTierLabel(tier: TroubleTier | undefined): string {
    if (!tier) return 'Unknown';
    return tier.charAt(0).toUpperCase() + tier.slice(1);
  }

  $: distribution = $analyticsStore.revenueDistribution;
  $: problematicArtists = $analyticsStore.problematicArtistRevenue;
  $: tierDist = $analyticsStore.tierDistribution;
  $: rates = $analyticsStore.payoutRates;
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
            Your Streaming Impact
          </h1>
          <p class="text-lg text-gray-600">
            See where your streaming revenue goes and how much reaches problematic artists.
          </p>
        </div>
        <div class="flex items-center gap-4">
          <select
            bind:value={selectedDays}
            on:change={handlePeriodChange}
            class="px-4 py-2 border border-gray-200 rounded-lg focus:border-indigo-500 focus:ring-2 focus:ring-indigo-200"
          >
            {#each periodOptions as option}
              <option value={option.value}>{option.label}</option>
            {/each}
          </select>
        </div>
      </div>
    </div>
  </div>

  <div class="max-w-6xl mx-auto px-4 py-6 sm:px-6 lg:px-8">
    {#if isLoading}
      <div class="flex justify-center py-12">
        <div class="w-8 h-8 border-4 border-indigo-500 border-t-transparent rounded-full animate-spin"></div>
      </div>
    {:else}
      <!-- Summary Cards -->
      <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mb-8">
        <div class="bg-white rounded-xl p-5 border border-gray-100 shadow-sm">
          <div class="flex items-center gap-3">
            <div class="w-12 h-12 bg-blue-100 rounded-full flex items-center justify-center text-xl">
              &#127911;
            </div>
            <div>
              <div class="text-2xl font-bold text-gray-900">
                {formatNumber(distribution?.total_streams ?? 0)}
              </div>
              <div class="text-sm text-gray-500">Total Streams</div>
            </div>
          </div>
        </div>

        <div class="bg-white rounded-xl p-5 border border-gray-100 shadow-sm">
          <div class="flex items-center gap-3">
            <div class="w-12 h-12 bg-green-100 rounded-full flex items-center justify-center text-xl">
              &#128176;
            </div>
            <div>
              <div class="text-2xl font-bold text-gray-900">
                {formatCurrency(distribution?.total_revenue)}
              </div>
              <div class="text-sm text-gray-500">Total Revenue</div>
            </div>
          </div>
        </div>

        <div class="bg-white rounded-xl p-5 border border-gray-100 shadow-sm">
          <div class="flex items-center gap-3">
            <div class="w-12 h-12 bg-red-100 rounded-full flex items-center justify-center text-xl">
              &#9888;
            </div>
            <div>
              <div class="text-2xl font-bold text-red-600">
                {formatCurrency(distribution?.revenue_to_problematic_artists)}
              </div>
              <div class="text-sm text-gray-500">To Problematic Artists</div>
            </div>
          </div>
        </div>

        <div class="bg-white rounded-xl p-5 border border-gray-100 shadow-sm">
          <div class="flex items-center gap-3">
            <div class="w-12 h-12 bg-purple-100 rounded-full flex items-center justify-center text-xl">
              &#128200;
            </div>
            <div>
              <div class="text-2xl font-bold text-purple-600">
                {(distribution?.problematic_percentage ?? 0).toFixed(1)}%
              </div>
              <div class="text-sm text-gray-500">Problematic Share</div>
            </div>
          </div>
        </div>
      </div>

      <!-- Revenue Distribution Chart -->
      <div class="bg-white rounded-xl p-6 border border-gray-100 shadow-sm mb-8">
        <h2 class="text-lg font-semibold text-gray-900 mb-4">Revenue Distribution</h2>
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
                <span class="text-gray-700">Clean Artists</span>
              </div>
              <div class="text-right">
                <div class="font-semibold">{formatCurrency(distribution?.revenue_to_clean_artists)}</div>
                <div class="text-sm text-gray-500">{(100 - (distribution?.problematic_percentage ?? 0)).toFixed(1)}%</div>
              </div>
            </div>
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <div class="w-4 h-4 rounded bg-red-500"></div>
                <span class="text-gray-700">Problematic Artists</span>
              </div>
              <div class="text-right">
                <div class="font-semibold text-red-600">{formatCurrency(distribution?.revenue_to_problematic_artists)}</div>
                <div class="text-sm text-gray-500">{(distribution?.problematic_percentage ?? 0).toFixed(1)}%</div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Tier Distribution -->
      {#if tierDist}
        <div class="bg-white rounded-xl p-6 border border-gray-100 shadow-sm mb-8">
          <h2 class="text-lg font-semibold text-gray-900 mb-4">Artists by Trouble Tier</h2>
          <div class="grid grid-cols-4 gap-4">
            <div class="text-center p-4 bg-green-50 rounded-lg border border-green-200">
              <div class="text-2xl font-bold text-green-700">{tierDist.low}</div>
              <div class="text-sm text-green-600">Low</div>
            </div>
            <div class="text-center p-4 bg-yellow-50 rounded-lg border border-yellow-200">
              <div class="text-2xl font-bold text-yellow-700">{tierDist.moderate}</div>
              <div class="text-sm text-yellow-600">Moderate</div>
            </div>
            <div class="text-center p-4 bg-orange-50 rounded-lg border border-orange-200">
              <div class="text-2xl font-bold text-orange-700">{tierDist.high}</div>
              <div class="text-sm text-orange-600">High</div>
            </div>
            <div class="text-center p-4 bg-red-50 rounded-lg border border-red-200">
              <div class="text-2xl font-bold text-red-700">{tierDist.critical}</div>
              <div class="text-sm text-red-600">Critical</div>
            </div>
          </div>
        </div>
      {/if}

      <!-- Top Problematic Artists -->
      <div class="bg-white rounded-xl p-6 border border-gray-100 shadow-sm mb-8">
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-lg font-semibold text-gray-900">Top Problematic Artists by Your Revenue</h2>
          <select
            bind:value={selectedMinTier}
            on:change={handleTierChange}
            class="px-3 py-1 text-sm border border-gray-200 rounded-lg focus:border-indigo-500"
          >
            {#each tierOptions as option}
              <option value={option.value}>{option.label}</option>
            {/each}
          </select>
        </div>

        {#if problematicArtists.length === 0}
          <div class="text-center py-8 text-gray-500">
            <p>No problematic artists found in your listening history.</p>
            <p class="text-sm mt-2">This is great news! Your streaming revenue is going to clean artists.</p>
          </div>
        {:else}
          <div class="space-y-3">
            {#each problematicArtists as artist, index}
              <div class="flex items-center justify-between p-4 rounded-lg border {getTierBgColor(artist.trouble_tier)}">
                <div class="flex items-center gap-4">
                  <div class="w-8 h-8 rounded-full bg-gray-200 flex items-center justify-center font-bold text-gray-600">
                    {index + 1}
                  </div>
                  <div>
                    <div class="font-medium text-gray-900">{artist.artist_name}</div>
                    <div class="text-xs text-gray-500 flex items-center gap-2">
                      <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium {getTierColor(artist.trouble_tier)} text-white">
                        {getTierLabel(artist.trouble_tier)}
                      </span>
                      <span>Score: {(artist.trouble_score ?? 0).toFixed(2)}</span>
                    </div>
                  </div>
                </div>
                <div class="text-right">
                  <div class="font-semibold text-gray-900">{formatCurrency(artist.total_revenue)}</div>
                  <div class="text-xs text-gray-500">
                    {formatNumber(artist.total_streams)} streams ({artist.percentage_of_user_spend.toFixed(1)}%)
                  </div>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Platform Payout Rates -->
      <div class="bg-white rounded-xl p-6 border border-gray-100 shadow-sm">
        <h2 class="text-lg font-semibold text-gray-900 mb-4">Platform Payout Rates</h2>
        <p class="text-sm text-gray-500 mb-4">
          Average payout per stream by platform. Actual rates vary based on subscription tier, region, and other factors.
        </p>
        <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-4">
          {#each rates as rate}
            <div class="text-center p-4 bg-gray-50 rounded-lg">
              <div class="text-lg font-bold text-indigo-600">
                {formatCurrency(rate.rate_per_stream)}
              </div>
              <div class="text-sm font-medium text-gray-700 capitalize">{rate.platform.replace('_', ' ')}</div>
              <div class="text-xs text-gray-500">{rate.rate_tier}</div>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </div>
</div>
