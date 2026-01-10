<script lang="ts">
  import { onMount } from 'svelte';
  import {
    analyticsStore,
    analyticsActions,
    type GlobalCategoryRevenue,
    type CategoryRevenue,
    type CategoryArtistRevenue,
  } from '../stores/analytics';

  // Props
  export let showDetails: boolean = true;
  export let maxCategories: number = 12;
  export let onArtistClick: ((artistId: string) => void) | null = null;

  // Local state
  let isLoading = false;
  let error: string | null = null;
  let selectedCategory: string | null = null;
  let categoryDetails: CategoryRevenue | null = null;

  // Reactive data from store
  $: globalRevenue = $analyticsStore.globalCategoryRevenue;

  // Category colors for visual consistency
  const categoryColors: Record<string, string> = {
    sexual_misconduct: 'bg-rose-600',
    domestic_violence: 'bg-red-600',
    child_abuse: 'bg-red-800',
    hate_speech: 'bg-orange-600',
    racism: 'bg-orange-700',
    antisemitism: 'bg-amber-700',
    homophobia: 'bg-amber-600',
    violent_crime: 'bg-red-500',
    drug_trafficking: 'bg-purple-600',
    fraud: 'bg-blue-600',
    animal_abuse: 'bg-emerald-600',
    certified_creeper: 'bg-pink-600',
    other: 'bg-zinc-500',
  };

  function getCategoryColor(category: string): string {
    return categoryColors[category] || 'bg-zinc-500';
  }

  function formatCurrency(value: string | number): string {
    const num = typeof value === 'string' ? parseFloat(value) : value;
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 0,
      maximumFractionDigits: 0,
    }).format(num);
  }

  function formatNumber(value: number): string {
    return new Intl.NumberFormat('en-US', {
      notation: 'compact',
      compactDisplay: 'short',
    }).format(value);
  }

  async function loadData() {
    isLoading = true;
    error = null;

    const result = await analyticsActions.fetchGlobalCategoryRevenue();

    if (!result.success) {
      error = result.message || 'Failed to load category revenue data';
    }

    isLoading = false;
  }

  async function selectCategory(category: string) {
    if (selectedCategory === category) {
      selectedCategory = null;
      categoryDetails = null;
      return;
    }

    selectedCategory = category;
    const result = await analyticsActions.fetchCategoryRevenue(category, 10);

    if (result.success && result.data) {
      categoryDetails = result.data;
    }
  }

  function handleArtistClick(artistId: string) {
    if (onArtistClick) {
      onArtistClick(artistId);
    }
  }

  onMount(() => {
    loadData();
  });
</script>

<div class="category-revenue-breakdown" data-testid="category-revenue-breakdown">
  <!-- Header -->
  <div class="flex items-center justify-between mb-4">
    <h3 class="text-lg font-semibold text-zinc-100">Revenue by Offense Category</h3>
    <button
      on:click={loadData}
      disabled={isLoading}
      class="text-sm text-zinc-400 hover:text-zinc-200 disabled:opacity-50"
      data-testid="refresh-button"
    >
      {isLoading ? 'Loading...' : 'Refresh'}
    </button>
  </div>

  {#if error}
    <div class="bg-red-900/20 border border-red-800 rounded-lg p-4 text-red-400" data-testid="error-message">
      {error}
    </div>
  {:else if isLoading && !globalRevenue}
    <div class="flex items-center justify-center py-8" data-testid="loading-spinner">
      <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-zinc-400"></div>
    </div>
  {:else if globalRevenue}
    <!-- Summary Cards -->
    <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6" data-testid="summary-cards">
      <div class="bg-zinc-800 rounded-lg p-4">
        <div class="text-sm text-zinc-400">Total Simulated Revenue</div>
        <div class="text-xl font-bold text-zinc-100" data-testid="total-revenue">
          {formatCurrency(globalRevenue.total_simulated_revenue)}
        </div>
        <div class="text-xs text-zinc-500">per month</div>
      </div>

      <div class="bg-zinc-800 rounded-lg p-4">
        <div class="text-sm text-zinc-400">Problematic Artists</div>
        <div class="text-xl font-bold text-red-400" data-testid="problematic-revenue">
          {formatCurrency(globalRevenue.problematic_artist_revenue)}
        </div>
        <div class="text-xs text-zinc-500">{globalRevenue.problematic_percentage.toFixed(1)}% of total</div>
      </div>

      <div class="bg-zinc-800 rounded-lg p-4">
        <div class="text-sm text-zinc-400">Clean Artists</div>
        <div class="text-xl font-bold text-emerald-400" data-testid="clean-revenue">
          {formatCurrency(globalRevenue.clean_artist_revenue)}
        </div>
        <div class="text-xs text-zinc-500">{(100 - globalRevenue.problematic_percentage).toFixed(1)}% of total</div>
      </div>

      <div class="bg-zinc-800 rounded-lg p-4">
        <div class="text-sm text-zinc-400">Artists with Offenses</div>
        <div class="text-xl font-bold text-zinc-100" data-testid="artist-count">
          {globalRevenue.total_artists_with_offenses}
        </div>
        <div class="text-xs text-zinc-500">documented</div>
      </div>
    </div>

    <!-- Category Breakdown Bar -->
    <div class="mb-6">
      <div class="text-sm text-zinc-400 mb-2">Revenue Distribution by Category</div>
      <div class="h-6 rounded-full overflow-hidden flex bg-zinc-700" data-testid="category-bar">
        {#each globalRevenue.by_category.slice(0, maxCategories) as category}
          {#if category.percentage_of_total > 0.5}
            <div
              class="{getCategoryColor(category.category)} cursor-pointer hover:opacity-80 transition-opacity"
              style="width: {category.percentage_of_total}%"
              title="{category.display_name}: {formatCurrency(category.simulated_revenue)} ({category.percentage_of_total.toFixed(1)}%)"
              on:click={() => selectCategory(category.category)}
              on:keypress={(e) => e.key === 'Enter' && selectCategory(category.category)}
              role="button"
              tabindex="0"
              data-testid="category-segment-{category.category}"
            />
          {/if}
        {/each}
      </div>
    </div>

    <!-- Category List -->
    {#if showDetails}
      <div class="space-y-2" data-testid="category-list">
        {#each globalRevenue.by_category.slice(0, maxCategories) as category}
          <button
            class="w-full text-left bg-zinc-800 rounded-lg p-3 hover:bg-zinc-700 transition-colors
                   {selectedCategory === category.category ? 'ring-2 ring-zinc-500' : ''}"
            on:click={() => selectCategory(category.category)}
            data-testid="category-item-{category.category}"
          >
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="w-3 h-3 rounded-full {getCategoryColor(category.category)}"></div>
                <span class="text-zinc-200">{category.display_name}</span>
                <span class="text-xs text-zinc-500">({category.artist_count} artists)</span>
              </div>
              <div class="text-right">
                <div class="text-zinc-100 font-medium">{formatCurrency(category.simulated_revenue)}</div>
                <div class="text-xs text-zinc-500">{category.percentage_of_total.toFixed(1)}%</div>
              </div>
            </div>

            <!-- Expanded Category Details -->
            {#if selectedCategory === category.category && categoryDetails}
              <div class="mt-3 pt-3 border-t border-zinc-700" data-testid="category-details">
                <div class="text-sm text-zinc-400 mb-2">Top Artists in {category.display_name}</div>
                <div class="space-y-2">
                  {#each categoryDetails.top_artists as artist}
                    <div
                      class="flex items-center justify-between py-1 px-2 rounded hover:bg-zinc-600 cursor-pointer"
                      on:click|stopPropagation={() => handleArtistClick(artist.artist_id)}
                      on:keypress={(e) => e.key === 'Enter' && handleArtistClick(artist.artist_id)}
                      role="button"
                      tabindex="0"
                      data-testid="artist-item-{artist.artist_id}"
                    >
                      <div>
                        <span class="text-zinc-200">{artist.artist_name}</span>
                        <span class="text-xs text-zinc-500 ml-2">
                          {artist.offense_count} offense{artist.offense_count !== 1 ? 's' : ''}
                        </span>
                      </div>
                      <div class="text-sm text-zinc-300">
                        {formatCurrency(artist.simulated_revenue)}
                      </div>
                    </div>
                  {/each}
                </div>
              </div>
            {/if}
          </button>
        {/each}
      </div>
    {/if}

    <!-- Footer -->
    <div class="mt-4 text-xs text-zinc-500 text-center" data-testid="footer-note">
      Revenue figures are simulated based on average streaming payouts (~$0.004/stream)
    </div>
  {:else}
    <div class="text-center py-8 text-zinc-500" data-testid="no-data">
      No category revenue data available
    </div>
  {/if}
</div>

<style>
  .category-revenue-breakdown {
    background-color: #18181b;
    border-radius: 0.75rem;
    padding: 1.5rem;
  }
</style>
