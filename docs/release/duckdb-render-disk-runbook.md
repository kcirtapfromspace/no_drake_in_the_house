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

## Writer-path status ([NOD-253](/NOD/issues/NOD-253) shipped)

The application-side writer is now plumbed end-to-end. Two production
entry points construct a file-backed `DuckDbClient` against
`DUCKDB_PATH`:

- `run_duckdb_startup_probe` in `backend/src/runtime.rs` runs
  `DuckDbClient::new` at boot (analytics/monolith modes) and crashloops
  the service on any open / `initialize_schema` / `record_sync_metrics`
  / count-verify failure. On Render that surfaces as the analytics
  service failing its rollout health check rather than silently
  writing nowhere.
- `POST /api/v1/analytics/duckdb/probe-write` in
  `backend/src/handlers/analytics_v2.rs` exposes the same writer for
  on-demand operator use (see §4a).

Net effect on this runbook:

- Probes 1–3 and 5 (env, mount, file-state-before, persistence) are
  unchanged and must pass.
- Probe 4 (`analytics.duckdb` materializes after an analytics write)
  is now meaningful and is satisfied either by the startup writer
  probe or by hitting the §4a endpoint.

Tracking: live-prod durability for [NOD-192](/NOD/issues/NOD-192) is
closed once both lanes ship — NOD-252 (Render runtime: this runbook)
and [NOD-253](/NOD/issues/NOD-253) (application writer-path) — per the
CTO HIGH 1 scope split.

To verify the writer path is still wired against any checkout:

```bash
# Expect: matches in runtime.rs (run_duckdb_startup_probe) and
# analytics_v2.rs (probe-write handler), beyond #[cfg(test)] usage.
git grep -nE 'DuckDbClient::(new|open)' backend/
git grep -n 'DuckDbClient::in_memory' backend/
```

If the first command stops returning non-test matches, the write-path
has regressed — restore the writer wiring before re-running probe 4.

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

### 3. DB file state before the service boots on a fresh disk

```bash
ls -lah "$DUCKDB_PATH" || true
```

Expected on a fresh disk **before the analytics service has booted**:
`ls: cannot access ...: No such file or directory`. The `|| true`
keeps the probe non-fatal so we still get a deterministic before/after
pair against §4.

Once the service has booted, the startup writer probe (§4) creates
`analytics.duckdb` on first run, so this "missing file" expectation
only holds against a brand-new disk that has never had the service
running on top of it.

### 4. `analytics.duckdb` materializes after the startup writer probe

The `run_duckdb_startup_probe` task in `backend/src/runtime.rs` opens
`DUCKDB_PATH` and writes a row through `record_sync_metrics` on every
boot in analytics/monolith modes. After a successful Render rollout,
the file must exist on the disk:

```bash
ls -lah "$DUCKDB_PATH"
stat -c '%y %s %n' "$DUCKDB_PATH"
```

Expected: a non-zero `analytics.duckdb` owned by the runtime
user/group. If it is missing or zero-byte after a clean boot, the
service either crashlooped on the startup probe or failed to find the
mount — check the Render service logs for `DuckDB analytics storage
probe passed` (storage validation, NOD-196) and the writer-probe
success log line referenced in §4a, then inspect the disk attachment
in the Render dashboard.

If you need an additional sanity check that the runtime UID/GID can
write to the mount path itself (independent of DuckDB), the legacy
Render-shell write probe is still safe to run as a belt-and-braces
disk-health check:

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

### 4a. Force a deterministic writer probe (single API action)

Use this when you need an immediate proof write without waiting for
background traffic or a fresh rollout:

```bash
curl -sS -X POST "https://<api-host>/api/v1/analytics/duckdb/probe-write" \
  -H "Authorization: Bearer <admin-jwt>" \
  -H "Content-Type: application/json"
```

Expected response:

- `"success": true`
- `"data.runbook_probe_rows"` increments on repeated calls
- service logs include `DuckDB writer runbook probe succeeded`

### 5. Persistence across rollout

Persistence is now verified against `analytics.duckdb` itself, since
the startup writer probe materializes (and the §4a endpoint can extend)
the real DB file on the Render disk. Capture `stat` on the file before
the rollout, redeploy `ndith-analytics` from the Render dashboard or
CLI (manual deploy, not just a restart), and re-`stat` once the new
instance is healthy:

```bash
stat -c '%y %s %n' "$DUCKDB_PATH"
# ... trigger Render rollout and wait for healthy ...
ls -lah "$DUCKDB_PATH"
stat -c '%y %s %n' "$DUCKDB_PATH"
```

Expected: the `analytics.duckdb` file survives the rollout — same
inode-stable mtime/size or a strictly larger size if the new instance's
startup writer probe (§4) appended a row. A missing or zero-byte file
on the new instance indicates the `ndith-analytics-duckdb` disk is not
persisting — verify in the Render dashboard that the disk is attached
and bound to the service across deploys.

For a belt-and-braces check that does not depend on the DB file, drop
a `.persist_probe` marker in `$(dirname "$DUCKDB_PATH")` alongside the
`stat` calls; it should also survive the rollout.

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
  With the writer path now live ([NOD-253](/NOD/issues/NOD-253)), set
  a Render dashboard alert when the disk reaches 70 % to leave headroom
  for compaction.
- **Backup:** **No backup story exists today.** The DuckDB file is a
  live single point of data loss now that the writer path
  ([NOD-253](/NOD/issues/NOD-253)) ships real rows on every boot. This
  is acceptable as a starter posture for a sync-metrics OLAP store
  (lossy is okay; analytics can be back-filled from PostgreSQL), but
  it must be tracked. Schedule a `pg_dump`-equivalent snapshot job
  (e.g. nightly `EXPORT DATABASE` to S3) before the table retention
  exceeds what we can re-derive from Postgres — track as a follow-up
  ticket on top of NOD-252 / [NOD-253](/NOD/issues/NOD-253).
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
