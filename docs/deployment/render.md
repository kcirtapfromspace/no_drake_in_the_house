# Render deployment

This repo includes a production-focused Render Blueprint at `render.yaml`.

It provisions:

- `ndith-backend`: image-backed web service for the Rust API
- `ndith-frontend`: Docker-based web service that serves the SPA and reverse-proxies backend traffic on the same origin
- `ndith-postgres`: managed Postgres
- `ndith-redis`: Render Key Value for `REDIS_URL`

## URL contract

The production URL layout is:

- Frontend app: `https://nodrakeinthe.house`
- Public backend hostname: `https://api.nodrakeinthe.house`
- Browser-facing API traffic: `https://nodrakeinthe.house/api/...`

The frontend no longer calls the backend's public `onrender.com` hostname directly. Render serves the SPA at the apex domain and proxies `/api`, `/oauth`, `/metrics`, and `/monitoring` to the backend over the private network. This avoids the CORS failure that drops the UI into the maintenance screen when the apex domain is attached before the backend CORS settings are updated.

Because Render Blueprint files do not support variable interpolation, the URL-sensitive backend variables stay as `sync: false` in `render.yaml` and must be supplied in the dashboard during the initial import.

## Publish the backend image first

The backend deploys from a prebuilt Docker image instead of building on Render.

Mainline publishing is handled by GitHub Actions in `.github/workflows/render-backend-image.yml`.
On every push to `main`, the workflow publishes a `linux/amd64` image to Docker Hub with:

- an immutable short-SHA tag
- `:latest` for the default Render image reference in `render.yaml`

Required GitHub repository secrets:

- `DOCKERHUB_USERNAME`
- `DOCKERHUB_TOKEN`
- `RENDER_NDITH_BACKEND_DEPLOY_HOOK_URL` if you want the workflow to trigger the Render backend deploy after publishing

Manual fallback:

```bash
./scripts/publish-render-backend-image.sh
```

## Import the Blueprint and first deploy

1. Push the repo to `main` and wait for the `Publish Render Backend Image` workflow to finish.
2. Confirm `docker.io/kcirtapfromspace/ndith-backend:latest` exists, or publish manually if you are bootstrapping outside GitHub Actions.
3. In Render, choose `New` -> `Blueprint`.
4. Point Render at this repository and approve `render.yaml`.
5. Before the first deploy, fill in the required secrets:
   - `OAUTH_ENCRYPTION_KEY`
   - `KMS_MOCK_MASTER_KEY`
6. For the first deploy, set the backend URL-sensitive variables to the default Render frontend hostname:
   - `OAUTH_FRONTEND_BASE_URL=https://ndith-frontend.onrender.com`
   - `CORS_ALLOWED_ORIGINS=https://ndith-frontend.onrender.com`

Generate those secrets with:

```bash
openssl rand -base64 32
openssl rand -hex 32
```

Use the base64 output for `OAUTH_ENCRYPTION_KEY` and the hex output for `KMS_MOCK_MASTER_KEY`.

The frontend does not require a manually-entered `VITE_API_URL` during import. The Docker build hard-codes `VITE_API_URL=__RELATIVE__`, and the runtime receives `BACKEND_HOSTPORT` from Render via `fromService.property=hostport`.

## Backend env vars

The Blueprint wires these automatically:

- `DATABASE_URL` from `ndith-postgres`
- `REDIS_URL` from `ndith-redis`
- `JWT_SECRET` as a generated secret
- `ENVIRONMENT=production`
- `HOST=0.0.0.0`
- `PORT=3000`

You must provide these during the Blueprint import because they depend on your public frontend URL:

- `OAUTH_FRONTEND_BASE_URL`
- `CORS_ALLOWED_ORIGINS`

Provider credentials are still required for whichever integrations you enable:

- Google: `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET`, `GOOGLE_REDIRECT_URI`
- GitHub: `GITHUB_CLIENT_ID`, `GITHUB_CLIENT_SECRET`, `GITHUB_REDIRECT_URI`
- Apple: `APPLE_CLIENT_ID`, `APPLE_TEAM_ID`, `APPLE_KEY_ID`, `APPLE_PRIVATE_KEY`, `APPLE_REDIRECT_URI`
- Spotify: `SPOTIFY_CLIENT_ID`, `SPOTIFY_CLIENT_SECRET`, `SPOTIFY_REDIRECT_URI`
- Tidal: `TIDAL_CLIENT_ID`, `TIDAL_CLIENT_SECRET`, `TIDAL_REDIRECT_URI`
- YouTube Music: `YOUTUBE_MUSIC_CLIENT_ID`, `YOUTUBE_MUSIC_CLIENT_SECRET`, `YOUTUBE_MUSIC_REDIRECT_URI`

## Custom-domain cutover

`render.yaml` now declares the custom domains directly:

- frontend service: `nodrakeinthe.house`
- backend service: `api.nodrakeinthe.house`

After the Blueprint sync:

1. Open both services in the Render dashboard and confirm the custom domains are attached.
2. Add DNS records with your DNS provider:
   - apex/root `nodrakeinthe.house` -> `ndith-frontend.onrender.com`
   - `api.nodrakeinthe.house` CNAME -> `ndith-backend.onrender.com`
3. Wait for Render domain verification and TLS issuance.
4. Update the backend URL-sensitive variables:
   - `OAUTH_FRONTEND_BASE_URL=https://nodrakeinthe.house`
   - `CORS_ALLOWED_ORIGINS=https://nodrakeinthe.house`
5. Redeploy the backend after the env change.

If your DNS provider supports ALIAS/ANAME or CNAME flattening for apex domains, use that for `nodrakeinthe.house`. If it does not, use the exact apex-record instructions Render shows in the dashboard for the frontend service.

On Hobby workspaces, `nodrakeinthe.house` and `api.nodrakeinthe.house` consume both available custom-domain slots.

## OAuth callback URLs

Provider redirect URIs should continue to target the backend custom domain. The backend receives the provider callback, then redirects the browser into the SPA callback route on the apex domain.

Use these values after custom domains are active:

- Google: `https://api.nodrakeinthe.house/auth/callback/google`
- GitHub: `https://api.nodrakeinthe.house/auth/callback/github`
- Apple: `https://api.nodrakeinthe.house/auth/callback/apple`
- Spotify: `https://api.nodrakeinthe.house/auth/callback/spotify`
- Tidal: `https://api.nodrakeinthe.house/auth/callback/tidal`
- YouTube Music: `https://api.nodrakeinthe.house/auth/callback/youtube`

The frontend callback base should be:

- `OAUTH_FRONTEND_BASE_URL=https://nodrakeinthe.house`

## Post-deploy checks

Run these checks after the first deploy on `.onrender.com`, then repeat them after custom-domain cutover:

```bash
curl -I https://ndith-frontend.onrender.com/
curl -fsS https://ndith-frontend.onrender.com/api/health
curl -fsS https://ndith-frontend.onrender.com/api/health/ready
curl -fsS https://ndith-backend.onrender.com/health
```

Final custom-domain rollout checks:

```bash
curl -I https://nodrakeinthe.house/
curl -fsS https://nodrakeinthe.house/api/health
curl -fsS https://nodrakeinthe.house/api/health/ready
curl -fsS https://api.nodrakeinthe.house/health
```

Smoke test:

```bash
BACKEND_URL=https://api.nodrakeinthe.house \
FRONTEND_URL=https://nodrakeinthe.house \
./scripts/render-smoke-test.sh
```

If the backend custom domain has not propagated yet, you can temporarily run the smoke test against the default backend hostname:

```bash
BACKEND_URL=https://ndith-backend.onrender.com \
FRONTEND_URL=https://nodrakeinthe.house \
./scripts/render-smoke-test.sh
```

## Notes

- The frontend is now a Docker-based web service instead of a Render static site.
- The frontend image serves the SPA and proxies backend traffic over Render's private network using `BACKEND_HOSTPORT`.
- `render.yaml` still points the backend at `docker.io/kcirtapfromspace/ndith-backend:latest`, which is maintained by the GitHub Actions publish workflow.
- The backend health contract on Render remains `/health`, `/health/ready`, and `/metrics` on port `3000`.
- The frontend health check path on Render is `/render-health`.
