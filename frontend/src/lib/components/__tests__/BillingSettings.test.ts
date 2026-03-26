/**
 * BillingSettings Component Tests
 * Tests current plan display, billing period, manage/upgrade buttons, and states
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';

const { mockInitiatePortal } = vi.hoisted(() => ({
  mockInitiatePortal: vi.fn(),
}));

const mockSubscriptionData = {
  plan: 'pro',
  status: 'active',
  currentPeriodStart: '2026-03-15T12:00:00Z',
  currentPeriodEnd: '2026-04-15T12:00:00Z',
  cancelAtPeriodEnd: false,
};

let subscribedValue: any = mockSubscriptionData;
let planValue = 'pro';

vi.mock('../../stores/billing', () => ({
  subscription: {
    subscribe: (fn: (v: any) => void) => {
      fn(subscribedValue);
      return () => {};
    },
  },
  currentPlan: {
    subscribe: (fn: (v: string) => void) => {
      fn(planValue);
      return () => {};
    },
  },
  initiateCheckout: vi.fn(),
  initiatePortal: mockInitiatePortal,
  featureAccess: vi.fn().mockReturnValue(true),
}));

const { mockNavigateTo } = vi.hoisted(() => ({
  mockNavigateTo: vi.fn(),
}));

vi.mock('../../utils/simple-router', () => ({
  navigateTo: mockNavigateTo,
  currentRoute: {
    subscribe: (fn: (v: string) => void) => {
      fn('settings');
      return () => {};
    },
  },
}));

vi.mock('../../stores/auth', () => ({
  currentUser: {
    subscribe: (fn: (v: any) => void) => {
      fn({ id: 'test-user-id', email: 'test@example.com' });
      return () => {};
    },
  },
  isAuthenticated: {
    subscribe: (fn: (v: boolean) => void) => {
      fn(true);
      return () => {};
    },
  },
}));

import BillingSettings from '../BillingSettings.svelte';

describe('BillingSettings Component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    subscribedValue = mockSubscriptionData;
    planValue = 'pro';
  });

  describe('Current Plan Display', () => {
    it('should show current plan name and status', () => {
      render(BillingSettings);

      expect(screen.getByTestId('current-plan-name').textContent).toContain('Pro');
      expect(screen.getByTestId('plan-status-badge').textContent).toContain('Active');
    });
  });

  describe('Billing Period', () => {
    it('should show billing period start and end dates', () => {
      render(BillingSettings);

      const periodEl = screen.getByTestId('billing-period');
      expect(periodEl.textContent).toContain('Mar');
      expect(periodEl.textContent).toContain('Apr');
    });
  });

  describe('Manage Billing Button', () => {
    it('should show Manage Billing button that calls initiatePortal', async () => {
      render(BillingSettings);

      const btn = screen.getByTestId('manage-billing-btn');
      expect(btn).toBeTruthy();
      await fireEvent.click(btn);

      expect(mockInitiatePortal).toHaveBeenCalled();
    });
  });

  describe('Upgrade Button for Free Users', () => {
    it('should show Upgrade button for free users', () => {
      subscribedValue = null;
      planValue = 'free';

      render(BillingSettings);

      const btn = screen.getByTestId('upgrade-btn');
      expect(btn).toBeTruthy();
      expect(btn.textContent).toContain('Upgrade');
    });
  });

  describe('Cancel Status', () => {
    it('should show cancel notice if cancelAtPeriodEnd is true', () => {
      subscribedValue = { ...mockSubscriptionData, cancelAtPeriodEnd: true };

      render(BillingSettings);

      const notice = screen.getByTestId('cancel-notice');
      expect(notice).toBeTruthy();
      expect(notice.textContent).toContain('cancel');
    });
  });

  describe('Loading State', () => {
    it('should handle loading state', () => {
      subscribedValue = undefined;

      render(BillingSettings);

      const loader = screen.getByTestId('billing-loading');
      expect(loader).toBeTruthy();
    });
  });

  describe('Error State', () => {
    it('should handle error state', () => {
      subscribedValue = { error: 'Failed to load billing info' };

      render(BillingSettings);

      const error = screen.getByTestId('billing-error');
      expect(error).toBeTruthy();
      expect(error.textContent).toContain('Failed to load billing info');
    });
  });
});
