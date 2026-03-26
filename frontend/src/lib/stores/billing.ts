import { writable, derived } from 'svelte/store';
import { convexQuery, convexAction, anyApi, isConvexEnabled, hasConvexAuth } from '../convex/client';

export type PlanType = 'free' | 'pro' | 'team';

export interface Subscription {
  plan: PlanType;
  status: string;
  currentPeriodStart: string;
  currentPeriodEnd: string;
  cancelAtPeriodEnd?: boolean;
  seats?: number;
  isActive: boolean;
  activePlan: PlanType;
}

export interface FeatureAccess {
  allowed: boolean;
  reason?: string;
  currentUsage?: number;
  limit?: number;
}

export interface BillingState {
  subscription: Subscription | null;
  isLoading: boolean;
  error: string | null;
  featureCache: Map<string, FeatureAccess>;
}

const initialState: BillingState = {
  subscription: null,
  isLoading: false,
  error: null,
  featureCache: new Map(),
};

export const billingStore = writable<BillingState>(initialState);

export const currentPlan = derived(billingStore, ($b) =>
  $b.subscription?.activePlan ?? 'free',
);

export const isPro = derived(currentPlan, ($plan) => $plan === 'pro' || $plan === 'team');

export const isFreePlan = derived(currentPlan, ($plan) => $plan === 'free');

export const billingActions = {
  fetchSubscription: async () => {
    if (!isConvexEnabled() || !hasConvexAuth()) return;

    billingStore.update((s) => ({ ...s, isLoading: true, error: null }));

    try {
      const sub = await convexQuery<Subscription | null>(
        anyApi.stripe.getSubscription,
      );
      billingStore.update((s) => ({
        ...s,
        subscription: sub,
        isLoading: false,
      }));
    } catch (error) {
      billingStore.update((s) => ({
        ...s,
        isLoading: false,
        error: error instanceof Error ? error.message : 'Failed to load subscription',
      }));
    }
  },

  checkFeature: async (feature: string): Promise<FeatureAccess> => {
    if (!isConvexEnabled() || !hasConvexAuth()) {
      return { allowed: true };
    }

    let cached: FeatureAccess | undefined;
    billingStore.subscribe((s) => {
      cached = s.featureCache.get(feature);
    })();

    if (cached) return cached;

    try {
      const access = await convexQuery<FeatureAccess>(
        anyApi.stripe.getFeatureAccess,
        { feature },
      );
      billingStore.update((s) => {
        const newCache = new Map(s.featureCache);
        newCache.set(feature, access);
        return { ...s, featureCache: newCache };
      });
      return access;
    } catch {
      return { allowed: true };
    }
  },

  initiateCheckout: async (plan: 'pro' | 'team') => {
    if (!isConvexEnabled() || !hasConvexAuth()) return;

    try {
      const result = await convexAction<{ sessionUrl: string }>(
        anyApi.billing.initiateCheckout,
        {
          plan,
          successUrl: window.location.origin + '/#settings?checkout=success',
          cancelUrl: window.location.origin + '/#settings?checkout=cancel',
        },
      );
      window.location.href = result.sessionUrl;
    } catch (error) {
      billingStore.update((s) => ({
        ...s,
        error: error instanceof Error ? error.message : 'Failed to start checkout',
      }));
    }
  },

  openPortal: async () => {
    if (!isConvexEnabled() || !hasConvexAuth()) return;

    try {
      const result = await convexAction<{ portalUrl: string }>(
        anyApi.billing.initiatePortal,
        {
          returnUrl: window.location.origin + '/#settings',
        },
      );
      window.location.href = result.portalUrl;
    } catch (error) {
      billingStore.update((s) => ({
        ...s,
        error: error instanceof Error ? error.message : 'Failed to open billing portal',
      }));
    }
  },

  clearFeatureCache: () => {
    billingStore.update((s) => ({
      ...s,
      featureCache: new Map(),
    }));
  },
};
