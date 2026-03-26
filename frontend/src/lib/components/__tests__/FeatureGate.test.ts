/**
 * FeatureGate Component Tests
 * Tests rendering children when accessible, showing UpgradePrompt when gated,
 * loading skeleton, and different feature handling
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/svelte';

const { mockFeatureAccess } = vi.hoisted(() => ({
  mockFeatureAccess: vi.fn().mockReturnValue(true),
}));

vi.mock('../../stores/billing', () => ({
  subscription: {
    subscribe: (fn: (v: any) => void) => {
      fn(null);
      return () => {};
    },
  },
  currentPlan: {
    subscribe: (fn: (v: string) => void) => {
      fn('free');
      return () => {};
    },
  },
  initiateCheckout: vi.fn(),
  initiatePortal: vi.fn(),
  featureAccess: mockFeatureAccess,
}));

vi.mock('../../utils/simple-router', () => ({
  navigateTo: vi.fn(),
  currentRoute: {
    subscribe: (fn: (v: string) => void) => {
      fn('home');
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

import FeatureGateTestWrapper from './FeatureGateTestWrapper.svelte';

describe('FeatureGate Component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockFeatureAccess.mockReturnValue(true);
  });

  describe('Accessible Feature', () => {
    it('should render children when feature is accessible', () => {
      mockFeatureAccess.mockReturnValue(true);

      render(FeatureGateTestWrapper, {
        props: { feature: 'connections', accessible: true },
      });

      expect(screen.getByTestId('gated-content')).toBeTruthy();
    });
  });

  describe('Gated Feature', () => {
    it('should render UpgradePrompt when feature is gated', () => {
      mockFeatureAccess.mockReturnValue(false);

      render(FeatureGateTestWrapper, {
        props: { feature: 'connections', accessible: false },
      });

      expect(screen.getByTestId('upgrade-prompt')).toBeTruthy();
    });
  });

  describe('Loading State', () => {
    it('should show loading skeleton during check', () => {
      render(FeatureGateTestWrapper, {
        props: { feature: 'connections', loading: true },
      });

      expect(screen.getByTestId('feature-gate-loading')).toBeTruthy();
    });
  });

  describe('Different Features', () => {
    it('should handle connections feature', () => {
      mockFeatureAccess.mockReturnValue(false);

      render(FeatureGateTestWrapper, {
        props: { feature: 'connections', accessible: false },
      });

      // The UpgradePrompt should be shown with the connections feature
      expect(screen.getByTestId('upgrade-prompt')).toBeTruthy();
      expect(screen.getByTestId('upgrade-prompt').textContent).toContain('connections');
    });

    it('should handle scans feature', () => {
      mockFeatureAccess.mockReturnValue(false);

      render(FeatureGateTestWrapper, {
        props: { feature: 'scans', accessible: false },
      });

      expect(screen.getByTestId('upgrade-prompt')).toBeTruthy();
      expect(screen.getByTestId('upgrade-prompt').textContent).toContain('scan');
    });

    it('should handle enforcement feature', () => {
      mockFeatureAccess.mockReturnValue(true);

      render(FeatureGateTestWrapper, {
        props: { feature: 'enforcement', accessible: true },
      });

      expect(screen.getByTestId('gated-content')).toBeTruthy();
    });

    it('should handle export feature', () => {
      mockFeatureAccess.mockReturnValue(true);

      render(FeatureGateTestWrapper, {
        props: { feature: 'export', accessible: true },
      });

      expect(screen.getByTestId('gated-content')).toBeTruthy();
    });
  });
});
