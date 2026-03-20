import {
  type Auth0Client,
  type RedirectLoginOptions,
  createAuth0Client,
} from '@auth0/auth0-spa-js';
import config from '../utils/config';
import { setConvexAuthToken } from '../convex/client';

let auth0ClientPromise: Promise<Auth0Client> | null = null;
let redirectHandled = false;

function isBrowser(): boolean {
  return typeof window !== 'undefined';
}

export function isAuth0Mode(): boolean {
  return config.auth.mode === 'auth0';
}

function shouldHandleAuth0Redirect(): boolean {
  if (!isBrowser() || redirectHandled) {
    return false;
  }

  if (window.location.pathname.startsWith('/auth/callback')) {
    return false;
  }

  const params = new URLSearchParams(window.location.search);
  return params.has('code') && params.has('state');
}

function buildAuthorizationParams() {
  const params: Record<string, string> = {
    redirect_uri: `${window.location.origin}${config.auth.auth0.redirectPath}`,
  };

  if (config.auth.auth0.audience) {
    params.audience = config.auth.auth0.audience;
  }

  if (config.auth.auth0.scope) {
    params.scope = config.auth.auth0.scope;
  }

  return params;
}

export async function getAuth0Client(): Promise<Auth0Client> {
  if (!isAuth0Mode()) {
    throw new Error('Auth0 mode is not enabled.');
  }

  if (!isBrowser()) {
    throw new Error('Auth0 client is only available in the browser.');
  }

  if (!auth0ClientPromise) {
    auth0ClientPromise = createAuth0Client({
      domain: config.auth.auth0.domain,
      clientId: config.auth.auth0.clientId,
      authorizationParams: buildAuthorizationParams(),
      useRefreshTokens: true,
      cacheLocation: 'localstorage',
    });
  }

  return await auth0ClientPromise!;
}

export async function syncAuthToken(): Promise<string | null> {
  if (!isAuth0Mode()) {
    return localStorage.getItem('auth_token');
  }

  const client = await getAuth0Client();
  const authenticated = await client.isAuthenticated();

  if (!authenticated) {
    localStorage.removeItem('auth_token');
    localStorage.removeItem('refresh_token');
    setConvexAuthToken(null);
    return null;
  }

  const claims = await client.getIdTokenClaims();
  const idToken = claims?.__raw ?? null;

  if (idToken) {
    localStorage.setItem('auth_token', idToken);
    localStorage.removeItem('refresh_token');
    setConvexAuthToken(idToken);
  } else {
    localStorage.removeItem('auth_token');
    setConvexAuthToken(null);
  }

  return idToken;
}

export async function initializeAuthSession(): Promise<boolean> {
  if (!isAuth0Mode() || !isBrowser()) {
    return Boolean(localStorage.getItem('auth_token'));
  }

  const client = await getAuth0Client();

  if (shouldHandleAuth0Redirect()) {
    const { appState } = await client.handleRedirectCallback();
    redirectHandled = true;

    const returnTo =
      typeof appState?.returnTo === 'string' && appState.returnTo
        ? appState.returnTo
        : '/';

    window.history.replaceState({}, document.title, returnTo);
  }

  const token = await syncAuthToken();
  return Boolean(token);
}

export async function refreshAuthSession(): Promise<boolean> {
  if (!isAuth0Mode()) {
    return Boolean(localStorage.getItem('auth_token'));
  }

  try {
    const client = await getAuth0Client();
    const authenticated = await client.isAuthenticated();
    if (!authenticated) {
      await syncAuthToken();
      return false;
    }

    await client.getTokenSilently();
    const token = await syncAuthToken();
    return Boolean(token);
  } catch (error) {
    console.error('Failed to refresh Auth0 session:', error);
    await clearAuthSession();
    return false;
  }
}

export async function loginWithAuth0(options?: {
  mode?: 'login' | 'register';
  provider?: string;
  email?: string;
  returnTo?: string;
}): Promise<void> {
  const client = await getAuth0Client();

  const authorizationParams: RedirectLoginOptions['authorizationParams'] = {
    ...buildAuthorizationParams(),
  };

  if (options?.mode === 'register') {
    authorizationParams.screen_hint = 'signup';
  }

  if (options?.email) {
    authorizationParams.login_hint = options.email;
  }

  const connection = resolveProviderConnection(options?.provider);
  if (connection) {
    authorizationParams.connection = connection;
  }

  await client.loginWithRedirect({
    authorizationParams,
    appState: {
      returnTo: options?.returnTo ?? window.location.pathname,
    },
  });
}

function resolveProviderConnection(provider?: string): string | undefined {
  switch (provider) {
    case 'google':
      return config.auth.auth0.connections.google;
    case 'github':
      return config.auth.auth0.connections.github;
    case 'apple':
      return config.auth.auth0.connections.apple;
    default:
      return undefined;
  }
}

export async function getAuth0User() {
  const client = await getAuth0Client();
  return await client.getUser();
}

export async function clearAuthSession(): Promise<void> {
  localStorage.removeItem('auth_token');
  localStorage.removeItem('refresh_token');
  setConvexAuthToken(null);
}

export async function logoutFromAuth0(returnTo?: string): Promise<void> {
  if (!isAuth0Mode()) {
    return;
  }

  const client = await getAuth0Client();
  await clearAuthSession();
  await client.logout({
    logoutParams: {
      returnTo: returnTo ?? `${window.location.origin}${config.auth.auth0.redirectPath}`,
    },
  });
}
