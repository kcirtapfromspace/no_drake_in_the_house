# Retroactive Evaluation — PR #18 against High-Churn PR Checklist

**PR:** `feat: admin dashboard + security hardening (#18)`
**Merge SHA:** `1164801`
**Date:** 2026-03-28
**Churn:** 14 files, +1187 / −16 (1203 LOC) — well over the 500 LOC trigger.
**Domains touched:** Convex schema/queries (5 files), backend handler fmt
(per commit body), frontend stores (`admin.ts`, `auth.ts`),
frontend routing (`simple-router.ts`, `App.svelte`, `Breadcrumb.svelte`),
frontend UI (`AdminDashboard.svelte`, `Layout.svelte`),
Convex cron (`crons.ts`), auth helper (`convex/lib/auth.ts`).
Multi-domain trigger fires.

This evaluation is reconstruction-only — the checklist did not exist at merge
time. Findings are recorded so the gate's value is demonstrated against a
real, already-shipped change rather than a hypothetical.

## Verdict

**Would have been gated for Staff review.** The PR clears most slices but
has gaps in slices 3 (Connection store consistency — author auto-refresh
test added 2 commits later) and 5 (Rollback plan — no documented revert).
None are blockers in retrospect; both are exactly the class of follow-up
that the checklist would have surfaced before merge.

## Slice-by-slice findings

### 1. OAuth callback flow — n/a (not touched in this PR)

The PR's OAuth-adjacent change is rustfmt-only on
`oauth_config_validator`, `token_vault`, and `spotify_connection`. No flow
or redirect-URI change. Marking `n/a — fmt-only OAuth touch` would have
been the right call.

### 2. Sync status correctness — partially covered

- The PR touches `convex/sync.ts` (auth gate on sync-run queries) but does
  not change sync state machinery.
- **Gap:** the same retro window contains PR #28 (`fix(sync): run provider
  library syncs in background`) and `14aefd24` (`auto-refresh expired
  Spotify access tokens`). Those landed *after* PR #18. Under the
  checklist, the live-sync slice would have asked the author to verify
  the connections list still showed Spotify when its access token had
  expired — exactly the bug `14aefd24` later fixed. **The checklist would
  not have caught the timeout bug, but it would have surfaced the
  needs_reauth filtering bug one merge earlier.**

### 3. Connection store consistency — gap

- PR #18 added `frontend/src/lib/stores/admin.ts` (137 lines) and updated
  `frontend/src/lib/stores/auth.ts` by 1 line.
- **Finding:** the follow-up `f5a4433` (`test: add authenticatedRequest
  mock to connections test`) on 2026-03-30 says explicitly: *"The
  auto-refresh logic in fetchConnections calls
  apiClient.authenticatedRequest for the refresh endpoint. Add the mock
  to prevent TypeError in test."* This is the failure mode slice 3
  encodes. Under the checklist this PR would have been blocked for the
  test mock until that follow-up was inlined.

### 4. CI pipeline sanity — eventually green, but with retries

- The merge body itself contains a fixup commit
  (`fix: resolve rustfmt formatting and frontend TypeScript errors`)
  applying rustfmt and fixing missing `admin` route + unused import.
  Under the checklist requirement that **CI is green on the head commit**,
  the author would have been forced to squash-or-fixup these into the
  same commit before the Staff approval, instead of merging a multi-commit
  PR with intermediate red checks.
- The broader retro window has many `chore(ci):` and `fix(ci):` commits
  (`a71937a`, `7495642`, `0e9453c`, `861d067`, `6a078f4`, `609f401`).
  This is the exact "clippy-strictness drift" the checklist warns about.
  Slice 4 would have routed these into Staff-tracked CI work rather than
  one-off post-merge patches.

### 5. Rollback plan — gap

- No revert command, no feature flag, no migration-down listed in the PR
  body. The Convex auth-gate change (`requireOwner` on
  `systemHealth`, `migration.tableCounts`, sync-run queries; analytics →
  `requireCurrentUser`) is **not** trivially reverted — clients calling
  those queries unauthenticated will start failing the moment the change
  ships. A naive `git revert` is safe (re-opens the auth holes), but the
  *recommended* rollback (feature-flag the new `requireOwner` calls) is
  absent.
- Under the checklist this would have been a forced conversation: either
  ship behind a flag (`AUTH_HARDENING_ENABLED`) or ship a Staff-approved
  acknowledgement that the only rollback is `git revert` and that doing
  so re-introduces the auth bypass. Either is fine; silence is not.

## What the checklist would have changed

| Slice | Outcome | Cost |
|-------|---------|------|
| 1     | n/a, signed off | seconds |
| 2     | live-sync probe per provider | 5–10 min |
| 3     | inline the `f5a4433` test mock | 10 min |
| 4     | squash CI fixups before merge | 5 min |
| 5     | one-line rollback note | 2 min |

Total added cost on a 1.2k-LOC PR: roughly 25 minutes. That is the budget
the retro is buying.

## Recommendation: ownership model

See NOD-385 comment for the canonical statement. In short:

- **Author** owns filling the checklist. Self-label `high-churn`.
- **Staff Engineer** owns enforcement. Auto-requested on `high-churn`.
  Blocking review until every box is checked or `n/a`-justified.
- **Release Engineer** owns the merge gate. No `high-churn` PR merges
  without Staff approval **regardless of CI state**.
- **CTO** owns the rule itself. Adjusts thresholds and slice list as the
  retro evidence evolves; reviews this checklist quarterly.
