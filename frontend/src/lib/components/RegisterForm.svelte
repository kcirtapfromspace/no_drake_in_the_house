<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  
  const dispatch = createEventDispatcher();
  
  export let isLoading = false;
  export let error = '';
  export let success = '';
  export let fieldErrors: Record<string, string> = {};
  
  let email = '';
  let password = '';
  let confirmPassword = '';
  let termsAccepted = false;
  
  // Password strength requirements
  $: passwordLength = password.length >= 8;
  $: passwordUppercase = /[A-Z]/.test(password);
  $: passwordLowercase = /[a-z]/.test(password);
  $: passwordNumber = /\d/.test(password);
  $: passwordSpecial = /[!@#$%^&*(),.?":{}|<>]/.test(password);
  
  $: passwordStrength = [
    passwordLength,
    passwordUppercase,
    passwordLowercase,
    passwordNumber,
    passwordSpecial
  ].filter(Boolean).length;
  
  $: passwordStrengthText = passwordStrength === 0 ? '' :
    passwordStrength <= 2 ? 'Weak' :
    passwordStrength <= 3 ? 'Fair' :
    passwordStrength <= 4 ? 'Good' : 'Strong';

  // Validation
  $: emailValid = email.length === 0 || /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email);
  $: passwordValid = password.length === 0 || (passwordLength && passwordStrength >= 3);
  $: passwordsMatch = confirmPassword.length === 0 || password === confirmPassword;
  $: formValid = emailValid && passwordValid && passwordsMatch && termsAccepted &&
                 email.length > 0 && password.length > 0 && confirmPassword.length > 0;
  
  function handleSubmit() {
    if (!formValid || isLoading) return;
    
    dispatch('register', {
      email: email.trim(),
      password,
      confirm_password: confirmPassword,
      terms_accepted: termsAccepted
    });
  }
</script>

<style>
  /* Component-specific overrides */
  .register-form-container {
    min-height: fit-content;
  }
</style>

<div class="register-form-container w-full max-w-md mx-auto space-y-6 px-4 sm:px-0">
  <div>
    <h2 class="text-center text-zinc-4002xl sm:text-zinc-4003xl font-extrabold text-zinc-400darker">
      Create your account
    </h2>
    <p class="mt-2 text-center text-zinc-400 text-zinc-400darker">
      Join the music blocklist community
    </p>
  </div>
  
  <form class="space-y-4 sm:space-y-6" on:submit|preventDefault={handleSubmit}>
    <!-- Email Field -->
    <div class="form-field-uswds">
      <label for="register-email" class="form-label-uswds">
        Email address
      </label>
      <input
        id="register-email"
        name="email"
        type="email"
        autocomplete="email"
        required
        bind:value={email}
        class="form-input-uswds"
        class:form-input--error={!emailValid}
        placeholder="Enter your email"
      />
      {#if fieldErrors.email}
        <div class="validation-message validation-message--error">
          <svg class="icon-uswds icon-uswds--sm icon-uswds--error" fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
            <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-8-3a1 1 0 00-.867.5 1 1 0 11-1.731-1A3 3 0 0113 8a3.001 3.001 0 01-2 2.83V11a1 1 0 11-2 0v-1a1 1 0 011-1 1 1 0 100-2zm0 8a1 1 0 100-2 1 1 0 000 2z" clip-rule="evenodd" />
          </svg>
          {fieldErrors.email}
        </div>
      {:else if !emailValid}
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
      <label for="register-password" class="form-label-uswds">
        Password
      </label>
      <input
        id="register-password"
        name="password"
        type="password"
        autocomplete="new-password"
        required
        bind:value={password}
        class="form-input-uswds"
        class:form-input--error={fieldErrors.password || (!passwordValid && password.length > 0)}
        placeholder="Create a strong password"
      />
        
      <!-- Password Strength Indicator -->
      {#if password.length > 0}
        <div class="password-requirements">
          <div class="password-strength">
            <span class="password-strength__label">Password strength:</span>
            <span class="password-strength__value password-strength__value--{passwordStrengthText.toLowerCase()}">
              {passwordStrengthText}
            </span>
          </div>
          <div class="password-strength-bars">
            {#each Array(5) as _, i}
              <div 
                class="password-strength-bar"
                class:password-strength-bar--active-weak={passwordStrength <= 2 && i < passwordStrength}
                class:password-strength-bar--active-fair={passwordStrength === 3 && i < passwordStrength}
                class:password-strength-bar--active-good={passwordStrength === 4 && i < passwordStrength}
                class:password-strength-bar--active-strong={passwordStrength === 5 && i < passwordStrength}
              ></div>
            {/each}
          </div>
          
          <div class="requirements-grid">
            <div class="requirement-item">
              <div class="requirement-icon">
                {#if passwordLength}
                  <svg class="icon-uswds icon-uswds--sm icon-uswds--success" fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                  </svg>
                {:else}
                  <div class="icon-uswds icon-uswds--sm icon-uswds--neutral">
                    <svg class="icon-uswds icon-uswds--sm" fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
                      <circle cx="10" cy="10" r="8" stroke="currentColor" stroke-width="2" fill="none" />
                    </svg>
                  </div>
                {/if}
              </div>
              <span class="requirement-text" class:requirement-text--satisfied={passwordLength}>
                At least 8 characters
              </span>
            </div>
            
            <div class="requirement-item">
              <div class="requirement-icon">
                {#if passwordUppercase}
                  <svg class="icon-uswds icon-uswds--sm icon-uswds--success" fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                  </svg>
                {:else}
                  <div class="icon-uswds icon-uswds--sm icon-uswds--neutral">
                    <svg class="icon-uswds icon-uswds--sm" fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
                      <circle cx="10" cy="10" r="8" stroke="currentColor" stroke-width="2" fill="none" />
                    </svg>
                  </div>
                {/if}
              </div>
              <span class="requirement-text" class:requirement-text--satisfied={passwordUppercase}>
                One uppercase letter
              </span>
            </div>
            
            <div class="requirement-item">
              <div class="requirement-icon">
                {#if passwordLowercase}
                  <svg class="icon-uswds icon-uswds--sm icon-uswds--success" fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                  </svg>
                {:else}
                  <div class="icon-uswds icon-uswds--sm icon-uswds--neutral">
                    <svg class="icon-uswds icon-uswds--sm" fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
                      <circle cx="10" cy="10" r="8" stroke="currentColor" stroke-width="2" fill="none" />
                    </svg>
                  </div>
                {/if}
              </div>
              <span class="requirement-text" class:requirement-text--satisfied={passwordLowercase}>
                One lowercase letter
              </span>
            </div>
            
            <div class="requirement-item">
              <div class="requirement-icon">
                {#if passwordNumber}
                  <svg class="icon-uswds icon-uswds--sm icon-uswds--success" fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                  </svg>
                {:else}
                  <div class="icon-uswds icon-uswds--sm icon-uswds--neutral">
                    <svg class="icon-uswds icon-uswds--sm" fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
                      <circle cx="10" cy="10" r="8" stroke="currentColor" stroke-width="2" fill="none" />
                    </svg>
                  </div>
                {/if}
              </div>
              <span class="requirement-text" class:requirement-text--satisfied={passwordNumber}>
                One number
              </span>
            </div>
            
            <div class="requirement-item">
              <div class="requirement-icon">
                {#if passwordSpecial}
                  <svg class="icon-uswds icon-uswds--sm icon-uswds--success" fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                  </svg>
                {:else}
                  <div class="icon-uswds icon-uswds--sm icon-uswds--neutral">
                    <svg class="icon-uswds icon-uswds--sm" fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
                      <circle cx="10" cy="10" r="8" stroke="currentColor" stroke-width="2" fill="none" />
                    </svg>
                  </div>
                {/if}
              </div>
              <span class="requirement-text" class:requirement-text--satisfied={passwordSpecial}>
                One special character (!@#$%^&*)
              </span>
            </div>
          </div>
        </div>
      {/if}
      
      {#if fieldErrors.password}
        <div class="validation-message validation-message--error">
          <svg class="icon-uswds icon-uswds--sm icon-uswds--error" fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
            <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-8-3a1 1 0 00-.867.5 1 1 0 11-1.731-1A3 3 0 0113 8a3.001 3.001 0 01-2 2.83V11a1 1 0 11-2 0v-1a1 1 0 011-1 1 1 0 100-2zm0 8a1 1 0 100-2 1 1 0 000 2z" clip-rule="evenodd" />
          </svg>
          {fieldErrors.password}
        </div>
      {/if}
    </div>

    <!-- Confirm Password Field -->
    <div class="form-field-uswds">
      <label for="confirm-password" class="form-label-uswds">
        Confirm Password
      </label>
      <input
        id="confirm-password"
        name="confirmPassword"
        type="password"
        autocomplete="new-password"
        required
        bind:value={confirmPassword}
        class="form-input-uswds"
        class:form-input--error={!passwordsMatch && confirmPassword.length > 0}
        placeholder="Confirm your password"
      />
      {#if fieldErrors.confirm_password}
        <div class="validation-message validation-message--error">
          <svg class="icon-uswds icon-uswds--sm icon-uswds--error" fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
            <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-8-3a1 1 0 00-.867.5 1 1 0 11-1.731-1A3 3 0 0113 8a3.001 3.001 0 01-2 2.83V11a1 1 0 11-2 0v-1a1 1 0 011-1 1 1 0 100-2zm0 8a1 1 0 100-2 1 1 0 000 2z" clip-rule="evenodd" />
          </svg>
          {fieldErrors.confirm_password}
        </div>
      {:else if !passwordsMatch && confirmPassword.length > 0}
        <div class="validation-message validation-message--error">
          <svg class="icon-uswds icon-uswds--sm icon-uswds--error" fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
            <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-8-3a1 1 0 00-.867.5 1 1 0 11-1.731-1A3 3 0 0113 8a3.001 3.001 0 01-2 2.83V11a1 1 0 11-2 0v-1a1 1 0 011-1 1 1 0 100-2zm0 8a1 1 0 100-2 1 1 0 000 2z" clip-rule="evenodd" />
          </svg>
          Passwords do not match
        </div>
      {/if}
    </div>

    <!-- Terms Acceptance -->
    <div class="flex items-start space-x-3">
      <div class="flex items-center icon icon-md mt-0.5">
        <input
          id="terms-accepted"
          name="termsAccepted"
          type="checkbox"
          required
          bind:checked={termsAccepted}
          class="focus:ring-indigo-500 icon-uswds icon-uswds--sm text-primary rounded transition-colors duration-200" style="border: 1px solid #52525b;"
        />
      </div>
      <div class="flex-1 text-zinc-400">
        <label for="terms-accepted" class="text-zinc-400darker leading-relaxed cursor-pointer">
          I agree to the 
          <a href="/terms" target="_blank" class="text-indigo-600 hover:text-indigo-500 underline font-medium">
            Terms of Service
          </a>
          and 
          <a href="/privacy" target="_blank" class="text-indigo-600 hover:text-indigo-500 underline font-medium">
            Privacy Policy
          </a>
        </label>
        {#if fieldErrors.terms_accepted}
          <p class="mt-1 text-zinc-400 text-zinc-400">{fieldErrors.terms_accepted}</p>
        {:else if !termsAccepted && formValid === false}
          <p class="mt-1 text-zinc-400 text-zinc-400">You must accept the terms to continue</p>
        {/if}
      </div>
    </div>

    <!-- Success Message -->
    {#if success}
      <div class="alert-uswds alert-uswds-success">
        <div class="alert__icon">
          <svg class="icon-uswds icon-uswds--lg icon-uswds--success" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
          </svg>
        </div>
        <div class="alert__content">
          <p class="alert__text">{success}</p>
        </div>
      </div>
    {/if}

    <!-- Error Message -->
    {#if error}
      <div class="alert-uswds alert-uswds-error">
        <div class="alert__icon">
          <svg class="icon-uswds icon-uswds--lg icon-uswds--error" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
          </svg>
        </div>
        <div class="alert__content">
          <p class="alert__text">{error}</p>
        </div>
      </div>
    {/if}

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
          Creating account...
        {:else}
          Create account
        {/if}
      </button>
    </div>

    <!-- Switch to Login -->
    <div class="text-center">
      <button
        type="button"
        on:click={() => dispatch('switchMode')}
        class="text-indigo-600 hover:text-indigo-500 text-zinc-400 font-medium"
      >
        Already have an account? Sign in
      </button>
    </div>
  </form>
</div>