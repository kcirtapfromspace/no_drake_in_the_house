<script lang="ts">
	import { onMount } from "svelte";
	import { isAuthenticated, authActions } from "./lib/stores/auth";
	import { initRouter, currentRoute } from "./lib/utils/simple-router";
	import Login from "./lib/components/Login.svelte";
	import Home from "./lib/components/Home.svelte";
	import Settings from "./lib/components/Settings.svelte";
	import OAuthCallback from "./lib/components/OAuthCallback.svelte";
	import SyncDashboard from "./lib/components/SyncDashboard.svelte";
	import AnalyticsDashboard from "./lib/components/AnalyticsDashboard.svelte";
	import GraphExplorer from "./lib/components/GraphExplorer.svelte";

	let isInitialized = false;

	onMount(async () => {
		try {
			initRouter();
			await authActions.fetchProfile();
		} catch (error) {
			console.error("Init error:", error);
		} finally {
			isInitialized = true;
		}
	});
</script>

{#if !isInitialized}
	<div class="min-h-screen flex items-center justify-center bg-gray-900">
		<div class="text-center">
			<div class="w-8 h-8 border-4 border-red-500 border-t-transparent rounded-full animate-spin mx-auto"></div>
			<p class="mt-4 text-gray-400">Loading...</p>
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
	{#if $currentRoute === 'settings'}
		<Settings />
	{:else if $currentRoute === 'sync'}
		<SyncDashboard />
	{:else if $currentRoute === 'analytics'}
		<AnalyticsDashboard />
	{:else if $currentRoute === 'graph'}
		<GraphExplorer />
	{:else}
		<Home />
	{/if}
{:else}
	<Login />
{/if}

<style>
	:global(body) {
		background-color: rgb(17, 24, 39);
	}
</style>
