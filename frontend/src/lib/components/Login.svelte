<script lang="ts">
  import { authActions } from '../stores/auth';
  import LoginForm from './LoginForm.svelte';
  import RegisterForm from './RegisterForm.svelte';
  import TwoFactorSetup from './TwoFactorSetup.svelte';
  import TwoFactorVerification from './TwoFactorVerification.svelte';
  
  let mode = 'login'; // 'login', 'register', '2fa-setup', '2fa-verify'
  let isLoading = false;
  let error = '';
  let success = '';
  
  // 2FA setup data
  let qrCodeUrl = '';
  let secret = '';
  let pendingEmail = '';
  
  // Component references
  let twoFactorSetupRef: TwoFactorSetup;

  async function handleLogin(event: CustomEvent) {
    const { email, password, totpCode } = event.detail;
    isLoading = true;
    error = '';
    
    try {
      const result = await authActions.login(email, password, totpCode);
      
      if (!result.success) {
        if (result.message?.toLowerCase().includes('2fa') || result.message?.toLowerCase().includes('totp')) {
          pendingEmail = email;
          mode = '2fa-verify';
        } else {
          error = result.message || 'Login failed';
        }
      }
      // Success is handled by the auth store updating isAuthenticated
    } catch (err) {
      error = 'An unexpected error occurred';
    } finally {
      isLoading = false;
    }
  }

  async function handleRegister(event: CustomEvent) {
    const { email, password } = event.detail;
    isLoading = true;
    error = '';
    success = '';
    
    try {
      const result = await authActions.register(email, password);
      
      if (result.success) {
        success = 'Account created successfully! You can now sign in.';
        mode = 'login';
      } else {
        error = result.message || 'Registration failed';
      }
    } catch (err) {
      error = 'An unexpected error occurred';
    } finally {
      isLoading = false;
    }
  }

  async function handle2FAVerification(event: CustomEvent) {
    const { code } = event.detail;
    isLoading = true;
    error = '';
    
    try {
      // For login verification, we need to retry the login with the TOTP code
      // This assumes we stored the email/password from the previous attempt
      // In a real implementation, you might want to handle this differently
      const result = await authActions.verify2FA(code);
      
      if (!result.success) {
        error = result.message || 'Invalid verification code';
      }
      // Success is handled by the auth store
    } catch (err) {
      error = 'An unexpected error occurred';
    } finally {
      isLoading = false;
    }
  }

  async function handle2FASetup() {
    isLoading = true;
    error = '';
    
    try {
      const result = await authActions.setup2FA();
      
      if (result.success) {
        qrCodeUrl = result.qrCodeUrl;
        secret = result.secret;
        mode = '2fa-setup';
      } else {
        error = result.message || 'Failed to setup 2FA';
      }
    } catch (err) {
      error = 'An unexpected error occurred';
    } finally {
      isLoading = false;
    }
  }

  async function handle2FASetupVerification(event: CustomEvent) {
    const { code } = event.detail;
    isLoading = true;
    error = '';
    
    try {
      const result = await authActions.verify2FA(code);
      
      if (result.success) {
        twoFactorSetupRef?.showSuccess();
      } else {
        error = result.message || 'Invalid verification code';
      }
    } catch (err) {
      error = 'An unexpected error occurred';
    } finally {
      isLoading = false;
    }
  }

  function switchToLogin() {
    mode = 'login';
    error = '';
    success = '';
  }

  function switchToRegister() {
    mode = 'register';
    error = '';
    success = '';
  }

  function backToLogin() {
    mode = 'login';
    error = '';
    pendingEmail = '';
  }

  function cancel2FASetup() {
    mode = 'login';
    error = '';
    qrCodeUrl = '';
    secret = '';
  }

  function complete2FASetup() {
    // 2FA setup complete, user should now be authenticated
    mode = 'login';
    qrCodeUrl = '';
    secret = '';
  }
</script>

<div class="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
  <div class="max-w-md w-full">
    {#if mode === 'login'}
      <LoginForm 
        {isLoading} 
        {error} 
        on:login={handleLogin}
        on:switchMode={switchToRegister}
      />
      
      <!-- Optional 2FA Setup Link for existing users -->
      <div class="mt-6 text-center">
        <button
          type="button"
          on:click={handle2FASetup}
          class="text-sm text-gray-500 hover:text-gray-700"
        >
          Set up Two-Factor Authentication
        </button>
      </div>
      
    {:else if mode === 'register'}
      <RegisterForm 
        {isLoading} 
        {error}
        {success}
        on:register={handleRegister}
        on:switchMode={switchToLogin}
      />
      
    {:else if mode === '2fa-verify'}
      <TwoFactorVerification
        {isLoading}
        {error}
        email={pendingEmail}
        on:verify={handle2FAVerification}
        on:back={backToLogin}
        on:resend={() => {
          // Could implement resend logic here
          error = 'Please try logging in again';
          backToLogin();
        }}
      />
      
    {:else if mode === '2fa-setup'}
      <TwoFactorSetup
        bind:this={twoFactorSetupRef}
        {qrCodeUrl}
        {secret}
        {isLoading}
        {error}
        on:verify={handle2FASetupVerification}
        on:cancel={cancel2FASetup}
        on:complete={complete2FASetup}
      />
    {/if}
  </div>
</div>