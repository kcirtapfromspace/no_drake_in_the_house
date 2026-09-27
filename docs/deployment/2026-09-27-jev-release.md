# Jev release — 2026-09-27

Initial application revision: `7d1682274af9e806b905a3f81671f5ed54204474`, branch `codex/jev-evaluation-core`. Runtime fixes use Rust revision `85fcda2` (Redis TLS) and frontend revision `3bdff1a` (server-side wake-up). See [implementation verification](../evaluations/2026-09-27-verification.md) for the test suite and [AWS runbook](aws-ecs.md) for repeatable release commands.

## Release state

Deployed at https://nodrakeinthe.house. Redis access is restored; Frontend, Backend and News rollouts are complete and their tasks are healthy. The browser reaches sign-in. Production cold starts, authenticated research routing, source fetching and live Jev evaluation passed. Reviewer assignment and the full human approval workflow remain pending.

| Component | Deployment | Result |
| --- | --- | --- |
| Convex | `scrupulous-emu-861` | Functions deployed at 17:40 UTC; typecheck passed |
| Frontend | ECS `ndith-production-ClusterCluster-ttavanod-Frontend:5` | Rollout completed; healthy; expected revision served; browser reaches sign-in |
| Backend | ECS `ndith-production-ClusterCluster-ttavanod-Backend:4` | Rollout completed; healthy; public health/readiness 200; Redis TLS verified |
| News | ECS `ndith-production-ClusterCluster-ttavanod-News:3` | Rollout completed; healthy; Redis TLS, automatic wake-up and authenticated route verified |

Frontend immutable image: `631342039360.dkr.ecr.us-east-1.amazonaws.com/ndith-frontend@sha256:22b8373d7a5e1fe45ec65ef2bb89a33f47dd52b91ac274b5efe1912ef39a9366`.

API immutable image: `631342039360.dkr.ecr.us-east-1.amazonaws.com/ndith-backend@sha256:ccfaef9119d0f8c2ffdfeaec5785e98062955bb8ab87deaf1497d6add531f4b7`.

News immutable image: `631342039360.dkr.ecr.us-east-1.amazonaws.com/ndith-backend@sha256:391cb70fb2319207f61d1736b17ac47ba8bc2ed97ee9426a0749a99a567eb167`.

All three final native ARM64 build workflows completed successfully: [Frontend](https://github.com/kcirtapfromspace/no_drake_in_the_house/actions/runs/36342036701), [API](https://github.com/kcirtapfromspace/no_drake_in_the_house/actions/runs/36341629047), [News](https://github.com/kcirtapfromspace/no_drake_in_the_house/actions/runs/36341630999). Their ECR release tags were created without overwriting existing `latest` tags; ECS uses the digests above.

Frontend rollback task: `ndith-production-ClusterCluster-ttavanod-Frontend:4` (Jev UI, without the proxy wake fix). Backend revision 2 is the previous known healthy API. Do not roll back to Backend revision 3 or News revision 2: those images omit Redis TLS support. Legacy News revision 1 is incompatible with the new ingestion contract. Other ECS services, DNS, autoscaling policies and data stores were not changed.

## Completed checks and configuration

- Confirmed Cloudflare DNS resolves the public app to the existing AWS ALB. Render configuration in the repository does not represent current production hosting.
- Recovered and preserved production nginx behavior: runtime service discovery, warming responses, OAuth/JWKS routes and upstream configuration.
- Built the frontend natively for ARM64. Local container `/render-health` and `nginx -t` passed. An unreachable upstream returned the expected 503 warming response.
- Compared old/new ECS task settings for Frontend, Backend and News in memory: only the application image changed. Frontend ALB target healthy, ECS rollout completed, and public health endpoint reports the expected full revision.
- Verified the final frontend clears its startup gate to the sign-in page after the idle API wakes automatically. No account login or independent approval flow was performed.
- Authenticated official Convex code generation and deployment passed; root tests again passed 72/72 after code generation.
- Configured Jev model/key and Firecrawl key on the live Convex deployment using existing secret sources, without printing or committing values. Verified AWS and Convex use the same research service key.
- Set Convex `NDITH_BACKEND_URL=https://nodrakeinthe.house`, replacing a dead Render URL.
- Verified research ingress: unauthenticated request returns 401; authenticated unsupported operation returns 400 without creating a production record.
- Ran the paginated verified-only artist-index rebuild and confirmed the resulting index is empty, consistent with the existing inventory of 56 offenses and zero verified offenses. Source offense records were not changed.
- During the initial diagnosis, Aurora connected and migrations completed, then Redis startup failed. A direct Redis PING independently confirmed the Upstash rate-limit rejection. Recovery and final runtime checks are recorded below.

No credentials were added to the repository. `.env.local` created by the Convex CLI contains the deployment selector and public URLs and remains gitignored. No streaming-provider write, live payment, privilege grant, historical offense promotion or change to Redis billing was performed.

## Remaining work

1. Provision appropriate independent reviewers through the trusted administration path after the user identifies the accounts. The live inventory has four users and no owner/reviewer role; no account was arbitrarily promoted during deployment.
2. Exercise real research acquisition → review → independent approval with authorized identities, including a failure/retry case. The passing connectivity checks do not prove the full human approval workflow.
3. Investigate scheduler warnings from optional Twitter, Reddit, RSS and NewsAPI collectors. Their individual feed availability was not verified by this deployment; the News service and research request handler are healthy.

## Redis recovery and runtime verification

- After the user restored Upstash, fresh Redis TLS PING succeeded at 18:37 UTC. No connection credentials or database plan were changed by the deployment.
- The initial new Rust images failed because their Redis build omitted TLS support. Reproduced this exact failure with a pool-construction regression test, enabled the Tokio rustls connector, and passed all five `ndith-db` unit tests. Existing CI runs that crate's tests.
- Temporarily restored the previous API image while rebuilding. Its public readiness endpoint returned 200 and the browser reached sign-in. Legacy News ingestion remained stopped.
- Rolled the corrected API image to ECS Backend revision 4. Redis initialization succeeded over TLS, the task became healthy, the rollout completed, and public `/api/health` and `/api/health/ready` returned 200.
- The new frontend image passed CI checks for cold GET/POST requests, DNS failure, targeted wake-up, omission of caller credentials/query/body, 30-second request coalescing, internal-route privacy, healthy/auth response preservation, and an absent wake URL. A local instance also called the actual AWS wake function over verified HTTPS and recorded a 200 response, while preserving the client's 503 response.
- At 19:08 UTC, News had zero desired/running tasks. A public POST received 503 and automatically woke News to one task without a manual ECS update. The corrected service then initialized Redis over TLS and became healthy. An unauthenticated research request returned 401; an authenticated empty-name request returned the expected validation error (400), without starting research. A stale research completion callback returned `accepted: false`.
- Backend also scaled idle under the existing policy and woke automatically through the frontend. Readiness returned 200, and the browser transitioned to sign-in. All three final task revisions subsequently reported completed ECS rollouts with one running task and no pending tasks. Existing idle scaling remains enabled.
- A clearly labeled synthetic non-evidence submission passed through the live authenticated research ingress, Convex scheduler and Jev worker. It completed in one attempt with `no_support`, retained its source hash, and created no offense. See [the supplied-text smoke record](../evaluations/2026-09-27-deployed-jev-smoke.json).
- A separate URL-only job fetched the IANA example page through Firecrawl and evaluated it with Jev. It also completed in one attempt with `no_support`, a retained source hash and no offense. See [the source-fetch smoke record](../evaluations/2026-09-27-deployed-source-fetch-smoke.json). Both diagnostic jobs remain for provenance. Neither check approved evidence or exercised discovery of real allegations through the Rust research service.

The broader audit's unresolved findings remain outside this release's claim. Deployment does not certify the entire application or historical evidence quality.
