import { evaluateEvidence } from "../../convex/lib/jev";
import { DEFAULT_JEV_MODEL } from "../../convex/lib/evaluationPolicy";
import cases from "./cases.json";

// The key is injected into this process, never read from tracked files or printed.
const apiKey = process.env.TYPESAFE_API_KEY;
if (!apiKey) throw new Error("Inject TYPESAFE_API_KEY before running the live evaluation.");
const results = [];
for (const sample of cases) {
  const started = Date.now();
  const decision = await evaluateEvidence({ artist: { id: "synthetic-artist", name: "Example Singer" },
    sourceUrl: `https://audit.invalid/${sample.id}`, sourceText: sample.text },
    { apiKey, model: process.env.TYPESAFE_MODEL || DEFAULT_JEV_MODEL });
  const matchesExpected = decision.route === sample.expected &&
    (!sample.expectedRole || decision.role === sample.expectedRole) &&
    (!sample.expectedProcedure || decision.proceduralState === sample.expectedProcedure) &&
    (!sample.expectedCategory || (decision.categories as string[]).includes(sample.expectedCategory));
  results.push({ id: sample.id, expected: sample.expected, expectedRole: sample.expectedRole, expectedProcedure: sample.expectedProcedure,
    expectedCategory: sample.expectedCategory, route: decision.route, matchesExpected,
    role: decision.role, procedure: decision.proceduralState, categories: decision.categories, model: decision.model,
    inputTokens: decision.responses.reduce((total, response) => total + response.usage.input_tokens, 0), latencyMs: Date.now() - started,
    selectedPassage: decision.passage?.text ?? null, responses: decision.responses });
}
console.log(JSON.stringify({ timestamp: new Date().toISOString(), syntheticOnly: true, cases: results.length,
  matched: results.filter((result) => result.matchesExpected).length, results }, null, 2));
if (results.some((result) => !result.matchesExpected)) process.exitCode = 1;
