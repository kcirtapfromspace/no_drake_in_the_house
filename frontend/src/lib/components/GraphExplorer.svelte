<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import {
    graphStore,
    graphActions,
    networkNodes,
    networkEdges,
    atRiskArtists
  } from '../stores/graph';
  import type { GraphArtist, NetworkNode } from '../stores/graph';
  import { navigateTo } from '../utils/simple-router';

  let searchQuery = '';
  let searchResults: GraphArtist[] = [];
  let isSearching = false;
  let selectedDepth = 2;
  let showPathFinder = false;
  let pathSourceId = '';
  let pathTargetId = '';

  // Simple force-directed layout simulation
  let nodes: (NetworkNode & { x: number; y: number; vx: number; vy: number })[] = [];
  let simulationInterval: ReturnType<typeof setInterval> | null = null;

  onMount(async () => {
    await Promise.all([
      graphActions.fetchHealth(),
      graphActions.fetchStats(),
    ]);
  });

  onDestroy(() => {
    if (simulationInterval) {
      clearInterval(simulationInterval);
    }
  });

  async function handleSearch() {
    if (!searchQuery.trim()) {
      searchResults = [];
      return;
    }

    isSearching = true;
    try {
      // Use the sync service for artist search
      const response = await fetch(`/api/v1/sync/search?q=${encodeURIComponent(searchQuery)}`);
      const data = await response.json();
      searchResults = data.artists || [];
    } catch (e) {
      console.error('Search failed:', e);
      searchResults = [];
    }
    isSearching = false;
  }

  async function selectArtist(artist: GraphArtist) {
    graphActions.selectArtist(artist);
    await graphActions.fetchArtistNetwork(artist.id, selectedDepth);
    await graphActions.fetchCollaborators(artist.id);
    searchQuery = '';
    searchResults = [];
    initializeSimulation();
  }

  function initializeSimulation() {
    if (simulationInterval) {
      clearInterval(simulationInterval);
    }

    const width = 600;
    const height = 400;

    // Initialize node positions
    nodes = $networkNodes.map((node) => ({
      ...node,
      x: width / 2 + (Math.random() - 0.5) * 200,
      y: height / 2 + (Math.random() - 0.5) * 200,
      vx: 0,
      vy: 0,
    }));

    // Run simple force simulation
    let iterations = 0;
    simulationInterval = setInterval(() => {
      if (iterations > 100) {
        if (simulationInterval) clearInterval(simulationInterval);
        return;
      }

      // Apply forces
      nodes.forEach((node, i) => {
        // Repulsion between nodes
        nodes.forEach((other, j) => {
          if (i === j) return;
          const dx = node.x - other.x;
          const dy = node.y - other.y;
          const dist = Math.sqrt(dx * dx + dy * dy) || 1;
          const force = 500 / (dist * dist);
          node.vx += (dx / dist) * force;
          node.vy += (dy / dist) * force;
        });

        // Attraction along edges
        $networkEdges.forEach(edge => {
          if (edge.source === node.id || edge.target === node.id) {
            const otherId = edge.source === node.id ? edge.target : edge.source;
            const other = nodes.find(n => n.id === otherId);
            if (other) {
              const dx = other.x - node.x;
              const dy = other.y - node.y;
              node.vx += dx * 0.01;
              node.vy += dy * 0.01;
            }
          }
        });

        // Center gravity
        node.vx += (width / 2 - node.x) * 0.001;
        node.vy += (height / 2 - node.y) * 0.001;

        // Damping
        node.vx *= 0.9;
        node.vy *= 0.9;

        // Update position
        node.x += node.vx;
        node.y += node.vy;

        // Bounds
        node.x = Math.max(30, Math.min(width - 30, node.x));
        node.y = Math.max(30, Math.min(height - 30, node.y));
      });

      nodes = [...nodes]; // Trigger reactivity
      iterations++;
    }, 50);
  }

  async function handleFindPath() {
    if (!pathSourceId || !pathTargetId) return;
    await graphActions.findPath(pathSourceId, pathTargetId);
  }

  async function handleAnalyzeBlocked() {
    await graphActions.analyzeBlockedNetwork();
  }

  function getNodeColor(node: NetworkNode): string {
    if (node.is_blocked) return '#EF4444'; // red
    if (node.type === 'label') return '#8B5CF6'; // purple
    if (node.type === 'track') return '#10B981'; // green
    return '#6366F1'; // indigo for artists
  }

  function getEdgeColor(type: string): string {
    switch (type) {
      case 'collaborated_with': return '#94A3B8';
      case 'signed_to': return '#8B5CF6';
      case 'mentioned_in': return '#F59E0B';
      default: return '#CBD5E1';
    }
  }

  $: currentNetwork = $graphStore.currentNetwork;
  $: selectedArtist = $graphStore.selectedArtist;
  $: collaborators = $graphStore.collaborators;
  $: pathResult = $graphStore.pathResult;
  $: blockedAnalysis = $graphStore.blockedAnalysis;
  $: stats = $graphStore.stats;
</script>

<div class="min-h-screen" style="background: #18181b;">
  <!-- Header -->
  <div style="background: #27272a; border-bottom: 2px solid #52525b;">
    <div class="max-w-6xl mx-auto px-4 py-8 sm:px-6 lg:px-8">
      <button
        type="button"
        on:click={() => navigateTo('home')}
        class="flex items-center gap-2 text-zinc-400 hover:text-white transition-colors mb-4"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
        </svg>
        Back to Home
      </button>
      <div class="flex items-center justify-between">
        <div>
          <h1 class="text-3xl font-bold text-white mb-2">
            Graph Explorer
          </h1>
          <p class="text-lg text-zinc-400">
            Explore artist collaboration networks and connections.
          </p>
        </div>
        <div class="flex items-center gap-3">
          <button
            type="button"
            on:click={() => showPathFinder = !showPathFinder}
            class="px-4 py-2 text-zinc-300 rounded-lg font-medium transition-colors flex items-center gap-2"
            style="border: 2px solid #52525b; background: #3f3f46;"
          >
            <span>🔍</span> Path Finder
          </button>
          <button
            type="button"
            on:click={handleAnalyzeBlocked}
            disabled={$graphStore.isLoading}
            class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 font-medium transition-colors flex items-center gap-2 disabled:opacity-50"
          >
            <span>⚠️</span> Analyze Blocked
          </button>
        </div>
      </div>
    </div>
  </div>

  <div class="max-w-6xl mx-auto px-4 py-6 sm:px-6 lg:px-8">
    <!-- Stats Bar -->
    {#if stats}
      <div class="grid grid-cols-4 gap-4 mb-6">
        <div class="rounded-lg p-4 shadow-sm text-center" style="background: #27272a; border: 2px solid #52525b;">
          <div class="text-2xl font-bold text-indigo-400">{stats.artist_count.toLocaleString()}</div>
          <div class="text-xs text-zinc-400">Artists</div>
        </div>
        <div class="rounded-lg p-4 shadow-sm text-center" style="background: #27272a; border: 2px solid #52525b;">
          <div class="text-2xl font-bold text-indigo-400">{stats.collaboration_count.toLocaleString()}</div>
          <div class="text-xs text-zinc-400">Collaborations</div>
        </div>
        <div class="rounded-lg p-4 shadow-sm text-center" style="background: #27272a; border: 2px solid #52525b;">
          <div class="text-2xl font-bold text-indigo-400">{stats.label_count.toLocaleString()}</div>
          <div class="text-xs text-zinc-400">Labels</div>
        </div>
        <div class="rounded-lg p-4 shadow-sm text-center" style="background: #27272a; border: 2px solid #52525b;">
          <div class="text-2xl font-bold text-indigo-400">{stats.track_count.toLocaleString()}</div>
          <div class="text-xs text-zinc-400">Tracks</div>
        </div>
      </div>
    {/if}

    <!-- Error display -->
    {#if $graphStore.error}
      <div class="bg-red-50 border border-red-200 rounded-xl p-4 mb-6">
        <div class="flex items-center gap-2 text-red-700">
          <span>❌</span>
          <span>{$graphStore.error}</span>
          <button type="button" on:click={graphActions.clearError} class="ml-auto text-red-500 hover:text-red-700">
            Dismiss
          </button>
        </div>
      </div>
    {/if}

    <!-- Path Finder Panel -->
    {#if showPathFinder}
      <div class="rounded-xl p-6 shadow-sm mb-6" style="background: #27272a; border: 2px solid #52525b;">
        <h3 class="text-lg font-semibold text-white mb-4">Find Path Between Artists</h3>
        <div class="grid md:grid-cols-3 gap-4">
          <div>
            <label for="path-source" class="block text-sm font-medium text-zinc-300 mb-2">Source Artist ID</label>
            <input
              id="path-source"
              type="text"
              bind:value={pathSourceId}
              placeholder="Enter artist UUID"
              class="w-full px-4 py-2 rounded-lg text-white focus:border-indigo-500 focus:ring-2 focus:ring-indigo-200"
              style="background: #3f3f46; border: 2px solid #52525b;"
            />
          </div>
          <div>
            <label for="path-target" class="block text-sm font-medium text-zinc-300 mb-2">Target Artist ID</label>
            <input
              id="path-target"
              type="text"
              bind:value={pathTargetId}
              placeholder="Enter artist UUID"
              class="w-full px-4 py-2 rounded-lg text-white focus:border-indigo-500 focus:ring-2 focus:ring-indigo-200"
              style="background: #3f3f46; border: 2px solid #52525b;"
            />
          </div>
          <div class="flex items-end">
            <button
              type="button"
              on:click={handleFindPath}
              disabled={!pathSourceId || !pathTargetId || $graphStore.isLoadingPath}
              class="w-full px-4 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 font-medium transition-colors disabled:opacity-50"
            >
              {#if $graphStore.isLoadingPath}
                Finding...
              {:else}
                Find Path
              {/if}
            </button>
          </div>
        </div>
        {#if pathResult}
          <div class="mt-4 p-4 bg-green-900/30 rounded-lg">
            <div class="font-medium text-green-400 mb-2">Path Found! ({pathResult.total_distance} hops)</div>
            <div class="flex items-center gap-2 flex-wrap">
              {#each pathResult.path as node, i}
                <span class="px-2 py-1 rounded text-sm text-white" style="background: #3f3f46; border: 2px solid #52525b;">{node.name}</span>
                {#if i < pathResult.path.length - 1}
                  <span class="text-green-600">→</span>
                {/if}
              {/each}
            </div>
          </div>
        {/if}
      </div>
    {/if}

    <!-- Main Content Grid -->
    <div class="grid lg:grid-cols-3 gap-6">
      <!-- Search & Selection Panel -->
      <div class="lg:col-span-1">
        <div class="rounded-xl p-6 shadow-sm mb-6" style="background: #27272a; border: 2px solid #52525b;">
          <h3 class="text-lg font-semibold text-white mb-4">Search Artist</h3>
          <div class="relative">
            <input
              type="text"
              bind:value={searchQuery}
              on:input={handleSearch}
              placeholder="Search for an artist..."
              class="w-full px-4 py-3 rounded-xl text-white focus:border-indigo-500 focus:ring-2 focus:ring-indigo-200"
              style="background: #3f3f46; border: 2px solid #52525b;"
            />
            {#if isSearching}
              <div class="absolute right-3 top-1/2 -translate-y-1/2">
                <div class="w-4 h-4 border-2 border-indigo-500 border-t-transparent rounded-full animate-spin"></div>
              </div>
            {/if}
          </div>

          {#if searchResults.length > 0}
            <div class="mt-3 space-y-2 max-h-60 overflow-y-auto">
              {#each searchResults as artist}
                <button
                  type="button"
                  on:click={() => selectArtist(artist)}
                  class="w-full p-3 text-left hover:bg-indigo-900/30 rounded-lg transition-colors"
                >
                  <div class="font-medium text-white">{artist.name}</div>
                  {#if artist.genres?.length}
                    <div class="text-xs text-zinc-400">{artist.genres.slice(0, 3).join(', ')}</div>
                  {/if}
                </button>
              {/each}
            </div>
          {/if}
        </div>

        <!-- Depth Control -->
        <div class="rounded-xl p-6 shadow-sm mb-6" style="background: #27272a; border: 2px solid #52525b;">
          <h3 class="text-lg font-semibold text-white mb-4">Network Depth</h3>
          <div class="flex items-center gap-4">
            <input
              type="range"
              min="1"
              max="4"
              bind:value={selectedDepth}
              class="flex-1"
            />
            <span class="text-lg font-medium text-indigo-600 w-8">{selectedDepth}</span>
          </div>
          <p class="text-xs text-zinc-400 mt-2">Higher depth = more connections but slower loading</p>
        </div>

        <!-- Collaborators List -->
        {#if selectedArtist && collaborators.length > 0}
          <div class="rounded-xl p-6 shadow-sm" style="background: #27272a; border: 2px solid #52525b;">
            <h3 class="text-lg font-semibold text-white mb-4">
              Collaborators of {selectedArtist.name}
            </h3>
            <div class="space-y-2 max-h-80 overflow-y-auto">
              {#each collaborators as collab}
                <div class="p-3 rounded-lg" style="background: #3f3f46;">
                  <div class="font-medium text-white">{collab.artist_name}</div>
                  <div class="text-xs text-zinc-400 flex items-center gap-2">
                    <span class="capitalize">{collab.collab_type}</span>
                    {#if collab.track_title}
                      <span>• {collab.track_title}</span>
                    {/if}
                    {#if collab.year}
                      <span>• {collab.year}</span>
                    {/if}
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>

      <!-- Network Visualization -->
      <div class="lg:col-span-2">
        <div class="rounded-xl p-6 shadow-sm" style="background: #27272a; border: 2px solid #52525b;">
          <div class="flex items-center justify-between mb-4">
            <h3 class="text-lg font-semibold text-white">Network Visualization</h3>
            {#if currentNetwork}
              <div class="text-sm text-zinc-400">
                {$networkNodes.length} nodes • {$networkEdges.length} edges
              </div>
            {/if}
          </div>

          {#if $graphStore.isLoading}
            <div class="flex justify-center items-center h-96">
              <div class="w-8 h-8 border-4 border-indigo-500 border-t-transparent rounded-full animate-spin"></div>
            </div>
          {:else if !currentNetwork}
            <div class="flex flex-col items-center justify-center h-96 text-zinc-400">
              <span class="text-6xl mb-4">🕸️</span>
              <p>Search for an artist to explore their network</p>
            </div>
          {:else}
            <svg
              viewBox="0 0 600 400"
              class="w-full h-96 rounded-lg"
              style="background: #3f3f46;"
            >
              <!-- Edges -->
              {#each $networkEdges as edge}
                {@const source = nodes.find(n => n.id === edge.source)}
                {@const target = nodes.find(n => n.id === edge.target)}
                {#if source && target}
                  <line
                    x1={source.x}
                    y1={source.y}
                    x2={target.x}
                    y2={target.y}
                    stroke={getEdgeColor(edge.type)}
                    stroke-width={edge.weight}
                    stroke-opacity="0.6"
                  />
                {/if}
              {/each}

              <!-- Nodes -->
              {#each nodes as node}
                <g transform="translate({node.x}, {node.y})">
                  <circle
                    r={node.id === currentNetwork.center_artist_id ? 16 : 10}
                    fill={getNodeColor(node)}
                    stroke="white"
                    stroke-width="2"
                    class="cursor-pointer hover:opacity-80 transition-opacity"
                  />
                  <text
                    y={node.id === currentNetwork.center_artist_id ? 28 : 22}
                    text-anchor="middle"
                    font-size="10"
                    fill="#a1a1aa"
                    class="pointer-events-none"
                  >
                    {node.name.length > 12 ? node.name.substring(0, 12) + '...' : node.name}
                  </text>
                </g>
              {/each}
            </svg>

            <!-- Legend -->
            <div class="flex items-center gap-6 mt-4 text-xs text-zinc-400 justify-center">
              <div class="flex items-center gap-1">
                <span class="w-3 h-3 rounded-full bg-indigo-500"></span>
                <span>Artist</span>
              </div>
              <div class="flex items-center gap-1">
                <span class="w-3 h-3 rounded-full bg-red-500"></span>
                <span>Blocked</span>
              </div>
              <div class="flex items-center gap-1">
                <span class="w-3 h-3 rounded-full bg-purple-500"></span>
                <span>Label</span>
              </div>
              <div class="flex items-center gap-1">
                <span class="w-3 h-3 rounded-full bg-green-500"></span>
                <span>Track</span>
              </div>
            </div>
          {/if}
        </div>

        <!-- Blocked Network Analysis -->
        {#if blockedAnalysis}
          <div class="rounded-xl p-6 shadow-sm mt-6" style="background: #27272a; border: 2px solid #52525b;">
            <h3 class="text-lg font-semibold text-white mb-4">Blocked Network Analysis</h3>

            <!-- Summary -->
            <div class="grid grid-cols-3 gap-4 mb-6">
              <div class="text-center p-3 bg-red-900/30 rounded-lg">
                <div class="text-2xl font-bold text-red-400">{blockedAnalysis.summary.total_blocked}</div>
                <div class="text-xs text-zinc-400">Blocked Artists</div>
              </div>
              <div class="text-center p-3 bg-yellow-900/30 rounded-lg">
                <div class="text-2xl font-bold text-yellow-400">{blockedAnalysis.summary.total_at_risk}</div>
                <div class="text-xs text-zinc-400">At-Risk Artists</div>
              </div>
              <div class="text-center p-3 rounded-lg" style="background: #3f3f46;">
                <div class="text-2xl font-bold text-zinc-300">{blockedAnalysis.summary.avg_collaborations_per_blocked.toFixed(1)}</div>
                <div class="text-xs text-zinc-400">Avg. Collaborations</div>
              </div>
            </div>

            <!-- At-Risk Artists -->
            {#if $atRiskArtists.length > 0}
              <h4 class="font-medium text-white mb-3">At-Risk Artists</h4>
              <div class="space-y-2 max-h-48 overflow-y-auto">
                {#each $atRiskArtists.slice(0, 10) as item}
                  <div class="flex items-center justify-between p-3 bg-yellow-900/30 rounded-lg">
                    <div>
                      <div class="font-medium text-white">{item.artist.name}</div>
                      <div class="text-xs text-zinc-400">{item.blocked_collaborators} blocked collaborators</div>
                    </div>
                    <div class="text-right">
                      <div class="font-medium text-yellow-400">{(item.risk_score * 100).toFixed(0)}%</div>
                      <div class="text-xs text-zinc-400">risk</div>
                    </div>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>
