# DuckDB Runtime Storage Runbook (NOD-196)

This runbook covers the persistent DuckDB analytics store for the
`ndith-api` backend in production.

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

### 4. DuckDB file is created on first analytics write

After traffic exercises the analytics path (sync run, scheduled probe,
or forced enforcement run):

```bash
kubectl exec -n ndith-production "$POD" -- ls -lh /var/lib/ndith/analytics
```

Expected: `analytics.duckdb` exists and is non-zero. If the file is
absent after analytics-relevant traffic, something has overridden the
canonical `DUCKDB_PATH` or no analytics writes have run yet — re-check
`printenv DUCKDB_PATH` and the backend logs for the
`DuckDB analytics storage probe passed` log line emitted on startup.

### 5. Persistence across pod restart

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

Expected: same file (mtime, size) survives the rollout.

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
