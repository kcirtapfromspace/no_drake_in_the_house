<script lang="ts">
  import { onMount } from 'svelte';
  import { getGradeColor, getGlowColor } from '../utils/playlist-helpers';

  export let score: number = 0;
  export let grade: string = 'F';
  export let size: number = 160;
  export let animated: boolean = true;

  let mounted = false;
  onMount(() => { requestAnimationFrame(() => { mounted = true; }); });

  $: radius = (size - 20) / 2;
  $: circumference = 2 * Math.PI * radius;
  $: targetOffset = circumference - (score / 100) * circumference;
  $: dashOffset = animated && mounted ? targetOffset : circumference;
  $: gradeColor = getGradeColor(grade);
  $: glowColor = getGlowColor(grade);
</script>

<div class="gauge" style="width: {size}px; height: {size}px;">
  <svg viewBox="0 0 {size} {size}" class="gauge__svg">
    <defs>
      <filter id="glow-{grade}" x="-50%" y="-50%" width="200%" height="200%">
        <feGaussianBlur in="SourceGraphic" stdDeviation="4" result="blur" />
        <feMerge>
          <feMergeNode in="blur" />
          <feMergeNode in="SourceGraphic" />
        </feMerge>
      </filter>
    </defs>

    <!-- Background circle -->
    <circle
      cx={size / 2}
      cy={size / 2}
      r={radius}
      fill="none"
      stroke="var(--color-border-subtle, #1f1f22)"
      stroke-width="8"
      opacity="0.6"
    />

    <!-- Track marks (subtle tick marks around the circle) -->
    {#each Array(24) as _, i}
      <line
        x1={size / 2 + (radius - 2) * Math.cos((i * 15 - 90) * Math.PI / 180)}
        y1={size / 2 + (radius - 2) * Math.sin((i * 15 - 90) * Math.PI / 180)}
        x2={size / 2 + (radius + 2) * Math.cos((i * 15 - 90) * Math.PI / 180)}
        y2={size / 2 + (radius + 2) * Math.sin((i * 15 - 90) * Math.PI / 180)}
        stroke="var(--color-border-subtle, #1f1f22)"
        stroke-width="1"
        opacity="0.3"
      />
    {/each}

    <!-- Score arc with glow -->
    <circle
      cx={size / 2}
      cy={size / 2}
      r={radius}
      fill="none"
      stroke={gradeColor}
      stroke-width="9"
      stroke-dasharray={circumference}
      stroke-dashoffset={dashOffset}
      stroke-linecap="round"
      transform="rotate(-90 {size / 2} {size / 2})"
      class="gauge__arc"
      filter="url(#glow-{grade})"
    />
  </svg>

  <div class="gauge__content" class:gauge__content--visible={mounted || !animated}>
    <span class="gauge__grade" style="color: {gradeColor}; text-shadow: 0 0 20px {glowColor};">{grade}</span>
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
    transition: stroke-dashoffset 1.2s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .gauge__content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.0625rem;
    z-index: 1;
    opacity: 0;
    transform: scale(0.8);
    transition: opacity 0.5s ease 0.4s, transform 0.5s cubic-bezier(0.34, 1.56, 0.64, 1) 0.4s;
  }

  .gauge__content--visible {
    opacity: 1;
    transform: scale(1);
  }

  .gauge__grade {
    font-size: 2.25rem;
    font-weight: 800;
    line-height: 1;
    letter-spacing: -0.02em;
  }

  .gauge__score {
    font-size: 0.8125rem;
    color: var(--color-text-tertiary, #71717a);
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    letter-spacing: 0.02em;
  }
</style>
