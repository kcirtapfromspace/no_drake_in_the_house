<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  
  const dispatch = createEventDispatcher();
  
  export let isLoading = false;
  export let error = '';
  export let email = '';
  
  let verificationCode = '';
  
  $: codeValid = verificationCode.length === 6 && /^\d{6}$/.test(verificationCode);
  
  function handleSubmit() {
    if (!codeValid) return;
    
    dispatch('verify', {
      code: verificationCode
    });
  }
  
  function handleBack() {
    dispatch('back');
  }
  
  // Auto-submit when 6 digits are entered
  $: if (codeValid && !isLoading) {
    handleSubmit();
  }
</script>

<div class="max-w-md mx-auto space-y-6">
  <div class="text-center">
    <div class="mx-auto avatar avatar--xl bg-indigo-100">
      <svg aria-hidden="true" class="icon-uswds icon-uswds--lg text-primary" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
      </svg>
    </div>
    
    <h2 class="mt-4 text-zinc-4002xl font-bold text-zinc-400darker">
      Two-Factor Authentication
    </h2>
    <p class="mt-2 text-zinc-400 text-zinc-400darker">
      Enter the 6-digit code from your authenticator app
    </p>
    {#if email}
      <p class="text-zinc-400 text-zinc-400darker">
        Signing in as <span class="font-medium">{email}</span>
      </p>
    {/if}
  </div>

  <div class="space-y-4">
    <!-- Code Input -->
    <div>
      <label for="verification-code" class="block text-zinc-400 font-medium text-zinc-400darker text-center mb-2">
        Authentication Code
      </label>
      <div class="max-w-xs mx-auto">
        <input
          id="verification-code"
          type="text"
          bind:value={verificationCode}
          maxlength="6"
          pattern="[0-9]{6}"
          placeholder="000000"
          autocomplete="one-time-code"
          class="block w-full text-center text-zinc-4002xl font-mono px-3 py-3 rounded-lg text-zinc-300 placeholder-zinc-500 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
          class:border-red-300={verificationCode.length > 0 && !codeValid}
          class:border-green-300={codeValid}
          style="background: #3f3f46; border: 1px solid #52525b;"
        />
        {#if verificationCode.length > 0 && !codeValid}
          <p class="mt-1 text-zinc-400 text-zinc-400 text-center">Please enter a 6-digit code</p>
        {/if}
      </div>
    </div>

    <!-- Error Message -->
    {#if error}
      <div class="rounded-uswds-md bg-red-50 p-uswds-4">
        <div class="flex">
          <div class="">
            <svg aria-hidden="true" class="icon-uswds icon-uswds--md text-zinc-400" viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
            </svg>
          </div>
          <div class="ml-3">
            <p class="text-zinc-400 text-zinc-400">{error}</p>
          </div>
        </div>
      </div>
    {/if}

    <!-- Loading State -->
    {#if isLoading}
      <div class="text-center">
        <div class="flex items-center px-4 py-2 text-zinc-400 text-zinc-400darker">
          <svg aria-hidden="true" class="animate-spin -ml-1 mr-3 icon-uswds icon-uswds--md text-indigo-600" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          Verifying code...
        </div>
      </div>
    {/if}

    <!-- Manual Submit Button (if auto-submit fails) -->
    {#if codeValid && !isLoading}
      <button
        type="button"
        on:click={handleSubmit}
        class="w-full py-2 px-4 border border-transparent rounded-uswds-md shadow-sm text-zinc-400 font-medium text-white bg-primary hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
      >
        Verify Code
      </button>
    {/if}

    <!-- Help Text -->
    <div class="text-center space-y-2">
      <p class="text-zinc-400 text-zinc-400darker">
        The code will automatically verify when you enter all 6 digits
      </p>
      
      <div class="space-y-1">
        <button
          type="button"
          class="text-zinc-400 text-indigo-600 hover:text-indigo-500"
          on:click={() => dispatch('resend')}
        >
          Didn't receive a code? Try again
        </button>
        
        <div>
          <button
            type="button"
            on:click={handleBack}
            class="text-zinc-400 text-zinc-400 hover:text-zinc-300"
          >
            ← Back to login
          </button>
        </div>
      </div>
    </div>
  </div>

  <!-- Troubleshooting -->
  <div class="bg-zinc-700lightest rounded-uswds-lg p-uswds-4">
    <h4 class="text-zinc-400 font-medium text-zinc-400darker mb-2">
      Having trouble?
    </h4>
    <ul class="text-zinc-400 text-zinc-400darker space-y-1">
      <li>• Make sure your device's time is correct</li>
      <li>• Try generating a new code in your authenticator app</li>
      <li>• Check that you're using the right account in your app</li>
    </ul>
  </div>
</div>