<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { dnpActions } from '../stores/dnp';
  
  const dispatch = createEventDispatcher();
  
  export let entry: any;
  export let selected = false;
  
  let isEditing = false;
  let editTags = entry.tags.join(', ');
  let editNote = entry.note || '';
  let isUpdating = false;
  let isRemoving = false;
  let error = '';

  function toggleSelect() {
    dispatch('toggleSelect');
  }

  function startEdit() {
    isEditing = true;
    editTags = entry.tags.join(', ');
    editNote = entry.note || '';
    error = '';
  }

  function cancelEdit() {
    isEditing = false;
    editTags = entry.tags.join(', ');
    editNote = entry.note || '';
    error = '';
  }

  async function saveEdit() {
    isUpdating = true;
    error = '';

    const tagArray = editTags.split(',').map((t: string) => t.trim()).filter((t: string) => t);
    
    const result = await dnpActions.updateEntry(
      entry.artist.id,
      tagArray,
      editNote.trim() || undefined
    );

    if (result.success) {
      isEditing = false;
    } else {
      error = result.message || 'Failed to update entry';
    }

    isUpdating = false;
  }

  async function removeArtist() {
    if (!confirm(`Are you sure you want to remove "${entry.artist.canonical_name}" from your DNP list?`)) {
      return;
    }

    isRemoving = true;
    
    const result = await dnpActions.removeArtist(entry.artist.id);
    
    if (!result.success) {
      error = result.message || 'Failed to remove artist';
    }
    
    isRemoving = false;
  }

  function getProviderBadges(artist: any) {
    const badges = [];
    if (artist.external_ids.spotify) badges.push({ name: 'Spotify', color: 'bg-green-100 text-green-800' });
    if (artist.external_ids.apple) badges.push({ name: 'Apple', color: 'bg-zinc-700 text-zinc-300' });
    if (artist.external_ids.musicbrainz) badges.push({ name: 'MusicBrainz', color: 'bg-blue-100 text-blue-800' });
    return badges;
  }

  function formatDate(dateString: string) {
    return new Date(dateString).toLocaleDateString();
  }
</script>

<li class="px-4 py-4 sm:px-6 {selected ? 'bg-indigo-50' : 'hover:bg-zinc-700lightest'}">
  <div class="flex flex-col sm:flex-row sm:items-center space-y-3 sm:space-y-0 sm:space-x-4">
    <!-- Mobile: Checkbox and Actions Row -->
    <div class="flex items-center justify-between sm:hidden">
      <input
        type="checkbox"
        checked={selected}
        on:change={toggleSelect}
        class="icon-uswds icon-uswds--sm text-primary focus:ring-indigo-500 rounded-lg"
        style="border: 2px solid #52525b;"
      />
      <div class="flex items-center space-x-2">
        {#if !isEditing}
          <button
            on:click={startEdit}
            class="text-indigo-600 hover:text-indigo-900 text-zinc-400"
          >
            Edit
          </button>
          <button
            on:click={removeArtist}
            disabled={isRemoving}
            class="text-zinc-400 hover:text-red-900 text-zinc-400 disabled:opacity-50"
          >
            {isRemoving ? 'Removing...' : 'Remove'}
          </button>
        {/if}
      </div>
    </div>

    <!-- Desktop: Checkbox -->
    <input
      type="checkbox"
      checked={selected}
      on:change={toggleSelect}
      class="hidden sm:block icon-uswds icon-uswds--sm text-indigo-600 focus:ring-indigo-500 rounded-lg"
      style="border: 2px solid #52525b;"
    />

    <!-- Artist Image -->
    <div class="flex-shrink-0">
      {#if entry.artist.metadata.image}
        <img
          src={entry.artist.metadata.image}
          alt={entry.artist.canonical_name}
          class="avatar avatar--xl sm:avatar--lg object-cover"
        />
      {:else}
        <div class="avatar avatar--xl sm:avatar--lg bg-zinc-700lightest avatar__placeholder">
          <svg aria-hidden="true" class="icon-uswds icon-uswds--lg sm:icon icon-md sm: text-zinc-400darker" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
          </svg>
        </div>
      {/if}
    </div>

    <!-- Artist Info -->
    <div class="flex-1 min-w-0">
      <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between">
        <div class="flex-1 min-w-0">
          <p class="text-zinc-400 font-medium text-zinc-400darker truncate">
            {entry.artist.canonical_name}
          </p>
          
          {#if entry.artist.metadata.genres && entry.artist.metadata.genres.length > 0}
            <p class="text-zinc-400 text-zinc-400darker truncate">
              {entry.artist.metadata.genres.slice(0, 2).join(', ')}
            </p>
          {/if}
          
          <div class="flex flex-wrap items-center gap-uswds-2 mt-1">
            <!-- Provider Badges -->
            <div class="flex flex-wrap gap-uswds-1">
              {#each getProviderBadges(entry.artist) as badge}
                <span class="flex items-center px-2 py-0.5 rounded text-zinc-400 font-medium {badge.color}">
                  {badge.name}
                </span>
              {/each}
            </div>
            
            <!-- Added Date -->
            <span class="text-zinc-400 text-zinc-400darker whitespace-nowrap">
              Added {formatDate(entry.created_at)}
            </span>
          </div>
        </div>

        <!-- Desktop Actions -->
        <div class="hidden sm:flex items-center space-x-2 ml-4">
          {#if !isEditing}
            <button
              on:click={startEdit}
              class="text-indigo-600 hover:text-indigo-900 text-zinc-400 whitespace-nowrap"
            >
              Edit
            </button>
            <button
              on:click={removeArtist}
              disabled={isRemoving}
              class="text-zinc-400 hover:text-red-900 text-zinc-400 disabled:opacity-50 whitespace-nowrap"
            >
              {isRemoving ? 'Removing...' : 'Remove'}
            </button>
          {/if}
        </div>
      </div>

      <!-- Tags and Note Display -->
      {#if !isEditing}
        <div class="mt-2">
          {#if entry.tags.length > 0}
            <div class="flex flex-wrap gap-uswds-1 mb-2">
              {#each entry.tags as tag}
                <span class="inline-flex items-center px-2 py-0.5 rounded-full text-zinc-400 font-medium bg-zinc-700lightest text-zinc-400darker">
                  {tag}
                </span>
              {/each}
            </div>
          {/if}
          
          {#if entry.note}
            <p class="text-zinc-400 text-zinc-400darker italic">
              "{entry.note}"
            </p>
          {/if}
        </div>
      {/if}

      <!-- Edit Form -->
      {#if isEditing}
        <div class="mt-3 space-y-3 sm:mt-4">
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-uswds-3">
            <div>
              <label for="edit-tags-{entry.artist.id}" class="block text-zinc-400 font-medium text-zinc-300">Tags</label>
              <input
                id="edit-tags-{entry.artist.id}"
                type="text"
                bind:value={editTags}
                placeholder="comma-separated tags"
                class="mt-1 block w-full rounded-lg px-2 py-1 text-zinc-400 text-zinc-300 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
                style="background: #3f3f46; border: 2px solid #52525b;"
              />
            </div>
            
            <div class="sm:col-span-2">
              <label for="edit-note-{entry.artist.id}" class="block text-zinc-400 font-medium text-zinc-300">Note</label>
              <textarea
                id="edit-note-{entry.artist.id}"
                bind:value={editNote}
                rows="2"
                placeholder="Personal note..."
                class="mt-1 block w-full rounded-lg px-2 py-1 text-zinc-400 text-zinc-300 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
                style="background: #3f3f46; border: 2px solid #52525b;"
              ></textarea>
            </div>
          </div>

          {#if error}
            <p class="text-zinc-400 text-zinc-400">{error}</p>
          {/if}

          <div class="flex flex-col sm:flex-row justify-end space-y-2 sm:space-y-0 sm:space-x-2">
            <button
              type="button"
              on:click={cancelEdit}
              class="w-full sm:w-auto px-3 py-2 sm:py-1 rounded-lg text-zinc-400 sm:text-zinc-400 font-medium text-zinc-300 hover:bg-zinc-700"
              style="background: #3f3f46; border: 2px solid #52525b;"
            >
              Cancel
            </button>
            <button
              type="button"
              on:click={saveEdit}
              disabled={isUpdating}
              class="w-full sm:w-auto px-3 py-2 sm:py-1 border border-transparent rounded-lg text-zinc-400 sm:text-zinc-400 font-medium text-white bg-primary hover:bg-indigo-700 disabled:opacity-50"
            >
              {isUpdating ? 'Saving...' : 'Save'}
            </button>
          </div>
        </div>
      {/if}
    </div>
  </div>
</li>