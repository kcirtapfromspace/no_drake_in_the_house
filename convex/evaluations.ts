import { ConvexError, v } from "convex/values";
import { paginationOptsValidator } from "convex/server";
import { internalMutation, mutation, query, type MutationCtx } from "./_generated/server";
import { internal } from "./_generated/api";
import type { Id } from "./_generated/dataModel";
import { nowIso, requireCurrentUser } from "./lib/auth";
import { CATEGORIES, DEFAULT_JEV_MODEL, LEASE_MS, MAX_ATTEMPTS, MAX_SOURCE_LENGTH, POLICY_VERSION, SEVERITIES, PROCEDURAL_STATES, canonicalSourceUrl, isReviewer, normalizedSeverity } from "./lib/evaluationPolicy";
import { refreshArtistIndex } from "./offensePipeline";
import type { EvaluationDecision } from "./lib/jev";

type Intake = { artistId: Id<"artists">; url: string; sourceText?: string; sourceTitle?: string; userId?: Id<"users"> };

/** Quota, deduplication and durable scheduling are one transaction. */
export async function enqueueEvidence(ctx: MutationCtx, args: Intake): Promise<Id<"evaluationJobs">> {
  if (!await ctx.db.get(args.artistId)) throw new ConvexError("Artist not found.");
  const sourceUrl = canonicalSourceUrl(args.url);
  const model = process.env.TYPESAFE_MODEL || DEFAULT_JEV_MODEL;
  const key = `${POLICY_VERSION}:${model}:${args.userId ?? "research"}:${args.artistId}:${sourceUrl}`;
  const existing = await ctx.db.query("evaluationJobs").withIndex("by_key", (q) => q.eq("key", key)).unique();
  if (existing) {
    if (args.sourceText && !existing.sourceText && existing.status === "queued") {
      await ctx.db.patch(existing._id, { sourceText: args.sourceText.slice(0, MAX_SOURCE_LENGTH + 1), sourceTitle: args.sourceTitle?.slice(0, 500), updatedAt: nowIso() });
    }
    return existing._id;
  }
  if (args.userId) {
    const day = new Date().toISOString().slice(0, 10);
    const quota = await ctx.db.query("evaluationQuotas").withIndex("by_userId_and_day", (q) => q.eq("userId", args.userId!).eq("day", day)).unique();
    if ((quota?.attempts ?? 0) >= 10) throw new ConvexError("Maximum 10 evidence submissions per UTC day.");
    if (quota) await ctx.db.patch(quota._id, { attempts: quota.attempts + 1 });
    else await ctx.db.insert("evaluationQuotas", { userId: args.userId, day, attempts: 1 });
  }
  const now = nowIso();
  const jobId = await ctx.db.insert("evaluationJobs", {
    key, artistId: args.artistId, sourceUrl, sourceText: args.sourceText?.slice(0, MAX_SOURCE_LENGTH + 1), sourceTitle: args.sourceTitle?.slice(0, 500),
    submittedByUserId: args.userId, origin: args.userId ? "user" : "research",
    policyVersion: POLICY_VERSION, model, status: "queued", attempts: 0, generation: 0, createdAt: now, updatedAt: now,
  });
  await ctx.scheduler.runAfter(0, internal.evaluationWorker.run, { jobId });
  return jobId;
}

export const submit = mutation({
  args: { artistId: v.id("artists"), url: v.string() },
  handler: async (ctx, args): Promise<{ jobId: Id<"evaluationJobs">; accepted: true; verified: false }> => {
    const { user } = await requireCurrentUser(ctx);
    return { jobId: await enqueueEvidence(ctx, { ...args, userId: user._id }), accepted: true, verified: false };
  },
});
export const enqueueResearch = internalMutation({
  args: { artistId: v.id("artists"), url: v.string(), sourceText: v.optional(v.string()), sourceTitle: v.optional(v.string()) },
  handler: async (ctx, args): Promise<{ jobId: Id<"evaluationJobs"> }> => ({ jobId: await enqueueEvidence(ctx, args) }),
});

export const claim = internalMutation({
  args: { jobId: v.id("evaluationJobs") },
  handler: async (ctx, { jobId }) => {
    const job = await ctx.db.get(jobId);
    if (!job || !["queued", "retry_wait"].includes(job.status) || (job.nextAttemptAt ?? 0) > Date.now()) return null;
    const artist = await ctx.db.get(job.artistId);
    if (job.attempts >= MAX_ATTEMPTS || !artist) {
      await ctx.db.patch(jobId, { status: "failed", lastError: artist ? "attempts_exhausted" : "artist_missing", updatedAt: nowIso() });
      return null;
    }
    const generation = job.generation + 1;
    await ctx.db.patch(jobId, { status: "running", attempts: job.attempts + 1, generation, leaseUntil: Date.now() + LEASE_MS, nextAttemptAt: undefined, lastError: undefined, updatedAt: nowIso() });
    await ctx.scheduler.runAfter(LEASE_MS, internal.evaluations.recover, { jobId, generation });
    return { ...job, attempts: job.attempts + 1, generation, artist: { id: artist._id, name: artist.canonicalName, aliases: artist.aliases ?? [] } };
  },
});
export const saveSource = internalMutation({
  args: { jobId: v.id("evaluationJobs"), generation: v.number(), sourceText: v.string(), sourceHash: v.string(), sourceTitle: v.optional(v.string()), sourceTruncated: v.boolean() },
  handler: async (ctx, args) => {
    const job = await ctx.db.get(args.jobId);
    if (!job || job.status !== "running" || job.generation !== args.generation) return false;
    await ctx.db.patch(job._id, { sourceText: args.sourceText.slice(0, MAX_SOURCE_LENGTH), sourceHash: args.sourceHash, sourceTruncated: args.sourceTruncated, sourceTitle: args.sourceTitle?.slice(0, 500) ?? job.sourceTitle, updatedAt: nowIso() });
    return true;
  },
});
export const finish = internalMutation({
  args: { jobId: v.id("evaluationJobs"), generation: v.number(), decision: v.any() },
  handler: async (ctx, args) => {
    const job = await ctx.db.get(args.jobId);
    if (!job || job.status !== "running" || job.generation !== args.generation) return false;
    const decision = args.decision as EvaluationDecision;
    if (decision.policyVersion !== job.policyVersion || decision.model !== job.model || decision.sourceHash !== job.sourceHash || !["needs_review", "no_support"].includes(decision.route)) throw new ConvexError("Evaluation provenance mismatch.");
    if (decision.passage && job.sourceText?.slice(decision.passage.start, decision.passage.end) !== decision.passage.text) throw new ConvexError("Evidence passage mismatch.");
    await ctx.db.patch(job._id, { status: decision.route, decision, leaseUntil: undefined, lastError: undefined, updatedAt: nowIso() });
    return true;
  },
});
export const fail = internalMutation({
  args: { jobId: v.id("evaluationJobs"), generation: v.number(), code: v.string(), retryable: v.boolean(), retryAfterMs: v.optional(v.number()) },
  handler: async (ctx, args) => {
    const job = await ctx.db.get(args.jobId);
    if (!job || job.status !== "running" || job.generation !== args.generation) return false;
    const retry = args.retryable && job.attempts < MAX_ATTEMPTS;
    const delay = Math.max(1000 * 2 ** job.attempts, Math.min(args.retryAfterMs ?? 0, 3_600_000));
    await ctx.db.patch(job._id, { status: retry ? "retry_wait" : "failed", leaseUntil: undefined, nextAttemptAt: retry ? Date.now() + delay : undefined, lastError: args.code.slice(0, 100), updatedAt: nowIso() });
    if (retry) await ctx.scheduler.runAfter(delay, internal.evaluationWorker.run, { jobId: job._id });
    return true;
  },
});
export const recover = internalMutation({
  args: { jobId: v.id("evaluationJobs"), generation: v.number() },
  handler: async (ctx, { jobId, generation }) => {
    const job = await ctx.db.get(jobId);
    if (!job || job.status !== "running" || job.generation !== generation || (job.leaseUntil ?? 0) > Date.now()) return;
    const retry = job.attempts < MAX_ATTEMPTS;
    await ctx.db.patch(jobId, { status: retry ? "queued" : "failed", leaseUntil: undefined, lastError: "worker_lease_expired", updatedAt: nowIso() });
    if (retry) await ctx.scheduler.runAfter(0, internal.evaluationWorker.run, { jobId });
  },
});

export const get = query({
  args: { jobId: v.id("evaluationJobs") },
  handler: async (ctx, { jobId }) => {
    const { user } = await requireCurrentUser(ctx);
    const job = await ctx.db.get(jobId);
    if (!job || (job.submittedByUserId !== user._id && !isReviewer(user))) throw new ConvexError("Evaluation not found.");
    const { sourceText: _text, key: _key, ...view } = job;
    return view;
  },
});
export const listForArtist = query({
  args: { artistId: v.id("artists") },
  handler: async (ctx, { artistId }) => {
    const { user } = await requireCurrentUser(ctx);
    if (!isReviewer(user)) return [];
    return await ctx.db.query("evaluationJobs").withIndex("by_artistId", (q) => q.eq("artistId", artistId)).order("desc").take(30);
  },
});
export const reviewQueue = query({
  args: { paginationOpts: paginationOptsValidator },
  handler: async (ctx, args) => {
    const { user } = await requireCurrentUser(ctx);
    if (!isReviewer(user)) throw new ConvexError("Reviewer access required.");
    return await ctx.db.query("evaluationJobs").withIndex("by_status", (q) => q.eq("status", "needs_review")).paginate(args.paginationOpts);
  },
});
export const retry = mutation({
  args: { jobId: v.id("evaluationJobs") },
  handler: async (ctx, { jobId }) => {
    const { user } = await requireCurrentUser(ctx);
    if (!isReviewer(user)) throw new ConvexError("Reviewer access required.");
    const job = await ctx.db.get(jobId);
    if (!job || job.status !== "failed") throw new ConvexError("Only failed evaluations can be retried.");
    // Never reset generation: an old worker must not overwrite a manual retry.
    await ctx.db.patch(jobId, { status: "queued", attempts: 0, nextAttemptAt: undefined, lastError: undefined, updatedAt: nowIso() });
    await ctx.scheduler.runAfter(0, internal.evaluationWorker.run, { jobId });
  },
});
export const review = mutation({
  args: { jobId: v.id("evaluationJobs"), approve: v.boolean(), reason: v.string(), category: v.optional(v.string()), severity: v.optional(v.string()), proceduralState: v.optional(v.string()), excerpt: v.optional(v.string()) },
  handler: async (ctx, args) => {
    const { user } = await requireCurrentUser(ctx);
    if (!isReviewer(user)) throw new ConvexError("Reviewer access required.");
    const job = await ctx.db.get(args.jobId);
    if (!job || !["needs_review", "no_support"].includes(job.status)) throw new ConvexError("Evaluation is not awaiting review.");
    if (job.submittedByUserId === user._id) throw new ConvexError("An independent reviewer must review this submission.");
    const reason = args.reason.trim();
    if (reason.length < 10 || reason.length > 2000) throw new ConvexError("Give a review reason between 10 and 2000 characters.");
    const now = nowIso();
    let offenseId: Id<"artistOffenses"> | undefined;
    if (args.approve) {
      const decision = job.decision as EvaluationDecision | undefined;
      const excerpt = args.excerpt ?? decision?.passage?.text ?? "";
      const start = job.sourceText?.indexOf(excerpt) ?? -1;
      if (!decision || !job.sourceText || excerpt.trim().length < 20 || excerpt.length > 1200 || start < 0) throw new ConvexError("An exact supporting passage from the retained source is required (20–1200 characters).");
      if (!args.category || !Object.hasOwn(CATEGORIES, args.category) || !SEVERITIES.includes(args.severity as typeof SEVERITIES[number])) throw new ConvexError("Choose a category and severity.");
      if (!PROCEDURAL_STATES.includes(args.proceduralState as typeof PROCEDURAL_STATES[number])) throw new ConvexError("Confirm the procedural state reported by the source.");
      offenseId = await ctx.db.insert("artistOffenses", { legacyKey: `evaluation:${job._id}`, artistId: job.artistId, category: args.category,
        severity: normalizedSeverity(args.severity!), title: job.sourceTitle || `Reviewed source: ${args.category.replaceAll("_", " ")}`,
        description: excerpt, sourceArticleUrl: job.sourceUrl, status: "verified", proceduralState: args.proceduralState,
        submittedByUserId: job.submittedByUserId, verifiedByUserId: user._id, verifiedAt: now,
        metadata: { evaluationId: job._id, sourceHash: job.sourceHash, model: job.model, policyVersion: job.policyVersion, reviewReason: reason }, createdAt: now, updatedAt: now });
      await ctx.db.insert("offenseEvidence", { legacyKey: `evaluation:${job._id}:evidence`, offenseId, url: job.sourceUrl, title: job.sourceTitle,
        excerpt, sourceType: "reviewed_source", isPrimarySource: false, submittedByUserId: job.submittedByUserId,
        metadata: { evaluationId: job._id, sourceHash: job.sourceHash, start, end: start + excerpt.length }, createdAt: now, updatedAt: now });
      await refreshArtistIndex(ctx, job.artistId);
      await ctx.scheduler.runAfter(0, internal.offensePipeline.recomputeAffectedUsers, { artistId: job.artistId });
    }
    await ctx.db.patch(job._id, { status: args.approve ? "approved" : "rejected", reviewedByUserId: user._id, reviewedAt: now, reviewReason: reason, offenseId, updatedAt: now });
    return { offenseId: offenseId ?? null, status: args.approve ? "approved" : "rejected" };
  },
});
