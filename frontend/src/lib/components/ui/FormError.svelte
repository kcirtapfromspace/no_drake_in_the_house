<script lang="ts">
  // Props
  export let message: string;
  export let id: string = '';  // Used for aria-describedby linking to input
</script>

{#if message}
  <div
    class="form-error"
    role="alert"
    aria-live="polite"
    {id}
  >
    <svg
      class="form-error__icon"
      fill="currentColor"
      viewBox="0 0 20 20"
      aria-hidden="true"
    >
      <path
        fill-rule="evenodd"
        d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z"
        clip-rule="evenodd"
      />
    </svg>
    <span>{message}</span>
  </div>
{/if}

<style>
  .form-error {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-top: 0.5rem;
    font-size: var(--text-sm);
    color: var(--color-error);
  }

  .form-error__icon {
    width: 1rem;
    height: 1rem;
    flex-shrink: 0;
  }
</style>
