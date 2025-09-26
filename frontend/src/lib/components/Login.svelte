<script lang="ts">
  import { authActions } from '../stores/auth';
  
  let email = '';
  let password = '';
  let totpCode = '';
  let isLogin = true;
  let isLoading = false;
  let error = '';
  let requiresTotp = false;

  async function handleSubmit() {
    if (!email || !password) {
      error = 'Please fill in all required fields';
      return;
    }

    isLoading = true;
    error = '';

    try {
      if (isLogin) {
        const result = await authActions.login(email, password, totpCode || undefined);
        if (!result.success) {
          if (result.message?.includes('TOTP')) {
            requiresTotp = true;
            error = 'Please enter your 2FA code';
          } else {
            error = result.message || 'Login failed';
          }
        }
      } else {
        const result = await authActions.register(email, password);
        if (result.success) {
          isLogin = true;
          error = '';
          email = '';
          password = '';
        } else {
          error = result.message || 'Registration failed';
        }
      }
    } catch (err) {
      error = 'An unexpected error occurred';
    } finally {
      isLoading = false;
    }
  }

  function toggleMode() {
    isLogin = !isLogin;
    error = '';
    requiresTotp = false;
    totpCode = '';
  }
</script>

<div class="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
  <div class="max-w-md w-full space-y-8">
    <div>
      <h2 class="mt-6 text-center text-3xl font-extrabold text-gray-900">
        {isLogin ? 'Sign in to your account' : 'Create your account'}
      </h2>
      <p class="mt-2 text-center text-sm text-gray-600">
        Music Streaming Blocklist Manager
      </p>
    </div>
    
    <form class="mt-8 space-y-6" on:submit|preventDefault={handleSubmit}>
      <div class="rounded-md shadow-sm -space-y-px">
        <div>
          <label for="email" class="sr-only">Email address</label>
          <input
            id="email"
            name="email"
            type="email"
            autocomplete="email"
            required
            bind:value={email}
            class="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-t-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm"
            placeholder="Email address"
          />
        </div>
        <div>
          <label for="password" class="sr-only">Password</label>
          <input
            id="password"
            name="password"
            type="password"
            autocomplete={isLogin ? 'current-password' : 'new-password'}
            required
            bind:value={password}
            class="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 {requiresTotp ? '' : 'rounded-b-md'} focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm"
            placeholder="Password"
          />
        </div>
        {#if requiresTotp}
          <div>
            <label for="totp" class="sr-only">2FA Code</label>
            <input
              id="totp"
              name="totp"
              type="text"
              autocomplete="one-time-code"
              bind:value={totpCode}
              class="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-b-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm"
              placeholder="2FA Code"
            />
          </div>
        {/if}
      </div>

      {#if error}
        <div class="text-red-600 text-sm text-center">
          {error}
        </div>
      {/if}

      <div>
        <button
          type="submit"
          disabled={isLoading}
          class="group relative w-full flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {#if isLoading}
            <svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            {isLogin ? 'Signing in...' : 'Creating account...'}
          {:else}
            {isLogin ? 'Sign in' : 'Create account'}
          {/if}
        </button>
      </div>

      <div class="text-center">
        <button
          type="button"
          on:click={toggleMode}
          class="text-indigo-600 hover:text-indigo-500 text-sm"
        >
          {isLogin ? "Don't have an account? Sign up" : 'Already have an account? Sign in'}
        </button>
      </div>
    </form>
  </div>
</div>