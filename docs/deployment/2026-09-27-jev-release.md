# Jev release — 2026-09-27

Application revision: `7d1682274af9e806b905a3f81671f5ed54204474`, branch `codex/jev-evaluation-core`. See [implementation verification](../evaluations/2026-09-27-verification.md) for the test suite and [AWS runbook](aws-ecs.md) for repeatable release commands.

## Release state

The frontend and Convex changes are deployed. API and News task revisions are configured with the new images and retain their original desired count zero. The public frontend passes its container and load-balancer health checks, but the application's startup screen remains because the existing Upstash database rejects Redis PING. API and research runtime readiness cannot be claimed while that provider dependency is blocked.

| Component | Deployment | Result |
| --- | --- | --- |
| Convex | `scrupulous-emu-861` | Functions deployed at 17:40 UTC; typecheck passed |
| Frontend | ECS `ndith-production-ClusterCluster-ttavanod-Frontend:4` | Rollout completed; one healthy running task; `/render-health` reports application revision |
| Backend | ECS `ndith-production-ClusterCluster-ttavanod-Backend:3` | New ARM64 image registered; desired count zero; runtime verification blocked by Redis |
| News | ECS `ndith-production-ClusterCluster-ttavanod-News:2` | New ARM64 image registered; desired count zero; runtime verification blocked by Redis |

Frontend immutable image: `631342039360.dkr.ecr.us-east-1.amazonaws.com/ndith-frontend@sha256:f90718eb213348d9008bb47391f4d0af4ef2cae992b6fca38d68a337065a3d9d`.

API immutable image: `631342039360.dkr.ecr.us-east-1.amazonaws.com/ndith-backend@sha256:90a0f98b19e92092a53c0cab4fa2d78f4621cb89e3f23816ecb4dfa7a1db2510`.

News immutable image: `631342039360.dkr.ecr.us-east-1.amazonaws.com/ndith-backend@sha256:5afe468d23a42b80025c0b6d38cc5b923124516649ac3a0321021260ed3c0d0e`.

All three native ARM64 build workflows completed successfully for the application revision: [Frontend](https://github.com/kcirtapfromspace/no_drake_in_the_house/actions/runs/36337796318), [API](https://github.com/kcirtapfromspace/no_drake_in_the_house/actions/runs/36338610322), [News](https://github.com/kcirtapfromspace/no_drake_in_the_house/actions/runs/36337794301). Their ECR release tags were created without overwriting existing `latest` tags; ECS uses the digests above.

Frontend rollback task: `ndith-production-ClusterCluster-ttavanod-Frontend:3`. Before deployment, Backend used revision 2 and News revision 1, both at desired count zero. Other ECS services, DNS, autoscaling policies and data stores were not changed.

## Completed checks and configuration

- Confirmed Cloudflare DNS resolves the public app to the existing AWS ALB. Render configuration in the repository does not represent current production hosting.
- Recovered and preserved production nginx behavior: runtime service discovery, warming responses, OAuth/JWKS routes and upstream configuration.
- Built the frontend natively for ARM64. Local container `/render-health` and `nginx -t` passed. An unreachable upstream returned the expected 503 warming response.
- Compared old/new ECS task settings for Frontend, Backend and News in memory: only the application image changed. Frontend ALB target healthy, ECS rollout completed, and public health endpoint reports the expected full revision.
- Loaded the public app in a browser: static frontend renders; existing backend startup gate remains visible. This is not a successful end-to-end login/evidence test.
- Authenticated official Convex code generation and deployment passed; root tests again passed 72/72 after code generation.
- Configured Jev model/key and Firecrawl key on the live Convex deployment using existing secret sources, without printing or committing values. Verified AWS and Convex use the same research service key.
- Set Convex `NDITH_BACKEND_URL=https://nodrakeinthe.house`, replacing a dead Render URL.
- Verified research ingress: unauthenticated request returns 401; authenticated unsupported operation returns 400 without creating a production record.
- Ran the paginated verified-only artist-index rebuild and confirmed the resulting index is empty, consistent with the existing inventory of 56 offenses and zero verified offenses. Source offense records were not changed.
- Warmed the original API for diagnosis: Aurora connected and migrations completed, then Redis startup failed. Returned the API to its original desired count zero. A direct Redis PING independently confirmed the same Upstash rate-limit rejection.

No credentials were added to the repository. `.env.local` created by the Convex CLI contains the deployment selector and public URLs and remains gitignored. No streaming-provider write, live payment, privilege grant, historical offense promotion or change to Redis billing was performed.

## Remaining work

1. Restore access/quota for the existing Upstash database. The provider response requests contacting Upstash support; deployment code cannot repair this condition.
2. Once Redis PING succeeds, warm Backend and News and verify readiness, research dispatch/callback and the browser's startup gate clearing. Preserve the intended idle scaling afterward.
3. Provision appropriate independent reviewers through the trusted administration path. The live inventory has four users and no owner/reviewer role; no account was arbitrarily promoted during deployment.
4. Exercise source acquisition → Jev worker → review → independent approval with authorized identities, including a failure/retry case. The local synthetic Jev evaluation passed eight cases, but that does not prove the full deployed workflow.

The broader audit's unresolved findings remain outside this release's claim. Deployment does not certify the entire application or historical evidence quality.
