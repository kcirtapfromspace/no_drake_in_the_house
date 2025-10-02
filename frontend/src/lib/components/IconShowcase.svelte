<!--
  Icon Showcase Component
  
  This component demonstrates the proper usage of the design system icon classes
  and serves as a reference for developers.
-->

<script lang="ts">
  import { onMount } from 'svelte';
  import { validateIconSizes, generateIconUsageReport } from '../utils/icon-validation';
  
  let showReport = false;
  let iconReport: any = null;
  
  onMount(() => {
    // Validate icons when component mounts (development only)
    if (import.meta.env.DEV) {
      validateIconSizes();
    }
  });
  
  function toggleReport() {
    showReport = !showReport;
    if (showReport) {
      iconReport = generateIconUsageReport();
    }
  }
</script>

<div class="container space-y-8 py-8">
  <div class="space-y-4">
    <h1 class="text-uswds-3xl font-bold text-uswds-base-darker">Design System Icons</h1>
    <p class="text-uswds-base-darker">
      This showcase demonstrates the proper usage of icon classes in our design system.
      All icons are constrained to prevent oversizing issues.
    </p>
  </div>

  <!-- Icon Sizes -->
  <section class="space-y-4">
    <h2 class="text-uswds-2xl font-semibold text-uswds-base-darker">Icon Sizes</h2>
    <div class="grid grid-cols-1 md:grid-cols-5 gap-uswds-6">
      
      <div class="text-center space-y-2">
        <div class="flex justify-center">
          <svg class="icon-uswds icon-uswds--xs text-primary-600" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
        </div>
        <div class="text-uswds-sm">
          <code class="bg-uswds-base-lightest px-2 py-1 rounded">icon--xs</code>
          <p class="text-uswds-base-darker mt-1">12px - Inline text</p>
        </div>
      </div>

      <div class="text-center space-y-2">
        <div class="flex justify-center">
          <svg class="icon-uswds icon-uswds--sm text-primary-600" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
        </div>
        <div class="text-uswds-sm">
          <code class="bg-uswds-base-lightest px-2 py-1 rounded">icon--sm</code>
          <p class="text-uswds-base-darker mt-1">16px - Standard UI</p>
        </div>
      </div>

      <div class="text-center space-y-2">
        <div class="flex justify-center">
          <svg class="icon-uswds icon-uswds--md text-primary-600" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
        </div>
        <div class="text-uswds-sm">
          <code class="bg-uswds-base-lightest px-2 py-1 rounded">icon--md</code>
          <p class="text-uswds-base-darker mt-1">20px - Emphasized</p>
        </div>
      </div>

      <div class="text-center space-y-2">
        <div class="flex justify-center">
          <svg class="icon-uswds icon-uswds--lg text-primary-600" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
        </div>
        <div class="text-uswds-sm">
          <code class="bg-uswds-base-lightest px-2 py-1 rounded">icon--lg</code>
          <p class="text-uswds-base-darker mt-1">24px - Headers</p>
        </div>
      </div>

      <div class="text-center space-y-2">
        <div class="flex justify-center">
          <svg class="icon-uswds icon-uswds--xl text-primary-600" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
        </div>
        <div class="text-uswds-sm">
          <code class="bg-uswds-base-lightest px-2 py-1 rounded">icon--xl</code>
          <p class="text-uswds-base-darker mt-1">32px - Hero (max)</p>
        </div>
      </div>
    </div>
  </section>

  <!-- Semantic Colors -->
  <section class="space-y-4">
    <h2 class="text-uswds-2xl font-semibold text-uswds-base-darker">Semantic Colors</h2>
    <div class="grid grid-cols-2 md:grid-cols-5 gap-uswds-4">
      
      <div class="flex items-center space-x-2">
        <svg class="icon-uswds icon-uswds--md icon-uswds--success" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
        </svg>
        <code class="text-uswds-sm">icon--success</code>
      </div>

      <div class="flex items-center space-x-2">
        <svg class="icon-uswds icon-uswds--md icon-uswds--error" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
        <code class="text-uswds-sm">icon--error</code>
      </div>

      <div class="flex items-center space-x-2">
        <svg class="icon-uswds icon-uswds--md icon-uswds--warning" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z" />
        </svg>
        <code class="text-uswds-sm">icon--warning</code>
      </div>

      <div class="flex items-center space-x-2">
        <svg class="icon-uswds icon-uswds--md icon-uswds--neutral" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        <code class="text-uswds-sm">icon--neutral</code>
      </div>

      <div class="flex items-center space-x-2">
        <svg class="icon-uswds icon-uswds--md icon-uswds--primary" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" />
        </svg>
        <code class="text-uswds-sm">icon--primary</code>
      </div>
    </div>
  </section>

  <!-- Context Examples -->
  <section class="space-y-4">
    <h2 class="text-uswds-2xl font-semibold text-uswds-base-darker">Context Examples</h2>
    
    <div class="space-y-6">
      <!-- Button with icon -->
      <div class="space-y-2">
        <h3 class="text-uswds-lg font-medium">Button Icons</h3>
        <button class="btn-uswds btn-uswds-primary">
          <svg class="btn-icon" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          Add Item
        </button>
      </div>

      <!-- Form field with icon -->
      <div class="space-y-2">
        <h3 class="text-uswds-lg font-medium">Form Field Icons</h3>
        <div class="relative">
          <input type="text" class="form-input-uswds pl-10" placeholder="Search..." />
          <div class="absolute inset-y-0 left-0 pl-3 flex items-center">
            <svg class="field-icon" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
          </div>
        </div>
      </div>

      <!-- Loading spinner -->
      <div class="space-y-2">
        <h3 class="text-uswds-lg font-medium">Loading Spinner</h3>
        <div class="flex items-center space-x-2">
          <svg class="loading-icon text-primary-600" aria-hidden="true" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          <span class="text-uswds-base-darker">Loading...</span>
        </div>
      </div>

      <!-- Avatar examples -->
      <div class="space-y-2">
        <h3 class="text-uswds-lg font-medium">Avatars (Not Icons)</h3>
        <div class="flex items-center space-x-4">
          <div class="avatar avatar--sm">
            <div class="avatar__placeholder">
              <svg class="icon-uswds icon-uswds--sm text-uswds-base-darker" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
              </svg>
            </div>
          </div>
          <div class="avatar avatar--md">
            <div class="avatar__placeholder">
              <svg class="icon-uswds icon-uswds--md text-uswds-base-darker" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
              </svg>
            </div>
          </div>
          <div class="avatar avatar--lg">
            <div class="avatar__placeholder">
              <svg class="icon-uswds icon-uswds--lg text-uswds-base-darker" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
              </svg>
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>

  <!-- Development Tools -->
  {#if import.meta.env.DEV}
    <section class="space-y-4 border-t pt-8">
      <h2 class="text-uswds-2xl font-semibold text-uswds-base-darker">Development Tools</h2>
      
      <div class="space-y-4">
        <button 
          class="btn-uswds btn-uswds-secondary"
          on:click={toggleReport}
        >
          <svg class="btn-icon" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
          </svg>
          {showReport ? 'Hide' : 'Show'} Icon Report
        </button>

        {#if showReport && iconReport}
          <div class="bg-uswds-base-lightest p-uswds-4 rounded-uswds-lg space-y-2">
            <h3 class="font-medium">Icon Usage Report</h3>
            <div class="grid grid-cols-2 md:grid-cols-4 gap-uswds-4 text-uswds-sm">
              <div>
                <span class="text-uswds-base-darker">Total Icons:</span>
                <span class="font-medium">{iconReport.totalIcons}</span>
              </div>
              <div>
                <span class="text-uswds-base-darker">Properly Classified:</span>
                <span class="font-medium text-uswds-green-50">{iconReport.properlyClassified}</span>
              </div>
              <div>
                <span class="text-uswds-base-darker">Oversized:</span>
                <span class="font-medium text-uswds-red-50">{iconReport.oversized}</span>
              </div>
              <div>
                <span class="text-uswds-base-darker">Missing ARIA:</span>
                <span class="font-medium text-warning">{iconReport.missingAria}</span>
              </div>
            </div>
            
            {#if iconReport.suggestions.length > 0}
              <div class="mt-4">
                <h4 class="font-medium text-uswds-sm mb-2">Suggestions:</h4>
                <ul class="text-uswds-sm text-uswds-base-darker space-y-1">
                  {#each iconReport.suggestions as suggestion}
                    <li>• {suggestion}</li>
                  {/each}
                </ul>
              </div>
            {/if}
          </div>
        {/if}
      </div>
    </section>
  {/if}
</div>

<style>
  /* Component-specific styles if needed */
</style>