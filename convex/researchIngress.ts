import { httpAction } from "./_generated/server";
import { internal } from "./_generated/api";
import type { FunctionReference } from "convex/server";

const routes: Record<string, FunctionReference<"mutation", "internal">> = {
  "newsIngestion:ingestArticle": internal.newsIngestion.ingestArticle,
  "newsIngestion:ingestEntities": internal.newsIngestion.ingestEntities,
  "newsIngestion:ingestClassification": internal.newsIngestion.ingestClassification,
  "newsIngestion:batchIngestArticles": internal.newsIngestion.batchIngestArticles,
  "newsIngestion:updateArticleStatus": internal.newsIngestion.updateArticleStatus,
  "newsIngestion:updateArtistResearchQuality": internal.newsIngestion.updateArtistResearchQuality,
  "evaluations:enqueueResearch": internal.evaluations.enqueueResearch,
  "researchLifecycle:complete": internal.researchLifecycle.complete,
};

function reply(status: number, body: unknown) {
  return new Response(JSON.stringify(body), { status, headers: { "Content-Type": "application/json", "Cache-Control": "no-store" } });
}

export const ingest = httpAction(async (ctx, request) => {
  const expected = process.env.NDITH_SERVICE_KEY;
  if (!expected) return reply(503, { status: "error", errorMessage: "Research ingress is not configured" });
  const provided = request.headers.get("authorization") ?? "";
  const a = new TextEncoder().encode(provided);
  const b = new TextEncoder().encode(`Bearer ${expected}`);
  let difference = a.length ^ b.length;
  for (let i = 0; i < b.length; i++) difference |= (a[i] ?? 0) ^ b[i];
  if (difference !== 0) return reply(401, { status: "error", errorMessage: "Unauthorized" });
  const raw = await request.text();
  if (raw.length > 500_000) return reply(413, { status: "error", errorMessage: "Research batch too large" });
  try {
    const body = JSON.parse(raw);
    if (typeof body.path !== "string" || !Object.hasOwn(routes, body.path) || !body.args || typeof body.args !== "object" || Array.isArray(body.args)) {
      return reply(400, { status: "error", errorMessage: "Unsupported research operation" });
    }
    const value = await ctx.runMutation(routes[body.path], body.args);
    return reply(200, { status: "success", value });
  } catch {
    // Do not echo arbitrary request/provider data into logs or responses.
    return reply(400, { status: "error", errorMessage: "Invalid research request" });
  }
});
