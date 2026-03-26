import posthog from 'posthog-js';
import { config } from './config';

let initialized = false;

export function initPostHog(): void {
  const apiKey = config.posthog.apiKey;
  const apiHost = config.posthog.apiHost;

  if (!apiKey) {
    if (config.isDevelopment()) {
      console.warn('[PostHog] No API key configured — skipping initialization');
    }
    return;
  }

  posthog.init(apiKey, {
    api_host: apiHost,
    capture_pageview: false, // we handle this manually on route changes
    capture_pageleave: true,
    persistence: 'localStorage+cookie',
    autocapture: true,
    disable_session_recording: config.isDevelopment(),
  });

  initialized = true;
}

export function identifyUser(userId: string, properties?: Record<string, unknown>): void {
  if (!initialized) return;
  posthog.identify(userId, properties);
}

export function resetUser(): void {
  if (!initialized) return;
  posthog.reset();
}

export function capturePageView(route: string, properties?: Record<string, unknown>): void {
  if (!initialized) return;
  posthog.capture('$pageview', {
    $current_url: window.location.href,
    route,
    ...properties,
  });
}

export function captureEvent(event: string, properties?: Record<string, unknown>): void {
  if (!initialized) return;
  posthog.capture(event, properties);
}

export { posthog };
