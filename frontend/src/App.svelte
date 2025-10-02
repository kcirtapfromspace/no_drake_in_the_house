<script lang="ts">
	import { onMount } from "svelte";
	import { isAuthenticated, authActions } from "./lib/stores/auth";
	import { initRouter, currentRoute } from "./lib/utils/simple-router";
	import Login from "./lib/components/Login.svelte";
	import Dashboard from "./lib/components/Dashboard.svelte";
	import OAuthCallback from "./lib/components/OAuthCallback.svelte";
	// import { fixTextLayout, fixIconConstraints, fixSpacing } from "./lib/utils/layout-fixes.js";

	let isInitialized = false;

	onMount(async () => {
		console.log("App mounting...");
		try {
			// Initialize router
			initRouter();
			console.log("Router initialized");

			// Initialize auth state
			await authActions.fetchProfile();
			console.log("Auth profile fetched");

			// Apply layout fixes
			// fixTextLayout();
			// fixIconConstraints();
			// fixSpacing();
			// console.log("Layout fixes applied");
		} catch (error) {
			console.error("Error during app initialization:", error);
		} finally {
			isInitialized = true;
			console.log("App initialized, isAuthenticated:", $isAuthenticated);
		}
	});
</script>

{#if !isInitialized}
	<div class="min-h-screen flex items-center justify-center bg-gray-50">
		<div class="text-center">
			<div class="icon icon-xl icon-primary animate-spin rounded-full border-b-2 border-primary mx-auto"></div>
			<p class="mt-4 text-gray-600">Loading...</p>
		</div>
	</div>
{:else if $currentRoute === 'oauth-callback'}
	<!-- Handle OAuth callback regardless of authentication state -->
	{@const urlParams = new URLSearchParams(window.location.search)}
	{@const pathParts = window.location.pathname.split('/')}
	{@const provider = pathParts[pathParts.length - 1] || 'unknown'}
	<OAuthCallback {provider} />
{:else if $currentRoute === 'oauth-error'}
	<!-- Handle OAuth errors -->
	<div class="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
		<div class="max-w-md w-full space-y-8">
			<div class="text-center">
				<div class="mx-auto h-12 w-12 text-red-600">
					<svg class="h-12 w-12" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16c-.77.833.192 2.5 1.732 2.5z"></path>
					</svg>
				</div>
				<h2 class="mt-6 text-3xl font-extrabold text-gray-900">
					Authentication Error
				</h2>
				<p class="mt-2 text-sm text-gray-600">
					There was a problem with your social login. Please try again.
				</p>
				<div class="mt-6">
					<button
						type="button"
						on:click={() => window.location.href = '/'}
						class="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
					>
						Return to Login
					</button>
				</div>
			</div>
		</div>
	</div>
{:else if $isAuthenticated}
	<Dashboard />
{:else}
	<Login />
{/if}
