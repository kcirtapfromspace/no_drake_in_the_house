# Render Persistent DuckDB Disk Runbook (NOD-252)

This runbook covers the persistent DuckDB analytics store for the
`ndith-analytics` Render service in production. It is the Render-side
counterpart to the Kubernetes/Helm runbook tracked under
[NOD-196](../../tasks). Out of scope: K8s manifests and the
`ndith-production` cluster.

## Canonical configuration

| Field             | Value                                          |
| ----------------- | ---------------------------------------------- |
| Render service    | `ndith-analytics`                              |
| `DUCKDB_PATH`     | `/var/lib/ndith/analytics/analytics.duckdb`    |
| Mount path        | `/var/lib/ndith/analytics`                     |
| Disk name         | `ndith-analytics-duckdb`                       |
| Disk size         | `10 GB`                                        |
| `NDITH_SERVICE_MODE` | `analytics` (diagnostic only)               |

The values are sourced from `render.yaml`. The mount path matches the
parent directory of `DUCKDB_PATH` so the runtime can create the file
under a writable Render disk. The path also matches the K8s/Helm
canonical path so a single backend image works on both runtimes
without per-environment overrides.

> **Note on `NDITH_SERVICE_MODE`** — `backend/src/bin/ndith-analytics-service.rs`
> hard-codes `run_service(ServiceMode::Analytics)`, so this env var is
> not read by the analytics binary. It is shipped as a diagnostic
> annotation only, so an SRE inspecting the service env can see the
> intended mode at a glance. Do **not** treat it as a runtime override
> — flipping it on the analytics service has no effect.

## Known production gap (Staff review HIGH 1)

As of this runbook's commit, **no production code path constructs a
file-backed `DuckDbClient` against `DUCKDB_PATH`**. `grep -rn
'DuckDbClient::(new|open)' backend/` returns zero hits outside
`#[cfg(test)]` calls to `DuckDbClient::in_memory()`. The startup
`ensure_duckdb_storage_ready` probe (NOD-196) validates the directory
is writable but never opens the DB file.

Net effect on this runbook:

- Probes 1–3 and 5 (env, mount, file-state-before, persistence) are
  meaningful and must pass.
- Probe 4 (file appears after an analytics write) **cannot pass with
  current code**. The "trigger one analytics write" step is
  intentionally written below as a writability proof, not a production
  write proof, until the writer wiring lands.

Tracking: write-path wiring is the live-prod durability gap on
[NOD-192](/NOD/issues/NOD-192). The CTO scope split (HIGH 1 decision)
locked this as Option (B): **NOD-252 stays the infra-readiness lane,
and application writer-path enablement is owned in
[NOD-253](/NOD/issues/NOD-253) "Implement production DuckDB writer
path in ndith-analytics-service".** When [NOD-253](/NOD/issues/NOD-253)
ships, replace probe 4 below with the real production write trigger
from that issue's runbook section and re-run the full sequence on a
redeploy.

## Pre-deploy checks (run from a workstation)

```bash
grep -A4 'name: ndith-analytics' render.yaml \
  | grep -E 'disk:|name:|mountPath:|sizeGB:|DUCKDB_PATH|NDITH_SERVICE_MODE' -A1
```

Expected: `disk:` block with `name: ndith-analytics-duckdb`,
`mountPath: /var/lib/ndith/analytics`, `sizeGB: 10`, plus env vars
`NDITH_SERVICE_MODE=analytics` and
`DUCKDB_PATH=/var/lib/ndith/analytics/analytics.duckdb`.

## Post-deploy probes (run from the Render shell on `ndith-analytics`)

Run after every production rollout that touches `render.yaml` or the
`ndith-backend:analytics-latest` image. Capture the raw stdout/stderr
of every command and paste it verbatim into the issue tracker.

### 1. Env var presence

```bash
printenv DUCKDB_PATH
```

Expected: `/var/lib/ndith/analytics/analytics.duckdb`. Empty output
means the env var is missing — check Render env settings before
continuing.

### 2. Mount directory exists and is writable

```bash
ls -lah "$(dirname "$DUCKDB_PATH")"
```

Expected: directory listing for `/var/lib/ndith/analytics` with the
runtime user/group having write access. The directory itself is the
disk mount point so it must exist immediately on first boot.

### 3. DB file state before first analytics write

```bash
ls -lah "$DUCKDB_PATH" || true
```

Expected on a fresh disk: `ls: cannot access ...: No such file or
directory`. The `|| true` keeps the probe non-fatal so we still get a
deterministic before/after pair.

### 4. Writability proof (interim — see "Known production gap")

Until a real production writer lands (see "Known production gap"
above), use a Render-shell write probe to prove the disk is writable
by the runtime UID/GID. **This proves the disk works, not that
analytics writes are persisted.**

```bash
TS=$(date -u +%Y%m%dT%H%M%SZ)
echo "render-disk-probe $TS" > "$(dirname "$DUCKDB_PATH")/.runbook_probe_$TS"
ls -lah "$(dirname "$DUCKDB_PATH")/.runbook_probe_$TS"
cat "$(dirname "$DUCKDB_PATH")/.runbook_probe_$TS"
rm "$(dirname "$DUCKDB_PATH")/.runbook_probe_$TS"
```

Expected: `echo` writes the file, `ls` shows it owned by the runtime
user with non-zero size, `cat` returns the timestamp string, `rm`
removes it. Failure of any step means the mount is not writable —
inspect the disk attachment in the Render dashboard.

```bash
ls -lah "$DUCKDB_PATH" || true
```

Expected (current state, until [NOD-253](/NOD/issues/NOD-253) ships):
`No such file or directory`. The DuckDB file legitimately does not
exist because no production code opens it yet. Once
[NOD-253](/NOD/issues/NOD-253) lands a real writer, replace this
section with:

> Hit the writer trigger from [NOD-253](/NOD/issues/NOD-253); then
> `ls -lah "$DUCKDB_PATH"` returns a non-zero `analytics.duckdb` file.

### 5. Persistence across rollout

From the Render dashboard or CLI, redeploy `ndith-analytics` (manual
deploy, not just a restart). After the new instance is healthy:

```bash
ls -lah "$DUCKDB_PATH"
stat "$DUCKDB_PATH"
```

Expected: same file (mtime, size) as before the rollout. If size or
mtime reset to a fresh file, the disk is not persisting — verify in
the Render dashboard that the `ndith-analytics-duckdb` disk is
attached and bound to the service across deploys.

## Failure modes the backend catches at startup

The backend runs `ensure_duckdb_storage_ready(mode)` early in
`run_service` (see `backend/src/runtime.rs`, landed in NOD-196). For
`NDITH_SERVICE_MODE=analytics` it exits non-zero and logs
`DuckDB analytics storage validation failed: ...` if:

- `DUCKDB_PATH` is unset or empty.
- The parent directory cannot be created.
- The parent directory is not writable.

In Render, that surfaces as the analytics service repeatedly crash-
looping on deploy with the validation log line at the top of every
attempt. If you see that, fix the disk mount or env var before
re-deploying.

## Rollback

`ndith-analytics` now owns a Render disk, which forces deploys into a
**stop-then-start** rollout (Render disks attach to one running
instance at a time, and DuckDB takes a process-exclusive file lock so
multi-instance is unsafe anyway). There is a brief window during
rollout — typically 30–90 s — where `/analytics/*` returns 503. The
frontend's `ANALYTICS_UPSTREAM_URL` is wired to this service
(`render.yaml` `services` block), so the 503 is user-visible.

### When to roll back

Roll back if any of:

- The startup probe `ensure_duckdb_storage_ready` is logging
  `DuckDB analytics storage validation failed` after a deploy.
- 503s on `/analytics/*` persist past the normal rollout window.
- A regression is reported on the analytics surface.

### How to roll back

`render-backend-image.yml` already pushes immutable
`analytics-<short-sha>` tags on every successful build (alongside the
mutable `analytics-latest`). To pin to a known-good SHA:

1. Find the last good SHA from the `Publish Analytics Image` workflow
   run history, e.g. `analytics-7495642`.
2. In `render.yaml`, change the `ndith-analytics` service `image` to
   `ghcr.io/<org>/ndith-backend:analytics-7495642` (commit the change
   on a hotfix branch).
3. Render will stop the running instance, attach the disk to the new
   instance, and start it. Expect the same 503 window as a normal
   rollout.
4. Once stable, open a regular fix PR against `main` and re-pin to
   `analytics-latest` after the fix lands.

> **Follow-up:** pinning the analytics service to a SHA tag in
> `render.yaml` permanently (instead of the mutable `analytics-latest`)
> is the right long-term posture but is out of scope for NOD-252.
> Track in a separate ticket.

## Capacity & backup

- **Disk size:** 10 GiB starter. The DuckDB single-file size is
  expected to grow roughly with retention × (rows/day × avg row size).
  Once a real writer is in place, set a Render dashboard alert when
  the disk reaches 70 % to leave headroom for compaction.
- **Backup:** **No backup story exists today.** The DuckDB file is a
  single point of data loss the moment a real write path lands. This
  is acceptable as a starter posture for a sync-metrics OLAP store
  (lossy is okay; analytics can be back-filled from PostgreSQL), but
  it must be tracked. [NOD-253](/NOD/issues/NOD-253) (writer-path
  enablement) should also schedule a `pg_dump`-equivalent
  snapshot job (e.g. nightly `EXPORT DATABASE` to S3) before the table
  retention exceeds what we can re-derive from Postgres.
- **Monitoring:** add a Prometheus alert (or Render log alert) on the
  `DuckDB analytics storage validation failed` log line so a broken
  disk surfaces immediately rather than via empty dashboards.

## Known overrides

- Local development (`docker-compose`, Tilt): `DUCKDB_PATH` defaults to
  `./data/analytics.duckdb` via `.env.example`. Keep the same env name
  to avoid drift from production.
- The Render disk is per-service. Do not mount the same disk on more
  than one service; DuckDB takes a process-exclusive lock on the file.
- If a future deploy changes the mount path, update both `DUCKDB_PATH`
  and `disk.mountPath` together; they must agree on the parent
  directory.
