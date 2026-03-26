/**
 * Analytics Provider Implementations
 *
 * - ConsoleProvider  — pretty-prints events to the browser console (dev mode)
 * - NoopProvider     — does nothing (safe for tests & SSR)
 * - PostHogProvider  — stub placeholder for PostHog integration (lazy-load)
 */

import type { AnalyticsEvent, AnalyticsProvider } from './analytics';

/* ================================================================== */
/*  ConsoleProvider                                                    */
/* ================================================================== */

export class ConsoleProvider implements AnalyticsProvider {
  name = 'console' as const;

  track(event: AnalyticsEvent): void {
    // eslint-disable-next-line no-console
    console.group(
      `%c[Analytics] ${event.name}`,
      'color: #6366f1; font-weight: bold',
    );
    // eslint-disable-next-line no-console
    console.log('properties:', event.properties);
    // eslint-disable-next-line no-console
    console.log('timestamp :', event.timestamp);
    // eslint-disable-next-line no-console
    console.log('sessionId :', event.sessionId);
    // eslint-disable-next-line no-console
    console.log('anonymousId:', event.anonymousId);
    if (event.userId) {
      // eslint-disable-next-line no-console
      console.log('userId    :', event.userId);
    }
    // eslint-disable-next-line no-console
    console.groupEnd();
  }

  identify(userId: string, traits?: Record<string, unknown>): void {
    // eslint-disable-next-line no-console
    console.log(
      `%c[Analytics] identify: ${userId}`,
      'color: #6366f1; font-weight: bold',
      traits ?? '',
    );
  }

  reset(): void {
    // eslint-disable-next-line no-console
    console.log(
      '%c[Analytics] reset',
      'color: #6366f1; font-weight: bold',
    );
  }
}

/* ================================================================== */
/*  NoopProvider                                                       */
/* ================================================================== */

export class NoopProvider implements AnalyticsProvider {
  name = 'noop' as const;

  track(_event: AnalyticsEvent): void {
    // intentionally empty
  }

  identify(_userId: string, _traits?: Record<string, unknown>): void {
    // intentionally empty
  }

  reset(): void {
    // intentionally empty
  }
}

/* ================================================================== */
/*  PostHogProvider (stub)                                             */
/* ================================================================== */

/**
 * Placeholder for a PostHog integration.
 *
 * When the real PostHog SDK is wired up, this class would lazy-load
 * `posthog-js` on first `track` / `identify` call and forward events.
 * For now it is a harmless no-op so consumers can register it without
 * introducing an external dependency.
 */
export class PostHogProvider implements AnalyticsProvider {
  name = 'posthog' as const;

  track(_event: AnalyticsEvent): void {
    // Stub — will forward to posthog.capture() once SDK is loaded
  }

  identify(_userId: string, _traits?: Record<string, unknown>): void {
    // Stub — will forward to posthog.identify() once SDK is loaded
  }

  reset(): void {
    // Stub — will forward to posthog.reset() once SDK is loaded
  }
}
