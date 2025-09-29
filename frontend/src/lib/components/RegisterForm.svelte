<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  
  const dispatch = createEventDispatcher();
  
  export let isLoading = false;
  export let error = '';
  export let success = '';
  
  let email = '';
  let password = '';
  let confirmPassword = '';
  
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
  
  $: passwordStrengthColor = passwordStrength === 0 ? '' :
    passwordStrength <= 2 ? 'text-red-600' :
    passwordStrength <= 3 ? 'text-yellow-600' :
    passwordStrength <= 4 ? 'text-blue-600' : 'text-green-600';
  
  // Validation
  $: emailValid = email.length === 0 || /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email);
  $: passwordValid = password.length === 0 || (passwordLength && passwordStrength >= 3);
  $: passwordsMatch = confirmPassword.length === 0 || password === confirmPassword;
  $: formValid = emailValid && passwordValid && passwordsMatch && 
                 email.length > 0 && password.length > 0 && confirmPassword.length > 0;
  
  function handleSubmit() {
    if (!formValid) return;
    
    dispatch('register', {
      email: email.trim(),
      password
    });
  }
</script>

<div class="space-y-6">
  <div>
    <h2 class="text-center text-3xl font-extrabold text-gray-900">
      Create your account
    </h2>
    <p class="mt-2 text-center text-sm text-gray-600">
      Join the music blocklist community
    </p>
  </div>
  
  <form class="space-y-4" on:submit|preventDefault={handleSubmit}>
    <!-- Email Field -->
    <div>
      <label for="register-email" class="block text-sm font-medium text-gray-700">
        Email address
      </label>
      <div class="mt-1">
        <input
          id="register-email"
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
      <label for="register-password" class="block text-sm font-medium text-gray-700">
        Password
      </label>
      <div class="mt-1">
        <input
          id="register-password"
          name="password"
          type="password"
          autocomplete="new-password"
          required
          bind:value={password}
          class="appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
          class:border-red-300={!passwordValid && password.length > 0}
          class:focus:ring-red-500={!passwordValid && password.length > 0}
          class:focus:border-red-500={!passwordValid && password.length > 0}
          placeholder="Create a strong password"
        />
        
        <!-- Password Strength Indicator -->
        {#if password.length > 0}
          <div class="mt-2">
            <div class="flex items-center justify-between">
              <span class="text-sm text-gray-600">Password strength:</span>
              <span class="text-sm font-medium {passwordStrengthColor}">
                {passwordStrengthText}
              </span>
            </div>
            <div class="mt-1 flex space-x-1">
              {#each Array(5) as _, i}
                <div 
                  class="h-1 flex-1 rounded-full"
                  class:bg-red-200={passwordStrength <= 2}
                  class:bg-yellow-200={passwordStrength === 3}
                  class:bg-blue-200={passwordStrength === 4}
                  class:bg-green-200={passwordStrength === 5}
                  class:bg-red-500={passwordStrength <= 2 && i < passwordStrength}
                  class:bg-yellow-500={passwordStrength === 3 && i < passwordStrength}
                  class:bg-blue-500={passwordStrength === 4 && i < passwordStrength}
                  class:bg-green-500={passwordStrength === 5 && i < passwordStrength}
                  class:bg-gray-200={i >= passwordStrength}
                ></div>
              {/each}
            </div>
          </div>
          
          <!-- Password Requirements -->
          <div class="mt-2 space-y-1">
            <p class="text-xs text-gray-600">Password must contain:</p>
            <div class="grid grid-cols-1 gap-1 text-xs">
              <div class="flex items-center">
                <svg class="h-3 w-3 mr-1" class:text-green-500={passwordLength} class:text-gray-400={!passwordLength} fill="currentColor" viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                </svg>
                <span class:text-green-600={passwordLength} class:text-gray-500={!passwordLength}>
                  At least 8 characters
                </span>
              </div>
              <div class="flex items-center">
                <svg class="h-3 w-3 mr-1" class:text-green-500={passwordUppercase} class:text-gray-400={!passwordUppercase} fill="currentColor" viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                </svg>
                <span class:text-green-600={passwordUppercase} class:text-gray-500={!passwordUppercase}>
                  One uppercase letter
                </span>
              </div>
              <div class="flex items-center">
                <svg class="h-3 w-3 mr-1" class:text-green-500={passwordLowercase} class:text-gray-400={!passwordLowercase} fill="currentColor" viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                </svg>
                <span class:text-green-600={passwordLowercase} class:text-gray-500={!passwordLowercase}>
                  One lowercase letter
                </span>
              </div>
              <div class="flex items-center">
                <svg class="h-3 w-3 mr-1" class:text-green-500={passwordNumber} class:text-gray-400={!passwordNumber} fill="currentColor" viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                </svg>
                <span class:text-green-600={passwordNumber} class:text-gray-500={!passwordNumber}>
                  One number
                </span>
              </div>
              <div class="flex items-center">
                <svg class="h-3 w-3 mr-1" class:text-green-500={passwordSpecial} class:text-gray-400={!passwordSpecial} fill="currentColor" viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                </svg>
                <span class:text-green-600={passwordSpecial} class:text-gray-500={!passwordSpecial}>
                  One special character
                </span>
              </div>
            </div>
          </div>
        {/if}
      </div>
    </div>

    <!-- Confirm Password Field -->
    <div>
      <label for="confirm-password" class="block text-sm font-medium text-gray-700">
        Confirm Password
      </label>
      <div class="mt-1">
        <input
          id="confirm-password"
          name="confirmPassword"
          type="password"
          autocomplete="new-password"
          required
          bind:value={confirmPassword}
          class="appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
          class:border-red-300={!passwordsMatch && confirmPassword.length > 0}
          class:focus:ring-red-500={!passwordsMatch && confirmPassword.length > 0}
          class:focus:border-red-500={!passwordsMatch && confirmPassword.length > 0}
          placeholder="Confirm your password"
        />
        {#if !passwordsMatch && confirmPassword.length > 0}
          <p class="mt-1 text-sm text-red-600">Passwords do not match</p>
        {/if}
      </div>
    </div>

    <!-- Success Message -->
    {#if success}
      <div class="rounded-md bg-green-50 p-4">
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
        class="text-indigo-600 hover:text-indigo-500 text-sm font-medium"
      >
        Already have an account? Sign in
      </button>
    </div>
  </form>
</div>