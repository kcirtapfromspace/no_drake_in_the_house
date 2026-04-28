# OAuth Release Gate

Deterministic pre-merge check that fails on OAuth probe / SLO threshold breaches.
Tracking: NOD-165 (gate), NOD-146 (parent: probes + SLO alerts), NOD-142 (program).

## What it does

`scripts/oauth-release-gate.sh <metrics.json>` evaluates a JSON snapshot of OAuth
provider metrics against the NOD-164 SLO contract. Exit code 0 means PASS or
PASS-with-P2-warnings; exit code 1 means BLOCKED (a P1 violation was found, the
metrics file was missing/invalid, or required keys are absent).

The gate is fail-closed by design: any of these blocks the merge:

- missing or unreadable metrics file
- invalid JSON
- missing top-level `providers` or `global` keys
- a metric required by a rule is missing for a provider (treated as a P1 violation)
- any P1 rule comparison fires

## Thresholds (NOD-164 SLO contract)

| Code | Metric | Comparator | Threshold | Window | Priority |
|------|--------|------------|-----------|--------|----------|
| OAUTH-ERR-001 | error_rate | > | 0.05 | 5min | P1 |
| OAUTH-ERR-002 | failed_auth_count | > | 10 | 5min | P1 |
| OAUTH-ERR-003 | failed_system_count (global) | > | 1 | 1min | P1 |
| OAUTH-ERR-004 | failed_provider_streak | >= | 3 | 10min | P1 |
| OAUTH-ERR-005 | rate_limited_duration_min | > | 15 | 15min | P1 |
| OAUTH-LAT-001 | callback_p95_ms | > | 3000 | 5min | P2 |
| OAUTH-LAT-002 | refresh_p95_ms | > | 2000 | 5min | P2 |
| OAUTH-LAT-003 | probe_p95_ms | > | 5000 | 5min | P2 |
| OAUTH-PRB-001 | probe_consecutive_fail | >= | 2 | sched | P2 |
| OAUTH-PRB-002 | probe_consecutive_fail | >= | 3 | sched | P1 |
| OAUTH-BUD-001 | callback_budget_burn | > | 6.0 | 1h | P1 |
| OAUTH-BUD-002 | refresh_budget_burn | > | 6.0 | 1h | P1 |

## Pipeline wiring

`.github/workflows/oauth-release-gate.yml` runs the gate on every PR to `main`
that touches OAuth handler/service/model code or the gate itself. The workflow
currently runs the smoke fixture (`scripts/fixtures/oauth-metrics-pass.json`).
Once the NOD-163 probe runner is producing live snapshots, replace the fixture
path with the published probe artifact.

## Fixtures

- `scripts/fixtures/oauth-metrics-pass.json` — all providers within thresholds.
- `scripts/fixtures/oauth-metrics-fail.json` — Spotify breaches `error_rate`,
  `failed_auth_count`, and `callback_budget_burn`.

## Release engineer usage

Run locally before opening or merging an OAuth PR:

```bash
./scripts/oauth-release-gate.sh scripts/fixtures/oauth-metrics-pass.json   # exit 0
./scripts/oauth-release-gate.sh scripts/fixtures/oauth-metrics-fail.json   # exit 1
```

When the gate blocks a merge:

1. Read the `[CODE] PRIORITY scope/metric = value (threshold: X, window)` lines.
2. Triage each P1 violation against the live probe data referenced by NOD-146.
3. Either fix the regression on the branch and rerun the gate, or escalate to
   the Staff Engineer with the P1 codes for an explicit waiver — never bypass.

P2 warnings do not block merge but should be filed back to the OAuth program
(NOD-142) for follow-up.

## Handoff to Staff review

This gate is the deterministic merge guard for OAuth changes. Staff Engineer
review (per NOD-142 success criteria) is required for any change that:

- modifies the rules table in `scripts/oauth-release-gate.sh`
- changes the workflow path filter in `.github/workflows/oauth-release-gate.yml`
- introduces a new fixture intended to bypass a rule
- swaps the metrics input from the NOD-163 probe runner to any other source
