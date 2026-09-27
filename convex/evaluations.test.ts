/// <reference types="vite/client" />
import { convexTest } from "convex-test";
import { afterEach, beforeEach, expect, test, vi } from "vitest";
import { api, internal } from "./_generated/api";
import schema from "./schema";
import { answerQuestions } from "./testing/jevResponses";
import { DEFAULT_JEV_MODEL, LEASE_MS, POLICY_VERSION } from "./lib/evaluationPolicy";
import { ingestArticle } from "./newsIngestion";
import { attachAuthSubject } from "./users";

const modules = import.meta.glob("./**/*.ts");
const identity = { tokenIdentifier: "https://audit.invalid|user", subject: "user", issuer: "https://audit.invalid" };
const reviewerIdentity = { ...identity, tokenIdentifier: "https://audit.invalid|reviewer", subject: "reviewer" };
const otherIdentity = { ...identity, tokenIdentifier: "https://audit.invalid|other", subject: "other" };
const source = "Example Singer faces financial fraud charges in a pending case. The artist denies the allegation.";
const life = { createdAt: "2026-09-27T00:00:00Z", updatedAt: "2026-09-27T00:00:00Z" };

beforeEach(() => { vi.useFakeTimers(); vi.stubEnv("TYPESAFE_API_KEY", "test-key"); vi.stubEnv("FIRECRAWL_API_KEY", "test-key"); });
afterEach(() => { vi.clearAllTimers(); vi.useRealTimers(); vi.unstubAllEnvs(); vi.unstubAllGlobals(); });

async function setup() {
  const t = convexTest(schema, modules);
  const ids = await t.run(async ctx => {
    const user = await ctx.db.insert("users", { ...life, legacyKey: "test:user", authSubject: identity.tokenIdentifier, email: "user@audit.invalid", roles: [] });
    const reviewer = await ctx.db.insert("users", { ...life, legacyKey: "test:reviewer", authSubject: reviewerIdentity.tokenIdentifier, roles: ["reviewer"] });
    const other = await ctx.db.insert("users", { ...life, legacyKey: "test:other", authSubject: otherIdentity.tokenIdentifier, roles: [] });
    const artist = await ctx.db.insert("artists", { ...life, legacyKey: "test:artist", canonicalName: "Example Singer" });
    return { user, reviewer, other, artist };
  });
  return { t, client: t.withIdentity(identity), reviewerClient: t.withIdentity(reviewerIdentity), ...ids };
}
function mockModel(choice = "p0") {
  const fetcher = vi.fn(async (url: unknown, init?: RequestInit) => {
    if (String(url).includes("firecrawl")) return new Response(JSON.stringify({ data: { markdown: source, metadata: { title: "Pending charges" } } }));
    return new Response(JSON.stringify(answerQuestions(JSON.parse(String(init?.body)).questions, { passage: choice, role: "accused", support: "supported", procedure: "charged" })));
  });
  vi.stubGlobal("fetch", fetcher);
  return fetcher;
}

test("intake is authenticated, idempotent, and reserves quota before external work", async () => {
  const { t, client, artist, user } = await setup();
  await expect(t.mutation(api.evaluations.submit, { artistId: artist, url: "https://audit.invalid/a" })).rejects.toThrow("Authentication");
  const args = { artistId: artist, url: "https://audit.invalid/a#fragment" };
  const first = await client.mutation(api.evaluations.submit, args);
  expect((await client.mutation(api.evaluations.submit, args)).jobId).toBe(first.jobId);
  expect(first).toMatchObject({ accepted: true, verified: false });
  const quota = await t.run(ctx => ctx.db.query("evaluationQuotas").withIndex("by_userId_and_day", q => q.eq("userId", user)).unique());
  expect(quota?.attempts).toBe(1);
  for (let i = 1; i < 10; i++) await client.mutation(api.evaluations.submit, { artistId: artist, url: `https://audit.invalid/${i}` });
  await expect(client.mutation(api.evaluations.submit, { artistId: artist, url: "https://audit.invalid/limit" })).rejects.toThrow("Maximum 10");
});

test("model success creates a review candidate, never an approved offense", async () => {
  const { t, client, artist } = await setup(); mockModel();
  const { jobId } = await client.action(api.evidenceVerifier.submitEvidence, { artistId: artist, url: "https://audit.invalid/a" });
  await t.action(internal.evaluationWorker.run, { jobId });
  const job = await client.query(api.evaluations.get, { jobId });
  expect(job.status).toBe("needs_review"); expect(job.decision.passage.text).toBe(source);
  expect(await t.run(ctx => ctx.db.query("artistOffenses").collect())).toEqual([]);
  await expect(t.withIdentity(otherIdentity).query(api.evaluations.get, { jobId })).rejects.toThrow("not found");
});

test("independent review atomically creates provenance and updates the approved index", async () => {
  const { t, client, reviewerClient, artist, user } = await setup(); mockModel();
  const { jobId } = await client.mutation(api.evaluations.submit, { artistId: artist, url: "https://audit.invalid/a" });
  await t.action(internal.evaluationWorker.run, { jobId });
  const review = { jobId, approve: true, reason: "Checked the source and the artist identity independently.", category: "financial_crimes", severity: "severe", proceduralState: "charged" };
  await expect(client.mutation(api.evaluations.review, review)).rejects.toThrow("Reviewer");
  await t.run(ctx => ctx.db.patch(user, { roles: ["reviewer"] }));
  await expect(client.mutation(api.evaluations.review, review)).rejects.toThrow("independent");
  const result = await reviewerClient.mutation(api.evaluations.review, review);
  const offense = await t.run(ctx => ctx.db.get(result.offenseId!));
  expect(offense).toMatchObject({ status: "verified", proceduralState: "charged" });
  expect(offense?.metadata.evaluationId).toBe(jobId);
  expect((await t.run(ctx => ctx.db.query("offendingArtistIndex").collect()))[0].severityTotal).toBe(7);
  await expect(reviewerClient.mutation(api.evaluations.review, review)).rejects.toThrow("not awaiting");
});

test("no-support and service failure cannot silently create offenses", async () => {
  const { t, client, artist } = await setup(); mockModel("none");
  const a = await client.mutation(api.evaluations.submit, { artistId: artist, url: "https://audit.invalid/a" });
  await t.action(internal.evaluationWorker.run, { jobId: a.jobId });
  expect((await client.query(api.evaluations.get, { jobId: a.jobId })).status).toBe("no_support");
  vi.stubEnv("TYPESAFE_API_KEY", "");
  const b = await client.mutation(api.evaluations.submit, { artistId: artist, url: "https://audit.invalid/b" });
  await t.action(internal.evaluationWorker.run, { jobId: b.jobId });
  expect(await client.query(api.evaluations.get, { jobId: b.jobId })).toMatchObject({ status: "failed", lastError: "typesafe_not_configured" });
  expect(await t.run(ctx => ctx.db.query("artistOffenses").collect())).toEqual([]);
});

test("lease recovery rejects stale results and manual retry preserves fencing generation", async () => {
  const { t, client, reviewerClient, artist } = await setup();
  const { jobId } = await client.mutation(api.evaluations.submit, { artistId: artist, url: "https://audit.invalid/a" });
  const first = await t.mutation(internal.evaluations.claim, { jobId });
  expect(await t.mutation(internal.evaluations.claim, { jobId })).toBeNull();
  vi.setSystemTime(Date.now() + LEASE_MS + 1);
  await t.mutation(internal.evaluations.recover, { jobId, generation: first!.generation });
  const second = await t.mutation(internal.evaluations.claim, { jobId });
  expect(second!.generation).toBeGreaterThan(first!.generation);
  expect(await t.mutation(internal.evaluations.finish, { jobId, generation: first!.generation, decision: {} })).toBe(false);
  await t.mutation(internal.evaluations.fail, { jobId, generation: second!.generation, code: "typesafe_http_401", retryable: false });
  await reviewerClient.mutation(api.evaluations.retry, { jobId });
  const third = await t.mutation(internal.evaluations.claim, { jobId });
  expect(third!.generation).toBeGreaterThan(second!.generation);
});

test("transient errors respect retry time and exhaust the retry budget", async () => {
  const { t, client, artist } = await setup();
  const { jobId } = await client.mutation(api.evaluations.submit, { artistId: artist, url: "https://audit.invalid/a" });
  for (let i = 0; i < 4; i++) {
    const job = await t.mutation(internal.evaluations.claim, { jobId });
    await t.mutation(internal.evaluations.fail, { jobId, generation: job!.generation, code: "typesafe_http_429", retryable: true, retryAfterMs: 20000 });
    expect(await t.mutation(internal.evaluations.claim, { jobId })).toBeNull();
    vi.setSystemTime(Date.now() + 21000);
  }
  expect((await client.query(api.evaluations.get, { jobId })).status).toBe("failed");
});

test("research ingress rejects anonymous requests and forbids privileged arbitrary operations", async () => {
  const { t, artist } = await setup(); vi.stubEnv("NDITH_SERVICE_KEY", "test-service-key");
  expect((await t.fetch("/research/ingest", { method: "POST", body: "{}" })).status).toBe(401);
  const headers = { Authorization: "Bearer test-service-key", "Content-Type": "application/json" };
  expect((await t.fetch("/research/ingest", { method: "POST", headers, body: JSON.stringify({ path: "users:attachAuthSubject", args: {} }) })).status).toBe(400);
  const response = await t.fetch("/research/ingest", { method: "POST", headers, body: JSON.stringify({ path: "evaluations:enqueueResearch", args: { artistId: artist, url: "https://audit.invalid/research", sourceText: source } }) });
  expect(response.status).toBe(200);
  const body = await response.json();
  expect(body.status).toBe("success");
  expect(await t.run(ctx => ctx.db.get(body.value.jobId))).toMatchObject({ status: "queued", origin: "research" });
  expect(ingestArticle.isInternal).toBe(true); expect(attachAuthSubject.isInternal).toBe(true);
});

test("batch ingestion evaluates an artist even with no keyword classification", async () => {
  const { t, artist } = await setup();
  await t.mutation(internal.newsIngestion.batchIngestArticles, { articles: [{ legacyKey: "article", url: "https://audit.invalid/batch", title: "Source", content: source,
    entities: [{ legacyKey: "entity", entityName: "Example Singer", entityType: "artist", artistId: artist }] }] });
  const jobs = await t.run(ctx => ctx.db.query("evaluationJobs").collect());
  expect(jobs).toHaveLength(1); expect(jobs[0]).toMatchObject({ artistId: artist, model: DEFAULT_JEV_MODEL, policyVersion: POLICY_VERSION });
});

test("private preferences and unapproved allegations stay out of global policy", async () => {
  const { t, user, artist } = await setup();
  await t.run(async ctx => {
    await ctx.db.insert("userArtistBlocks", { ...life, legacyKey: "block", userId: user, artistId: artist });
    await ctx.db.insert("artistOffenses", { ...life, legacyKey: "offense", artistId: artist, category: "financial_crimes", severity: "critical", title: "Unapproved", description: "Unapproved", status: "rejected" });
  });
  await t.mutation(internal.offensePipeline.rebuildArtistIndex, { artistId: artist });
  expect(await t.run(ctx => ctx.db.query("offendingArtistIndex").collect())).toEqual([]);
  expect((await t.query(api.extension.getLatestPublicSnapshot, {})).artists).toEqual([]);
});

test("unverified same-email identity cannot take over a reviewer account", async () => {
  const { t } = await setup();
  await expect(t.withIdentity({ ...otherIdentity, tokenIdentifier: "https://audit.invalid|new-attacker", email: "user@audit.invalid", emailVerified: false }).mutation(api.users.upsertCurrent, {})).rejects.toThrow("existing account");
  await expect(t.withIdentity({ ...otherIdentity, email: "user@audit.invalid", emailVerified: false }).mutation(api.users.upsertCurrent, {})).rejects.toThrow("existing account");
});


test("review can correct model abstention using an exact retained passage and explicit outcome", async () => {
  const { t, client, reviewerClient, artist } = await setup(); mockModel("none");
  const { jobId } = await client.mutation(api.evaluations.submit, { artistId: artist, url: "https://audit.invalid/missed" });
  await t.action(internal.evaluationWorker.run, { jobId });
  const args = { jobId, approve: true, reason: "Independent source review found the missed allegation.", category: "financial_crimes", severity: "moderate", proceduralState: "dismissed" };
  await expect(reviewerClient.mutation(api.evaluations.review, { ...args, excerpt: "A fabricated quotation that is not in this source." })).rejects.toThrow("exact supporting passage");
  const result = await reviewerClient.mutation(api.evaluations.review, { ...args, excerpt: source });
  expect(await t.run(ctx => ctx.db.get(result.offenseId!))).toMatchObject({ description: source, proceduralState: "dismissed", status: "verified" });
  expect((await client.query(api.evaluations.get, { jobId })).decision.route).toBe("no_support");
});

test("research acceptance is not completion; failed, expired and stale callbacks do not stamp freshness", async () => {
  const { t, artist } = await setup();
  const first = await t.mutation(internal.researchLifecycle.start, { artistId: artist });
  expect(first).toBe(1);
  await t.mutation(internal.researchLifecycle.queued, { artistId: artist, generation: first! });
  expect((await t.run(ctx => ctx.db.get(artist)))?.lastInvestigatedAt).toBeUndefined();
  expect(await t.mutation(internal.researchLifecycle.start, { artistId: artist })).toBeNull();
  await t.mutation(internal.researchLifecycle.complete, { artistId: artist, generation: first!, success: false });
  expect((await t.run(ctx => ctx.db.get(artist)))?.lastInvestigatedAt).toBeUndefined();
  vi.setSystemTime(Date.now() + 600_001);
  const second = await t.mutation(internal.researchLifecycle.start, { artistId: artist });
  await t.mutation(internal.researchLifecycle.complete, { artistId: artist, generation: first!, success: true });
  expect((await t.run(ctx => ctx.db.get(artist)))?.lastInvestigatedAt).toBeUndefined();
  vi.setSystemTime(Date.now() + 600_001);
  await t.mutation(internal.researchLifecycle.expire, { artistId: artist, generation: second! });
  expect(await t.run(ctx => ctx.db.get(artist))).toMatchObject({ investigationStatus: "failed" });
  const third = await t.mutation(internal.researchLifecycle.start, { artistId: artist });
  await t.mutation(internal.researchLifecycle.complete, { artistId: artist, generation: third!, success: true });
  await t.mutation(internal.researchLifecycle.queued, { artistId: artist, generation: third! });
  expect(await t.run(ctx => ctx.db.get(artist))).toMatchObject({ investigationStatus: "completed", lastInvestigatedAt: expect.any(String) });
});
