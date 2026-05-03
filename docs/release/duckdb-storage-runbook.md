# DuckDB Runtime Storage Runbook (NOD-196)

This runbook covers the persistent DuckDB analytics store for the
`ndith-api` backend in production.

> **Scope (locked by CTO under [NOD-196](/NOD/issues/NOD-196) +
> [NOD-252](/NOD/issues/NOD-252) split):** This runbook documents
> **K8s-side infrastructure readiness** for DuckDB persistence — env,
> volumeMount, PVC, fail-fast validation. **Live production durability
> closure for [NOD-192](/NOD/issues/NOD-192) is owned in [NOD-252](/NOD/issues/NOD-252)
> (Render runtime) plus [NOD-253](/NOD/issues/NOD-253)** (application
> writer-path enablement, shipped — see "Writer-path status" below).

## Writer-path status (NOD-253 shipped)

The application-side writer is now plumbed end-to-end. Two production
entry points construct a file-backed `DuckDbClient` against
`DUCKDB_PATH`:

- `run_duckdb_startup_probe` in `backend/src/runtime.rs` runs
  `DuckDbClient::new` at boot (analytics/monolith modes) and crashloops
  the pod on any open / `initialize_schema` / `record_sync_metrics` /
  count-verify failure.
- `POST /api/v1/analytics/duckdb/probe-write` in
  `backend/src/handlers/analytics_v2.rs` exposes the same writer for
  on-demand operator use (see §4a).

Net effect on this runbook:

- Probes 1–3 and 5 (PVC bound, env present, mount writable, persistence
  across pod restart) are unchanged and must pass.
- Probe 4 (`analytics.duckdb` materializes after an analytics write) is
  now meaningful and is satisfied either by the startup writer probe
  or by hitting the §4a endpoint.

To verify the writer path is still wired against any checkout:

```bash
# Expect: matches in runtime.rs (run_duckdb_startup_probe) and
# analytics_v2.rs (probe-write handler), beyond #[cfg(test)] usage.
git grep -nE 'DuckDbClient::(new|open)' backend/
git grep -n 'DuckDbClient::in_memory' backend/
```

If the first command stops returning non-test matches, the write-path
has regressed — restore the writer wiring before re-running probe 4.

## Canonical configuration

| Field             | Value                                          |
| ----------------- | ---------------------------------------------- |
| `DUCKDB_PATH`     | `/var/lib/ndith/analytics/analytics.duckdb`    |
| Mount path        | `/var/lib/ndith/analytics`                     |
| PVC name (K8s)    | `ndith-duckdb-data`                            |
| PVC name (Helm)   | `{release}-backend-duckdb`                     |
| Access mode       | `ReadWriteOnce`                                |
| StorageClass      | `fast-ssd`                                     |
| Requested size    | `20Gi` (raw K8s and Helm production)           |

The path is sourced from `k8s/configmap.yaml` (raw manifests) and
`backend.env.DUCKDB_PATH` in `helm/values.yaml` /
`helm/values-production.yaml`. Both render identically into the pod.

## Pre-deploy probe (Helm dry-run)

```bash
helm template ndith ./helm -f helm/values-production.yaml \
  | grep -A2 -E 'DUCKDB_PATH|duckdb-data|claimName|mountPath: /var/lib/ndith'
```

Expected: `DUCKDB_PATH` env var present on the backend Deployment, a
`duckdb-data` volumeMount at `/var/lib/ndith/analytics`, and a
`PersistentVolumeClaim` with `claimName` matching
`{release}-backend-duckdb`.

## Post-deploy probes

Run after every production rollout that touches the backend or analytics
storage. All commands assume `-n ndith-production`.

### 1. PVC is bound

```bash
kubectl get pvc ndith-duckdb-data -n ndith-production \
  -o jsonpath='{.status.phase}{"\n"}'
```

Expected: `Bound`.

### 2. Pod has DUCKDB_PATH and the volumeMount

```bash
POD=$(kubectl get pod -n ndith-production -l app=ndith-api \
  -o jsonpath='{.items[0].metadata.name}')

kubectl exec -n ndith-production "$POD" -- printenv DUCKDB_PATH
kubectl exec -n ndith-production "$POD" -- mount | grep /var/lib/ndith/analytics
```

Expected: `DUCKDB_PATH=/var/lib/ndith/analytics/analytics.duckdb` and a
mounted volume rooted at `/var/lib/ndith/analytics`.

### 3. Mount is writable by the runtime UID/GID

```bash
kubectl exec -n ndith-production "$POD" -- ls -ld /var/lib/ndith/analytics
kubectl exec -n ndith-production "$POD" -- id
kubectl exec -n ndith-production "$POD" -- sh -c \
  'touch /var/lib/ndith/analytics/.runbook_probe && \
   ls -l /var/lib/ndith/analytics/.runbook_probe && \
   rm /var/lib/ndith/analytics/.runbook_probe'
```

Expected: the directory is owned by group `2000` (the pod's `fsGroup`),
the runtime UID is `1000`, and the touch/ls/rm cycle succeeds.

### 4. `analytics.duckdb` materializes after the startup writer probe

The `run_duckdb_startup_probe` task in `backend/src/runtime.rs` opens
`DUCKDB_PATH` and writes a row through `record_sync_metrics` on every
boot in analytics/monolith modes. After a successful rollout, the
file must exist on the PVC:

```bash
kubectl exec -n ndith-production "$POD" -- ls -lh /var/lib/ndith/analytics
```

Expected: the directory listing includes a non-zero
`analytics.duckdb`. If it is missing or zero-byte after a clean boot,
the pod either crashlooped on the startup probe or failed to find the
mount — check the backend logs for `DuckDB analytics storage probe
passed` (storage validation) and the writer-probe success log line
referenced in §4a, then inspect PVC binding and pod
`fsGroup`/`runAsUser`.

If you need an additional sanity check that the runtime UID/GID can
write to the mount path itself (independent of DuckDB), the legacy
`kubectl exec` write probe is still safe to run:

```bash
kubectl exec -n ndith-production "$POD" -- sh -c '
  TS=$(date -u +%Y%m%dT%H%M%SZ)
  echo "k8s-runbook-probe $TS" > "/var/lib/ndith/analytics/.runbook_probe_$TS"
  ls -lah "/var/lib/ndith/analytics/.runbook_probe_$TS"
  cat "/var/lib/ndith/analytics/.runbook_probe_$TS"
  rm "/var/lib/ndith/analytics/.runbook_probe_$TS"
'
```

Expected: `echo` writes the file, `ls` shows it owned by the runtime
user with non-zero size, `cat` returns the timestamp string, `rm`
removes it.

### 4a. Force a deterministic writer probe (single API action)

Use this when you need an immediate proof write without waiting for
background traffic.

```bash
curl -sS -X POST "https://<api-host>/api/v1/analytics/duckdb/probe-write" \
  -H "Authorization: Bearer <admin-jwt>" \
  -H "Content-Type: application/json"
```

Expected response:

- `"success": true`
- `"data.runbook_probe_rows"` increments on repeated calls
- backend logs include `DuckDB writer runbook probe succeeded`

### 5. Persistence across pod restart

Persistence is now verified against `analytics.duckdb` itself, since
the startup writer probe materializes (and the §4a endpoint can extend)
the real DB file on the PVC. `stat` it, restart the deployment, and
confirm the file survives:

```bash
kubectl exec -n ndith-production "$POD" -- stat \
  -c '%y %s %n' /var/lib/ndith/analytics/analytics.duckdb
kubectl rollout restart deployment/ndith-api -n ndith-production
kubectl rollout status deployment/ndith-api -n ndith-production --timeout=300s
NEW_POD=$(kubectl get pod -n ndith-production -l app=ndith-api \
  -o jsonpath='{.items[0].metadata.name}')
kubectl exec -n ndith-production "$NEW_POD" -- stat \
  -c '%y %s %n' /var/lib/ndith/analytics/analytics.duckdb
```

Expected: the `analytics.duckdb` file survives the rollout — same
inode-stable mtime/size or a strictly larger size if the new pod's
startup writer probe appended a row. A missing or zero-byte file on
the new pod indicates a non-persistent volume binding.

For a belt-and-braces check that does not depend on the DB file, drop
a `.persist_probe` marker on the PVC alongside the `stat` calls; it
should also survive the rollout.

## Failure modes the backend now catches at startup

The backend runs `ensure_duckdb_storage_ready(mode)` early in
`run_service` (see `backend/src/runtime.rs`). For
`NDITH_SERVICE_MODE=monolith` and `analytics`, it will exit with
non-zero and log `DuckDB analytics storage validation failed: ...` if:

- `DUCKDB_PATH` is unset or empty.
- The parent directory cannot be created.
- The parent directory is not writable by the runtime UID/GID (e.g.
  missing `fsGroup`, wrong PVC permissions, read-only mount).

For `api`, `graph`, and `news` modes the probe is non-fatal and emits a
warning instead, since those modes are not expected to write the
analytics store.

## Known overrides

- Local development (`docker-compose`, Tilt): `DUCKDB_PATH` defaults to
  `./data/analytics.duckdb` via `.env.example`. Keep the same env name
  to avoid drift from production.
- If a deployment manifest changes the mount path, update both
  `DUCKDB_PATH` and the `volumeMounts.mountPath` together; they must
  agree on the parent directory.
