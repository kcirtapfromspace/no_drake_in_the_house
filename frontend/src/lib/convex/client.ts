import { ConvexHttpClient } from 'convex/browser';
import { anyApi } from 'convex/server';
import config from '../utils/config';

let convexClient: ConvexHttpClient | null = null;

export function isConvexEnabled(): boolean {
  return Boolean(config.convex.url);
}

export function getConvexClient(): ConvexHttpClient {
  if (!config.convex.url) {
    throw new Error('VITE_CONVEX_URL is not configured.');
  }

  if (!convexClient) {
    convexClient = new ConvexHttpClient(config.convex.url);
  }

  return convexClient;
}

export function setConvexAuthToken(token: string | null | undefined): void {
  if (!isConvexEnabled()) {
    return;
  }

  const client = getConvexClient();
  if (token) {
    client.setAuth(token);
  } else {
    client.clearAuth();
  }
}

export async function convexQuery<T>(funcRef: any, args?: Record<string, unknown>): Promise<T> {
  return await getConvexClient().query(funcRef, args ?? {});
}

export async function convexMutation<T>(funcRef: any, args?: Record<string, unknown>): Promise<T> {
  return await getConvexClient().mutation(funcRef, args ?? {});
}

export { anyApi };
