# OAuth Synthetic Probe Evidence

- generated_at: 2026-04-28T17:38:45.661Z
- provider_target: all
- records: 9

## spotify | login_callback_success | PASS

```json
{
  "provider": "spotify",
  "flow": "oauth_login_callback",
  "class": "login_callback_success",
  "last_success": "2026-04-28T17:38:45.662Z",
  "status": "pass",
  "timestamp": "2026-04-28T17:38:45.662Z",
  "probe_id": "spotify:oauth_login_callback:login_callback_success",
  "simulation": true,
  "simulation_label": "deterministic.classifier.login_callback_success",
  "details": {
    "expected_classification": "login_callback_success",
    "actual_classification": "login_callback_success",
    "signal": {
      "kind": "callback_response",
      "success": true
    },
    "deterministic": true
  }
}
```

## spotify | token_refresh_failure_class | PASS

```json
{
  "provider": "spotify",
  "flow": "oauth_token_refresh",
  "class": "token_refresh_failure_class",
  "last_success": "2026-04-28T17:38:45.662Z",
  "status": "pass",
  "timestamp": "2026-04-28T17:38:45.662Z",
  "probe_id": "spotify:oauth_token_refresh:token_refresh_failure_class",
  "simulation": true,
  "simulation_label": "deterministic.classifier.token_refresh_failure_class",
  "details": {
    "expected_classification": "token_refresh_failure_class",
    "actual_classification": "token_refresh_failure_class",
    "signal": {
      "kind": "token_refresh_response",
      "error": "invalid_grant"
    },
    "deterministic": true
  }
}
```

## spotify | provider_unavailable_timeout | PASS

```json
{
  "provider": "spotify",
  "flow": "provider_api",
  "class": "provider_unavailable_timeout",
  "last_success": "2026-04-28T17:38:45.662Z",
  "status": "pass",
  "timestamp": "2026-04-28T17:38:45.662Z",
  "probe_id": "spotify:provider_api:provider_unavailable_timeout",
  "simulation": true,
  "simulation_label": "deterministic.classifier.provider_unavailable_timeout",
  "details": {
    "expected_classification": "provider_unavailable_timeout",
    "actual_classification": "provider_unavailable_timeout",
    "signal": {
      "kind": "provider_request_outcome",
      "outcome": "timeout"
    },
    "deterministic": true
  }
}
```

## apple | login_callback_success | PASS

```json
{
  "provider": "apple",
  "flow": "oauth_login_callback",
  "class": "login_callback_success",
  "last_success": "2026-04-28T17:38:45.662Z",
  "status": "pass",
  "timestamp": "2026-04-28T17:38:45.662Z",
  "probe_id": "apple:oauth_login_callback:login_callback_success",
  "simulation": true,
  "simulation_label": "deterministic.classifier.login_callback_success",
  "details": {
    "expected_classification": "login_callback_success",
    "actual_classification": "login_callback_success",
    "signal": {
      "kind": "callback_response",
      "success": true
    },
    "deterministic": true
  }
}
```

## apple | token_refresh_failure_class | PASS

```json
{
  "provider": "apple",
  "flow": "oauth_token_refresh",
  "class": "token_refresh_failure_class",
  "last_success": "2026-04-28T17:38:45.662Z",
  "status": "pass",
  "timestamp": "2026-04-28T17:38:45.662Z",
  "probe_id": "apple:oauth_token_refresh:token_refresh_failure_class",
  "simulation": true,
  "simulation_label": "deterministic.classifier.token_refresh_failure_class",
  "details": {
    "expected_classification": "token_refresh_failure_class",
    "actual_classification": "token_refresh_failure_class",
    "signal": {
      "kind": "token_refresh_response",
      "error": "invalid_grant"
    },
    "deterministic": true
  }
}
```

## apple | provider_unavailable_timeout | PASS

```json
{
  "provider": "apple",
  "flow": "provider_api",
  "class": "provider_unavailable_timeout",
  "last_success": "2026-04-28T17:38:45.662Z",
  "status": "pass",
  "timestamp": "2026-04-28T17:38:45.662Z",
  "probe_id": "apple:provider_api:provider_unavailable_timeout",
  "simulation": true,
  "simulation_label": "deterministic.classifier.provider_unavailable_timeout",
  "details": {
    "expected_classification": "provider_unavailable_timeout",
    "actual_classification": "provider_unavailable_timeout",
    "signal": {
      "kind": "provider_request_outcome",
      "outcome": "timeout"
    },
    "deterministic": true
  }
}
```

## tidal | login_callback_success | PASS

```json
{
  "provider": "tidal",
  "flow": "oauth_login_callback",
  "class": "login_callback_success",
  "last_success": "2026-04-28T17:38:45.662Z",
  "status": "pass",
  "timestamp": "2026-04-28T17:38:45.662Z",
  "probe_id": "tidal:oauth_login_callback:login_callback_success",
  "simulation": true,
  "simulation_label": "deterministic.classifier.login_callback_success",
  "details": {
    "expected_classification": "login_callback_success",
    "actual_classification": "login_callback_success",
    "signal": {
      "kind": "callback_response",
      "success": true
    },
    "deterministic": true
  }
}
```

## tidal | token_refresh_failure_class | PASS

```json
{
  "provider": "tidal",
  "flow": "oauth_token_refresh",
  "class": "token_refresh_failure_class",
  "last_success": "2026-04-28T17:38:45.662Z",
  "status": "pass",
  "timestamp": "2026-04-28T17:38:45.662Z",
  "probe_id": "tidal:oauth_token_refresh:token_refresh_failure_class",
  "simulation": true,
  "simulation_label": "deterministic.classifier.token_refresh_failure_class",
  "details": {
    "expected_classification": "token_refresh_failure_class",
    "actual_classification": "token_refresh_failure_class",
    "signal": {
      "kind": "token_refresh_response",
      "error": "invalid_grant"
    },
    "deterministic": true
  }
}
```

## tidal | provider_unavailable_timeout | PASS

```json
{
  "provider": "tidal",
  "flow": "provider_api",
  "class": "provider_unavailable_timeout",
  "last_success": "2026-04-28T17:38:45.662Z",
  "status": "pass",
  "timestamp": "2026-04-28T17:38:45.662Z",
  "probe_id": "tidal:provider_api:provider_unavailable_timeout",
  "simulation": true,
  "simulation_label": "deterministic.classifier.provider_unavailable_timeout",
  "details": {
    "expected_classification": "provider_unavailable_timeout",
    "actual_classification": "provider_unavailable_timeout",
    "signal": {
      "kind": "provider_request_outcome",
      "outcome": "timeout"
    },
    "deterministic": true
  }
}
```

