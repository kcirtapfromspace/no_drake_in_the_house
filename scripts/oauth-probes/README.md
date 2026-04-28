# OAuth Synthetic Probes

Deterministic OAuth synthetic probes across Spotify, Apple, and Tidal.

Two execution modes share a single source of truth in
[`convex/lib/oauthSyntheticProbes.ts`](../../convex/lib/oauthSyntheticProbes.ts):
the probe definitions, the canonical classifier, and the record
builder. Both modes emit records satisfying the locked output contract
(provider, flow, class, last_success, status, timestamp).

- **One-shot dry-run** — local CLI runner at `scripts/oauth-probes/run.ts`.
- **Scheduled** — Convex cron at `convex/crons.ts` invoking
  `internal.oauthSyntheticProbes.runProbes` (defined in
  `convex/oauthSyntheticProbes.ts`).

## Paths

- Shared definitions/classifier: `convex/lib/oauthSyntheticProbes.ts`
- CLI runner: `scripts/oauth-probes/run.ts`
- Convex action: `convex/oauthSyntheticProbes.ts`
- Schedule config: `convex/crons.ts` → entry `run-oauth-synthetic-probes`
- CLI evidence artifacts (default): `docs/evidence/oauth-synthetic-probes/latest.{json,ndjson,md}`
- CLI last-success state file (runtime): `data/oauth-synthetic-probe-state.json`
- Convex last-success state: `oauthSyntheticProbeState` table (indexed by `probeId`)

## How status is decided

`status` is **not** hard-coded. Each probe definition carries a
deterministic `signal` that the runner feeds through
`classifyOAuthSignal`. If the classifier returns the probe's expected
`class`, status is `"pass"`. Any drift (classifier change, signal
mismatch) flips status to `"fail"` — that flip is the only SLO signal
these synthetic probes carry. The Rust counterpart at
`backend/crates/ndith-core/src/error/oauth.rs` is the production-path
classifier; the shared TS classifier mirrors it for the synthetic
classes the probes cover.

## One-shot dry-run command

```bash
npm run probe:oauth
```

Single provider:

```bash
npm run probe:oauth -- --provider spotify
```

The CLI runner writes a `latest.json` / `latest.ndjson` / `latest.md`
triplet containing one record per definition. Paths in the JSON
artifact are repo-relative (no machine-local absolute paths leak).

## Scheduled execution

The cron entry `run-oauth-synthetic-probes` in `convex/crons.ts` runs
every 15 minutes and calls `internal.oauthSyntheticProbes.runProbes`
with `{ provider: "all" }`. The action persists `last_success` per
probe in the `oauthSyntheticProbeState` table so the field survives
across runs (matching CLI semantics). It emits one structured log line
per probe (prefix `oauth_synthetic_probe`) plus a single run summary
line (prefix `oauth_synthetic_probe_run`).

Manual one-off invocation:

```bash
npx convex run --internal oauthSyntheticProbes:runProbes '{"provider":"all"}'
```

To tail scheduled output:

```bash
npx convex logs --follow | grep oauth_synthetic_probe
```

## Required environment variables

- CLI runner: none. The runner is fully deterministic and does not call
  live providers.
- Convex action: none beyond the existing Convex deployment env (e.g.
  `CONVEX_DEPLOY_KEY` for CI deploys, already set up by
  `.github/workflows/convex-deploy.yml`).

If/when real-provider probing is introduced, add the required secrets
to the Convex deployment environment and document them here.

## Output contract

Every result record always includes the required fields enforced by
`REQUIRED_RESULT_FIELDS` in the shared module and asserted by the CLI
on every run:

- `provider` — `spotify | apple | tidal`
- `flow` — provider-side flow under test (e.g. `oauth_login_callback`)
- `class` — failure class (`login_callback_success`,
  `token_refresh_failure_class`, `provider_unavailable_timeout`)
- `last_success` — last timestamp this probe passed (ISO-8601), or
  `null` if it has never passed
- `status` — `pass | fail`, derived by the canonical classifier
- `timestamp` — when this record was produced (ISO-8601)

Plus diagnostic fields: `probe_id`, `simulation`, `simulation_label`,
`details` (which includes `expected_classification`,
`actual_classification`, and the deterministic `signal`). The wrapper
payload also includes `generated_at`, `provider_target`, and
`probe_count`; the CLI variant additionally records `output_path` and
`state_path` (always repo-relative).
