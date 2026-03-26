/**
 * Analytics Tracking — Svelte Store Integration
 *
 * Bridges the core analytics module with Svelte's reactive stores.
 * - `analyticsReady`   writable boolean — true once initAnalytics() completes
 * - `consentGiven`     writable boolean — persisted to localStorage
 * - `initAnalyticsFromStore()` — reads auth store, inits analytics, identifies user
 * - Auto-tracks page views on route changes
 */

import { writable, get } from 'svelte/store';
import {
  initAnalytics,
  identify,
  reset as resetAnalytics,
  setConsent as setCoreConsent,
  trackPageView,
} from '../utils/analytics';
import { ConsoleProvider, NoopProvider } from '../utils/analytics-providers';
import { currentUser, isAuthenticated } from './auth';
import { currentRoute, currentRouteMeta } from '../utils/simple-router';

import type { AnalyticsConfig } from '../utils/analytics';

/* ------------------------------------------------------------------ */
/*  Stores                                                            */
/* ------------------------------------------------------------------ */

/** Whether the analytics module has been initialized. */
export const analyticsReady = writable<boolean>(false);

/** Whether the user has granted analytics consent. Persisted to localStorage. */
export const consentGiven = writable<boolean>(readStoredConsent());

/* ------------------------------------------------------------------ */
/*  Helpers                                                           */
/* ------------------------------------------------------------------ */

const CONSENT_KEY = 'ndith_analytics_consent';

function readStoredConsent(): boolean {
  try {
    return localStorage.getItem(CONSENT_KEY) === 'true';
  } catch {
    return false;
  }
}

function isDev(): boolean {
  try {
    return import.meta.env?.DEV === true;
  } catch {
    return false;
  }
}

/* ------------------------------------------------------------------ */
/*  Initialization                                                    */
/* ------------------------------------------------------------------ */

let routeUnsub: (() => void) | null = null;

/**
 * Initialize the analytics system using current auth state.
 *
 * Call this once from your root component (e.g. `onMount` in App.svelte)
 * after auth has been bootstrapped.
 */
export function initAnalyticsFromStore(
  overrides: Partial<AnalyticsConfig> = {},
): void {
  const providers = overrides.providers ?? (isDev()
    ? [new ConsoleProvider()]
    : [new NoopProvider()]);

  initAnalytics({
    enabled: true,
    debug: isDev(),
    providers,
    ...overrides,
  });

  // Sync consent from store into core module
  const consent = get(consentGiven);
  setCoreConsent(consent);

  // Identify user if already authenticated
  const user = get(currentUser);
  if (user && get(isAuthenticated)) {
    identify(user.id);
  }

  analyticsReady.set(true);

  // Auto-track page views on route changes
  if (!routeUnsub) {
    routeUnsub = currentRoute.subscribe((_route) => {
      const meta = get(currentRouteMeta);
      if (meta) {
        trackPageView(
          typeof window !== 'undefined' ? window.location.pathname : '/',
          meta.title,
        );
      }
    });
  }
}

/* ------------------------------------------------------------------ */
/*  Consent                                                           */
/* ------------------------------------------------------------------ */

/**
 * Update consent and propagate to both the Svelte store and the core module.
 */
export function setConsent(granted: boolean): void {
  consentGiven.set(granted);
  setCoreConsent(granted);
}

/* ------------------------------------------------------------------ */
/*  Auth reactivity                                                   */
/* ------------------------------------------------------------------ */

// Subscribe to auth changes — identify / reset automatically.
if (typeof window !== 'undefined') {
  let prevUserId: string | null = null;

  currentUser.subscribe((user) => {
    if (user && user.id !== prevUserId) {
      identify(user.id);
      prevUserId = user.id;
    } else if (!user && prevUserId) {
      resetAnalytics();
      prevUserId = null;
    }
  });
}
