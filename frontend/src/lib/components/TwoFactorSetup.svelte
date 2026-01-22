<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { FormError } from './ui';

  const dispatch = createEventDispatcher();
  
  export let qrCodeUrl = '';
  export let secret = '';
  export let isLoading = false;
  export let error = '';
  
  let verificationCode = '';
  let step = 1; // 1: Show QR, 2: Verify code, 3: Success
  
  $: codeValid = verificationCode.length === 6 && /^\d{6}$/.test(verificationCode);
  
  function handleVerify() {
    if (!codeValid) return;
    
    dispatch('verify', {
      code: verificationCode
    });
  }
  
  function handleCancel() {
    dispatch('cancel');
  }
  
  function copySecret() {
    navigator.clipboard.writeText(secret).then(() => {
      // Could add a toast notification here
    });
  }
  
  // Handle successful verification from parent
  export function showSuccess() {
    step = 3;
  }
</script>

<div class="max-w-md mx-auto space-y-6">
  <div class="text-center">
    <h2 class="text-zinc-4002xl font-bold text-zinc-400darker">
      Set up Two-Factor Authentication
    </h2>
    <p class="mt-2 text-zinc-400 text-zinc-400darker">
      Add an extra layer of security to your account
    </p>
  </div>

  {#if step === 1}
    <!-- QR Code Setup Step -->
    <div class="space-y-4">
      <div class="bg-zinc-700lightest rounded-uswds-lg p-uswds-6 text-center">
        <h3 class="text-zinc-400 font-medium text-zinc-400darker mb-4">
          Step 1: Scan QR Code
        </h3>
        
        {#if qrCodeUrl}
          <div class="p-uswds-4 rounded-uswds-lg inline-block" style="background: #3f3f46;">
            <img 
              src={qrCodeUrl} 
              alt="2FA QR Code" 
              class="w-48 h-48 mx-auto"
            />
          </div>
        {:else}
          <div class="w-48 h-48 mx-auto bg-zinc-700lightest rounded-uswds-lg flex items-center justify-center">
            <div class="animate-spin rounded-full icon icon-xl  border-b-2 border-indigo-600"></div>
          </div>
        {/if}
        
        <p class="mt-4 text-zinc-400 text-zinc-400darker">
          Scan this QR code with your authenticator app (Google Authenticator, Authy, etc.)
        </p>
      </div>

      <!-- Manual Entry Option -->
      <div class="bg-zinc-700 rounded-uswds-lg p-uswds-4">
        <h4 class="text-zinc-400 font-medium text-zinc-400 mb-2">
          Can't scan? Enter manually:
        </h4>
        <div class="flex items-center space-x-2">
          <code class="flex-1 px-3 py-2 rounded-lg text-zinc-400 font-mono text-zinc-300" style="background: #3f3f46; border: 1px solid #52525b;">
            {secret}
          </code>
          <button
            type="button"
            on:click={copySecret}
            class="px-3 py-2 text-zinc-400 bg-zinc-700 text-white rounded hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
          >
            Copy
          </button>
        </div>
      </div>

      <button
        type="button"
        on:click={() => step = 2}
        class="w-full py-2 px-4 border border-transparent rounded-uswds-md shadow-sm text-zinc-400 font-medium text-white bg-primary hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
      >
        I've added the account to my app
      </button>
    </div>
  
  {:else if step === 2}
    <!-- Verification Step -->
    <div class="space-y-4">
      <div class="bg-zinc-700lightest rounded-uswds-lg p-uswds-6 text-center">
        <h3 class="text-zinc-400 font-medium text-zinc-400darker mb-4">
          Step 2: Verify Setup
        </h3>
        
        <p class="text-zinc-400 text-zinc-400darker mb-4">
          Enter the 6-digit code from your authenticator app to complete setup
        </p>
        
        <div class="max-w-xs mx-auto">
          <input
            type="text"
            bind:value={verificationCode}
            maxlength="6"
            pattern="[0-9]{6}"
            placeholder="000000"
            class="block w-full text-center text-zinc-4002xl font-mono px-3 py-3 rounded-lg text-zinc-300 placeholder-zinc-500 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
            class:border-red-300={verificationCode.length > 0 && !codeValid}
            style="background: #3f3f46; border: 1px solid #52525b;"
          />
          {#if verificationCode.length > 0 && !codeValid}
            <p class="mt-1 text-zinc-400 text-zinc-400">Please enter a 6-digit code</p>
          {/if}
        </div>
      </div>

      <FormError message={error} id="2fa-setup-error" />

      <div class="flex space-x-3">
        <button
          type="button"
          on:click={() => step = 1}
          class="flex-1 py-2 px-4 rounded-lg shadow-sm text-zinc-400 font-medium text-zinc-300 hover:bg-zinc-600 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500" style="background: #3f3f46; border: 1px solid #52525b;"
        >
          Back
        </button>
        <button
          type="button"
          on:click={handleVerify}
          disabled={!codeValid || isLoading}
          class="flex-1 py-2 px-4 border border-transparent rounded-uswds-md shadow-sm text-zinc-400 font-medium text-white btn btn-primary focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {#if isLoading}
            <svg aria-hidden="true" class="animate-spin -ml-1 mr-2 icon-uswds icon-uswds--sm text-white inline" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            Verifying...
          {:else}
            Verify & Enable
          {/if}
        </button>
      </div>
    </div>
  
  {:else if step === 3}
    <!-- Success Step -->
    <div class="text-center space-y-4">
      <div class="mx-auto avatar avatar--xl bg-green-100">
        <svg aria-hidden="true" class="icon-uswds icon-uswds--lg text-zinc-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
        </svg>
      </div>
      
      <div>
        <h3 class="text-zinc-400 font-medium text-zinc-400darker">
          Two-Factor Authentication Enabled!
        </h3>
        <p class="mt-2 text-zinc-400 text-zinc-400darker">
          Your account is now protected with 2FA. You'll need to enter a code from your authenticator app each time you sign in.
        </p>
      </div>

      <div class="bg-yellow-50 rounded-uswds-lg p-uswds-4">
        <div class="flex">
          <div class="flex-shrink-0">
            <svg aria-hidden="true" class="icon-uswds icon-uswds--md text-yellow-400" viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
            </svg>
          </div>
          <div class="ml-3">
            <p class="text-zinc-400 text-yellow-800">
              <strong>Important:</strong> Save your recovery codes in a safe place. You'll need them if you lose access to your authenticator app.
            </p>
          </div>
        </div>
      </div>

      <button
        type="button"
        on:click={() => dispatch('complete')}
        class="w-full py-2 px-4 border border-transparent rounded-uswds-md shadow-sm text-zinc-400 font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
      >
        Continue to Dashboard
      </button>
    </div>
  {/if}

  <!-- Cancel Button (except on success step) -->
  {#if step !== 3}
    <div class="text-center">
      <button
        type="button"
        on:click={handleCancel}
        class="text-zinc-400 text-zinc-400 hover:text-zinc-300"
      >
        Skip for now
      </button>
    </div>
  {/if}
</div>