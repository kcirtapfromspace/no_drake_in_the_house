<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  
  const dispatch = createEventDispatcher();
  
  export let selectedCount = 0;
  
  function handleBulkDelete() {
    dispatch('bulkDelete');
  }
  
  function handleClearSelection() {
    dispatch('clearSelection');
  }
</script>

<div class="rounded-xl p-4" style="background: #27272a; border: 2px solid #52525b;">
  <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between space-y-3 sm:space-y-0">
    <div class="flex items-center">
      <svg aria-hidden="true" class="icon-uswds icon-uswds--md text-zinc-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
      <p class="ml-3 text-zinc-300">
        <span class="font-medium">{selectedCount}</span>
        {selectedCount === 1 ? 'artist' : 'artists'} selected
      </p>
    </div>
    
    <div class="flex flex-col sm:flex-row items-stretch sm:items-center space-y-2 sm:space-y-0 sm:space-x-3">
      <button
        on:click={handleClearSelection}
        class="text-zinc-400 text-primary hover:text-indigo-500 text-center sm:text-left"
      >
        Clear selection
      </button>
      
      <button
        on:click={handleBulkDelete}
        class="flex items-center justify-center px-3 py-2 border border-transparent text-zinc-400 leading-4 font-medium rounded-uswds-md text-zinc-400 bg-red-100 hover:bg-red-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500"
      >
        <svg aria-hidden="true" class="-ml-0.5 mr-2 icon-uswds icon-uswds--sm" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
        </svg>
        Remove selected
      </button>
    </div>
  </div>
</div>