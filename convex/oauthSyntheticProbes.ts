// Scheduled execution path for OAuth synthetic probes (NOD-185).
//
// This module is the in-Convex equivalent of the local CLI runner at
// scripts/oauth-probes/run.ts. Both paths import the canonical probe
// definitions, classifier, and record builder from
// convex/lib/oauthSyntheticProbes.ts so scheduled mode cannot fork
// from dry-run.
//
// `last_success` is persisted in the oauthSyntheticProbeState table so
// it survives across scheduled runs (the CLI mirrors this in
// data/oauth-synthetic-probe-state.json). `status` is derived by the
// shared classifier — the only signal these synthetic probes carry is
// "did the canonical classifier still route the simulated input to the
// expected class?". No alerting/paging is wired here; records emit as
// structured logs and are returned to the caller.
import { v } from "convex/values";
import { internal } from "./_generated/api";
import {
  internalAction,
  internalMutation,
  internalQuery,
} from "./_generated/server";
import {
  buildRecord,
  filterDefinitions,
  type ProbeRecord,
} from "./lib/oauthSyntheticProbes";

export const getProbeState = internalQuery({
  args: { probeIds: v.array(v.string()) },
  handler: async (ctx, args) => {
    const entries: Record<string, string | null> = {};
    for (const probeId of args.probeIds) {
      const row = await ctx.db
        .query("oauthSyntheticProbeState")
        .withIndex("by_probeId", (q) => q.eq("probeId", probeId))
        .unique();
      entries[probeId] = row?.lastSuccessAt ?? null;
    }
    return entries;
  },
});

export const upsertProbeState = internalMutation({
  args: {
    runAt: v.string(),
    updates: v.array(
      v.object({
        probeId: v.string(),
        lastSuccessAt: v.union(v.string(), v.null()),
      }),
    ),
  },
  handler: async (ctx, args) => {
    for (const update of args.updates) {
      const existing = await ctx.db
        .query("oauthSyntheticProbeState")
        .withIndex("by_probeId", (q) => q.eq("probeId", update.probeId))
        .unique();
      const lastSuccessAt = update.lastSuccessAt ?? undefined;
      if (existing) {
        await ctx.db.patch(existing._id, {
          lastSuccessAt,
          lastRunAt: args.runAt,
        });
      } else {
        await ctx.db.insert("oauthSyntheticProbeState", {
          probeId: update.probeId,
          lastSuccessAt,
          lastRunAt: args.runAt,
        });
      }
    }
  },
});

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
  handler: async (ctx, args) => {
    const target = args.provider ?? "all";
    const definitions = filterDefinitions(target);
    const generatedAt = new Date().toISOString();

    const probeIds = definitions.map((d) => d.id);
    const previousState: Record<string, string | null> = await ctx.runQuery(
      internal.oauthSyntheticProbes.getProbeState,
      { probeIds },
    );

    const results: ProbeRecord[] = definitions.map((definition) => {
      const timestamp = new Date().toISOString();
      const previousLastSuccess = previousState[definition.id] ?? null;
      return buildRecord(definition, timestamp, previousLastSuccess);
    });

    await ctx.runMutation(internal.oauthSyntheticProbes.upsertProbeState, {
      runAt: generatedAt,
      updates: results.map((r) => ({
        probeId: r.probe_id,
        lastSuccessAt: r.last_success,
      })),
    });

    const payload = {
      generated_at: generatedAt,
      provider_target: target,
      probe_count: results.length,
      results,
    };

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
