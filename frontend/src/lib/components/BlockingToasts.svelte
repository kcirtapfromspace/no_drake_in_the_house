<script lang="ts">
  import { onDestroy } from 'svelte';
  import { activeToasts, blockingStore } from '../stores/blocking';
  import { fly, fade } from 'svelte/transition';

  let toastTimers: Map<string, ReturnType<typeof setTimeout>> = new Map();

  const prefersReducedMotion = typeof window !== 'undefined'
    ? window.matchMedia('(prefers-reduced-motion: reduce)').matches
    : false;

  const flyTransition = prefersReducedMotion
    ? { x: 0, duration: 0 }
    : { x: 100, duration: 300 };

  const fadeTransition = prefersReducedMotion
    ? { duration: 0 }
    : { duration: 200 };

  function dismissToast(id: string) {
    blockingStore.removeToast(id);
    const timer = toastTimers.get(id);
    if (timer) {
      clearTimeout(timer);
      toastTimers.delete(id);
    }
  }

  $: {
    $activeToasts.forEach(toast => {
      if (toast.dismissible && toast.duration && !toastTimers.has(toast.id)) {
        const timerId = setTimeout(() => dismissToast(toast.id), toast.duration);
        toastTimers.set(toast.id, timerId);
      }
    });
  }

  onDestroy(() => {
    toastTimers.forEach(timer => clearTimeout(timer));
  });
</script>

<div
  class="toast-container"
  role="region"
  aria-label="Notifications"
  aria-live="polite"
  aria-atomic="false"
>
  {#each $activeToasts as toast (toast.id)}
    {@const isError = toast.type === 'error'}
    <div
      class="toast toast--{toast.type}"
      role="alert"
      aria-live={isError ? 'assertive' : 'polite'}
      in:fly={flyTransition}
      out:fade={fadeTransition}
    >
      <div class="toast__body">
        <div class="toast__content">
          <div class="toast__icon" aria-hidden="true">
            {#if toast.type === 'success'}
              <svg fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
                <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
              </svg>
            {:else if toast.type === 'error'}
              <svg fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
                <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
              </svg>
            {:else if toast.type === 'warning'}
              <svg fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
                <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
              </svg>
            {:else}
              <svg class="{prefersReducedMotion ? '' : 'toast__spinner'}" fill="none" viewBox="0 0 24 24" aria-hidden="true">
                <circle opacity="0.25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path opacity="0.75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
            {/if}
          </div>

          <div class="toast__text">
            <p class="toast__message">{toast.message}</p>

            {#if toast.artistName && toast.type === 'info'}
              <div class="toast__badges">
                <span class="toast__badge toast__badge--spotify">Spotify</span>
                <span class="toast__badge toast__badge--apple">Apple Music</span>
              </div>
            {/if}
          </div>

          {#if toast.dismissible}
            <button
              type="button"
              class="toast__dismiss"
              on:click={() => dismissToast(toast.id)}
              aria-label="Dismiss notification"
            >
              <svg fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          {/if}
        </div>
      </div>

      {#if toast.progress !== undefined && toast.progress < 100}
        <div class="toast__progress-track">
          <div
            class="toast__progress-bar"
            style="width: {toast.progress}%"
          ></div>
        </div>
      {/if}
    </div>
  {/each}
</div>

<style>
  .toast-container {
    position: fixed;
    bottom: var(--space-4);
    right: var(--space-4);
    z-index: var(--z-tooltip, 70);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    max-width: 24rem;
    width: 100%;
    pointer-events: none;
  }

  .toast {
    pointer-events: auto;
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    overflow: hidden;
    backdrop-filter: blur(8px);
    border: 1px solid;
  }

  .toast--success {
    background: var(--color-success-muted);
    border-color: var(--color-success);
    --toast-icon-color: var(--color-success);
    --toast-progress-color: var(--color-success);
  }

  .toast--error {
    background: var(--color-error-muted);
    border-color: var(--color-error);
    --toast-icon-color: var(--color-error);
    --toast-progress-color: var(--color-error);
  }

  .toast--warning {
    background: var(--color-warning-muted);
    border-color: var(--color-warning);
    --toast-icon-color: var(--color-warning);
    --toast-progress-color: var(--color-warning);
  }

  .toast--info {
    background: var(--color-bg-elevated);
    border-color: var(--color-border-default);
    --toast-icon-color: var(--color-text-tertiary);
    --toast-progress-color: var(--color-info);
  }

  .toast__body {
    padding: var(--space-3);
  }

  .toast__content {
    display: flex;
    align-items: flex-start;
    gap: var(--space-3);
  }

  .toast__icon {
    flex-shrink: 0;
    margin-top: 0.125rem;
    color: var(--toast-icon-color);
    width: 1.25rem;
    height: 1.25rem;
  }

  .toast__icon svg {
    width: 100%;
    height: 100%;
  }

  .toast__spinner {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .toast__text {
    flex: 1;
    min-width: 0;
  }

  .toast__message {
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .toast__badges {
    display: flex;
    gap: var(--space-1);
    margin-top: var(--space-2);
  }

  .toast__badge {
    padding: 0.125rem var(--space-1\.5);
    font-size: var(--text-xs);
    border-radius: var(--radius-sm);
  }

  .toast__badge--spotify {
    background: var(--color-spotify-muted);
    color: var(--color-spotify);
  }

  .toast__badge--apple {
    background: var(--color-apple-muted);
    color: var(--color-apple);
  }

  .toast__dismiss {
    flex-shrink: 0;
    padding: var(--space-1);
    border-radius: var(--radius-md);
    background: transparent;
    border: none;
    cursor: pointer;
    color: var(--color-text-tertiary);
    transition: background var(--transition-fast);
  }

  .toast__dismiss:hover {
    background: var(--color-overlay-subtle);
  }

  .toast__dismiss svg {
    width: 1rem;
    height: 1rem;
  }

  .toast__progress-track {
    height: 0.25rem;
    background: var(--color-overlay-inset);
  }

  .toast__progress-bar {
    height: 100%;
    background: var(--toast-progress-color);
    transition: width 0.3s;
  }
</style>
