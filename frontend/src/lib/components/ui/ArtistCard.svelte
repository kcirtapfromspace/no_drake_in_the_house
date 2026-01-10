<script lang="ts">
  import type { ArtistStatus } from '../../stores/artist';
  import { getStatusColor } from '../../stores/artist';
  import { navigateToArtist } from '../../utils/simple-router';

  export let id: string;
  export let name: string;
  export let imageUrl: string | undefined = undefined;
  export let status: ArtistStatus = 'clean';
  export let subtitle: string | undefined = undefined;
  export let offenseCount: number = 0;
  export let hasOffenses: boolean = false;
  export let isBlocked: boolean = false;
  export let showBlockedBadge: boolean = true;
  export let size: 'sm' | 'md' | 'lg' = 'md';
  export let onClick: (() => void) | undefined = undefined;

  $: statusColors = getStatusColor(status);

  function handleClick() {
    if (onClick) {
      onClick();
    } else {
      navigateToArtist(id);
    }
  }

  const sizeConfig = {
    sm: { padding: 'p-3', imageSize: 'aspect-square', nameSize: 'text-sm', subtitleSize: 'text-xs' },
    md: { padding: 'p-4', imageSize: 'aspect-square', nameSize: 'text-base', subtitleSize: 'text-sm' },
    lg: { padding: 'p-5', imageSize: 'aspect-square', nameSize: 'text-lg', subtitleSize: 'text-base' },
  };

  $: config = sizeConfig[size];

  // Determine card border based on status
  $: borderColor = status === 'flagged' ? '#EF4444' :
                   status === 'certified_creeper' ? '#EC4899' :
                   isBlocked ? '#DC2626' : '#52525b';
</script>

<button
  type="button"
  class="group {config.padding} rounded-xl text-left transition-all hover:scale-[1.02] w-full"
  style="background: #27272a; border: 1px solid {borderColor};"
  on:click={handleClick}
>
  <!-- Artist Image -->
  <div class="relative {config.imageSize} mb-3 rounded-lg overflow-hidden" style="background: linear-gradient(135deg, #1a1a2e, #16213e);">
    {#if imageUrl}
      <img
        src={imageUrl}
        alt={name}
        class="w-full h-full object-cover transition-transform group-hover:scale-105"
        style={status === 'flagged' ? 'filter: grayscale(30%);' : ''}
      />
    {:else}
      <div class="w-full h-full flex items-center justify-center text-3xl font-bold text-zinc-500 group-hover:text-zinc-400 transition-colors">
        {name.charAt(0).toUpperCase()}
      </div>
    {/if}

    <!-- Offense Badge -->
    {#if hasOffenses || offenseCount > 0}
      <div class="absolute top-2 right-2 w-7 h-7 rounded-full flex items-center justify-center bg-red-500 shadow-lg">
        <svg class="w-4 h-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
        </svg>
      </div>
    {/if}

    <!-- Blocked Badge -->
    {#if isBlocked && showBlockedBadge}
      <div class="absolute top-2 left-2 w-7 h-7 rounded-full flex items-center justify-center bg-red-600 shadow-lg">
        <svg class="w-3.5 h-3.5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </div>
    {/if}

    <!-- Hover Overlay -->
    <div class="absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
      <div class="w-12 h-12 rounded-full flex items-center justify-center bg-blue-500 shadow-lg">
        <svg class="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
        </svg>
      </div>
    </div>
  </div>

  <!-- Artist Info -->
  <h3 class="font-semibold text-white truncate {config.nameSize}">{name}</h3>
  {#if subtitle}
    <p class="text-zinc-400 truncate {config.subtitleSize}">{subtitle}</p>
  {:else if hasOffenses && offenseCount > 0}
    <p class="text-red-400 {config.subtitleSize}">
      {offenseCount} offense{offenseCount !== 1 ? 's' : ''}
    </p>
  {/if}
</button>
