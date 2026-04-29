# Spotify Post-Connect Guidance Canary

Issue: [NOD-229](/NOD/issues/NOD-229)

## Scope
- Provider: Spotify only (Spotify-first canary)
- Surface: Sync dashboard in-product guidance card after successful Spotify connect
- Completion path: `connect -> first successful sync`

## Rollout Controls
Frontend env vars:

- `VITE_ENABLE_POST_CONNECT_SPOTIFY_GUIDANCE_CANARY`
  - `false` = kill switch (guidance disabled for all users)
  - `true` = canary enabled, cohort decided by rollout percent
- `VITE_POST_CONNECT_SPOTIFY_GUIDANCE_ROLLOUT_PERCENT`
  - Integer `0..100`
  - User cohort is deterministic via stable bucket hashing

Recommended launch progression:

1. Set `VITE_ENABLE_POST_CONNECT_SPOTIFY_GUIDANCE_CANARY=true`
2. Start with `VITE_POST_CONNECT_SPOTIFY_GUIDANCE_ROLLOUT_PERCENT=10`
3. Hold for one canary window and inspect signals
4. Increase to `25`, then `50`, then `100` only if no regression signals

## Instrumentation Events (PostHog)

- `spotify_post_connect_connected`
  - Fired on successful Spotify callback
  - Properties: `provider`, `source`, `connected_at`, `flow`

- `spotify_post_connect_guidance_shown`
  - Fired when canary guidance UI is first shown
  - Properties: `provider`, `source`, `connected_at`, `rollout_percent`, `canary_bucket`

- `spotify_post_connect_guidance_sync_started`
  - Fired when first Spotify sync starts after connect marker exists
  - Properties: `provider`, `source`, `connected_at`, `first_sync_started_at`

- `spotify_post_connect_guidance_sync_completed`
  - Fired when first Spotify sync reaches completed state
  - Properties: `provider`, `source`, `connected_at`, `first_sync_started_at`, `completed_at`, `elapsed_ms`

- `spotify_post_connect_guidance_dismissed`
  - Fired when guidance card is dismissed
  - Properties: `provider`, `source`, `connected_at`

## Canary Monitoring Checks

Track these during the canary window:

1. Completion trend for `connect -> first successful sync`
2. Sync error-rate trend for Spotify library sync endpoints
3. Reauth/reconnect signal trend (`failed_auth`, `needs_reauth`, repeated callback failures)

## Rollback Criteria

Immediately stop rollout (set `VITE_ENABLE_POST_CONNECT_SPOTIFY_GUIDANCE_CANARY=false`) if any occur:

1. Incident-shaped Spotify sync failure pattern reappears
2. Reliability owner flags regression relative to post-fix [NOD-142](/NOD/issues/NOD-142) posture
3. Material drop in completion trend during canary window with no confounding external incident

## Rollback Procedure

1. Set `VITE_ENABLE_POST_CONNECT_SPOTIFY_GUIDANCE_CANARY=false`
2. Deploy frontend config change
3. Confirm `spotify_post_connect_guidance_shown` volume drops to zero
4. Keep connected/sync monitoring active until stable
