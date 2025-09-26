<script lang="ts">
  import { enforcementActions, enforcementStore } from '../stores/enforcement';
  
  $: options = $enforcementStore.options;

  function updateAggressiveness(level: 'conservative' | 'moderate' | 'aggressive') {
    enforcementActions.updateOptions({ aggressiveness: level });
  }

  function toggleOption(option: keyof typeof options) {
    enforcementActions.updateOptions({ [option]: !options[option] });
  }
</script>

<div class="space-y-6">
  <!-- Aggressiveness Level -->
  <div>
    <h4 class="text-base font-medium text-gray-900">Enforcement Aggressiveness</h4>
    <p class="text-sm leading-5 text-gray-500">
      Choose how thoroughly to apply your blocklist across your music library.
    </p>
    <fieldset class="mt-4">
      <legend class="sr-only">Aggressiveness level</legend>
      <div class="space-y-4">
        <div class="flex items-center">
          <input
            id="conservative"
            name="aggressiveness"
            type="radio"
            checked={options.aggressiveness === 'conservative'}
            on:change={() => updateAggressiveness('conservative')}
            class="focus:ring-indigo-500 h-4 w-4 text-indigo-600 border-gray-300"
          />
          <label for="conservative" class="ml-3 block text-sm font-medium text-gray-700">
            Conservative
          </label>
        </div>
        <div class="ml-7 text-sm text-gray-500">
          Only remove explicitly saved/liked content. Preserves playlists and recommendations.
        </div>

        <div class="flex items-center">
          <input
            id="moderate"
            name="aggressiveness"
            type="radio"
            checked={options.aggressiveness === 'moderate'}
            on:change={() => updateAggressiveness('moderate')}
            class="focus:ring-indigo-500 h-4 w-4 text-indigo-600 border-gray-300"
          />
          <label for="moderate" class="ml-3 block text-sm font-medium text-gray-700">
            Moderate (Recommended)
          </label>
        </div>
        <div class="ml-7 text-sm text-gray-500">
          Remove from saved content and playlists. Filters recommendations where possible.
        </div>

        <div class="flex items-center">
          <input
            id="aggressive"
            name="aggressiveness"
            type="radio"
            checked={options.aggressiveness === 'aggressive'}
            on:change={() => updateAggressiveness('aggressive')}
            class="focus:ring-indigo-500 h-4 w-4 text-indigo-600 border-gray-300"
          />
          <label for="aggressive" class="ml-3 block text-sm font-medium text-gray-700">
            Aggressive
          </label>
        </div>
        <div class="ml-7 text-sm text-gray-500">
          Maximum removal including radio seeds, recommendations, and related content.
        </div>
      </div>
    </fieldset>
  </div>

  <!-- Collaboration and Featuring Options -->
  <div>
    <h4 class="text-base font-medium text-gray-900">Collaboration Handling</h4>
    <p class="text-sm leading-5 text-gray-500">
      Configure how to handle songs where blocked artists appear as collaborators or featured artists.
    </p>
    <div class="mt-4 space-y-4">
      <div class="flex items-start">
        <div class="flex items-center h-5">
          <input
            id="block-collabs"
            type="checkbox"
            checked={options.blockCollabs}
            on:change={() => toggleOption('blockCollabs')}
            class="focus:ring-indigo-500 h-4 w-4 text-indigo-600 border-gray-300 rounded"
          />
        </div>
        <div class="ml-3 text-sm">
          <label for="block-collabs" class="font-medium text-gray-700">
            Block collaborations
          </label>
          <p class="text-gray-500">
            Remove songs where blocked artists are listed as collaborators or co-writers.
          </p>
        </div>
      </div>

      <div class="flex items-start">
        <div class="flex items-center h-5">
          <input
            id="block-featuring"
            type="checkbox"
            checked={options.blockFeaturing}
            on:change={() => toggleOption('blockFeaturing')}
            class="focus:ring-indigo-500 h-4 w-4 text-indigo-600 border-gray-300 rounded"
          />
        </div>
        <div class="ml-3 text-sm">
          <label for="block-featuring" class="font-medium text-gray-700">
            Block featuring
          </label>
          <p class="text-gray-500">
            Remove songs where blocked artists are featured (e.g., "Song Title (feat. Blocked Artist)").
          </p>
        </div>
      </div>

      <div class="flex items-start">
        <div class="flex items-center h-5">
          <input
            id="block-songwriter-only"
            type="checkbox"
            checked={options.blockSongwriterOnly}
            on:change={() => toggleOption('blockSongwriterOnly')}
            class="focus:ring-indigo-500 h-4 w-4 text-indigo-600 border-gray-300 rounded"
          />
        </div>
        <div class="ml-3 text-sm">
          <label for="block-songwriter-only" class="font-medium text-gray-700">
            Block songwriter credits only
          </label>
          <p class="text-gray-500">
            Remove songs where blocked artists are credited only as songwriters (most restrictive).
          </p>
        </div>
      </div>
    </div>
  </div>

  <!-- Warning for Aggressive Settings -->
  {#if options.aggressiveness === 'aggressive' || options.blockSongwriterOnly}
    <div class="bg-yellow-50 border border-yellow-200 rounded-md p-4">
      <div class="flex">
        <div class="flex-shrink-0">
          <svg class="h-5 w-5 text-yellow-400" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
          </svg>
        </div>
        <div class="ml-3">
          <h3 class="text-sm font-medium text-yellow-800">
            Aggressive Settings Warning
          </h3>
          <div class="mt-2 text-sm text-yellow-700">
            <p>
              These settings may remove a significant amount of content from your library. 
              We recommend reviewing the enforcement preview carefully before executing.
            </p>
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>