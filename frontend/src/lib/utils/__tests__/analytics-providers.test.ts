/**
 * Analytics Providers Tests
 * Tests for ConsoleProvider, NoopProvider, and PostHogProvider stub.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

import {
  ConsoleProvider,
  NoopProvider,
  PostHogProvider,
} from '../analytics-providers';

import type { AnalyticsEvent } from '../analytics';

/* ------------------------------------------------------------------ */
/*  Helpers                                                           */
/* ------------------------------------------------------------------ */

function makeEvent(overrides: Partial<AnalyticsEvent> = {}): AnalyticsEvent {
  return {
    name: 'test_event',
    properties: { key: 'value' },
    timestamp: '2026-03-26T12:00:00.000Z',
    sessionId: 'sess-abc',
    anonymousId: 'anon-xyz',
    ...overrides,
  };
}

/* ------------------------------------------------------------------ */
/*  Setup / Teardown                                                  */
/* ------------------------------------------------------------------ */

beforeEach(() => {
  vi.clearAllMocks();
});

afterEach(() => {
  vi.restoreAllMocks();
});

/* ================================================================== */
/*  ConsoleProvider                                                    */
/* ================================================================== */

describe('ConsoleProvider', () => {
  it('should have name "console"', () => {
    const provider = new ConsoleProvider();
    expect(provider.name).toBe('console');
  });

  it('should log to console.group when tracking', () => {
    const groupSpy = vi.spyOn(console, 'group').mockImplementation(() => {});
    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
    const groupEndSpy = vi.spyOn(console, 'groupEnd').mockImplementation(() => {});

    const provider = new ConsoleProvider();
    const event = makeEvent({ name: 'signup', properties: { method: 'email' } });

    provider.track(event);

    expect(groupSpy).toHaveBeenCalled();
    // The group title should include the event name
    const groupTitle = groupSpy.mock.calls[0][0] as string;
    expect(groupTitle).toContain('signup');

    expect(logSpy).toHaveBeenCalled();
    expect(groupEndSpy).toHaveBeenCalled();
  });

  it('should log identify calls', () => {
    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});

    const provider = new ConsoleProvider();
    provider.identify('user-1', { plan: 'pro' });

    expect(logSpy).toHaveBeenCalled();
    const loggedMsg = logSpy.mock.calls.flat().join(' ');
    expect(loggedMsg).toContain('user-1');
  });

  it('should log reset calls', () => {
    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});

    const provider = new ConsoleProvider();
    provider.reset();

    expect(logSpy).toHaveBeenCalled();
  });
});

/* ================================================================== */
/*  NoopProvider                                                       */
/* ================================================================== */

describe('NoopProvider', () => {
  it('should have name "noop"', () => {
    const provider = new NoopProvider();
    expect(provider.name).toBe('noop');
  });

  it('should have callable track that does nothing', () => {
    const provider = new NoopProvider();
    expect(() => provider.track(makeEvent())).not.toThrow();
  });

  it('should have callable identify that does nothing', () => {
    const provider = new NoopProvider();
    expect(() => provider.identify('user-1')).not.toThrow();
  });

  it('should have callable reset that does nothing', () => {
    const provider = new NoopProvider();
    expect(() => provider.reset()).not.toThrow();
  });

  it('should not produce any side effects', () => {
    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

    const provider = new NoopProvider();
    provider.track(makeEvent());
    provider.identify('user-1');
    provider.reset();

    expect(logSpy).not.toHaveBeenCalled();
    expect(warnSpy).not.toHaveBeenCalled();
  });
});

/* ================================================================== */
/*  PostHogProvider (stub)                                             */
/* ================================================================== */

describe('PostHogProvider (stub)', () => {
  it('should have name "posthog"', () => {
    const provider = new PostHogProvider();
    expect(provider.name).toBe('posthog');
  });

  it('should not throw when tracking', () => {
    const provider = new PostHogProvider();
    expect(() => provider.track(makeEvent())).not.toThrow();
  });

  it('should not throw when identifying', () => {
    const provider = new PostHogProvider();
    expect(() => provider.identify('user-1', { plan: 'free' })).not.toThrow();
  });

  it('should not throw when resetting', () => {
    const provider = new PostHogProvider();
    expect(() => provider.reset()).not.toThrow();
  });
});
