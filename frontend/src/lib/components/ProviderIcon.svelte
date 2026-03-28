<script lang="ts">
  export let provider: string;
  export let size: number = 18;
  export let providers: string[] = [];
  export let maxVisible: number = 3;

  interface ProviderMeta {
    color: string;
    label: string;
  }

  const PROVIDER_META: Record<string, ProviderMeta> = {
    spotify:       { color: '#1DB954', label: 'Spotify' },
    apple:         { color: '#FA2D48', label: 'Apple Music' },
    apple_music:   { color: '#FA2D48', label: 'Apple Music' },
    youtube:       { color: '#FF0000', label: 'YouTube Music' },
    youtube_music: { color: '#FF0000', label: 'YouTube Music' },
    tidal:         { color: '#FFFFFF', label: 'Tidal' },
    deezer:        { color: '#FEAA2D', label: 'Deezer' },
  };

  function getMeta(p: string): ProviderMeta {
    return PROVIDER_META[p] ?? { color: '#71717a', label: p };
  }

  $: allProviders = providers.length > 0 ? providers : provider ? [provider] : [];
  $: visible = allProviders.slice(0, maxVisible);
  $: overflow = allProviders.length - maxVisible;
  $: overflowTooltip = allProviders.slice(maxVisible).map(p => getMeta(p).label).join(', ');
</script>

<span class="provider-icon-stack" style="--icon-size: {size}px">
  {#each visible as p}
    {@const meta = getMeta(p)}
    <span
      class="provider-icon"
      title={meta.label}
      style="color: {meta.color}"
    >
      {#if p === 'spotify'}
        <svg viewBox="0 0 24 24" fill="currentColor" width={size} height={size}>
          <path d="M12 0C5.4 0 0 5.4 0 12s5.4 12 12 12 12-5.4 12-12S18.66 0 12 0zm5.521 17.34c-.24.359-.66.48-1.021.24-2.82-1.74-6.36-2.101-10.561-1.141-.418.122-.779-.179-.899-.539-.12-.421.18-.78.54-.9 4.56-1.021 8.52-.6 11.64 1.32.42.18.479.659.301 1.02zm1.44-3.3c-.301.42-.841.6-1.262.3-3.239-1.98-8.159-2.58-11.939-1.38-.479.12-1.02-.12-1.14-.6-.12-.48.12-1.021.6-1.141C9.6 9.9 15 10.561 18.72 12.84c.361.181.54.78.241 1.2zm.12-3.36C15.24 8.4 8.82 8.16 5.16 9.301c-.6.179-1.2-.181-1.38-.721-.18-.601.18-1.2.72-1.381 4.26-1.26 11.28-1.02 15.721 1.621.539.3.719 1.02.419 1.56-.299.421-1.02.599-1.559.3z"/>
        </svg>
      {:else if p === 'apple' || p === 'apple_music'}
        <svg viewBox="0 0 24 24" fill="currentColor" width={size} height={size}>
          <path d="M23.994 6.124a9.23 9.23 0 0 0-.24-2.19c-.317-1.31-1.062-2.31-2.18-3.043A5.022 5.022 0 0 0 19.32.089a10.16 10.16 0 0 0-1.898-.067H6.574a10.12 10.12 0 0 0-1.898.067A5.03 5.03 0 0 0 2.42.891C1.3 1.624.556 2.624.24 3.934a9.23 9.23 0 0 0-.24 2.19v11.752c.007.68.072 1.36.24 2.19.317 1.31 1.062 2.31 2.18 3.043A5.022 5.022 0 0 0 4.676 23.91c.629.062 1.263.072 1.898.067h10.852c.635.005 1.269-.005 1.898-.067a5.03 5.03 0 0 0 2.256-.801c1.118-.733 1.863-1.733 2.18-3.043a9.23 9.23 0 0 0 .24-2.19l-.006-11.746zM16.95 16.61c0 1.225-.397 2.186-1.194 2.876-.752.65-1.675.975-2.77.975-.476 0-.973-.088-1.484-.262a2.56 2.56 0 0 1-1.133-.756c-.333.256-.712.463-1.132.619-.534.199-1.089.299-1.664.299-1.14 0-2.11-.369-2.906-1.107-.795-.737-1.194-1.714-1.194-2.93V10.09c0-.186.062-.343.186-.47a.625.625 0 0 1 .464-.193h.78c.186 0 .344.064.47.193a.637.637 0 0 1 .193.47v6.166c0 .687.21 1.245.633 1.672.422.427.959.64 1.609.64.53 0 1.003-.149 1.421-.448a.887.887 0 0 1-.016-.169V10.09c0-.186.063-.343.187-.47a.625.625 0 0 1 .464-.193h.78c.186 0 .343.064.469.193a.637.637 0 0 1 .193.47v7.842c0 .648.209 1.18.627 1.598.418.418.95.627 1.595.627.646 0 1.178-.209 1.596-.627.418-.418.627-.95.627-1.598V10.09c0-.186.063-.343.187-.47a.625.625 0 0 1 .464-.193h.78c.186 0 .343.064.47.193a.637.637 0 0 1 .192.47v6.52zM11.12 7.095c-.193.198-.433.298-.718.298a.978.978 0 0 1-.719-.298.978.978 0 0 1-.298-.719c0-.28.1-.52.298-.718a.978.978 0 0 1 .72-.298c.284 0 .524.1.717.298.193.198.29.438.29.718s-.097.52-.29.719zm5.16 0c-.193.198-.433.298-.718.298a.978.978 0 0 1-.719-.298.978.978 0 0 1-.298-.719c0-.28.1-.52.298-.718a.978.978 0 0 1 .72-.298c.284 0 .524.1.717.298.193.198.29.438.29.718s-.097.52-.29.719z"/>
        </svg>
      {:else if p === 'youtube' || p === 'youtube_music'}
        <svg viewBox="0 0 24 24" fill="currentColor" width={size} height={size}>
          <path d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z"/>
        </svg>
      {:else if p === 'tidal'}
        <svg viewBox="0 0 24 24" fill="currentColor" width={size} height={size}>
          <path d="M12.012 3.992L8.008 7.996 4.004 3.992 0 7.996l4.004 4.004L8.008 8l4.004 4 4.004-4-4.004-4.004zm0 8.008l-4.004 4.004L4.004 12 0 16.004 4.004 20.008 8.008 16.004l4.004 4.004L16.016 16.004 12.012 12zm4.004-4.004L20.02 3.992 24.024 7.996 20.02 12.0l-4.004-4.004z"/>
        </svg>
      {:else if p === 'deezer'}
        <svg viewBox="0 0 24 24" fill="currentColor" width={size} height={size}>
          <path d="M18.81 4.16v3.03H24V4.16h-5.19zM6.27 8.38v3.027h5.189V8.38H6.27zm12.54 0v3.027H24V8.38h-5.19zM6.27 12.594v3.027h5.189v-3.027H6.27zm6.271 0v3.027h5.19v-3.027h-5.19zm6.27 0v3.027H24v-3.027h-5.19zM0 16.81v3.029h5.19V16.81H0zm6.27 0v3.029h5.189V16.81H6.27zm6.271 0v3.029h5.19V16.81h-5.19zm6.27 0v3.029H24V16.81h-5.19z"/>
        </svg>
      {:else}
        <svg viewBox="0 0 24 24" fill="currentColor" width={size} height={size}>
          <circle cx="12" cy="12" r="10"/>
          <path d="M8 15V9l7 3-7 3z" fill="#27272a"/>
        </svg>
      {/if}
    </span>
  {/each}
  {#if overflow > 0}
    <span class="provider-icon-overflow" title={overflowTooltip}>+{overflow}</span>
  {/if}
</span>

<style>
  .provider-icon-stack {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }

  .provider-icon {
    display: inline-flex;
    align-items: center;
    flex-shrink: 0;
    line-height: 0;
  }

  .provider-icon svg {
    width: var(--icon-size, 18px);
    height: var(--icon-size, 18px);
  }

  .provider-icon-overflow {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    font-size: 10px;
    font-weight: 600;
    color: var(--color-text-tertiary, #a1a1aa);
    background: var(--color-surface-elevated, #3f3f46);
    border-radius: 9999px;
    min-width: var(--icon-size, 18px);
    height: var(--icon-size, 18px);
    padding: 0 4px;
    cursor: default;
  }
</style>
