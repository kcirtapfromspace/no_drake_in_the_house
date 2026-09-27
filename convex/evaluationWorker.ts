import { v } from "convex/values";
import { internalAction } from "./_generated/server";
import { internal } from "./_generated/api";
import { evaluateEvidence, EvaluationError, hashSource } from "./lib/jev";
import { MAX_SOURCE_LENGTH } from "./lib/evaluationPolicy";

export const run = internalAction({
  args: { jobId: v.id("evaluationJobs") },
  handler: async (ctx, { jobId }): Promise<void> => {
    const job = await ctx.runMutation(internal.evaluations.claim, { jobId });
    if (!job) return;
    try {
      const apiKey = process.env.TYPESAFE_API_KEY;
      if (!apiKey) throw new EvaluationError("typesafe_not_configured");
      let sourceText = job.sourceText;
      let sourceTitle = job.sourceTitle;
      if (!sourceText) {
        const firecrawlKey = process.env.FIRECRAWL_API_KEY;
        if (!firecrawlKey) throw new EvaluationError("source_fetch_not_configured");
        const controller = new AbortController();
        const timer = setTimeout(() => controller.abort(), 30_000);
        try {
          const response = await fetch("https://api.firecrawl.dev/v2/scrape", { method: "POST", signal: controller.signal,
            headers: { "Content-Type": "application/json", Authorization: `Bearer ${firecrawlKey}` },
            body: JSON.stringify({ url: job.sourceUrl, formats: ["markdown"], onlyMainContent: true }) });
          if (!response.ok) throw new EvaluationError(`source_http_${response.status}`, response.status === 429 || response.status >= 500);
          const body = await response.json();
          if (typeof body?.data?.markdown !== "string") throw new EvaluationError("invalid_source_response");
          sourceText = body.data.markdown.slice(0, MAX_SOURCE_LENGTH + 1);
          sourceTitle = typeof body.data.metadata?.title === "string" ? body.data.metadata.title.slice(0, 500) : undefined;
        } finally { clearTimeout(timer); }
      }
      if (!sourceText) throw new EvaluationError("empty_source");
      const sourceHash = await hashSource(sourceText.slice(0, MAX_SOURCE_LENGTH));
      const sourceTruncated = sourceText.length > MAX_SOURCE_LENGTH || job.sourceTruncated === true;
      if (!await ctx.runMutation(internal.evaluations.saveSource, { jobId, generation: job.generation, sourceText, sourceTitle, sourceHash, sourceTruncated })) return;
      const decision = await evaluateEvidence({ artist: job.artist, sourceUrl: job.sourceUrl, sourceText }, { apiKey, model: job.model });
      decision.sourceTruncated = sourceTruncated;
      await ctx.runMutation(internal.evaluations.finish, { jobId, generation: job.generation, decision });
    } catch (error) {
      const failure = error instanceof EvaluationError ? error : new EvaluationError("evaluation_execution_failed", true);
      await ctx.runMutation(internal.evaluations.fail, { jobId, generation: job.generation, code: failure.code, retryable: failure.retryable, retryAfterMs: failure.retryAfterMs });
    }
  },
});
