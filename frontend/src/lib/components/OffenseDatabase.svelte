<script lang="ts">
  import { onMount } from 'svelte';
  import { navigateTo } from '../utils/simple-router';
  import {
    getFlaggedArtistsDatabase,
    getOffenseWithEvidence,
    createOffense,
    addEvidence,
    searchArtists,
    severityConfig,
    categoryLabels,
    type FlaggedArtist,
    type OffenseSeverity,
    type OffenseCategory,
    type OffenseWithEvidence,
    type CreateOffenseRequest,
    type AddEvidenceRequest,
  } from '../stores/library';

  // State
  let flaggedArtists: FlaggedArtist[] = [];
  let loading = true;
  let error: string | null = null;
  let severityFilter: OffenseSeverity | null = null;

  // View modes
  type ViewMode = 'browse' | 'detail' | 'submit-offense' | 'add-evidence';
  let viewMode: ViewMode = 'browse';

  // Detail view state
  let selectedOffense: OffenseWithEvidence | null = null;
  let loadingDetail = false;

  // Submit offense form state
  let offenseForm: Partial<CreateOffenseRequest> = {
    category: 'domestic_violence',
    severity: 'moderate',
    title: '',
    description: '',
    incident_date_approximate: true,
    arrested: false,
    charged: false,
    convicted: false,
    settled: false,
  };
  let artistSearchQuery = '';
  let artistSearchResults: { id: string; name: string }[] = [];
  let selectedArtist: { id: string; name: string } | null = null;
  let submittingOffense = false;

  // Add evidence form state
  let evidenceForm: Partial<AddEvidenceRequest> = {
    url: '',
    source_name: '',
    source_type: 'news_article',
    title: '',
    excerpt: '',
    is_primary_source: false,
  };
  let selectedOffenseId: string | null = null;
  let submittingEvidence = false;

  const sourceTypes = [
    { value: 'news_article', label: 'News Article' },
    { value: 'court_document', label: 'Court Document' },
    { value: 'police_report', label: 'Police Report' },
    { value: 'video', label: 'Video Evidence' },
    { value: 'social_media', label: 'Social Media' },
    { value: 'official_statement', label: 'Official Statement' },
    { value: 'other', label: 'Other' },
  ];

  const categories: { value: OffenseCategory; label: string }[] = [
    { value: 'domestic_violence', label: 'Domestic Violence' },
    { value: 'sexual_misconduct', label: 'Sexual Misconduct' },
    { value: 'sexual_assault', label: 'Sexual Assault' },
    { value: 'child_abuse', label: 'Child Abuse' },
    { value: 'hate_speech', label: 'Hate Speech' },
    { value: 'racism', label: 'Racism' },
    { value: 'homophobia', label: 'Homophobia' },
    { value: 'antisemitism', label: 'Antisemitism' },
    { value: 'violent_crime', label: 'Violent Crime' },
    { value: 'drug_trafficking', label: 'Drug Trafficking' },
    { value: 'fraud', label: 'Fraud' },
    { value: 'animal_abuse', label: 'Animal Abuse' },
    { value: 'other', label: 'Other' },
  ];

  const severities: { value: OffenseSeverity; label: string; description: string }[] = [
    { value: 'minor', label: 'Minor', description: 'Controversial statements' },
    { value: 'moderate', label: 'Moderate', description: 'Arrests, credible allegations' },
    { value: 'severe', label: 'Severe', description: 'Convictions, proven abuse' },
    { value: 'egregious', label: 'Egregious', description: 'Multiple severe offenses, ongoing patterns' },
  ];

  onMount(async () => {
    await loadArtists();
  });

  async function loadArtists() {
    loading = true;
    error = null;
    try {
      flaggedArtists = await getFlaggedArtistsDatabase(severityFilter ?? undefined);
    } catch (e) {
      error = 'Failed to load offense database';
    } finally {
      loading = false;
    }
  }

  async function handleSeverityFilter(severity: OffenseSeverity | null) {
    severityFilter = severity;
    await loadArtists();
  }

  async function viewOffenseDetail(offenseId: string) {
    loadingDetail = true;
    selectedOffense = await getOffenseWithEvidence(offenseId);
    viewMode = 'detail';
    loadingDetail = false;
  }

  function openAddEvidenceForm(offenseId: string) {
    selectedOffenseId = offenseId;
    evidenceForm = {
      url: '',
      source_name: '',
      source_type: 'news_article',
      title: '',
      excerpt: '',
      is_primary_source: false,
    };
    viewMode = 'add-evidence';
  }

  function openSubmitOffenseForm() {
    offenseForm = {
      category: 'domestic_violence',
      severity: 'moderate',
      title: '',
      description: '',
      incident_date_approximate: true,
      arrested: false,
      charged: false,
      convicted: false,
      settled: false,
    };
    selectedArtist = null;
    artistSearchQuery = '';
    artistSearchResults = [];
    viewMode = 'submit-offense';
  }

  let searchTimeout: ReturnType<typeof setTimeout>;
  async function handleArtistSearch() {
    clearTimeout(searchTimeout);
    if (artistSearchQuery.length < 2) {
      artistSearchResults = [];
      return;
    }
    searchTimeout = setTimeout(async () => {
      artistSearchResults = await searchArtists(artistSearchQuery);
    }, 300);
  }

  function selectArtist(artist: { id: string; name: string }) {
    selectedArtist = artist;
    artistSearchQuery = artist.name;
    artistSearchResults = [];
  }

  async function submitOffense() {
    if (!selectedArtist || !offenseForm.title || !offenseForm.description) {
      error = 'Please fill in all required fields';
      return;
    }

    submittingOffense = true;
    error = null;

    try {
      const result = await createOffense({
        artist_id: selectedArtist.id,
        category: offenseForm.category as OffenseCategory,
        severity: offenseForm.severity as OffenseSeverity,
        title: offenseForm.title,
        description: offenseForm.description,
        incident_date: offenseForm.incident_date,
        incident_date_approximate: offenseForm.incident_date_approximate,
        arrested: offenseForm.arrested,
        charged: offenseForm.charged,
        convicted: offenseForm.convicted,
        settled: offenseForm.settled,
      });

      if (result) {
        // Offer to add evidence
        selectedOffenseId = result.id;
        viewMode = 'add-evidence';
      } else {
        error = 'Failed to submit offense report';
      }
    } catch (e) {
      error = 'Failed to submit offense report';
    } finally {
      submittingOffense = false;
    }
  }

  async function submitEvidence() {
    if (!selectedOffenseId || !evidenceForm.url) {
      error = 'Please provide at least a URL';
      return;
    }

    submittingEvidence = true;
    error = null;

    try {
      const result = await addEvidence({
        offense_id: selectedOffenseId,
        url: evidenceForm.url,
        source_name: evidenceForm.source_name,
        source_type: evidenceForm.source_type,
        title: evidenceForm.title,
        excerpt: evidenceForm.excerpt,
        published_date: evidenceForm.published_date,
        is_primary_source: evidenceForm.is_primary_source,
      });

      if (result) {
        // Clear form for another evidence
        evidenceForm = {
          url: '',
          source_name: '',
          source_type: 'news_article',
          title: '',
          excerpt: '',
          is_primary_source: false,
        };
        // Show success message
        alert('Evidence submitted successfully! Add more or go back to browse.');
      } else {
        error = 'Failed to submit evidence';
      }
    } catch (e) {
      error = 'Failed to submit evidence';
    } finally {
      submittingEvidence = false;
    }
  }

  function goBack() {
    if (viewMode === 'detail' || viewMode === 'submit-offense') {
      viewMode = 'browse';
      selectedOffense = null;
    } else if (viewMode === 'add-evidence') {
      if (selectedOffense) {
        viewMode = 'detail';
      } else {
        viewMode = 'browse';
      }
      selectedOffenseId = null;
    }
  }
</script>

<div class="min-h-screen bg-gradient-to-b from-amber-50 to-white">
  <!-- Header -->
  <div class="bg-white border-b border-gray-100">
    <div class="max-w-4xl mx-auto px-4 py-8 sm:px-6 lg:px-8">
      <button
        type="button"
        on:click={() => viewMode === 'browse' ? navigateTo('dashboard') : goBack()}
        class="text-gray-500 hover:text-gray-700 mb-4 flex items-center text-sm"
      >
        <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
        </svg>
        {viewMode === 'browse' ? 'Back to Dashboard' : 'Back'}
      </button>
      <h1 class="text-3xl font-bold text-gray-900 mb-2">
        {#if viewMode === 'browse'}
          Offense Database
        {:else if viewMode === 'detail'}
          Offense Details
        {:else if viewMode === 'submit-offense'}
          Report Artist Misconduct
        {:else}
          Add Evidence
        {/if}
      </h1>
      <p class="text-lg text-gray-600">
        {#if viewMode === 'browse'}
          Browse documented cases of artist misconduct with verified evidence.
        {:else if viewMode === 'detail'}
          View offense details and supporting evidence.
        {:else if viewMode === 'submit-offense'}
          Submit a new report with documentation.
        {:else}
          Add supporting evidence to this offense report.
        {/if}
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
            <button type="button" on:click={() => error = null} class="text-red-600 underline text-sm mt-1">Dismiss</button>
          </div>
        </div>
      </div>
    {/if}

    {#if viewMode === 'browse'}
      <!-- Browse view -->
      <div class="space-y-6">
        <!-- Action bar -->
        <div class="flex items-center justify-between flex-wrap gap-4">
          <div class="flex items-center gap-2 flex-wrap">
            <span class="text-sm text-gray-500">Filter by severity:</span>
            <button
              type="button"
              on:click={() => handleSeverityFilter(null)}
              class="px-3 py-1.5 text-sm rounded-lg transition-colors {severityFilter === null ? 'bg-gray-900 text-white' : 'bg-gray-100 text-gray-700 hover:bg-gray-200'}"
            >
              All
            </button>
            {#each severities as sev}
              <button
                type="button"
                on:click={() => handleSeverityFilter(sev.value)}
                class="px-3 py-1.5 text-sm rounded-lg transition-colors {severityFilter === sev.value ? severityConfig[sev.value].color + ' text-white' : 'bg-gray-100 text-gray-700 hover:bg-gray-200'}"
              >
                {sev.label}
              </button>
            {/each}
          </div>

          <button
            type="button"
            on:click={openSubmitOffenseForm}
            class="px-4 py-2 bg-amber-600 text-white text-sm font-medium rounded-lg hover:bg-amber-700 transition-colors flex items-center"
          >
            <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
            Report Misconduct
          </button>
        </div>

        <!-- Artists list -->
        {#if loading}
          <div class="bg-white rounded-2xl shadow-lg border border-gray-100 p-8 text-center">
            <div class="w-8 h-8 border-4 border-amber-600 border-t-transparent rounded-full animate-spin mx-auto"></div>
            <p class="mt-4 text-gray-600">Loading offense database...</p>
          </div>
        {:else if flaggedArtists.length === 0}
          <div class="bg-white rounded-2xl shadow-lg border border-gray-100 p-8 text-center">
            <div class="w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center mx-auto mb-4">
              <svg class="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
            </div>
            <h3 class="text-lg font-semibold text-gray-900 mb-2">No records found</h3>
            <p class="text-gray-600 mb-4">Be the first to document artist misconduct.</p>
            <button
              type="button"
              on:click={openSubmitOffenseForm}
              class="px-4 py-2 bg-amber-600 text-white text-sm font-medium rounded-lg hover:bg-amber-700 transition-colors"
            >
              Submit a Report
            </button>
          </div>
        {:else}
          <div class="space-y-4">
            {#each flaggedArtists as artist}
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
                          {artist.offenses.length} documented offense{artist.offenses.length !== 1 ? 's' : ''}
                        </p>
                      </div>
                    </div>
                    <span class="px-3 py-1 {severityConfig[artist.severity].color} text-white text-xs font-medium rounded-full">
                      {severityConfig[artist.severity].label}
                    </span>
                  </div>

                  <div class="space-y-2">
                    {#each artist.offenses as offense}
                      <div class="bg-gray-50 rounded-xl p-4">
                        <div class="flex items-start justify-between">
                          <div class="flex-1">
                            <span class="text-xs font-medium text-amber-700 uppercase tracking-wide">
                              {categoryLabels[offense.category]}
                            </span>
                            <h5 class="font-medium text-gray-900 mt-1">{offense.title}</h5>
                            <p class="text-sm text-gray-500 mt-1">
                              {offense.date} - {offense.evidence_count} evidence source{offense.evidence_count !== 1 ? 's' : ''}
                            </p>
                          </div>
                          <button
                            type="button"
                            on:click={() => viewOffenseDetail(artist.id)}
                            class="text-sm text-amber-600 hover:text-amber-700 font-medium whitespace-nowrap ml-4"
                          >
                            View Details
                          </button>
                        </div>
                      </div>
                    {/each}
                  </div>
                </div>
              </div>
            {/each}
          </div>
        {/if}

        <!-- Info card -->
        <div class="bg-amber-50 rounded-2xl p-6 mt-8">
          <div class="flex items-start">
            <div class="w-12 h-12 bg-amber-100 rounded-full flex items-center justify-center mr-4 flex-shrink-0">
              <svg class="w-6 h-6 text-amber-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </div>
            <div>
              <h4 class="font-medium text-amber-900 mb-1">About This Database</h4>
              <p class="text-sm text-amber-800">
                All entries require documented evidence from reputable sources. Reports are reviewed
                before being marked as verified. Please submit only factual information with proper citations.
              </p>
            </div>
          </div>
        </div>
      </div>

    {:else if viewMode === 'detail'}
      <!-- Detail view -->
      {#if loadingDetail}
        <div class="bg-white rounded-2xl shadow-lg border border-gray-100 p-8 text-center">
          <div class="w-8 h-8 border-4 border-amber-600 border-t-transparent rounded-full animate-spin mx-auto"></div>
          <p class="mt-4 text-gray-600">Loading offense details...</p>
        </div>
      {:else if selectedOffense}
        <div class="space-y-6">
          <div class="bg-white rounded-2xl shadow-lg border border-gray-100 p-6">
            <div class="flex items-start justify-between mb-6">
              <div>
                <span class="text-xs font-medium text-amber-700 uppercase tracking-wide">
                  {categoryLabels[selectedOffense.offense.category]}
                </span>
                <h2 class="text-xl font-bold text-gray-900 mt-1">{selectedOffense.offense.title}</h2>
                <p class="text-gray-500">Artist: {selectedOffense.artist_name}</p>
              </div>
              <span class="px-3 py-1 {severityConfig[selectedOffense.offense.severity].color} text-white text-xs font-medium rounded-full">
                {severityConfig[selectedOffense.offense.severity].label}
              </span>
            </div>

            <div class="prose prose-sm max-w-none text-gray-700 mb-6">
              <p>{selectedOffense.offense.description}</p>
            </div>

            <div class="flex flex-wrap gap-2 text-sm">
              {#if selectedOffense.offense.incident_date}
                <span class="px-3 py-1 bg-gray-100 text-gray-700 rounded-full">
                  Date: {selectedOffense.offense.incident_date} {selectedOffense.offense.incident_date_approximate ? '(approx)' : ''}
                </span>
              {/if}
              {#if selectedOffense.offense.arrested}
                <span class="px-3 py-1 bg-red-100 text-red-700 rounded-full">Arrested</span>
              {/if}
              {#if selectedOffense.offense.charged}
                <span class="px-3 py-1 bg-red-100 text-red-700 rounded-full">Charged</span>
              {/if}
              {#if selectedOffense.offense.convicted}
                <span class="px-3 py-1 bg-red-100 text-red-700 rounded-full">Convicted</span>
              {/if}
              {#if selectedOffense.offense.settled}
                <span class="px-3 py-1 bg-yellow-100 text-yellow-700 rounded-full">Settled</span>
              {/if}
              <span class="px-3 py-1 bg-gray-100 text-gray-700 rounded-full">
                Status: {selectedOffense.offense.status}
              </span>
            </div>
          </div>

          <!-- Evidence section -->
          <div class="flex items-center justify-between">
            <h3 class="text-lg font-semibold text-gray-900">Evidence ({selectedOffense.evidence.length})</h3>
            <button
              type="button"
              on:click={() => openAddEvidenceForm(selectedOffense?.offense.id ?? '')}
              class="px-4 py-2 bg-amber-600 text-white text-sm font-medium rounded-lg hover:bg-amber-700 transition-colors flex items-center"
            >
              <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
              </svg>
              Add Evidence
            </button>
          </div>

          {#if selectedOffense.evidence.length === 0}
            <div class="bg-gray-50 rounded-xl p-6 text-center">
              <p class="text-gray-600">No evidence has been submitted yet.</p>
              <button
                type="button"
                on:click={() => openAddEvidenceForm(selectedOffense?.offense.id ?? '')}
                class="mt-3 text-amber-600 hover:text-amber-700 font-medium text-sm"
              >
                Be the first to add evidence
              </button>
            </div>
          {:else}
            <div class="space-y-4">
              {#each selectedOffense.evidence as ev}
                <div class="bg-white rounded-xl border border-gray-200 p-4">
                  <div class="flex items-start justify-between">
                    <div class="flex-1">
                      <div class="flex items-center gap-2 mb-2">
                        {#if ev.is_primary_source}
                          <span class="px-2 py-0.5 bg-green-100 text-green-700 text-xs font-medium rounded">Primary Source</span>
                        {/if}
                        {#if ev.source_type}
                          <span class="px-2 py-0.5 bg-gray-100 text-gray-600 text-xs rounded capitalize">{ev.source_type.replace('_', ' ')}</span>
                        {/if}
                      </div>
                      <h5 class="font-medium text-gray-900">{ev.title || ev.source_name || 'Evidence Link'}</h5>
                      {#if ev.excerpt}
                        <p class="text-sm text-gray-600 mt-2 italic">"{ev.excerpt}"</p>
                      {/if}
                      {#if ev.published_date}
                        <p class="text-xs text-gray-500 mt-2">Published: {ev.published_date}</p>
                      {/if}
                    </div>
                    <a
                      href={ev.url}
                      target="_blank"
                      rel="noopener noreferrer"
                      class="text-amber-600 hover:text-amber-700 ml-4"
                    >
                      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                      </svg>
                    </a>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {:else}
        <div class="bg-red-50 rounded-xl p-6 text-center">
          <p class="text-red-800">Failed to load offense details.</p>
        </div>
      {/if}

    {:else if viewMode === 'submit-offense'}
      <!-- Submit offense form -->
      <div class="bg-white rounded-2xl shadow-lg border border-gray-100 p-6">
        <form on:submit|preventDefault={submitOffense} class="space-y-6">
          <!-- Artist search -->
          <div>
            <label for="artist-search" class="block text-sm font-medium text-gray-700 mb-2">
              Artist Name <span class="text-red-500">*</span>
            </label>
            <div class="relative">
              <input
                id="artist-search"
                type="text"
                bind:value={artistSearchQuery}
                on:input={handleArtistSearch}
                placeholder="Search for artist..."
                class="w-full px-4 py-3 border border-gray-200 rounded-xl focus:ring-2 focus:ring-amber-500 focus:border-amber-500"
              />
              {#if artistSearchResults.length > 0}
                <div class="absolute z-10 w-full mt-1 bg-white border border-gray-200 rounded-xl shadow-lg max-h-60 overflow-auto">
                  {#each artistSearchResults as result}
                    <button
                      type="button"
                      on:click={() => selectArtist(result)}
                      class="w-full px-4 py-3 text-left hover:bg-gray-50 border-b border-gray-100 last:border-0"
                    >
                      {result.name}
                    </button>
                  {/each}
                </div>
              {/if}
            </div>
            {#if selectedArtist}
              <p class="text-sm text-green-600 mt-1">Selected: {selectedArtist.name}</p>
            {/if}
          </div>

          <!-- Category -->
          <div>
            <label for="category" class="block text-sm font-medium text-gray-700 mb-2">
              Category <span class="text-red-500">*</span>
            </label>
            <select
              id="category"
              bind:value={offenseForm.category}
              class="w-full px-4 py-3 border border-gray-200 rounded-xl focus:ring-2 focus:ring-amber-500 focus:border-amber-500"
            >
              {#each categories as cat}
                <option value={cat.value}>{cat.label}</option>
              {/each}
            </select>
          </div>

          <!-- Severity -->
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-2">
              Severity <span class="text-red-500">*</span>
            </label>
            <div class="grid grid-cols-2 gap-3">
              {#each severities as sev}
                <label class="relative cursor-pointer">
                  <input
                    type="radio"
                    name="severity"
                    value={sev.value}
                    bind:group={offenseForm.severity}
                    class="sr-only peer"
                  />
                  <div class="p-4 border-2 rounded-xl peer-checked:border-amber-500 peer-checked:bg-amber-50 hover:bg-gray-50 transition-colors">
                    <div class="font-medium text-gray-900">{sev.label}</div>
                    <div class="text-xs text-gray-500 mt-1">{sev.description}</div>
                  </div>
                </label>
              {/each}
            </div>
          </div>

          <!-- Title -->
          <div>
            <label for="title" class="block text-sm font-medium text-gray-700 mb-2">
              Title <span class="text-red-500">*</span>
            </label>
            <input
              id="title"
              type="text"
              bind:value={offenseForm.title}
              placeholder="Brief description of the offense"
              class="w-full px-4 py-3 border border-gray-200 rounded-xl focus:ring-2 focus:ring-amber-500 focus:border-amber-500"
            />
          </div>

          <!-- Description -->
          <div>
            <label for="description" class="block text-sm font-medium text-gray-700 mb-2">
              Description <span class="text-red-500">*</span>
            </label>
            <textarea
              id="description"
              bind:value={offenseForm.description}
              rows="4"
              placeholder="Provide a factual summary of the incident..."
              class="w-full px-4 py-3 border border-gray-200 rounded-xl focus:ring-2 focus:ring-amber-500 focus:border-amber-500"
            ></textarea>
          </div>

          <!-- Incident date -->
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label for="incident-date" class="block text-sm font-medium text-gray-700 mb-2">
                Incident Date
              </label>
              <input
                id="incident-date"
                type="date"
                bind:value={offenseForm.incident_date}
                class="w-full px-4 py-3 border border-gray-200 rounded-xl focus:ring-2 focus:ring-amber-500 focus:border-amber-500"
              />
            </div>
            <div class="flex items-end">
              <label class="flex items-center">
                <input
                  type="checkbox"
                  bind:checked={offenseForm.incident_date_approximate}
                  class="w-4 h-4 text-amber-600 border-gray-300 rounded focus:ring-amber-500"
                />
                <span class="ml-2 text-sm text-gray-600">Date is approximate</span>
              </label>
            </div>
          </div>

          <!-- Legal status checkboxes -->
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-2">Legal Status</label>
            <div class="flex flex-wrap gap-4">
              <label class="flex items-center">
                <input type="checkbox" bind:checked={offenseForm.arrested} class="w-4 h-4 text-amber-600 border-gray-300 rounded focus:ring-amber-500" />
                <span class="ml-2 text-sm text-gray-600">Arrested</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={offenseForm.charged} class="w-4 h-4 text-amber-600 border-gray-300 rounded focus:ring-amber-500" />
                <span class="ml-2 text-sm text-gray-600">Charged</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={offenseForm.convicted} class="w-4 h-4 text-amber-600 border-gray-300 rounded focus:ring-amber-500" />
                <span class="ml-2 text-sm text-gray-600">Convicted</span>
              </label>
              <label class="flex items-center">
                <input type="checkbox" bind:checked={offenseForm.settled} class="w-4 h-4 text-amber-600 border-gray-300 rounded focus:ring-amber-500" />
                <span class="ml-2 text-sm text-gray-600">Settled</span>
              </label>
            </div>
          </div>

          <button
            type="submit"
            disabled={submittingOffense || !selectedArtist}
            class="w-full py-3 bg-amber-600 text-white font-medium rounded-xl hover:bg-amber-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {submittingOffense ? 'Submitting...' : 'Submit Report & Add Evidence'}
          </button>
        </form>
      </div>

    {:else if viewMode === 'add-evidence'}
      <!-- Add evidence form -->
      <div class="bg-white rounded-2xl shadow-lg border border-gray-100 p-6">
        <div class="mb-6 p-4 bg-amber-50 rounded-xl">
          <p class="text-sm text-amber-800">
            Evidence should be from reputable sources: news articles, court documents, police reports, or official statements.
          </p>
        </div>

        <form on:submit|preventDefault={submitEvidence} class="space-y-6">
          <!-- URL -->
          <div>
            <label for="evidence-url" class="block text-sm font-medium text-gray-700 mb-2">
              Source URL <span class="text-red-500">*</span>
            </label>
            <input
              id="evidence-url"
              type="url"
              bind:value={evidenceForm.url}
              placeholder="https://..."
              class="w-full px-4 py-3 border border-gray-200 rounded-xl focus:ring-2 focus:ring-amber-500 focus:border-amber-500"
            />
          </div>

          <!-- Source name -->
          <div>
            <label for="source-name" class="block text-sm font-medium text-gray-700 mb-2">
              Source Name
            </label>
            <input
              id="source-name"
              type="text"
              bind:value={evidenceForm.source_name}
              placeholder="e.g., New York Times, AP News"
              class="w-full px-4 py-3 border border-gray-200 rounded-xl focus:ring-2 focus:ring-amber-500 focus:border-amber-500"
            />
          </div>

          <!-- Source type -->
          <div>
            <label for="source-type" class="block text-sm font-medium text-gray-700 mb-2">
              Source Type
            </label>
            <select
              id="source-type"
              bind:value={evidenceForm.source_type}
              class="w-full px-4 py-3 border border-gray-200 rounded-xl focus:ring-2 focus:ring-amber-500 focus:border-amber-500"
            >
              {#each sourceTypes as st}
                <option value={st.value}>{st.label}</option>
              {/each}
            </select>
          </div>

          <!-- Title -->
          <div>
            <label for="evidence-title" class="block text-sm font-medium text-gray-700 mb-2">
              Article/Document Title
            </label>
            <input
              id="evidence-title"
              type="text"
              bind:value={evidenceForm.title}
              placeholder="Title of the article or document"
              class="w-full px-4 py-3 border border-gray-200 rounded-xl focus:ring-2 focus:ring-amber-500 focus:border-amber-500"
            />
          </div>

          <!-- Excerpt -->
          <div>
            <label for="excerpt" class="block text-sm font-medium text-gray-700 mb-2">
              Key Excerpt
            </label>
            <textarea
              id="excerpt"
              bind:value={evidenceForm.excerpt}
              rows="3"
              placeholder="Quote a relevant passage from the source..."
              class="w-full px-4 py-3 border border-gray-200 rounded-xl focus:ring-2 focus:ring-amber-500 focus:border-amber-500"
            ></textarea>
          </div>

          <!-- Published date -->
          <div>
            <label for="published-date" class="block text-sm font-medium text-gray-700 mb-2">
              Published Date
            </label>
            <input
              id="published-date"
              type="date"
              bind:value={evidenceForm.published_date}
              class="w-full px-4 py-3 border border-gray-200 rounded-xl focus:ring-2 focus:ring-amber-500 focus:border-amber-500"
            />
          </div>

          <!-- Primary source -->
          <div>
            <label class="flex items-center">
              <input
                type="checkbox"
                bind:checked={evidenceForm.is_primary_source}
                class="w-4 h-4 text-amber-600 border-gray-300 rounded focus:ring-amber-500"
              />
              <span class="ml-2 text-sm text-gray-600">This is a primary source (court document, police report, direct video)</span>
            </label>
          </div>

          <div class="flex gap-3">
            <button
              type="submit"
              disabled={submittingEvidence || !evidenceForm.url}
              class="flex-1 py-3 bg-amber-600 text-white font-medium rounded-xl hover:bg-amber-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {submittingEvidence ? 'Submitting...' : 'Submit Evidence'}
            </button>
            <button
              type="button"
              on:click={goBack}
              class="px-6 py-3 border border-gray-200 text-gray-700 font-medium rounded-xl hover:bg-gray-50 transition-colors"
            >
              Done
            </button>
          </div>
        </form>
      </div>
    {/if}
  </div>
</div>
