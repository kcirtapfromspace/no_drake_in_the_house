<script lang="ts">
	import { onMount } from "svelte";
	import { isAuthenticated, authActions } from "./lib/stores/auth";
	import { router } from "./lib/utils/router";
	import Login from "./lib/components/Login.svelte";
	import Dashboard from "./lib/components/Dashboard.svelte";

	let isInitialized = false;

	onMount(async () => {
		console.log("App mounting...");
		try {
			// Initialize router
			router.init();
			console.log("Router initialized");

			// Initialize auth state
			await authActions.fetchProfile();
			console.log("Auth profile fetched");
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
			<div
				class="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600 mx-auto"
			></div>
			<p class="mt-4 text-gray-600">Loading...</p>
		</div>
	</div>
{:else if $isAuthenticated}
	<Dashboard />
{:else}
	<Login />
{/if}
