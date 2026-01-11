<script>
  // Simple test component to verify styles are working
</script>

<div class="min-h-screen flex items-center justify-center" style="background: #27272a;">
  <div class="max-w-md w-full rounded-lg shadow p-6" style="background: #3f3f46;">
    <h1 class="text-2xl font-bold text-zinc-300 mb-4">Style Test</h1>
    <p class="text-zinc-400 mb-4">If you can see this styled properly, the CSS is working!</p>

    <div class="flex gap-4 mb-4">
      <div class="icon icon-md icon-primary">
        <svg viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
        </svg>
      </div>
      <span class="text-success">Icons working</span>
    </div>

    <button class="btn btn-primary w-full">
      Test Button
    </button>
  </div>
</div>
