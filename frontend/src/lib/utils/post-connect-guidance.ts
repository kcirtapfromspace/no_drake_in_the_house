import { config } from './config';

const SPOTIFY_POST_CONNECT_GUIDANCE_STORAGE_KEY =
  'spotify_post_connect_guidance_canary_v1';
const SPOTIFY_POST_CONNECT_GUIDANCE_ANON_SEED_KEY =
  'spotify_post_connect_guidance_anon_seed_v1';
let inMemoryGuidanceStateRaw: string | null = null;
let inMemoryAnonSeed: string | null = null;

export type SpotifyGuidanceSource = 'oauth_callback' | 'popup';

export interface SpotifyPostConnectGuidanceState {
  provider: 'spotify';
  connectedAt: string;
  source: SpotifyGuidanceSource;
  shownAt?: string;
  firstSyncStartedAt?: string;
  completedAt?: string;
  dismissedAt?: string;
}

export interface SpotifyGuidanceCanaryDecision {
  enabled: boolean;
  rolloutPercent: number;
  bucket: number;
  reason:
    | 'disabled'
    | 'no_rollout'
    | 'out_of_cohort'
    | 'eligible'
    | 'invalid_state';
}

function getStorage(): Storage | null {
  if (typeof window === 'undefined') return null;
  return window.localStorage;
}

function parseState(
  raw: string | null
): SpotifyPostConnectGuidanceState | null {
  if (!raw) return null;

  try {
    const parsed = JSON.parse(raw) as Partial<SpotifyPostConnectGuidanceState>;
    if (parsed.provider !== 'spotify' || typeof parsed.connectedAt !== 'string') {
      return null;
    }
    return {
      provider: 'spotify',
      connectedAt: parsed.connectedAt,
      source:
        parsed.source === 'oauth_callback' || parsed.source === 'popup'
          ? parsed.source
          : 'oauth_callback',
      ...(parsed.shownAt ? { shownAt: parsed.shownAt } : {}),
      ...(parsed.firstSyncStartedAt
        ? { firstSyncStartedAt: parsed.firstSyncStartedAt }
        : {}),
      ...(parsed.completedAt ? { completedAt: parsed.completedAt } : {}),
      ...(parsed.dismissedAt ? { dismissedAt: parsed.dismissedAt } : {}),
    };
  } catch {
    return null;
  }
}

function writeState(state: SpotifyPostConnectGuidanceState): void {
  const serialized = JSON.stringify(state);
  inMemoryGuidanceStateRaw = serialized;

  const storage = getStorage();
  if (!storage) return;
  try {
    storage.setItem(SPOTIFY_POST_CONNECT_GUIDANCE_STORAGE_KEY, serialized);
  } catch {
    // Ignore storage write issues and rely on in-memory fallback.
  }
}

function getAnonymousSeed(storage: Storage | null): string {
  if (inMemoryAnonSeed) {
    return inMemoryAnonSeed;
  }

  if (storage) {
    try {
      const existing = storage.getItem(SPOTIFY_POST_CONNECT_GUIDANCE_ANON_SEED_KEY);
      if (typeof existing === 'string' && existing.length > 0) {
        inMemoryAnonSeed = existing;
        return existing;
      }
    } catch {
      // Ignore storage read issues and use generated fallback.
    }
  }

  const generated = `${Date.now()}-${Math.floor(Math.random() * 1_000_000)}`;
  inMemoryAnonSeed = generated;

  if (storage) {
    try {
      storage.setItem(SPOTIFY_POST_CONNECT_GUIDANCE_ANON_SEED_KEY, generated);
    } catch {
      // Ignore storage write issues and rely on in-memory fallback.
    }
  }

  return generated;
}

function hashToPercentBucket(seed: string): number {
  let hash = 0;
  for (let i = 0; i < seed.length; i += 1) {
    hash = (hash * 31 + seed.charCodeAt(i)) >>> 0;
  }
  return hash % 100;
}

function resolveBucket(userId?: string | null): number {
  const normalizedUserId = (userId ?? '').trim();
  if (normalizedUserId) {
    return hashToPercentBucket(`user:${normalizedUserId}`);
  }

  return hashToPercentBucket(`anon:${getAnonymousSeed(getStorage())}`);
}

export function getSpotifyPostConnectGuidanceState(): SpotifyPostConnectGuidanceState | null {
  const storage = getStorage();
  if (storage) {
    try {
      const raw = storage.getItem(SPOTIFY_POST_CONNECT_GUIDANCE_STORAGE_KEY);
      if (typeof raw === 'string' && raw.length > 0) {
        return parseState(raw);
      }
    } catch {
      // Ignore storage read issues and use in-memory fallback.
    }
  }

  return parseState(inMemoryGuidanceStateRaw);
}

export function markSpotifyPostConnect(
  source: SpotifyGuidanceSource
): SpotifyPostConnectGuidanceState {
  const state: SpotifyPostConnectGuidanceState = {
    provider: 'spotify',
    connectedAt: new Date().toISOString(),
    source,
  };
  writeState(state);
  return state;
}

export function markSpotifyGuidanceShown(): SpotifyPostConnectGuidanceState | null {
  const current = getSpotifyPostConnectGuidanceState();
  if (!current || current.shownAt) return current;

  const nextState: SpotifyPostConnectGuidanceState = {
    ...current,
    shownAt: new Date().toISOString(),
  };
  writeState(nextState);
  return nextState;
}

export function markSpotifyFirstSyncStarted(): SpotifyPostConnectGuidanceState | null {
  const current = getSpotifyPostConnectGuidanceState();
  if (!current || current.firstSyncStartedAt) return current;

  const nextState: SpotifyPostConnectGuidanceState = {
    ...current,
    firstSyncStartedAt: new Date().toISOString(),
  };
  writeState(nextState);
  return nextState;
}

export function markSpotifyGuidanceCompleted(): SpotifyPostConnectGuidanceState | null {
  const current = getSpotifyPostConnectGuidanceState();
  if (!current || current.completedAt) return current;

  const nextState: SpotifyPostConnectGuidanceState = {
    ...current,
    completedAt: new Date().toISOString(),
  };
  writeState(nextState);
  return nextState;
}

export function markSpotifyGuidanceDismissed(): SpotifyPostConnectGuidanceState | null {
  const current = getSpotifyPostConnectGuidanceState();
  if (!current || current.dismissedAt) return current;

  const nextState: SpotifyPostConnectGuidanceState = {
    ...current,
    dismissedAt: new Date().toISOString(),
  };
  writeState(nextState);
  return nextState;
}

export function clearSpotifyPostConnectGuidance(): void {
  inMemoryGuidanceStateRaw = null;

  const storage = getStorage();
  if (!storage) return;
  try {
    storage.removeItem(SPOTIFY_POST_CONNECT_GUIDANCE_STORAGE_KEY);
  } catch {
    // Ignore storage cleanup issues.
  }
}

export function getSpotifyGuidanceCanaryDecision(
  userId?: string | null
): SpotifyGuidanceCanaryDecision {
  const rolloutPercent = Math.max(
    0,
    Math.min(100, config.features.postConnectSpotifyGuidanceRolloutPercent)
  );
  const bucket = resolveBucket(userId);
  const state = getSpotifyPostConnectGuidanceState();

  if (!state) {
    return {
      enabled: false,
      rolloutPercent,
      bucket,
      reason: 'invalid_state',
    };
  }

  if (!config.features.postConnectSpotifyGuidanceCanary) {
    return {
      enabled: false,
      rolloutPercent,
      bucket,
      reason: 'disabled',
    };
  }

  if (rolloutPercent <= 0) {
    return {
      enabled: false,
      rolloutPercent,
      bucket,
      reason: 'no_rollout',
    };
  }

  if (bucket >= rolloutPercent) {
    return {
      enabled: false,
      rolloutPercent,
      bucket,
      reason: 'out_of_cohort',
    };
  }

  return {
    enabled: true,
    rolloutPercent,
    bucket,
    reason: 'eligible',
  };
}
