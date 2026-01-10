<script lang="ts">
  export let provider: string;
  export let displayName: string;
  export let description: string;
  export let status: 'connected' | 'disconnected' | 'error' | 'connecting';
  export let lastConnected: string | undefined = undefined;
  export let errorMessage: string | undefined = undefined;
  export let onConnect: () => void;
  export let onDisconnect: () => void;
  
  function getStatusColor(status: string) {
    switch (status) {
      case 'connected': return 'text-green-400 bg-green-900/50';
      case 'connecting': return 'text-blue-400 bg-blue-900/50';
      case 'error': return 'text-red-400 bg-red-900/50';
      default: return 'text-zinc-400 bg-zinc-700';
    }
  }
  
  function getStatusText(status: string) {
    switch (status) {
      case 'connected': return 'Connected';
      case 'connecting': return 'Connecting...';
      case 'error': return 'Error';
      default: return 'Not Connected';
    }
  }
  
  function getProviderIcon(provider: string) {
    switch (provider) {
      case 'spotify':
        return `<svg class="w-8 h-8" fill="currentColor" viewBox="0 0 24 24">
          <path d="M12 0C5.4 0 0 5.4 0 12s5.4 12 12 12 12-5.4 12-12S18.66 0 12 0zm5.521 17.34c-.24.359-.66.48-1.021.24-2.82-1.74-6.36-2.101-10.561-1.141-.418.122-.779-.179-.899-.539-.12-.421.18-.78.54-.9 4.56-1.021 8.52-.6 11.64 1.32.42.18.479.659.301 1.02zm1.44-3.3c-.301.42-.841.6-1.262.3-3.239-1.98-8.159-2.58-11.939-1.38-.479.12-1.02-.12-1.14-.6-.12-.48.12-1.021.6-1.141C9.6 9.9 15 10.561 18.72 12.84c.361.181.54.78.241 1.2zm.12-3.36C15.24 8.4 8.82 8.16 5.16 9.301c-.6.179-1.2-.181-1.38-.721-.18-.601.18-1.2.72-1.381 4.26-1.26 11.28-1.02 15.721 1.621.539.3.719 1.02.42 1.56-.299.421-1.02.599-1.559.3z"/>
        </svg>`;
      case 'apple':
        return `<svg class="w-8 h-8" fill="currentColor" viewBox="0 0 24 24">
          <path d="M18.71 19.5c-.83 1.24-1.71 2.45-3.05 2.47-1.34.03-1.77-.79-3.29-.79-1.53 0-2 .77-3.27.82-1.31.05-2.3-1.32-3.14-2.53C4.25 17 2.94 12.45 4.7 9.39c.87-1.52 2.43-2.48 4.12-2.51 1.28-.02 2.5.87 3.29.87.78 0 2.26-1.07 3.81-.91.65.03 2.47.26 3.64 1.98-.09.06-2.17 1.28-2.15 3.81.03 3.02 2.65 4.03 2.68 4.04-.03.07-.42 1.44-1.38 2.83M13 3.5c.73-.83 1.94-1.46 2.94-1.5.13 1.17-.34 2.35-1.04 3.19-.69.85-1.83 1.51-2.95 1.42-.15-1.15.41-2.35 1.05-3.11z"/>
        </svg>`;
      case 'youtube':
        return `<svg class="w-8 h-8" fill="currentColor" viewBox="0 0 24 24">
          <path d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z"/>
        </svg>`;
      default:
        return `<svg class="w-8 h-8" fill="currentColor" viewBox="0 0 24 24">
          <path d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1"/>
        </svg>`;
    }
  }
  
  function formatDate(dateString: string) {
    return new Date(dateString).toLocaleDateString();
  }
</script>

<div class="rounded-xl shadow-sm p-6 hover:shadow-md transition-shadow duration-200" style="background: #27272a; border: 2px solid #52525b;">
  <!-- Header with icon and status -->
  <div class="flex items-start justify-between mb-4">
    <div class="flex items-center">
      <div class="text-zinc-300 mr-3">
        {@html getProviderIcon(provider)}
      </div>
      <div>
        <h3 class="text-lg font-semibold text-white">{displayName}</h3>
        <p class="text-sm text-zinc-400">{description}</p>
      </div>
    </div>
    
    <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {getStatusColor(status)}">
      {getStatusText(status)}
    </span>
  </div>
  
  <!-- Connection details -->
  {#if status === 'connected' && lastConnected}
    <div class="mb-4 text-sm text-zinc-400">
      Connected on {formatDate(lastConnected)}
    </div>
  {/if}

  {#if status === 'error' && errorMessage}
    <div class="mb-4 p-3 bg-red-900/30 rounded-lg" style="border: 2px solid #52525b;">
      <p class="text-sm text-red-400">{errorMessage}</p>
    </div>
  {/if}
  
  <!-- Actions -->
  <div class="flex space-x-3">
    {#if status === 'connected'}
      <button
        on:click={onDisconnect}
        class="flex-1 bg-red-600 text-white px-4 py-2 rounded-lg text-sm font-medium hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500 transition-colors duration-200"
      >
        Disconnect
      </button>
      <button
        on:click={onConnect}
        class="px-4 py-2 text-zinc-300 rounded-lg text-sm font-medium hover:bg-zinc-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 transition-colors duration-200"
        style="border: 2px solid #52525b;"
      >
        Refresh
      </button>
    {:else if status === 'connecting'}
      <button
        disabled
        class="flex-1 bg-blue-600 text-white px-4 py-2 rounded-lg text-sm font-medium opacity-75 cursor-not-allowed flex items-center justify-center"
      >
        <svg class="animate-spin -ml-1 mr-2 h-4 w-4 text-white" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
        Connecting...
      </button>
    {:else}
      <button
        on:click={onConnect}
        class="flex-1 bg-indigo-600 text-white px-4 py-2 rounded-lg text-sm font-medium hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 transition-colors duration-200"
      >
        Connect {displayName}
      </button>
      {#if status === 'error'}
        <button
          on:click={onConnect}
          class="px-4 py-2 text-zinc-300 rounded-lg text-sm font-medium hover:bg-zinc-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 transition-colors duration-200"
          style="border: 2px solid #52525b;"
        >
          Retry
        </button>
      {/if}
    {/if}
  </div>
</div>