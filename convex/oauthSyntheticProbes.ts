// Scheduled execution path for OAuth synthetic probes (NOD-185).
//
// This module is the in-Convex equivalent of the local CLI runner at
// scripts/oauth-probes/run.mjs. Both paths produce records that
// satisfy the same locked output contract: every record exposes
// provider, flow, class, last_success, status, and timestamp.
//
// The CLI runner is the developer one-shot dry-run entrypoint; this
// internal action is the deterministic scheduled entrypoint invoked by
// convex/crons.ts. By mirroring the same definitions and deterministic
// execution logic, scheduled mode never diverges from dry-run.
//
// No alerting/paging is wired here — records are emitted as structured
// logs (observable via `npx convex logs`) and returned to the caller.
import { v } from "convex/values";
import { internalAction } from "./_generated/server";

type ProbeProvider = "spotify" | "apple" | "tidal";
type ProbeFlow =
  | "oauth_login_callback"
  | "oauth_token_refresh"
  | "provider_api";
type ProbeClass =
  | "login_callback_success"
  | "token_refresh_failure_class"
  | "provider_unavailable_timeout";

type ProbeDefinition = {
  id: string;
  provider: ProbeProvider;
  flow: ProbeFlow;
  class: ProbeClass;
  strategy: "deterministic";
};

const DEFINITIONS: ProbeDefinition[] = [
  {
    id: "spotify:oauth_login_callback:login_callback_success",
    provider: "spotify",
    flow: "oauth_login_callback",
    class: "login_callback_success",
    strategy: "deterministic",
  },
  {
    id: "spotify:oauth_token_refresh:token_refresh_failure_class",
    provider: "spotify",
    flow: "oauth_token_refresh",
    class: "token_refresh_failure_class",
    strategy: "deterministic",
  },
  {
    id: "spotify:provider_api:provider_unavailable_timeout",
    provider: "spotify",
    flow: "provider_api",
    class: "provider_unavailable_timeout",
    strategy: "deterministic",
  },
  {
    id: "apple:oauth_login_callback:login_callback_success",
    provider: "apple",
    flow: "oauth_login_callback",
    class: "login_callback_success",
    strategy: "deterministic",
  },
  {
    id: "apple:oauth_token_refresh:token_refresh_failure_class",
    provider: "apple",
    flow: "oauth_token_refresh",
    class: "token_refresh_failure_class",
    strategy: "deterministic",
  },
  {
    id: "apple:provider_api:provider_unavailable_timeout",
    provider: "apple",
    flow: "provider_api",
    class: "provider_unavailable_timeout",
    strategy: "deterministic",
  },
  {
    id: "tidal:oauth_login_callback:login_callback_success",
    provider: "tidal",
    flow: "oauth_login_callback",
    class: "login_callback_success",
    strategy: "deterministic",
  },
  {
    id: "tidal:oauth_token_refresh:token_refresh_failure_class",
    provider: "tidal",
    flow: "oauth_token_refresh",
    class: "token_refresh_failure_class",
    strategy: "deterministic",
  },
  {
    id: "tidal:provider_api:provider_unavailable_timeout",
    provider: "tidal",
    flow: "provider_api",
    class: "provider_unavailable_timeout",
    strategy: "deterministic",
  },
];

type ProbeExecution = {
  status: "pass" | "fail";
  simulationLabel: string;
  details: Record<string, unknown>;
};

function executeDefinition(definition: ProbeDefinition): ProbeExecution {
  switch (definition.class) {
    case "login_callback_success":
      return {
        status: "pass",
        simulationLabel: "deterministic.mock.login_callback_success",
        details: {
          assertion: "callback path resolves to success redirect handling",
          deterministic: true,
          reason: "scheduled deterministic safety",
        },
      };
    case "token_refresh_failure_class":
      return {
        status: "pass",
        simulationLabel: "deterministic.mock.token_refresh_failure_class",
        details: {
          simulated_error: "invalid_grant",
          expected_classification: "token_refresh_failure_class",
          deterministic: true,
          reason: "real provider token invalidation is unsafe in synthetic runs",
        },
      };
    case "provider_unavailable_timeout":
      return {
        status: "pass",
        simulationLabel: "deterministic.mock.provider_unavailable_timeout",
        details: {
          simulated_transport: "timeout",
          timeout_ms: 5000,
          deterministic: true,
          reason: "real provider outage simulation is unsafe",
        },
      };
  }
}

function filterDefinitions(target: ProbeProvider | "all"): ProbeDefinition[] {
  if (target === "all") {
    return DEFINITIONS;
  }
  return DEFINITIONS.filter((definition) => definition.provider === target);
}

export const runProbes = internalAction({
  args: {
    provider: v.optional(
      v.union(
        v.literal("spotify"),
        v.literal("apple"),
        v.literal("tidal"),
        v.literal("all"),
      ),
    ),
  },
  handler: async (_ctx, args) => {
    const target: ProbeProvider | "all" = args.provider ?? "all";
    const definitions = filterDefinitions(target);
    const generatedAt = new Date().toISOString();

    const results = definitions.map((definition) => {
      const timestamp = new Date().toISOString();
      const execution = executeDefinition(definition);
      const lastSuccess = execution.status === "pass" ? timestamp : null;
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
    });

    const payload = {
      generated_at: generatedAt,
      dry_run: false,
      provider_target: target,
      probe_count: results.length,
      results,
    };

    // Structured log line per probe — readable in `npx convex logs` for
    // operational verification without any alerting/paging.
    for (const result of results) {
      console.log(
        `oauth_synthetic_probe ${JSON.stringify({
          provider: result.provider,
          flow: result.flow,
          class: result.class,
          last_success: result.last_success,
          status: result.status,
          timestamp: result.timestamp,
          probe_id: result.probe_id,
          simulation_label: result.simulation_label,
        })}`,
      );
    }
    console.log(
      `oauth_synthetic_probe_run ${JSON.stringify({
        generated_at: payload.generated_at,
        provider_target: payload.provider_target,
        probe_count: payload.probe_count,
      })}`,
    );

    return payload;
  },
});
