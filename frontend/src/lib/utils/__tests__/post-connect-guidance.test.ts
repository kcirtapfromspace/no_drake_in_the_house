import { beforeEach, describe, expect, it } from 'vitest';
import { config } from '../config';
import {
  clearSpotifyPostConnectGuidance,
  getSpotifyGuidanceCanaryDecision,
  getSpotifyPostConnectGuidanceState,
  markSpotifyFirstSyncStarted,
  markSpotifyGuidanceCompleted,
  markSpotifyGuidanceDismissed,
  markSpotifyGuidanceShown,
  markSpotifyPostConnect,
} from '../post-connect-guidance';

const originalCanaryEnabled = config.features.postConnectSpotifyGuidanceCanary;
const originalRolloutPercent = config.features.postConnectSpotifyGuidanceRolloutPercent;

describe('post-connect guidance canary helpers', () => {
  beforeEach(() => {
    clearSpotifyPostConnectGuidance();
    localStorage.removeItem('spotify_post_connect_guidance_anon_seed_v1');
    config.features.postConnectSpotifyGuidanceCanary = true;
    config.features.postConnectSpotifyGuidanceRolloutPercent = 100;
  });

  it('stores and reads guidance state transitions', () => {
    markSpotifyPostConnect('oauth_callback');
    expect(getSpotifyPostConnectGuidanceState()?.provider).toBe('spotify');

    markSpotifyGuidanceShown();
    expect(getSpotifyPostConnectGuidanceState()?.shownAt).toBeDefined();

    markSpotifyFirstSyncStarted();
    expect(getSpotifyPostConnectGuidanceState()?.firstSyncStartedAt).toBeDefined();

    markSpotifyGuidanceCompleted();
    expect(getSpotifyPostConnectGuidanceState()?.completedAt).toBeDefined();

    markSpotifyGuidanceDismissed();
    expect(getSpotifyPostConnectGuidanceState()?.dismissedAt).toBeDefined();
  });

  it('returns disabled decision when canary flag is off', () => {
    markSpotifyPostConnect('popup');
    config.features.postConnectSpotifyGuidanceCanary = false;

    const decision = getSpotifyGuidanceCanaryDecision('user-123');
    expect(decision.enabled).toBe(false);
    expect(decision.reason).toBe('disabled');
  });

  it('returns eligible decision for in-cohort users', () => {
    markSpotifyPostConnect('popup');
    config.features.postConnectSpotifyGuidanceRolloutPercent = 100;

    const decision = getSpotifyGuidanceCanaryDecision('user-123');
    expect(decision.enabled).toBe(true);
    expect(decision.reason).toBe('eligible');
    expect(decision.rolloutPercent).toBe(100);
  });

  it('returns stable bucket decisions for anonymous users', () => {
    markSpotifyPostConnect('popup');
    config.features.postConnectSpotifyGuidanceRolloutPercent = 30;

    const first = getSpotifyGuidanceCanaryDecision();
    const second = getSpotifyGuidanceCanaryDecision();

    expect(first.bucket).toBe(second.bucket);
    expect(second.reason === 'eligible' || second.reason === 'out_of_cohort').toBe(
      true
    );
  });

  it('returns invalid_state when no marker exists', () => {
    const decision = getSpotifyGuidanceCanaryDecision('user-123');
    expect(decision.enabled).toBe(false);
    expect(decision.reason).toBe('invalid_state');
  });
});

afterAll(() => {
  config.features.postConnectSpotifyGuidanceCanary = originalCanaryEnabled;
  config.features.postConnectSpotifyGuidanceRolloutPercent = originalRolloutPercent;
  clearSpotifyPostConnectGuidance();
});
