<script lang="ts">
  import FeatureGate from '../FeatureGate.svelte';

  export let feature: string = 'connections';
  export let accessible: boolean = true;
  export let loading: boolean = false;
</script>

<FeatureGate {feature} {accessible} {loading}>
  <div data-testid="gated-content">Protected content here</div>
</FeatureGate>
