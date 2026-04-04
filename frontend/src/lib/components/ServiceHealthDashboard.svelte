<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { apiClient } from '../utils/api-client';
  import { navigateTo } from '../utils/simple-router';

  interface SystemHealth {
    overall: 'healthy' | 'degraded' | 'unhealthy';
    databases: {
      postgres: boolean;
      redis: boolean;
      duckdb: boolean;
      ladybugdb: boolean;
      lancedb: boolean;
    };
    latencies_ms?: {
      postgres?: number;
      redis?: number;
      duckdb?: number;
      ladybugdb?: number;
      lancedb?: number;
    };
  }

  let isLoading = true;
  let lastChecked: Date | null = null;
  let refreshInterval: ReturnType<typeof setInterval> | null = null;
  let systemHealth: SystemHealth | null = null;

  const REFRESH_INTERVAL_MS = 30000; // 30 seconds

  interface ServiceInfo {
    name: string;
    key: keyof SystemHealth['databases'];
    description: string;
  }

  const services: ServiceInfo[] = [
    { name: 'PostgreSQL', key: 'postgres', description: 'Primary relational database' },
    { name: 'Redis', key: 'redis', description: 'Session cache and rate limiting' },
    { name: 'DuckDB', key: 'duckdb', description: 'Analytics and OLAP queries' },
    { name: 'LadybugDB', key: 'ladybugdb', description: 'Graph database for relationships' },
    { name: 'LanceDB', key: 'lancedb', description: 'Vector search and embeddings' },
  ];

  async function fetchHealth() {
    isLoading = true;
    try {
      const result = await apiClient.authenticatedRequest<any>('GET', '/api/v1/analytics/health');
      if (result.success && result.data) {
        const apiData = result.data;
        systemHealth = {
          overall: apiData.overall_status || 'unhealthy',
          databases: {
            postgres: apiData.services?.postgres?.healthy ?? false,
            redis: apiData.services?.redis?.healthy ?? false,
            duckdb: apiData.services?.duckdb?.healthy ?? false,
            ladybugdb: apiData.services?.ladybugdb?.healthy ?? apiData.services?.kuzu?.healthy ?? false,
            lancedb: apiData.services?.lancedb?.healthy ?? false,
          },
          latencies_ms: {
            postgres: apiData.services?.postgres?.latency_ms,
            redis: apiData.services?.redis?.latency_ms,
            duckdb: apiData.services?.duckdb?.latency_ms,
            ladybugdb: apiData.services?.ladybugdb?.latency_ms ?? apiData.services?.kuzu?.latency_ms,
            lancedb: apiData.services?.lancedb?.latency_ms,
          },
        };
      }
    } catch (error) {
      // Silently fail - will show Unknown status
    }
    lastChecked = new Date();
    isLoading = false;
  }

  function getStatusColor(isHealthy: boolean | undefined): string {
    if (isHealthy === undefined) return 'text-zinc-400 bg-zinc-400/15';
    return isHealthy ? 'text-green-400 bg-green-400/15' : 'text-red-400 bg-red-400/15';
  }

  function getStatusText(isHealthy: boolean | undefined): string {
    if (isHealthy === undefined) return 'Unknown';
    return isHealthy ? 'Healthy' : 'Unhealthy';
  }

  function formatLatency(latency: number | undefined): string {
    return latency === undefined ? '--' : `${latency}ms`;
  }

  function formatLastChecked(date: Date | null): string {
    return date ? date.toLocaleTimeString() : 'Never';
  }

  onMount(() => {
    fetchHealth();
    refreshInterval = setInterval(fetchHealth, REFRESH_INTERVAL_MS);
  });

  onDestroy(() => {
    if (refreshInterval) {
      clearInterval(refreshInterval);
    }
  });
</script>

<div class="health-dashboard brand-page surface-page">
  <div class="health-dashboard__container brand-page__inner brand-page__stack">
    <section class="brand-hero">
      <div class="brand-hero__header">
        <div class="brand-hero__copy">
          <button
            type="button"
            on:click={() => navigateTo('settings')}
            class="brand-back"
          >
            <svg fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
            </svg>
            Back to Settings
          </button>
          <div class="brand-kickers">
            <span class="brand-kicker">Developer Tools</span>
            <span class="brand-kicker brand-kicker--accent">Runtime Health</span>
          </div>
          <h1 class="brand-title brand-title--compact">Keep the backend visible while the product stays polished.</h1>
          <p class="brand-subtitle">
            Monitor core services, confirm response times, and catch degraded infrastructure without dropping into an off-brand admin panel.
          </p>
          <div class="brand-meta">
            <span class="brand-meta__item">Last checked: {formatLastChecked(lastChecked)}</span>
          </div>
        </div>

        <div class="brand-hero__aside">
          <div class="brand-stat-grid brand-stat-grid--compact" aria-label="Health overview">
            <div class="brand-stat">
              <span class="brand-stat__value">{systemHealth ? systemHealth.overall.toUpperCase() : 'NO DATA'}</span>
              <span class="brand-stat__label">Overall status</span>
            </div>
            <div class="brand-stat">
              <span class="brand-stat__value">{services.filter((service) => systemHealth?.databases?.[service.key]).length}</span>
              <span class="brand-stat__label">Healthy services</span>
            </div>
          </div>

          <div class="brand-actions">
            <button
              type="button"
              on:click={fetchHealth}
              disabled={isLoading}
              class="brand-button brand-button--secondary"
            >
              {#if isLoading}
                <div class="brand-button__spinner"></div>
              {:else}
                <svg fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                </svg>
              {/if}
              <span>Refresh</span>
            </button>
          </div>
        </div>
      </div>
    </section>

    <!-- Services Grid -->
    <div class="health-dashboard__grid">
      {#each services as service}
        {@const isHealthy = systemHealth?.databases?.[service.key]}
        {@const latency = systemHealth?.latencies_ms?.[service.key]}
        <div class="health-dashboard__card">
          <div class="health-dashboard__card-header">
            <h3 class="health-dashboard__card-title">{service.name}</h3>
            <span class="health-dashboard__card-badge {getStatusColor(isHealthy)}">
              {getStatusText(isHealthy)}
            </span>
          </div>
          <p class="health-dashboard__card-desc">{service.description}</p>
          <div class="health-dashboard__card-metrics">
            <div class="health-dashboard__metric">
              <span class="health-dashboard__metric-label">Response Time</span>
              <span class="health-dashboard__metric-value">{formatLatency(latency)}</span>
            </div>
          </div>
        </div>
      {/each}
    </div>

    <!-- Auto-refresh notice -->
    <p class="health-dashboard__notice">
      Auto-refreshes every 30 seconds
    </p>
  </div>
</div>

<style>
  .health-dashboard {
    min-height: 100vh;
  }

  .health-dashboard__container {
    width: 100%;
  }

  .health-dashboard__grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 1rem;
  }

  .health-dashboard__card {
    background:
      linear-gradient(180deg, rgba(255, 255, 255, 0.045), rgba(255, 255, 255, 0.02)),
      rgba(17, 17, 19, 0.88);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: var(--radius-xl);
    padding: 1.25rem;
    transition: border-color var(--transition-fast), transform var(--transition-fast);
    box-shadow: 0 20px 48px rgba(0, 0, 0, 0.18);
    backdrop-filter: blur(12px);
  }

  .health-dashboard__card:hover {
    border-color: rgba(244, 63, 94, 0.2);
    transform: translateY(-1px);
  }

  .health-dashboard__card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    margin-bottom: 0.5rem;
  }

  .health-dashboard__card-title {
    font-size: var(--text-base);
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .health-dashboard__card-badge {
    padding: 0.25rem 0.625rem;
    font-size: var(--text-xs);
    font-weight: 600;
    border-radius: var(--radius-full);
  }

  .health-dashboard__card-desc {
    font-size: var(--text-sm);
    color: var(--color-text-tertiary);
    margin-bottom: 1rem;
  }

  .health-dashboard__card-metrics {
    display: flex;
    gap: 1.5rem;
  }

  .health-dashboard__metric {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .health-dashboard__metric-label {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .health-dashboard__metric-value {
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--color-text-primary);
    display: flex;
    align-items: center;
  }

  .health-dashboard__notice {
    text-align: center;
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    margin-top: 1.5rem;
  }
</style>
