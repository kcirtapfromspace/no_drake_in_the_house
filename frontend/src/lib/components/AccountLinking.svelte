<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { currentUser } from '../stores/auth';
  import { api } from '../utils/api';
  
  const dispatch = createEventDispatcher();
  
  export let isVisible = false;
  
  let linkedAccounts: LinkedAccount[] = [];
  let availableProviders: string[] = ['google', 'apple', 'github'];
  let isLoading = false;
  let error = '';
  let success = '';
  let linkingProvider: string | null = null;

  interface LinkedAccount {
    provider: string;
    provider_user_id: string;
    email?: string;
    display_name?: string;
    avatar_url?: string;
    linked_at: string;
  }

  interface LinkAccountResponse {
    authorization_url: string;
    state: string;
  }

  onMount(() => {
    if (isVisible) {
      loadLinkedAccounts();
    }
  });

  $: if (isVisible) {
    loadLinkedAccounts();
  }

  async function loadLinkedAccounts() {
    if (!$currentUser) return;
    
    isLoading = true;
    error = '';
    
    try {
      const result = await api.get<LinkedAccount[]>('/auth/oauth/accounts');
      
      if (result.success) {
        linkedAccounts = result.data;
      } else {
        error = result.message || 'Failed to load linked accounts';
      }
    } catch (err: any) {
      error = err.message || 'Failed to load linked accounts';
    } finally {
      isLoading = false;
    }
  }

  async function linkAccount(provider: string) {
    if (linkingProvider) return;
    
    linkingProvider = provider;
    error = '';
    success = '';
    
    try {
      const result = await api.post<LinkAccountResponse>(`/auth/oauth/${provider}/link`);
      
      if (result.success) {
        // Store state for validation when callback returns
        sessionStorage.setItem(`oauth_link_state_${provider}`, result.data.state);
        
        // Open OAuth flow in popup window
        const popup = window.open(
          result.data.authorization_url,
          `link_${provider}`,
          'width=500,height=600,scrollbars=yes,resizable=yes'
        );
        
        // Listen for popup completion
        const checkClosed = setInterval(() => {
          if (popup?.closed) {
            clearInterval(checkClosed);
            linkingProvider = null;
            
            // Check if linking was successful by reloading accounts
            setTimeout(() => {
              loadLinkedAccounts();
            }, 1000);
          }
        }, 1000);
        
      } else {
        error = result.message || `Failed to initiate ${provider} linking`;
        linkingProvider = null;
      }
    } catch (err: any) {
      error = err.message || `Network error during ${provider} linking`;
      linkingProvider = null;
    }
  }

  async function unlinkAccount(provider: string) {
    if (!confirm(`Are you sure you want to unlink your ${getProviderName(provider)} account?`)) {
      return;
    }
    
    isLoading = true;
    error = '';
    success = '';
    
    try {
      const result = await api.delete(`/auth/oauth/${provider}/unlink`);
      
      if (result.success) {
        success = `Successfully unlinked ${getProviderName(provider)} account`;
        await loadLinkedAccounts();
      } else {
        error = result.message || `Failed to unlink ${provider} account`;
      }
    } catch (err: any) {
      error = err.message || `Failed to unlink ${provider} account`;
    } finally {
      isLoading = false;
    }
  }

  function getProviderName(provider: string): string {
    switch (provider) {
      case 'google':
        return 'Google';
      case 'apple':
        return 'Apple';
      case 'github':
        return 'GitHub';
      default:
        return provider;
    }
  }

  function getProviderIcon(provider: string): string {
    switch (provider) {
      case 'google':
        return `<svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
          <path d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z" fill="#4285F4"/>
          <path d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z" fill="#34A853"/>
          <path d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z" fill="#FBBC05"/>
          <path d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z" fill="#EA4335"/>
        </svg>`;
      case 'apple':
        return `<svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
          <path d="M18.71 19.5c-.83 1.24-1.71 2.45-3.05 2.47-1.34.03-1.77-.79-3.29-.79-1.53 0-2 .77-3.27.82-1.31.05-2.3-1.32-3.14-2.53C4.25 17 2.94 12.45 4.7 9.39c.87-1.52 2.43-2.48 4.12-2.51 1.28-.02 2.5.87 3.29.87.78 0 2.26-1.07 3.81-.91.65.03 2.47.26 3.64 1.98-.09.06-2.17 1.28-2.15 3.81.03 3.02 2.65 4.03 2.68 4.04-.03.07-.42 1.44-1.38 2.83M13 3.5c.73-.83 1.94-1.46 2.94-1.5.13 1.17-.34 2.35-1.04 3.19-.69.85-1.83 1.51-2.95 1.42-.15-1.15.41-2.35 1.05-3.11z"/>
        </svg>`;
      case 'github':
        return `<svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
        </svg>`;
      default:
        return '';
    }
  }

  function isProviderLinked(provider: string): boolean {
    return linkedAccounts.some(account => account.provider === provider);
  }

  function getLinkedAccount(provider: string): LinkedAccount | undefined {
    return linkedAccounts.find(account => account.provider === provider);
  }

  function closeModal() {
    dispatch('close');
  }

  function formatDate(dateString: string): string {
    return new Date(dateString).toLocaleDateString();
  }
</script>

{#if isVisible}
  <!-- Modal backdrop -->
  <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
  <div class="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50" on:click={closeModal}>
    <!-- Modal content -->
    <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
    <div class="relative top-20 mx-auto p-5 border w-11/12 md:w-3/4 lg:w-1/2 xl:w-2/5 shadow-lg rounded-md bg-white" on:click|stopPropagation>
      <!-- Modal header -->
      <div class="flex items-center justify-between pb-4 border-b">
        <h3 class="text-lg font-semibold text-gray-900">
          Linked Accounts
        </h3>
        <button
          type="button"
          on:click={closeModal}
          class="text-gray-400 hover:text-gray-600 focus:outline-none"
        >
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
          </svg>
        </button>
      </div>

      <!-- Modal body -->
      <div class="mt-4">
        <p class="text-sm text-gray-600 mb-6">
          Link your social accounts to sign in more easily and keep your profile information up to date.
        </p>

        {#if error}
          <div class="mb-4 p-3 bg-red-50 border border-red-200 rounded-md">
            <div class="flex">
              <div class="flex-shrink-0">
                <svg class="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
                  <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
                </svg>
              </div>
              <div class="ml-3">
                <p class="text-sm text-red-800">{error}</p>
              </div>
            </div>
          </div>
        {/if}

        {#if success}
          <div class="mb-4 p-3 bg-green-50 border border-green-200 rounded-md">
            <div class="flex">
              <div class="flex-shrink-0">
                <svg class="h-5 w-5 text-green-400" viewBox="0 0 20 20" fill="currentColor">
                  <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
                </svg>
              </div>
              <div class="ml-3">
                <p class="text-sm text-green-800">{success}</p>
              </div>
            </div>
          </div>
        {/if}

        {#if isLoading}
          <div class="flex justify-center py-4">
            <svg class="animate-spin h-6 w-6 text-blue-600" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
          </div>
        {:else}
          <div class="space-y-4">
            {#each availableProviders as provider}
              {@const linked = isProviderLinked(provider)}
              {@const account = getLinkedAccount(provider)}
              
              <div class="flex items-center justify-between p-4 border border-gray-200 rounded-lg">
                <div class="flex items-center space-x-3">
                  <div class="flex-shrink-0">
                    {@html getProviderIcon(provider)}
                  </div>
                  <div>
                    <h4 class="text-sm font-medium text-gray-900">
                      {getProviderName(provider)}
                    </h4>
                    {#if linked && account}
                      <p class="text-sm text-gray-500">
                        {account.display_name || account.email || 'Connected'}
                      </p>
                      <p class="text-xs text-gray-400">
                        Linked on {formatDate(account.linked_at)}
                      </p>
                    {:else}
                      <p class="text-sm text-gray-500">
                        Not connected
                      </p>
                    {/if}
                  </div>
                </div>
                
                <div class="flex-shrink-0">
                  {#if linked}
                    <button
                      type="button"
                      on:click={() => unlinkAccount(provider)}
                      disabled={isLoading}
                      class="px-3 py-1 text-sm text-red-600 hover:text-red-800 border border-red-300 hover:border-red-400 rounded-md transition-colors duration-200 disabled:opacity-50"
                    >
                      Unlink
                    </button>
                  {:else}
                    <button
                      type="button"
                      on:click={() => linkAccount(provider)}
                      disabled={isLoading || linkingProvider === provider}
                      class="px-3 py-1 text-sm text-blue-600 hover:text-blue-800 border border-blue-300 hover:border-blue-400 rounded-md transition-colors duration-200 disabled:opacity-50"
                    >
                      {#if linkingProvider === provider}
                        Linking...
                      {:else}
                        Link
                      {/if}
                    </button>
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Modal footer -->
      <div class="flex justify-end pt-4 border-t mt-6">
        <button
          type="button"
          on:click={closeModal}
          class="px-4 py-2 text-sm font-medium text-gray-700 bg-gray-100 hover:bg-gray-200 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
        >
          Close
        </button>
      </div>
    </div>
  </div>
{/if}