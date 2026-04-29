/// <reference types="vite/client" />
import { describe, expect, test } from "vitest";

import {
  buildRecord,
  classifyOAuthSignal,
  filterDefinitions,
  PROBE_DEFINITIONS,
  REQUIRED_RESULT_FIELDS,
  type ProbeRecord,
} from "./lib/oauthSyntheticProbes";

const PROVIDERS = ["spotify", "apple", "tidal"] as const;
const CLASSES = [
  "login_callback_success",
  "token_refresh_failure_class",
  "provider_unavailable_timeout",
] as const;

describe("oauth synthetic probe contract", () => {
  test("definitions cover every provider × class combination exactly once", () => {
    expect(PROBE_DEFINITIONS).toHaveLength(PROVIDERS.length * CLASSES.length);
    for (const provider of PROVIDERS) {
      for (const klass of CLASSES) {
        const matches = PROBE_DEFINITIONS.filter(
          (d) => d.provider === provider && d.class === klass,
        );
        expect(
          matches,
          `expected exactly one definition for ${provider}:${klass}`,
        ).toHaveLength(1);
      }
    }
  });

  test("probe ids are stable, unique, and follow provider:flow:class", () => {
    const seen = new Set<string>();
    for (const def of PROBE_DEFINITIONS) {
      expect(seen.has(def.id)).toBe(false);
      seen.add(def.id);
      expect(def.id).toBe(`${def.provider}:${def.flow}:${def.class}`);
    }
  });

  test("buildRecord emits every locked output-contract field", () => {
    const timestamp = "2026-04-28T17:00:00.000Z";
    for (const def of PROBE_DEFINITIONS) {
      const record = buildRecord(def, timestamp, null);
      for (const field of REQUIRED_RESULT_FIELDS) {
        expect(
          field in record,
          `record for ${def.id} missing required field ${field}`,
        ).toBe(true);
      }
    }
  });

  test("filterDefinitions narrows to a single provider", () => {
    for (const provider of PROVIDERS) {
      const subset = filterDefinitions(provider);
      expect(subset.length).toBe(CLASSES.length);
      for (const def of subset) {
        expect(def.provider).toBe(provider);
      }
    }
    expect(filterDefinitions("all")).toHaveLength(PROBE_DEFINITIONS.length);
  });
});

describe("oauth synthetic probe classifier", () => {
  test("classifier routes each definition's signal to the expected class", () => {
    for (const def of PROBE_DEFINITIONS) {
      expect(
        classifyOAuthSignal(def.signal),
        `classifier drifted from ${def.id}`,
      ).toBe(def.class);
    }
  });

  test("status reflects classifier output, not a hard-coded value", () => {
    const timestamp = "2026-04-28T17:00:00.000Z";

    // Happy path: the canonical signal must produce status=pass.
    for (const def of PROBE_DEFINITIONS) {
      const record = buildRecord(def, timestamp, null);
      expect(record.status).toBe("pass");
      expect(record.last_success).toBe(timestamp);
    }

    // Drift path: feed a signal that the classifier won't route to the
    // expected class, and assert status flips to fail. This is the SLO
    // signal the probes carry — a hard-coded "pass" would silently lose
    // it.
    const refreshDef = PROBE_DEFINITIONS.find(
      (d) => d.class === "token_refresh_failure_class",
    );
    expect(refreshDef).toBeDefined();
    const drifted = buildRecord(
      { ...refreshDef!, signal: { kind: "token_refresh_response", error: null } },
      timestamp,
      "2026-04-27T00:00:00.000Z",
    );
    expect(drifted.status).toBe("fail");
    // last_success must hold the prior success rather than reset.
    expect(drifted.last_success).toBe("2026-04-27T00:00:00.000Z");
  });

  test("classifier returns null for unrelated signals", () => {
    expect(
      classifyOAuthSignal({ kind: "callback_response", success: false }),
    ).toBeNull();
    expect(
      classifyOAuthSignal({ kind: "token_refresh_response", error: "rate_limited" }),
    ).toBeNull();
    expect(
      classifyOAuthSignal({ kind: "provider_request_outcome", outcome: "ok" }),
    ).toBeNull();
  });
});

describe("oauth synthetic probe last_success semantics", () => {
  test("preserves prior last_success on failure", () => {
    const refreshDef = PROBE_DEFINITIONS.find(
      (d) => d.class === "token_refresh_failure_class",
    )!;
    const driftedDef = {
      ...refreshDef,
      signal: { kind: "token_refresh_response", error: null } as const,
    };
    const prior = "2026-04-27T00:00:00.000Z";
    const record: ProbeRecord = buildRecord(
      driftedDef,
      "2026-04-28T00:00:00.000Z",
      prior,
    );
    expect(record.status).toBe("fail");
    expect(record.last_success).toBe(prior);
  });

  test("advances last_success to current timestamp on pass", () => {
    const def = PROBE_DEFINITIONS[0];
    const ts = "2026-05-01T00:00:00.000Z";
    const record = buildRecord(def, ts, "2026-04-30T00:00:00.000Z");
    expect(record.status).toBe("pass");
    expect(record.last_success).toBe(ts);
  });
});
