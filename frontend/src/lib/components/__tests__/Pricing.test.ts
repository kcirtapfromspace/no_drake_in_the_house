/**
 * Pricing Component Tests
 * Tests tier cards, pricing toggle, feature comparison, and checkout flow
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';

// vi.mock factories are hoisted -- use vi.hoisted for shared state
const { mockInitiateCheckout, mockCurrentPlanSubscribers } = vi.hoisted(() => {
  return {
    mockInitiateCheckout: vi.fn(),
    mockCurrentPlanSubscribers: [] as Array<(v: string) => void>,
  };
});

let currentPlanValue = 'free';

vi.mock('../../stores/billing', () => ({
  subscription: {
    subscribe: (fn: (v: any) => void) => {
      fn(null);
      return () => {};
    },
  },
  currentPlan: {
    subscribe: (fn: (v: string) => void) => {
      fn(currentPlanValue);
      mockCurrentPlanSubscribers.push(fn);
      return () => {};
    },
  },
  initiateCheckout: mockInitiateCheckout,
  initiatePortal: vi.fn(),
  featureAccess: vi.fn().mockReturnValue(false),
}));

vi.mock('../../utils/simple-router', () => ({
  navigateTo: vi.fn(),
  currentRoute: {
    subscribe: (fn: (v: string) => void) => {
      fn('pricing');
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

import Pricing from '../Pricing.svelte';

describe('Pricing Component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    currentPlanValue = 'free';
  });

  describe('Tier Cards Rendering', () => {
    it('should render 3 tier cards (Free, Pro, Team)', () => {
      render(Pricing);

      expect(screen.getByTestId('tier-card-free')).toBeTruthy();
      expect(screen.getByTestId('tier-card-pro')).toBeTruthy();
      expect(screen.getByTestId('tier-card-team')).toBeTruthy();
    });

    it('should show $0/mo for Free tier', () => {
      render(Pricing);

      const freeCard = screen.getByTestId('tier-card-free');
      expect(freeCard.textContent).toContain('$0');
    });

    it('should show $5/mo for Pro tier', () => {
      render(Pricing);

      const proCard = screen.getByTestId('tier-card-pro');
      expect(proCard.textContent).toContain('$5');
    });

    it('should show $12/mo for Team tier', () => {
      render(Pricing);

      const teamCard = screen.getByTestId('tier-card-team');
      expect(teamCard.textContent).toContain('$12');
    });
  });

  describe('Tier Features', () => {
    it('should list correct features for Free tier', () => {
      render(Pricing);

      const freeCard = screen.getByTestId('tier-card-free');
      expect(freeCard.textContent).toContain('1 streaming connection');
      expect(freeCard.textContent).toContain('Manual scans');
    });

    it('should list correct features for Pro tier', () => {
      render(Pricing);

      const proCard = screen.getByTestId('tier-card-pro');
      expect(proCard.textContent).toContain('3 streaming connections');
      expect(proCard.textContent).toContain('Auto-scan weekly');
    });

    it('should list correct features for Team tier', () => {
      render(Pricing);

      const teamCard = screen.getByTestId('tier-card-team');
      expect(teamCard.textContent).toContain('Unlimited connections');
      expect(teamCard.textContent).toContain('Real-time enforcement');
    });
  });

  describe('Plan Buttons for Free Users', () => {
    it('should show "Current Plan" button on Free tier for free users', () => {
      render(Pricing);

      const freeCard = screen.getByTestId('tier-card-free');
      const btn = freeCard.querySelector('[data-testid="tier-btn-free"]');
      expect(btn).toBeTruthy();
      expect(btn!.textContent).toContain('Current Plan');
    });

    it('should show "Upgrade" button on Pro tier for free users', () => {
      render(Pricing);

      const proCard = screen.getByTestId('tier-card-pro');
      const btn = proCard.querySelector('[data-testid="tier-btn-pro"]');
      expect(btn).toBeTruthy();
      expect(btn!.textContent).toContain('Upgrade');
    });

    it('should show "Upgrade" button on Team tier for free users', () => {
      render(Pricing);

      const teamCard = screen.getByTestId('tier-card-team');
      const btn = teamCard.querySelector('[data-testid="tier-btn-team"]');
      expect(btn).toBeTruthy();
      expect(btn!.textContent).toContain('Upgrade');
    });
  });

  describe('Plan Buttons for Pro Users', () => {
    it('should show "Current Plan" for pro users on Pro tier', () => {
      currentPlanValue = 'pro';

      render(Pricing);

      const proCard = screen.getByTestId('tier-card-pro');
      const btn = proCard.querySelector('[data-testid="tier-btn-pro"]');
      expect(btn).toBeTruthy();
      expect(btn!.textContent).toContain('Current Plan');
    });
  });

  describe('Checkout Flow', () => {
    it('should call initiateCheckout when clicking Upgrade on Pro', async () => {
      render(Pricing);

      const btn = screen.getByTestId('tier-btn-pro');
      await fireEvent.click(btn);

      expect(mockInitiateCheckout).toHaveBeenCalledWith('pro');
    });
  });

  describe('Annual Toggle', () => {
    it('should show discounted annual prices when toggled', async () => {
      render(Pricing);

      const toggle = screen.getByTestId('billing-toggle');
      await fireEvent.click(toggle);

      // Pro annual: $48/yr, Team annual: $108/yr
      const proCard = screen.getByTestId('tier-card-pro');
      expect(proCard.textContent).toContain('$48');

      const teamCard = screen.getByTestId('tier-card-team');
      expect(teamCard.textContent).toContain('$108');
    });
  });

  describe('Feature Comparison Table', () => {
    it('should render the feature comparison table', () => {
      render(Pricing);

      const table = screen.getByTestId('feature-comparison');
      expect(table).toBeTruthy();
    });
  });

  describe('FAQ Section', () => {
    it('should render FAQ section', () => {
      render(Pricing);

      const faq = screen.getByTestId('pricing-faq');
      expect(faq).toBeTruthy();
      expect(faq.textContent).toContain('Frequently Asked Questions');
    });
  });

  describe('Pro Tier Highlight', () => {
    it('should highlight Pro as Most Popular', () => {
      render(Pricing);

      const proCard = screen.getByTestId('tier-card-pro');
      expect(proCard.textContent).toContain('Most Popular');
    });
  });
});
