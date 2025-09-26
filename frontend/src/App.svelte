<script lang="ts">
	import { onMount } from 'svelte';
	import { isAuthenticated, authActions } from './lib/stores/auth';
	import Login from './lib/components/Login.svelte';
	import Dashboard from './lib/components/Dashboard.svelte';

	onMount(() => {
		// Initialize auth state
		authActions.fetchProfile();
	});
</script>

{#if $isAuthenticated}
	<Dashboard />
{:else}
	<Login />
{/if}