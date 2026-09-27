import type { Question } from "../lib/jev";
import { DEFAULT_JEV_MODEL } from "../lib/evaluationPolicy";

export function answerQuestions(questions: Record<string, Question>, choices: Record<string, string> = {}) {
  return { model: DEFAULT_JEV_MODEL, usage: { input_tokens: 100, output_tokens: 0 }, answers: Object.fromEntries(Object.entries(questions).map(([id, question]) => {
    if (question.type === "noul") return [id, { type: "noul", noul: id === "category_financial_crimes" ? 0.98 : 0.02 }];
    const keys = Object.keys(question.criteria);
    const selected = choices[id] ?? keys[0];
    return [id, { type: "choice", choice: selected, confidence: 0.96, probabilities: Object.fromEntries(keys.map(key => [key, key === selected ? 1 : 0])) }];
  })) };
}
