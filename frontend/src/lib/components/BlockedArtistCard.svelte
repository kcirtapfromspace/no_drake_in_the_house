<script lang="ts">
  export let artist: {
    id: string;
    canonical_name: string;
    metadata?: {
      image?: string;
      genres?: string[];
    };
  };
  export let blockedAt: string;
  export let tags: string[] = [];
  export let note: string | undefined = undefined;
  export let onUnblock: () => void;
  
  let isUnblocking = false;
  
  async function handleUnblock() {
    isUnblocking = true;
    try {
      await onUnblock();
    } finally {
      isUnblocking = false;
    }
  }
  
  function formatDate(dateString: string) {
    const date = new Date(dateString);
    const now = new Date();
    const diffTime = Math.abs(now.getTime() - date.getTime());
    const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24));
    
    if (diffDays === 1) return 'Yesterday';
    if (diffDays < 7) return `${diffDays} days ago`;
    if (diffDays < 30) return `${Math.ceil(diffDays / 7)} weeks ago`;
    return date.toLocaleDateString();
  }
</script>

<div class="bg-white rounded-lg shadow-sm border border-gray-200 p-4 hover:shadow-md transition-shadow duration-200">
  <div class="flex items-start space-x-4">
    <!-- Artist image -->
    <div class="flex-shrink-0">
      {#if artist.metadata?.image}
        <img 
          src={artist.metadata.image} 
          alt={artist.canonical_name}
          class="w-16 h-16 rounded-lg object-cover"
        />
      {:else}
        <div class="w-16 h-16 rounded-lg bg-gray-200 flex items-center justify-center">
          <svg class="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3" />
          </svg>
        </div>
      {/if}
    </div>
    
    <!-- Artist info -->
    <div class="flex-1 min-w-0">
      <div class="flex items-start justify-between">
        <div class="flex-1">
          <h3 class="text-lg font-semibold text-gray-900 truncate">
            {artist.canonical_name}
          </h3>
          
          <!-- Genres -->
          {#if artist.metadata?.genres && artist.metadata.genres.length > 0}
            <p class="text-sm text-gray-600 mt-1">
              {artist.metadata.genres.slice(0, 3).join(', ')}
            </p>
          {/if}
          
          <!-- Blocked date -->
          <p class="text-xs text-gray-500 mt-2">
            Blocked {formatDate(blockedAt)}
          </p>
        </div>
        
        <!-- Unblock button -->
        <button
          on:click={handleUnblock}
          disabled={isUnblocking}
          class="ml-4 px-3 py-1.5 text-sm font-medium text-red-600 bg-red-50 border border-red-200 rounded-md hover:bg-red-100 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500 transition-colors duration-200 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {#if isUnblocking}
            <svg class="animate-spin w-4 h-4" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
          {:else}
            Unblock
          {/if}
        </button>
      </div>
      
      <!-- Tags -->
      {#if tags && tags.length > 0}
        <div class="flex flex-wrap gap-1 mt-3">
          {#each tags as tag}
            <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-800">
              {tag}
            </span>
          {/each}
        </div>
      {/if}
      
      <!-- Note -->
      {#if note}
        <p class="text-sm text-gray-600 mt-2 italic">
          "{note}"
        </p>
      {/if}
    </div>
  </div>
</div>