# AWS ECS deployment

Verified against the live environment on 2026-09-27. The public website and API use AWS ECS behind an ALB and Cloudflare. Repository Render workflows are legacy build infrastructure; their names do not identify the current hosting provider. Do not use their Render deployment hooks for this environment.

## Targets

| Component | Live target |
| --- | --- |
| Website | `https://nodrakeinthe.house` |
| AWS profile / region | `AdministratorAccess-631342039360` / `us-east-1` |
| ECS cluster | `ndith-production-ClusterCluster-ttavanod` |
| Application services | `Frontend`, `Backend`, `News` |
| Container platform | `linux/arm64` |
| Convex | `scrupulous-emu-861` (`https://scrupulous-emu-861.convex.cloud`) |
| Research ingress | `https://scrupulous-emu-861.convex.site/research/ingest` |

The live Convex deployment is labelled **dev** by Convex. The project's default production deployment, `benevolent-lapwing-467`, is a different database. Never infer the live target from that label. Inspect the public frontend configuration and explicitly select the deployment before releasing.

Frontend proxies to Cloud Map services under `production.ndith.sst`, using runtime DNS resolution so idle services do not prevent nginx from starting. Preserve the live entrypoint, warming behavior, OAuth/JWKS routes and upstream environment when building images. Backend and News normally scale to zero; Frontend normally has one task.

Cold upstream responses remain HTTP 503 with `Retry-After: 10`. An internal nginx mirror calls the existing wake function for the routed service, including for browser `fetch` and Convex requests. Wake calls contain only the allowlisted service name; client credentials, cookies, query parameters and bodies are excluded. Successful wake responses are cached per service for 30 seconds. Healthy and authorization-denied upstream responses do not trigger wake-up. Deployments without `WAKE_URL` keep the 503 behavior without calling an external service.

The frontend image workflow exercises these behaviors against local HTTP stubs, including an unresolved upstream hostname. To repeat the packaged-image check:

```sh
python3 scripts/deployment/test-nginx-wake.py --image FRONTEND_IMAGE_REFERENCE --use-image-files
```

Without `--use-image-files`, the test mounts the current checkout's nginx template, entrypoint and warming page into an existing frontend image. Containers and mock servers are removed after the check.

## Publish and roll out

1. Run the checks in [the verification record](../evaluations/2026-09-27-verification.md). Commit and push the intended release branch. Keep infrastructure-only follow-ups distinct from the application image revision.
2. Dispatch the frontend and selected backend image workflows on that revision, with `platform=linux/arm64` and `deploy_to_render=false`. The backend workflow accepts `service=ndith-backend` or `service=ndith-news`. A branch build publishes revision tags without overwriting `latest`.
3. Wait for successful image publication. Inspect architecture using `crane config`. Authenticate Docker to the existing ECR registry through `aws ecr get-login-password` piped to `docker login --password-stdin`; do not print credentials. Copy the revision image with `crane copy --no-clobber` into the corresponding existing ECR repository. Resolve its immutable digest.
4. Run the ECS helper first without `--apply`, then with it after inspecting the target. Replace `IMAGE_DIGEST_REFERENCE` with the complete ECR `repository@sha256:...` reference:

   ```sh
   uv run scripts/deployment/ecs-release.py \
     --profile AdministratorAccess-631342039360 \
     --cluster ndith-production-ClusterCluster-ttavanod \
     --service Frontend \
     --image IMAGE_DIGEST_REFERENCE
   ```

   The helper checks architecture, repository, account and region; clones existing task settings in memory; replaces only the application image; and preserves desired count, networking, IAM roles, resources, environment, secrets and health checks. It emits a receipt containing the old/new task definition. Save the receipt without adding secret values. A repeated release of the same digest is a no-op.
5. Observe ECS deployment status and stopped-task reasons. For Frontend, require ALB target health and confirm `/render-health` reports the expected full application commit SHA. Check the browser as well: healthy static hosting does not prove the API is available.
6. For an idle Backend or News service, registration at desired count zero stages the release but does not verify runtime health. After dependencies are healthy, temporarily warm the service to one task, verify startup and readiness, then preserve the intended scaling policy. Do not leave a service crash-looping against a blocked provider.

## Convex and research configuration

For this existing live deployment, the authenticated deployment command is:

```sh
CONVEX_DEPLOYMENT=dev:scrupulous-emu-861 npx convex dev --once --typecheck enable --tail-logs disable
```

The ordinary `convex deploy` default targets the separate production-labelled database. A CI deployment key must be checked for the intended target before use. The command above uses the operator's existing Convex authentication and may create a gitignored `.env.local` deployment selector.

Configure server-side `TYPESAFE_API_KEY`, `TYPESAFE_MODEL=jev-1.13.0`, `FIRECRAWL_API_KEY`, and `NDITH_SERVICE_KEY` through the secret manager. Keep the Convex service key identical to the existing AWS `/ndith/production/NDITH_SERVICE_KEY` SSM parameter. Set `NDITH_BACKEND_URL=https://nodrakeinthe.house`; nginx routes `/api/v1/news/` to News. Never put provider/service credentials in `VITE_*` variables.

Pause legacy research ingestion during the coordinated Convex and News release. The old public mutation transport is intentionally incompatible with the authenticated ingress. After deployment, verify that an unauthenticated request returns 401 and an authenticated unsupported operation returns 400. The latter confirms the shared key without creating production records. Then exercise acquisition and the independent reviewer workflow with authorized test identities.

## Rollback and external blockers

Use the previous task ARN from the deployment receipt with `aws ecs update-service --cluster ... --service ... --task-definition ...`, specifying the same profile/region. Confirm stability and restore the original desired count. Do not overwrite mutable image tags as a rollback mechanism. A frontend rollback is independent of data, but a legacy News rollback requires a compatible Convex contract; do not restart unauthenticated legacy ingestion.

On 2026-09-27, the existing Upstash database initially rejected PING with a temporary rate-limit error. After the user restored access, a fresh TLS connection succeeded at 18:37 UTC. The first new Rust images then exposed missing Redis TLS build features. The workspace now enables `tokio-rustls-comp`, and `ndith-db` tests production `rediss://` pool construction. Keep certificate validation and TLS enabled; do not downgrade the connection URL, replace the database or change its billing as an incidental deployment action.

See [the Jev release record](2026-09-27-jev-release.md) for actual revisions, completed checks and outstanding work.
