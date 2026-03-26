<script lang="ts">
  import { currentPlan, initiateCheckout } from '../stores/billing';

  let isAnnual = false;

  interface Tier {
    id: string;
    name: string;
    monthlyPrice: number;
    annualPrice: number;
    description: string;
    features: string[];
    highlight?: boolean;
    badge?: string;
  }

  const tiers: Tier[] = [
    {
      id: 'free',
      name: 'Free',
      monthlyPrice: 0,
      annualPrice: 0,
      description: 'Get started with basic music blocklist features.',
      features: [
        '1 streaming connection',
        'Manual scans',
        'Basic blocklist',
        'Community categories',
        'Email support',
      ],
    },
    {
      id: 'pro',
      name: 'Pro',
      monthlyPrice: 5,
      annualPrice: 48,
      description: 'For serious listeners who want automated protection.',
      features: [
        '3 streaming connections',
        'Auto-scan weekly',
        'Automatic enforcement',
        'CSV export',
        'Full blocklist management',
        'Basic analytics',
        'Priority support',
      ],
      highlight: true,
      badge: 'Most Popular',
    },
    {
      id: 'team',
      name: 'Team',
      monthlyPrice: 12,
      annualPrice: 108,
      description: 'For families and groups who share listening spaces.',
      features: [
        'Unlimited connections',
        'Real-time enforcement',
        'All export formats',
        'Full analytics dashboard',
        'Team member management',
        'Custom category lists',
        'Dedicated support',
      ],
    },
  ];

  interface ComparisonRow {
    feature: string;
    free: string;
    pro: string;
    team: string;
  }

  const comparisonRows: ComparisonRow[] = [
    { feature: 'Streaming connections', free: '1', pro: '3', team: 'Unlimited' },
    { feature: 'Library scans', free: 'Manual', pro: 'Weekly auto', team: 'Real-time' },
    { feature: 'Enforcement', free: 'Manual', pro: 'Automatic', team: 'Real-time' },
    { feature: 'Blocklist size', free: '50 artists', pro: 'Unlimited', team: 'Unlimited' },
    { feature: 'Export', free: '-', pro: 'CSV', team: 'All formats' },
    { feature: 'Analytics', free: '-', pro: 'Basic', team: 'Full' },
    { feature: 'Team members', free: '-', pro: '-', team: 'Up to 10' },
    { feature: 'Support', free: 'Email', pro: 'Priority', team: 'Dedicated' },
  ];

  interface FaqItem {
    question: string;
    answer: string;
  }

  const faqItems: FaqItem[] = [
    {
      question: 'Can I change plans later?',
      answer: 'Yes, you can upgrade or downgrade at any time. Changes take effect at the start of your next billing cycle.',
    },
    {
      question: 'What happens when I cancel?',
      answer: 'You keep access to your current plan features until the end of your billing period. After that, you revert to the Free plan.',
    },
    {
      question: 'Is there a free trial for Pro?',
      answer: 'New users get a 14-day free trial of Pro features when they sign up. No credit card required.',
    },
    {
      question: 'What streaming services are supported?',
      answer: 'We support Spotify, Apple Music, YouTube Music, Tidal, and Deezer. The number of simultaneous connections depends on your plan.',
    },
    {
      question: 'How does enforcement work?',
      answer: 'Enforcement automatically removes or skips blocked artists from your playlists and library. Free users do this manually, Pro gets weekly automation, and Team gets real-time protection.',
    },
  ];

  let expandedFaq: number | null = null;

  function toggleFaq(index: number) {
    expandedFaq = expandedFaq === index ? null : index;
  }

  function handleTierClick(tierId: string) {
    if (tierId === plan || tierId === 'free') return;
    initiateCheckout(tierId);
  }

  $: plan = $currentPlan;

  // Reactive price computations -- Svelte 4 tracks these via $:
  $: priceMap = Object.fromEntries(
    tiers.map((t) => [
      t.id,
      {
        price: t.monthlyPrice === 0 ? '$0' : isAnnual ? `$${t.annualPrice}` : `$${t.monthlyPrice}`,
        period: t.monthlyPrice === 0 ? '/mo' : isAnnual ? '/yr' : '/mo',
        btnText: t.id === plan ? 'Current Plan' : t.id === 'free' && plan === 'free' ? 'Current Plan' : 'Upgrade',
        isCurrent: t.id === plan || (t.id === 'free' && plan === 'free'),
      },
    ])
  );
</script>

<div class="pricing-page">
  <!-- Header -->
  <div class="pricing-header">
    <h1 class="pricing-title">Choose Your Plan</h1>
    <p class="pricing-subtitle">
      Protect your ears. Block problematic artists across all your streaming services.
    </p>

    <!-- Billing Toggle -->
    <div class="billing-toggle-wrapper">
      <span class="toggle-label" class:toggle-label--active={!isAnnual}>Monthly</span>
      <button
        type="button"
        data-testid="billing-toggle"
        class="billing-toggle"
        class:billing-toggle--annual={isAnnual}
        on:click={() => (isAnnual = !isAnnual)}
        aria-label={isAnnual ? 'Switch to monthly billing' : 'Switch to annual billing'}
      >
        <span class="billing-toggle__thumb" />
      </button>
      <span class="toggle-label" class:toggle-label--active={isAnnual}>
        Annual
        <span class="toggle-save">Save 20%</span>
      </span>
    </div>
  </div>

  <!-- Tier Cards -->
  <div class="tier-grid">
    {#each tiers as tier}
      <div
        data-testid="tier-card-{tier.id}"
        class="tier-card"
        class:tier-card--highlight={tier.highlight}
      >
        {#if tier.badge}
          <div class="tier-badge">{tier.badge}</div>
        {/if}

        <div class="tier-header">
          <h2 class="tier-name">{tier.name}</h2>
          <p class="tier-description">{tier.description}</p>
        </div>

        <div class="tier-price">
          <span class="tier-price__amount">{priceMap[tier.id].price}</span>
          <span class="tier-price__period">{priceMap[tier.id].period}</span>
        </div>

        <button
          type="button"
          data-testid="tier-btn-{tier.id}"
          class="tier-btn"
          class:tier-btn--primary={!priceMap[tier.id].isCurrent && tier.id !== 'free'}
          class:tier-btn--current={priceMap[tier.id].isCurrent}
          disabled={priceMap[tier.id].isCurrent}
          on:click={() => handleTierClick(tier.id)}
        >
          {priceMap[tier.id].btnText}
        </button>

        <ul class="tier-features">
          {#each tier.features as feature}
            <li class="tier-feature">
              <svg class="tier-feature__icon" viewBox="0 0 20 20" fill="currentColor">
                <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
              </svg>
              {feature}
            </li>
          {/each}
        </ul>
      </div>
    {/each}
  </div>

  <!-- Feature Comparison -->
  <div data-testid="feature-comparison" class="comparison-section">
    <h2 class="comparison-title">Compare Plans</h2>
    <div class="comparison-table-wrapper">
      <table class="comparison-table">
        <thead>
          <tr>
            <th class="comparison-th comparison-th--feature">Feature</th>
            <th class="comparison-th">Free</th>
            <th class="comparison-th comparison-th--highlight">Pro</th>
            <th class="comparison-th">Team</th>
          </tr>
        </thead>
        <tbody>
          {#each comparisonRows as row}
            <tr class="comparison-row">
              <td class="comparison-td comparison-td--feature">{row.feature}</td>
              <td class="comparison-td">{row.free}</td>
              <td class="comparison-td comparison-td--highlight">{row.pro}</td>
              <td class="comparison-td">{row.team}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>

  <!-- FAQ -->
  <div data-testid="pricing-faq" class="faq-section">
    <h2 class="faq-title">Frequently Asked Questions</h2>
    <div class="faq-list">
      {#each faqItems as faq, i}
        <div class="faq-item">
          <button
            type="button"
            class="faq-question"
            on:click={() => toggleFaq(i)}
            aria-expanded={expandedFaq === i}
          >
            <span>{faq.question}</span>
            <svg
              class="faq-chevron"
              class:faq-chevron--open={expandedFaq === i}
              viewBox="0 0 20 20"
              fill="currentColor"
            >
              <path fill-rule="evenodd" d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" clip-rule="evenodd" />
            </svg>
          </button>
          {#if expandedFaq === i}
            <div class="faq-answer">
              <p>{faq.answer}</p>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  </div>
</div>

<style>
  .pricing-page {
    max-width: 72rem;
    margin: 0 auto;
    padding: 2rem 1rem;
  }

  .pricing-header {
    text-align: center;
    margin-bottom: 3rem;
  }

  .pricing-title {
    font-size: 2rem;
    font-weight: 700;
    color: var(--color-text-primary, white);
    margin-bottom: 0.5rem;
  }

  .pricing-subtitle {
    color: var(--color-text-secondary, #a1a1aa);
    font-size: 1.125rem;
    margin-bottom: 2rem;
  }

  .billing-toggle-wrapper {
    display: inline-flex;
    align-items: center;
    gap: 0.75rem;
  }

  .toggle-label {
    font-size: 0.875rem;
    color: var(--color-text-tertiary, #71717a);
    transition: color 0.2s;
  }

  .toggle-label--active {
    color: var(--color-text-primary, white);
    font-weight: 500;
  }

  .toggle-save {
    display: inline-block;
    background: rgba(16, 185, 129, 0.15);
    color: #34d399;
    font-size: 0.7rem;
    padding: 0.125rem 0.5rem;
    border-radius: 9999px;
    margin-left: 0.25rem;
    font-weight: 600;
  }

  .billing-toggle {
    position: relative;
    width: 3rem;
    height: 1.5rem;
    border-radius: 9999px;
    background: var(--color-bg-interactive, #27272a);
    border: 1px solid var(--color-border-default, #3f3f46);
    cursor: pointer;
    transition: background 0.2s;
    padding: 0;
  }

  .billing-toggle--annual {
    background: #10b981;
    border-color: #10b981;
  }

  .billing-toggle__thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 1.125rem;
    height: 1.125rem;
    border-radius: 9999px;
    background: white;
    transition: transform 0.2s;
  }

  .billing-toggle--annual .billing-toggle__thumb {
    transform: translateX(1.5rem);
  }

  /* Tier Grid */
  .tier-grid {
    display: grid;
    grid-template-columns: 1fr;
    gap: 1.5rem;
    margin-bottom: 4rem;
  }

  @media (min-width: 768px) {
    .tier-grid {
      grid-template-columns: repeat(3, 1fr);
    }
  }

  .tier-card {
    position: relative;
    background: var(--color-bg-elevated, #18181b);
    border: 1px solid var(--color-border-default, #3f3f46);
    border-radius: 0.75rem;
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    transition: all 0.2s;
  }

  .tier-card:hover {
    border-color: var(--color-border-hover, #52525b);
  }

  .tier-card--highlight {
    border-color: #10b981;
    box-shadow: 0 0 0 1px #10b981;
  }

  .tier-card--highlight:hover {
    border-color: #34d399;
    box-shadow: 0 0 0 1px #34d399;
  }

  .tier-badge {
    position: absolute;
    top: -0.75rem;
    left: 50%;
    transform: translateX(-50%);
    background: #10b981;
    color: white;
    font-size: 0.75rem;
    font-weight: 600;
    padding: 0.25rem 0.75rem;
    border-radius: 9999px;
    white-space: nowrap;
  }

  .tier-header {
    margin-bottom: 1.5rem;
  }

  .tier-name {
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--color-text-primary, white);
    margin-bottom: 0.25rem;
  }

  .tier-description {
    font-size: 0.875rem;
    color: var(--color-text-secondary, #a1a1aa);
    line-height: 1.5;
  }

  .tier-price {
    margin-bottom: 1.5rem;
    display: flex;
    align-items: baseline;
    gap: 0.25rem;
  }

  .tier-price__amount {
    font-size: 2.5rem;
    font-weight: 700;
    color: var(--color-text-primary, white);
  }

  .tier-price__period {
    font-size: 0.875rem;
    color: var(--color-text-tertiary, #71717a);
  }

  .tier-btn {
    width: 100%;
    padding: 0.75rem 1rem;
    border-radius: 0.5rem;
    font-weight: 600;
    font-size: 0.875rem;
    cursor: pointer;
    transition: all 0.2s;
    border: 1px solid transparent;
    margin-bottom: 1.5rem;
  }

  .tier-btn--primary {
    background: #10b981;
    color: white;
    border-color: #10b981;
  }

  .tier-btn--primary:hover {
    background: #059669;
    border-color: #059669;
  }

  .tier-btn--current {
    background: var(--color-bg-interactive, #27272a);
    color: var(--color-text-tertiary, #71717a);
    border-color: var(--color-border-default, #3f3f46);
    cursor: default;
  }

  .tier-features {
    list-style: none;
    padding: 0;
    margin: 0;
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .tier-feature {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
    color: var(--color-text-secondary, #d4d4d8);
  }

  .tier-feature__icon {
    width: 1rem;
    height: 1rem;
    color: #10b981;
    flex-shrink: 0;
  }

  /* Comparison Table */
  .comparison-section {
    margin-bottom: 4rem;
  }

  .comparison-title {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--color-text-primary, white);
    text-align: center;
    margin-bottom: 2rem;
  }

  .comparison-table-wrapper {
    overflow-x: auto;
  }

  .comparison-table {
    width: 100%;
    border-collapse: collapse;
  }

  .comparison-th {
    padding: 0.75rem 1rem;
    text-align: center;
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-text-primary, white);
    border-bottom: 1px solid var(--color-border-default, #3f3f46);
  }

  .comparison-th--feature {
    text-align: left;
  }

  .comparison-th--highlight {
    color: #34d399;
  }

  .comparison-row:hover {
    background: rgba(255, 255, 255, 0.02);
  }

  .comparison-td {
    padding: 0.75rem 1rem;
    text-align: center;
    font-size: 0.875rem;
    color: var(--color-text-secondary, #a1a1aa);
    border-bottom: 1px solid rgba(63, 63, 70, 0.5);
  }

  .comparison-td--feature {
    text-align: left;
    color: var(--color-text-primary, #d4d4d8);
    font-weight: 500;
  }

  .comparison-td--highlight {
    color: #d4d4d8;
    font-weight: 500;
  }

  /* FAQ */
  .faq-section {
    max-width: 48rem;
    margin: 0 auto;
  }

  .faq-title {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--color-text-primary, white);
    text-align: center;
    margin-bottom: 2rem;
  }

  .faq-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .faq-item {
    background: var(--color-bg-elevated, #18181b);
    border: 1px solid var(--color-border-default, #3f3f46);
    border-radius: 0.75rem;
    overflow: hidden;
  }

  .faq-question {
    width: 100%;
    padding: 1rem 1.25rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.9375rem;
    font-weight: 500;
    color: var(--color-text-primary, white);
    background: none;
    border: none;
    cursor: pointer;
    text-align: left;
    transition: color 0.2s;
  }

  .faq-question:hover {
    color: #34d399;
  }

  .faq-chevron {
    width: 1.25rem;
    height: 1.25rem;
    color: var(--color-text-tertiary, #71717a);
    transition: transform 0.2s;
    flex-shrink: 0;
  }

  .faq-chevron--open {
    transform: rotate(180deg);
  }

  .faq-answer {
    padding: 0 1.25rem 1rem;
  }

  .faq-answer p {
    font-size: 0.875rem;
    color: var(--color-text-secondary, #a1a1aa);
    line-height: 1.625;
  }
</style>
