/**
 * UpgradePrompt Component Tests
 * Tests gate message rendering, CTA button, dismiss, and feature-specific messages
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';

const { mockNavigateTo } = vi.hoisted(() => ({
  mockNavigateTo: vi.fn(),
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
  featureAccess: vi.fn().mockReturnValue(false),
}));

vi.mock('../../utils/simple-router', () => ({
  navigateTo: mockNavigateTo,
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

import UpgradePrompt from '../UpgradePrompt.svelte';

describe('UpgradePrompt Component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Gate Message', () => {
    it('should render gate message for the feature', () => {
      render(UpgradePrompt, { props: { feature: 'connections' } });

      const prompt = screen.getByTestId('upgrade-prompt');
      expect(prompt).toBeTruthy();
      expect(prompt.textContent).toContain('connections');
    });
  });

  describe('Upgrade CTA Button', () => {
    it('should show upgrade CTA button', () => {
      render(UpgradePrompt, { props: { feature: 'connections' } });

      const btn = screen.getByTestId('upgrade-cta-btn');
      expect(btn).toBeTruthy();
      expect(btn.textContent).toContain('Upgrade');
    });
  });

  describe('Current Plan Info', () => {
    it('should show current plan info', () => {
      render(UpgradePrompt, { props: { feature: 'connections' } });

      const planInfo = screen.getByTestId('current-plan-info');
      expect(planInfo).toBeTruthy();
      expect(planInfo.textContent).toContain('Free');
    });
  });

  describe('Feature-Specific Messages', () => {
    it('should show connections message', () => {
      render(UpgradePrompt, { props: { feature: 'connections' } });

      expect(screen.getByTestId('upgrade-prompt').textContent).toContain('streaming connections');
    });

    it('should show scans message', () => {
      render(UpgradePrompt, { props: { feature: 'scans' } });

      expect(screen.getByTestId('upgrade-prompt').textContent).toContain('scan');
    });

    it('should show enforcement message', () => {
      render(UpgradePrompt, { props: { feature: 'enforcement' } });

      expect(screen.getByTestId('upgrade-prompt').textContent).toContain('enforcement');
    });

    it('should show export message', () => {
      render(UpgradePrompt, { props: { feature: 'export' } });

      expect(screen.getByTestId('upgrade-prompt').textContent).toContain('export');
    });
  });

  describe('Dismiss Button', () => {
    it('should have a dismiss button that works', async () => {
      render(UpgradePrompt, { props: { feature: 'connections' } });

      const dismissBtn = screen.getByTestId('dismiss-btn');
      expect(dismissBtn).toBeTruthy();

      await fireEvent.click(dismissBtn);

      // After dismiss, the prompt should not be visible
      expect(screen.queryByTestId('upgrade-prompt')).toBeNull();
    });
  });

  describe('Navigate to Pricing', () => {
    it('should navigate to pricing page when clicking upgrade', async () => {
      render(UpgradePrompt, { props: { feature: 'connections' } });

      const btn = screen.getByTestId('upgrade-cta-btn');
      await fireEvent.click(btn);

      expect(mockNavigateTo).toHaveBeenCalledWith('pricing');
    });
  });
});
