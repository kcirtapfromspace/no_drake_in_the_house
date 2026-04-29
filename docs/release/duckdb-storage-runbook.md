# DuckDB Runtime Storage Runbook (NOD-196)

This runbook covers the persistent DuckDB analytics store for the
`ndith-api` backend in production.

> **Scope (locked by CTO under [NOD-196](/NOD/issues/NOD-196) +
> [NOD-252](/NOD/issues/NOD-252) split):** This runbook documents
> **K8s-side infrastructure readiness** for DuckDB persistence — env,
> volumeMount, PVC, fail-fast validation. **Live production durability
> closure for [NOD-192](/NOD/issues/NOD-192) is owned in [NOD-252](/NOD/issues/NOD-252)
> (Render runtime) plus [NOD-253](/NOD/issues/NOD-253)** (application
> writer-path enablement). See "Known production gap" below before
> running probe 4.

## Known production gap (read first)

As of this runbook's commit, **no production code path constructs a
file-backed `DuckDbClient` against `DUCKDB_PATH`**. `grep -rn
'DuckDbClient::(new|open)' backend/` returns zero hits outside
`#[cfg(test)]` calls to `DuckDbClient::in_memory()`. The startup
`ensure_duckdb_storage_ready` probe (this issue) validates the
directory is writable but never opens the DB file.

Net effect on this runbook:

- Probes 1–3 and 5 (PVC bound, env present, mount writable, persistence
  across pod restart) are meaningful and must pass.
- Probe 4 (`analytics.duckdb` appears after an analytics write)
  **cannot pass with current code**. The application-side writer
  enablement is tracked in [NOD-253](/NOD/issues/NOD-253). Until that
  ships, probe 4 below is replaced with an explicit writability proof
  that demonstrates the mount is usable but does **not** prove a
  production write is persisted.

When [NOD-253](/NOD/issues/NOD-253) ships, replace probe 4 with the
real production write trigger from that issue's runbook section and
re-run the full sequence on a redeploy.

To re-verify this gap still holds against any checkout:

```bash
# Expect: no matches outside #[cfg(test)] in_memory() calls
git grep -nE 'DuckDbClient::(new|open)' backend/
git grep -n 'DuckDbClient::in_memory' backend/
```

If the first command starts returning matches in non-test code, the
write-path is now plumbed and probe 4 below should be swapped back to
the real `analytics.duckdb` materialization expectation.

## Canonical configuration

| Field             | Value                                          |
| ----------------- | ---------------------------------------------- |
| `DUCKDB_PATH`     | `/var/lib/ndith/analytics/analytics.duckdb`    |
| Mount path        | `/var/lib/ndith/analytics`                     |
| PVC name (K8s)    | `ndith-duckdb-data`                            |
| PVC name (Helm)   | `{release}-backend-duckdb`                     |
| Access mode       | `ReadWriteOnce`                                |
| StorageClass      | `fast-ssd`                                     |
| Requested size    | `10Gi` (raw K8s) / `20Gi` (Helm production)    |

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

### 4. Writability proof (interim — see "Known production gap")

Until [NOD-253](/NOD/issues/NOD-253) lands a real production writer,
use a `kubectl exec` write probe to prove the disk is writable by the
runtime UID/GID. **This proves the volume works, not that analytics
writes are persisted.**

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
removes it. Failure of any step means the mount is not writable —
inspect the PVC binding and the pod's `fsGroup`/`runAsUser`.

```bash
kubectl exec -n ndith-production "$POD" -- ls -lh /var/lib/ndith/analytics
```

Expected (current state, until [NOD-253](/NOD/issues/NOD-253) ships):
the directory listing does **not** include `analytics.duckdb`. The
DuckDB file legitimately does not exist because no production code
opens it yet. Once [NOD-253](/NOD/issues/NOD-253) lands a real writer,
replace this section with:

> Hit the writer trigger from [NOD-253](/NOD/issues/NOD-253) (e.g.
> startup heartbeat or HTTP request that fans out to the analytics
> service); then `kubectl exec ... -- ls -lh /var/lib/ndith/analytics`
> shows a non-zero `analytics.duckdb`.

### 5. Persistence across pod restart

Until [NOD-253](/NOD/issues/NOD-253) ships, persistence is verified
against the writability-probe artifact rather than `analytics.duckdb`.
Place a marker file on the PVC, restart the deployment, and confirm
the marker survives:

```bash
kubectl exec -n ndith-production "$POD" -- sh -c \
  'echo "persist-probe $(date -u +%Y%m%dT%H%M%SZ)" \
     > /var/lib/ndith/analytics/.persist_probe'
kubectl exec -n ndith-production "$POD" -- stat \
  -c '%y %s %n' /var/lib/ndith/analytics/.persist_probe
kubectl rollout restart deployment/ndith-api -n ndith-production
kubectl rollout status deployment/ndith-api -n ndith-production --timeout=300s
NEW_POD=$(kubectl get pod -n ndith-production -l app=ndith-api \
  -o jsonpath='{.items[0].metadata.name}')
kubectl exec -n ndith-production "$NEW_POD" -- stat \
  -c '%y %s %n' /var/lib/ndith/analytics/.persist_probe
kubectl exec -n ndith-production "$NEW_POD" -- rm \
  /var/lib/ndith/analytics/.persist_probe
```

Expected: the `.persist_probe` file (mtime, size, content) survives
the rollout. After [NOD-253](/NOD/issues/NOD-253) ships, replace this
with the same `stat` sequence against `analytics.duckdb` to prove the
real DB file persists.

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
