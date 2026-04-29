# Operator Probe Lane

Status: live (NOD-237). Owner: Release Engineer. Oversight: CTO.

The operator probe lane is the only sanctioned credentialed surface for
running read + audit-marker probes against prod. It exists to retire
the loop where an audit-log fix routes through a board interaction →
CEO dispatch → human paste-and-run in the Render shell (NOD-195 →
NOD-235).

## What the lane gives you

- A one-button trigger via GitHub Actions `workflow_dispatch` callable
  by board / CTO without DB shell access.
- A registered probe set executed against prod with credentials that
  never leave the CI runner.
- Captured output auto-posted as a comment on a Paperclip issue.
- An `operator_probe_run_started` / `operator_probe_run_completed`
  audit trail in `audit_log` keyed by `probe_run_id` and probe-set hash.

## What the lane does **not** give you

- Destructive prod actions. The DB role (`probe_runner`) has SELECT on
  all tables and INSERT-only on `audit_log`. UPDATE / DELETE / TRUNCATE
  are explicitly revoked in `backend/migrations/044_operator_probe_lane.sql`.
- Self-service for arbitrary agents. The trigger requires GitHub Actions
  write access on the repo, gated to the `operator-probe-prod` /
  `operator-probe-staging` deployment environments.
- Replacement for the SOC2 audit log in normal application code paths.

## How to trigger a probe

1. Open `Actions` → `Operator Probe Lane` in GitHub.
2. Click `Run workflow` and pick:
   - `probe_name` — registered probe (default `audit-log-health`).
   - `target_issue` — Paperclip identifier (e.g. `NOD-195`) or UUID
     that should receive the evidence comment.
   - `target_env` — `prod` or `staging`.
   - `dry_run` — `true` to validate without writing to `audit_log` or
     posting a comment.
3. The job runs against the env's secrets and posts the evidence comment
   automatically. Both the started and completed audit rows include the
   `probe_run_id` so you can correlate them with the captured output.

## How to register a new probe set

1. Add a new script under `scripts/operator-probes/<probe-name>.sh`.
   - The script must be self-contained and idempotent.
   - It must be **read + audit_log marker-insert only**. Anything else
     does not belong in this lane and must be implemented as a regular
     application code path with proper review.
   - It must accept its run id via `PROBE_RUN_ID` env and stamp the
     marker rows it inserts with that id.
   - Stdout is the evidence; stderr is for diagnostics. The runner
     captures stdout into the comment body and the workflow artifact.
2. Add the probe name to the `probe_name` choice list in
   `.github/workflows/operator-probe.yml`. This keeps the trigger
   surface explicit and prevents accidental dispatch to ad-hoc scripts.
3. If the probe needs new SQL access (a non-default-public table), add
   a follow-up migration that grants only the minimum new privilege to
   `probe_runner`. Do not widen `audit_log` privileges beyond INSERT.
4. Smoke-test locally:
   ```bash
   PROBE_DRY_RUN=true \
   PROBE_NAME=<probe-name> \
   TARGET_ISSUE=NOD-XXX \
   DATABASE_URL=postgresql://... \
   PAPERCLIP_API_URL=http://localhost:3100 \
   PAPERCLIP_API_KEY=... \
   ./scripts/operator-probe-runner.sh
   ```
   `PROBE_DRY_RUN=true` skips the audit insert and the Paperclip post
   so you can verify probe output without polluting the audit log.

## Required GitHub secrets and environments

The workflow expects two GitHub Environments named
`operator-probe-prod` and `operator-probe-staging`. Each must define:

- `PROBE_RUNNER_DATABASE_URL` — libpq URL using the `probe_runner`
  role provisioned by migration `044_operator_probe_lane.sql`. The
  password is rotated out-of-band and never appears in the repo.
- `PAPERCLIP_API_URL` — base URL of the Paperclip control plane.
- `PAPERCLIP_PROBE_API_KEY` — Paperclip API token scoped to comment
  creation on the relevant issues.

Apply protected-environment rules so dispatch requires reviewer
approval (CTO + one Release Engineer) before secrets are exposed to the
runner.

## Audit trail

Every run writes two rows to `audit_log`:

| stage         | action                          | event_type     | severity | details (jsonb keys) |
| ------------- | ------------------------------- | -------------- | -------- | -------------------- |
| start         | `operator_probe_run_started`    | `system_event` | `info`   | `probe_run_id`, `probe_name`, `probe_hash`, `actor`, `target_env`, `target_issue`, `stage`, `origin` |
| end (success) | `operator_probe_run_completed`  | `system_event` | `info`   | same shape + `exit_code` |
| end (failure) | `operator_probe_run_failed`     | `system_event` | `info`   | same shape + non-zero `exit_code` |

The probe set itself may write additional rows (e.g.
`audit_schema_smoke_probe` from `audit-log-health`). Any new probe must
likewise tag its inserts with `probe_run_id` so the trail is
reconstructable.

## Fallback (manual paste in Render shell)

The probe scripts are intentionally pasteable so a human board
operator can execute them when the lane itself is broken or
inaccessible. The acceptance runbook for NOD-195 lives in
[NOD-235 comment 42bb1697](/NOD/issues/NOD-235#comment-42bb1697-8d03-4608-bdde-6c513d5fa60b);
the equivalent here is:

```bash
DATABASE_URL=$DATABASE_URL bash scripts/audit-log-schema-probe.sh
```

When the lane is healthy, prefer the GitHub Actions trigger so the
audit trail is automatic.
