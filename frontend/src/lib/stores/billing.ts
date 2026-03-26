import { writable, derived } from 'svelte/store';
import { apiClient } from '../utils/api-client';

export interface Subscription {
  plan: 'free' | 'pro' | 'team';
  status: 'active' | 'past_due' | 'canceled' | 'trialing';
  currentPeriodStart: string;
  currentPeriodEnd: string;
  cancelAtPeriodEnd: boolean;
  error?: string;
}

// undefined = loading, null = free/no subscription, Subscription = active plan
export const subscription = writable<Subscription | null | undefined>(undefined);

export const currentPlan = derived(subscription, ($sub) => {
  if (!$sub || $sub.error) return 'free';
  return $sub.plan || 'free';
});

// Feature entitlements per plan
const planFeatures: Record<string, string[]> = {
  free: ['connections:1', 'scans:manual', 'blocklist:basic'],
  pro: ['connections:3', 'scans:weekly', 'enforcement:auto', 'export:csv', 'blocklist:full', 'analytics:basic'],
  team: ['connections:unlimited', 'scans:realtime', 'enforcement:realtime', 'export:all', 'blocklist:full', 'analytics:full', 'team:manage'],
};

export function featureAccess(feature: string): boolean {
  let plan = 'free';
  subscription.subscribe((sub) => {
    if (sub && !sub.error) {
      plan = sub.plan || 'free';
    }
  })();

  const features = planFeatures[plan] || planFeatures.free;

  // Check if any feature entry starts with the feature name
  return features.some((f) => f.startsWith(feature));
}

export async function initiateCheckout(plan: string): Promise<void> {
  try {
    const result = await apiClient.authenticatedRequest<{ checkout_url: string }>(
      'POST',
      '/api/v1/billing/checkout',
      { plan },
    );

    if (result.success && result.data?.checkout_url) {
      window.location.href = result.data.checkout_url;
    }
  } catch (error) {
    console.error('Failed to initiate checkout:', error);
  }
}

export async function initiatePortal(): Promise<void> {
  try {
    const result = await apiClient.authenticatedRequest<{ portal_url: string }>(
      'POST',
      '/api/v1/billing/portal',
    );

    if (result.success && result.data?.portal_url) {
      window.location.href = result.data.portal_url;
    }
  } catch (error) {
    console.error('Failed to open billing portal:', error);
  }
}

export async function fetchSubscription(): Promise<void> {
  try {
    const result = await apiClient.authenticatedRequest<Subscription>(
      'GET',
      '/api/v1/billing/subscription',
    );

    if (result.success && result.data) {
      subscription.set(result.data);
    } else {
      // No subscription = free plan
      subscription.set(null);
    }
  } catch (error) {
    console.error('Failed to fetch subscription:', error);
    subscription.set(null);
  }
}
