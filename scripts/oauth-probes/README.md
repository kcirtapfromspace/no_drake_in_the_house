# OAuth Synthetic Probes

Definitions and runner for deterministic OAuth synthetic probes across
Spotify, Apple, and Tidal.

Two execution modes share the same locked output schema (provider,
flow, class, last_success, status, timestamp):

- **One-shot dry-run** — local CLI runner at `scripts/oauth-probes/run.mjs`.
- **Scheduled** — Convex cron at `convex/crons.ts` invoking
  `internal.oauthSyntheticProbes.runProbes` (defined in
  `convex/oauthSyntheticProbes.ts`).

## Paths

- Definitions (CLI): `scripts/oauth-probes/definitions.json`
- CLI runner: `scripts/oauth-probes/run.mjs`
- Convex action: `convex/oauthSyntheticProbes.ts`
- Schedule config: `convex/crons.ts` → entry `run-oauth-synthetic-probes`
- CLI evidence artifacts (default): `docs/evidence/oauth-synthetic-probes/latest.{json,ndjson,md}`
- CLI last-success state file (runtime): `data/oauth-synthetic-probe-state.json`

## One-shot dry-run command

```bash
npm run probe:oauth -- --dry-run
```

Single provider:

```bash
npm run probe:oauth -- --dry-run --provider spotify
```

The CLI runner writes a `latest.json` / `latest.ndjson` / `latest.md`
triplet containing one record per definition.

## Scheduled execution

The cron entry `run-oauth-synthetic-probes` in `convex/crons.ts` runs
every 15 minutes and calls `internal.oauthSyntheticProbes.runProbes`
with `{ provider: "all" }`. The action mirrors the deterministic
execution logic in the CLI runner and emits one structured log line
per probe (prefix `oauth_synthetic_probe`) plus a single run summary
line (prefix `oauth_synthetic_probe_run`). The action also returns the
full payload to its caller.

Manual one-off invocation (e.g. via the Convex dashboard or `npx convex
run`) uses the same action:

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

## Output contract (shared by CLI dry-run and scheduled action)

Each result record always includes the required fields:

- `provider` — `spotify | apple | tidal`
- `flow` — provider-side flow under test (e.g. `oauth_login_callback`)
- `class` — failure class (`login_callback_success`,
  `token_refresh_failure_class`, `provider_unavailable_timeout`)
- `last_success` — last timestamp this probe passed (ISO-8601), or
  `null` if it has never passed within the run
- `status` — `pass | fail`
- `timestamp` — when this record was produced (ISO-8601)

Plus diagnostic fields: `probe_id`, `simulation`, `simulation_label`,
`details`. The wrapper payload also includes `generated_at`,
`dry_run`, `provider_target`, and `probe_count` (the CLI variant
additionally records `definitions_path` and `state_path`).
