/**
 * Analytics Module — client-side event tracking
 *
 * Privacy-first, zero-dependency, pluggable provider architecture.
 * Respects user consent, strips PII, manages sessions.
 */

/* ------------------------------------------------------------------ */
/*  Types                                                             */
/* ------------------------------------------------------------------ */

export interface AnalyticsEvent {
  name: string;
  properties: Record<string, unknown>;
  timestamp: string;
  sessionId: string;
  userId?: string;
  anonymousId: string;
}

export interface AnalyticsProvider {
  name: string;
  track(event: AnalyticsEvent): void;
  identify(userId: string, traits?: Record<string, unknown>): void;
  reset(): void;
}

export interface AnalyticsConfig {
  enabled: boolean;
  debug: boolean;
  providers: AnalyticsProvider[];
  sessionTimeoutMs: number;
}

/* ------------------------------------------------------------------ */
/*  Constants                                                         */
/* ------------------------------------------------------------------ */

const DEFAULT_SESSION_TIMEOUT_MS = 30 * 60 * 1000; // 30 minutes
const ANONYMOUS_ID_KEY = 'ndith_anonymous_id';
const CONSENT_KEY = 'ndith_analytics_consent';

/**
 * Property keys that may contain PII and must be stripped from events.
 */
const PII_KEYS = new Set([
  'email',
  'password',
  'phone',
  'address',
  'ssn',
  'credit_card',
  'creditCard',
  'ip',
  'ip_address',
  'ipAddress',
  'first_name',
  'firstName',
  'last_name',
  'lastName',
  'full_name',
  'fullName',
  'date_of_birth',
  'dateOfBirth',
  'dob',
]);

/* ------------------------------------------------------------------ */
/*  Internal state                                                    */
/* ------------------------------------------------------------------ */

interface InternalState {
  initialized: boolean;
  config: AnalyticsConfig;
  providers: AnalyticsProvider[];
  userId: string | undefined;
  anonymousId: string;
  sessionId: string;
  lastActivityTs: number;
  consentGiven: boolean;
  queue: Array<{ name: string; properties: Record<string, unknown> }>;
}

let state: InternalState = createFreshState();

function createFreshState(): InternalState {
  return {
    initialized: false,
    config: {
      enabled: true,
      debug: false,
      providers: [],
      sessionTimeoutMs: DEFAULT_SESSION_TIMEOUT_MS,
    },
    providers: [],
    userId: undefined,
    anonymousId: '',
    sessionId: '',
    lastActivityTs: 0,
    consentGiven: false,
    queue: [],
  };
}

/* ------------------------------------------------------------------ */
/*  Utilities                                                         */
/* ------------------------------------------------------------------ */

function generateId(): string {
  // Simple pseudo-UUID v4 without crypto dependency
  const s4 = (): string =>
    Math.floor((1 + Math.random()) * 0x10000)
      .toString(16)
      .substring(1);
  return `${s4()}${s4()}-${s4()}-4${s4().substring(1)}-${s4()}-${s4()}${s4()}${s4()}`;
}

function getOrCreateAnonymousId(): string {
  try {
    const stored = localStorage.getItem(ANONYMOUS_ID_KEY);
    if (stored) return stored;
  } catch {
    // localStorage unavailable — fall through
  }
  const id = generateId();
  try {
    localStorage.setItem(ANONYMOUS_ID_KEY, id);
  } catch {
    // ignore
  }
  return id;
}

function readPersistedConsent(): boolean {
  try {
    return localStorage.getItem(CONSENT_KEY) === 'true';
  } catch {
    return false;
  }
}

function stripPii(
  properties: Record<string, unknown>,
): Record<string, unknown> {
  const cleaned: Record<string, unknown> = {};
  for (const [key, value] of Object.entries(properties)) {
    if (!PII_KEYS.has(key)) {
      cleaned[key] = value;
    }
  }
  return cleaned;
}

function ensureSession(): void {
  const now = Date.now();
  const timeout = state.config.sessionTimeoutMs || DEFAULT_SESSION_TIMEOUT_MS;

  if (
    !state.sessionId ||
    (state.lastActivityTs > 0 && now - state.lastActivityTs > timeout)
  ) {
    state.sessionId = generateId();
  }

  state.lastActivityTs = now;
}

/* ------------------------------------------------------------------ */
/*  Core API                                                          */
/* ------------------------------------------------------------------ */

/**
 * Initialize the analytics module.
 */
export function initAnalytics(config: Partial<AnalyticsConfig>): void {
  state.config = {
    enabled: config.enabled ?? true,
    debug: config.debug ?? false,
    providers: config.providers ?? [],
    sessionTimeoutMs: config.sessionTimeoutMs ?? DEFAULT_SESSION_TIMEOUT_MS,
  };
  state.providers = [...state.config.providers];
  state.anonymousId = getOrCreateAnonymousId();
  state.consentGiven = readPersistedConsent() || state.consentGiven;
  state.initialized = true;

  ensureSession();
  flushQueue();
}

/**
 * Track a named event with optional properties.
 */
export function track(
  name: string,
  properties: Record<string, unknown> = {},
): void {
  if (!state.config.enabled) return;

  if (!state.initialized) {
    // Queue events fired before init
    state.queue.push({ name, properties });
    return;
  }

  if (!state.consentGiven) return;

  ensureSession();

  const event: AnalyticsEvent = {
    name,
    properties: stripPii(properties),
    timestamp: new Date().toISOString(),
    sessionId: state.sessionId,
    anonymousId: state.anonymousId,
    ...(state.userId ? { userId: state.userId } : {}),
  };

  fanOut('track', event);
}

/**
 * Identify the current user.
 */
export function identify(
  userId: string,
  traits?: Record<string, unknown>,
): void {
  state.userId = userId;

  if (!state.config.enabled || !state.consentGiven) return;

  for (const provider of state.providers) {
    try {
      provider.identify(userId, traits);
    } catch {
      // Swallow provider errors
    }
  }
}

/**
 * Reset identity (e.g. on logout).
 */
export function reset(): void {
  state.userId = undefined;

  for (const provider of state.providers) {
    try {
      provider.reset();
    } catch {
      // Swallow provider errors
    }
  }
}

/**
 * Register an additional analytics provider at runtime.
 */
export function registerProvider(provider: AnalyticsProvider): void {
  state.providers.push(provider);
}

/**
 * Set / revoke analytics consent. Persists to localStorage.
 */
export function setConsent(granted: boolean): void {
  state.consentGiven = granted;
  try {
    localStorage.setItem(CONSENT_KEY, String(granted));
  } catch {
    // ignore
  }
}

/* ------------------------------------------------------------------ */
/*  Pre-defined event helpers                                         */
/* ------------------------------------------------------------------ */

export function trackSignup(method: string): void {
  track('signup', { method });
}

export function trackProviderConnected(provider: string): void {
  track('provider_connected', { provider });
}

export function trackFirstScan(provider: string, trackCount: number): void {
  track('first_scan', { provider, trackCount });
}

export function trackFirstEnforcement(
  provider: string,
  actionCount: number,
): void {
  track('first_enforcement', { provider, actionCount });
}

export function trackUpgradeClicked(
  source: string,
  targetPlan: string,
): void {
  track('upgrade_clicked', { source, targetPlan });
}

export function trackSubscriptionStarted(
  plan: string,
  interval: string,
): void {
  track('subscription_started', { plan, interval });
}

export function trackSubscriptionCanceled(
  plan: string,
  reason?: string,
): void {
  track('subscription_canceled', { plan, ...(reason ? { reason } : {}) });
}

export function trackPageView(path: string, title: string): void {
  track('page_view', { path, title });
}

export function trackFeatureGateHit(
  feature: string,
  currentPlan: string,
): void {
  track('feature_gate_hit', { feature, currentPlan });
}

/* ------------------------------------------------------------------ */
/*  Internal helpers                                                  */
/* ------------------------------------------------------------------ */

function fanOut(method: 'track', event: AnalyticsEvent): void {
  for (const provider of state.providers) {
    try {
      provider.track(event);
    } catch {
      // Swallow — a failing provider must not block others
    }
  }
}

function flushQueue(): void {
  if (!state.consentGiven) return;

  const queued = [...state.queue];
  state.queue = [];

  for (const { name, properties } of queued) {
    track(name, properties);
  }
}

/* ------------------------------------------------------------------ */
/*  Test-only helpers (prefixed with underscore)                      */
/* ------------------------------------------------------------------ */

/**
 * Reset all internal state. Intended exclusively for test isolation.
 */
export function _resetForTesting(): void {
  state = createFreshState();
}

/**
 * Expose internal state for assertions. Test-only.
 */
export function _getState(): Readonly<{
  initialized: boolean;
  config: Readonly<AnalyticsConfig>;
  userId: string | undefined;
  anonymousId: string;
  sessionId: string;
  consentGiven: boolean;
  queueLength: number;
}> {
  return {
    initialized: state.initialized,
    config: state.config,
    userId: state.userId,
    anonymousId: state.anonymousId,
    sessionId: state.sessionId,
    consentGiven: state.consentGiven,
    queueLength: state.queue.length,
  };
}
