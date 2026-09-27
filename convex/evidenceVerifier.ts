import { v } from "convex/values";
import { action, internalAction } from "./_generated/server";
import { api, internal } from "./_generated/api";
import type { Id } from "./_generated/dataModel";

/** Acceptance is not verification. Clients follow the durable evaluation job. */
export const submitEvidence = action({
  args: { artistId: v.id("artists"), url: v.string(), category: v.optional(v.string()) },
  handler: async (ctx, args): Promise<{ jobId: Id<"evaluationJobs">; accepted: true; verified: false }> =>
    await ctx.runMutation(api.evaluations.submit, { artistId: args.artistId, url: args.url }),
});

export const verifyAndIngestUrl = internalAction({
  args: { artistId: v.id("artists"), url: v.string(), category: v.optional(v.string()), userId: v.optional(v.id("users")) },
  handler: async (ctx, args): Promise<{ jobId: Id<"evaluationJobs"> }> =>
    await ctx.runMutation(internal.evaluations.enqueueResearch, { artistId: args.artistId, url: args.url }),
});
