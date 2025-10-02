<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { communityActions } from '../stores/community';
  
  const dispatch = createEventDispatcher();
  
  let name = '';
  let description = '';
  let criteria = '';
  let governanceUrl = '';
  let updateCadence = 'monthly';
  let visibility = 'public';
  let isCreating = false;
  let error = '';

  async function handleSubmit() {
    if (!name.trim() || !description.trim() || !criteria.trim()) {
      error = 'Please fill in all required fields';
      return;
    }

    isCreating = true;
    error = '';

    const result = await communityActions.createList({
      name: name.trim(),
      description: description.trim(),
      criteria: criteria.trim(),
      governance_url: governanceUrl.trim() || undefined,
      update_cadence: updateCadence,
      visibility: visibility as 'public' | 'private',
    });

    if (result.success) {
      // Reset form
      name = '';
      description = '';
      criteria = '';
      governanceUrl = '';
      updateCadence = 'monthly';
      visibility = 'public';
      dispatch('listCreated');
    } else {
      error = result.message || 'Failed to create community list';
    }

    isCreating = false;
  }
</script>

<form on:submit|preventDefault={handleSubmit} class="space-y-6">
  <!-- Name -->
  <div>
    <label for="name" class="block text-uswds-sm font-medium text-uswds-base-darker">
      List Name *
    </label>
    <input
      id="name"
      type="text"
      bind:value={name}
      placeholder="e.g., Controversial Artists List"
      class="mt-1 block w-full border border-gray-300 rounded-uswds-md px-3 py-2 placeholder-gray-500 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-uswds-sm"
      required
    />
  </div>

  <!-- Description -->
  <div>
    <label for="description" class="block text-uswds-sm font-medium text-uswds-base-darker">
      Description *
    </label>
    <textarea
      id="description"
      bind:value={description}
      rows="3"
      placeholder="Describe the purpose and scope of this list..."
      class="mt-1 block w-full border border-gray-300 rounded-uswds-md px-3 py-2 placeholder-gray-500 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-uswds-sm"
      required
    ></textarea>
  </div>

  <!-- Criteria -->
  <div>
    <label for="criteria" class="block text-uswds-sm font-medium text-uswds-base-darker">
      Inclusion Criteria *
    </label>
    <textarea
      id="criteria"
      bind:value={criteria}
      rows="4"
      placeholder="Define clear, neutral criteria for including artists in this list. Avoid subjective language or personal opinions."
      class="mt-1 block w-full border border-gray-300 rounded-uswds-md px-3 py-2 placeholder-gray-500 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-uswds-sm"
      required
    ></textarea>
    <p class="mt-1 text-uswds-xs text-uswds-base-darker">
      Criteria must be factual and neutral. Avoid subjective terms or personal opinions.
    </p>
  </div>

  <!-- Governance URL -->
  <div>
    <label for="governance-url" class="block text-uswds-sm font-medium text-uswds-base-darker">
      Governance Documentation URL
    </label>
    <input
      id="governance-url"
      type="url"
      bind:value={governanceUrl}
      placeholder="https://example.com/governance-process"
      class="mt-1 block w-full border border-gray-300 rounded-uswds-md px-3 py-2 placeholder-gray-500 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-uswds-sm"
    />
    <p class="mt-1 text-uswds-xs text-uswds-base-darker">
      Link to documentation explaining your list's governance process and appeals procedure.
    </p>
  </div>

  <!-- Update Cadence -->
  <div>
    <label for="update-cadence" class="block text-uswds-sm font-medium text-uswds-base-darker">
      Update Cadence
    </label>
    <select
      id="update-cadence"
      bind:value={updateCadence}
      class="mt-1 block w-full border border-gray-300 rounded-uswds-md px-3 py-2 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-uswds-sm"
    >
      <option value="daily">Daily</option>
      <option value="weekly">Weekly</option>
      <option value="monthly">Monthly</option>
      <option value="as-needed">As Needed</option>
    </select>
  </div>

  <!-- Visibility -->
  <div>
    <h4 class="block text-uswds-sm font-medium text-uswds-base-darker">Visibility</h4>
    <div class="mt-2 space-y-2">
      <div class="flex items-center">
        <input
          id="public"
          type="radio"
          bind:group={visibility}
          value="public"
          class="focus:ring-indigo-500 icon-uswds icon-uswds--sm text-primary border-gray-300"
        />
        <label for="public" class="ml-3 block text-uswds-sm text-uswds-base-darker">
          Public - Anyone can discover and subscribe
        </label>
      </div>
      <div class="flex items-center">
        <input
          id="private"
          type="radio"
          bind:group={visibility}
          value="private"
          class="focus:ring-indigo-500 icon-uswds icon-uswds--sm text-indigo-600 border-gray-300"
        />
        <label for="private" class="ml-3 block text-uswds-sm text-uswds-base-darker">
          Private - Only you can manage, others need direct link
        </label>
      </div>
    </div>
  </div>

  {#if error}
    <div class="text-uswds-red-50 text-uswds-sm">
      {error}
    </div>
  {/if}

  <!-- Submit Button -->
  <div class="flex justify-end space-x-3">
    <button
      type="button"
      on:click={() => dispatch('listCreated')}
      class="px-4 py-2 border border-gray-300 rounded-uswds-md text-uswds-sm font-medium text-uswds-base-darker bg-white hover:bg-uswds-base-lightest focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
    >
      Cancel
    </button>
    <button
      type="submit"
      disabled={isCreating || !name.trim() || !description.trim() || !criteria.trim()}
      class="px-4 py-2 border border-transparent rounded-uswds-md shadow-sm text-uswds-sm font-medium text-white bg-primary hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"
    >
      {#if isCreating}
        <svg aria-hidden="true" class="animate-spin -ml-1 mr-2 icon-uswds icon-uswds--sm text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
        Creating...
      {:else}
        Create List
      {/if}
    </button>
  </div>

  <!-- Guidelines -->
  <div class="bg-yellow-50 border border-yellow-200 rounded-uswds-md p-uswds-4">
    <div class="flex">
      <div class="">
        <svg aria-hidden="true" class="icon-uswds icon-uswds--md text-yellow-400" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
        </svg>
      </div>
      <div class="ml-3">
        <h3 class="text-uswds-sm font-medium text-yellow-800">
          Community List Guidelines
        </h3>
        <div class="mt-2 text-uswds-sm text-yellow-700">
          <ul class="list-disc list-inside space-y-1">
            <li>Use neutral, factual language in criteria</li>
            <li>Provide clear governance and appeals processes</li>
            <li>Maintain transparency about list updates</li>
            <li>Respect platform terms of service</li>
            <li>Focus on user preferences, not personal judgments</li>
          </ul>
        </div>
      </div>
    </div>
  </div>
</form>