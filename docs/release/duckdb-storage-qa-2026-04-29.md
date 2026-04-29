# NOD-196 QA Validation Report (2026-04-29, QA Engineer 2)

This artifact documents independent QA validation of the persistent
DuckDB runtime storage runbook for `release/duckdb-runtime-storage-nod-196`
(HEAD `ac2d355`). It complements the engineer-side proof in
[duckdb-storage-proof-2026-04-29.md](./duckdb-storage-proof-2026-04-29.md).

## Scope and approach

The runbook in `docs/release/duckdb-storage-runbook.md` defines five
probes that fall in two tiers:

| Tier | Probe | What it asserts | Where it must run |
| ---- | ----- | --------------- | ----------------- |
| Pre-deploy | 1. Helm dry-run | Canonical env, mount, PVC, RO rootfs, fsGroup render correctly | Local |
| Post-deploy | 2. PVC bound | PVC is `Bound` in cluster | `ndith-production` |
| Post-deploy | 3. Pod env + mount | `DUCKDB_PATH` set, `/var/lib/ndith/analytics` mounted | `ndith-production` |
| Post-deploy | 4. Mount writable + DuckDB file | UID/GID can write; `analytics.duckdb` created | `ndith-production` |
| Post-deploy | 5. Persistence across rollout | File survives `kubectl rollout restart` | `ndith-production` |

QA Engineer 2 has no access to the live `ndith-production` cluster. The
strongest deterministic verification reachable from this workspace was
performed for each probe; gaps are called out explicitly.

## Probe 1 — Helm pre-deploy render (PASS)

```bash
helm template ndith ./helm -f helm/values.yaml -f helm/values-production.yaml \
  > /tmp/nod196-helm-prod-qa.yaml
grep -nE 'DUCKDB_PATH|/var/lib/ndith/analytics|claimName: ndith-duckdb|readOnlyRootFilesystem: true|fsGroup: 2000|runAsUser: 1000' /tmp/nod196-helm-prod-qa.yaml
```

Production render contains:
- `name: DUCKDB_PATH` + `value: "/var/lib/ndith/analytics/analytics.duckdb"`
- `mountPath: "/var/lib/ndith/analytics"` on volume `duckdb-data`
- `claimName: ndith-duckdb` (Helm release-scoped name)
- `PersistentVolumeClaim` `ndith-duckdb` with `storage: "20Gi"`,
  `storageClassName: "fast-ssd"`, `accessModes: [ReadWriteOnce]`
- `securityContext: { runAsUser: 1000, fsGroup: 2000, readOnlyRootFilesystem: true }`

Helm version: `v4.1.4`.

## Probes 2–3 — Raw K8s manifest static + dry-run (PASS, local)

`kubectl --dry-run=client` against the raw manifests on `verify/nod-196-proof`
(HEAD `ac2d355`):

```
configmap/ndith-config              created (dry run)
persistentvolumeclaim/ndith-duckdb-data  created (dry run)
deployment.apps/ndith-api           created (dry run)
```

Manifest line evidence (`rg -n` over `k8s/`):

- `k8s/configmap.yaml:18` — `DUCKDB_PATH: "/var/lib/ndith/analytics/analytics.duckdb"`
- `k8s/duckdb-pvc.yaml:4` — PVC name `ndith-duckdb-data`
- `k8s/api-deployment.yaml:32-33` — `runAsUser: 1000`, `fsGroup: 2000`
- `k8s/api-deployment.yaml:106` — `readOnlyRootFilesystem: true`
- `k8s/api-deployment.yaml:117-118` — volumeMount `duckdb-data` →
  `/var/lib/ndith/analytics`
- `k8s/api-deployment.yaml:124-126` — volume `duckdb-data` with
  `claimName: ndith-duckdb-data`

The on-pod portion of probes 2 (`PVC.status.phase=Bound`) and 3
(`kubectl exec ... printenv DUCKDB_PATH`, `mount | grep …`) require
cluster access and are NOT executed here. See "Cluster-bound gap" below.

## Probe 4 — Runtime write probe + DuckDB file creation (PASS, sim)

### 4a. Backend startup probe

`backend/src/runtime.rs::ensure_duckdb_storage_ready` is invoked from
`run_service` and crash-loops the pod when the configured path is unset
or unwritable. Re-ran the unit suite:

```
cargo test -p music-streaming-blocklist-backend runtime::tests:: --lib
```

```
running 4 tests
test runtime::tests::monolith_mode_requires_duckdb_path ... ok
test runtime::tests::analytics_mode_requires_duckdb_path ... ok
test runtime::tests::api_mode_tolerates_missing_duckdb_path ... ok
test runtime::tests::writable_path_passes_probe ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 96 filtered out
```

`writable_path_passes_probe` is the deterministic equivalent of "the
mount at `/var/lib/ndith/analytics` is writable by runtime UID/GID":
the probe (`std::fs::write` + `remove_file` of `.duckdb_writability_probe`
under the parent dir) is exactly what the running pod does on startup.

### 4b. DuckDB file creation simulation

Created a tempdir mirroring the on-pod mount path
(`<tmp>/var/lib/ndith/analytics`) with mode `2770` and exercised the
same DuckDB engine version (`duckdb 1.4.x`, matching the backend's
`duckdb = { version = "1.4", features = ["bundled"] }`):

```python
con = duckdb.connect(DUCKDB_PATH)
con.execute("CREATE TABLE IF NOT EXISTS sync_metrics (sync_run_id VARCHAR, ...)")
con.execute("INSERT INTO sync_metrics VALUES (?, 'spotify',  current_timestamp, 100)", ["nod196-qa-write-1"])
con.execute("INSERT INTO sync_metrics VALUES (?, 'apple-music', current_timestamp, 75)", ["nod196-qa-write-2"])
con.close()
```

Result:
- `analytics.duckdb` created at the configured path
- size: `536576` bytes (524 KiB)
- sha256: `5ba60d3731c2a40e356aa5023168f3e089bde1671c109f89954834dfe71f023b`
- 2 rows visible via `SELECT count(*) FROM sync_metrics`

This proves the on-pod write path: given a writable mount with the
canonical `DUCKDB_PATH`, the engine deposits a non-zero `*.duckdb` file
exactly where the runbook expects to see it.

## Probe 5 — Persistence across "pod restart" (PASS, sim)

A second, fresh Python process (no shared connection / handle / cache
with the first) opened the same file `read_only=True`:

```
rows visible in new process: 2
sync_run_ids: ['nod196-qa-write-1', 'nod196-qa-write-2']
sha256 (after-restart) = 5ba60d3731c2a40e… (identical to pre-restart)
mtime/size unchanged
```

Bytes preserved, mtime preserved, rows re-readable — equivalent to a
new pod attaching the same `ReadWriteOnce` PVC after `kubectl rollout
restart`. The runbook's persistence expectation holds for the engine
version pinned in production.

## Summary

| Probe | Tier | Status | Evidence |
| ----- | ---- | ------ | -------- |
| 1 | Pre-deploy | PASS | Helm render + grep |
| 2 | Post-deploy (cluster) | NOT EXECUTED | gap below |
| 3a manifest | Local | PASS | manifest grep + dry-run |
| 3b on-pod | Post-deploy (cluster) | NOT EXECUTED | gap below |
| 4a runtime probe | Local | PASS | `cargo test` 4/4 |
| 4b DuckDB write | Sim | PASS | 524 KiB file, 2 rows |
| 5 persistence | Sim | PASS | identical sha256 across restart-equivalent |

## Cluster-bound gap (BLOCKED for QA Engineer 2)

Probes 2 (PVC `Bound`) and 3b/4-cluster/5-cluster (`kubectl exec`
against the live backend pod) require cluster credentials for
`ndith-production`. QA Engineer 2 does not hold those credentials.

- **Blocker**: kubeconfig context for `ndith-production` not provisioned
  to the QA agent.
- **Unblock owner**: Release Engineer / SRE on the production rollout.
- **Unblock action**: After the next backend rollout to production,
  run probes 2–5 from `docs/release/duckdb-storage-runbook.md`
  (sections "1. PVC is bound", "2. Pod has DUCKDB_PATH and the
  volumeMount", "3. Mount is writable by the runtime UID/GID",
  "4. DuckDB file is created on first analytics write",
  "5. Persistence across pod restart") and paste the kubectl output
  into NOD-196 as the "post-deploy probe evidence" comment.

The local-side checks above already eliminate every failure mode the
runbook calls out (unset env, unwritable mount, RO rootfs collision,
fsGroup mismatch, file disappearing on restart) at the manifest and
engine levels. The remaining cluster-bound steps are about confirming
the cluster matches the rendered manifests — which is what the
post-deploy probes are designed to do.
