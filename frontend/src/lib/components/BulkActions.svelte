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

<div class="bg-indigo-50 border border-indigo-200 rounded-md p-4">
  <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between space-y-3 sm:space-y-0">
    <div class="flex items-center">
      <svg class="h-5 w-5 text-indigo-400 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
      <p class="ml-3 text-sm text-indigo-800">
        <span class="font-medium">{selectedCount}</span>
        {selectedCount === 1 ? 'artist' : 'artists'} selected
      </p>
    </div>
    
    <div class="flex flex-col sm:flex-row items-stretch sm:items-center space-y-2 sm:space-y-0 sm:space-x-3">
      <button
        on:click={handleClearSelection}
        class="text-sm text-indigo-600 hover:text-indigo-500 text-center sm:text-left"
      >
        Clear selection
      </button>
      
      <button
        on:click={handleBulkDelete}
        class="inline-flex items-center justify-center px-3 py-2 border border-transparent text-sm leading-4 font-medium rounded-md text-red-700 bg-red-100 hover:bg-red-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500"
      >
        <svg class="-ml-0.5 mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
        </svg>
        Remove selected
      </button>
    </div>
  </div>
</div>