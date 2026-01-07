<script lang="ts">
  import { authActions } from '../stores/auth';

  let mode: 'login' | 'register' = 'login';
  let isLoading = false;
  let error = '';
  let success = '';

  // Form fields
  let email = '';
  let password = '';
  let confirmPassword = '';

  async function handleSubmit() {
    error = '';
    success = '';
    isLoading = true;

    try {
      if (mode === 'login') {
        const result = await authActions.login(email, password);
        if (!result.success) {
          error = result.message || 'Login failed';
        }
      } else {
        if (password !== confirmPassword) {
          error = 'Passwords do not match';
          isLoading = false;
          return;
        }
        const result = await authActions.register(email, password, confirmPassword, true);
        if (result.success) {
          success = 'Account created!';
        } else {
          error = result.message || 'Registration failed';
        }
      }
    } catch (err) {
      error = 'Something went wrong. Please try again.';
    } finally {
      isLoading = false;
    }
  }

  function switchMode() {
    mode = mode === 'login' ? 'register' : 'login';
    error = '';
    success = '';
    password = '';
    confirmPassword = '';
  }
</script>

<div class="min-h-screen flex flex-col items-center justify-center bg-gray-900 px-4">
  <div class="w-full max-w-sm">
    <!-- Logo -->
    <div class="text-center mb-8">
      <span class="text-6xl">🚫</span>
      <h1 class="text-3xl font-bold text-white mt-4">No Drake</h1>
      <p class="text-gray-400 mt-2">Block problematic artists from your music</p>
    </div>

    <!-- Form -->
    <form on:submit|preventDefault={handleSubmit} class="space-y-4">
      {#if error}
        <div class="bg-red-900/50 border border-red-700 text-red-200 px-4 py-3 rounded-lg text-sm">
          {error}
        </div>
      {/if}

      {#if success}
        <div class="bg-green-900/50 border border-green-700 text-green-200 px-4 py-3 rounded-lg text-sm">
          {success}
        </div>
      {/if}

      <div>
        <input
          type="email"
          bind:value={email}
          placeholder="Email"
          required
          class="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-3 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-red-500 focus:border-transparent"
        />
      </div>

      <div>
        <input
          type="password"
          bind:value={password}
          placeholder="Password"
          required
          minlength="8"
          class="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-3 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-red-500 focus:border-transparent"
        />
      </div>

      {#if mode === 'register'}
        <div>
          <input
            type="password"
            bind:value={confirmPassword}
            placeholder="Confirm password"
            required
            minlength="8"
            class="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-3 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-red-500 focus:border-transparent"
          />
        </div>
      {/if}

      <button
        type="submit"
        disabled={isLoading}
        class="w-full bg-red-600 hover:bg-red-700 text-white font-medium py-3 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center"
      >
        {#if isLoading}
          <div class="w-5 h-5 border-2 border-white border-t-transparent rounded-full animate-spin mr-2"></div>
        {/if}
        {mode === 'login' ? 'Sign in' : 'Create account'}
      </button>
    </form>

    <!-- Switch mode -->
    <div class="mt-6 text-center">
      <button
        type="button"
        on:click={switchMode}
        class="text-gray-400 hover:text-white transition-colors"
      >
        {mode === 'login' ? "Don't have an account? Sign up" : 'Already have an account? Sign in'}
      </button>
    </div>

    <!-- Features preview -->
    <div class="mt-12 space-y-3">
      <div class="flex items-center space-x-3 text-gray-500">
        <svg class="w-5 h-5 text-red-500" fill="currentColor" viewBox="0 0 20 20">
          <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
        </svg>
        <span class="text-sm">AI-curated blocklists by category</span>
      </div>
      <div class="flex items-center space-x-3 text-gray-500">
        <svg class="w-5 h-5 text-red-500" fill="currentColor" viewBox="0 0 20 20">
          <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
        </svg>
        <span class="text-sm">Block artists on Spotify & Apple Music</span>
      </div>
      <div class="flex items-center space-x-3 text-gray-500">
        <svg class="w-5 h-5 text-red-500" fill="currentColor" viewBox="0 0 20 20">
          <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
        </svg>
        <span class="text-sm">Blocks features & collaborations too</span>
      </div>
    </div>
  </div>
</div>
