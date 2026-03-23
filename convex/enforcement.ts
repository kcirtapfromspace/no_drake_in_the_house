import { ConvexError, v } from "convex/values";
import type { Doc } from "./_generated/dataModel";
import { action, mutation, query } from "./_generated/server";
import { nowIso, requireCurrentUser } from "./lib/auth";

export const getProgress = query({
  args: {
    batchId: v.id("actionBatches"),
  },
  handler: async (ctx, args) => {
    const batch = await ctx.db.get(args.batchId);
    if (!batch) return null;

    const items = await ctx.db
      .query("actionItems")
      .withIndex("by_batchId", (q) => q.eq("batchId", args.batchId))
      .collect();

    const completed = items.filter((i) => i.status === "completed").length;
    const failed = items.filter((i) => i.status === "failed").length;
    const skipped = items.filter((i) => i.status === "skipped").length;

    return {
      id: batch._id,
      provider: batch.provider,
      status: batch.status,
      options: batch.options,
      summary: {
        totalItems: items.length,
        completedItems: completed,
        failedItems: failed,
        skippedItems: skipped,
      },
      items: items.map((i) => ({
        id: i._id,
        entityType: i.entityType,
        entityId: i.entityId,
        action: i.action,
        beforeState: i.beforeState,
        afterState: i.afterState,
        status: i.status,
        errorMessage: i.errorMessage,
      })),
      createdAt: batch.createdAt,
      completedAt: batch.completedAt,
    };
  },
});

export const history = query({
  args: {},
  handler: async (ctx) => {
    const { user } = await requireCurrentUser(ctx);
    const batches = await ctx.db
      .query("actionBatches")
      .withIndex("by_userId", (q) => q.eq("userId", user._id))
      .collect();

    batches.sort((a, b) => b.createdAt.localeCompare(a.createdAt));

    const withItems = await Promise.all(
      batches.map(async (batch) => {
        const items = await ctx.db
          .query("actionItems")
          .withIndex("by_batchId", (q) => q.eq("batchId", batch._id))
          .collect();

        const completed = items.filter((i) => i.status === "completed").length;
        const failed = items.filter((i) => i.status === "failed").length;
        const skipped = items.filter((i) => i.status === "skipped").length;

        return {
          id: batch._id,
          provider: batch.provider,
          status: batch.status,
          options: batch.options,
          summary: {
            totalItems: items.length,
            completedItems: completed,
            failedItems: failed,
            skippedItems: skipped,
          },
          items: items.map((i) => ({
            id: i._id,
            entityType: i.entityType,
            entityId: i.entityId,
            action: i.action,
            status: i.status,
            errorMessage: i.errorMessage,
          })),
          createdAt: batch.createdAt,
          completedAt: batch.completedAt,
        };
      }),
    );

    return withItems;
  },
});

export const planEnforcement = action({
  args: {
    providers: v.array(v.string()),
    options: v.any(),
    dryRun: v.optional(v.boolean()),
  },
  handler: async (ctx, args) => {
    const batchId = await ctx.runMutation(
      // @ts-expect-error -- internal reference
      "enforcement:_createBatch" as any,
      {
        provider: args.providers[0] ?? "spotify",
        options: args.options,
        dryRun: args.dryRun ?? true,
      },
    );

    return {
      planId: batchId,
      idempotencyKey: `plan-${batchId}`,
      impact: {},
      capabilities: {},
      estimatedDuration: "~2 minutes",
      resumable: true,
    };
  },
});

export const executePlan = action({
  args: {
    planId: v.string(),
    dryRun: v.optional(v.boolean()),
  },
  handler: async (ctx, args) => {
    await ctx.runMutation(
      // @ts-expect-error -- internal reference
      "enforcement:_updateBatchStatus" as any,
      { batchId: args.planId, status: "running" },
    );

    return {
      id: args.planId,
      status: "running",
      message: "Enforcement execution started",
    };
  },
});

export const rollback = action({
  args: {
    batchId: v.string(),
  },
  handler: async (ctx, args) => {
    await ctx.runMutation(
      // @ts-expect-error -- internal reference
      "enforcement:_updateBatchStatus" as any,
      { batchId: args.batchId, status: "rolled_back" },
    );

    return { success: true, message: "Rollback initiated" };
  },
});

export const _createBatch = mutation({
  args: {
    provider: v.string(),
    options: v.any(),
    dryRun: v.optional(v.boolean()),
  },
  handler: async (ctx, args) => {
    const { user } = await requireCurrentUser(ctx);
    const now = nowIso();

    const batchId = await ctx.db.insert("actionBatches", {
      legacyKey: `runtime:batch:${user._id}:${Date.now()}`,
      userId: user._id,
      provider: args.provider,
      idempotencyKey: `batch-${Date.now()}`,
      dryRun: args.dryRun,
      status: "pending",
      options: args.options,
      summary: {
        totalItems: 0,
        completedItems: 0,
        failedItems: 0,
        skippedItems: 0,
      },
      createdAt: now,
      updatedAt: now,
    });

    return batchId;
  },
});

export const _updateBatchStatus = mutation({
  args: {
    batchId: v.string(),
    status: v.string(),
  },
  handler: async (ctx, args) => {
    const batch = await ctx.db.get(args.batchId as any);
    if (!batch) {
      throw new ConvexError("Batch not found.");
    }

    const update: any = {
      status: args.status,
      updatedAt: nowIso(),
    };

    if (
      args.status === "completed" ||
      args.status === "failed" ||
      args.status === "rolled_back"
    ) {
      update.completedAt = nowIso();
    }

    await ctx.db.patch(batch._id, update);
    return { success: true };
  },
});
