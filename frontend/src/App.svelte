<script lang="ts">
	import { onMount } from "svelte";
	import { isAuthenticated, authActions } from "./lib/stores/auth";
	import { initRouter, currentRoute, routeParams } from "./lib/utils/simple-router";
	import Login from "./lib/components/Login.svelte";
	import Home from "./lib/components/Home.svelte";
	import Settings from "./lib/components/Settings.svelte";
	import OAuthCallback from "./lib/components/OAuthCallback.svelte";
	import SyncDashboard from "./lib/components/SyncDashboard.svelte";
	import AnalyticsDashboard from "./lib/components/AnalyticsDashboard.svelte";
	import GraphExplorer from "./lib/components/GraphExplorer.svelte";
	import ArtistProfile from "./lib/components/ArtistProfile.svelte";
	import Layout from "./lib/components/Layout.svelte";

	let isInitialized = false;
	let initError = false;
	let isRetrying = false;

	const INIT_TIMEOUT_MS = 10000; // 10 second timeout

	async function initializeApp() {
		initError = false;
		isRetrying = true;

		try {
			initRouter();

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
		}
	}

	function handleRetry() {
		isInitialized = false;
		initializeApp();
	}

	onMount(() => {
		initializeApp();
	});
</script>

{#if !isInitialized}
	<div class="min-h-screen flex items-center justify-center bg-gray-900">
		<div class="text-center">
			<div class="w-8 h-8 border-4 border-red-500 border-t-transparent rounded-full animate-spin mx-auto"></div>
			<p class="mt-4 text-gray-400">Loading...</p>
		</div>
	</div>
{:else if initError && !$isAuthenticated}
	<div class="min-h-screen flex items-center justify-center bg-gray-900 py-12 px-4">
		<div class="max-w-md w-full text-center">
			<div class="w-16 h-16 rounded-full bg-rose-500/20 flex items-center justify-center mx-auto mb-4">
				<svg class="w-8 h-8 text-rose-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
				</svg>
			</div>
			<h2 class="text-2xl font-bold text-white mb-2">Connection Issues</h2>
			<p class="text-zinc-400 mb-6">
				Unable to connect to the server. Please check your internet connection and try again.
			</p>
			<button
				on:click={handleRetry}
				disabled={isRetrying}
				class="px-6 py-3 bg-rose-500 hover:bg-rose-600 text-white rounded-lg font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed inline-flex items-center gap-2"
			>
				{#if isRetrying}
					<div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
					Retrying...
				{:else}
					<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
					</svg>
					Try Again
				{/if}
			</button>
		</div>
	</div>
{:else if $currentRoute === 'oauth-callback'}
	<OAuthCallback />
{:else if $currentRoute === 'oauth-error'}
	<div class="min-h-screen flex items-center justify-center bg-gray-900 py-12 px-4">
		<div class="max-w-md w-full text-center">
			<div class="text-red-500 text-6xl mb-4">!</div>
			<h2 class="text-2xl font-bold text-white mb-2">Connection Failed</h2>
			<p class="text-gray-400 mb-6">There was a problem connecting your account. Please try again.</p>
			<button
				on:click={() => window.location.href = '/'}
				class="px-6 py-3 bg-red-600 hover:bg-red-700 text-white rounded-lg font-medium transition-colors"
			>
				Go Back
			</button>
		</div>
	</div>
{:else if $isAuthenticated}
	{#if $currentRoute === 'artist-profile'}
		<ArtistProfile artistId={$routeParams.id || ''} />
	{:else}
		<Layout>
			{#if $currentRoute === 'settings'}
				<Settings />
			{:else if $currentRoute === 'sync'}
				<SyncDashboard />
			{:else if $currentRoute === 'analytics' || $currentRoute === 'revenue-impact'}
				<AnalyticsDashboard />
			{:else if $currentRoute === 'graph'}
				<GraphExplorer />
			{:else}
				<Home />
			{/if}
		</Layout>
	{/if}
{:else}
	<Login />
{/if}

<style>
	:global(body) {
		background-color: rgb(17, 24, 39);
	}
</style>
