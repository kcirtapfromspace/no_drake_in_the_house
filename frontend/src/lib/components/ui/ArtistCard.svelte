<script lang="ts">
  import type { ArtistStatus } from '../../stores/artist';
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

  function handleClick() {
    if (onClick) {
      onClick();
    } else {
      navigateToArtist(id);
    }
  }

  // Determine card border based on status
  $: borderVar = status === 'flagged' ? 'var(--color-error)' :
                  status === 'certified_creeper' ? '#EC4899' :
                  isBlocked ? 'var(--color-error)' : 'var(--color-border-default)';
</script>

<button
  type="button"
  class="artist-card artist-card--{size}"
  style="--artist-card-border: {borderVar};"
  on:click={handleClick}
>
  <!-- Artist Image -->
  <div class="artist-card__image" class:artist-card__image--flagged={status === 'flagged'}>
    {#if imageUrl}
      <img
        src={imageUrl}
        alt={name}
        class="artist-card__img"
      />
    {:else}
      <div class="artist-card__placeholder">
        {name.charAt(0).toUpperCase()}
      </div>
    {/if}

    <!-- Offense Badge -->
    {#if hasOffenses || offenseCount > 0}
      <div class="artist-card__badge artist-card__badge--offense">
        <svg class="artist-card__badge-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
        </svg>
      </div>
    {/if}

    <!-- Blocked Badge -->
    {#if isBlocked && showBlockedBadge}
      <div class="artist-card__badge artist-card__badge--blocked">
        <svg class="artist-card__badge-icon artist-card__badge-icon--sm" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </div>
    {/if}

    <!-- Hover Overlay -->
    <div class="artist-card__overlay">
      <div class="artist-card__overlay-btn">
        <svg class="artist-card__overlay-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
        </svg>
      </div>
    </div>
  </div>

  <!-- Artist Info -->
  <h3 class="artist-card__name">{name}</h3>
  {#if subtitle}
    <p class="artist-card__subtitle">{subtitle}</p>
  {:else if hasOffenses && offenseCount > 0}
    <p class="artist-card__subtitle artist-card__subtitle--offense">
      {offenseCount} offense{offenseCount !== 1 ? 's' : ''}
    </p>
  {/if}
</button>

<style>
  .artist-card {
    display: block;
    width: 100%;
    text-align: left;
    border-radius: var(--radius-xl);
    background-color: var(--color-bg-elevated);
    border: 1px solid var(--artist-card-border, var(--color-border-default));
    cursor: pointer;
    transition: transform var(--transition-fast);
    font-family: inherit;
  }

  .artist-card:hover {
    transform: scale(1.02);
  }

  .artist-card:focus-visible {
    outline: none;
    box-shadow: 0 0 0 2px var(--color-brand-primary), 0 0 0 4px var(--color-bg-primary);
  }

  /* Size variants */
  .artist-card--sm { padding: 0.75rem; }
  .artist-card--md { padding: 1rem; }
  .artist-card--lg { padding: 1.25rem; }

  /* Image container */
  .artist-card__image {
    position: relative;
    aspect-ratio: 1;
    margin-bottom: 0.75rem;
    border-radius: var(--radius-lg);
    overflow: hidden;
    background: linear-gradient(135deg, var(--color-bg-sunken), var(--color-bg-elevated));
  }

  .artist-card__img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    transition: transform var(--transition-fast);
  }

  .artist-card:hover .artist-card__img {
    transform: scale(1.05);
  }

  .artist-card__image--flagged .artist-card__img {
    filter: grayscale(30%);
  }

  .artist-card__placeholder {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 2rem;
    font-weight: 700;
    color: var(--color-text-muted);
    transition: color var(--transition-fast);
  }

  .artist-card:hover .artist-card__placeholder {
    color: var(--color-text-secondary);
  }

  /* Badges */
  .artist-card__badge {
    position: absolute;
    width: 1.75rem;
    height: 1.75rem;
    border-radius: var(--radius-full);
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  }

  .artist-card__badge--offense {
    top: 0.5rem;
    right: 0.5rem;
    background-color: var(--color-error);
  }

  .artist-card__badge--blocked {
    top: 0.5rem;
    left: 0.5rem;
    background-color: var(--color-error);
  }

  .artist-card__badge-icon {
    width: 1rem;
    height: 1rem;
    color: white;
  }

  .artist-card__badge-icon--sm {
    width: 0.875rem;
    height: 0.875rem;
  }

  /* Hover overlay */
  .artist-card__overlay {
    position: absolute;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.4);
    opacity: 0;
    transition: opacity var(--transition-fast);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .artist-card:hover .artist-card__overlay {
    opacity: 1;
  }

  .artist-card__overlay-btn {
    width: 3rem;
    height: 3rem;
    border-radius: var(--radius-full);
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: var(--color-brand-primary);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  }

  .artist-card__overlay-icon {
    width: 1.25rem;
    height: 1.25rem;
    color: white;
  }

  /* Text */
  .artist-card__name {
    font-weight: 600;
    color: var(--color-text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    margin: 0;
  }

  .artist-card--sm .artist-card__name { font-size: var(--text-sm); }
  .artist-card--md .artist-card__name { font-size: var(--text-base); }
  .artist-card--lg .artist-card__name { font-size: 1.125rem; }

  .artist-card__subtitle {
    color: var(--color-text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    margin: 0;
  }

  .artist-card--sm .artist-card__subtitle { font-size: var(--text-xs); }
  .artist-card--md .artist-card__subtitle { font-size: var(--text-sm); }
  .artist-card--lg .artist-card__subtitle { font-size: var(--text-base); }

  .artist-card__subtitle--offense {
    color: var(--color-error);
  }

  @media (prefers-reduced-motion: reduce) {
    .artist-card,
    .artist-card__img,
    .artist-card__overlay {
      transition: none;
    }

    .artist-card:hover {
      transform: none;
    }

    .artist-card:hover .artist-card__img {
      transform: none;
    }
  }
</style>
