<script lang="ts">
  import { authActions } from '../stores/auth';

  let mode: 'login' | 'register' = 'login';
  let isLoading = false;
  let error = '';
  let success = '';

  // Form fields
  let email = '';
  let password = '';
  let confirmPassword = '';

  async function handleSubmit() {
    error = '';
    success = '';
    isLoading = true;

    try {
      if (mode === 'login') {
        const result = await authActions.login(email, password);
        if (!result.success) {
          error = result.message || 'Login failed';
        }
      } else {
        if (password !== confirmPassword) {
          error = 'Passwords do not match';
          isLoading = false;
          return;
        }
        const result = await authActions.register(email, password, confirmPassword, true);
        if (result.success) {
          success = 'Account created! You can now sign in.';
          mode = 'login';
          password = '';
          confirmPassword = '';
        } else {
          error = result.message || 'Registration failed';
        }
      }
    } catch (err) {
      error = 'Something went wrong. Please try again.';
    } finally {
      isLoading = false;
    }
  }

  function switchMode() {
    mode = mode === 'login' ? 'register' : 'login';
    error = '';
    success = '';
    password = '';
    confirmPassword = '';
  }
</script>

<div class="login">
  <div class="login__glow login__glow--rose" aria-hidden="true"></div>
  <div class="login__glow login__glow--blue" aria-hidden="true"></div>

  <div class="login__shell">
    <div class="login__card">
      <section class="login__brand-panel">
        <div class="login__eyebrow" aria-hidden="true">
          <span class="login__eyebrow-pill">Spotify + Apple Music</span>
          <span class="login__eyebrow-pill login__eyebrow-pill--muted">Evidence-led filters</span>
        </div>

        <div class="login__logo">
          <div class="login__icon">
            <svg class="login__icon-svg" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
            </svg>
          </div>
          <h1 class="login__title">
            <span class="login__title-brand">No Drake in the House</span>
            <span class="login__title-main">Clean your feed without collateral damage.</span>
          </h1>
          <p class="login__subtitle">Search artists, block by category, and keep exceptions where you need them across Spotify and Apple Music.</p>
        </div>

        <p class="login__manifesto">
          Built for people who want sharper filters, not scorched-earth playlists.
          The product stays opinionated about evidence while giving you room to manage exceptions.
        </p>

        <div class="login__features">
          <div class="login__feature">
            <div class="login__feature-icon">
              <svg fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
                <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
              </svg>
            </div>
            <div class="login__feature-copy">
              <span class="login__feature-title">Evidence-led artist blocklists</span>
              <span class="login__feature-text">Category-based filters backed by documented offenses.</span>
            </div>
          </div>
          <div class="login__feature">
            <div class="login__feature-icon">
              <svg fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
                <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
              </svg>
            </div>
            <div class="login__feature-copy">
              <span class="login__feature-title">Spotify + Apple Music</span>
              <span class="login__feature-text">One account, shared policy across your connected services.</span>
            </div>
          </div>
          <div class="login__feature">
            <div class="login__feature-icon">
              <svg fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
                <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
              </svg>
            </div>
            <div class="login__feature-copy">
              <span class="login__feature-title">Features and collaborations included</span>
              <span class="login__feature-text">Keep problem artists out of the edge cases, not just the obvious tracks.</span>
            </div>
          </div>
        </div>
      </section>

      <section class="login__form-panel">
        <div class="login__form-copy">
          <p class="login__form-kicker">{mode === 'login' ? 'Welcome back' : 'Create your account'}</p>
          <h2 class="login__form-title">
            {mode === 'login' ? 'Sign in to manage your filters.' : 'Start building a cleaner library.'}
          </h2>
          <p class="login__form-subtitle">
            {mode === 'login'
              ? 'Pick up where you left off and keep your blocklists in sync.'
              : 'Create an account and start shaping what stays out of your rotation.'}
          </p>
        </div>

        <form on:submit|preventDefault={handleSubmit} class="login__form">
          {#if error}
            <div class="login__alert login__alert--error" role="alert">
              <svg class="login__alert-icon icon icon--md" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              <span class="login__alert-text">{error}</span>
            </div>
          {/if}

          {#if success}
            <div class="login__alert login__alert--success" role="alert">
              <svg class="login__alert-icon icon icon--md" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              <span class="login__alert-text">{success}</span>
            </div>
          {/if}

          <div class="login__field">
            <label for="email" class="login__label">Email</label>
            <input
              id="email"
              type="email"
              bind:value={email}
              placeholder="name@example.com"
              autocomplete="email"
              required
              class="login__input"
            />
          </div>

          <div class="login__field">
            <label for="password" class="login__label">Password</label>
            <input
              id="password"
              type="password"
              bind:value={password}
              placeholder="Password"
              autocomplete={mode === 'login' ? 'current-password' : 'new-password'}
              required
              minlength="8"
              class="login__input"
            />
          </div>

          {#if mode === 'register'}
            <div class="login__field">
              <label for="confirmPassword" class="login__label">Confirm Password</label>
              <input
                id="confirmPassword"
                type="password"
                bind:value={confirmPassword}
                placeholder="Confirm password"
                autocomplete="new-password"
                required
                minlength="8"
                class="login__input"
              />
            </div>
          {/if}

          <button
            type="submit"
            disabled={isLoading}
            class="login__submit"
          >
            {#if isLoading}
              <div class="login__spinner"></div>
            {/if}
            {mode === 'login' ? 'Sign in' : 'Create account'}
          </button>
        </form>

        <div class="login__switch">
          <span class="login__switch-text">{mode === 'login' ? "Don't have an account?" : 'Already have an account?'}</span>
          <button
            type="button"
            on:click={switchMode}
            class="login__switch-btn"
          >
            {mode === 'login' ? 'Sign up' : 'Sign in'}
          </button>
        </div>
      </section>
    </div>
  </div>
</div>

<style>
  .login {
    /* Shadcn-style auth palette, kept independent from app theme tokens for readable contrast. */
    --login-bg-page: #09090b;
    --login-bg-elevated: rgba(9, 9, 11, 0.9);
    --login-bg-interactive: rgba(24, 24, 27, 0.92);
    --login-border: rgba(255, 255, 255, 0.08);
    --login-border-strong: rgba(244, 63, 94, 0.42);
    --login-text-primary: #fafafa;
    --login-text-secondary: #d4d4d8;
    --login-text-muted: #a1a1aa;
    --login-brand: #f43f5e;
    --login-brand-hover: #e11d48;
    --login-brand-muted: rgba(244, 63, 94, 0.18);
    --login-radius-lg: 0.875rem;
    --login-radius-xl: 1.25rem;
    --login-radius-pill: 9999px;
    --login-shadow: 0 28px 80px rgba(0, 0, 0, 0.5);
    min-height: 100svh;
    position: relative;
    display: grid;
    place-items: center;
    overflow-x: hidden;
    overflow-y: auto;
    padding: clamp(0.875rem, 3vw, 1.5rem);
    color: var(--login-text-primary);
    color-scheme: dark;
    background:
      radial-gradient(circle at top, rgba(244, 63, 94, 0.16), transparent 28%),
      radial-gradient(circle at bottom right, rgba(59, 130, 246, 0.12), transparent 26%),
      linear-gradient(180deg, #09090b 0%, #111113 48%, #050507 100%);
  }

  .login__glow {
    position: absolute;
    inset: auto;
    border-radius: 999px;
    filter: blur(72px);
    opacity: 0.55;
    pointer-events: none;
  }

  .login__glow--rose {
    top: -8rem;
    left: 10%;
    width: 16rem;
    height: 16rem;
    background: rgba(244, 63, 94, 0.18);
  }

  .login__glow--blue {
    right: -2rem;
    bottom: -8rem;
    width: 18rem;
    height: 18rem;
    background: rgba(37, 99, 235, 0.14);
  }

  .login__shell {
    position: relative;
    z-index: 1;
    width: 100%;
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: calc(100svh - 1.75rem);
  }

  .login__card {
    position: relative;
    width: min(100%, 58rem);
    margin-inline: auto;
    border-radius: var(--login-radius-xl);
    border: 1px solid var(--login-border);
    background:
      linear-gradient(180deg, rgba(255, 255, 255, 0.05), transparent 20%),
      rgba(9, 9, 11, 0.86);
    box-shadow: var(--login-shadow);
    backdrop-filter: blur(18px);
    display: grid;
    grid-template-columns: minmax(0, 0.94fr) minmax(0, 1.06fr);
    overflow: hidden;
  }

  .login__card::before {
    content: '';
    position: absolute;
    inset: 0;
    border-radius: inherit;
    padding: 1px;
    background: linear-gradient(135deg, rgba(255, 255, 255, 0.18), rgba(255, 255, 255, 0.02));
    -webkit-mask: linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0);
    -webkit-mask-composite: xor;
    mask: linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0);
    mask-composite: exclude;
    pointer-events: none;
  }

  .login__brand-panel,
  .login__form-panel {
    position: relative;
    z-index: 1;
    min-width: 0;
    padding: clamp(1.1rem, 2.5vw, 1.85rem);
  }

  .login__brand-panel {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    gap: 1.25rem;
    background:
      radial-gradient(circle at top left, rgba(244, 63, 94, 0.16), transparent 38%),
      linear-gradient(180deg, rgba(255, 255, 255, 0.03), rgba(255, 255, 255, 0));
    border-right: 1px solid rgba(255, 255, 255, 0.06);
  }

  .login__form-panel {
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 1rem;
  }

  .login__eyebrow {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    justify-content: flex-start;
  }

  .login__eyebrow-pill {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    min-height: 2rem;
    padding: 0.35rem 0.8rem;
    border-radius: var(--login-radius-pill);
    border: 1px solid rgba(255, 255, 255, 0.08);
    background: rgba(255, 255, 255, 0.03);
    color: var(--login-text-muted);
    font-size: 0.6875rem;
    font-weight: 700;
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }

  .login__eyebrow-pill--muted {
    color: var(--login-brand);
    background: var(--login-brand-muted);
    border-color: rgba(244, 63, 94, 0.2);
  }

  .login__logo {
    text-align: left;
  }

  .login__icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 3.5rem;
    height: 3.5rem;
    border-radius: var(--login-radius-pill);
    margin-bottom: 0.875rem;
    background:
      radial-gradient(circle at 30% 30%, rgba(255, 255, 255, 0.12), transparent 36%),
      linear-gradient(145deg, var(--login-brand), var(--login-brand-hover));
    box-shadow: 0 10px 24px rgba(244, 63, 94, 0.22);
  }

  .login__icon-svg {
    width: 1.5rem;
    height: 1.5rem;
    color: white;
    max-width: none;
    max-height: none;
  }

  .login__title {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    margin: 0;
    min-width: 0;
  }

  .login__title-brand {
    color: #fda4af;
    font-size: 0.72rem;
    font-weight: 700;
    letter-spacing: 0.16em;
    text-transform: uppercase;
  }

  .login__title-main {
    font-size: clamp(1.75rem, 4vw, 2rem);
    font-weight: 700;
    color: var(--login-text-primary);
    letter-spacing: -0.03em;
    line-height: 1.1;
  }

  .login__subtitle {
    max-width: 24rem;
    margin: 0.625rem 0 0;
    color: var(--login-text-secondary);
    font-size: 0.9375rem;
    line-height: 1.55;
  }

  .login__manifesto {
    max-width: 28rem;
    margin: 0;
    color: var(--login-text-muted);
    font-size: 0.875rem;
    line-height: 1.6;
  }

  .login__form-copy {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .login__form-kicker {
    margin: 0;
    color: #fda4af;
    font-size: 0.75rem;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .login__form-title {
    margin: 0;
    color: var(--login-text-primary);
    font-size: clamp(1.4rem, 2.4vw, 1.8rem);
    line-height: 1.15;
    letter-spacing: -0.03em;
  }

  .login__form-subtitle {
    margin: 0;
    color: var(--login-text-muted);
    font-size: 0.9rem;
    line-height: 1.55;
    max-width: 24rem;
  }

  .login__form {
    display: flex;
    flex-direction: column;
    gap: 0.875rem;
    width: 100%;
    min-width: 0;
  }

  .login__alert {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    padding: 0.75rem 1rem;
    border-radius: var(--login-radius-lg);
    border: 1px solid;
    min-width: 0;
  }

  .login__alert--error {
    background-color: var(--color-error-muted, rgba(239, 68, 68, 0.14));
    border-color: var(--color-error, #ef4444);
  }

  .login__alert--success {
    background-color: var(--color-success-muted, rgba(34, 197, 94, 0.14));
    border-color: var(--color-success, #22c55e);
  }

  .login__alert-icon {
    width: 1.25rem;
    height: 1.25rem;
    flex: 0 0 1.25rem;
    max-width: none;
    max-height: none;
    margin-top: 0.0625rem;
  }

  .login__alert--error .login__alert-icon {
    color: var(--color-error, #ef4444);
  }

  .login__alert--success .login__alert-icon {
    color: var(--color-success, #22c55e);
  }

  .login__alert-text {
    display: block;
    flex: 1 1 auto;
    min-width: 0;
    font-size: var(--text-sm, 0.875rem);
    line-height: 1.45;
    overflow-wrap: anywhere;
  }

  .login__alert--error .login__alert-text {
    color: var(--color-error, #ef4444);
  }

  .login__alert--success .login__alert-text {
    color: var(--color-success, #22c55e);
  }

  .login__field {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .login__label {
    display: block;
    font-size: var(--text-sm, 0.875rem);
    font-weight: 600;
    color: var(--login-text-primary);
    margin-bottom: 0.375rem;
  }

  .login__input {
    width: 100%;
    min-height: 3rem;
    border-radius: var(--login-radius-lg);
    padding: 0.8125rem 0.95rem;
    box-sizing: border-box;
    color: var(--login-text-primary);
    background:
      linear-gradient(180deg, rgba(255, 255, 255, 0.03), rgba(255, 255, 255, 0.01)),
      var(--login-bg-interactive);
    border: 1px solid var(--login-border);
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04);
    transition:
      border-color var(--transition-fast, 120ms ease),
      box-shadow var(--transition-fast, 120ms ease),
      transform var(--transition-fast, 120ms ease);
    font-size: var(--text-base, 0.9375rem);
    line-height: 1.4;
    font-family: var(--font-family-sans, 'DM Sans', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif);
    appearance: none;
  }

  .login__input::placeholder {
    color: var(--login-text-muted);
  }

  .login__input:focus {
    outline: none;
    border-color: var(--login-border-strong);
    box-shadow: 0 0 0 3px var(--login-brand-muted);
    transform: translateY(-1px);
  }

  .login__submit {
    width: 100%;
    min-height: 3rem;
    background:
      radial-gradient(circle at 20% 20%, rgba(255, 255, 255, 0.12), transparent 34%),
      linear-gradient(135deg, var(--login-brand), var(--login-brand-hover));
    color: #ffffff;
    font-weight: 700;
    padding: 0.8125rem 1rem;
    border-radius: var(--login-radius-pill);
    border: none;
    cursor: pointer;
    transition:
      transform var(--transition-fast, 120ms ease),
      filter var(--transition-fast, 120ms ease),
      box-shadow var(--transition-fast, 120ms ease);
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    margin-top: 0.25rem;
    font-size: var(--text-base, 0.9375rem);
    box-shadow: 0 10px 22px rgba(244, 63, 94, 0.22);
  }

  .login__submit:hover:not(:disabled) {
    transform: scale(1.02);
    filter: brightness(1.03);
  }

  .login__submit:active:not(:disabled) {
    transform: scale(0.98);
  }

  .login__submit:focus-visible {
    outline: none;
    box-shadow:
      0 0 0 3px var(--login-brand-muted),
      0 10px 22px rgba(244, 63, 94, 0.22);
  }

  .login__submit:disabled {
    opacity: 0.72;
    cursor: not-allowed;
    box-shadow: none;
  }

  .login__spinner {
    width: 1.25rem;
    height: 1.25rem;
    border: 2px solid currentColor;
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  .login__switch {
    text-align: left;
    font-size: 0.9375rem;
    padding-top: 0.125rem;
  }

  .login__switch-text {
    color: var(--login-text-muted);
  }

  .login__switch-btn {
    color: #fda4af;
    text-decoration: none;
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
    margin-left: 0.25rem;
    transition: color var(--transition-fast, 120ms ease);
  }

  .login__switch-btn:hover {
    color: #fecdd3;
  }

  .login__switch-btn:focus-visible {
    outline: 2px solid var(--login-brand);
    outline-offset: 2px;
    border-radius: 2px;
  }

  .login__features {
    margin-top: auto;
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0.75rem;
  }

  .login__feature {
    display: flex;
    align-items: flex-start;
    gap: 0.625rem;
    padding: 0.75rem 0.8rem;
    border-radius: calc(var(--login-radius-lg) + 0.125rem);
    border: 1px solid rgba(255, 255, 255, 0.05);
    background: rgba(255, 255, 255, 0.03);
  }

  .login__feature:nth-child(3) {
    grid-column: 1 / -1;
  }

  .login__feature-icon {
    width: 1.75rem;
    height: 1.75rem;
    border-radius: var(--login-radius-pill);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    background-color: var(--login-brand-muted);
    color: var(--login-brand);
  }

  .login__feature-icon svg {
    width: 1rem;
    height: 1rem;
    max-width: none;
    max-height: none;
  }

  .login__feature-copy {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }

  .login__feature-title {
    color: var(--login-text-primary);
    font-size: 0.8rem;
    font-weight: 600;
    line-height: 1.35;
  }

  .login__feature-text {
    color: #d4d4d8;
    font-size: 0.76rem;
    line-height: 1.45;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  @media (max-width: 880px) {
    .login {
      padding-block: 0.75rem;
    }

    .login__shell {
      min-height: auto;
      align-items: flex-start;
    }

    .login__card {
      grid-template-columns: 1fr;
      width: min(100%, 30rem);
      overflow: hidden;
    }

    .login__brand-panel {
      order: 1;
      gap: 0.8rem;
      border-right: none;
      border-bottom: 1px solid rgba(255, 255, 255, 0.06);
      padding-bottom: 0.9rem;
    }

    .login__form-panel {
      order: 2;
      gap: 0.85rem;
      padding-top: 0.95rem;
    }

    .login__eyebrow {
      gap: 0.4rem;
    }

    .login__eyebrow-pill {
      min-height: 1.75rem;
      padding: 0.28rem 0.65rem;
      font-size: 0.625rem;
    }

    .login__logo {
      display: grid;
      grid-template-columns: auto minmax(0, 1fr);
      gap: 0.85rem;
      align-items: flex-start;
    }

    .login__title {
      grid-column: 2;
    }

    .login__subtitle {
      grid-column: 1 / -1;
    }

    .login__icon {
      width: 2.85rem;
      height: 2.85rem;
      margin-bottom: 0;
    }

    .login__icon-svg {
      width: 1.2rem;
      height: 1.2rem;
    }

    .login__title {
      gap: 0.24rem;
    }

    .login__title-brand {
      font-size: 0.64rem;
    }

    .login__title-main {
      font-size: clamp(1.25rem, 6vw, 1.65rem);
    }

    .login__subtitle {
      margin-top: 0.35rem;
      font-size: 0.875rem;
      line-height: 1.45;
      max-width: none;
    }

    .login__manifesto,
    .login__features {
      display: none;
    }

    .login__form-copy {
      gap: 0.25rem;
    }

    .login__form-kicker {
      font-size: 0.6875rem;
    }

    .login__form-title {
      font-size: 1.25rem;
    }

    .login__form-subtitle {
      font-size: 0.85rem;
      line-height: 1.45;
    }

    .login__form {
      gap: 0.75rem;
    }

    .login__label {
      margin-bottom: 0.3rem;
      font-size: 0.8125rem;
    }

    .login__input,
    .login__submit {
      min-height: 2.875rem;
    }
  }

  @media (max-width: 420px) {
    .login {
      padding-inline: 0.75rem;
    }

    .login__brand-panel,
    .login__form-panel {
      padding: 0.9rem;
    }

    .login__title-main {
      font-size: 1.2rem;
    }

    .login__subtitle,
    .login__form-subtitle,
    .login__switch {
      font-size: 0.8125rem;
    }
  }

  @media (max-height: 840px) {
    .login {
      padding-block: 0.625rem;
    }

    .login__brand-panel,
    .login__form-panel {
      padding: 1rem 1.1rem;
    }

    .login__features {
      gap: 0.6rem;
    }

    .login__feature {
      padding: 0.625rem 0.7rem;
    }
  }

  @media (max-height: 760px) {
    .login__manifesto,
    .login__features {
      display: none;
    }

    .login__brand-panel,
    .login__form-panel {
      padding: 0.95rem 1rem;
    }

    .login__title-main {
      font-size: clamp(1.5rem, 3.2vw, 1.8rem);
    }

    .login__form-title {
      font-size: 1.3rem;
    }

    .login__form-subtitle {
      font-size: 0.85rem;
    }
  }
</style>
