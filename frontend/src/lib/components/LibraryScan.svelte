<script lang="ts">
  import { onMount } from 'svelte';
  import { navigateTo } from '../utils/simple-router';
  import {
    libraryStore,
    libraryActions,
    flaggedArtists,
    scanStats,
    severityConfig,
    categoryLabels,
    type FlaggedArtist,
    type OffenseSeverity
  } from '../stores/library';
  import { dnpActions } from '../stores/dnp';

  // Reactive state from store
  $: isScanning = $libraryStore.isScanning;
  $: scanProgress = $libraryStore.scanProgress;
  $: scanComplete = $libraryStore.scanResult !== null;
  $: error = $libraryStore.error;

  async function startScan() {
    await libraryActions.scanLibrary();
  }

  function getSeverityCount(severity: OffenseSeverity): number {
    return $flaggedArtists.filter(a => a.severity === severity).length;
  }

  async function blockArtist(artist: FlaggedArtist) {
    // Add to personal blocklist
    const tags = artist.offenses.map(o => o.category);
    const note = artist.offenses.map(o => `${categoryLabels[o.category]}: ${o.title} (${o.date})`).join('\n');

    await dnpActions.addArtist(artist.id, tags, note);

    // Update local state - remove from flagged list
    libraryStore.update(s => ({
      ...s,
      scanResult: s.scanResult ? {
        ...s.scanResult,
        flagged_artists: s.scanResult.flagged_artists.filter(a => a.id !== artist.id)
      } : null
    }));
  }

  async function blockAll() {
    for (const artist of $flaggedArtists) {
      await blockArtist(artist);
    }
  }

  function resetScan() {
    libraryActions.resetScan();
  }
</script>

<div class="min-h-screen bg-gradient-to-b from-purple-50 to-white">
  <!-- Header -->
  <div class="bg-white border-b border-gray-100">
    <div class="max-w-4xl mx-auto px-4 py-8 sm:px-6 lg:px-8">
      <button
        type="button"
        on:click={() => navigateTo('dashboard')}
        class="text-gray-500 hover:text-gray-700 mb-4 flex items-center text-sm"
      >
        <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
        </svg>
        Back to Dashboard
      </button>
      <h1 class="text-3xl font-bold text-gray-900 mb-2">
        Library Scan
      </h1>
      <p class="text-lg text-gray-600">
        Scan your imported music library to find artists with documented misconduct.
      </p>
    </div>
  </div>

  <div class="max-w-4xl mx-auto px-4 py-8 sm:px-6 lg:px-8">
    {#if error}
      <div class="bg-red-50 border border-red-200 rounded-xl p-4 mb-6">
        <div class="flex items-start">
          <svg class="w-5 h-5 text-red-500 mr-3 mt-0.5" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
          </svg>
          <div class="flex-1">
            <p class="text-red-800">{error}</p>
            <button
              type="button"
              on:click={libraryActions.clearError}
              class="text-red-600 underline text-sm mt-1"
            >
              Dismiss
            </button>
          </div>
        </div>
      </div>
    {/if}

    {#if !scanComplete && !isScanning}
      <!-- Start scan prompt -->
      <div class="bg-white rounded-2xl shadow-lg border border-gray-100 p-8 text-center">
        <div class="w-20 h-20 bg-purple-100 rounded-full flex items-center justify-center mx-auto mb-6">
          <svg class="w-10 h-10 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
        </div>
        <h2 class="text-2xl font-bold text-gray-900 mb-4">
          Check Your Library
        </h2>
        <p class="text-gray-600 max-w-lg mx-auto mb-8">
          We'll cross-reference your imported music library against our evidence database
          of artists with documented harmful behavior.
        </p>

        <div class="bg-gray-50 rounded-xl p-6 mb-8 text-left max-w-md mx-auto">
          <h3 class="font-medium text-gray-900 mb-3">Before you scan:</h3>
          <ul class="space-y-2 text-sm text-gray-600">
            <li class="flex items-start">
              <span class="text-purple-500 mr-2">1.</span>
              <span>Import your library from <button type="button" on:click={() => navigateTo('connections')} class="text-purple-600 underline">Connections</button></span>
            </li>
            <li class="flex items-start">
              <span class="text-purple-500 mr-2">2.</span>
              <span>Upload your Spotify or Apple Music export file</span>
            </li>
            <li class="flex items-start">
              <span class="text-purple-500 mr-2">3.</span>
              <span>Click scan to check against our database</span>
            </li>
          </ul>
        </div>

        <button
          type="button"
          on:click={startScan}
          class="px-8 py-4 bg-purple-600 text-white text-lg font-medium rounded-xl hover:bg-purple-700 transition-colors"
        >
          Start Library Scan
        </button>
      </div>

    {:else if isScanning}
      <!-- Scanning in progress -->
      <div class="bg-white rounded-2xl shadow-lg border border-gray-100 p-8 text-center">
        <div class="w-20 h-20 bg-purple-100 rounded-full flex items-center justify-center mx-auto mb-6 animate-pulse">
          <svg class="w-10 h-10 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
        </div>
        <h2 class="text-2xl font-bold text-gray-900 mb-4">
          Scanning Your Library...
        </h2>
        <p class="text-gray-600 mb-8">
          Cross-referencing your music against our evidence database
        </p>

        <div class="max-w-md mx-auto">
          <div class="h-4 bg-gray-200 rounded-full overflow-hidden">
            <div
              class="h-full bg-purple-600 transition-all duration-300"
              style="width: {scanProgress}%"
            ></div>
          </div>
          <p class="text-sm text-gray-500 mt-2">{scanProgress}% complete</p>
        </div>
      </div>

    {:else}
      <!-- Scan results -->
      <div class="space-y-6">
        <!-- Summary card -->
        <div class="bg-white rounded-2xl shadow-lg border border-gray-100 p-6">
          <div class="flex items-center justify-between mb-4">
            <h2 class="text-xl font-bold text-gray-900">Scan Results</h2>
            <button
              type="button"
              on:click={resetScan}
              class="text-sm text-gray-500 hover:text-gray-700"
            >
              Scan Again
            </button>
          </div>

          {#if $scanStats}
            <div class="grid grid-cols-2 sm:grid-cols-4 gap-4 mb-6">
              <div class="bg-gray-50 rounded-xl p-4 text-center">
                <div class="text-2xl font-bold text-gray-900">{$scanStats.totalTracks.toLocaleString()}</div>
                <div class="text-sm text-gray-500">Total Tracks</div>
              </div>
              <div class="bg-gray-50 rounded-xl p-4 text-center">
                <div class="text-2xl font-bold text-gray-900">{$scanStats.totalArtists}</div>
                <div class="text-sm text-gray-500">Artists</div>
              </div>
              <div class="bg-red-50 rounded-xl p-4 text-center">
                <div class="text-2xl font-bold text-red-600">{$scanStats.flaggedArtists}</div>
                <div class="text-sm text-red-600">Flagged Artists</div>
              </div>
              <div class="bg-red-50 rounded-xl p-4 text-center">
                <div class="text-2xl font-bold text-red-600">{$scanStats.flaggedTracks}</div>
                <div class="text-sm text-red-600">Flagged Tracks</div>
              </div>
            </div>
          {/if}

          <!-- Severity breakdown -->
          {#if $flaggedArtists.length > 0}
            <div class="flex items-center gap-4 text-sm flex-wrap">
              <span class="text-gray-500">By severity:</span>
              {#each Object.entries(severityConfig) as [key, config]}
                {@const count = getSeverityCount(key)}
                {#if count > 0}
                  <span class="flex items-center">
                    <span class="w-3 h-3 rounded-full {config.color} mr-1"></span>
                    {count} {config.label}
                  </span>
                {/if}
              {/each}
            </div>
          {/if}
        </div>

        <!-- Flagged artists list -->
        {#if $flaggedArtists.length > 0}
          <div class="flex items-center justify-between">
            <h3 class="text-lg font-semibold text-gray-900">Flagged Artists</h3>
            <button
              type="button"
              on:click={blockAll}
              class="px-4 py-2 bg-red-600 text-white text-sm font-medium rounded-lg hover:bg-red-700 transition-colors"
            >
              Block All Flagged
            </button>
          </div>

          <div class="space-y-4">
            {#each $flaggedArtists as artist}
              <div class="bg-white rounded-2xl border border-gray-100 shadow-sm overflow-hidden">
                <div class="p-5">
                  <div class="flex items-start justify-between mb-4">
                    <div class="flex items-center">
                      <div class="w-12 h-12 bg-gray-200 rounded-full flex items-center justify-center text-xl mr-4">
                        <svg class="w-6 h-6 text-gray-400" fill="currentColor" viewBox="0 0 20 20">
                          <path fill-rule="evenodd" d="M10 9a3 3 0 100-6 3 3 0 000 6zm-7 9a7 7 0 1114 0H3z" clip-rule="evenodd" />
                        </svg>
                      </div>
                      <div>
                        <h4 class="font-bold text-gray-900">{artist.name}</h4>
                        <p class="text-sm text-gray-500">
                          {artist.track_count} tracks in your library
                        </p>
                      </div>
                    </div>
                    <div class="flex items-center gap-2">
                      <span class="px-3 py-1 {severityConfig[artist.severity].color} text-white text-xs font-medium rounded-full">
                        {severityConfig[artist.severity].label}
                      </span>
                    </div>
                  </div>

                  <!-- Offenses -->
                  <div class="space-y-3 mb-4">
                    {#each artist.offenses as offense}
                      <div class="bg-red-50 rounded-xl p-4">
                        <div class="flex items-start justify-between">
                          <div>
                            <span class="text-xs font-medium text-red-600 uppercase tracking-wide">
                              {categoryLabels[offense.category]}
                            </span>
                            <h5 class="font-medium text-gray-900 mt-1">{offense.title}</h5>
                            <p class="text-sm text-gray-500 mt-1">
                              {offense.date} - {offense.evidence_count} evidence sources
                            </p>
                          </div>
                          <button
                            type="button"
                            on:click={() => navigateTo('offense-database')}
                            class="text-sm text-indigo-600 hover:text-indigo-700 font-medium"
                          >
                            View Evidence
                          </button>
                        </div>
                      </div>
                    {/each}
                  </div>

                  <!-- Actions -->
                  <div class="flex gap-3">
                    <button
                      type="button"
                      on:click={() => blockArtist(artist)}
                      class="flex-1 px-4 py-2.5 bg-red-600 text-white font-medium rounded-xl hover:bg-red-700 transition-colors"
                    >
                      Block Artist
                    </button>
                    <button
                      type="button"
                      on:click={() => {
                        libraryStore.update(s => ({
                          ...s,
                          scanResult: s.scanResult ? {
                            ...s.scanResult,
                            flagged_artists: s.scanResult.flagged_artists.filter(a => a.id !== artist.id)
                          } : null
                        }));
                      }}
                      class="px-4 py-2.5 border border-gray-200 text-gray-700 font-medium rounded-xl hover:bg-gray-50 transition-colors"
                    >
                      Ignore
                    </button>
                  </div>
                </div>
              </div>
            {/each}
          </div>
        {:else}
          <!-- Clean library -->
          <div class="bg-green-50 rounded-2xl p-8 text-center">
            <div class="w-16 h-16 bg-green-100 rounded-full flex items-center justify-center mx-auto mb-4">
              <svg class="w-8 h-8 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
            </div>
            <h3 class="text-xl font-semibold text-green-900 mb-2">
              Your library looks clean!
            </h3>
            <p class="text-green-700">
              We didn't find any artists with documented misconduct in your imported music library.
            </p>
          </div>
        {/if}
      </div>
    {/if}

    <!-- Import prompt -->
    {#if !scanComplete}
      <div class="mt-8 bg-blue-50 rounded-2xl p-6">
        <div class="flex items-start">
          <div class="w-12 h-12 bg-blue-100 rounded-full flex items-center justify-center mr-4 flex-shrink-0">
            <svg class="w-6 h-6 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
            </svg>
          </div>
          <div>
            <h4 class="font-medium text-blue-900 mb-1">Import Your Music Library</h4>
            <p class="text-sm text-blue-800 mb-3">
              To scan your library, first import your music data from your streaming service.
              Go to Connections to upload your Spotify or Apple Music export.
            </p>
            <button
              type="button"
              on:click={() => navigateTo('connections')}
              class="text-sm text-blue-700 font-medium hover:text-blue-800"
            >
              Go to Connections
            </button>
          </div>
        </div>
      </div>
    {/if}

    <!-- Evidence database info -->
    <div class="mt-8 bg-gray-50 rounded-2xl p-6">
      <div class="flex items-start">
        <div class="w-12 h-12 bg-gray-100 rounded-full flex items-center justify-center mr-4 flex-shrink-0">
          <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
        </div>
        <div>
          <h4 class="font-medium text-gray-900 mb-1">About Our Evidence Database</h4>
          <p class="text-sm text-gray-600 mb-3">
            Our database contains documented cases from court records, major news outlets,
            and verified reports. Each entry is reviewed for accuracy and includes links
            to primary sources.
          </p>
          <button
            type="button"
            on:click={() => navigateTo('community')}
            class="text-sm text-indigo-600 font-medium hover:text-indigo-700"
          >
            Learn more about our verification process
          </button>
        </div>
      </div>
    </div>
  </div>
</div>
