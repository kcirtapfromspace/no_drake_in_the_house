<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import SocialLoginButtons from './SocialLoginButtons.svelte';
  import { FormError } from './ui';

  const dispatch = createEventDispatcher();
  
  export let isLoading = false;
  export let error = '';
  
  let email = '';
  let password = '';
  let totpCode = '';
  let showTotpInput = false;
  let socialError = '';
  let socialLoading = false;
  
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

  function handleSocialLoading(event: CustomEvent) {
    const { loading } = event.detail;
    socialLoading = loading;
  }

  function handleSocialError(event: CustomEvent) {
    const { message } = event.detail;
    socialError = message;
    // Clear social error after 5 seconds
    setTimeout(() => {
      socialError = '';
    }, 5000);
  }
</script>

<style>
  /* Component-specific overrides */
  .login-form-container {
    min-height: fit-content;
  }
</style>

<div class="login-form-container w-full max-w-md mx-auto space-y-6 px-4 sm:px-0">
  <div>
    <h2 class="text-center text-zinc-4002xl sm:text-zinc-4003xl font-extrabold text-zinc-400darker">
      Sign in to your account
    </h2>
    <p class="mt-2 text-center text-zinc-400 text-zinc-400darker">
      Access your music blocklist manager
    </p>
  </div>
  
  <form class="space-y-4 sm:space-y-6" on:submit|preventDefault={handleSubmit}>
    <!-- Email Field -->
    <div class="form-field-uswds">
      <label for="login-email" class="form-label-uswds">
        Email address
      </label>
      <input
        id="login-email"
        name="email"
        type="email"
        autocomplete="email"
        required
        bind:value={email}
        class="form-input-uswds"
        class:form-input--error={!emailValid}
        placeholder="Enter your email"
      />
      {#if !emailValid}
        <div class="validation-message validation-message--error">
          <svg class="icon-uswds icon-uswds--sm icon-uswds--error" fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
            <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-8-3a1 1 0 00-.867.5 1 1 0 11-1.731-1A3 3 0 0113 8a3.001 3.001 0 01-2 2.83V11a1 1 0 11-2 0v-1a1 1 0 011-1 1 1 0 100-2zm0 8a1 1 0 100-2 1 1 0 000 2z" clip-rule="evenodd" />
          </svg>
          Please enter a valid email address
        </div>
      {/if}
    </div>

    <!-- Password Field -->
    <div class="form-field-uswds">
      <label for="login-password" class="form-label-uswds">
        Password
      </label>
      <input
        id="login-password"
        name="password"
        type="password"
        autocomplete="current-password"
        required
        bind:value={password}
        class="form-input-uswds"
        class:form-input--error={!passwordValid}
        placeholder="Enter your password"
      />
      {#if !passwordValid}
        <div class="validation-message validation-message--error">
          <svg class="icon-uswds icon-uswds--sm icon-uswds--error" fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
            <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-8-3a1 1 0 00-.867.5 1 1 0 11-1.731-1A3 3 0 0113 8a3.001 3.001 0 01-2 2.83V11a1 1 0 11-2 0v-1a1 1 0 011-1 1 1 0 100-2zm0 8a1 1 0 100-2 1 1 0 000 2z" clip-rule="evenodd" />
          </svg>
          Password must be at least 8 characters
        </div>
      {/if}
    </div>

    <!-- 2FA Code Field (shown when required) -->
    {#if showTotpInput}
      <div class="form-field-uswds">
        <label for="login-totp" class="form-label-uswds">
          2FA Authentication Code
        </label>
        <input
          id="login-totp"
          name="totp"
          type="text"
          autocomplete="one-time-code"
          bind:value={totpCode}
          maxlength="6"
          pattern="[0-9]{6}"
          class="form-input-uswds"
          placeholder="Enter 6-digit code"
        />
        <div class="validation-message">
          <svg class="icon-uswds icon-uswds--sm icon-uswds--neutral" fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
            <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd" />
          </svg>
          Enter the 6-digit code from your authenticator app
        </div>
      </div>
    {/if}

    <!-- Error Message -->
    <FormError message={error} id="login-error" />

    <!-- Submit Button -->
    <div style="margin-top: var(--space-6);">
      <button
        type="submit"
        disabled={!formValid || isLoading}
        class="btn-uswds btn-uswds-primary btn--full"
      >
        {#if isLoading}
          <svg class="icon-uswds icon-uswds--sm animate-spin" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" aria-hidden="true" style="margin-right: var(--space-2);">
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
        class="text-primary hover:text-indigo-500 text-zinc-400 font-medium"
      >
        Don't have an account? Create one
      </button>
    </div>
  </form>

  <!-- Social Login Options -->
  <div class="mt-6">
    <SocialLoginButtons 
      isLoading={socialLoading}
      error={socialError}
      on:loading={handleSocialLoading}
      on:error={handleSocialError}
    />
  </div>
</div>