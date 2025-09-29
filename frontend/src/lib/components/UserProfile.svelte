<script lang="ts">
  import { onMount } from 'svelte';
  import { authActions } from '../stores/auth';
  import { api } from '../utils/api';
  import TwoFactorSetup from './TwoFactorSetup.svelte';

  interface UserSettings {
    two_factor_enabled: boolean;
    email_notifications: boolean;
    privacy_mode: boolean;
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
    <h1 class="text-2xl font-bold text-gray-900">Account Settings</h1>
    <p class="mt-1 text-sm text-gray-600">
      Manage your account information, security settings, and preferences.
    </p>
  </div>

  <!-- Error/Success Messages -->
  {#if error}
    <div class="mb-6 bg-red-50 border border-red-200 rounded-md p-4">
      <div class="flex">
        <svg class="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
        </svg>
        <div class="ml-3">
          <p class="text-sm text-red-800">{error}</p>
        </div>
      </div>
    </div>
  {/if}

  {#if success}
    <div class="mb-6 bg-green-50 border border-green-200 rounded-md p-4">
      <div class="flex">
        <svg class="h-5 w-5 text-green-400" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
        </svg>
        <div class="ml-3">
          <p class="text-sm text-green-800">{success}</p>
        </div>
      </div>
    </div>
  {/if}

  {#if isLoading && !profile}
    <div class="text-center py-12">
      <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600 mx-auto"></div>
      <p class="mt-2 text-gray-600">Loading profile...</p>
    </div>
  {:else if profile}
    <!-- Tab Navigation -->
    <div class="border-b border-gray-200 mb-6">
      <nav class="-mb-px flex space-x-8">
        <button
          on:click={() => activeTab = 'profile'}
          class="py-2 px-1 border-b-2 font-medium text-sm {activeTab === 'profile' ? 'border-indigo-500 text-indigo-600' : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'}"
        >
          Profile
        </button>
        <button
          on:click={() => activeTab = 'security'}
          class="py-2 px-1 border-b-2 font-medium text-sm {activeTab === 'security' ? 'border-indigo-500 text-indigo-600' : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'}"
        >
          Security
        </button>
        <button
          on:click={() => activeTab = 'preferences'}
          class="py-2 px-1 border-b-2 font-medium text-sm {activeTab === 'preferences' ? 'border-indigo-500 text-indigo-600' : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'}"
        >
          Preferences
        </button>
        <button
          on:click={() => activeTab = 'data'}
          class="py-2 px-1 border-b-2 font-medium text-sm {activeTab === 'data' ? 'border-indigo-500 text-indigo-600' : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'}"
        >
          Data & Privacy
        </button>
      </nav>
    </div>

    <!-- Profile Tab -->
    {#if activeTab === 'profile'}
      <div class="bg-white shadow rounded-lg">
        <div class="px-6 py-4 border-b border-gray-200">
          <h3 class="text-lg font-medium text-gray-900">Profile Information</h3>
          <p class="mt-1 text-sm text-gray-600">
            Update your account's profile information and email address.
          </p>
        </div>
        <div class="px-6 py-4">
          <div class="space-y-6">
            <!-- Email -->
            <div>
              <label for="email" class="block text-sm font-medium text-gray-700">Email</label>
              {#if editingProfile}
                <div class="mt-1 flex rounded-md shadow-sm">
                  <input
                    type="email"
                    id="email"
                    bind:value={editEmail}
                    class="flex-1 min-w-0 block w-full px-3 py-2 rounded-md border-gray-300 focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                    placeholder="Enter your email"
                  />
                </div>
              {:else}
                <div class="mt-1 flex items-center justify-between">
                  <span class="text-sm text-gray-900">{profile.email}</span>
                  <button
                    on:click={() => editingProfile = true}
                    class="text-sm text-indigo-600 hover:text-indigo-500"
                  >
                    Edit
                  </button>
                </div>
              {/if}
            </div>

            <!-- Account Info -->
            <div class="grid grid-cols-1 gap-6 sm:grid-cols-2">
              <div>
                <span class="block text-sm font-medium text-gray-700">Account Created</span>
                <p class="mt-1 text-sm text-gray-900">{formatDate(profile.created_at)}</p>
              </div>
              <div>
                <span class="block text-sm font-medium text-gray-700">Last Updated</span>
                <p class="mt-1 text-sm text-gray-900">{formatDate(profile.updated_at)}</p>
              </div>
              {#if profile.last_login}
                <div>
                  <span class="block text-sm font-medium text-gray-700">Last Login</span>
                  <p class="mt-1 text-sm text-gray-900">{formatDate(profile.last_login)}</p>
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
                  class="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50"
                >
                  Cancel
                </button>
                <button
                  on:click={updateProfile}
                  disabled={isLoading}
                  class="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50"
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
        <div class="bg-white shadow rounded-lg">
          <div class="px-6 py-4 border-b border-gray-200">
            <h3 class="text-lg font-medium text-gray-900">Two-Factor Authentication</h3>
            <p class="mt-1 text-sm text-gray-600">
              Add an extra layer of security to your account with 2FA.
            </p>
          </div>
          <div class="px-6 py-4">
            <div class="flex items-center justify-between">
              <div>
                <p class="text-sm font-medium text-gray-900">
                  Status: {profile.totp_enabled ? 'Enabled' : 'Disabled'}
                </p>
                <p class="text-sm text-gray-600">
                  {profile.totp_enabled 
                    ? 'Your account is protected with 2FA' 
                    : 'Enable 2FA to secure your account'}
                </p>
              </div>
              <div>
                {#if profile.totp_enabled}
                  <button
                    on:click={() => show2FADisable = true}
                    class="px-4 py-2 border border-red-300 rounded-md text-sm font-medium text-red-700 hover:bg-red-50"
                  >
                    Disable 2FA
                  </button>
                {:else}
                  <button
                    on:click={() => show2FASetup = true}
                    class="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700"
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

    <!-- Preferences Tab -->
    {#if activeTab === 'preferences'}
      <div class="bg-white shadow rounded-lg">
        <div class="px-6 py-4 border-b border-gray-200">
          <h3 class="text-lg font-medium text-gray-900">Preferences</h3>
          <p class="mt-1 text-sm text-gray-600">
            Customize your experience and notification settings.
          </p>
        </div>
        <div class="px-6 py-4">
          <div class="space-y-6">
            <!-- Email Notifications -->
            <div class="flex items-center justify-between">
              <div>
                <span class="text-sm font-medium text-gray-900">Email Notifications</span>
                <p class="text-sm text-gray-600">Receive email updates about your account activity</p>
              </div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  bind:checked={editSettings.email_notifications}
                  class="sr-only peer"
                />
                <div class="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-indigo-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-indigo-600"></div>
              </label>
            </div>

            <!-- Privacy Mode -->
            <div class="flex items-center justify-between">
              <div>
                <span class="text-sm font-medium text-gray-900">Privacy Mode</span>
                <p class="text-sm text-gray-600">Hide your activity from other users</p>
              </div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  bind:checked={editSettings.privacy_mode}
                  class="sr-only peer"
                />
                <div class="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-indigo-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-indigo-600"></div>
              </label>
            </div>

            <!-- Save Button -->
            <div class="flex justify-end">
              <button
                on:click={updateSettings}
                disabled={isLoading}
                class="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50"
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
        <div class="bg-white shadow rounded-lg">
          <div class="px-6 py-4 border-b border-gray-200">
            <h3 class="text-lg font-medium text-gray-900">Data Export</h3>
            <p class="mt-1 text-sm text-gray-600">
              Download a copy of all your data for your records.
            </p>
          </div>
          <div class="px-6 py-4">
            <button
              on:click={exportData}
              disabled={exportLoading}
              class="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50"
            >
              {exportLoading ? 'Exporting...' : 'Export My Data'}
            </button>
          </div>
        </div>

        <!-- Account Deletion -->
        <div class="bg-white shadow rounded-lg border-red-200">
          <div class="px-6 py-4 border-b border-red-200">
            <h3 class="text-lg font-medium text-red-900">Delete Account</h3>
            <p class="mt-1 text-sm text-red-600">
              Permanently delete your account and all associated data.
            </p>
          </div>
          <div class="px-6 py-4">
            {#if !showDeleteConfirm}
              <button
                on:click={() => showDeleteConfirm = true}
                class="px-4 py-2 border border-red-300 rounded-md text-sm font-medium text-red-700 hover:bg-red-50"
              >
                Delete Account
              </button>
            {:else}
              <div class="space-y-4">
                <div class="bg-red-50 border border-red-200 rounded-md p-4">
                  <div class="flex">
                    <svg class="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
                      <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
                    </svg>
                    <div class="ml-3">
                      <h3 class="text-sm font-medium text-red-800">
                        This action cannot be undone
                      </h3>
                      <p class="mt-2 text-sm text-red-700">
                        This will permanently delete your account, DNP lists, and all associated data.
                      </p>
                    </div>
                  </div>
                </div>

                <div>
                  <label for="confirm-email" class="block text-sm font-medium text-gray-700">
                    Confirm your email address
                  </label>
                  <input
                    type="email"
                    id="confirm-email"
                    bind:value={deleteConfirmEmail}
                    placeholder={profile.email}
                    class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-red-500 focus:border-red-500 sm:text-sm"
                  />
                </div>

                <div>
                  <label for="delete-reason" class="block text-sm font-medium text-gray-700">
                    Reason for deletion (optional)
                  </label>
                  <textarea
                    id="delete-reason"
                    bind:value={deleteReason}
                    rows="3"
                    class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-red-500 focus:border-red-500 sm:text-sm"
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
                    class="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50"
                  >
                    Cancel
                  </button>
                  <button
                    on:click={deleteAccount}
                    disabled={deleteLoading || deleteConfirmEmail !== profile.email}
                    class="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-red-600 hover:bg-red-700 disabled:opacity-50"
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
  <div class="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
    <div class="relative top-20 mx-auto p-5 border w-96 shadow-lg rounded-md bg-white">
      <div class="mt-3 text-center">
        <h3 class="text-lg font-medium text-gray-900 mb-4">
          Disable Two-Factor Authentication
        </h3>
        <p class="text-sm text-gray-600 mb-6">
          Enter your 2FA code to disable two-factor authentication
        </p>
        
        <div class="mb-4">
          <input
            type="text"
            placeholder="Enter 6-digit code"
            maxlength="6"
            class="w-full px-3 py-2 border border-gray-300 rounded-md text-center text-lg tracking-widest"
            on:input={handleTotpInput}
          />
        </div>
        
        <div class="flex justify-center space-x-3">
          <button
            on:click={() => show2FADisable = false}
            class="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50"
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}