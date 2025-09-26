<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { dnpActions } from '../stores/dnp';
  
  const dispatch = createEventDispatcher();
  
  export let entry;
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

    const tagArray = editTags.split(',').map(t => t.trim()).filter(t => t);
    
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

  function getProviderBadges(artist) {
    const badges = [];
    if (artist.external_ids.spotify) badges.push({ name: 'Spotify', color: 'bg-green-100 text-green-800' });
    if (artist.external_ids.apple) badges.push({ name: 'Apple', color: 'bg-gray-100 text-gray-800' });
    if (artist.external_ids.musicbrainz) badges.push({ name: 'MusicBrainz', color: 'bg-blue-100 text-blue-800' });
    return badges;
  }

  function formatDate(dateString) {
    return new Date(dateString).toLocaleDateString();
  }
</script>

<li class="px-4 py-4 {selected ? 'bg-indigo-50' : 'hover:bg-gray-50'}">
  <div class="flex items-center space-x-4">
    <!-- Checkbox -->
    <input
      type="checkbox"
      checked={selected}
      on:change={toggleSelect}
      class="h-4 w-4 text-indigo-600 focus:ring-indigo-500 border-gray-300 rounded"
    />

    <!-- Artist Image -->
    <div class="flex-shrink-0">
      {#if entry.artist.metadata.image}
        <img
          src={entry.artist.metadata.image}
          alt={entry.artist.canonical_name}
          class="h-12 w-12 rounded-full object-cover"
        />
      {:else}
        <div class="h-12 w-12 rounded-full bg-gray-300 flex items-center justify-center">
          <svg class="h-6 w-6 text-gray-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
          </svg>
        </div>
      {/if}
    </div>

    <!-- Artist Info -->
    <div class="flex-1 min-w-0">
      <div class="flex items-center justify-between">
        <div class="flex-1">
          <p class="text-sm font-medium text-gray-900 truncate">
            {entry.artist.canonical_name}
          </p>
          
          {#if entry.artist.metadata.genres && entry.artist.metadata.genres.length > 0}
            <p class="text-xs text-gray-500 truncate">
              {entry.artist.metadata.genres.slice(0, 2).join(', ')}
            </p>
          {/if}
          
          <div class="flex items-center space-x-2 mt-1">
            <!-- Provider Badges -->
            <div class="flex space-x-1">
              {#each getProviderBadges(entry.artist) as badge}
                <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium {badge.color}">
                  {badge.name}
                </span>
              {/each}
            </div>
            
            <!-- Added Date -->
            <span class="text-xs text-gray-400">
              Added {formatDate(entry.created_at)}
            </span>
          </div>
        </div>

        <!-- Actions -->
        <div class="flex items-center space-x-2">
          {#if !isEditing}
            <button
              on:click={startEdit}
              class="text-indigo-600 hover:text-indigo-900 text-sm"
            >
              Edit
            </button>
            <button
              on:click={removeArtist}
              disabled={isRemoving}
              class="text-red-600 hover:text-red-900 text-sm disabled:opacity-50"
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
            <div class="flex flex-wrap gap-1 mb-2">
              {#each entry.tags as tag}
                <span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-800">
                  {tag}
                </span>
              {/each}
            </div>
          {/if}
          
          {#if entry.note}
            <p class="text-sm text-gray-600 italic">
              "{entry.note}"
            </p>
          {/if}
        </div>
      {/if}

      <!-- Edit Form -->
      {#if isEditing}
        <div class="mt-3 space-y-3">
          <div>
            <label for="edit-tags-{entry.artist.id}" class="block text-xs font-medium text-gray-700">Tags</label>
            <input
              id="edit-tags-{entry.artist.id}"
              type="text"
              bind:value={editTags}
              placeholder="comma-separated tags"
              class="mt-1 block w-full border border-gray-300 rounded-md px-2 py-1 text-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
            />
          </div>
          
          <div>
            <label for="edit-note-{entry.artist.id}" class="block text-xs font-medium text-gray-700">Note</label>
            <textarea
              id="edit-note-{entry.artist.id}"
              bind:value={editNote}
              rows="2"
              placeholder="Personal note..."
              class="mt-1 block w-full border border-gray-300 rounded-md px-2 py-1 text-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
            ></textarea>
          </div>

          {#if error}
            <p class="text-xs text-red-600">{error}</p>
          {/if}

          <div class="flex justify-end space-x-2">
            <button
              type="button"
              on:click={cancelEdit}
              class="px-3 py-1 border border-gray-300 rounded-md text-xs font-medium text-gray-700 bg-white hover:bg-gray-50"
            >
              Cancel
            </button>
            <button
              type="button"
              on:click={saveEdit}
              disabled={isUpdating}
              class="px-3 py-1 border border-transparent rounded-md text-xs font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50"
            >
              {isUpdating ? 'Saving...' : 'Save'}
            </button>
          </div>
        </div>
      {/if}
    </div>
  </div>
</li>