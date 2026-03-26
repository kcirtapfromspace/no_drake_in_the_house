<script lang="ts">
  let openIndex: number | null = null;

  function toggle(i: number) {
    openIndex = openIndex === i ? null : i;
  }

  const faqs = [
    {
      q: 'What is No Drake in the House?',
      a: 'A multi-platform music streaming tool that lets you create Do-Not-Play lists and automatically enforce them across Spotify, Apple Music, Tidal, YouTube Music, and Deezer. Scan your library, find problematic artists, and clean your playlists.'
    },
    {
      q: 'How does it work?',
      a: 'Connect your streaming services via OAuth. We scan your library and cross-reference it against our offense database — a curated, evidence-backed catalog of artist misconduct. You choose which artists to block, then we remove their tracks from your playlists automatically.'
    },
    {
      q: 'What streaming platforms are supported?',
      a: 'Spotify, Apple Music, Tidal, YouTube Music, and Deezer. Each connection uses official OAuth so your credentials are never stored.'
    },
    {
      q: 'What is the offense database?',
      a: 'A crowd-sourced, evidence-backed database of artist misconduct — covering categories like domestic violence, sexual assault, hate speech, fraud, and more. Every entry links to primary sources (court records, news articles, police reports). The database is publicly browsable without an account.'
    },
    {
      q: 'How are offenses verified?',
      a: 'Offenses go through a multi-step pipeline: news articles are ingested, entities are extracted via NLP, classifications are assigned with confidence scores, and moderators manually verify entries above the threshold. Human-verified entries are always promoted regardless of confidence score.'
    },
    {
      q: 'What does enforcement do?',
      a: 'Enforcement scans your playlists on a connected platform and removes tracks by blocked artists. You can preview changes with a dry-run before executing. Free users run enforcement manually; Pro users can schedule automatic enforcement.'
    },
    {
      q: 'What are community lists?',
      a: 'Community lists are curated blocklists maintained by other users or organizations. You can subscribe to lists and auto-update your personal blocklist as the community list evolves. Pro users can create and publish their own lists.'
    },
    {
      q: 'Is there a free plan?',
      a: 'Yes. Free includes 1 connected service, 1 library scan per month, manual enforcement, and browsing the offense database and community lists. Pro ($5/mo) unlocks unlimited connections, scans, auto-enforcement, community list creation, and exports. Team ($12/mo) adds up to 5 seats.'
    },
    {
      q: 'How is revenue impact calculated?',
      a: 'We estimate streaming revenue impact using an industry-average per-stream payout rate ($0.004/stream). This is a simulation for awareness purposes — actual payouts vary by platform, region, and contract.'
    },
    {
      q: 'What about collaboration detection?',
      a: 'We map artist collaboration networks from track credits. If you block an artist, we can flag tracks where they appear as a featured artist, producer, or songwriter — not just tracks under their own name.'
    },
    {
      q: 'Is there a browser extension?',
      a: 'Yes. The extension uses a signed bloom filter snapshot to check artists in real-time as you browse streaming platforms, without sending your listening data to our servers.'
    },
    {
      q: 'How do I report a new offense?',
      a: 'From the offense database, use the "Submit Offense" form. Provide the artist name, offense category, severity, description, and at least one evidence link. Submissions are reviewed by moderators before appearing publicly.'
    },
  ];
</script>

<div class="faq">
  <h1 class="faq__title">Frequently Asked Questions</h1>
  <p class="faq__subtitle">Everything you need to know about No Drake in the House.</p>

  <div class="faq__list">
    {#each faqs as item, i}
      <div class="faq__item" class:faq__item--open={openIndex === i}>
        <button type="button" class="faq__question" on:click={() => toggle(i)}>
          <span>{item.q}</span>
          <svg class="faq__chevron" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
            <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z" clip-rule="evenodd" />
          </svg>
        </button>
        {#if openIndex === i}
          <div class="faq__answer">
            <p>{item.a}</p>
          </div>
        {/if}
      </div>
    {/each}
  </div>
</div>

<style>
  .faq {
    max-width: 48rem;
    margin: 0 auto;
    padding: 2rem 0;
  }
  .faq__title {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--color-text-primary, #fafafa);
    margin-bottom: 0.25rem;
  }
  .faq__subtitle {
    color: var(--color-text-secondary, #a1a1aa);
    font-size: 0.9375rem;
    margin-bottom: 2rem;
  }
  .faq__list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .faq__item {
    background: var(--color-bg-elevated, #111113);
    border: 1px solid var(--color-border-default, #27272a);
    border-radius: 0.75rem;
    overflow: hidden;
  }
  .faq__item--open {
    border-color: var(--color-border-hover, #3f3f46);
  }
  .faq__question {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 1rem 1.25rem;
    background: none;
    border: none;
    color: var(--color-text-primary, #fafafa);
    font-size: 0.9375rem;
    font-weight: 500;
    text-align: left;
    cursor: pointer;
    transition: color 0.15s;
  }
  .faq__question:hover {
    color: var(--color-brand-primary, #10b981);
  }
  .faq__question:focus-visible {
    outline: 2px solid var(--color-brand-primary, #10b981);
    outline-offset: -2px;
  }
  .faq__chevron {
    width: 1.25rem;
    height: 1.25rem;
    flex-shrink: 0;
    color: var(--color-text-tertiary, #71717a);
    transition: transform 0.2s;
  }
  .faq__item--open .faq__chevron {
    transform: rotate(180deg);
  }
  .faq__answer {
    padding: 0 1.25rem 1rem;
  }
  .faq__answer p {
    color: var(--color-text-secondary, #a1a1aa);
    font-size: 0.875rem;
    line-height: 1.625;
  }
</style>
