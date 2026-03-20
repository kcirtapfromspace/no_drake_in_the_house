<script lang="ts">
  export let score: number = 0;
  export let grade: string = 'F';
  export let size: number = 160;

  $: radius = (size - 20) / 2;
  $: circumference = 2 * Math.PI * radius;
  $: dashOffset = circumference - (score / 100) * circumference;
  $: gradeColor = getGradeColor(grade);

  function getGradeColor(g: string): string {
    switch (g) {
      case 'A': return 'var(--color-brand-success, #22c55e)';
      case 'B': return 'var(--color-brand-info, #3b82f6)';
      case 'C': return 'var(--color-brand-warning, #eab308)';
      case 'D': return 'var(--color-brand-caution, #f97316)';
      default:  return 'var(--color-brand-danger, #ef4444)';
    }
  }
</script>

<div class="gauge" style="width: {size}px; height: {size}px;">
  <svg viewBox="0 0 {size} {size}" class="gauge__svg">
    <!-- Background circle -->
    <circle
      cx={size / 2}
      cy={size / 2}
      r={radius}
      fill="none"
      stroke="var(--color-border-subtle, #333)"
      stroke-width="10"
    />
    <!-- Score arc -->
    <circle
      cx={size / 2}
      cy={size / 2}
      r={radius}
      fill="none"
      stroke={gradeColor}
      stroke-width="10"
      stroke-dasharray={circumference}
      stroke-dashoffset={dashOffset}
      stroke-linecap="round"
      transform="rotate(-90 {size / 2} {size / 2})"
      class="gauge__arc"
    />
  </svg>
  <div class="gauge__content">
    <span class="gauge__grade" style="color: {gradeColor};">{grade}</span>
    <span class="gauge__score">{score}%</span>
  </div>
</div>

<style>
  .gauge {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .gauge__svg {
    position: absolute;
    inset: 0;
  }

  .gauge__arc {
    transition: stroke-dashoffset 0.8s ease-out;
  }

  .gauge__content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.125rem;
    z-index: 1;
  }

  .gauge__grade {
    font-size: 2.5rem;
    font-weight: 800;
    line-height: 1;
  }

  .gauge__score {
    font-size: 0.875rem;
    color: var(--color-text-secondary, #999);
    font-weight: 500;
  }
</style>
