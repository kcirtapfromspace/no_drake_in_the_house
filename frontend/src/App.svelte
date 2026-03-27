<script lang="ts">
	import { onDestroy, onMount } from "svelte";
	import { isAuthenticated, authActions, currentUser } from "./lib/stores/auth";
	import { initRouter, currentRoute, routeParams } from "./lib/utils/simple-router";
	import { theme } from "./lib/stores/theme";
	import Login from "./lib/components/Login.svelte";
	import Home from "./lib/components/Home.svelte";
	import Settings from "./lib/components/Settings.svelte";
	import OAuthCallback from "./lib/components/OAuthCallback.svelte";
	import SyncDashboard from "./lib/components/SyncDashboard.svelte";
	import AnalyticsDashboard from "./lib/components/AnalyticsDashboard.svelte";
	import GraphExplorer from "./lib/components/GraphExplorer.svelte";
	import ArtistProfile from "./lib/components/ArtistProfile.svelte";
	import ConnectionsPage from "./lib/components/ConnectionsPage.svelte";
	import LibraryScan from "./lib/components/LibraryScan.svelte";
	import CommunityLists from "./lib/components/CommunityLists.svelte";
	import OffenseDatabase from "./lib/components/OffenseDatabase.svelte";
	import ServiceHealthDashboard from "./lib/components/ServiceHealthDashboard.svelte";
	import PlaylistSanitizer from "./lib/components/PlaylistSanitizer.svelte";
	import BlocklistPage from "./lib/components/BlocklistPage.svelte";
	import AdminDashboard from "./lib/components/AdminDashboard.svelte";
	import Layout from "./lib/components/Layout.svelte";
	import config from "./lib/utils/config";
	import { initPostHog, capturePageView, identifyUser, resetUser } from "./lib/utils/posthog";

	let isInitialized = false;
	let initError = false;
	let isRetrying = false;
	let isMaintenanceMode = false;
	let isInitializing = false;
	let maintenancePollTimer: ReturnType<typeof setInterval> | null = null;

	const INIT_TIMEOUT_MS = 10000; // 10 second timeout
	const BACKEND_HEALTH_TIMEOUT_MS = 15000; // 15s to survive Render cold-starts
	const MAINTENANCE_POLL_INTERVAL_MS = 5000;

	async function isBackendHealthy(): Promise<boolean> {
		const controller = new AbortController();
		const timeoutId = setTimeout(() => controller.abort(), BACKEND_HEALTH_TIMEOUT_MS);

			try {
				const response = await fetch(config.getBackendEndpoint("/health"), {
					method: "GET",
					cache: "no-store",
					signal: controller.signal
			});
			return response.ok;
		} catch (error) {
			console.warn("Backend health check failed:", error);
			return false;
		} finally {
			clearTimeout(timeoutId);
		}
	}

	function stopMaintenancePolling() {
		if (maintenancePollTimer !== null) {
			clearInterval(maintenancePollTimer);
			maintenancePollTimer = null;
		}
	}

	function startMaintenancePolling() {
		if (maintenancePollTimer !== null) return;

		maintenancePollTimer = setInterval(async () => {
			if (isInitializing) return;
			const backendHealthy = await isBackendHealthy();
			if (!backendHealthy) return;

			stopMaintenancePolling();
			isInitialized = false;
			await initializeApp();
		}, MAINTENANCE_POLL_INTERVAL_MS);
	}

	async function initializeApp() {
		if (isInitializing) return;

		isInitializing = true;
		initError = false;
		isRetrying = true;

		try {
			theme.init();
			initRouter();

			const backendHealthy = await isBackendHealthy();
			if (!backendHealthy) {
				isMaintenanceMode = true;
				startMaintenancePolling();
				return;
			}

			isMaintenanceMode = false;
			stopMaintenancePolling();

			// Create a timeout promise
			const timeoutPromise = new Promise((_, reject) => {
				setTimeout(() => reject(new Error('Connection timeout')), INIT_TIMEOUT_MS);
			});

			// Race between auth fetch and timeout
			await Promise.race([
				authActions.fetchProfile(),
				timeoutPromise
			]);
		} catch (error) {
			console.error("Init error:", error);
			initError = true;
		} finally {
			isInitialized = true;
			isRetrying = false;
			isInitializing = false;
		}
	}

	function handleRetry() {
		isInitialized = false;
		initializeApp();
	}

	// Track page views on route changes
	const unsubRoute = currentRoute.subscribe((route) => {
		if (isInitialized) {
			capturePageView(route);
		}
	});

	// Identify/reset user in PostHog when auth state changes
	const unsubUser = currentUser.subscribe((user) => {
		if (user) {
			identifyUser(user.id, {
				email: user.email,
				name: user.display_name,
			});
		} else {
			resetUser();
		}
	});

	onMount(() => {
		initPostHog();
		initializeApp();
	});

	onDestroy(() => {
		stopMaintenancePolling();
		unsubRoute();
		unsubUser();
	});
</script>

{#if !isInitialized}
	<div class="app-state brand-status-view surface-page">
		<div class="brand-status-shell">
			<div class="brand-status-card">
				<div class="brand-status-kickers">
					<span class="brand-kicker">No Drake in the House</span>
					<span class="brand-kicker brand-kicker--accent">Preparing your filters</span>
				</div>
				<div class="brand-status-icon brand-status-icon--loading">
					<div class="brand-status-spinner"></div>
				</div>
				<h2 class="brand-status-title">Loading your account surface.</h2>
				<p class="brand-status-copy">
					Checking the backend, restoring your profile, and warming up the evidence-driven dashboard.
				</p>
			</div>
		</div>
	</div>
{:else if initError && !$isAuthenticated}
	<div class="app-state brand-status-view surface-page">
		<div class="brand-status-shell">
			<div class="brand-status-card">
				<div class="brand-status-kickers">
					<span class="brand-kicker">Connection Issue</span>
					<span class="brand-kicker brand-kicker--accent">Backend unavailable</span>
				</div>
				<div class="brand-status-icon brand-status-icon--error">
					<svg fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
					</svg>
				</div>
				<h2 class="brand-status-title">Connection issues</h2>
				<p class="brand-status-copy">
					Unable to reach the server. Check the deployed backend and retry once the API is responding again.
				</p>
				<div class="brand-status-actions">
					<button
						type="button"
						on:click={handleRetry}
						disabled={isRetrying}
						class="brand-button brand-button--danger app-state__btn"
					>
						{#if isRetrying}
							<div class="brand-button__spinner"></div>
							Retrying...
						{:else}
							<svg class="app-state__btn-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
							</svg>
							Try again
						{/if}
					</button>
				</div>
			</div>
		</div>
	</div>
{:else if isMaintenanceMode}
	<div class="app-state brand-status-view surface-page">
		<div class="brand-status-shell app-state__shell--wide">
			<div class="brand-status-card">
				<div class="brand-status-kickers">
					<span class="brand-kicker">Maintenance</span>
					<span class="brand-kicker brand-kicker--accent">Startup in progress</span>
				</div>
				<div class="brand-status-icon brand-status-icon--warning">
					<svg fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M4.93 19h14.14c1.54 0 2.5-1.67 1.73-3L13.73 4c-.77-1.33-2.69-1.33-3.46 0L3.2 16c-.77 1.33.19 3 1.73 3z" />
					</svg>
				</div>
				<h2 class="brand-status-title">Maintenance in progress</h2>
				<p class="brand-status-copy">
					The backend is still starting. This page will continue automatically as soon as the health checks turn green.
				</p>
				<div class="brand-status-actions">
					<button
						type="button"
						on:click={handleRetry}
						disabled={isRetrying}
						class="brand-button brand-button--secondary app-state__btn"
					>
						{#if isRetrying}
							<div class="brand-button__spinner"></div>
							Checking...
						{:else}
							Check now
						{/if}
					</button>
				</div>
				<p class="brand-status-note">The app polls every few seconds while startup is in progress.</p>
			</div>
		</div>
	</div>
{:else if $currentRoute === 'oauth-callback'}
	<OAuthCallback />
{:else if $currentRoute === 'oauth-error'}
	<div class="app-state brand-status-view surface-page">
		<div class="brand-status-shell">
			<div class="brand-status-card">
				<div class="brand-status-kickers">
					<span class="brand-kicker">OAuth Error</span>
					<span class="brand-kicker brand-kicker--accent">Connection interrupted</span>
				</div>
				<div class="brand-status-icon brand-status-icon--error">
					<span class="brand-status-bang">!</span>
				</div>
				<h2 class="brand-status-title">Connection failed</h2>
				<p class="brand-status-copy">
					There was a problem linking the account. Return to the app and try the connection flow again.
				</p>
				<div class="brand-status-actions">
					<button
						type="button"
						on:click={() => window.location.href = '/'}
						class="brand-button brand-button--danger app-state__btn"
					>
						Go back
					</button>
				</div>
			</div>
		</div>
	</div>
{:else if $isAuthenticated}
	<Layout>
		{#key `${$currentRoute}:${$routeParams.id || ''}`}
			{#if $currentRoute === 'artist-profile'}
				<ArtistProfile artistId={$routeParams.id || ''} />
			{:else if $currentRoute === 'blocklist'}
				<BlocklistPage />
			{:else if $currentRoute === 'settings'}
				<Settings />
			{:else if $currentRoute === 'connections'}
				<ConnectionsPage />
			{:else if $currentRoute === 'sync'}
				<SyncDashboard />
			{:else if $currentRoute === 'library-scan'}
				<LibraryScan />
			{:else if $currentRoute === 'revenue-impact'}
				<AnalyticsDashboard />
			{:else if $currentRoute === 'graph'}
				<GraphExplorer />
			{:else if $currentRoute === 'community'}
				<CommunityLists />
			{:else if $currentRoute === 'offense-database'}
				<OffenseDatabase />
			{:else if $currentRoute === 'service-health'}
				<ServiceHealthDashboard />
			{:else if $currentRoute === 'playlist-sanitizer'}
				<PlaylistSanitizer />
			{:else if $currentRoute === 'admin'}
				<AdminDashboard />
			{:else}
				<Home />
			{/if}
		{/key}
	</Layout>
{:else}
	<Login />
{/if}

<style>
	.app-state {
		color: var(--color-text-primary);
	}

	.app-state__shell--wide {
		width: min(100%, 38rem);
	}
	.app-state__btn {
		min-width: 9.5rem;
	}

	.app-state__btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.app-state__btn:focus-visible {
		outline: 2px solid var(--color-brand-primary);
		outline-offset: 2px;
	}

	.app-state__btn-icon {
		width: 1rem;
		height: 1rem;
	}
</style>
