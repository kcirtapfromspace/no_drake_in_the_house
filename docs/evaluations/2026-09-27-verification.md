# Jev implementation verification — 2026-09-27

Branch: `codex/jev-evaluation-core`. Original audited revision: `f2c320ac44f236dde3629d5e2369549f5625d746`. These results describe implementation checks. Subsequent deployment checks are recorded in the [AWS/Convex release record](../deployment/2026-09-27-jev-release.md).

| Check | Result |
| --- | --- |
| Root `npm run typecheck` | Pass; the baseline's 29 errors are resolved |
| Root `npm test` | 72 passed across six test files |
| Frontend `npm run check` | Zero errors; 24 existing warnings in eight files |
| Frontend `npm run build` | Pass |
| Frontend `npm test` | 353 passed, 13 skipped; includes four new mounted-component tests |
| Rust `ndith-news` client unit tests | 10 passed |
| Rust news crate `cargo check` | Pass |
| Rust news-service binary `cargo check`, `--no-default-features --features news` | Pass |
| Rust API `cargo check`, `--no-default-features --features render-api` | Pass |
| Rust formatting / `git diff --check` | Pass |
| Live Jev synthetic evaluation | 8/8 expected routes; charged/dismissed cases also match expected attribution, outcome and category |

The live fixture uses `jev-1.13.0` with 9,529 input tokens in the recorded run. It includes music idioms, victim attribution, commentary, lyrics, another person's crime, a fraud charge, dismissal, and source prompt injection. Responses and local wall-clock timings are retained in [the JSON artifact](2026-09-27-jev-synthetic.json). Timings are observations from eight local requests, not an SLA or a load test. The test sends only synthetic text about “Example Singer.”

The supplied 1Password key was read into a process environment for this test. No credential was stored in the repository or printed. Tests of the Convex worker use mocked providers; the synthetic run does not exercise remote Convex scheduling or Firecrawl. Later server-side key configuration and deployment are documented in the separate release record.

## Repeatable commands

From the repository root:

```sh
npm ci
npm run typecheck
npm test
```

From `frontend/`:

```sh
npm ci
npm run check
npm run build
npm test
```

From `backend/`:

```sh
SQLX_OFFLINE=true cargo test --locked -p ndith-news --lib convex_client::tests
SQLX_OFFLINE=true cargo check --locked -p music-streaming-blocklist-backend --no-default-features --features news --bin ndith-news-service
SQLX_OFFLINE=true cargo check --locked -p music-streaming-blocklist-backend --no-default-features --features render-api
cargo fmt --all -- --check
```

For an explicitly requested live synthetic evaluation, inject `TYPESAFE_API_KEY` through the secret manager and run `npm run eval:jev`. CI does not call the paid provider. Keep credentials out of command arguments, shell tracing and artifacts.

The dependency patch from `ethnum` 1.5.2 to 1.5.3 restores compatibility with the installed Rust compiler. Optional-feature import gates were also corrected so the news-only service includes its unavailable-feature handlers. Future-incompatibility warnings for `redis` and `num-bigint-dig` remain. Frontend lint still treats the existing 24 warnings as failures. Thirteen skipped frontend tests remain skipped.

## Relationship to the original audit

[BUILD-AUDIT.md](../audits/2026-09-27/BUILD-AUDIT.md) and its evidence remain a snapshot of the original revision. Its reproduction suite intentionally demonstrates vulnerabilities at that revision and is not the acceptance suite for this branch. New rejection and lifecycle tests are in `convex/evaluations.test.ts` and `convex/jev.test.ts`.

| Original finding | Implementation status |
| --- | --- |
| F01–F02, identity rebinding | Public attach helper made internal; established subjects cannot be rebound by email, and existing identities cannot collide with another user's email |
| F03, public research writes | Ingestion made internal; service-authenticated allowlist and updated Rust transport; direct research offense creation removed |
| F04, enforcement helper access | Helpers made internal; batch/item access requires the authenticated owner before mutation/execution |
| F05, playlist grading isolation | Queries scoped to authenticated user and provider; identical playlist names within one account still need stable-ID migration |
| F06, arbitrary verification | Legacy verification restricted to owners with self-review blocked; evaluated records must use the independent review workflow |
| F07–F09, evidence integrity and quota | Verified-only projections; semantic source evaluation; atomic attempts quota and durable states |
| F10, severity mismatch | Canonical severity normalization in the approved artist index and new review path; historical data still needs reconciliation |
| F11, truncated scans | Artist-index rebuild and legacy promotion now paginate; other legacy aggregation/fan-out caps remain |
| F12, false research completion | Separate acquisition claim, 202-as-queued, completion callback, generation fencing and expiration; old incorrect freshness stamps are not migrated |
| F13, build/CI | Declared Convex test dependencies, official generated type layout, evaluation CI, Rust news compile fixes; no remote CI run claimed |
| F20, first fuzzy match | Removed first-result fallback in the touched DNP and investigation resolution paths; broader alias/entity-resolution quality remains to evaluate |
| F21, private blocks in public policy | Public extension catalog excludes private block choices and unverified records |
| F14–F19, F22 | Remain open: deployment storage, extension packaging/signature fail-open behavior, log redaction, broader dependency remediation, mobile readiness and first-subscription ownership |

Changes above are implementation progress with local checks, not certification that every audit concern is closed in production. Historical machine-verified rows, staging identity integration, service-secret coordination, real provider actions and deployment reconciliation remain release work. See [architecture and rollout](../architecture/jev-evaluation.md).
