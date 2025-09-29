<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  
  const dispatch = createEventDispatcher();
  
  export let isLoading = false;
  export let error = '';
  
  let email = '';
  let password = '';
  let totpCode = '';
  let showTotpInput = false;
  
  // Email validation
  $: emailValid = email.length === 0 || /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email);
  $: passwordValid = password.length === 0 || password.length >= 8;
  $: formValid = emailValid && passwordValid && email.length > 0 && password.length > 0;
  
  function handleSubmit() {
    if (!formValid) return;
    
    dispatch('login', {
      email: email.trim(),
      password,
      totpCode: totpCode || undefined
    });
  }
  

  
  // Reset TOTP input when error changes
  $: if (error && !error.toLowerCase().includes('2fa') && !error.toLowerCase().includes('totp')) {
    showTotpInput = false;
    totpCode = '';
  }
  
  // Show TOTP input if error indicates it's required
  $: if (error && (error.toLowerCase().includes('2fa') || error.toLowerCase().includes('totp'))) {
    showTotpInput = true;
  }
</script>

<div class="space-y-6">
  <div>
    <h2 class="text-center text-3xl font-extrabold text-gray-900">
      Sign in to your account
    </h2>
    <p class="mt-2 text-center text-sm text-gray-600">
      Access your music blocklist manager
    </p>
  </div>
  
  <form class="space-y-4" on:submit|preventDefault={handleSubmit}>
    <!-- Email Field -->
    <div>
      <label for="login-email" class="block text-sm font-medium text-gray-700">
        Email address
      </label>
      <div class="mt-1">
        <input
          id="login-email"
          name="email"
          type="email"
          autocomplete="email"
          required
          bind:value={email}
          class="appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
          class:border-red-300={!emailValid}
          class:focus:ring-red-500={!emailValid}
          class:focus:border-red-500={!emailValid}
          placeholder="Enter your email"
        />
        {#if !emailValid}
          <p class="mt-1 text-sm text-red-600">Please enter a valid email address</p>
        {/if}
      </div>
    </div>

    <!-- Password Field -->
    <div>
      <label for="login-password" class="block text-sm font-medium text-gray-700">
        Password
      </label>
      <div class="mt-1">
        <input
          id="login-password"
          name="password"
          type="password"
          autocomplete="current-password"
          required
          bind:value={password}
          class="appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
          class:border-red-300={!passwordValid}
          class:focus:ring-red-500={!passwordValid}
          class:focus:border-red-500={!passwordValid}
          placeholder="Enter your password"
        />
        {#if !passwordValid}
          <p class="mt-1 text-sm text-red-600">Password must be at least 8 characters</p>
        {/if}
      </div>
    </div>

    <!-- 2FA Code Field (shown when required) -->
    {#if showTotpInput}
      <div>
        <label for="login-totp" class="block text-sm font-medium text-gray-700">
          2FA Authentication Code
        </label>
        <div class="mt-1">
          <input
            id="login-totp"
            name="totp"
            type="text"
            autocomplete="one-time-code"
            bind:value={totpCode}
            maxlength="6"
            pattern="[0-9]{6}"
            class="appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
            placeholder="Enter 6-digit code"
          />
          <p class="mt-1 text-sm text-gray-500">
            Enter the 6-digit code from your authenticator app
          </p>
        </div>
      </div>
    {/if}

    <!-- Error Message -->
    {#if error}
      <div class="rounded-md bg-red-50 p-4">
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

    <!-- Submit Button -->
    <div>
      <button
        type="submit"
        disabled={!formValid || isLoading}
        class="group relative w-full flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {#if isLoading}
          <svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          Signing in...
        {:else}
          Sign in
        {/if}
      </button>
    </div>

    <!-- Switch to Register -->
    <div class="text-center">
      <button
        type="button"
        on:click={() => dispatch('switchMode')}
        class="text-indigo-600 hover:text-indigo-500 text-sm font-medium"
      >
        Don't have an account? Create one
      </button>
    </div>
  </form>
</div>