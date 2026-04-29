import type { ServiceConnection } from '../stores/connections';

export type CanonicalOAuthState =
  | 'disconnected'
  | 'authorizing'
  | 'callback_pending'
  | 'connected'
  | 'refreshing'
  | 'failed_auth'
  | 'rate_limited'
  | 'failed_provider'
  | 'failed_system';

export interface OAuthStateOptions {
  isAuthorizing?: boolean;
  isCallbackPending?: boolean;
  isRefreshing?: boolean;
  failureHint?: string | null;
}

export interface OAuthStateCopy {
  label: string;
  tone: 'idle' | 'connected' | 'warning' | 'error';
  message: string;
  reconnectCta: boolean;
}

export type OAuthAction = 'connect' | 'disconnect' | 'sync' | 'callback';

const AUTH_FAILURE_HINTS = [
  'failed_auth',
  'needs_reauth',
  'needs reauth',
  're-auth',
  'reauth',
  'invalid_grant',
  'invalid grant',
  'invalid refresh',
  'refresh token',
  'expired',
  'revoked',
  'unauthorized',
  'forbidden',
  'missing required scope',
  'scope',
  'auth failed',
  'authentication failed',
];

const RATE_LIMIT_HINTS = [
  'rate_limited',
  'rate limited',
  'rate-limited',
  'too many requests',
  'throttled',
  'throttle',
  'retry after',
  'retry-after',
  'http_429',
  ' 429',
];

const SYSTEM_FAILURE_HINTS = [
  'failed_system',
  'network error',
  'timed out',
  'timeout',
  'http_500',
  'http_502',
  'http_503',
  'http_504',
  'internal',
  'database',
  'redis',
];

function normalizeHint(value?: string | null): string {
  return (value ?? '').toLowerCase();
}

export function isAlreadyConnectedMessage(message?: string | null): boolean {
  const hint = normalizeHint(message);
  if (!hint) return false;

  return (
    hint.includes('already connected') ||
    hint.includes('already have an active') ||
    hint.includes('already linked') ||
    hint.includes('disconnect first to reconnect')
  );
}

export function classifyOAuthFailure(hint?: string | null): Extract<
  CanonicalOAuthState,
  'failed_auth' | 'rate_limited' | 'failed_provider' | 'failed_system'
> {
  const normalized = normalizeHint(hint);

  if (!normalized) return 'failed_provider';

  if (RATE_LIMIT_HINTS.some((token) => normalized.includes(token))) {
    return 'rate_limited';
  }

  if (AUTH_FAILURE_HINTS.some((token) => normalized.includes(token))) {
    return 'failed_auth';
  }

  if (SYSTEM_FAILURE_HINTS.some((token) => normalized.includes(token))) {
    return 'failed_system';
  }

  if (normalized.includes('failed_provider')) {
    return 'failed_provider';
  }

  return 'failed_provider';
}

function isCanonicalState(value: string | undefined): value is CanonicalOAuthState {
  return (
    value === 'disconnected' ||
    value === 'authorizing' ||
    value === 'callback_pending' ||
    value === 'connected' ||
    value === 'refreshing' ||
    value === 'failed_auth' ||
    value === 'rate_limited' ||
    value === 'failed_provider' ||
    value === 'failed_system'
  );
}

export function deriveCanonicalOAuthState(
  connection: ServiceConnection | null | undefined,
  options: OAuthStateOptions = {}
): CanonicalOAuthState {
  if (options.isAuthorizing) return 'authorizing';
  if (options.isCallbackPending) return 'callback_pending';
  if (options.isRefreshing) return 'refreshing';

  if (!connection) return 'disconnected';

  if (isCanonicalState(connection.oauth_state)) {
    return connection.oauth_state;
  }

  if (connection.status === 'active') {
    return 'connected';
  }

  const failureHint = options.failureHint ?? connection.oauth_error_class ?? connection.error_code;

  if (connection.status === 'expired') {
    return 'failed_auth';
  }

  if (connection.status === 'error') {
    return classifyOAuthFailure(failureHint);
  }

  if (connection.health_status === 'needs_reauth') {
    return 'failed_auth';
  }

  return classifyOAuthFailure(failureHint);
}

export function getOAuthStateCopy(state: CanonicalOAuthState, providerName: string): OAuthStateCopy {
  switch (state) {
    case 'authorizing':
      return {
        label: 'Authorizing',
        tone: 'warning',
        message: `Authorizing ${providerName}. Complete the provider prompt to continue.`,
        reconnectCta: false,
      };
    case 'callback_pending':
      return {
        label: 'Finalizing',
        tone: 'warning',
        message: `Finalizing the ${providerName} callback.`,
        reconnectCta: false,
      };
    case 'connected':
      return {
        label: 'Connected',
        tone: 'connected',
        message: `${providerName} is connected and ready for sync.`,
        reconnectCta: false,
      };
    case 'refreshing':
      return {
        label: 'Refreshing',
        tone: 'warning',
        message: `Refreshing ${providerName} state.`,
        reconnectCta: false,
      };
    case 'failed_auth':
      return {
        label: 'Reconnect Required',
        tone: 'warning',
        message: `${providerName} authorization is no longer valid. Reconnect to continue syncing.`,
        reconnectCta: true,
      };
    case 'rate_limited':
      return {
        label: 'Rate Limited',
        tone: 'warning',
        message: `${providerName} is rate-limiting requests. Wait a few minutes, then retry.`,
        reconnectCta: true,
      };
    case 'failed_system':
      return {
        label: 'System Error',
        tone: 'error',
        message: `A system error interrupted ${providerName}. Retry in a moment.`,
        reconnectCta: true,
      };
    case 'failed_provider':
      return {
        label: 'Provider Error',
        tone: 'error',
        message: `${providerName} returned an unexpected response. Retry shortly.`,
        reconnectCta: true,
      };
    case 'disconnected':
    default:
      return {
        label: 'Not Connected',
        tone: 'idle',
        message: `Connect ${providerName} to sync playlists, favorites, and library metadata.`,
        reconnectCta: false,
      };
  }
}

export function mapOAuthActionError(
  providerName: string,
  action: OAuthAction,
  hint?: string | null
): string {
  if (isAlreadyConnectedMessage(hint) && action === 'connect') {
    return `${providerName} is already connected. Use sync or disconnect first to reconnect.`;
  }

  const state = classifyOAuthFailure(hint);

  if (action === 'disconnect') {
    switch (state) {
      case 'failed_system':
        return `A system error prevented disconnecting ${providerName}. Try again.`;
      case 'rate_limited':
      case 'failed_provider':
      case 'failed_auth':
      default:
        return `Could not disconnect ${providerName}. Try again in a moment.`;
    }
  }

  if (action === 'sync') {
    switch (state) {
      case 'failed_auth':
        return `${providerName} authorization is invalid. Reconnect before syncing again.`;
      case 'rate_limited':
        return `${providerName} is rate-limiting requests. Wait a few minutes and retry sync.`;
      case 'failed_system':
        return `A system error interrupted ${providerName} sync. Retry shortly.`;
      case 'failed_provider':
      default:
        return `${providerName} sync failed on the provider side. Retry shortly.`;
    }
  }

  if (action === 'callback') {
    switch (state) {
      case 'failed_auth':
        return `${providerName} rejected the callback. Start connection again.`;
      case 'rate_limited':
        return `${providerName} temporarily rate-limited the callback. Retry shortly.`;
      case 'failed_system':
        return `A system error interrupted the ${providerName} callback. Retry.`;
      case 'failed_provider':
      default:
        return `${providerName} callback failed. Retry connection.`;
    }
  }

  switch (state) {
    case 'failed_auth':
      return `${providerName} authorization failed. Reconnect to continue.`;
    case 'rate_limited':
      return `${providerName} is rate-limiting requests. Wait a few minutes and retry.`;
    case 'failed_system':
      return `A system error interrupted ${providerName} connection. Retry in a moment.`;
    case 'failed_provider':
    default:
      return `${providerName} returned an error while connecting. Retry shortly.`;
  }
}
