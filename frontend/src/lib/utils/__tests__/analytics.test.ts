/**
 * Analytics Module Tests
 * Client-side event tracking with provider abstraction, consent, and sessions.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

// We test the module by importing it. Each test resets internal state.
import {
  initAnalytics,
  track,
  identify,
  reset,
  setConsent,
  registerProvider,
  trackSignup,
  trackProviderConnected,
  trackFirstScan,
  trackFirstEnforcement,
  trackUpgradeClicked,
  trackSubscriptionStarted,
  trackSubscriptionCanceled,
  trackPageView,
  trackFeatureGateHit,
  _resetForTesting,
  _getState,
  type AnalyticsEvent,
  type AnalyticsProvider,
} from '../analytics';

import { NoopProvider, ConsoleProvider } from '../analytics-providers';

/* ------------------------------------------------------------------ */
/*  Helpers                                                           */
/* ------------------------------------------------------------------ */

function createMockProvider(name = 'mock'): AnalyticsProvider & {
  trackCalls: AnalyticsEvent[];
  identifyCalls: Array<{ userId: string; traits?: Record<string, unknown> }>;
  resetCalls: number;
} {
  const provider = {
    name,
    trackCalls: [] as AnalyticsEvent[],
    identifyCalls: [] as Array<{ userId: string; traits?: Record<string, unknown> }>,
    resetCalls: 0,
    track(event: AnalyticsEvent) {
      provider.trackCalls.push(event);
    },
    identify(userId: string, traits?: Record<string, unknown>) {
      provider.identifyCalls.push({ userId, traits });
    },
    reset() {
      provider.resetCalls += 1;
    },
  };
  return provider;
}

/* ------------------------------------------------------------------ */
/*  Setup / Teardown                                                  */
/* ------------------------------------------------------------------ */

beforeEach(() => {
  _resetForTesting();
  vi.useFakeTimers();
  // localStorage is already mocked in test setup – clear call records
  (localStorage.getItem as ReturnType<typeof vi.fn>).mockReset();
  (localStorage.setItem as ReturnType<typeof vi.fn>).mockReset();
  (localStorage.removeItem as ReturnType<typeof vi.fn>).mockReset();
});

afterEach(() => {
  vi.useRealTimers();
  vi.restoreAllMocks();
});

/* ================================================================== */
/*  Test Suites                                                       */
/* ================================================================== */

describe('Analytics Module', () => {
  /* ---------------------------------------------------------------- */
  describe('initialization', () => {
    it('should initialize with default config', () => {
      initAnalytics({});
      const state = _getState();
      expect(state.initialized).toBe(true);
      expect(state.config.enabled).toBe(true);
      expect(state.config.debug).toBe(false);
      expect(state.config.sessionTimeoutMs).toBe(30 * 60 * 1000);
    });

    it('should not track when disabled', () => {
      const provider = createMockProvider();
      initAnalytics({ enabled: false, providers: [provider] });
      setConsent(true);

      track('test_event');

      expect(provider.trackCalls).toHaveLength(0);
    });

    it('should queue events before initialization', () => {
      const provider = createMockProvider();
      // Consent given but not yet initialized
      setConsent(true);
      track('queued_event_1');
      track('queued_event_2');

      // Now initialize – queue should flush
      initAnalytics({ providers: [provider] });

      expect(provider.trackCalls).toHaveLength(2);
      expect(provider.trackCalls[0].name).toBe('queued_event_1');
      expect(provider.trackCalls[1].name).toBe('queued_event_2');
    });

    it('should flush queue after initialization', () => {
      const provider = createMockProvider();
      setConsent(true);
      track('early_event');

      expect(provider.trackCalls).toHaveLength(0); // not yet delivered

      initAnalytics({ providers: [provider] });

      expect(provider.trackCalls).toHaveLength(1);
      expect(provider.trackCalls[0].name).toBe('early_event');
    });
  });

  /* ---------------------------------------------------------------- */
  describe('event tracking', () => {
    let provider: ReturnType<typeof createMockProvider>;

    beforeEach(() => {
      provider = createMockProvider();
      initAnalytics({ providers: [provider] });
      setConsent(true);
    });

    it('should track signup event with correct properties', () => {
      trackSignup('email');

      expect(provider.trackCalls).toHaveLength(1);
      expect(provider.trackCalls[0].name).toBe('signup');
      expect(provider.trackCalls[0].properties.method).toBe('email');
    });

    it('should track provider_connected with provider name', () => {
      trackProviderConnected('spotify');

      expect(provider.trackCalls).toHaveLength(1);
      expect(provider.trackCalls[0].name).toBe('provider_connected');
      expect(provider.trackCalls[0].properties.provider).toBe('spotify');
    });

    it('should track first_scan with provider and track count', () => {
      trackFirstScan('spotify', 1234);

      expect(provider.trackCalls).toHaveLength(1);
      expect(provider.trackCalls[0].name).toBe('first_scan');
      expect(provider.trackCalls[0].properties.provider).toBe('spotify');
      expect(provider.trackCalls[0].properties.trackCount).toBe(1234);
    });

    it('should track first_enforcement with provider and action count', () => {
      trackFirstEnforcement('tidal', 42);

      expect(provider.trackCalls).toHaveLength(1);
      expect(provider.trackCalls[0].name).toBe('first_enforcement');
      expect(provider.trackCalls[0].properties.provider).toBe('tidal');
      expect(provider.trackCalls[0].properties.actionCount).toBe(42);
    });

    it('should track upgrade_clicked with source and target plan', () => {
      trackUpgradeClicked('settings_page', 'pro');

      expect(provider.trackCalls).toHaveLength(1);
      expect(provider.trackCalls[0].name).toBe('upgrade_clicked');
      expect(provider.trackCalls[0].properties.source).toBe('settings_page');
      expect(provider.trackCalls[0].properties.targetPlan).toBe('pro');
    });

    it('should track subscription_started with plan and billing interval', () => {
      trackSubscriptionStarted('pro', 'monthly');

      expect(provider.trackCalls).toHaveLength(1);
      expect(provider.trackCalls[0].name).toBe('subscription_started');
      expect(provider.trackCalls[0].properties.plan).toBe('pro');
      expect(provider.trackCalls[0].properties.interval).toBe('monthly');
    });

    it('should track subscription_canceled with plan and reason', () => {
      trackSubscriptionCanceled('pro', 'too_expensive');

      expect(provider.trackCalls).toHaveLength(1);
      expect(provider.trackCalls[0].name).toBe('subscription_canceled');
      expect(provider.trackCalls[0].properties.plan).toBe('pro');
      expect(provider.trackCalls[0].properties.reason).toBe('too_expensive');
    });

    it('should track page_view with path and title', () => {
      trackPageView('/settings', 'Settings');

      expect(provider.trackCalls).toHaveLength(1);
      expect(provider.trackCalls[0].name).toBe('page_view');
      expect(provider.trackCalls[0].properties.path).toBe('/settings');
      expect(provider.trackCalls[0].properties.title).toBe('Settings');
    });

    it('should track feature_gate_hit with feature name and current plan', () => {
      trackFeatureGateHit('advanced_reports', 'free');

      expect(provider.trackCalls).toHaveLength(1);
      expect(provider.trackCalls[0].name).toBe('feature_gate_hit');
      expect(provider.trackCalls[0].properties.feature).toBe('advanced_reports');
      expect(provider.trackCalls[0].properties.currentPlan).toBe('free');
    });

    it('should include common properties (timestamp, sessionId, anonymousId) in all events', () => {
      track('any_event', { foo: 'bar' });

      const event = provider.trackCalls[0];
      expect(event.timestamp).toBeDefined();
      expect(typeof event.timestamp).toBe('string');
      // Should be ISO-8601
      expect(() => new Date(event.timestamp)).not.toThrow();

      expect(event.sessionId).toBeDefined();
      expect(event.sessionId.length).toBeGreaterThan(0);

      expect(event.anonymousId).toBeDefined();
      expect(event.anonymousId.length).toBeGreaterThan(0);
    });

    it('should not include PII in event properties', () => {
      // Track events with user data – the module must strip PII
      identify('user-123', { email: 'user@example.com', name: 'Test User' });
      track('some_event', { email: 'shouldnt@appear.com', password: 'secret' });

      const event = provider.trackCalls[0];
      // The event should not contain email or password fields
      expect(event.properties.email).toBeUndefined();
      expect(event.properties.password).toBeUndefined();
      // The event should still contain any non-PII properties passed at the call site
      // (In this case there were none apart from the PII fields, so properties may be empty)
    });
  });

  /* ---------------------------------------------------------------- */
  describe('provider abstraction', () => {
    it('should support registering custom providers', () => {
      const provider = createMockProvider('custom');
      initAnalytics({});
      setConsent(true);

      registerProvider(provider);
      track('test_event');

      expect(provider.trackCalls).toHaveLength(1);
    });

    it('should fan out events to all registered providers', () => {
      const provider1 = createMockProvider('p1');
      const provider2 = createMockProvider('p2');
      initAnalytics({ providers: [provider1, provider2] });
      setConsent(true);

      track('fan_out_event');

      expect(provider1.trackCalls).toHaveLength(1);
      expect(provider2.trackCalls).toHaveLength(1);
      expect(provider1.trackCalls[0].name).toBe('fan_out_event');
      expect(provider2.trackCalls[0].name).toBe('fan_out_event');
    });

    it('should handle provider errors without blocking', () => {
      const badProvider: AnalyticsProvider = {
        name: 'bad',
        track() { throw new Error('boom'); },
        identify() { throw new Error('boom'); },
        reset() { throw new Error('boom'); },
      };
      const goodProvider = createMockProvider('good');

      initAnalytics({ providers: [badProvider, goodProvider] });
      setConsent(true);

      // Should not throw even though badProvider throws
      expect(() => track('safe_event')).not.toThrow();

      // Good provider should still receive the event
      expect(goodProvider.trackCalls).toHaveLength(1);
    });

    it('should support console provider for development', () => {
      const consoleProvider = new ConsoleProvider();
      expect(consoleProvider.name).toBe('console');

      // Should be callable without errors
      expect(() => {
        consoleProvider.track({
          name: 'test',
          properties: {},
          timestamp: new Date().toISOString(),
          sessionId: 'sess-1',
          anonymousId: 'anon-1',
        });
      }).not.toThrow();
    });

    it('should support noop provider for testing', () => {
      const noopProvider = new NoopProvider();
      expect(noopProvider.name).toBe('noop');

      // Should be callable without side effects
      expect(() => {
        noopProvider.track({
          name: 'test',
          properties: {},
          timestamp: new Date().toISOString(),
          sessionId: 'sess-1',
          anonymousId: 'anon-1',
        });
        noopProvider.identify('user-1');
        noopProvider.reset();
      }).not.toThrow();
    });
  });

  /* ---------------------------------------------------------------- */
  describe('user identification', () => {
    let provider: ReturnType<typeof createMockProvider>;

    beforeEach(() => {
      provider = createMockProvider();
      initAnalytics({ providers: [provider] });
      setConsent(true);
    });

    it('should identify user with userId', () => {
      identify('user-42');

      expect(provider.identifyCalls).toHaveLength(1);
      expect(provider.identifyCalls[0].userId).toBe('user-42');
    });

    it('should reset on logout', () => {
      identify('user-42');
      reset();

      expect(provider.resetCalls).toBe(1);

      // After reset, tracked events should have no userId
      track('after_reset');
      expect(provider.trackCalls[0].userId).toBeUndefined();
    });

    it('should persist anonymous ID across page loads', () => {
      // First "page load": initAnalytics should store anonymousId
      const storedId = 'persisted-anon-id-123';
      (localStorage.getItem as ReturnType<typeof vi.fn>).mockImplementation((key: string) => {
        if (key === 'ndith_anonymous_id') return storedId;
        return null;
      });

      _resetForTesting();
      initAnalytics({ providers: [provider] });
      setConsent(true);

      track('check_anon');
      expect(provider.trackCalls[0].anonymousId).toBe(storedId);
    });

    it('should alias anonymous ID to user ID on signup', () => {
      track('before_signup');
      const anonId = provider.trackCalls[0].anonymousId;

      identify('user-new');

      // After identification, events carry both userId and the same anonymousId
      track('after_signup');
      expect(provider.trackCalls[1].userId).toBe('user-new');
      expect(provider.trackCalls[1].anonymousId).toBe(anonId);
    });
  });

  /* ---------------------------------------------------------------- */
  describe('consent management', () => {
    let provider: ReturnType<typeof createMockProvider>;

    beforeEach(() => {
      provider = createMockProvider();
      initAnalytics({ providers: [provider] });
    });

    it('should not track until consent given', () => {
      // Consent not yet given (default)
      track('no_consent_event');

      expect(provider.trackCalls).toHaveLength(0);
    });

    it('should track after consent given', () => {
      setConsent(true);
      track('consented_event');

      expect(provider.trackCalls).toHaveLength(1);
      expect(provider.trackCalls[0].name).toBe('consented_event');
    });

    it('should stop tracking after consent revoked', () => {
      setConsent(true);
      track('before_revoke');

      setConsent(false);
      track('after_revoke');

      expect(provider.trackCalls).toHaveLength(1);
      expect(provider.trackCalls[0].name).toBe('before_revoke');
    });

    it('should persist consent preference', () => {
      setConsent(true);

      expect(localStorage.setItem).toHaveBeenCalledWith(
        'ndith_analytics_consent',
        'true',
      );

      setConsent(false);

      expect(localStorage.setItem).toHaveBeenCalledWith(
        'ndith_analytics_consent',
        'false',
      );
    });
  });

  /* ---------------------------------------------------------------- */
  describe('session tracking', () => {
    let provider: ReturnType<typeof createMockProvider>;

    beforeEach(() => {
      provider = createMockProvider();
      initAnalytics({ providers: [provider] });
      setConsent(true);
    });

    it('should generate unique session IDs', () => {
      track('event_a');

      const sessionId = provider.trackCalls[0].sessionId;
      expect(sessionId).toBeDefined();
      expect(sessionId.length).toBeGreaterThan(8);
    });

    it('should maintain session across page navigations', () => {
      track('event_1');
      track('event_2');
      track('event_3');

      const ids = provider.trackCalls.map((e) => e.sessionId);
      expect(ids[0]).toBe(ids[1]);
      expect(ids[1]).toBe(ids[2]);
    });

    it('should start new session after 30 min inactivity', () => {
      track('before_timeout');
      const firstSession = provider.trackCalls[0].sessionId;

      // Advance time by 31 minutes
      vi.advanceTimersByTime(31 * 60 * 1000);

      track('after_timeout');
      const secondSession = provider.trackCalls[1].sessionId;

      expect(firstSession).not.toBe(secondSession);
    });
  });
});
