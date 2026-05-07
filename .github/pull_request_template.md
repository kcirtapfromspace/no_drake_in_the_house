<!--
PR template — covers all PRs.

If this PR is high-churn (>500 LOC churn OR multi-domain — see
docs/release/high-churn-pr-checklist.md), the High-Churn Checklist section
below is REQUIRED. Self-label the PR `high-churn` so Staff Engineer review
is auto-requested.
-->

## Summary

<!-- 1–3 bullets on what this PR does and why. -->

## Linked issues

<!-- e.g. NOD-385, NOD-382. Use full identifiers so Paperclip backlinks. -->

## Test plan

- [ ] Unit / integration tests added or updated
- [ ] Manual verification (describe)
- [ ] CI green on head commit

---

## High-Churn PR Checklist

> Required if the PR is **>500 LOC churn** or touches **more than one** of:
> backend handlers, Convex, frontend stores/routing, OAuth flow, CI/CD.
> Otherwise delete this section. Reference: `docs/release/high-churn-pr-checklist.md`.

**Trigger:** <!-- "size: NNN LOC" or "domains: backend + frontend + convex" -->

### 1. OAuth callback flow
- [ ] End-to-end callback completed on production-shaped origin for every
      affected provider (evidence: ___)
- [ ] Popup → parent signal path verified under
      `Cross-Origin-Opener-Policy: same-origin` (evidence: ___)
- [ ] Redirect URI derived from custom domain and matches provider console
      (evidence: ___)
- [ ] `oauth-release-gate.yml` green or P1-coded waiver from Staff (evidence: ___)

### 2. Sync status correctness
- [ ] Live sync triggered per provider; UI shows distinct terminal state
      (`synced` / `rate_limited` / `needs_reauth` / `timeout`) (evidence: ___)
- [ ] Long provider work runs in background (no Cloudflare 524 / Render 502)
      (evidence: ___)
- [ ] Expired tokens auto-refresh; `needs_reauth` connections still visible
      in UI (evidence: ___)
- [ ] Rate-limit / provider-specific errors (429 / 406 / etc.) mapped to
      user messages (evidence: ___)

### 3. Connection store consistency
- [ ] `connections` store, backend `/connections/*`, and Convex
      `connections` table report the same state — list the three values
      compared (evidence: ___)
- [ ] Refresh / reauth paths covered by tests that mock
      `apiClient.authenticatedRequest` (evidence: ___)
- [ ] No direct provider call bypasses the store (evidence: ___)

### 4. CI pipeline sanity
- [ ] `ci.yml` green on head commit (rust fmt/clippy, frontend
      typecheck/lint) (link: ___)
- [ ] `pr-validation.yml` green (semantic title, size, deps) (link: ___)
- [ ] `oauth-release-gate.yml` green if OAuth code changed (link: ___)
- [ ] Provider e2e suite green on head commit if applicable (link: ___)
- [ ] `secret-scan.yml` green; new fixtures justified (link: ___)

### 5. Rollback plan
- [ ] Rollback path documented — one of:
  - [ ] Revert command: `git revert <sha>` and follow-up: ___
  - [ ] Feature flag: ___ (default off)
  - [ ] Migration pair (up + tested down): ___
- [ ] Schema/migration impact described (what drops on rollback, whether
      old reads succeed against new data) (evidence: ___)
- [ ] OAuth provider-console reverts listed if redirect URI changed
      (evidence: ___)

### Sign-off

- [ ] Author: every box above is checked or marked `n/a — <reason>`
- [ ] Staff Engineer: approved with checklist complete (Release Engineer
      does not merge `high-churn` without this)
