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
| `NDITH_SERVICE_MODE` | `analytics`                                 |

The values are sourced from `render.yaml`. The mount path matches the
parent directory of `DUCKDB_PATH` so the runtime can create the file
under a writable Render disk. The path also matches the K8s/Helm
canonical path so a single backend image works on both runtimes
without per-environment overrides.

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

### 4. Trigger one analytics write, then re-check

Pick the smallest deterministic write surface available. Options in
order of preference:

1. Hit a scoped analytics endpoint that records sync metrics, e.g.
   `curl -s -X POST http://localhost:$PORT/internal/analytics/record-sync-probe`
   if the route is wired.
2. Run `cargo run --bin analytics-write-probe` if a probe binary is
   shipped in the image.
3. Trigger one upstream sync from `ndith-backend` that fans out to
   `ndith-analytics` via the existing service path.

After the write returns success:

```bash
ls -lah "$DUCKDB_PATH"
```

Expected: `analytics.duckdb` exists, size > 0. If the file is still
absent, re-check `printenv DUCKDB_PATH` and the analytics service log
for the `DuckDB analytics storage probe passed` startup line (added in
NOD-196). If it never logged, `DUCKDB_PATH` is being overridden
somewhere or the analytics image is not running in `analytics` mode.

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

## Known overrides

- Local development (`docker-compose`, Tilt): `DUCKDB_PATH` defaults to
  `./data/analytics.duckdb` via `.env.example`. Keep the same env name
  to avoid drift from production.
- The Render disk is per-service. Do not mount the same disk on more
  than one service; DuckDB takes a process-exclusive lock on the file.
- If a future deploy changes the mount path, update both `DUCKDB_PATH`
  and `disk.mountPath` together; they must agree on the parent
  directory.
