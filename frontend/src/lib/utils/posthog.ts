import posthog from 'posthog-js';
import { config } from './config';

let initialized = false;
const POSTHOG_PROXY_HOST = 'https://t.nodrakeinthe.house';
const POSTHOG_DIRECT_HOST_PATTERN = /(^|\.)posthog\.com$/i;

const resolveSafePostHogHost = (host: string): string => {
  const normalizedHost = host.trim();

  if (!normalizedHost) {
    return config.isProduction() ? POSTHOG_PROXY_HOST : '';
  }

  if (!config.isProduction()) {
    return normalizedHost;
  }

  try {
    const parsed = new URL(normalizedHost);
    if (POSTHOG_DIRECT_HOST_PATTERN.test(parsed.hostname)) {
      console.warn('[PostHog] Blocking direct posthog.com browser ingestion in production; using managed proxy host');
      return POSTHOG_PROXY_HOST;
    }
    return normalizedHost;
  } catch {
    console.warn('[PostHog] Invalid PostHog host in production; using managed proxy host');
    return POSTHOG_PROXY_HOST;
  }
};

export function initPostHog(): void {
  const apiKey = config.posthog.apiKey;
  const apiHost = resolveSafePostHogHost(config.posthog.apiHost);

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
