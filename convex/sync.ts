import { ConvexError, v } from "convex/values";
import { action, mutation, query } from "./_generated/server";
import { nowIso, requireCurrentUser } from "./lib/auth";

export const status = query({
  args: {},
  handler: async (ctx) => {
    const { user } = await requireCurrentUser(ctx);
    const runs = await ctx.db
      .query("platformSyncRuns")
      .withIndex("by_status", (q) => q.eq("status", "running"))
      .collect();

    const latestByPlatform = new Map<string, any>();
    for (const run of runs) {
      const existing = latestByPlatform.get(run.platform);
      if (
        !existing ||
        (run.startedAt ?? run.createdAt) >
          (existing.startedAt ?? existing.createdAt)
      ) {
        latestByPlatform.set(run.platform, run);
      }
    }

    return {
      active_syncs: runs.length,
      platforms: Object.fromEntries(
        [...latestByPlatform.entries()].map(([platform, run]) => [
          platform,
          {
            status: run.status,
            started_at: run.startedAt,
            platform: run.platform,
          },
        ]),
      ),
    };
  },
});

export const listRuns = query({
  args: {
    platform: v.optional(v.string()),
    limit: v.optional(v.number()),
  },
  handler: async (ctx, args) => {
    let runsQuery = args.platform
      ? ctx.db
          .query("platformSyncRuns")
          .withIndex("by_platform", (q) => q.eq("platform", args.platform!))
      : ctx.db.query("platformSyncRuns");

    const allRuns = await runsQuery.collect();
    allRuns.sort((a, b) =>
      (b.startedAt ?? b.createdAt).localeCompare(a.startedAt ?? a.createdAt),
    );

    const limited = allRuns.slice(0, args.limit ?? 20);

    return {
      runs: limited.map((r) => ({
        id: r._id,
        platform: r.platform,
        status: r.status,
        started_at: r.startedAt,
        completed_at: r.completedAt,
        error_log: r.errorLog,
        metadata: r.metadata,
        created_at: r.createdAt,
      })),
      total: allRuns.length,
    };
  },
});

export const getRun = query({
  args: {
    runId: v.id("platformSyncRuns"),
  },
  handler: async (ctx, args) => {
    const run = await ctx.db.get(args.runId);
    if (!run) return null;

    return {
      id: run._id,
      platform: run.platform,
      status: run.status,
      started_at: run.startedAt,
      completed_at: run.completedAt,
      error_log: run.errorLog,
      checkpoint_data: run.checkpointData,
      metadata: run.metadata,
      created_at: run.createdAt,
    };
  },
});

export const health = query({
  args: {},
  handler: async (ctx) => {
    const allRuns = await ctx.db.query("platformSyncRuns").collect();
    const recentRuns = allRuns
      .sort((a, b) =>
        (b.startedAt ?? b.createdAt).localeCompare(
          a.startedAt ?? a.createdAt,
        ),
      )
      .slice(0, 50);

    const failedRecent = recentRuns.filter((r) => r.status === "failed").length;
    const runningCount = allRuns.filter((r) => r.status === "running").length;

    return {
      healthy: failedRecent < 5 && runningCount < 10,
      total_runs: allRuns.length,
      running: runningCount,
      recent_failures: failedRecent,
      last_run: recentRuns[0]
        ? {
            platform: recentRuns[0].platform,
            status: recentRuns[0].status,
            started_at: recentRuns[0].startedAt,
          }
        : null,
    };
  },
});

export const cancelRun = mutation({
  args: {
    runId: v.id("platformSyncRuns"),
  },
  handler: async (ctx, args) => {
    const run = await ctx.db.get(args.runId);
    if (!run) {
      throw new ConvexError("Sync run not found.");
    }

    if (run.status !== "running") {
      throw new ConvexError("Can only cancel running sync operations.");
    }

    await ctx.db.patch(run._id, {
      status: "cancelled",
      completedAt: nowIso(),
      updatedAt: nowIso(),
    });

    return { success: true };
  },
});

export const triggerSync = action({
  args: {
    platform: v.string(),
  },
  handler: async (ctx, args) => {
    const runId = await ctx.runMutation(
      "sync:_createRun" as any,
      { platform: args.platform },
    );

    return {
      run_id: runId,
      platform: args.platform,
      status: "running",
      message: `Sync triggered for ${args.platform}`,
    };
  },
});

export const triggerProviderSync = action({
  args: {
    provider: v.string(),
  },
  handler: async (ctx, args) => {
    const runId = await ctx.runMutation(
      "sync:_createRun" as any,
      { platform: args.provider },
    );

    return {
      run_id: runId,
      provider: args.provider,
      status: "running",
      message: `Library sync triggered for ${args.provider}`,
    };
  },
});

export const _createRun = mutation({
  args: {
    platform: v.string(),
  },
  handler: async (ctx, args) => {
    const now = nowIso();
    const runId = await ctx.db.insert("platformSyncRuns", {
      legacyKey: `runtime:sync:${args.platform}:${Date.now()}`,
      platform: args.platform,
      status: "running",
      startedAt: now,
      errorLog: [],
      checkpointData: {},
      metadata: {},
      createdAt: now,
      updatedAt: now,
    });
    return runId;
  },
});
