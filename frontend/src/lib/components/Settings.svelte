<script lang="ts">
  import { onMount } from 'svelte';
  import { currentUser, authActions } from '../stores/auth';
  import { navigateTo } from '../utils/simple-router';
  import {
    connectionsStore,
    connectionActions,
    type ServiceConnection,
  } from '../stores/connections';
  import {
    deriveCanonicalOAuthState,
    getOAuthStateCopy,
    isAlreadyConnectedMessage,
    mapOAuthActionError,
  } from '../utils/oauth-state';
  import { theme, resolvedTheme } from '../stores/theme';
  import {
    billingStore,
    billingActions,
    currentPlan,
    isFreePlan,
  } from '../stores/billing';

  type ServiceId = 'spotify' | 'apple' | 'youtube' | 'tidal' | 'deezer';

  interface ServicePlatform {
    id: ServiceId;
    name: string;
    icon: 'spotify' | 'apple' | 'youtube' | 'tidal' | 'deezer';
    description: string;
    connectedDescription: string;
    connectionProvider?: string;
    statusLabel: string;
    disabled?: boolean;
    catalogOnly?: boolean;
  }

  const services: ServicePlatform[] = [
    {
      id: 'spotify',
      name: 'Spotify',
      icon: 'spotify',
      description: 'Connect Spotify to sync saved tracks, playlists, and followed artists.',
      connectedDescription:
        'Spotify is connected. Use Library Control to import saved tracks, playlists, and followed artists.',
      connectionProvider: 'spotify',
      statusLabel: 'Ready',
    },
    {
      id: 'apple',
      name: 'Apple Music',
      icon: 'apple',
      description: 'Connect your Apple Music account to sync the full library into analysis.',
      connectedDescription:
        'Apple Music is connected. Use Library Control to import and refresh your library.',
      connectionProvider: 'apple_music',
      statusLabel: 'Ready',
    },
    {
      id: 'youtube',
      name: 'YouTube Music',
      icon: 'youtube',
      description: 'Connect YouTube Music to sync playlists, likes, and subscriptions.',
      connectedDescription:
        'YouTube Music is connected. Use Library Control to import your YouTube library.',
      connectionProvider: 'youtube_music',
      statusLabel: 'Ready',
    },
    {
      id: 'tidal',
      name: 'Tidal',
      icon: 'tidal',
      description: 'Connect Tidal to import favorites, albums, artists, and playlists.',
      connectedDescription:
        'Tidal is connected. Use Library Control to import your Tidal favorites.',
      connectionProvider: 'tidal',
      statusLabel: 'Ready',
    },
    {
      id: 'deezer',
      name: 'Deezer',
      icon: 'deezer',
      description: 'Deezer currently powers catalog metadata only.',
      connectedDescription: 'Deezer is available for catalog lookups only.',
      statusLabel: 'Catalog Only',
      catalogOnly: true,
    },
  ];

  let isLoggingOut = false;
  let isLoadingConnections = true;
  let connectingProvider: string | null = null;
  let connectionError: string | null = null;
  let connectionSuccess: string | null = null;
  let confirmingDisconnect: string | null = null;
  let connectionBannerTimeout: ReturnType<typeof setTimeout> | null = null;

  let blockFeatured = true;
  let blockProducers = false;
  let notifications = true;
  let checkoutAction: string | null = null;

  let connectionsByProvider = new Map<string, ServiceConnection>();

  $: connectionsByProvider = new Map<string, ServiceConnection>(
    ($connectionsStore.connections ?? []).map((connection) => [
      connection.provider,
      connection,
    ])
  );

  onMount(async () => {
    const connectionsPromise = loadConnections();
    const billingPromise = billingActions.fetchSubscription();

    // Apple Music prewarm is best-effort and should not block connection state rendering.
    void connectionActions.prepareAppleMusic();

    await Promise.all([connectionsPromise, billingPromise]);
  });

  function planStatusLabel(status: string | undefined): string {
    if (!status) return 'Free';
    switch (status) {
      case 'active': return 'Active';
      case 'trialing': return 'Trialing';
      case 'past_due': return 'Past Due';
      case 'canceled': return 'Canceled';
      default: return status.charAt(0).toUpperCase() + status.slice(1);
    }
  }

  function planStatusTone(status: string | undefined): string {
    if (!status) return 'settings__status-pill--idle';
    switch (status) {
      case 'active':
      case 'trialing':
        return 'settings__status-pill--connected';
      case 'past_due':
        return 'settings__status-pill--warning';
      case 'canceled':
        return 'settings__status-pill--error';
      default:
        return 'settings__status-pill--idle';
    }
  }

  async function handleUpgrade(plan: 'pro' | 'team') {
    checkoutAction = plan;
    await billingActions.initiateCheckout(plan);
    checkoutAction = null;
  }

  async function handleManageSubscription() {
    checkoutAction = 'portal';
    await billingActions.openPortal();
    checkoutAction = null;
  }

  async function loadConnections() {
    isLoadingConnections = true;
    try {
      await connectionActions.fetchConnections();
    } finally {
      isLoadingConnections = false;
    }
  }

  function clearConnectionBannerTimer(): void {
    if (connectionBannerTimeout) {
      clearTimeout(connectionBannerTimeout);
      connectionBannerTimeout = null;
    }
  }

  function showConnectionSuccess(message: string, duration = 5000): void {
    clearConnectionBannerTimer();
    connectionError = null;
    connectionSuccess = message;
    if (duration > 0) {
      connectionBannerTimeout = setTimeout(() => {
        connectionSuccess = null;
        connectionBannerTimeout = null;
      }, duration);
    }
  }

  function showConnectionError(message: string): void {
    clearConnectionBannerTimer();
    connectionSuccess = null;
    connectionError = message;
  }

  function isSuccessLikeMessage(message: string | undefined): boolean {
    if (!message) return false;
    const normalized = message.toLowerCase();
    return (
      normalized.includes('connected successfully') ||
      normalized.includes('disconnected successfully') ||
      normalized.includes('already connected')
    );
  }

  function getConnection(service: ServicePlatform): ServiceConnection | null {
    if (!service.connectionProvider) return null;
    return connectionsByProvider.get(service.connectionProvider) ?? null;
  }

  function isConnected(service: ServicePlatform): boolean {
    return getConnection(service)?.status === 'active';
  }

  function getServiceOAuthState(
    service: ServicePlatform,
    connection: ServiceConnection | null
  ) {
    return deriveCanonicalOAuthState(connection, {
      isAuthorizing: connectingProvider === service.connectionProvider,
      failureHint: connection?.error_code,
    });
  }

  function settingsToneClass(tone: 'idle' | 'connected' | 'warning' | 'error'): string {
    if (tone === 'connected') return 'settings__status-pill--connected';
    if (tone === 'warning') return 'settings__status-pill--warning';
    if (tone === 'error') return 'settings__status-pill--error';
    return 'settings__status-pill--idle';
  }

  function serviceStatusLabel(
    service: ServicePlatform,
    connection: ServiceConnection | null
  ): string {
    if (service.disabled) return service.statusLabel;
    if (service.catalogOnly) return service.statusLabel;
    if (isLoadingConnections && !connection) return 'Checking...';

    return getOAuthStateCopy(
      getServiceOAuthState(service, connection),
      service.name
    ).label;
  }

  function serviceStatusTone(
    service: ServicePlatform,
    connection: ServiceConnection | null
  ): string {
    if (service.disabled) return 'settings__status-pill--paused';
    if (service.catalogOnly) return 'settings__status-pill--catalog';
    return settingsToneClass(
      getOAuthStateCopy(getServiceOAuthState(service, connection), service.name).tone
    );
  }

  function requestDisconnect(serviceId: ServiceId) {
    confirmingDisconnect = serviceId;
  }

  function cancelDisconnect() {
    confirmingDisconnect = null;
  }

  async function connectService(service: ServicePlatform) {
    if (service.catalogOnly) return;

    const provider = service.connectionProvider;
    if (!provider) return;

    // Check connection limit before connecting
    const access = await billingActions.checkFeature('connections');
    if (!access.allowed) {
      showConnectionError(access.reason || 'Connection limit reached. Upgrade your plan.');
      return;
    }

    connectingProvider = provider;
    connectionError = null;
    connectionSuccess = null;

    try {
      if (service.id === 'apple') {
        const result = await connectionActions.connectAppleMusic();
        if (result.success || isSuccessLikeMessage(result.message)) {
          await loadConnections();
          showConnectionSuccess(
            `${service.name} connected. Open Library Control to start syncing.`
          );
        } else if (isAlreadyConnectedMessage(result.message)) {
          showConnectionSuccess(
            `${service.name} is already connected. Open Library Control to sync or disconnect first.`
          );
        } else {
          showConnectionError(mapOAuthActionError(service.name, 'connect', result.message));
        }
        return;
      }

      if (service.id === 'spotify') {
        const result = await connectionActions.initiateSpotifyAuth();
        if (!result.success) {
          if (isAlreadyConnectedMessage(result.message)) {
            showConnectionSuccess(
              `${service.name} is already connected. Open Library Control to sync or disconnect first.`
            );
          } else {
            showConnectionError(mapOAuthActionError(service.name, 'connect', result.message));
          }
        }
        return;
      }

      if (service.id === 'youtube') {
        const result = await connectionActions.initiateYouTubeAuth();
        if (!result.success) {
          if (isAlreadyConnectedMessage(result.message)) {
            showConnectionSuccess(
              `${service.name} is already connected. Open Library Control to sync or disconnect first.`
            );
          } else {
            showConnectionError(mapOAuthActionError(service.name, 'connect', result.message));
          }
        }
        return;
      }

      if (service.id === 'tidal') {
        const result = await connectionActions.initiateTidalAuth();
        if (!result.success) {
          if (isAlreadyConnectedMessage(result.message)) {
            showConnectionSuccess(
              `${service.name} is already connected. Open Library Control to sync or disconnect first.`
            );
          } else {
            showConnectionError(mapOAuthActionError(service.name, 'connect', result.message));
          }
        }
      }
    } catch (error) {
      const hint = error instanceof Error ? error.message : '';
      showConnectionError(mapOAuthActionError(service.name, 'connect', hint));
    } finally {
      connectingProvider = null;
    }
  }

  async function confirmDisconnect(service: ServicePlatform) {
    confirmingDisconnect = null;

    const provider = service.connectionProvider;
    if (!provider) return;

    connectingProvider = provider;
    try {
      let result: { success: boolean; message?: string };

      if (service.id === 'apple') {
        result = await connectionActions.disconnectAppleMusic();
      } else if (service.id === 'youtube') {
        result = await connectionActions.disconnectYouTube();
      } else if (service.id === 'tidal') {
        result = await connectionActions.disconnectTidal();
      } else {
        result = await connectionActions.disconnectSpotify();
      }

      if (result.success || isSuccessLikeMessage(result.message)) {
        await loadConnections();
        showConnectionSuccess(`${service.name} disconnected.`);
      } else {
        showConnectionError(mapOAuthActionError(service.name, 'disconnect', result.message));
      }
    } catch (error) {
      const hint = error instanceof Error ? error.message : '';
      showConnectionError(mapOAuthActionError(service.name, 'disconnect', hint));
    } finally {
      connectingProvider = null;
    }
  }

  async function handleLogout() {
    isLoggingOut = true;
    try {
      await authActions.logout();
      window.location.href = '/';
    } catch (_error) {
      isLoggingOut = false;
    }
  }
</script>

<div class="settings brand-page surface-page">
  <div class="settings__container brand-page__inner brand-page__inner--narrow brand-page__stack">
    <section class="brand-hero settings__hero">
      <div class="brand-hero__header">
        <div class="brand-hero__copy">
          <button type="button" on:click={() => navigateTo('home')} class="brand-back">
            <svg fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M15 19l-7-7 7-7"
              />
            </svg>
            Back to Home
          </button>
          <div class="brand-kickers">
            <span class="brand-kicker">Account Controls</span>
            <span class="brand-kicker brand-kicker--accent">Services + Preferences</span>
          </div>
          <h1 class="brand-title brand-title--compact">
            Tune the account, not just the blocklist.
          </h1>
          <p class="brand-subtitle">
            Keep your theme, linked services, preferences, and developer tools on the same visual
            system as the rest of the product.
          </p>
          {#if $currentUser?.email}
            <div class="brand-meta">
              <span class="brand-meta__item">{$currentUser.email}</span>
            </div>
          {/if}
        </div>
      </div>
    </section>

    <div class="settings__sections">
      <section class="settings__section">
        <div class="settings__section-header">
          <h2 class="settings__section-title">Account</h2>
        </div>
        <div class="settings__section-body">
          <p class="settings__label">Email</p>
          <p class="settings__value">{$currentUser?.email || 'Not signed in'}</p>
        </div>
      </section>

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
              on:click={() => theme.setTheme('system')}>Auto</button
            >
            <button
              type="button"
              class="settings__theme-btn"
              class:settings__theme-btn--active={$theme === 'light'}
              on:click={() => theme.setTheme('light')}>Light</button
            >
            <button
              type="button"
              class="settings__theme-btn"
              class:settings__theme-btn--active={$theme === 'dark'}
              on:click={() => theme.setTheme('dark')}>Dark</button
            >
          </div>
        </div>
      </section>

      <section class="settings__section">
        <div class="settings__section-header">
          <h2 class="settings__section-title">Music Services</h2>
          <p class="settings__section-desc">
            Connection states and service badges now match Library Control.
          </p>
        </div>

        {#if connectionError}
          <div class="brand-alert brand-alert--error settings__banner">
            <span aria-hidden="true">✕</span>
            <span>{connectionError}</span>
            <button type="button" on:click={() => (connectionError = null)} class="brand-alert__dismiss">
              Dismiss
            </button>
          </div>
        {/if}

        {#if connectionSuccess}
          <div class="brand-alert brand-alert--success settings__banner">
            <svg
              class="settings__banner-check"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
              aria-hidden="true"
            >
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
            </svg>
            <span>{connectionSuccess}</span>
            <button type="button" on:click={() => (connectionSuccess = null)} class="brand-alert__dismiss">
              Dismiss
            </button>
          </div>
        {/if}

        <div class="settings__service-grid">
          {#each services as service}
            {@const connection = getConnection(service)}
            {@const connected = isConnected(service)}
            <article class="settings__service-card">
              <div class="settings__service-top">
                <div
                  class="settings__service-avatar"
                  style={`--svc-color: var(--color-${service.id})`}
                >
                  {#if service.icon === 'spotify'}
                    <svg class="settings__service-glyph" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                      <path d="M12 0C5.4 0 0 5.4 0 12s5.4 12 12 12 12-5.4 12-12S18.66 0 12 0zm5.521 17.34c-.24.359-.66.48-1.021.24-2.82-1.74-6.36-2.101-10.561-1.141-.418.122-.779-.179-.899-.539-.12-.421.18-.78.54-.9 4.56-1.021 8.52-.6 11.64 1.32.42.18.479.659.301 1.02zm1.44-3.3c-.301.42-.841.6-1.262.3-3.239-1.98-8.159-2.58-11.939-1.38-.479.12-1.02-.12-1.14-.6-.12-.48.12-1.021.6-1.141C9.6 9.9 15 10.561 18.72 12.84c.361.181.54.78.241 1.2zm.12-3.36C15.24 8.4 8.82 8.16 5.16 9.301c-.6.179-1.2-.181-1.38-.721-.18-.601.18-1.2.72-1.381 4.26-1.26 11.28-1.02 15.721 1.621.539.3.719 1.02.42 1.56-.299.421-1.02.599-1.559.3z" />
                    </svg>
                  {:else if service.icon === 'apple'}
                    <svg class="settings__service-glyph" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                      <path d="M18.71 19.5c-.83 1.24-1.71 2.45-3.05 2.47-1.34.03-1.77-.79-3.29-.79-1.53 0-2 .77-3.27.82-1.31.05-2.3-1.32-3.14-2.53C4.25 17 2.94 12.45 4.7 9.39c.87-1.52 2.43-2.48 4.12-2.51 1.28-.02 2.5.87 3.29.87.78 0 2.26-1.07 3.81-.91.65.03 2.47.26 3.64 1.98-.09.06-2.17 1.28-2.15 3.81.03 3.02 2.65 4.03 2.68 4.04-.03.07-.42 1.44-1.38 2.83M13 3.5c.73-.83 1.94-1.46 2.94-1.5.13 1.17-.34 2.35-1.04 3.19-.69.85-1.83 1.51-2.95 1.42-.15-1.15.41-2.35 1.05-3.11z" />
                    </svg>
                  {:else if service.icon === 'youtube'}
                    <svg class="settings__service-glyph" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                      <path d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z" />
                    </svg>
                  {:else if service.icon === 'tidal'}
                    <svg class="settings__service-glyph" fill="white" viewBox="0 0 24 24" aria-hidden="true">
                      <path d="M12.012 3.992L8.008 7.996 4.004 3.992 0 7.996 4.004 12l4.004-4.004L12.012 12l4.004-4.004L12.012 3.992zM12.012 12l-4.004 4.004L12.012 20.008l4.004-4.004L12.012 12zM20.02 7.996L16.016 3.992l-4.004 4.004 4.004 4.004 4.004-4.004L24.024 3.992 20.02 7.996z" />
                    </svg>
                  {:else}
                    <svg class="settings__service-glyph" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                      <path d="M18.81 4.16v3.03H24V4.16h-5.19zM6.27 8.38v3.027h5.189V8.38h-5.19zm12.54 0v3.027H24V8.38h-5.19zM6.27 12.594v3.027h5.189v-3.027h-5.19zm6.271 0v3.027h5.19v-3.027h-5.19zm6.27 0v3.027H24v-3.027h-5.19zM0 16.81v3.028h5.19v-3.027H0zm6.27 0v3.028h5.189v-3.027h-5.19zm6.271 0v3.028h5.19v-3.027h-5.19zm6.27 0v3.028H24v-3.027h-5.19z" />
                    </svg>
                  {/if}
                </div>

                <div class="settings__service-copy">
                  <div class="settings__service-title-row">
                    <h3 class="settings__service-name">{service.name}</h3>
                    <span class={`settings__status-pill ${serviceStatusTone(service, connection)}`}>
                      {serviceStatusLabel(service, connection)}
                    </span>
                  </div>
                  <p class="settings__service-desc">
                    {connected ? service.connectedDescription : service.description}
                  </p>
                </div>
              </div>

              <div class="settings__service-actions">
                {#if service.catalogOnly}
                  <button type="button" class="settings__service-btn settings__service-btn--secondary" disabled>
                    Catalog Only
                  </button>
                {:else if service.disabled}
                  <button type="button" class="settings__service-btn settings__service-btn--paused" disabled>
                    Paused
                  </button>
                {:else if connected}
                  {#if confirmingDisconnect === service.id}
                    <div class="settings__confirm-strip">
                      <span class="settings__confirm-label">Disconnect {service.name}?</span>
                      <button type="button" on:click={() => confirmDisconnect(service)} class="settings__confirm-yes">
                        Yes
                      </button>
                      <button type="button" on:click={cancelDisconnect} class="settings__confirm-no">
                        No
                      </button>
                    </div>
                  {:else}
                    <button
                      type="button"
                      class="settings__service-btn settings__service-btn--secondary"
                      on:click={() => navigateTo('sync')}
                    >
                      Open Library
                    </button>
                    <button
                      type="button"
                      class="settings__service-btn settings__service-btn--disconnect"
                      on:click={() => requestDisconnect(service.id)}
                      disabled={connectingProvider === service.connectionProvider}
                    >
                      {connectingProvider === service.connectionProvider ? 'Disconnecting...' : 'Disconnect'}
                    </button>
                  {/if}
                {:else}
                  <button
                    type="button"
                    class={`settings__service-btn ${
                      service.icon === 'tidal'
                        ? 'settings__service-btn--tidal'
                        : service.icon === 'youtube'
                          ? 'settings__service-btn--youtube'
                          : 'settings__service-btn--primary'
                    }`}
                    on:click={() => connectService(service)}
                    disabled={connectingProvider === service.connectionProvider}
                  >
                    {connectingProvider === service.connectionProvider ? 'Connecting...' : 'Connect Account'}
                  </button>
                {/if}
              </div>
            </article>
          {/each}
        </div>
      </section>

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
              on:click={() => (blockFeatured = !blockFeatured)}
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
              on:click={() => (blockProducers = !blockProducers)}
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
              on:click={() => (notifications = !notifications)}
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

      <section class="settings__section">
        <div class="settings__section-header">
          <h2 class="settings__section-title">Plan & Billing</h2>
          <p class="settings__section-desc">Manage your subscription and feature limits</p>
        </div>

        <div class="settings__row settings__row--border">
          <div class="settings__row-text">
            <p class="settings__row-label">
              {$billingStore.subscription
                ? $billingStore.subscription.activePlan.charAt(0).toUpperCase() + $billingStore.subscription.activePlan.slice(1) + ' Plan'
                : 'Free Plan'}
            </p>
            <p class="settings__row-desc">
              {#if $billingStore.subscription?.currentPeriodEnd}
                {$billingStore.subscription.cancelAtPeriodEnd
                  ? 'Cancels'
                  : 'Renews'} {new Date($billingStore.subscription.currentPeriodEnd).toLocaleDateString()}
              {:else}
                No active subscription
              {/if}
            </p>
          </div>
          <span class={`settings__status-pill ${planStatusTone($billingStore.subscription?.status)}`}>
            {planStatusLabel($billingStore.subscription?.status)}
          </span>
        </div>

        <div class="settings__row settings__row--border">
          <div class="settings__row-text" style="width:100%">
            <div class="brand-stat-grid brand-stat-grid--compact" aria-label="Usage overview" style="grid-template-columns:repeat(3,1fr);">
              <div class="brand-stat">
                <span class="brand-stat__value">
                  {$isFreePlan ? '1' : '5'}/{$isFreePlan ? '1' : '5'}
                </span>
                <span class="brand-stat__label">Connections</span>
              </div>
              <div class="brand-stat">
                <span class="brand-stat__value">
                  {$isFreePlan ? '1' : '∞'}
                </span>
                <span class="brand-stat__label">Monthly Scans</span>
              </div>
              <div class="brand-stat">
                <span class="brand-stat__value">
                  {$currentPlan === 'free' ? 'Free' : $currentPlan === 'pro' ? 'Pro' : 'Team'}
                </span>
                <span class="brand-stat__label">Current Plan</span>
              </div>
            </div>
          </div>
        </div>

        {#if $billingStore.subscription?.cancelAtPeriodEnd}
          <div class="brand-alert brand-alert--warning" style="margin:1rem 1.25rem 0;">
            <svg fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true" style="width:1.1rem;height:1.1rem;flex-shrink:0;">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
            <span>Your subscription is set to cancel at the end of the current period. You'll lose access to Pro features.</span>
          </div>
        {/if}

        {#if $billingStore.error}
          <div class="brand-alert brand-alert--error" style="margin:1rem 1.25rem 0;">
            <span aria-hidden="true">✕</span>
            <span>{$billingStore.error}</span>
          </div>
        {/if}

        <div class="settings__row">
          {#if $isFreePlan}
            <button
              type="button"
              class="settings__service-btn settings__service-btn--primary"
              on:click={() => handleUpgrade('pro')}
              disabled={checkoutAction !== null}
            >
              {checkoutAction === 'pro' ? 'Redirecting...' : 'Upgrade to Pro'}
            </button>
          {:else}
            <button
              type="button"
              class="settings__service-btn settings__service-btn--secondary"
              on:click={handleManageSubscription}
              disabled={checkoutAction !== null}
            >
              {checkoutAction === 'portal' ? 'Redirecting...' : 'Manage Subscription'}
            </button>
          {/if}
        </div>

        <div class="settings__plan-grid">
          <article class="settings__service-card" class:settings__plan-card--active={$currentPlan === 'free'}>
            <div class="settings__service-top">
              <div class="settings__service-copy">
                <h3 class="settings__service-name">Free</h3>
                <p class="settings__service-desc" style="margin-top:0.25rem;">$0 / month</p>
              </div>
            </div>
            <ul class="settings__plan-features">
              <li>1 music service connection</li>
              <li>1 library scan per month</li>
              <li>3 playlist grades</li>
              <li>1 community list subscription</li>
            </ul>
            {#if $currentPlan === 'free'}
              <span class="settings__service-btn settings__service-btn--secondary" style="text-align:center;cursor:default;">Current Plan</span>
            {:else}
              <span class="settings__service-btn settings__service-btn--secondary" style="text-align:center;opacity:0.5;cursor:default;">—</span>
            {/if}
          </article>

          <article class="settings__service-card" class:settings__plan-card--active={$currentPlan === 'pro'}>
            <div class="settings__service-top">
              <div class="settings__service-copy">
                <h3 class="settings__service-name">Pro</h3>
                <p class="settings__service-desc" style="margin-top:0.25rem;">$5 / month</p>
              </div>
            </div>
            <ul class="settings__plan-features">
              <li>5 music service connections</li>
              <li>Unlimited library scans</li>
              <li>Unlimited playlist grades</li>
              <li>Unlimited community list subscriptions</li>
              <li>Create community lists</li>
              <li>Auto-enforcement</li>
              <li>Data export</li>
            </ul>
            {#if $currentPlan === 'pro'}
              <span class="settings__service-btn settings__service-btn--secondary" style="text-align:center;cursor:default;">Current Plan</span>
            {:else}
              <button
                type="button"
                class="settings__service-btn settings__service-btn--primary"
                on:click={() => handleUpgrade('pro')}
                disabled={checkoutAction !== null}
              >
                {checkoutAction === 'pro' ? 'Redirecting...' : 'Upgrade'}
              </button>
            {/if}
          </article>

          <article class="settings__service-card" class:settings__plan-card--active={$currentPlan === 'team'}>
            <div class="settings__service-top">
              <div class="settings__service-copy">
                <h3 class="settings__service-name">Team</h3>
                <p class="settings__service-desc" style="margin-top:0.25rem;">$12 / month</p>
              </div>
            </div>
            <ul class="settings__plan-features">
              <li>Everything in Pro</li>
              <li>Up to 5 team seats</li>
              <li>Shared blocklists</li>
              <li>Team management</li>
            </ul>
            {#if $currentPlan === 'team'}
              <span class="settings__service-btn settings__service-btn--secondary" style="text-align:center;cursor:default;">Current Plan</span>
            {:else}
              <button
                type="button"
                class="settings__service-btn settings__service-btn--primary"
                on:click={() => handleUpgrade('team')}
                disabled={checkoutAction !== null}
              >
                {checkoutAction === 'team' ? 'Redirecting...' : 'Upgrade'}
              </button>
            {/if}
          </article>
        </div>
      </section>

      <section class="settings__section">
        <div class="settings__section-header">
          <h2 class="settings__section-title">Developer</h2>
          <p class="settings__section-desc">Tools for monitoring and debugging</p>
        </div>
        <div class="settings__row">
          <div class="settings__row-text">
            <p class="settings__row-label">Service Health</p>
            <p class="settings__row-desc">Monitor backend database and service status</p>
          </div>
          <button
            type="button"
            on:click={() => navigateTo('service-health')}
            class="settings__service-btn settings__service-btn--secondary"
          >
            View
          </button>
        </div>
      </section>

      <section class="settings__section">
        <div class="settings__section-header">
          <h2 class="settings__section-title">Account Actions</h2>
        </div>
        <div class="settings__actions">
          <button
            type="button"
            on:click={handleLogout}
            disabled={isLoggingOut}
            class="settings__logout-btn"
          >
            {#if isLoggingOut}
              <div class="settings__spinner"></div>
              <span>Signing out...</span>
            {:else}
              <svg fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"
                />
              </svg>
              <span>Sign out</span>
            {/if}
          </button>
          <button type="button" class="settings__delete-btn">Delete account</button>
        </div>
      </section>

      <p class="settings__version">No Drake in the House v1.0.0</p>
    </div>
  </div>
</div>

<style>
  .settings {
    min-height: 100vh;
  }

  .settings__container {
    width: 100%;
  }

  .settings__hero {
    max-width: 100%;
  }

  .settings__sections {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }

  .settings__section {
    background: var(--color-bg-elevated, var(--color-bg-surface));
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-2xl);
    overflow: hidden;
    box-shadow: var(--shadow-lg, 0 20px 48px rgba(0, 0, 0, 0.08));
  }

  .settings__section-header {
    padding: 1.1rem 1.25rem;
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .settings__section-title {
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--color-text-primary);
    letter-spacing: 0.01em;
  }

  .settings__section-desc {
    margin-top: 0.1875rem;
    font-size: var(--text-xs);
    color: var(--color-text-tertiary);
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
    margin-top: 0.25rem;
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .settings__row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 1rem 1.25rem;
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
    margin-top: 0.125rem;
    font-size: var(--text-xs);
    color: var(--color-text-tertiary);
  }

  .settings__theme-btns {
    display: flex;
    gap: 0.125rem;
    padding: 0.1875rem;
    background: var(--color-bg-interactive, var(--color-bg-elevated));
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-lg);
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
    color: var(--color-text-primary);
    background: var(--color-bg-hover);
    box-shadow: var(--shadow-sm);
  }

  .settings__banner {
    margin: 1rem 1.25rem 0;
  }

  .settings__banner-check {
    width: 1.25rem;
    height: 1.25rem;
    flex-shrink: 0;
  }

  .settings__service-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 1rem;
    padding: 1.25rem;
  }

  .settings__service-card {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    gap: 1rem;
    min-height: 15rem;
    padding: 1.1rem;
    background: var(--color-bg-interactive, var(--color-bg-elevated));
    border: 1px solid var(--color-border-default);
    border-radius: 1.1rem;
    transition: border-color var(--transition-fast), box-shadow var(--transition-fast);
  }

  .settings__service-card:hover {
    border-color: var(--color-border-hover);
    box-shadow: var(--shadow-md);
  }

  .settings__service-top {
    display: flex;
    gap: 0.9rem;
    align-items: flex-start;
  }

  .settings__service-avatar {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 3rem;
    height: 3rem;
    flex-shrink: 0;
    border: 1px solid color-mix(in srgb, var(--svc-color) 25%, transparent);
    border-radius: 0.95rem;
    background-color: color-mix(in srgb, var(--svc-color) 12%, transparent);
    color: var(--svc-color);
  }

  .settings__service-glyph {
    width: 1.55rem;
    height: 1.55rem;
    flex-shrink: 0;
  }

  .settings__service-copy {
    min-width: 0;
    flex: 1;
  }

  .settings__service-title-row {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    flex-wrap: wrap;
  }

  .settings__service-name {
    font-size: 1rem;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .settings__service-desc {
    margin-top: 0.45rem;
    font-size: 0.83rem;
    line-height: 1.55;
    color: var(--color-text-secondary);
  }

  .settings__status-pill {
    display: inline-flex;
    align-items: center;
    padding: 0.22rem 0.55rem;
    border: 1px solid transparent;
    border-radius: 999px;
    font-size: 0.72rem;
    font-weight: 600;
    letter-spacing: 0.02em;
  }

  .settings__status-pill--connected {
    color: var(--color-success);
    background: var(--color-success-muted);
    border-color: color-mix(in srgb, var(--color-success) 20%, transparent);
  }

  .settings__status-pill--warning {
    color: var(--color-warning);
    background: var(--color-warning-muted);
    border-color: color-mix(in srgb, var(--color-warning) 22%, transparent);
  }

  .settings__status-pill--error {
    color: var(--color-brand-accent);
    background: var(--color-brand-primary-muted);
    border-color: color-mix(in srgb, var(--color-brand-accent) 24%, transparent);
  }

  .settings__status-pill--paused {
    color: var(--color-warning);
    background: var(--color-warning-muted);
    border-color: color-mix(in srgb, var(--color-warning) 20%, transparent);
  }

  .settings__status-pill--catalog {
    color: var(--color-text-secondary);
    background: var(--color-bg-interactive);
    border-color: var(--color-border-default);
  }

  .settings__status-pill--idle {
    color: var(--color-text-secondary);
    background: var(--color-bg-hover);
    border-color: var(--color-border-default);
  }

  .settings__service-actions {
    display: flex;
    gap: 0.7rem;
    align-items: center;
    flex-wrap: wrap;
  }

  .settings__service-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-height: 2.625rem;
    padding: 0.675rem 1rem;
    font-family: var(--font-family-sans);
    font-size: var(--text-xs);
    font-weight: 700;
    border: 1px solid transparent;
    border-radius: 0.9rem;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .settings__service-btn:disabled {
    opacity: 0.65;
    cursor: not-allowed;
  }

  .settings__service-btn--primary {
    color: var(--color-text-on-brand);
    background: var(--color-brand-primary);
    box-shadow: 0 12px 28px color-mix(in srgb, var(--color-brand-primary) 22%, transparent);
  }

  .settings__service-btn--primary:hover:not(:disabled) {
    background: var(--color-brand-primary-hover);
    transform: translateY(-1px);
    box-shadow: 0 18px 34px color-mix(in srgb, var(--color-brand-primary) 26%, transparent);
  }

  .settings__service-btn--youtube {
    color: var(--color-text-on-brand);
    background: var(--color-youtube);
    box-shadow: 0 12px 28px color-mix(in srgb, var(--color-youtube) 18%, transparent);
  }

  .settings__service-btn--youtube:hover:not(:disabled) {
    transform: translateY(-1px);
    box-shadow: 0 18px 34px color-mix(in srgb, var(--color-youtube) 22%, transparent);
  }

  .settings__service-btn--tidal {
    color: var(--color-text-on-brand);
    background: var(--color-tidal);
    border-color: var(--color-border-default);
  }

  .settings__service-btn--tidal:hover:not(:disabled) {
    transform: translateY(-1px);
    border-color: var(--color-border-hover);
  }

  .settings__service-btn--paused {
    color: var(--color-warning);
    background: var(--color-warning-muted);
    border-color: color-mix(in srgb, var(--color-warning) 16%, transparent);
  }

  .settings__service-btn--secondary {
    color: var(--color-text-primary);
    background: var(--color-bg-interactive, var(--color-bg-elevated));
    border-color: var(--color-border-default);
  }

  .settings__service-btn--secondary:hover:not(:disabled) {
    background: var(--color-bg-hover);
  }

  .settings__service-btn--disconnect {
    color: var(--color-error);
    background: var(--color-error-muted);
    border-color: color-mix(in srgb, var(--color-error) 16%, transparent);
  }

  .settings__service-btn--disconnect:hover:not(:disabled) {
    opacity: 0.9;
  }

  .settings__confirm-strip {
    display: flex;
    gap: 0.45rem;
    align-items: center;
    flex-wrap: wrap;
  }

  .settings__confirm-label {
    font-size: var(--text-xs);
    color: var(--color-text-tertiary);
  }

  .settings__confirm-yes,
  .settings__confirm-no {
    padding: 0.45rem 0.7rem;
    font-size: var(--text-xs);
    font-weight: 700;
    border-radius: 0.75rem;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .settings__confirm-yes {
    color: var(--color-text-on-brand);
    background: var(--color-brand-primary);
    border: none;
  }

  .settings__confirm-no {
    color: var(--color-text-secondary);
    background: var(--color-bg-interactive, var(--color-bg-elevated));
    border: 1px solid var(--color-border-default);
  }

  .settings__actions {
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
    padding: 1rem 1.25rem;
  }

  .settings__logout-btn,
  .settings__delete-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 0.55rem;
    width: 100%;
    min-height: 2.9rem;
    padding: 0.8rem 1rem;
    font-family: var(--font-family-sans);
    font-size: var(--text-sm);
    font-weight: 600;
    border-radius: 1rem;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .settings__logout-btn {
    color: var(--color-text-primary);
    background: var(--color-bg-interactive, var(--color-bg-elevated));
    border: 1px solid var(--color-border-default);
  }

  .settings__logout-btn:hover:not(:disabled) {
    background: var(--color-bg-hover);
  }

  .settings__logout-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .settings__logout-btn svg {
    width: 1.1rem;
    height: 1.1rem;
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
    color: var(--color-error);
    background: var(--color-error-muted);
    border: 1px solid color-mix(in srgb, var(--color-error) 16%, transparent);
  }

  .settings__delete-btn:hover {
    opacity: 0.9;
  }

  .settings__version {
    margin-top: 0.5rem;
    text-align: center;
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }

  .settings__plan-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1rem;
    padding: 1.25rem;
  }

  @media (max-width: 640px) {
    .settings__plan-grid {
      grid-template-columns: 1fr;
    }
  }

  .settings__plan-card--active {
    border-color: color-mix(in srgb, var(--color-brand-primary) 40%, transparent);
  }

  .settings__plan-features {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    flex: 1;
  }

  .settings__plan-features li {
    font-size: 0.8rem;
    color: var(--color-text-secondary);
    padding-left: 1rem;
    position: relative;
  }

  .settings__plan-features li::before {
    content: '✓';
    position: absolute;
    left: 0;
    color: var(--color-success);
    font-size: 0.7rem;
    font-weight: 700;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }

    to {
      transform: rotate(360deg);
    }
  }

  @media (max-width: 768px) {
    .settings__row {
      align-items: flex-start;
      flex-direction: column;
    }

    .settings__theme-btns {
      width: 100%;
      justify-content: space-between;
    }

    .settings__theme-btn {
      flex: 1;
    }
  }
</style>
