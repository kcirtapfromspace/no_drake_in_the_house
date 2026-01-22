<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { activeToasts, blockingStore, type Toast } from '../stores/blocking';
  import { fly, fade } from 'svelte/transition';

  let toastTimers: Map<string, ReturnType<typeof setTimeout>> = new Map();

  // Check for reduced motion preference
  const prefersReducedMotion = typeof window !== 'undefined'
    ? window.matchMedia('(prefers-reduced-motion: reduce)').matches
    : false;

  // Transition options respecting reduced motion
  const flyTransition = prefersReducedMotion
    ? { x: 0, duration: 0 }
    : { x: 100, duration: 300 };

  const fadeTransition = prefersReducedMotion
    ? { duration: 0 }
    : { duration: 200 };

  // Keyboard handler for dismissing toasts with Escape
  function handleKeydown(event: KeyboardEvent, toastId: string) {
    if (event.key === 'Escape') {
      dismissToast(toastId);
    }
  }

  function dismissToast(id: string) {
    blockingStore.removeToast(id);
    const timer = toastTimers.get(id);
    if (timer) {
      clearTimeout(timer);
      toastTimers.delete(id);
    }
  }

  function setupAutoRemove(toast: Toast) {
    if (toast.duration && toast.dismissible) {
      const timerId = setTimeout(() => {
        dismissToast(toast.id);
      }, toast.duration);
      toastTimers.set(toast.id, timerId);
    }
  }

  $: {
    $activeToasts.forEach(toast => {
      if (toast.dismissible && toast.duration && !toastTimers.has(toast.id)) {
        setupAutoRemove(toast);
      }
    });
  }

  onDestroy(() => {
    toastTimers.forEach(timer => clearTimeout(timer));
  });

  function getToastStyles(type: Toast['type']) {
    switch (type) {
      case 'success':
        return {
          bg: 'bg-green-900/90',
          border: 'border-green-500',
          icon: 'text-green-400',
          progress: 'bg-green-500',
        };
      case 'error':
        return {
          bg: 'bg-red-900/90',
          border: 'border-red-500',
          icon: 'text-red-400',
          progress: 'bg-red-500',
        };
      case 'warning':
        return {
          bg: 'bg-yellow-900/90',
          border: 'border-yellow-500',
          icon: 'text-yellow-400',
          progress: 'bg-yellow-500',
        };
      default:
        return {
          bg: 'bg-zinc-800/90',
          border: 'border-zinc-600',
          icon: 'text-zinc-400',
          progress: 'bg-indigo-500',
        };
    }
  }
</script>

<!-- Toast Container - Fixed bottom right with ARIA live region for accessibility -->
<div
  class="fixed bottom-4 right-4 z-50 flex flex-col gap-2 max-w-sm w-full pointer-events-none"
  role="region"
  aria-label="Notifications"
  aria-live="polite"
  aria-atomic="false"
>
  {#each $activeToasts as toast (toast.id)}
    {@const styles = getToastStyles(toast.type)}
    {@const isError = toast.type === 'error'}
    <div
      class="pointer-events-auto rounded-lg shadow-xl overflow-hidden backdrop-blur-sm {styles.bg} border {styles.border}"
      role="alert"
      aria-live={isError ? 'assertive' : 'polite'}
      in:fly={flyTransition}
      out:fade={fadeTransition}
      on:keydown={(e) => handleKeydown(e, toast.id)}
    >
      <div class="p-3">
        <div class="flex items-start gap-3">
          <!-- Icon (decorative, hidden from screen readers) -->
          <div class="flex-shrink-0 mt-0.5" aria-hidden="true">
            {#if toast.type === 'success'}
              <svg class="w-5 h-5 {styles.icon}" fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
                <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
              </svg>
            {:else if toast.type === 'error'}
              <svg class="w-5 h-5 {styles.icon}" fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
                <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
              </svg>
            {:else if toast.type === 'warning'}
              <svg class="w-5 h-5 {styles.icon}" fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
                <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
              </svg>
            {:else}
              <!-- Loading spinner for info/progress -->
              <svg class="w-5 h-5 {styles.icon} {prefersReducedMotion ? '' : 'animate-spin'}" fill="none" viewBox="0 0 24 24" aria-hidden="true">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
            {/if}
          </div>

          <!-- Content -->
          <div class="flex-1 min-w-0">
            <p class="text-sm font-medium text-white">
              {toast.message}
            </p>

            <!-- Platform badges for blocking operations -->
            {#if toast.artistName && toast.type === 'info'}
              <div class="flex gap-1 mt-2">
                <span class="px-1.5 py-0.5 text-xs rounded bg-green-500/20 text-green-400">Spotify</span>
                <span class="px-1.5 py-0.5 text-xs rounded bg-rose-500/20 text-rose-400">Apple Music</span>
              </div>
            {/if}
          </div>

          <!-- Dismiss button -->
          {#if toast.dismissible}
            <button
              type="button"
              class="flex-shrink-0 p-1 rounded hover:bg-white/10 transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-white/50"
              on:click={() => dismissToast(toast.id)}
              aria-label="Dismiss notification"
            >
              <svg class="w-4 h-4 text-zinc-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          {/if}
        </div>
      </div>

      <!-- Progress bar -->
      {#if toast.progress !== undefined && toast.progress < 100}
        <div class="h-1 bg-black/20">
          <div
            class="h-full transition-all duration-300 {styles.progress}"
            style="width: {toast.progress}%"
          ></div>
        </div>
      {/if}
    </div>
  {/each}
</div>
