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
          success = 'Account created! You can now sign in.';
          mode = 'login';
          password = '';
          confirmPassword = '';
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

<div class="min-h-screen flex flex-col items-center justify-center px-4" style="background: #18181b;">
  <div class="w-full max-w-sm">
    <!-- Logo -->
    <div class="text-center mb-8">
      <div class="inline-flex items-center justify-center w-16 h-16 rounded-full mb-5 bg-rose-500">
        <svg class="w-9 h-9 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
        </svg>
      </div>
      <h1 class="text-3xl font-bold text-white tracking-tight">No Drake in the House</h1>
      <p class="text-zinc-300 mt-2">Take control of your music</p>
    </div>

    <!-- Form -->
    <form on:submit|preventDefault={handleSubmit} class="space-y-4">
      {#if error}
        <div class="flex items-center gap-3 px-4 py-3 rounded-lg" style="background: rgba(239, 68, 68, 0.15); border: 1px solid rgba(239, 68, 68, 0.4);">
          <svg class="w-5 h-5 text-red-400 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <span class="text-red-200 text-sm">{error}</span>
        </div>
      {/if}

      {#if success}
        <div class="flex items-center gap-3 px-4 py-3 rounded-lg" style="background: rgba(16, 185, 129, 0.15); border: 1px solid rgba(16, 185, 129, 0.4);">
          <svg class="w-5 h-5 text-green-400 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <span class="text-green-200 text-sm">{success}</span>
        </div>
      {/if}

      <div>
        <label for="email" class="block text-sm font-medium text-white mb-2">Email</label>
        <input
          id="email"
          type="email"
          bind:value={email}
          placeholder="name@example.com"
          required
          class="w-full rounded-lg px-4 py-3 text-white placeholder-zinc-500 focus:outline-none transition-colors"
          style="background: #27272a; border: 1px solid #52525b;"
        />
      </div>

      <div>
        <label for="password" class="block text-sm font-medium text-white mb-2">Password</label>
        <input
          id="password"
          type="password"
          bind:value={password}
          placeholder="Password"
          required
          minlength="8"
          class="w-full rounded-lg px-4 py-3 text-white placeholder-zinc-500 focus:outline-none transition-colors"
          style="background: #27272a; border: 1px solid #52525b;"
        />
      </div>

      {#if mode === 'register'}
        <div>
          <label for="confirmPassword" class="block text-sm font-medium text-white mb-2">Confirm Password</label>
          <input
            id="confirmPassword"
            type="password"
            bind:value={confirmPassword}
            placeholder="Confirm password"
            required
            minlength="8"
            class="w-full rounded-lg px-4 py-3 text-white placeholder-zinc-500 focus:outline-none transition-colors"
            style="background: #27272a; border: 1px solid #52525b;"
          />
        </div>
      {/if}

      <button
        type="submit"
        disabled={isLoading}
        class="w-full bg-rose-500 hover:bg-rose-600 text-white font-semibold py-3 rounded-full transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2 mt-6 hover:scale-[1.02] active:scale-[0.98]"
      >
        {#if isLoading}
          <div class="w-5 h-5 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
        {/if}
        {mode === 'login' ? 'Sign in' : 'Create account'}
      </button>
    </form>

    <!-- Divider -->
    <div class="flex items-center gap-4 my-6">
      <div class="flex-1 h-px" style="background: #3f3f46;"></div>
      <span class="text-zinc-400 text-sm">or</span>
      <div class="flex-1 h-px" style="background: #3f3f46;"></div>
    </div>

    <!-- Switch mode -->
    <div class="text-center">
      <span class="text-zinc-300">{mode === 'login' ? "Don't have an account?" : 'Already have an account?'}</span>
      <button
        type="button"
        on:click={switchMode}
        class="text-white underline hover:text-rose-400 transition-colors ml-1"
      >
        {mode === 'login' ? 'Sign up' : 'Sign in'}
      </button>
    </div>

    <!-- Features -->
    <div class="mt-10 pt-8" style="border-top: 1px solid #3f3f46;">
      <div class="space-y-4">
        <div class="flex items-center gap-3">
          <div class="w-8 h-8 rounded-full flex items-center justify-center flex-shrink-0" style="background: rgba(244, 63, 94, 0.15);">
            <svg class="w-4 h-4 text-rose-400" fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
              <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
            </svg>
          </div>
          <span class="text-zinc-200 text-sm">Evidence-based artist blocklists</span>
        </div>
        <div class="flex items-center gap-3">
          <div class="w-8 h-8 rounded-full flex items-center justify-center flex-shrink-0" style="background: rgba(244, 63, 94, 0.15);">
            <svg class="w-4 h-4 text-rose-400" fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
              <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
            </svg>
          </div>
          <span class="text-zinc-200 text-sm">Works with Spotify and Apple Music</span>
        </div>
        <div class="flex items-center gap-3">
          <div class="w-8 h-8 rounded-full flex items-center justify-center flex-shrink-0" style="background: rgba(244, 63, 94, 0.15);">
            <svg class="w-4 h-4 text-rose-400" fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
              <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
            </svg>
          </div>
          <span class="text-zinc-200 text-sm">Blocks features and collaborations</span>
        </div>
      </div>
    </div>
  </div>
</div>
