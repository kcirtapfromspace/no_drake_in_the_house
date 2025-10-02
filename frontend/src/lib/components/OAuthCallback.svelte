<script lang="ts">
  import { onMount } from 'svelte';
  import { authActions } from '../stores/auth';
  import { api } from '../utils/api';
  
  export let provider: string = '';
  
  let isProcessing = true;
  let error = '';
  let success = false;

  interface OAuthCallbackResponse {
    access_token: string;
    refresh_token: string;
    user: any;
  }

  onMount(async () => {
    await handleOAuthCallback();
  });

  async function handleOAuthCallback() {
    try {
      // Get URL parameters
      const urlParams = new URLSearchParams(window.location.search);
      const code = urlParams.get('code');
      const state = urlParams.get('state');
      const errorParam = urlParams.get('error');
      const errorDescription = urlParams.get('error_description');

      // Handle OAuth errors from provider
      if (errorParam) {
        const errorMsg = errorDescription || `OAuth error: ${errorParam}`;
        throw new Error(errorMsg);
      }

      if (!code || !state) {
        throw new Error('Missing authorization code or state parameter');
      }

      // Validate state parameter
      const storedState = sessionStorage.getItem(`oauth_state_${provider}`);
      if (!storedState || storedState !== state) {
        throw new Error('Invalid state parameter - possible CSRF attack');
      }

      // Exchange code for tokens
      const result = await api.post<OAuthCallbackResponse>(`/auth/oauth/${provider}/callback`, {
        code,
        state,
        redirect_uri: window.location.origin + window.location.pathname
      });

      if (result.success) {
        const { access_token, refresh_token } = result.data;
        
        // Store tokens
        localStorage.setItem('auth_token', access_token);
        localStorage.setItem('refresh_token', refresh_token);
        
        // Update auth store
        authActions.fetchProfile();
        
        success = true;
        
        // Clean up stored state
        sessionStorage.removeItem(`oauth_state_${provider}`);
        
        // Redirect to dashboard after a brief success message
        setTimeout(() => {
          window.location.href = '/';
        }, 2000);
        
      } else {
        throw new Error(result.message || 'OAuth authentication failed');
      }
      
    } catch (err: any) {
      console.error('OAuth callback error:', err);
      error = err.message || 'Authentication failed';
      
      // Clean up stored state on error
      sessionStorage.removeItem(`oauth_state_${provider}`);
      
    } finally {
      isProcessing = false;
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

  function retryLogin() {
    window.location.href = '/';
  }
</script>

<div class="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
  <div class="max-w-md w-full space-y-8">
    <div class="text-center">
      {#if isProcessing}
        <div class="mx-auto h-12 w-12 text-blue-600">
          <svg class="animate-spin h-12 w-12" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
        </div>
        <h2 class="mt-6 text-3xl font-extrabold text-gray-900">
          Completing {getProviderName(provider)} Sign In
        </h2>
        <p class="mt-2 text-sm text-gray-600">
          Please wait while we complete your authentication...
        </p>
        
      {:else if success}
        <div class="mx-auto h-12 w-12 text-green-600">
          <svg class="h-12 w-12" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"></path>
          </svg>
        </div>
        <h2 class="mt-6 text-3xl font-extrabold text-gray-900">
          Welcome!
        </h2>
        <p class="mt-2 text-sm text-gray-600">
          Successfully signed in with {getProviderName(provider)}. Redirecting to your dashboard...
        </p>
        
      {:else if error}
        <div class="mx-auto h-12 w-12 text-red-600">
          <svg class="h-12 w-12" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16c-.77.833.192 2.5 1.732 2.5z"></path>
          </svg>
        </div>
        <h2 class="mt-6 text-3xl font-extrabold text-gray-900">
          Authentication Failed
        </h2>
        <div class="mt-4 p-4 bg-red-50 border border-red-200 rounded-md">
          <p class="text-sm text-red-800">{error}</p>
        </div>
        
        <div class="mt-6">
          <button
            type="button"
            on:click={retryLogin}
            class="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
          >
            Try Again
          </button>
        </div>
      {/if}
    </div>
  </div>
</div>