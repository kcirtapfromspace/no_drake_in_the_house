// Shared source of truth for OAuth synthetic probes.
//
// Both execution paths import from here so the locked CTO constraint
// (scheduled mode must not fork from dry-run) is enforced at the type
// and data level rather than relying on review:
//
//   - CLI dry-run runner:   scripts/oauth-probes/run.ts
//   - Scheduled action:     convex/oauthSyntheticProbes.ts
//
// The output contract every probe record satisfies:
//   provider, flow, class, last_success, status, timestamp
//
// `class` is the failure class the probe is simulating. `status` is
// derived by feeding the probe's deterministic simulated signal through
// the canonical classifier and asserting the classifier returns the
// expected class. If the classifier ever drifts away from the probe
// expectation, status flips to "fail" — that is the only SLO signal
// these synthetic probes carry.

export type ProbeProvider = "spotify" | "apple" | "tidal";

export type ProbeFlow =
  | "oauth_login_callback"
  | "oauth_token_refresh"
  | "provider_api";

export type ProbeClass =
  | "login_callback_success"
  | "token_refresh_failure_class"
  | "provider_unavailable_timeout";

// Deterministic, side-effect-free signal each probe feeds into the
// canonical classifier. The shape is intentionally narrow: real probe
// telemetry would include more fields, but the classifier only consumes
// what it needs to assert routing.
export type ClassifierSignal =
  | { kind: "callback_response"; success: boolean }
  | { kind: "token_refresh_response"; error: string | null }
  | { kind: "provider_request_outcome"; outcome: "ok" | "timeout" | "error" };

export type ProbeDefinition = {
  id: string;
  provider: ProbeProvider;
  flow: ProbeFlow;
  class: ProbeClass;
  strategy: "deterministic";
  signal: ClassifierSignal;
};

const PROVIDERS: ReadonlyArray<ProbeProvider> = ["spotify", "apple", "tidal"];

function buildProviderDefinitions(provider: ProbeProvider): ProbeDefinition[] {
  return [
    {
      id: `${provider}:oauth_login_callback:login_callback_success`,
      provider,
      flow: "oauth_login_callback",
      class: "login_callback_success",
      strategy: "deterministic",
      signal: { kind: "callback_response", success: true },
    },
    {
      id: `${provider}:oauth_token_refresh:token_refresh_failure_class`,
      provider,
      flow: "oauth_token_refresh",
      class: "token_refresh_failure_class",
      strategy: "deterministic",
      signal: { kind: "token_refresh_response", error: "invalid_grant" },
    },
    {
      id: `${provider}:provider_api:provider_unavailable_timeout`,
      provider,
      flow: "provider_api",
      class: "provider_unavailable_timeout",
      strategy: "deterministic",
      signal: { kind: "provider_request_outcome", outcome: "timeout" },
    },
  ];
}

export const PROBE_DEFINITIONS: ProbeDefinition[] = PROVIDERS.flatMap(
  buildProviderDefinitions,
);

export const REQUIRED_RESULT_FIELDS = [
  "provider",
  "flow",
  "class",
  "last_success",
  "status",
  "timestamp",
] as const;

// Canonical classifier. This intentionally lives next to the probe
// definitions so any change to error routing forces a corresponding
// change here, which the probes will flag if they disagree. The Rust
// counterpart in backend/crates/ndith-core/src/error/oauth.rs is the
// production-path classifier; this is the shared synthetic-path mirror
// scoped to the three classes the probes care about.
export function classifyOAuthSignal(
  signal: ClassifierSignal,
): ProbeClass | null {
  switch (signal.kind) {
    case "callback_response":
      return signal.success ? "login_callback_success" : null;
    case "token_refresh_response": {
      // Mirrors parse_provider_error in oauth.rs: invalid_grant /
      // bad_verification_code / expired_token funnel to the token
      // refresh failure class for synthetic-probe purposes.
      const TOKEN_REFRESH_FAILURE_CODES = new Set([
        "invalid_grant",
        "bad_verification_code",
        "expired_token",
        "invalid_token",
      ]);
      if (signal.error && TOKEN_REFRESH_FAILURE_CODES.has(signal.error)) {
        return "token_refresh_failure_class";
      }
      return null;
    }
    case "provider_request_outcome":
      return signal.outcome === "timeout"
        ? "provider_unavailable_timeout"
        : null;
  }
}

export type ProbeStatus = "pass" | "fail";

export type ProbeExecutionDetails = {
  expected_classification: ProbeClass;
  actual_classification: ProbeClass | null;
  signal: ClassifierSignal;
  deterministic: true;
};

export type ProbeExecution = {
  status: ProbeStatus;
  simulationLabel: string;
  details: ProbeExecutionDetails;
};

export function executeDefinition(definition: ProbeDefinition): ProbeExecution {
  const actual = classifyOAuthSignal(definition.signal);
  const status: ProbeStatus = actual === definition.class ? "pass" : "fail";
  return {
    status,
    simulationLabel: `deterministic.classifier.${definition.class}`,
    details: {
      expected_classification: definition.class,
      actual_classification: actual,
      signal: definition.signal,
      deterministic: true,
    },
  };
}

export function filterDefinitions(
  target: ProbeProvider | "all",
): ProbeDefinition[] {
  if (target === "all") {
    return PROBE_DEFINITIONS;
  }
  return PROBE_DEFINITIONS.filter((def) => def.provider === target);
}

export type ProbeRecord = {
  provider: ProbeProvider;
  flow: ProbeFlow;
  class: ProbeClass;
  last_success: string | null;
  status: ProbeStatus;
  timestamp: string;
  probe_id: string;
  simulation: true;
  simulation_label: string;
  details: ProbeExecutionDetails;
};

export function buildRecord(
  definition: ProbeDefinition,
  timestamp: string,
  previousLastSuccess: string | null,
): ProbeRecord {
  const execution = executeDefinition(definition);
  const lastSuccess =
    execution.status === "pass" ? timestamp : previousLastSuccess;
  return {
    provider: definition.provider,
    flow: definition.flow,
    class: definition.class,
    last_success: lastSuccess,
    status: execution.status,
    timestamp,
    probe_id: definition.id,
    simulation: true,
    simulation_label: execution.simulationLabel,
    details: execution.details,
  };
}
