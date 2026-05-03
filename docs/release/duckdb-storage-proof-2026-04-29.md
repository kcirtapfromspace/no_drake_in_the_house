# NOD-196 Production Provisioning Proof (2026-04-29)

This artifact captures deterministic verification for persistent DuckDB runtime storage on `release/duckdb-runtime-storage-nod-196`.

## Scope

- Canonical `DUCKDB_PATH` present in raw K8s and Helm production values.
- Writable persistent mount wired for backend pod.
- Read-only root filesystem retained with writable mounted path.
- Startup validation tests for `DUCKDB_PATH` behavior pass.

## 1) Raw K8s manifest checks

Command:

```bash
rg -n "DUCKDB_PATH|/var/lib/ndith/analytics|ndith-duckdb-data|volumeMounts|claimName|fsGroup|readOnlyRootFilesystem" \
  k8s/api-deployment.yaml k8s/configmap.yaml k8s/duckdb-pvc.yaml -S
```

Key evidence:

- `k8s/configmap.yaml`: `DUCKDB_PATH: "/var/lib/ndith/analytics/analytics.duckdb"`
- `k8s/api-deployment.yaml`: `readOnlyRootFilesystem: true`
- `k8s/api-deployment.yaml`: `volumeMounts.mountPath: /var/lib/ndith/analytics`
- `k8s/api-deployment.yaml`: `claimName: ndith-duckdb-data`
- `k8s/api-deployment.yaml`: `fsGroup: 2000`
- `k8s/duckdb-pvc.yaml`: PVC `ndith-duckdb-data`

Dry-run apply:

```bash
kubectl apply --dry-run=client -f k8s/configmap.yaml
kubectl apply --dry-run=client -f k8s/duckdb-pvc.yaml
kubectl apply --dry-run=client -f k8s/api-deployment.yaml
```

Output:

- `configmap/ndith-config created (dry run)`
- `persistentvolumeclaim/ndith-duckdb-data created (dry run)`
- `deployment.apps/ndith-api created (dry run)`

## 2) Helm production render checks

Production hardening applied:

- `helm/values-production.yaml`: explicit `securityContext.readOnlyRootFilesystem: true`

Render command:

```bash
helm template ndith ./helm -f helm/values.yaml -f helm/values-production.yaml > /tmp/nod196-helm-prod.yaml
rg -n "name: DUCKDB_PATH|value: \"/var/lib/ndith/analytics/analytics.duckdb\"|mountPath: \"/var/lib/ndith/analytics\"|claimName: .*duckdb|readOnlyRootFilesystem: true|fsGroup: 2000" /tmp/nod196-helm-prod.yaml -S
```

Key evidence:

- `fsGroup: 2000`
- `readOnlyRootFilesystem: true`
- `name: DUCKDB_PATH` + `value: "/var/lib/ndith/analytics/analytics.duckdb"`
- `mountPath: "/var/lib/ndith/analytics"`
- `claimName: ndith-duckdb`

## 3) Runtime startup validation tests

Command:

```bash
cd backend
cargo test -p music-streaming-blocklist-backend runtime::tests:: --lib
```

Result:

- `4 passed; 0 failed`
- `analytics_mode_requires_duckdb_path ... ok`
- `monolith_mode_requires_duckdb_path ... ok`
- `api_mode_tolerates_missing_duckdb_path ... ok`
- `writable_path_passes_probe ... ok`

## Conclusion

Provisioning path is fully wired for persistent DuckDB storage with read-only root filesystem constraints preserved and startup validation enforced.
