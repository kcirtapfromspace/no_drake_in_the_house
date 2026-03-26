<script lang="ts">
  import { navigateTo } from '../utils/simple-router';
  import { authActions } from '../stores/auth';

  function scrollToSection(id: string) {
    const el = document.getElementById(id);
    if (el) el.scrollIntoView({ behavior: 'smooth' });
  }
</script>

<div class="landing">
  <!-- Navigation Bar -->
  <nav class="landing-nav">
    <div class="landing-nav__inner">
      <div class="landing-nav__brand">
        <svg class="landing-nav__logo" viewBox="0 0 32 32" aria-hidden="true">
          <path d="M16 2 L4 8 L4 16 C4 23 9 28.5 16 30 C23 28.5 28 23 28 16 L28 8 Z" fill="#09090b" stroke="#10b981" stroke-width="1.5"/>
          <circle cx="13" cy="21" r="2.5" fill="#10b981"/>
          <rect x="15" y="10" width="1.8" height="11" fill="#10b981"/>
          <path d="M15.5 10 Q20 8 21 11 Q20 9.5 16.8 10.5" fill="#10b981"/>
          <line x1="8" y1="8" x2="24" y2="24" stroke="#ef4444" stroke-width="2.5" stroke-linecap="round"/>
          <line x1="24" y1="8" x2="8" y2="24" stroke="#ef4444" stroke-width="2.5" stroke-linecap="round"/>
        </svg>
        <span class="landing-nav__title">No Drake in the House</span>
      </div>
      <div class="landing-nav__links">
        <button type="button" class="landing-nav__link" on:click={() => scrollToSection('features')}>Features</button>
        <button type="button" class="landing-nav__link" on:click={() => scrollToSection('how-it-works')}>How It Works</button>
        <button type="button" class="landing-nav__link" on:click={() => navigateTo('pricing')}>Pricing</button>
        <button type="button" class="landing-nav__link landing-nav__link--cta" on:click={() => navigateTo('login')}>Sign In</button>
      </div>
    </div>
  </nav>

  <!-- Hero Section -->
  <section class="hero">
    <div class="hero__inner">
      <div class="hero__content">
        <h1 class="hero__headline">Take Control of Your Music Library</h1>
        <p class="hero__subhead">Automatically block problematic artists across all your streaming platforms. Create Do-Not-Play lists, scan your library, and enforce your values.</p>
        <div class="hero__actions">
          <button type="button" class="btn btn--primary btn--lg" on:click={() => navigateTo('login')}>Get Started Free</button>
          <button type="button" class="btn btn--outline btn--lg" on:click={() => scrollToSection('how-it-works')}>See How It Works</button>
        </div>
      </div>
      <div class="hero__visual">
        <div class="hero__mock">
          <div class="mock-header">
            <div class="mock-dot mock-dot--red"></div>
            <div class="mock-dot mock-dot--yellow"></div>
            <div class="mock-dot mock-dot--green"></div>
            <span class="mock-title">Library Scan Results</span>
          </div>
          <div class="mock-body">
            <div class="mock-stat">
              <span class="mock-stat__number">2,847</span>
              <span class="mock-stat__label">Tracks scanned</span>
            </div>
            <div class="mock-stat">
              <span class="mock-stat__number mock-stat__number--warn">23</span>
              <span class="mock-stat__label">Flagged tracks</span>
            </div>
            <div class="mock-stat">
              <span class="mock-stat__number mock-stat__number--ok">98%</span>
              <span class="mock-stat__label">Library grade</span>
            </div>
            <div class="mock-row">
              <div class="mock-row__indicator mock-row__indicator--blocked"></div>
              <div class="mock-row__info">
                <span class="mock-row__name">Track removed from "Chill Vibes"</span>
                <span class="mock-row__detail">Blocked artist detected</span>
              </div>
              <span class="mock-row__badge">Enforced</span>
            </div>
            <div class="mock-row">
              <div class="mock-row__indicator mock-row__indicator--blocked"></div>
              <div class="mock-row__info">
                <span class="mock-row__name">Track removed from "Workout Mix"</span>
                <span class="mock-row__detail">Blocked artist detected</span>
              </div>
              <span class="mock-row__badge">Enforced</span>
            </div>
            <div class="mock-row">
              <div class="mock-row__indicator mock-row__indicator--clean"></div>
              <div class="mock-row__info">
                <span class="mock-row__name">"Daily Mix 1" is clean</span>
                <span class="mock-row__detail">No blocked artists found</span>
              </div>
              <span class="mock-row__badge mock-row__badge--clean">Clean</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>

  <!-- Features Section -->
  <section id="features" class="features">
    <div class="section__inner">
      <h2 class="section__heading">Everything You Need to Curate Responsibly</h2>
      <p class="section__subheading">Powerful tools to identify, block, and remove problematic artists from your music library.</p>
      <div class="features__grid">
        <div class="feature-card">
          <div class="feature-card__icon">
            <svg aria-hidden="true" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
              <path d="M9 12l2 2 4-4"/>
            </svg>
          </div>
          <h3 class="feature-card__title">Multi-Platform Sync</h3>
          <p class="feature-card__desc">Connect Spotify, Apple Music, Tidal, YouTube Music, and Deezer. One blocklist, enforced everywhere.</p>
        </div>
        <div class="feature-card">
          <div class="feature-card__icon">
            <svg aria-hidden="true" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="11" cy="11" r="8"/>
              <line x1="21" y1="21" x2="16.65" y2="16.65"/>
              <line x1="8" y1="11" x2="14" y2="11"/>
              <line x1="11" y1="8" x2="11" y2="14"/>
            </svg>
          </div>
          <h3 class="feature-card__title">Smart Library Scanning</h3>
          <p class="feature-card__desc">Deep-scan your saved tracks, playlists, and liked songs. Instant detection of blocked artists hiding in your library.</p>
        </div>
        <div class="feature-card">
          <div class="feature-card__icon">
            <svg aria-hidden="true" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/>
              <polyline points="22 4 12 14.01 9 11.01"/>
            </svg>
          </div>
          <h3 class="feature-card__title">Automatic Enforcement</h3>
          <p class="feature-card__desc">Set it and forget it. Blocked tracks are removed from playlists automatically with dry-run previews and 30-day undo.</p>
        </div>
        <div class="feature-card">
          <div class="feature-card__icon">
            <svg aria-hidden="true" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/>
              <circle cx="9" cy="7" r="4"/>
              <path d="M23 21v-2a4 4 0 0 0-3-3.87"/>
              <path d="M16 3.13a4 4 0 0 1 0 7.75"/>
            </svg>
          </div>
          <h3 class="feature-card__title">Community Lists</h3>
          <p class="feature-card__desc">Subscribe to curated blocklists maintained by the community. Share your lists and help others clean their libraries.</p>
        </div>
        <div class="feature-card">
          <div class="feature-card__icon">
            <svg aria-hidden="true" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
              <polyline points="14 2 14 8 20 8"/>
              <line x1="16" y1="13" x2="8" y2="13"/>
              <line x1="16" y1="17" x2="8" y2="17"/>
              <polyline points="10 9 9 9 8 9"/>
            </svg>
          </div>
          <h3 class="feature-card__title">Offense Database</h3>
          <p class="feature-card__desc">Evidence-backed database of artist offenses with source citations, severity ratings, and category tags.</p>
        </div>
        <div class="feature-card">
          <div class="feature-card__icon">
            <svg aria-hidden="true" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="12" y1="20" x2="12" y2="10"/>
              <line x1="18" y1="20" x2="18" y2="4"/>
              <line x1="6" y1="20" x2="6" y2="16"/>
            </svg>
          </div>
          <h3 class="feature-card__title">Revenue Impact Analysis</h3>
          <p class="feature-card__desc">See exactly how much of your streaming spend goes to blocked artists. Make informed decisions about your music consumption.</p>
        </div>
      </div>
    </div>
  </section>

  <!-- How It Works -->
  <section id="how-it-works" class="how-it-works">
    <div class="section__inner">
      <h2 class="section__heading">How It Works</h2>
      <p class="section__subheading">Three simple steps to a cleaner music library.</p>
      <div class="steps">
        <div class="step">
          <div class="step__number">1</div>
          <h3 class="step__title">Connect Your Services</h3>
          <p class="step__desc">Link your Spotify, Apple Music, Tidal, YouTube Music, or Deezer accounts with secure OAuth. Your credentials are never stored.</p>
        </div>
        <div class="step__connector" aria-hidden="true">
          <svg aria-hidden="true" viewBox="0 0 40 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M0 12 L32 12 M28 6 L36 12 L28 18" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </div>
        <div class="step">
          <div class="step__number">2</div>
          <h3 class="step__title">Scan for Problematic Artists</h3>
          <p class="step__desc">Our engine cross-references your library against the offense database and your personal blocklist. See exactly what needs attention.</p>
        </div>
        <div class="step__connector" aria-hidden="true">
          <svg aria-hidden="true" viewBox="0 0 40 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M0 12 L32 12 M28 6 L36 12 L28 18" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </div>
        <div class="step">
          <div class="step__number">3</div>
          <h3 class="step__title">Automatically Clean Your Playlists</h3>
          <p class="step__desc">Preview changes with a dry-run, then enforce with one click. Blocked tracks are removed across all connected platforms.</p>
        </div>
      </div>
    </div>
  </section>

  <!-- Supported Platforms -->
  <section class="platforms">
    <div class="section__inner">
      <h2 class="section__heading">Works With Your Favorite Platforms</h2>
      <div class="platforms__grid">
        <div class="platform-card">
          <svg aria-hidden="true" class="platform-card__icon" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 0C5.4 0 0 5.4 0 12s5.4 12 12 12 12-5.4 12-12S18.66 0 12 0zm5.521 17.34c-.24.359-.66.48-1.021.24-2.82-1.74-6.36-2.101-10.561-1.141-.418.122-.779-.179-.899-.539-.12-.421.18-.78.54-.9 4.56-1.021 8.52-.6 11.64 1.32.42.18.479.659.301 1.02zm1.44-3.3c-.301.42-.841.6-1.262.3-3.239-1.98-8.159-2.58-11.939-1.38-.479.12-1.02-.12-1.14-.6-.12-.48.12-1.021.6-1.141C9.6 9.9 15 10.561 18.72 12.84c.361.181.54.78.241 1.2zm.12-3.36C15.24 8.4 8.82 8.16 5.16 9.301c-.6.179-1.2-.181-1.38-.721-.18-.601.18-1.2.72-1.381 4.26-1.26 11.28-1.02 15.721 1.621.539.3.719 1.02.419 1.56-.299.421-1.02.599-1.559.3z"/>
          </svg>
          <span class="platform-card__name">Spotify</span>
        </div>
        <div class="platform-card">
          <svg aria-hidden="true" class="platform-card__icon" viewBox="0 0 24 24" fill="currentColor">
            <path d="M23.994 6.124a9.23 9.23 0 00-.24-2.19c-.317-1.31-1.062-2.31-2.18-3.043A5.022 5.022 0 0019.7.175a10.065 10.065 0 00-1.838-.114C17.09.017 16.32 0 12.003 0h-.008C7.676 0 6.906.017 6.14.06a10.065 10.065 0 00-1.838.114 5.02 5.02 0 00-1.874.717C1.31 1.59.565 2.59.248 3.9a9.23 9.23 0 00-.24 2.19C-.009 6.87 0 7.64 0 12s-.009 5.13.007 5.876c.017.714.086 1.428.24 2.19.317 1.31 1.062 2.31 2.18 3.043.585.374 1.21.625 1.874.717.598.084 1.218.123 1.838.137.77.03 1.54.047 5.86.047h.008c4.32 0 5.09-.017 5.856-.06a10.065 10.065 0 001.838-.114 5.022 5.022 0 001.874-.717c1.118-.732 1.863-1.732 2.18-3.043.154-.762.223-1.476.24-2.19.017-.746.026-1.516.026-5.876s.008-5.13-.032-5.876zM11.72 17.637c-3.313 0-5.986-2.673-5.986-5.986s2.673-5.986 5.986-5.986 5.986 2.673 5.986 5.986-2.673 5.986-5.986 5.986zm6.226-10.794a1.4 1.4 0 110-2.8 1.4 1.4 0 010 2.8z"/>
            <circle cx="11.72" cy="11.65" r="3.87"/>
          </svg>
          <span class="platform-card__name">Apple Music</span>
        </div>
        <div class="platform-card">
          <svg aria-hidden="true" class="platform-card__icon" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12.012 3.992L8.008 7.996 4.004 3.992 0 7.996v8.008l4.004 4.004 4.004-4.004 4.004 4.004 4.004-4.004 4.004 4.004L24.024 7.996l-4.004-4.004-4.004 4.004z"/>
          </svg>
          <span class="platform-card__name">Tidal</span>
        </div>
        <div class="platform-card">
          <svg aria-hidden="true" class="platform-card__icon" viewBox="0 0 24 24" fill="currentColor">
            <path d="M21.582 6.186a2.477 2.477 0 00-1.768-1.768C18.254 4 12 4 12 4s-6.254 0-7.814.418c-.86.23-1.538.908-1.768 1.768C2 7.746 2 12 2 12s0 4.254.418 5.814c.23.86.908 1.538 1.768 1.768C5.746 20 12 20 12 20s6.254 0 7.814-.418a2.477 2.477 0 001.768-1.768C22 16.254 22 12 22 12s0-4.254-.418-5.814zM10 15.5v-7l6 3.5-6 3.5z"/>
          </svg>
          <span class="platform-card__name">YouTube Music</span>
        </div>
        <div class="platform-card">
          <svg aria-hidden="true" class="platform-card__icon" viewBox="0 0 24 24" fill="currentColor">
            <path d="M18.81 4.16v3.03C14.39 4.79 9.47 5.6 6.18 8.47c-1.37 1.19-2.43 2.72-3.09 4.47-.66 1.75-.9 3.63-.7 5.49.08.74.23 1.47.43 2.17.4 1.41 1.07 2.72 1.97 3.84.12.15.24.29.37.43-.29-.53-.54-1.09-.74-1.67a11.39 11.39 0 01.68-8.81c.87-1.8 2.24-3.27 3.92-4.22 3.89-2.21 8.64-2.09 12.39.09V6.36h2.87L19.08 0l-5.77 6.36h2.87v-.2c-.89-.59-1.87-1.06-2.92-1.38l.55-1.01z"/>
          </svg>
          <span class="platform-card__name">Deezer</span>
        </div>
      </div>
    </div>
  </section>

  <!-- CTA Section -->
  <section class="cta">
    <div class="section__inner">
      <h2 class="cta__heading">Ready to Clean Up Your Library?</h2>
      <p class="cta__subheading">Join thousands of listeners who take an active stand with their streaming choices.</p>
      <button type="button" class="btn btn--primary btn--lg" on:click={() => navigateTo('login')}>Sign Up Free</button>
    </div>
  </section>

  <!-- Footer -->
  <footer class="landing-footer">
    <div class="landing-footer__inner">
      <div class="landing-footer__brand">
        <svg class="landing-footer__logo" viewBox="0 0 32 32" aria-hidden="true">
          <path d="M16 2 L4 8 L4 16 C4 23 9 28.5 16 30 C23 28.5 28 23 28 16 L28 8 Z" fill="#09090b" stroke="#10b981" stroke-width="1.5"/>
          <circle cx="13" cy="21" r="2.5" fill="#10b981"/>
          <rect x="15" y="10" width="1.8" height="11" fill="#10b981"/>
          <path d="M15.5 10 Q20 8 21 11 Q20 9.5 16.8 10.5" fill="#10b981"/>
          <line x1="8" y1="8" x2="24" y2="24" stroke="#ef4444" stroke-width="2.5" stroke-linecap="round"/>
          <line x1="24" y1="8" x2="8" y2="24" stroke="#ef4444" stroke-width="2.5" stroke-linecap="round"/>
        </svg>
        <span>No Drake in the House</span>
      </div>
      <div class="landing-footer__links">
        <button type="button" class="landing-footer__link" on:click={() => scrollToSection('features')}>About</button>
        <button type="button" class="landing-footer__link" on:click={() => navigateTo('pricing')}>Pricing</button>
        <button type="button" class="landing-footer__link" on:click={() => navigateTo('offense-database')}>Offense Database</button>
        <button type="button" class="landing-footer__link" on:click={() => navigateTo('community')}>Community</button>
        <a class="landing-footer__link" href="/privacy">Privacy</a>
        <a class="landing-footer__link" href="/terms">Terms</a>
      </div>
      <p class="landing-footer__copy">&copy; 2026 No Drake in the House. All rights reserved.</p>
    </div>
  </footer>
</div>

<style>
  /* =================== */
  /* Base & Layout        */
  /* =================== */
  .landing {
    background: var(--color-bg-page);
    color: var(--color-text-primary);
    min-height: 100vh;
    font-family: 'DM Sans', -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    overflow-x: hidden;
  }

  .section__inner {
    max-width: 72rem;
    margin: 0 auto;
    padding: 5rem 1.5rem;
  }

  .section__heading {
    font-size: 2rem;
    font-weight: 700;
    text-align: center;
    margin: 0 0 0.75rem;
    color: var(--color-text-primary);
  }

  .section__subheading {
    text-align: center;
    color: var(--color-text-secondary);
    font-size: 1.125rem;
    max-width: 36rem;
    margin: 0 auto 3rem;
    line-height: 1.6;
  }

  /* =================== */
  /* Navigation           */
  /* =================== */
  .landing-nav {
    position: sticky;
    top: 0;
    z-index: 50;
    background: var(--nav-bg);
    backdrop-filter: blur(12px);
    border-bottom: 1px solid var(--color-border-default);
  }

  .landing-nav__inner {
    max-width: 72rem;
    margin: 0 auto;
    padding: 0.875rem 1.5rem;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .landing-nav__brand {
    display: flex;
    align-items: center;
    gap: 0.625rem;
  }

  .landing-nav__logo {
    width: 2rem;
    height: 2rem;
  }

  .landing-nav__title {
    font-weight: 700;
    font-size: 1.125rem;
    color: var(--color-text-primary);
  }

  .landing-nav__links {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .landing-nav__link {
    background: none;
    border: none;
    color: var(--color-text-secondary);
    cursor: pointer;
    padding: 0.5rem 0.875rem;
    font-size: 0.875rem;
    font-family: inherit;
    border-radius: 0.375rem;
    transition: color 0.15s, background 0.15s;
  }

  .landing-nav__link:hover {
    color: var(--color-text-primary);
    background: var(--color-bg-interactive);
  }

  .landing-nav__link--cta {
    color: var(--color-success);
    font-weight: 600;
  }

  .landing-nav__link--cta:hover {
    background: #0d3b2e;
    color: var(--color-success);
  }

  /* =================== */
  /* Buttons              */
  /* =================== */
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border: none;
    border-radius: 0.5rem;
    font-family: inherit;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .btn--lg {
    padding: 0.875rem 2rem;
    font-size: 1rem;
  }

  .btn--primary {
    background: var(--color-success);
    color: var(--color-text-inverse);
  }

  .btn--primary:hover {
    background: #059669;
    transform: translateY(-1px);
    box-shadow: 0 4px 16px rgba(16, 185, 129, 0.25);
  }

  .btn--outline {
    background: transparent;
    color: var(--color-text-secondary);
    border: 1px solid var(--color-border-hover);
  }

  .btn--outline:hover {
    background: var(--color-bg-interactive);
    border-color: var(--color-text-muted);
    color: var(--color-text-primary);
  }

  /* =================== */
  /* Hero                 */
  /* =================== */
  .hero {
    padding: 2rem 0 0;
    background: var(--color-bg-page);
  }

  .hero__inner {
    max-width: 72rem;
    margin: 0 auto;
    padding: 4rem 1.5rem 5rem;
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 4rem;
    align-items: center;
  }

  .hero__headline {
    font-size: 3.25rem;
    font-weight: 800;
    line-height: 1.1;
    margin: 0 0 1.25rem;
    color: var(--color-text-primary);
    letter-spacing: -0.02em;
  }

  .hero__subhead {
    font-size: 1.25rem;
    line-height: 1.6;
    color: var(--color-text-secondary);
    margin: 0 0 2rem;
    max-width: 32rem;
  }

  .hero__actions {
    display: flex;
    gap: 1rem;
    flex-wrap: wrap;
  }

  /* Mock dashboard visual */
  .hero__mock {
    background: var(--color-bg-interactive);
    border: 1px solid var(--color-border-default);
    border-radius: 0.75rem;
    overflow: hidden;
    box-shadow: 0 24px 48px rgba(0,0,0,0.4);
  }

  .mock-header {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.75rem 1rem;
    background: var(--color-bg-surface);
    border-bottom: 1px solid var(--color-border-default);
  }

  .mock-dot {
    width: 0.625rem;
    height: 0.625rem;
    border-radius: 50%;
  }

  .mock-dot--red { background: var(--color-error); }
  .mock-dot--yellow { background: var(--color-warning); }
  .mock-dot--green { background: var(--color-success); }

  .mock-title {
    font-size: 0.75rem;
    color: var(--color-text-tertiary);
    margin-left: 0.5rem;
  }

  .mock-body {
    padding: 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .mock-stat {
    display: flex;
    flex-direction: column;
    display: inline-flex;
    gap: 0.125rem;
  }

  .mock-body > :nth-child(1),
  .mock-body > :nth-child(2),
  .mock-body > :nth-child(3) {
    display: inline-flex;
    flex-direction: column;
  }

  .mock-body {
    display: grid;
    grid-template-columns: 1fr 1fr 1fr;
    grid-template-rows: auto auto auto;
    gap: 0.75rem;
  }

  .mock-body > .mock-stat:nth-child(1) { grid-column: 1; grid-row: 1; }
  .mock-body > .mock-stat:nth-child(2) { grid-column: 2; grid-row: 1; }
  .mock-body > .mock-stat:nth-child(3) { grid-column: 3; grid-row: 1; }
  .mock-body > .mock-row { grid-column: 1 / -1; }

  .mock-stat__number {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--color-text-primary);
  }

  .mock-stat__number--warn {
    color: var(--color-warning);
  }

  .mock-stat__number--ok {
    color: var(--color-success);
  }

  .mock-stat__label {
    font-size: 0.6875rem;
    color: var(--color-text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .mock-row {
    display: flex;
    align-items: center;
    gap: 0.625rem;
    padding: 0.625rem 0.75rem;
    background: var(--color-bg-page);
    border-radius: 0.375rem;
  }

  .mock-row__indicator {
    width: 0.5rem;
    height: 0.5rem;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .mock-row__indicator--blocked { background: var(--color-error); }
  .mock-row__indicator--clean { background: var(--color-success); }

  .mock-row__info {
    flex: 1;
    min-width: 0;
  }

  .mock-row__name {
    display: block;
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .mock-row__detail {
    font-size: 0.6875rem;
    color: var(--color-text-muted);
  }

  .mock-row__badge {
    flex-shrink: 0;
    font-size: 0.6875rem;
    font-weight: 600;
    padding: 0.125rem 0.5rem;
    border-radius: 9999px;
    background: var(--color-error-muted);
    color: var(--color-error);
  }

  .mock-row__badge--clean {
    background: var(--color-success-muted);
    color: var(--color-success);
  }

  /* =================== */
  /* Features Grid        */
  /* =================== */
  .features {
    background: var(--color-bg-page);
    border-top: 1px solid var(--color-bg-interactive);
  }

  .features__grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1.5rem;
  }

  .feature-card {
    background: var(--color-bg-interactive);
    border: 1px solid var(--color-border-default);
    border-radius: 0.75rem;
    padding: 1.75rem;
    transition: border-color 0.2s, transform 0.2s;
  }

  .feature-card:hover {
    border-color: var(--color-success);
    transform: translateY(-2px);
  }

  .feature-card__icon {
    width: 2.5rem;
    height: 2.5rem;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #0d3b2e;
    border-radius: 0.5rem;
    color: var(--color-success);
    margin-bottom: 1rem;
  }

  .feature-card__icon svg {
    width: 1.25rem;
    height: 1.25rem;
  }

  .feature-card__title {
    font-size: 1.0625rem;
    font-weight: 600;
    margin: 0 0 0.5rem;
    color: var(--color-text-primary);
  }

  .feature-card__desc {
    font-size: 0.9375rem;
    line-height: 1.6;
    color: var(--color-text-secondary);
    margin: 0;
  }

  /* =================== */
  /* How It Works         */
  /* =================== */
  .how-it-works {
    background: var(--color-bg-inset);
    border-top: 1px solid var(--color-bg-interactive);
  }

  .steps {
    display: flex;
    align-items: flex-start;
    justify-content: center;
    gap: 1rem;
  }

  .step {
    flex: 1;
    max-width: 18rem;
    text-align: center;
  }

  .step__number {
    width: 3rem;
    height: 3rem;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: var(--color-success);
    color: var(--color-text-inverse);
    font-weight: 800;
    font-size: 1.25rem;
    border-radius: 50%;
    margin-bottom: 1.25rem;
  }

  .step__title {
    font-size: 1.0625rem;
    font-weight: 600;
    margin: 0 0 0.5rem;
    color: var(--color-text-primary);
  }

  .step__desc {
    font-size: 0.9375rem;
    line-height: 1.6;
    color: var(--color-text-secondary);
    margin: 0;
  }

  .step__connector {
    display: flex;
    align-items: center;
    padding-top: 0.875rem;
    color: var(--color-border-hover);
    flex-shrink: 0;
  }

  .step__connector svg {
    width: 2.5rem;
    height: 1.5rem;
  }

  /* =================== */
  /* Platforms            */
  /* =================== */
  .platforms {
    background: var(--color-bg-page);
    border-top: 1px solid var(--color-bg-interactive);
  }

  .platforms__grid {
    display: flex;
    justify-content: center;
    flex-wrap: wrap;
    gap: 1.5rem;
  }

  .platform-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.75rem;
    padding: 1.5rem 2rem;
    background: var(--color-bg-interactive);
    border: 1px solid var(--color-border-default);
    border-radius: 0.75rem;
    min-width: 8rem;
    transition: border-color 0.2s;
  }

  .platform-card:hover {
    border-color: var(--color-border-hover);
  }

  .platform-card__icon {
    width: 2.5rem;
    height: 2.5rem;
    color: var(--color-text-secondary);
  }

  .platform-card__name {
    font-size: 0.9375rem;
    font-weight: 600;
    color: var(--color-text-secondary);
  }

  /* =================== */
  /* CTA Section          */
  /* =================== */
  .cta {
    background: var(--color-bg-inset);
    border-top: 1px solid var(--color-bg-interactive);
    text-align: center;
  }

  .cta__heading {
    font-size: 2.25rem;
    font-weight: 700;
    margin: 0 0 0.75rem;
    color: var(--color-text-primary);
  }

  .cta__subheading {
    color: var(--color-text-secondary);
    font-size: 1.125rem;
    margin: 0 0 2rem;
  }

  /* =================== */
  /* Footer               */
  /* =================== */
  .landing-footer {
    background: var(--color-bg-page);
    border-top: 1px solid var(--color-border-default);
  }

  .landing-footer__inner {
    max-width: 72rem;
    margin: 0 auto;
    padding: 2.5rem 1.5rem;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1.25rem;
  }

  .landing-footer__brand {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-weight: 700;
    font-size: 1rem;
    color: var(--color-text-secondary);
  }

  .landing-footer__logo {
    width: 1.5rem;
    height: 1.5rem;
  }

  .landing-footer__links {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 0.25rem;
  }

  .landing-footer__link {
    background: none;
    border: none;
    color: var(--color-text-tertiary);
    cursor: pointer;
    padding: 0.375rem 0.75rem;
    font-size: 0.875rem;
    font-family: inherit;
    text-decoration: none;
    border-radius: 0.25rem;
    transition: color 0.15s;
  }

  .landing-footer__link:hover {
    color: var(--color-text-secondary);
  }

  .landing-footer__copy {
    font-size: 0.8125rem;
    color: var(--color-text-muted);
    margin: 0;
  }

  /* =================== */
  /* Responsive           */
  /* =================== */
  @media (max-width: 768px) {
    .hero__inner {
      grid-template-columns: 1fr;
      gap: 2.5rem;
      padding: 2.5rem 1.25rem 3rem;
    }

    .hero__headline {
      font-size: 2.25rem;
    }

    .hero__subhead {
      font-size: 1.0625rem;
    }

    .hero__visual {
      order: -1;
    }

    .features__grid {
      grid-template-columns: 1fr;
    }

    .steps {
      flex-direction: column;
      align-items: center;
    }

    .step {
      max-width: 100%;
    }

    .step__connector {
      transform: rotate(90deg);
      padding-top: 0;
    }

    .section__heading {
      font-size: 1.625rem;
    }

    .cta__heading {
      font-size: 1.625rem;
    }

    .landing-nav__links {
      gap: 0;
    }

    .landing-nav__link {
      padding: 0.5rem 0.5rem;
      font-size: 0.8125rem;
    }

    .platforms__grid {
      gap: 0.75rem;
    }

    .platform-card {
      padding: 1rem 1.25rem;
      min-width: 6rem;
    }
  }

  @media (min-width: 769px) and (max-width: 1024px) {
    .features__grid {
      grid-template-columns: repeat(2, 1fr);
    }

    .hero__headline {
      font-size: 2.5rem;
    }
  }

  .landing-nav__link:focus-visible,
  .landing-nav__link--cta:focus-visible,
  .btn:focus-visible,
  .landing-footer__link:focus-visible {
    outline: 2px solid var(--color-brand-primary, #10b981);
    outline-offset: 2px;
  }
</style>
