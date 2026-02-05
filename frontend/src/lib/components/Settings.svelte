<script lang="ts">
  import { onMount } from 'svelte';
  import { currentUser, authActions } from '../stores/auth';
  import { navigateTo } from '../utils/simple-router';
  import { apiClient } from '../utils/api-client';
  import { theme, resolvedTheme } from '../stores/theme';

  interface ConnectedAccount {
    provider: string;
    provider_user_id: string;
    email?: string;
    display_name?: string;
    linked_at: string;
  }

  let isLoggingOut = false;
  let connectedAccounts: ConnectedAccount[] = [];
  let isLoadingConnections = true;
  let connectingProvider: string | null = null;
  let connectionError: string | null = null;

  let blockFeatured = true;
  let blockProducers = false;
  let notifications = true;

  let confirmingDisconnect: string | null = null;

  onMount(async () => {
    await loadConnections();
  });

  async function loadConnections() {
    isLoadingConnections = true;
    try {
      const result = await apiClient.get<ConnectedAccount[]>('/api/v1/auth/oauth/accounts');
      if (result.success && result.data) {
        connectedAccounts = result.data;
      }
    } catch (e) {
      console.error('Failed to load connections:', e);
    } finally {
      isLoadingConnections = false;
    }
  }

  function isConnected(provider: string): boolean {
    return connectedAccounts.some(a => a.provider === provider);
  }

  async function initiateOAuth(provider: string) {
    connectingProvider = provider;
    connectionError = null;
    try {
      const result = await apiClient.post<{ auth_url: string }>(`/api/v1/auth/oauth/${provider}/link`);
      if (result.success && result.data?.auth_url) {
        window.location.href = result.data.auth_url;
      } else {
        connectionError = 'Failed to initiate connection';
      }
    } catch (e) {
      connectionError = 'Failed to connect. Please try again.';
    } finally {
      connectingProvider = null;
    }
  }

  function requestDisconnect(provider: string) {
    confirmingDisconnect = provider;
  }

  function cancelDisconnect() {
    confirmingDisconnect = null;
  }

  async function confirmDisconnect(provider: string) {
    confirmingDisconnect = null;
    connectingProvider = provider;
    try {
      const result = await apiClient.delete(`/api/v1/auth/oauth/${provider}/unlink`);
      if (result.success) {
        connectedAccounts = connectedAccounts.filter(a => a.provider !== provider);
      }
    } catch (e) {
      console.error('Disconnect failed:', e);
    } finally {
      connectingProvider = null;
    }
  }

  async function handleLogout() {
    isLoggingOut = true;
    try {
      await authActions.logout();
      window.location.href = '/';
    } catch (e) {
      isLoggingOut = false;
    }
  }
</script>

<div class="settings">
  <div class="settings__container">
    <!-- Header -->
    <div class="settings__header">
      <button
        type="button"
        on:click={() => navigateTo('home')}
        class="settings__back"
      >
        <svg fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
        </svg>
      </button>
      <h1 class="settings__title">Settings</h1>
    </div>

    <div class="settings__sections">
      <!-- Account -->
      <section class="settings__section">
        <div class="settings__section-header">
          <h2 class="settings__section-title">Account</h2>
        </div>
        <div class="settings__section-body">
          <p class="settings__label">Email</p>
          <p class="settings__value">{$currentUser?.email || 'Not signed in'}</p>
        </div>
      </section>

      <!-- Appearance -->
      <section class="settings__section">
        <div class="settings__section-header">
          <h2 class="settings__section-title">Appearance</h2>
        </div>
        <div class="settings__row settings__row--border">
          <div class="settings__row-text">
            <p class="settings__row-label">Theme</p>
            <p class="settings__row-desc">
              {#if $theme === 'system'}
                Following system preference ({$resolvedTheme})
              {:else}
                {$resolvedTheme === 'dark' ? 'Dark' : 'Light'} theme active
              {/if}
            </p>
          </div>
          <div class="settings__theme-btns">
            <button
              type="button"
              class="settings__theme-btn"
              class:settings__theme-btn--active={$theme === 'system'}
              on:click={() => theme.setTheme('system')}
            >Auto</button>
            <button
              type="button"
              class="settings__theme-btn"
              class:settings__theme-btn--active={$theme === 'light'}
              on:click={() => theme.setTheme('light')}
            >Light</button>
            <button
              type="button"
              class="settings__theme-btn"
              class:settings__theme-btn--active={$theme === 'dark'}
              on:click={() => theme.setTheme('dark')}
            >Dark</button>
          </div>
        </div>
      </section>

      <!-- Music Services -->
      <section class="settings__section">
        <div class="settings__section-header">
          <h2 class="settings__section-title">Music Services</h2>
          <p class="settings__section-desc">Connect your streaming accounts to sync your blocklist</p>
        </div>

        {#if connectionError}
          <div class="settings__alert">
            <p>{connectionError}</p>
          </div>
        {/if}

        <div>
          <!-- Spotify -->
          <div class="settings__row settings__row--border">
            <div class="settings__service-info">
              <div class="settings__service-icon settings__service-icon--spotify">
                <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                  <path d="M12 0C5.4 0 0 5.4 0 12s5.4 12 12 12 12-5.4 12-12S18.66 0 12 0zm5.521 17.34c-.24.359-.66.48-1.021.24-2.82-1.74-6.36-2.101-10.561-1.141-.418.122-.779-.179-.899-.539-.12-.421.18-.78.54-.9 4.56-1.021 8.52-.6 11.64 1.32.42.18.479.659.301 1.02zm1.44-3.3c-.301.42-.841.6-1.262.3-3.239-1.98-8.159-2.58-11.939-1.38-.479.12-1.02-.12-1.14-.6-.12-.48.12-1.021.6-1.141C9.6 9.9 15 10.561 18.72 12.84c.361.181.54.78.241 1.2zm.12-3.36C15.24 8.4 8.82 8.16 5.16 9.301c-.6.179-1.2-.181-1.38-.721-.18-.601.18-1.2.72-1.381 4.26-1.26 11.28-1.02 15.721 1.621.539.3.719 1.02.419 1.56-.299.421-1.02.599-1.559.3z"/>
                </svg>
              </div>
              <div>
                <p class="settings__service-name">Spotify</p>
                {#if isLoadingConnections}
                  <p class="settings__service-status">Loading...</p>
                {:else if isConnected('spotify')}
                  <p class="settings__service-status settings__service-status--connected">Connected</p>
                {:else}
                  <p class="settings__service-status">Not connected</p>
                {/if}
              </div>
            </div>
            {#if isConnected('spotify')}
              {#if confirmingDisconnect === 'spotify'}
                <div class="settings__confirm-group">
                  <span class="settings__confirm-label">Disconnect?</span>
                  <button type="button" on:click={() => confirmDisconnect('spotify')} class="settings__confirm-yes">Yes</button>
                  <button type="button" on:click={cancelDisconnect} class="settings__confirm-no">No</button>
                </div>
              {:else}
                <button
                  type="button"
                  on:click={() => requestDisconnect('spotify')}
                  disabled={connectingProvider === 'spotify'}
                  class="settings__service-btn settings__service-btn--disconnect"
                >
                  {connectingProvider === 'spotify' ? 'Disconnecting...' : 'Disconnect'}
                </button>
              {/if}
            {:else}
              <button
                type="button"
                on:click={() => initiateOAuth('spotify')}
                disabled={connectingProvider === 'spotify'}
                class="settings__service-btn settings__service-btn--spotify"
              >
                {connectingProvider === 'spotify' ? 'Connecting...' : 'Connect'}
              </button>
            {/if}
          </div>

          <!-- Apple Music -->
          <div class="settings__row">
            <div class="settings__service-info">
              <div class="settings__service-icon settings__service-icon--apple">
                <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                  <path d="M23.994 6.124a9.23 9.23 0 00-.24-2.19c-.317-1.31-1.062-2.31-2.18-3.043a5.022 5.022 0 00-1.877-.726 10.496 10.496 0 00-1.564-.15c-.04-.003-.083-.01-.124-.013H5.986c-.152.01-.303.017-.455.026-.747.043-1.49.123-2.193.4-1.336.53-2.3 1.452-2.865 2.78-.192.448-.292.925-.363 1.408-.056.392-.088.785-.1 1.18 0 .032-.007.062-.01.093v12.223c.01.14.017.283.027.424.05.815.154 1.624.497 2.373.65 1.42 1.738 2.353 3.234 2.801.42.127.856.187 1.293.228.555.053 1.11.06 1.667.06h11.03a12.5 12.5 0 001.57-.1c.822-.106 1.596-.35 2.295-.81a5.046 5.046 0 001.88-2.207c.186-.42.293-.87.37-1.324.113-.675.138-1.358.137-2.04-.002-3.8 0-7.595-.003-11.393zm-6.423 3.99v5.712c0 .417-.058.827-.244 1.206-.29.59-.76.962-1.388 1.14-.35.1-.706.157-1.07.173-.95.042-1.785-.476-2.144-1.32-.238-.56-.223-1.136-.017-1.7.303-.825.96-1.277 1.743-1.49.294-.08.595-.13.893-.18.323-.054.65-.1.973-.157.274-.048.47-.202.53-.486a.707.707 0 00.017-.146c.002-1.633.002-3.265.002-4.898v-.07l-.06-.01c-2.097.4-4.194.8-6.29 1.202-.014.002-.032.014-.037.026-.006.016-.003.037-.003.056v7.36c0 .418-.052.832-.227 1.218-.282.622-.76 1.02-1.416 1.207-.313.09-.634.138-.96.166-.906.08-1.732-.4-2.134-1.203-.268-.534-.278-1.1-.096-1.66.267-.817.864-1.304 1.64-1.55.376-.12.763-.185 1.148-.25.278-.047.558-.088.832-.145.317-.065.522-.25.58-.574a.504.504 0 00.007-.115v-8.41c0-.25.042-.493.15-.72.183-.385.486-.62.882-.728.17-.047.346-.073.522-.11 2.55-.526 5.1-1.05 7.65-1.573.093-.02.19-.03.285-.03.316.004.528.2.613.5.032.113.044.233.044.35v5.9z"/>
                </svg>
              </div>
              <div>
                <p class="settings__service-name">Apple Music</p>
                {#if isLoadingConnections}
                  <p class="settings__service-status">Loading...</p>
                {:else if isConnected('apple')}
                  <p class="settings__service-status settings__service-status--connected">Connected</p>
                {:else}
                  <p class="settings__service-status">Not connected</p>
                {/if}
              </div>
            </div>
            {#if isConnected('apple')}
              {#if confirmingDisconnect === 'apple'}
                <div class="settings__confirm-group">
                  <span class="settings__confirm-label">Disconnect?</span>
                  <button type="button" on:click={() => confirmDisconnect('apple')} class="settings__confirm-yes">Yes</button>
                  <button type="button" on:click={cancelDisconnect} class="settings__confirm-no">No</button>
                </div>
              {:else}
                <button
                  type="button"
                  on:click={() => requestDisconnect('apple')}
                  disabled={connectingProvider === 'apple'}
                  class="settings__service-btn settings__service-btn--disconnect"
                >
                  {connectingProvider === 'apple' ? 'Disconnecting...' : 'Disconnect'}
                </button>
              {/if}
            {:else}
              <button
                type="button"
                on:click={() => initiateOAuth('apple')}
                disabled={connectingProvider === 'apple'}
                class="settings__service-btn settings__service-btn--apple"
              >
                {connectingProvider === 'apple' ? 'Connecting...' : 'Connect'}
              </button>
            {/if}
          </div>
        </div>
      </section>

      <!-- Preferences -->
      <section class="settings__section">
        <div class="settings__section-header">
          <h2 class="settings__section-title">Preferences</h2>
        </div>
        <div>
          <div class="settings__row settings__row--border">
            <div class="settings__row-text">
              <p class="settings__row-label">Block featured artists</p>
              <p class="settings__row-desc">Also block songs where artist is featured</p>
            </div>
            <button
              type="button"
              on:click={() => blockFeatured = !blockFeatured}
              class="toggle"
              class:toggle--active={blockFeatured}
              role="switch"
              aria-checked={blockFeatured}
            >
              <span class="toggle__knob"></span>
            </button>
          </div>
          <div class="settings__row settings__row--border">
            <div class="settings__row-text">
              <p class="settings__row-label">Block producer credits</p>
              <p class="settings__row-desc">Block songs produced by blocked artists</p>
            </div>
            <button
              type="button"
              on:click={() => blockProducers = !blockProducers}
              class="toggle"
              class:toggle--active={blockProducers}
              role="switch"
              aria-checked={blockProducers}
            >
              <span class="toggle__knob"></span>
            </button>
          </div>
          <div class="settings__row">
            <div class="settings__row-text">
              <p class="settings__row-label">News notifications</p>
              <p class="settings__row-desc">Get notified when new artists are added</p>
            </div>
            <button
              type="button"
              on:click={() => notifications = !notifications}
              class="toggle"
              class:toggle--active={notifications}
              role="switch"
              aria-checked={notifications}
            >
              <span class="toggle__knob"></span>
            </button>
          </div>
        </div>
      </section>

      <!-- Account Actions -->
      <section class="settings__section">
        <div class="settings__section-header">
          <h2 class="settings__section-title">Account Actions</h2>
        </div>
        <div class="settings__actions">
          <button type="button" on:click={handleLogout} disabled={isLoggingOut} class="settings__logout-btn">
            {#if isLoggingOut}
              <div class="settings__spinner"></div>
              <span>Signing out...</span>
            {:else}
              <svg fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
              </svg>
              <span>Sign out</span>
            {/if}
          </button>
          <button type="button" class="settings__delete-btn">
            Delete account
          </button>
        </div>
      </section>

      <p class="settings__version">No Drake in the House v1.0.0</p>
    </div>
  </div>
</div>

<style>
  .settings {
    min-height: 100vh;
    background-color: var(--color-bg-page);
  }

  .settings__container {
    max-width: 36rem;
    margin: 0 auto;
    padding: 1.5rem 1rem 3rem;
  }

  .settings__header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 2rem;
  }

  .settings__back {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 2rem;
    height: 2rem;
    border-radius: var(--radius-md);
    background: none;
    border: none;
    color: var(--color-text-tertiary);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .settings__back:hover {
    color: var(--color-text-primary);
    background-color: var(--color-bg-interactive);
  }

  .settings__back svg {
    width: 1.125rem;
    height: 1.125rem;
    max-width: none;
    max-height: none;
  }

  .settings__title {
    font-size: var(--text-2xl);
    font-weight: 700;
    color: var(--color-text-primary);
    letter-spacing: -0.02em;
  }

  .settings__sections {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }

  /* Section */
  .settings__section {
    background-color: var(--color-bg-elevated);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-2xl);
    overflow: hidden;
  }

  .settings__section-header {
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .settings__section-title {
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--color-text-primary);
    letter-spacing: 0.01em;
  }

  .settings__section-desc {
    font-size: var(--text-xs);
    color: var(--color-text-tertiary);
    margin-top: 0.1875rem;
  }

  .settings__section-body {
    padding: 1rem 1.25rem;
  }

  .settings__label {
    font-size: var(--text-xs);
    color: var(--color-text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    font-weight: 500;
  }

  .settings__value {
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--color-text-primary);
    margin-top: 0.25rem;
  }

  /* Rows */
  .settings__row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.25rem;
    gap: 1rem;
  }

  .settings__row--border {
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .settings__row-text {
    flex: 1;
    min-width: 0;
  }

  .settings__row-label {
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .settings__row-desc {
    font-size: var(--text-xs);
    color: var(--color-text-tertiary);
    margin-top: 0.125rem;
  }

  /* Theme buttons */
  .settings__theme-btns {
    display: flex;
    background-color: var(--color-bg-interactive);
    border-radius: var(--radius-lg);
    padding: 0.1875rem;
    gap: 0.125rem;
  }

  .settings__theme-btn {
    padding: 0.375rem 0.75rem;
    font-family: var(--font-family-sans);
    font-size: var(--text-xs);
    font-weight: 500;
    color: var(--color-text-tertiary);
    background: none;
    border: none;
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .settings__theme-btn:hover:not(.settings__theme-btn--active) {
    color: var(--color-text-secondary);
  }

  .settings__theme-btn--active {
    background-color: var(--color-bg-elevated);
    color: var(--color-text-primary);
    box-shadow: var(--shadow-sm);
  }

  /* Toggle (using design system classes) */

  /* Music Services */
  .settings__service-info {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .settings__service-icon {
    width: 2.5rem;
    height: 2.5rem;
    border-radius: var(--radius-full);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .settings__service-icon svg {
    width: 1.375rem;
    height: 1.375rem;
    max-width: none;
    max-height: none;
    color: white;
  }

  .settings__service-icon--spotify {
    background-color: #1DB954;
  }

  .settings__service-icon--apple {
    background: linear-gradient(135deg, #FA2D48, #A833B9);
  }

  .settings__service-name {
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .settings__service-status {
    font-size: var(--text-xs);
    color: var(--color-text-tertiary);
    margin-top: 0.0625rem;
  }

  .settings__service-status--connected {
    color: var(--color-success);
    font-weight: 500;
  }

  .settings__service-btn {
    flex-shrink: 0;
    padding: 0.375rem 0.875rem;
    font-family: var(--font-family-sans);
    font-size: var(--text-xs);
    font-weight: 600;
    border: none;
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .settings__service-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .settings__service-btn--spotify {
    background-color: #1DB954;
    color: white;
  }

  .settings__service-btn--spotify:hover:not(:disabled) {
    background-color: #1ed760;
  }

  .settings__service-btn--apple {
    background: linear-gradient(90deg, #FA2D48, #A833B9);
    color: white;
  }

  .settings__service-btn--apple:hover:not(:disabled) {
    opacity: 0.9;
  }

  .settings__service-btn--disconnect {
    background-color: var(--color-bg-interactive);
    color: var(--color-text-secondary);
    border: 1px solid var(--color-border-default);
  }

  .settings__service-btn--disconnect:hover:not(:disabled) {
    background-color: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .settings__confirm-group {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    flex-shrink: 0;
  }

  .settings__confirm-label {
    font-size: var(--text-xs);
    color: var(--color-text-tertiary);
    white-space: nowrap;
  }

  .settings__confirm-yes {
    padding: 0.3125rem 0.625rem;
    font-size: var(--text-xs);
    font-weight: 600;
    font-family: var(--font-family-sans);
    color: white;
    background-color: var(--color-brand-primary);
    border: none;
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: background-color var(--transition-fast);
  }

  .settings__confirm-yes:hover {
    background-color: var(--color-brand-primary-hover);
  }

  .settings__confirm-no {
    padding: 0.3125rem 0.625rem;
    font-size: var(--text-xs);
    font-weight: 500;
    font-family: var(--font-family-sans);
    color: var(--color-text-secondary);
    background-color: var(--color-bg-interactive);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .settings__confirm-no:hover {
    background-color: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .settings__alert {
    margin: 0.75rem 1.25rem 0;
    padding: 0.75rem;
    border-radius: var(--radius-lg);
    background-color: var(--color-brand-primary-muted);
    border: 1px solid rgba(244, 63, 94, 0.3);
    font-size: var(--text-sm);
    color: var(--color-brand-primary);
  }

  /* Actions */
  .settings__actions {
    padding: 1rem 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .settings__logout-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    width: 100%;
    padding: 0.75rem;
    font-family: var(--font-family-sans);
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--color-text-secondary);
    background-color: var(--color-bg-interactive);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-lg);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .settings__logout-btn:hover:not(:disabled) {
    background-color: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .settings__logout-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .settings__logout-btn svg {
    width: 1.125rem;
    height: 1.125rem;
    max-width: none;
    max-height: none;
  }

  .settings__spinner {
    width: 1rem;
    height: 1rem;
    border: 2px solid var(--color-text-muted);
    border-top-color: transparent;
    border-radius: var(--radius-full);
    animation: spin 1s linear infinite;
  }

  .settings__delete-btn {
    width: 100%;
    padding: 0.75rem;
    font-family: var(--font-family-sans);
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--color-brand-primary);
    background-color: var(--color-brand-primary-muted);
    border: 1px solid rgba(244, 63, 94, 0.2);
    border-radius: var(--radius-lg);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .settings__delete-btn:hover {
    background-color: var(--color-brand-primary);
    color: white;
    border-color: var(--color-brand-primary);
  }

  .settings__version {
    text-align: center;
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    margin-top: 0.5rem;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
