# Convex Migration Runbook

This repository now contains the first production-facing Convex/Auth0 migration slice:

- Convex backend schema and domain functions live in [`/Users/thinkstudio/.codex/worktrees/cadd/no_drake_in_the_house/app/convex`](/Users/thinkstudio/.codex/worktrees/cadd/no_drake_in_the_house/app/convex).
- Postgres -> Convex import scripts live in [`/Users/thinkstudio/.codex/worktrees/cadd/no_drake_in_the_house/app/scripts/convex`](/Users/thinkstudio/.codex/worktrees/cadd/no_drake_in_the_house/app/scripts/convex).
- The Svelte web app can now authenticate with Auth0 and route core DNP/category/graph/connections reads through Convex.
- The browser extension now prefers Convex signed bloom-filter snapshots from `/extension/signed-update`.

## Environment

Root `.env` values:

- `DATABASE_URL`
- `CONVEX_URL`
- `CONVEX_DEPLOYMENT`
- `CONVEX_AUTH0_DOMAIN`
- `CONVEX_AUTH0_WEB_CLIENT_ID`
- `CONVEX_AUTH0_ANDROID_CLIENT_ID`
- `MIGRATION_API_KEY`
- `AUTH0_DOMAIN`
- `AUTH0_MANAGEMENT_CLIENT_ID`
- `AUTH0_MANAGEMENT_CLIENT_SECRET`
- `AUTH0_USER_IMPORT_CONNECTION_ID`
- `EXTENSION_SIGNING_PRIVATE_KEY_PEM`

Frontend `.env` values:

- `VITE_CONVEX_URL`
- `VITE_AUTH_MODE=auth0`
- `VITE_AUTH0_DOMAIN`
- `VITE_AUTH0_CLIENT_ID`
- `VITE_AUTH0_AUDIENCE`
- `VITE_AUTH0_SCOPE`
- `VITE_AUTH0_REDIRECT_PATH`
- `VITE_AUTH0_CONNECTION_GOOGLE`
- `VITE_AUTH0_CONNECTION_GITHUB`
- `VITE_AUTH0_CONNECTION_APPLE`
- `VITE_EXTENSION_SIGNED_UPDATE_URL`

## Commands

Generate Convex code after binding a deployment:

```bash
npm run convex:codegen
```

Export Auth0 bulk-import users from Postgres:

```bash
npm run migrate:auth0:users
```

Run the Postgres -> Convex importer:

```bash
npm run migrate:convex
```

Start the Convex dev environment:

```bash
npm run convex:dev
```

## Current Convex-backed UI coverage

- Auth0 session bootstrap and logout
- Current user sync into Convex `users`
- DNP search/list/add/update/remove
- Category list/subscription toggles
- Blocked artists by category
- Provider connection list/disconnect
- Graph search, collaborators, network, blocked-network analysis, stats, health
- Extension signed snapshot refresh

## Remaining cutover work

- Community lists, sync dashboards, analytics dashboards, enforcement execution, and service-health screens still contain legacy REST assumptions.
- Android is not switched to the Convex/Auth0 client yet.
- Provider OAuth linking flows for Spotify/Tidal/YouTube/Apple Music still need Convex action-backed replacements.
- `convex/_generated/*` is a checked-in bootstrap surface until a real deployment is configured and codegen is rerun.
- `npm run check` still reports a large number of pre-existing Svelte/test lint-type issues outside the new Convex/Auth0 path. `npm run build` succeeds.

## Recommended cutover sequence

1. Bind a real Convex deployment and rerun `npm run convex:codegen`.
2. Stand up Auth0 apps and verify Convex Auth config with staging tokens.
3. Import users into Auth0 and run the Convex importer into a shadow environment.
4. Verify DNP/category/graph parity in staging web and extension builds.
5. Port the remaining frontend stores and Android client.
6. Freeze writes, run a final delta import, rotate extension signing keys, and flip clients to Auth0 + Convex.
