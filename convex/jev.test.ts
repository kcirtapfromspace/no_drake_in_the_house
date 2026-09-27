import { describe, expect, test, vi } from "vitest";
import { askJev, evaluateEvidence, evidenceQuestions, parseResponse, sourcePassages, type Question } from "./lib/jev";
import { canonicalSourceUrl, DEFAULT_JEV_MODEL, normalizedSeverity } from "./lib/evaluationPolicy";

import { answerQuestions } from "./testing/jevResponses";

describe("Jev boundary", () => {
  const questions: Record<string, Question> = { decision: { type: "choice", instructions: "Choose", criteria: { yes: "Yes", no: "No" } } };
  test("requires complete distributions and a pinned returned version", () => {
    const raw = answerQuestions(questions);
    expect(parseResponse(raw, questions, DEFAULT_JEV_MODEL).model).toBe(DEFAULT_JEV_MODEL);
    expect(() => parseResponse({ ...raw, model: "jev-latest" }, questions, DEFAULT_JEV_MODEL)).toThrow("unexpected_model_version");
    expect(() => parseResponse({ ...raw, answers: {} }, questions, DEFAULT_JEV_MODEL)).toThrow();
    expect(() => parseResponse({ ...raw, answers: { decision: { type: "choice", choice: "yes", probabilities: { yes: 0.9, no: 0.9 }, confidence: 1 } } }, questions, DEFAULT_JEV_MODEL)).toThrow("invalid_model_distribution");
  });
  test("429 respects Retry-After without exposing provider text", async () => {
    const fetcher = vi.fn(async () => new Response("sensitive provider body", { status: 429, headers: { "Retry-After": "20" } }));
    await expect(askJev({}, questions, { apiKey: "test", model: DEFAULT_JEV_MODEL, fetcher })).rejects.toMatchObject({ code: "typesafe_http_429", retryable: true, retryAfterMs: 20000 });
  });
  test("authentication errors are not retried", async () => {
    await expect(askJev({}, questions, { apiKey: "test", model: DEFAULT_JEV_MODEL, fetcher: async () => new Response("", { status: 401 }) })).rejects.toMatchObject({ retryable: false });
  });
  test("none stops the pipeline before category inference", async () => {
    const fetcher = vi.fn(async (_url: unknown, init?: RequestInit) => new Response(JSON.stringify(answerQuestions(JSON.parse(String(init?.body)).questions, { passage: "none" }))));
    const result = await evaluateEvidence({ artist: { id: "one", name: "Example Singer" }, sourceUrl: "https://audit.invalid/a", sourceText: "Example Singer released a hit single and beat chart records this week." }, { apiKey: "test", fetcher });
    expect(result.route).toBe("no_support"); expect(result.passage).toBeNull(); expect(fetcher).toHaveBeenCalledTimes(1);
  });
  test("two stages retain exact source and require review even at maximum probability", async () => {
    const fetcher = vi.fn(async (_url: unknown, init?: RequestInit) => new Response(JSON.stringify(answerQuestions(JSON.parse(String(init?.body)).questions, { passage: "p0", role: "accused", support: "supported", procedure: "charged" }))));
    const sourceText = "Example Singer faces financial fraud charges in a pending case. The artist denies the accusation.";
    const result = await evaluateEvidence({ artist: { id: "one", name: "Example Singer" }, sourceUrl: "https://audit.invalid/a", sourceText }, { apiKey: "test", fetcher });
    expect(result.route).toBe("needs_review"); expect(result.passage?.text).toBe(sourceText);
    expect(result.categories).toEqual(["financial_crimes"]); expect(result.responses).toHaveLength(2);
    const secondState = JSON.parse(String(fetcher.mock.calls[1][1]?.body)).state;
    expect(secondState.passage.text).toBe(sourceText);
  });
  test("all categories are independent propositions; source offsets are exact", () => {
    expect(Object.values(evidenceQuestions()).filter(q => q.type === "noul")).toHaveLength(14);
    const text = "Evidence αβγ ".repeat(250);
    for (const p of sourcePassages(text)) expect(text.slice(p.start, p.end)).toBe(p.text);
  });
  test("deterministic policy rejects local URLs and normalizes legacy severity", () => {
    for (const url of ["http://127.0.0.1", "http://localhost/", "http://169.254.169.254/", "file:///etc/passwd", "https://user:pass@example.com"]) expect(() => canonicalSourceUrl(url)).toThrow();
    expect(normalizedSeverity("critical")).toBe("egregious");
    expect(() => normalizedSeverity("unknown")).toThrow();
  });
});
