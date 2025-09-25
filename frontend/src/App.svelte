<script lang="ts">
	export let name: string;
</script>

<main class="container mx-auto px-4 py-8">
	<h1 class="text-4xl font-bold text-center mb-8 text-gray-800">
		{name}
	</h1>
	
	<div class="max-w-2xl mx-auto">
		<div class="bg-white rounded-lg shadow-md p-6">
			<h2 class="text-2xl font-semibold mb-4 text-gray-700">
				Welcome to your music blocklist manager
			</h2>
			<p class="text-gray-600 mb-4">
				Manage your Do-Not-Play lists across multiple streaming platforms.
			</p>
			<div class="flex space-x-4">
				<button class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded">
					Connect Spotify
				</button>
				<button class="bg-gray-500 hover:bg-gray-700 text-white font-bold py-2 px-4 rounded">
					Manage DNP List
				</button>
			</div>
		</div>
	</div>
</main>

<style>
	main {
		text-align: center;
		padding: 1em;
		max-width: 240px;
		margin: 0 auto;
	}

	@media (min-width: 640px) {
		main {
			max-width: none;
		}
	}
</style>