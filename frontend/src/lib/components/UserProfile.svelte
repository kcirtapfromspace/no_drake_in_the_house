<script lang="ts">
  import { onMount } from 'svelte';
  import { authActions } from '../stores/auth';
  import { api } from '../utils/api';
  import TwoFactorSetup from './TwoFactorSetup.svelte';
  import AccountLinking from './AccountLinking.svelte';

  interface UserSettings {
    two_factor_enabled: boolean;
    email_notifications: boolean;
    privacy_mode: boolean;
  }

  interface OAuthAccount {
    provider: string;
    display_name?: string;
    email?: string;
    linked_at: string;
  }

  interface UserProfile {
    id: string;
    email: string;
    email_verified: boolean;
    totp_enabled: boolean;
    created_at: string;
    updated_at: string;
    last_login?: string;
    settings: UserSettings;
    oauth_accounts?: OAuthAccount[];
  }

  let profile: UserProfile | null = null;
  let isLoading = false;
  let error = '';
  let success = '';
  let activeTab = 'profile';
  
  // Profile editing
  let editingProfile = false;
  let editEmail = '';
  
  // Settings editing
  let editSettings: UserSettings = {
    two_factor_enabled: false,
    email_notifications: true,
    privacy_mode: false
  };

  // 2FA management
  let show2FASetup = false;
  let show2FADisable = false;

  // Data export
  let exportLoading = false;

  // Account deletion
  let showDeleteConfirm = false;
  let deleteConfirmEmail = '';
  let deleteReason = '';
  let deleteLoading = false;

  // Account linking
  let showAccountLinking = false;

  onMount(async () => {
    await loadProfile();
  });

  async function loadProfile() {
    isLoading = true;
    error = '';
    
    try {
      const result = await api.get('/users/profile');
      if (result.success) {
        profile = result.data;
        editEmail = profile?.email || '';
        editSettings = { 
          two_factor_enabled: profile?.settings?.two_factor_enabled || false,
          email_notifications: profile?.settings?.email_notifications || true,
          privacy_mode: profile?.settings?.privacy_mode || false
        };
      } else {
        error = result.message || 'Failed to load profile';
      }
    } catch (err: any) {
      error = err.message || 'Failed to load profile';
    } finally {
      isLoading = false;
    }
  }

  async function updateProfile() {
    if (!profile) return;
    
    isLoading = true;
    error = '';
    success = '';

    try {
      const result = await api.put('/users/profile', {
        email: editEmail !== profile.email ? editEmail : undefined
      });

      if (result.success) {
        profile = result.data;
        editingProfile = false;
        success = 'Profile updated successfully';
        // Update auth store
        await authActions.fetchProfile();
      } else {
        error = result.message || 'Failed to update profile';
      }
    } catch (err: any) {
      error = err.message || 'Failed to update profile';
    } finally {
      isLoading = false;
    }
  }

  async function updateSettings() {
    if (!profile) return;
    
    isLoading = true;
    error = '';
    success = '';

    try {
      const result = await api.put('/users/profile', {
        settings: editSettings
      });

      if (result.success) {
        profile = result.data;
        success = 'Settings updated successfully';
      } else {
        error = result.message || 'Failed to update settings';
      }
    } catch (err: any) {
      error = err.message || 'Failed to update settings';
    } finally {
      isLoading = false;
    }
  }

  async function exportData() {
    exportLoading = true;
    error = '';

    try {
      const result = await api.get('/users/export');
      if (result.success) {
        // Create and download file
        const blob = new Blob([JSON.stringify(result.data, null, 2)], { 
          type: 'application/json' 
        });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `no-drake-data-export-${new Date().toISOString().split('T')[0]}.json`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
        success = 'Data exported successfully';
      } else {
        error = result.message || 'Failed to export data';
      }
    } catch (err: any) {
      error = err.message || 'Failed to export data';
    } finally {
      exportLoading = false;
    }
  }

  async function deleteAccount() {
    if (!profile || deleteConfirmEmail !== profile.email) {
      error = 'Email confirmation does not match';
      return;
    }

    deleteLoading = true;
    error = '';

    try {
      const result = await api.delete('/users/account', {
        confirmation_email: deleteConfirmEmail,
        reason: deleteReason || undefined
      });

      if (result.success) {
        // Account deleted, logout user
        await authActions.logout();
      } else {
        error = result.message || 'Failed to delete account';
      }
    } catch (err: any) {
      error = err.message || 'Failed to delete account';
    } finally {
      deleteLoading = false;
    }
  }

  function handle2FASetupComplete() {
    show2FASetup = false;
    loadProfile(); // Reload to get updated 2FA status
    success = '2FA enabled successfully';
  }

  function handle2FASetupCancel() {
    show2FASetup = false;
  }

  async function handle2FADisable(code: string) {
    try {
      const result = await authActions.disable2FA(code);
      if (result.success) {
        show2FADisable = false;
        await loadProfile();
        success = '2FA disabled successfully';
      } else {
        error = result.message || 'Failed to disable 2FA';
      }
    } catch (err: any) {
      error = err.message || 'Failed to disable 2FA';
    }
  }

  function formatDate(dateString: string) {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  }

  function handleTotpInput(event: Event) {
    const target = event.target as HTMLInputElement;
    const code = target.value;
    if (code.length === 6 && /^\d{6}$/.test(code)) {
      handle2FADisable(code);
    }
  }
</script>

<div class="max-w-4xl mx-auto py-6 px-4 sm:px-6 lg:px-8">
  <!-- Header -->
  <div class="mb-8">
    <h1 class="text-uswds-2xl font-bold text-uswds-base-darker">Account Settings</h1>
    <p class="mt-1 text-uswds-sm text-uswds-base-darker">
      Manage your account information, security settings, and preferences.
    </p>
  </div>

  <!-- Error/Success Messages -->
  {#if error}
    <div class="mb-6 bg-red-50 border border-red-200 rounded-uswds-md p-uswds-4">
      <div class="flex">
        <svg aria-hidden="true" class="icon-uswds icon-uswds--md text-uswds-red-50" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
        </svg>
        <div class="ml-3">
          <p class="text-uswds-sm text-uswds-red-50">{error}</p>
        </div>
      </div>
    </div>
  {/if}

  {#if success}
    <div class="mb-6 bg-green-50 border border-green-200 rounded-uswds-md p-uswds-4">
      <div class="flex">
        <svg aria-hidden="true" class="icon-uswds icon-uswds--md text-uswds-green-50" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
        </svg>
        <div class="ml-3">
          <p class="text-uswds-sm text-uswds-green-50">{success}</p>
        </div>
      </div>
    </div>
  {/if}

  {#if isLoading && !profile}
    <div class="text-center py-12">
      <div class="animate-spin rounded-full icon icon-xl  border-b-2 border-indigo-600 mx-auto"></div>
      <p class="mt-2 text-uswds-base-darker">Loading profile...</p>
    </div>
  {:else if profile}
    <!-- Tab Navigation -->
    <div class="border-b border-gray-200 mb-6">
      <nav class="-mb-px flex space-x-8">
        <button
          on:click={() => activeTab = 'profile'}
          class="py-2 px-1 border-b-2 font-medium text-uswds-sm {activeTab === 'profile' ? 'border-indigo-500 text-primary' : 'border-transparent text-uswds-base-darker hover:text-gray-700 hover:border-gray-300'}"
        >
          Profile
        </button>
        <button
          on:click={() => activeTab = 'security'}
          class="py-2 px-1 border-b-2 font-medium text-uswds-sm {activeTab === 'security' ? 'border-indigo-500 text-indigo-600' : 'border-transparent text-uswds-base-darker hover:text-gray-700 hover:border-gray-300'}"
        >
          Security
        </button>
        <button
          on:click={() => activeTab = 'accounts'}
          class="py-2 px-1 border-b-2 font-medium text-uswds-sm {activeTab === 'accounts' ? 'border-indigo-500 text-indigo-600' : 'border-transparent text-uswds-base-darker hover:text-gray-700 hover:border-gray-300'}"
        >
          Linked Accounts
        </button>
        <button
          on:click={() => activeTab = 'preferences'}
          class="py-2 px-1 border-b-2 font-medium text-uswds-sm {activeTab === 'preferences' ? 'border-indigo-500 text-indigo-600' : 'border-transparent text-uswds-base-darker hover:text-gray-700 hover:border-gray-300'}"
        >
          Preferences
        </button>
        <button
          on:click={() => activeTab = 'data'}
          class="py-2 px-1 border-b-2 font-medium text-uswds-sm {activeTab === 'data' ? 'border-indigo-500 text-indigo-600' : 'border-transparent text-uswds-base-darker hover:text-gray-700 hover:border-gray-300'}"
        >
          Data & Privacy
        </button>
      </nav>
    </div>

    <!-- Profile Tab -->
    {#if activeTab === 'profile'}
      <div class="bg-white shadow rounded-uswds-lg">
        <div class="px-6 py-4 border-b border-gray-200">
          <h3 class="text-uswds-lg font-medium text-uswds-base-darker">Profile Information</h3>
          <p class="mt-1 text-uswds-sm text-uswds-base-darker">
            Update your account's profile information and email address.
          </p>
        </div>
        <div class="px-6 py-4">
          <div class="space-y-6">
            <!-- Email -->
            <div>
              <label for="email" class="block text-uswds-sm font-medium text-uswds-base-darker">Email</label>
              {#if editingProfile}
                <div class="mt-1 flex rounded-uswds-md shadow-sm">
                  <input
                    type="email"
                    id="email"
                    bind:value={editEmail}
                    class="flex-1 min-w-0 block w-full px-3 py-2 rounded-uswds-md border-gray-300 focus:ring-indigo-500 focus:border-indigo-500 sm:text-uswds-sm"
                    placeholder="Enter your email"
                  />
                </div>
              {:else}
                <div class="mt-1 flex items-center justify-between">
                  <span class="text-uswds-sm text-uswds-base-darker">{profile.email}</span>
                  <button
                    on:click={() => editingProfile = true}
                    class="text-uswds-sm text-indigo-600 hover:text-indigo-500"
                  >
                    Edit
                  </button>
                </div>
              {/if}
            </div>

            <!-- Account Info -->
            <div class="grid grid-cols-1 gap-uswds-6 sm:grid-cols-2">
              <div>
                <span class="block text-uswds-sm font-medium text-uswds-base-darker">Account Created</span>
                <p class="mt-1 text-uswds-sm text-uswds-base-darker">{formatDate(profile.created_at)}</p>
              </div>
              <div>
                <span class="block text-uswds-sm font-medium text-uswds-base-darker">Last Updated</span>
                <p class="mt-1 text-uswds-sm text-uswds-base-darker">{formatDate(profile.updated_at)}</p>
              </div>
              {#if profile.last_login}
                <div>
                  <span class="block text-uswds-sm font-medium text-uswds-base-darker">Last Login</span>
                  <p class="mt-1 text-uswds-sm text-uswds-base-darker">{formatDate(profile.last_login)}</p>
                </div>
              {/if}
            </div>

            <!-- Action Buttons -->
            {#if editingProfile}
              <div class="flex justify-end space-x-3">
                <button
                  on:click={() => {
                    editingProfile = false;
                    editEmail = profile?.email || '';
                  }}
                  class="px-4 py-2 border border-gray-300 rounded-uswds-md text-uswds-sm font-medium text-uswds-base-darker hover:bg-uswds-base-lightest"
                >
                  Cancel
                </button>
                <button
                  on:click={updateProfile}
                  disabled={isLoading}
                  class="px-4 py-2 border border-transparent rounded-uswds-md shadow-sm text-uswds-sm font-medium text-white bg-primary hover:bg-indigo-700 disabled:opacity-50"
                >
                  {isLoading ? 'Saving...' : 'Save Changes'}
                </button>
              </div>
            {/if}
          </div>
        </div>
      </div>
    {/if}

    <!-- Security Tab -->
    {#if activeTab === 'security'}
      <div class="space-y-6">
        <!-- Two-Factor Authentication -->
        <div class="bg-white shadow rounded-uswds-lg">
          <div class="px-6 py-4 border-b border-gray-200">
            <h3 class="text-uswds-lg font-medium text-uswds-base-darker">Two-Factor Authentication</h3>
            <p class="mt-1 text-uswds-sm text-uswds-base-darker">
              Add an extra layer of security to your account with 2FA.
            </p>
          </div>
          <div class="px-6 py-4">
            <div class="flex items-center justify-between">
              <div>
                <p class="text-uswds-sm font-medium text-uswds-base-darker">
                  Status: {profile.totp_enabled ? 'Enabled' : 'Disabled'}
                </p>
                <p class="text-uswds-sm text-uswds-base-darker">
                  {profile.totp_enabled 
                    ? 'Your account is protected with 2FA' 
                    : 'Enable 2FA to secure your account'}
                </p>
              </div>
              <div>
                {#if profile.totp_enabled}
                  <button
                    on:click={() => show2FADisable = true}
                    class="px-4 py-2 border border-red-300 rounded-uswds-md text-uswds-sm font-medium text-uswds-red-50 hover:bg-red-50"
                  >
                    Disable 2FA
                  </button>
                {:else}
                  <button
                    on:click={() => show2FASetup = true}
                    class="px-4 py-2 border border-transparent rounded-uswds-md shadow-sm text-uswds-sm font-medium text-white btn btn-primary"
                  >
                    Enable 2FA
                  </button>
                {/if}
              </div>
            </div>
          </div>
        </div>
      </div>
    {/if}

    <!-- Linked Accounts Tab -->
    {#if activeTab === 'accounts'}
      <div class="bg-white shadow rounded-uswds-lg">
        <div class="px-6 py-4 border-b border-gray-200">
          <h3 class="text-uswds-lg leading-6 font-medium text-uswds-base-darker">
            Linked Accounts
          </h3>
          <p class="mt-1 text-uswds-sm text-uswds-base-darker">
            Manage your connected social accounts for easier sign-in and profile synchronization.
          </p>
        </div>
        <div class="px-6 py-4">
          <button
            type="button"
            on:click={() => showAccountLinking = true}
            class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
          >
            <svg class="-ml-1 mr-2 h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1"></path>
            </svg>
            Manage Linked Accounts
          </button>
          
          {#if profile?.oauth_accounts && profile.oauth_accounts.length > 0}
            <div class="mt-6">
              <h4 class="text-sm font-medium text-gray-900 mb-4">Currently Linked:</h4>
              <div class="space-y-3">
                {#each profile.oauth_accounts as account}
                  <div class="flex items-center justify-between p-3 border border-gray-200 rounded-md">
                    <div class="flex items-center space-x-3">
                      <div class="flex-shrink-0">
                        {#if account.provider === 'google'}
                          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z" fill="#4285F4"/>
                            <path d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z" fill="#34A853"/>
                            <path d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z" fill="#FBBC05"/>
                            <path d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z" fill="#EA4335"/>
                          </svg>
                        {:else if account.provider === 'apple'}
                          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M18.71 19.5c-.83 1.24-1.71 2.45-3.05 2.47-1.34.03-1.77-.79-3.29-.79-1.53 0-2 .77-3.27.82-1.31.05-2.3-1.32-3.14-2.53C4.25 17 2.94 12.45 4.7 9.39c.87-1.52 2.43-2.48 4.12-2.51 1.28-.02 2.5.87 3.29.87.78 0 2.26-1.07 3.81-.91.65.03 2.47.26 3.64 1.98-.09.06-2.17 1.28-2.15 3.81.03 3.02 2.65 4.03 2.68 4.04-.03.07-.42 1.44-1.38 2.83M13 3.5c.73-.83 1.94-1.46 2.94-1.5.13 1.17-.34 2.35-1.04 3.19-.69.85-1.83 1.51-2.95 1.42-.15-1.15.41-2.35 1.05-3.11z"/>
                          </svg>
                        {:else if account.provider === 'github'}
                          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
                          </svg>
                        {/if}
                      </div>
                      <div>
                        <p class="text-sm font-medium text-gray-900 capitalize">
                          {account.provider}
                        </p>
                        <p class="text-sm text-gray-500">
                          {account.display_name || account.email || 'Connected'}
                        </p>
                      </div>
                    </div>
                    <div class="text-sm text-gray-500">
                      Linked {new Date(account.linked_at).toLocaleDateString()}
                    </div>
                  </div>
                {/each}
              </div>
            </div>
          {:else}
            <div class="mt-6 text-center py-8">
              <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1"></path>
              </svg>
              <h3 class="mt-2 text-sm font-medium text-gray-900">No linked accounts</h3>
              <p class="mt-1 text-sm text-gray-500">
                Link your social accounts to sign in more easily.
              </p>
            </div>
          {/if}
        </div>
      </div>
    {/if}

    <!-- Preferences Tab -->
    {#if activeTab === 'preferences'}
      <div class="bg-white shadow rounded-uswds-lg">
        <div class="px-6 py-4 border-b border-gray-200">
          <h3 class="text-uswds-lg font-medium text-uswds-base-darker">Preferences</h3>
          <p class="mt-1 text-uswds-sm text-uswds-base-darker">
            Customize your experience and notification settings.
          </p>
        </div>
        <div class="px-6 py-4">
          <div class="space-y-6">
            <!-- Email Notifications -->
            <div class="flex items-center justify-between">
              <div>
                <span class="text-uswds-sm font-medium text-uswds-base-darker">Email Notifications</span>
                <p class="text-uswds-sm text-uswds-base-darker">Receive email updates about your account activity</p>
              </div>
              <label class="relative flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  bind:checked={editSettings.email_notifications}
                  class="sr-only peer"
                />
                <div class="w-11 icon icon-lg bg-uswds-base-lightest peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-indigo-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:icon icon-md after: after:transition-all peer-checked:bg-indigo-600"></div>
              </label>
            </div>

            <!-- Privacy Mode -->
            <div class="flex items-center justify-between">
              <div>
                <span class="text-uswds-sm font-medium text-uswds-base-darker">Privacy Mode</span>
                <p class="text-uswds-sm text-uswds-base-darker">Hide your activity from other users</p>
              </div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  bind:checked={editSettings.privacy_mode}
                  class="sr-only peer"
                />
                <div class="w-11 h-6 bg-uswds-base-lightest peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-indigo-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-indigo-600"></div>
              </label>
            </div>

            <!-- Save Button -->
            <div class="flex justify-end">
              <button
                on:click={updateSettings}
                disabled={isLoading}
                class="px-4 py-2 border border-transparent rounded-uswds-md shadow-sm text-uswds-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50"
              >
                {isLoading ? 'Saving...' : 'Save Preferences'}
              </button>
            </div>
          </div>
        </div>
      </div>
    {/if}

    <!-- Data & Privacy Tab -->
    {#if activeTab === 'data'}
      <div class="space-y-6">
        <!-- Data Export -->
        <div class="bg-white shadow rounded-uswds-lg">
          <div class="px-6 py-4 border-b border-gray-200">
            <h3 class="text-uswds-lg font-medium text-uswds-base-darker">Data Export</h3>
            <p class="mt-1 text-uswds-sm text-uswds-base-darker">
              Download a copy of all your data for your records.
            </p>
          </div>
          <div class="px-6 py-4">
            <button
              on:click={exportData}
              disabled={exportLoading}
              class="px-4 py-2 border border-transparent rounded-uswds-md shadow-sm text-uswds-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50"
            >
              {exportLoading ? 'Exporting...' : 'Export My Data'}
            </button>
          </div>
        </div>

        <!-- Account Deletion -->
        <div class="bg-white shadow rounded-uswds-lg border-red-200">
          <div class="px-6 py-4 border-b border-red-200">
            <h3 class="text-uswds-lg font-medium text-uswds-red-50">Delete Account</h3>
            <p class="mt-1 text-uswds-sm text-uswds-red-50">
              Permanently delete your account and all associated data.
            </p>
          </div>
          <div class="px-6 py-4">
            {#if !showDeleteConfirm}
              <button
                on:click={() => showDeleteConfirm = true}
                class="px-4 py-2 border border-red-300 rounded-uswds-md text-uswds-sm font-medium text-uswds-red-50 hover:bg-red-50"
              >
                Delete Account
              </button>
            {:else}
              <div class="space-y-4">
                <div class="bg-red-50 border border-red-200 rounded-uswds-md p-uswds-4">
                  <div class="flex">
                    <svg aria-hidden="true" class="icon-uswds icon-uswds--md text-uswds-red-50" viewBox="0 0 20 20" fill="currentColor">
                      <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
                    </svg>
                    <div class="ml-3">
                      <h3 class="text-uswds-sm font-medium text-uswds-red-50">
                        This action cannot be undone
                      </h3>
                      <p class="mt-2 text-uswds-sm text-uswds-red-50">
                        This will permanently delete your account, DNP lists, and all associated data.
                      </p>
                    </div>
                  </div>
                </div>

                <div>
                  <label for="confirm-email" class="block text-uswds-sm font-medium text-uswds-base-darker">
                    Confirm your email address
                  </label>
                  <input
                    type="email"
                    id="confirm-email"
                    bind:value={deleteConfirmEmail}
                    placeholder={profile.email}
                    class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-uswds-md shadow-sm focus:ring-red-500 focus:border-red-500 sm:text-uswds-sm"
                  />
                </div>

                <div>
                  <label for="delete-reason" class="block text-uswds-sm font-medium text-uswds-base-darker">
                    Reason for deletion (optional)
                  </label>
                  <textarea
                    id="delete-reason"
                    bind:value={deleteReason}
                    rows="3"
                    class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-uswds-md shadow-sm focus:ring-red-500 focus:border-red-500 sm:text-uswds-sm"
                    placeholder="Help us improve by telling us why you're leaving..."
                  ></textarea>
                </div>

                <div class="flex justify-end space-x-3">
                  <button
                    on:click={() => {
                      showDeleteConfirm = false;
                      deleteConfirmEmail = '';
                      deleteReason = '';
                    }}
                    class="px-4 py-2 border border-gray-300 rounded-uswds-md text-uswds-sm font-medium text-uswds-base-darker hover:bg-uswds-base-lightest"
                  >
                    Cancel
                  </button>
                  <button
                    on:click={deleteAccount}
                    disabled={deleteLoading || deleteConfirmEmail !== profile.email}
                    class="px-4 py-2 border border-transparent rounded-uswds-md shadow-sm text-uswds-sm font-medium text-white bg-error hover:bg-red-700 disabled:opacity-50"
                  >
                    {deleteLoading ? 'Deleting...' : 'Delete Account'}
                  </button>
                </div>
              </div>
            {/if}
          </div>
        </div>
      </div>
    {/if}
  {/if}
</div>

<!-- 2FA Setup Modal -->
{#if show2FASetup}
  <TwoFactorSetup
    on:complete={handle2FASetupComplete}
    on:cancel={handle2FASetupCancel}
  />
{/if}

<!-- 2FA Disable Modal -->
{#if show2FADisable}
  <div class="fixed inset-0 bg-uswds-base-lightest bg-opacity-50 overflow-y-auto h-full w-full z-50">
    <div class="relative top-uswds-20 mx-auto p-5 border w-96 shadow-lg rounded-uswds-md bg-white">
      <div class="mt-3 text-center">
        <h3 class="text-uswds-lg font-medium text-uswds-base-darker mb-4">
          Disable Two-Factor Authentication
        </h3>
        <p class="text-uswds-sm text-uswds-base-darker mb-6">
          Enter your 2FA code to disable two-factor authentication
        </p>
        
        <div class="mb-4">
          <input
            type="text"
            placeholder="Enter 6-digit code"
            maxlength="6"
            class="w-full px-3 py-2 border border-gray-300 rounded-uswds-md text-center text-uswds-lg tracking-widest"
            on:input={handleTotpInput}
          />
        </div>
        
        <div class="flex justify-center space-x-3">
          <button
            on:click={() => show2FADisable = false}
            class="px-4 py-2 border border-gray-300 rounded-uswds-md text-uswds-sm font-medium text-uswds-base-darker hover:bg-uswds-base-lightest"
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}

<!-- Account Linking Modal -->
<AccountLinking 
  isVisible={showAccountLinking}
  on:close={() => showAccountLinking = false}
/>