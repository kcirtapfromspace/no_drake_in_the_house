# High-Churn PR Checklist

Lightweight pre-merge gate for **integration-heavy** changes. Tracking: NOD-385
(definition), NOD-382 (retrospective driver). Owner: Staff Engineer review.

This checklist exists because high-churn PRs in the March 25–31 retro window
shipped real regressions (Cloudflare 524 timeouts on YouTube/Tidal sync, COOP
breaking the YouTube Music popup, Spotify needs_reauth from un-refreshed
tokens, missing auth on Convex queries). Standard CI passes were not enough.
The checklist forces a domain-by-domain validation slice before merge.

## When this checklist applies

A PR is **high-churn** if **either** is true:

- More than **500 LOC** total churn (insertions + deletions, excluding lockfiles
  and generated code), OR
- It touches **more than one** of these domains in the same PR:
  - Backend handlers (`backend/src/handlers/**`)
  - Convex functions / schema (`convex/**`)
  - Frontend stores or routing (`frontend/src/lib/stores/**`,
    `frontend/src/lib/utils/simple-router.ts`, `frontend/src/App.svelte`)
  - OAuth flow (any provider `*_connection.rs`, OAuth helpers, callback
    components, redirect URI / COOP handling)
  - CI/CD pipeline (`.github/workflows/**`, `helm/**`, `k8s/**`,
    `render.yaml`, `Tiltfile*`)

The `pr-validation.yml` size-check job already labels PRs over 1000 LOC as
`size/large`. This checklist runs from 500 LOC; the gap is intentional —
500–1000 LOC integration changes are where the retro found regressions.

## Required validation slices

Each box must be checked **with evidence** (linked artifact, screenshot, log
URL, or the literal string `n/a — <reason>`). "Looks fine" is not evidence.

### 1. OAuth callback flow

- [ ] Manually completed an end-to-end OAuth callback for **every provider
      whose code changed** (Spotify / Tidal / YouTube Music) on a
      production-shaped origin (custom domain, not `localhost`).
- [ ] Verified the callback closes the popup and lands the parent on the
      connected state. Note the signal path used (`window.opener`,
      `BroadcastChannel`, `popup.closed` polling) and confirm it survives
      `Cross-Origin-Opener-Policy: same-origin`.
- [ ] Confirmed the redirect URI is derived from the deployed origin (custom
      domain), not `*.onrender.com`, and matches the provider console.
- [ ] OAuth release gate (`scripts/oauth-release-gate.sh`) is green or, if
      bypassed, has an explicit P1-coded waiver from Staff Engineer.

### 2. Sync status correctness

- [ ] Triggered a real library sync for each affected provider and confirmed
      the UI surfaces the **distinct** terminal state — `synced`,
      `rate_limited`, `needs_reauth`, `timeout` — not a generic error.
- [ ] Long-running provider work is `tokio::spawn`'d (or equivalent
      background path) so the HTTP request returns before the Cloudflare
      524 / Render 502 timeout window.
- [ ] Expired access tokens are auto-refreshed before use; a connection in
      `needs_reauth` is reachable from the UI (not silently filtered out of
      the connections list).
- [ ] Rate-limit responses (HTTP 429, provider-specific 406, etc.) are mapped
      to user-facing messages, not raw API errors.

### 3. Connection store consistency

- [ ] After connect / disconnect / reauth, `connections` store, the backend
      `/connections/*` endpoints, and the Convex `connections` table all
      report the same provider list and state. List the three values you
      compared.
- [ ] Frontend store auto-refresh logic (e.g. expired-token detection in
      `fetchConnections`) is covered by a test that mocks
      `apiClient.authenticatedRequest` for the refresh endpoint — adding a
      refresh path without the test mock has previously broken the
      connections page.
- [ ] No direct provider call bypasses the connection store; new code routes
      through the existing store actions.

### 4. CI pipeline sanity

- [ ] `ci.yml` (rust quality, fmt, clippy, frontend typecheck/lint) is
      green on the head commit — not just an earlier commit on the branch.
      Clippy-strictness drift has bounced this gate repeatedly; rebase if
      `rust-1.95` lint set has shifted.
- [ ] `pr-validation.yml` (semantic title, breaking-change detection, size
      label, dependency review) is green.
- [ ] If OAuth code changed: `oauth-release-gate.yml` is green against the
      live probe artifact (or fixture, with rationale).
- [ ] If a provider e2e suite exists for the touched provider, it ran on
      the head commit. Auth bootstrap mocks in e2e fixtures are the
      most common breakage — confirm they still resolve.
- [ ] `secret-scan.yml` is green; new fixtures or test JWTs are added to
      the historical-mock allowlist if and only if they are non-secret.

### 5. Rollback plan

- [ ] One of: (a) a documented revert path with the exact `git revert`
      command and any data-fix follow-up, (b) a feature flag that disables
      the new behavior without a redeploy, or (c) a database migration
      pair (`up` + tested `down`). State which.
- [ ] If the PR adds a Convex schema change or a backend migration, list
      what gets dropped on rollback and whether reads on the prior schema
      will succeed against the new data.
- [ ] If the PR shifts an OAuth redirect URI, list the provider consoles
      that must be reverted in lockstep.

## Author / reviewer workflow

1. **Author** opens the PR and pastes the High-Churn Checklist section from
   `.github/pull_request_template.md` into the description. They fill in
   evidence inline.
2. **Author** self-labels the PR `high-churn` if either trigger condition is
   met. CI applies `size/large` automatically over 1000 LOC; `high-churn`
   is the broader label and is the one Staff review keys off.
3. **Staff Engineer** is auto-requested on any PR labeled `high-churn` and
   blocks merge until every box is checked or marked `n/a` with reason. A
   Staff approval without all boxes checked is itself a checklist violation.
4. **Release Engineer** does not merge a `high-churn` PR without Staff
   approval, regardless of CI state. CI passing while the checklist is
   incomplete is the exact failure mode this gate exists to catch.

## Out of scope

- Pure dependency bumps, lockfile-only changes, generated code refresh.
- Single-domain refactors under 500 LOC (normal review applies).
- Hotfix reverts that strictly undo a prior commit (note the reverted SHA
  in the PR body and skip).

## References

- NOD-382 — weekly engineering retrospective (driver).
- NOD-385 — this checklist's defining issue.
- `docs/release/oauth-release-gate.md` — deterministic OAuth gate.
- `.github/workflows/pr-validation.yml` — size and breaking-change checks.
