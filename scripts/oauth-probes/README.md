# OAuth Synthetic Probes

Definitions and runner for deterministic OAuth synthetic probes across
Spotify, Apple, and Tidal.

The same entrypoint (`scripts/oauth-probes/run.mjs`) is used for both
ad-hoc dry-runs and scheduled execution. Scheduled mode is wired via
GitHub Actions and produces an identical output schema.

## Paths

- Definitions: `scripts/oauth-probes/definitions.json`
- Runner: `scripts/oauth-probes/run.mjs`
- Schedule config: `.github/workflows/oauth-synthetic-probes.yml`
- Evidence artifacts (default): `docs/evidence/oauth-synthetic-probes/latest.{json,ndjson,md}`
- Last-success state file (runtime): `data/oauth-synthetic-probe-state.json`

## One-shot dry-run command

```bash
npm run probe:oauth -- --dry-run
```

Single provider:

```bash
npm run probe:oauth -- --dry-run --provider spotify
```

## Scheduled execution

The workflow at `.github/workflows/oauth-synthetic-probes.yml` runs
every 15 minutes and invokes the same `npm run probe:oauth` entrypoint.
Probe artifacts are uploaded as a workflow artifact named
`oauth-synthetic-probes-<run-id>` and retained for 30 days.

Manual trigger from GitHub UI: Actions → "OAuth Synthetic Probes" →
Run workflow.

## Required environment variables

The runner is fully deterministic and does not call live providers, so
no secrets or environment variables are required at runtime. The probe
runner does not access the network in any flow class today.

If/when real-provider probing is introduced, document the required
secrets here and surface them through the workflow `env:` block.

## Output contract (shared by dry-run and scheduled mode)

Each result record always includes the required fields:

- `provider` — `spotify | apple | tidal`
- `flow` — provider-side flow under test (e.g. `oauth_login_callback`)
- `class` — failure class (`login_callback_success`,
  `token_refresh_failure_class`, `provider_unavailable_timeout`)
- `last_success` — last timestamp this probe passed (ISO-8601)
- `status` — `pass | fail`
- `timestamp` — when this record was produced (ISO-8601)

Plus diagnostic fields: `probe_id`, `simulation`, `simulation_label`,
`details`. The wrapper JSON also includes `generated_at`, `dry_run`,
`provider_target`, `definitions_path`, `state_path`, and `probe_count`.
