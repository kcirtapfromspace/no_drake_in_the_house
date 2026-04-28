# OAuth Synthetic Probe Dry-Run Evidence

- generated_at: 2026-04-28T17:14:48.783Z
- provider_target: all
- dry_run: true
- records: 9

## spotify | login_callback_success | PASS

```json
{
  "provider": "spotify",
  "flow": "oauth_login_callback",
  "class": "login_callback_success",
  "last_success": "2026-04-28T17:14:48.790Z",
  "status": "pass",
  "timestamp": "2026-04-28T17:14:48.790Z",
  "probe_id": "spotify:oauth_login_callback:login_callback_success",
  "simulation": true,
  "simulation_label": "deterministic.mock.login_callback_success",
  "details": {
    "assertion": "callback path resolves to success redirect handling",
    "deterministic": true,
    "reason": "dry-run deterministic safety"
  }
}
```

## spotify | token_refresh_failure_class | PASS

```json
{
  "provider": "spotify",
  "flow": "oauth_token_refresh",
  "class": "token_refresh_failure_class",
  "last_success": "2026-04-28T17:14:48.790Z",
  "status": "pass",
  "timestamp": "2026-04-28T17:14:48.790Z",
  "probe_id": "spotify:oauth_token_refresh:token_refresh_failure_class",
  "simulation": true,
  "simulation_label": "deterministic.mock.token_refresh_failure_class",
  "details": {
    "simulated_error": "invalid_grant",
    "expected_classification": "token_refresh_failure_class",
    "deterministic": true,
    "reason": "real provider token invalidation is unsafe in synthetic runs"
  }
}
```

## spotify | provider_unavailable_timeout | PASS

```json
{
  "provider": "spotify",
  "flow": "provider_api",
  "class": "provider_unavailable_timeout",
  "last_success": "2026-04-28T17:14:48.790Z",
  "status": "pass",
  "timestamp": "2026-04-28T17:14:48.790Z",
  "probe_id": "spotify:provider_api:provider_unavailable_timeout",
  "simulation": true,
  "simulation_label": "deterministic.mock.provider_unavailable_timeout",
  "details": {
    "simulated_transport": "timeout",
    "timeout_ms": 5000,
    "deterministic": true,
    "reason": "real provider outage simulation is unsafe"
  }
}
```

## apple | login_callback_success | PASS

```json
{
  "provider": "apple",
  "flow": "oauth_login_callback",
  "class": "login_callback_success",
  "last_success": "2026-04-28T17:14:48.790Z",
  "status": "pass",
  "timestamp": "2026-04-28T17:14:48.790Z",
  "probe_id": "apple:oauth_login_callback:login_callback_success",
  "simulation": true,
  "simulation_label": "deterministic.mock.login_callback_success",
  "details": {
    "assertion": "callback path resolves to success redirect handling",
    "deterministic": true,
    "reason": "dry-run deterministic safety"
  }
}
```

## apple | token_refresh_failure_class | PASS

```json
{
  "provider": "apple",
  "flow": "oauth_token_refresh",
  "class": "token_refresh_failure_class",
  "last_success": "2026-04-28T17:14:48.790Z",
  "status": "pass",
  "timestamp": "2026-04-28T17:14:48.790Z",
  "probe_id": "apple:oauth_token_refresh:token_refresh_failure_class",
  "simulation": true,
  "simulation_label": "deterministic.mock.token_refresh_failure_class",
  "details": {
    "simulated_error": "invalid_grant",
    "expected_classification": "token_refresh_failure_class",
    "deterministic": true,
    "reason": "real provider token invalidation is unsafe in synthetic runs"
  }
}
```

## apple | provider_unavailable_timeout | PASS

```json
{
  "provider": "apple",
  "flow": "provider_api",
  "class": "provider_unavailable_timeout",
  "last_success": "2026-04-28T17:14:48.790Z",
  "status": "pass",
  "timestamp": "2026-04-28T17:14:48.790Z",
  "probe_id": "apple:provider_api:provider_unavailable_timeout",
  "simulation": true,
  "simulation_label": "deterministic.mock.provider_unavailable_timeout",
  "details": {
    "simulated_transport": "timeout",
    "timeout_ms": 5000,
    "deterministic": true,
    "reason": "real provider outage simulation is unsafe"
  }
}
```

## tidal | login_callback_success | PASS

```json
{
  "provider": "tidal",
  "flow": "oauth_login_callback",
  "class": "login_callback_success",
  "last_success": "2026-04-28T17:14:48.790Z",
  "status": "pass",
  "timestamp": "2026-04-28T17:14:48.790Z",
  "probe_id": "tidal:oauth_login_callback:login_callback_success",
  "simulation": true,
  "simulation_label": "deterministic.mock.login_callback_success",
  "details": {
    "assertion": "callback path resolves to success redirect handling",
    "deterministic": true,
    "reason": "dry-run deterministic safety"
  }
}
```

## tidal | token_refresh_failure_class | PASS

```json
{
  "provider": "tidal",
  "flow": "oauth_token_refresh",
  "class": "token_refresh_failure_class",
  "last_success": "2026-04-28T17:14:48.790Z",
  "status": "pass",
  "timestamp": "2026-04-28T17:14:48.790Z",
  "probe_id": "tidal:oauth_token_refresh:token_refresh_failure_class",
  "simulation": true,
  "simulation_label": "deterministic.mock.token_refresh_failure_class",
  "details": {
    "simulated_error": "invalid_grant",
    "expected_classification": "token_refresh_failure_class",
    "deterministic": true,
    "reason": "real provider token invalidation is unsafe in synthetic runs"
  }
}
```

## tidal | provider_unavailable_timeout | PASS

```json
{
  "provider": "tidal",
  "flow": "provider_api",
  "class": "provider_unavailable_timeout",
  "last_success": "2026-04-28T17:14:48.790Z",
  "status": "pass",
  "timestamp": "2026-04-28T17:14:48.790Z",
  "probe_id": "tidal:provider_api:provider_unavailable_timeout",
  "simulation": true,
  "simulation_label": "deterministic.mock.provider_unavailable_timeout",
  "details": {
    "simulated_transport": "timeout",
    "timeout_ms": 5000,
    "deterministic": true,
    "reason": "real provider outage simulation is unsafe"
  }
}
```

