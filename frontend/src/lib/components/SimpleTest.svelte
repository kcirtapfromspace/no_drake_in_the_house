<script>
  import { currentRoute, navigateTo } from '../utils/simple-router';

  // Debug: Log route changes
  $: {
    console.log('SimpleTest: Current route changed to:', $currentRoute);
  }

  function testNavigation(route) {
    console.log('SimpleTest: Testing navigation to:', route);
    navigateTo(route);
  }
</script>

<div class="min-h-screen p-8" style="background: #27272a;">
  <div class="max-w-4xl mx-auto">
    <h1 class="text-2xl font-bold text-zinc-300 mb-6">CSS and Navigation Test</h1>

    <div class="rounded-lg shadow p-6 mb-6" style="background: #3f3f46;">
      <h2 class="text-lg font-semibold mb-4">Current Route: {$currentRoute}</h2>

      <div class="grid grid-cols-2 gap-4 mb-6">
        <div class="bg-blue-900/30 p-4 rounded border border-blue-700">
          <h3 class="font-medium text-blue-300">CSS Test</h3>
          <p class="text-blue-400 text-sm">If you can see this styled properly, CSS is working!</p>
        </div>

        <div class="bg-green-900/30 p-4 rounded border border-green-700">
          <h3 class="font-medium text-green-300">Layout Test</h3>
          <p class="text-green-400 text-sm">This should be in a 2-column grid layout.</p>
        </div>
      </div>

      <div class="flex gap-4">
        <button class="btn btn-primary">Primary Button</button>
        <button class="bg-zinc-700 px-4 py-2 rounded hover:bg-zinc-600">Secondary Button</button>
      </div>
    </div>

    <div class="rounded-lg shadow p-6" style="background: #3f3f46;">
      <h2 class="text-lg font-semibold mb-4">Navigation Test</h2>
      <p class="text-zinc-400 mb-4">Click the tabs above to test navigation. The route should change without page refresh.</p>

      <div class="text-sm text-zinc-500 mb-4">
        <p>Current route: <code class="bg-zinc-700 px-2 py-1 rounded">{$currentRoute}</code></p>
      </div>

      <div class="flex gap-2 flex-wrap">
        <button
          class="btn btn-primary text-sm"
          on:click={() => testNavigation('overview')}
        >
          Test Overview
        </button>
        <button
          class="btn btn-primary text-sm"
          on:click={() => testNavigation('connections')}
        >
          Test Connections
        </button>
        <button
          class="btn btn-primary text-sm"
          on:click={() => testNavigation('dnp')}
        >
          Test DNP
        </button>
        <button
          class="btn btn-primary text-sm"
          on:click={() => testNavigation('community')}
        >
          Test Community
        </button>
      </div>
    </div>
  </div>
</div>
