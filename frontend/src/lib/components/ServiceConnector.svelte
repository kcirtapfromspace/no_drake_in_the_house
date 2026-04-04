<script lang="ts">
  import { connectionActions, spotifyConnection, appleMusicConnection, tidalConnection, youtubeConnection } from '../stores/connections';

  $: services = [
    {
      id: 'spotify',
      name: 'Spotify',
      color: '#1DB954',
      connected: $spotifyConnection?.status === 'active',
      needsReconnect: !!$spotifyConnection && $spotifyConnection.status !== 'active',
    },
    {
      id: 'apple',
      name: 'Apple Music',
      color: '#FC3C44',
      connected: $appleMusicConnection?.status === 'active',
      needsReconnect: !!$appleMusicConnection && $appleMusicConnection.status !== 'active',
    },
    {
      id: 'tidal',
      name: 'Tidal',
      color: '#ffffff',
      connected: $tidalConnection?.status === 'active',
      needsReconnect: !!$tidalConnection && $tidalConnection.status !== 'active',
    },
    {
      id: 'youtube',
      name: 'YouTube Music',
      color: '#FF0000',
      connected: $youtubeConnection?.status === 'active',
      needsReconnect: !!$youtubeConnection && $youtubeConnection.status !== 'active',
    },
    {
      id: 'deezer',
      name: 'Deezer',
      color: '#A238FF',
      connected: false,
      needsReconnect: false,
    },
    {
      id: 'amazon',
      name: 'Amazon Music',
      color: '#25D1DA',
      connected: false,
      needsReconnect: false,
    },
    {
      id: 'soundcloud',
      name: 'SoundCloud',
      color: '#FF5500',
      connected: false,
      needsReconnect: false,
    },
    {
      id: 'lastfm',
      name: 'Last.fm',
      color: '#D51007',
      connected: false,
      needsReconnect: false,
    },
  ];

  let connectingId: string | null = null;

  async function handleConnect(service: typeof services[0]) {
    if (connectingId) return;
    connectingId = service.id;

    try {
      switch (service.id) {
        case 'spotify':
          await connectionActions.initiateSpotifyAuth();
          break;
        case 'apple':
          await connectionActions.connectAppleMusic();
          break;
        case 'tidal':
          await connectionActions.initiateTidalAuth();
          break;
        case 'youtube':
          await connectionActions.initiateYouTubeAuth();
          break;
      }
    } finally {
      connectingId = null;
    }
  }
</script>

<div class="sc">
  <div class="sc__grid">
    {#each services as service, i}
      <button
        type="button"
        class="sc__tile"
        class:sc__tile--connected={service.connected}
        class:sc__tile--reconnect={service.needsReconnect}
        class:sc__tile--connecting={connectingId === service.id}
        style="--sc-color: {service.color}; --stagger: {i * 40}ms;"
        on:click={() => handleConnect(service)}
        disabled={connectingId !== null}
      >
        <!-- Status indicator -->
        {#if service.connected}
          <span class="sc__status sc__status--live"></span>
        {:else if service.needsReconnect}
          <span class="sc__status sc__status--warn"></span>
        {/if}

        <!-- Icon -->
        <div class="sc__icon">
          {#if service.id === 'spotify'}
            <svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 0C5.4 0 0 5.4 0 12s5.4 12 12 12 12-5.4 12-12S18.66 0 12 0zm5.521 17.34c-.24.359-.66.48-1.021.24-2.82-1.74-6.36-2.101-10.561-1.141-.418.122-.779-.179-.899-.539-.12-.421.18-.78.54-.9 4.56-1.021 8.52-.6 11.64 1.32.42.18.479.659.301 1.02zm1.44-3.3c-.301.42-.841.6-1.262.3-3.239-1.98-8.159-2.58-11.939-1.38-.479.12-1.02-.12-1.14-.6-.12-.48.12-1.021.6-1.141C9.6 9.9 15 10.561 18.72 12.84c.361.181.54.78.241 1.2zm.12-3.36C15.24 8.4 8.82 8.16 5.16 9.301c-.6.179-1.2-.181-1.38-.721-.18-.601.18-1.2.72-1.381 4.26-1.26 11.28-1.02 15.721 1.621.539.3.719 1.02.42 1.56-.299.421-1.02.599-1.559.3z"/></svg>
          {:else if service.id === 'apple'}
            <svg viewBox="0 0 24 24" fill="currentColor"><path d="M18.71 19.5c-.83 1.24-1.71 2.45-3.05 2.47-1.34.03-1.77-.79-3.29-.79-1.53 0-2 .77-3.27.82-1.31.05-2.3-1.32-3.14-2.53C4.25 17 2.94 12.45 4.7 9.39c.87-1.52 2.43-2.48 4.12-2.51 1.28-.02 2.5.87 3.29.87.78 0 2.26-1.07 3.81-.91.65.03 2.47.26 3.64 1.98-.09.06-2.17 1.28-2.15 3.81.03 3.02 2.65 4.03 2.68 4.04-.03.07-.42 1.44-1.38 2.83M13 3.5c.73-.83 1.94-1.46 2.94-1.5.13 1.17-.34 2.35-1.04 3.19-.69.85-1.83 1.51-2.95 1.42-.15-1.15.41-2.35 1.05-3.11z"/></svg>
          {:else if service.id === 'tidal'}
            <svg viewBox="0 0 24 24" fill="currentColor"><path d="M12.012 3.992L8.008 7.996 4.004 3.992 0 7.996 4.004 12l4.004-4.004L12.012 12l4.004-4.004L12.012 3.992zM12.012 12l-4.004 4.004L12.012 20.008l4.004-4.004L12.012 12zM20.02 7.996L16.016 3.992l-4.004 4.004 4.004 4.004 4.004-4.004L24.024 3.992 20.02 7.996z"/></svg>
          {:else if service.id === 'youtube'}
            <svg viewBox="0 0 24 24" fill="currentColor"><path d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z"/></svg>
          {:else if service.id === 'deezer'}
            <svg viewBox="0 0 24 24" fill="currentColor"><path d="M18.81 4.16v3.03H24V4.16h-5.19zM6.27 8.38v3.027h5.189V8.38h-5.19zm12.54 0v3.027H24V8.38h-5.19zM6.27 12.594v3.027h5.189v-3.027h-5.19zm6.271 0v3.027h5.19v-3.027h-5.19zm6.27 0v3.027H24v-3.027h-5.19zM0 16.81v3.028h5.19v-3.027H0zm6.27 0v3.028h5.189v-3.027h-5.19zm6.271 0v3.028h5.19v-3.027h-5.19zm6.27 0v3.028H24v-3.027h-5.19z"/></svg>
          {:else if service.id === 'amazon'}
            <svg viewBox="0 0 24 24" fill="currentColor"><path d="M.045 18.02c.072-.116.187-.124.348-.022 3.636 2.11 7.594 3.166 11.87 3.166 2.852 0 5.668-.533 8.447-1.595l.315-.14c.138-.06.234-.1.293-.13.226-.088.39-.046.493.124.104.17.075.348-.088.534-.524.5-1.467 1.063-2.834 1.684-2.236.95-4.57 1.425-7.004 1.425-4.57 0-8.555-1.452-11.952-4.355-.21-.19-.247-.35-.088-.534zM7.394 14.228c0-1.077.308-2.003.925-2.777.617-.775 1.49-1.277 2.618-1.508.946-.192 1.97-.29 3.076-.29l1.583.045v-.584c0-.736-.105-1.243-.315-1.52-.393-.5-1.073-.748-2.038-.748h-.202c-.59.017-1.065.175-1.425.472-.36.297-.588.69-.686 1.18l-2.768-.36c.124-.94.594-1.712 1.41-2.315.815-.604 1.866-.917 3.153-.94h.473c1.705 0 2.96.41 3.767 1.232.538.55.86 1.17.967 1.856.033.192.05.572.05 1.14v4.107c0 .45.076.904.226 1.365.15.46.342.845.574 1.153l.27.36H14.89c-.14-.192-.265-.427-.371-.7-.032-.084-.07-.214-.113-.39-.644.622-1.26 1.037-1.85 1.243-.59.206-1.258.31-2.004.31-.897 0-1.62-.27-2.17-.81-.548-.54-.822-1.236-.822-2.09zm3.137-.135c0 .45.13.81.393 1.076.263.265.603.398 1.023.398.383 0 .756-.1 1.12-.297.364-.197.636-.47.82-.818.222-.417.333-1.076.333-1.975v-.675l-1.293.045c-1.028.038-1.748.222-2.16.555-.41.333-.615.814-.615 1.44v.25zM21.69 18.14c-.37.528-.636.857-.8 1-.328.295-.608.44-.84.44-.192 0-.35-.12-.47-.36-.12-.24-.18-.54-.18-.9l.015-.765c0-.5.168-.956.502-1.365.334-.41.87-.773 1.607-1.09l.505-.225c.234-.1.384-.174.45-.225.25-.175.435-.416.555-.72.12-.308.18-.596.18-.87v-.465h.84l-.045.9c-.028.496-.07.86-.12 1.09-.054.23-.167.483-.338.764-.17.28-.42.575-.752.884l-.51.465c-.16.17-.27.29-.33.36-.224.263-.36.49-.406.682-.047.19-.07.44-.07.747l.015.45c0 .15-.07.23-.208.23z"/></svg>
          {:else if service.id === 'soundcloud'}
            <svg viewBox="0 0 24 24" fill="currentColor"><path d="M1.175 12.225c-.051 0-.094.046-.101.1l-.233 2.154.233 2.105c.007.058.05.098.101.098.05 0 .09-.04.099-.098l.255-2.105-.27-2.154c0-.057-.045-.1-.09-.1m-.899.828c-.06 0-.091.037-.104.094L0 14.479l.165 1.308c0 .055.045.094.09.094s.089-.038.104-.094l.21-1.319-.21-1.334c0-.057-.044-.09-.09-.09m1.83-1.229c-.06 0-.12.045-.12.104l-.21 2.563.225 2.458c0 .06.045.12.119.12.061 0 .105-.061.121-.12l.254-2.474-.254-2.548c-.016-.06-.061-.12-.121-.12m.945-.089c-.075 0-.135.06-.15.135l-.193 2.64.21 2.544c.016.077.075.135.149.135.075 0 .135-.06.15-.135l.24-2.529-.24-2.655c-.015-.075-.06-.135-.15-.135m1.155.36c-.005-.09-.075-.149-.159-.149-.09 0-.158.06-.164.149l-.217 2.43.2 2.563c0 .09.075.157.159.157.074 0 .148-.068.148-.158l.227-2.563-.21-2.432m.809-1.709c-.101 0-.18.09-.18.181l-.21 3.957.187 2.563c0 .09.08.164.18.164.094 0 .174-.09.18-.18l.209-2.563-.209-3.972c-.008-.104-.088-.18-.18-.18m.959-.914c-.105 0-.195.09-.203.194l-.18 4.872.165 2.548c0 .12.09.209.195.209.104 0 .194-.089.21-.209l.193-2.548-.192-4.856c-.016-.12-.105-.21-.21-.21m.989-.449c-.121 0-.211.089-.225.209l-.165 5.275.15 2.563c.016.121.105.225.227.225.119 0 .209-.104.225-.225l.17-2.548-.17-5.275c0-.12-.105-.224-.227-.224m1.245.045c0-.135-.105-.24-.24-.24-.119 0-.24.105-.24.24l-.149 5.441.149 2.532c.016.149.121.255.256.255s.24-.12.24-.255l.164-2.532-.164-5.441m.479-1.205c-.016-.15-.135-.27-.271-.27s-.256.12-.271.27L7.5 14.39l.136 2.563c.016.15.12.27.271.27.15 0 .255-.12.271-.27l.149-2.563L8.178 8.1m1.216-.404c-.164 0-.3.135-.313.3L8.94 14.39l.149 2.534c0 .164.148.314.312.314.164 0 .299-.15.314-.314l.166-2.534-.151-5.906c-.015-.165-.15-.3-.315-.3m1.503-.59c-.18 0-.33.149-.344.33l-.125 6.18.14 2.5c0 .18.164.33.33.33.165 0 .315-.15.345-.33l.154-2.5-.17-6.18c0-.18-.164-.33-.345-.33m1.1-.12c-.195 0-.345.149-.36.33l-.12 5.985.135 2.5c.015.194.165.345.36.345.18 0 .345-.15.36-.345l.15-2.5-.15-5.985c-.015-.195-.18-.345-.36-.345m1.08-.12c-.209 0-.375.166-.375.375l-.12 5.775.135 2.474c0 .209.164.375.375.375.194 0 .375-.166.375-.375l.15-2.474-.165-5.79c0-.21-.18-.375-.39-.375m1.5.645c-.21 0-.39.18-.39.39l-.09 5.01.105 2.434c.015.21.18.39.39.39.21 0 .375-.18.375-.39l.12-2.434-.12-5.01c-.015-.21-.18-.39-.39-.39m.96-.42c-.226 0-.405.18-.405.405l-.105 5.46.12 2.418c0 .225.18.405.405.405.225 0 .405-.18.405-.405l.12-2.418-.135-5.46c0-.225-.18-.405-.405-.405m1.605.195c0-.24-.195-.42-.42-.42-.24 0-.42.18-.435.42l-.09 5.25.105 2.4c.015.24.195.42.42.42.24 0 .42-.18.435-.42l.12-2.4-.135-5.25m.99-.555c-.24 0-.435.194-.45.435l-.074 5.37.09 2.37c.015.255.21.449.449.449.24 0 .435-.194.435-.449l.119-2.37-.119-5.37c-.015-.255-.21-.45-.45-.45m2.58 1.095c-.255 0-.465.21-.465.465L20.7 14.4l.09 2.355c0 .255.21.465.465.465.24 0 .465-.21.465-.465L21.84 14.4l-.12-4.98c-.015-.255-.225-.465-.465-.465m1.5-.48c-.27 0-.48.21-.495.465L22.38 14.4l.09 2.31c.015.27.225.48.48.48.255 0 .48-.21.495-.48l.105-2.31-.105-5.325c-.015-.27-.24-.495-.495-.495"/></svg>
          {:else if service.id === 'lastfm'}
            <svg viewBox="0 0 24 24" fill="currentColor"><path d="M10.584 17.21l-.88-2.392s-1.43 1.594-3.573 1.594c-1.897 0-3.244-1.649-3.244-4.288 0-3.382 1.704-4.591 3.381-4.591 2.42 0 3.189 1.567 3.849 3.574l.88 2.749c.88 2.666 2.529 4.81 7.285 4.81 3.409 0 5.718-1.044 5.718-3.793 0-2.227-1.265-3.381-3.63-3.932l-1.758-.385c-1.21-.275-1.567-.77-1.567-1.595 0-.934.742-1.484 1.952-1.484 1.32 0 2.034.495 2.144 1.677l2.749-.33c-.22-2.474-1.924-3.492-4.729-3.492-2.474 0-4.893.935-4.893 3.932 0 1.87.907 3.051 3.189 3.601l1.87.44c1.402.33 1.87.825 1.87 1.68 0 1.018-.99 1.441-2.859 1.441-2.776 0-3.932-1.457-4.59-3.464l-.907-2.75c-1.155-3.573-2.997-4.893-6.653-4.893C2.144 5.333 0 7.89 0 12.233c0 4.18 2.144 6.434 5.993 6.434 3.106 0 4.591-1.457 4.591-1.457z"/></svg>
          {/if}
        </div>

        <!-- Name -->
        <span class="sc__name">{service.name}</span>

        <!-- Connecting spinner -->
        {#if connectingId === service.id}
          <div class="sc__spinner"></div>
        {/if}
      </button>
    {/each}
  </div>
</div>

<style>
  .sc {
    width: 100%;
  }

  .sc__grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
    gap: 0.75rem;
  }

  @media (min-width: 640px) {
    .sc__grid {
      grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
      gap: 1rem;
    }
  }

  .sc__tile {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.625rem;
    padding: 1.25rem 0.75rem;
    border-radius: 1.125rem;
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border-subtle);
    cursor: pointer;
    font-family: inherit;
    color: var(--color-text-secondary);
    transition: all 0.35s cubic-bezier(0.32, 0.72, 0, 1);
    animation: tileIn 0.4s cubic-bezier(0.22, 1, 0.36, 1) both;
    animation-delay: var(--stagger, 0ms);
    overflow: hidden;
  }

  @keyframes tileIn {
    from { opacity: 0; transform: scale(0.92) translateY(8px); }
    to { opacity: 1; transform: scale(1) translateY(0); }
  }

  .sc__tile:hover {
    border-color: color-mix(in srgb, var(--sc-color) 35%, var(--color-border-hover));
    background: color-mix(in srgb, var(--sc-color) 6%, var(--color-bg-elevated));
    transform: translateY(-3px) scale(1.03);
    box-shadow: 0 8px 24px -8px color-mix(in srgb, var(--sc-color) 15%, transparent);
  }

  .sc__tile:active {
    transform: translateY(-1px) scale(0.98);
    transition-duration: 0.1s;
  }

  .sc__tile--connected {
    border-color: color-mix(in srgb, var(--sc-color) 25%, var(--color-border-subtle));
  }

  .sc__tile--connected .sc__icon {
    color: var(--sc-color);
  }

  .sc__tile--reconnect {
    border-color: rgba(245, 158, 11, 0.25);
  }

  .sc__tile--connecting {
    pointer-events: none;
    opacity: 0.7;
  }

  /* Live status dot */
  .sc__status {
    position: absolute;
    top: 0.5rem;
    right: 0.5rem;
    width: 7px;
    height: 7px;
    border-radius: 50%;
  }

  .sc__status--live {
    background: #22c55e;
    box-shadow: 0 0 6px rgba(34, 197, 94, 0.5);
    animation: pulse 2s ease-in-out infinite;
  }

  .sc__status--warn {
    background: #f59e0b;
    box-shadow: 0 0 6px rgba(245, 158, 11, 0.4);
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  /* Icon */
  .sc__icon {
    width: 2rem;
    height: 2rem;
    color: var(--color-text-muted);
    transition: color 0.3s cubic-bezier(0.32, 0.72, 0, 1),
                transform 0.3s cubic-bezier(0.32, 0.72, 0, 1);
  }

  .sc__icon :global(svg) {
    width: 100%;
    height: 100%;
  }

  .sc__tile:hover .sc__icon {
    color: var(--sc-color);
    transform: scale(1.12);
  }

  /* Name */
  .sc__name {
    font-size: 0.6875rem;
    font-weight: 600;
    letter-spacing: 0.01em;
    text-align: center;
    line-height: 1.2;
    color: var(--color-text-tertiary);
    transition: color 0.3s;
  }

  .sc__tile:hover .sc__name {
    color: var(--color-text-primary);
  }

  .sc__tile--connected .sc__name {
    color: var(--color-text-secondary);
  }

  /* Spinner */
  .sc__spinner {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.5);
    border-radius: inherit;
    backdrop-filter: blur(4px);
  }

  .sc__spinner::after {
    content: '';
    width: 1.25rem;
    height: 1.25rem;
    border: 2px solid var(--sc-color);
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
