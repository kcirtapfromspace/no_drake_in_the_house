# Project guidance

- Read `convex/_generated/ai/guidelines.md` before changing Convex code.
- Use the repository's TypeSafe skill at `.agents/skills/typesafe-ai/SKILL.md` for semantic evaluation work. Read its linked live documentation before changing the API contract or questions.
- Jev supplies typed judgments; code owns authentication, quotas, job state, review permissions, and enforcement. Model output alone must never mark an allegation verified.
- Research and user submissions use the shared evaluation pipeline. Preserve source passages, model and policy versions, uncertainty, and reviewer provenance.
- Run `npm run typecheck` and `npm test` for Convex/evaluation changes. See `docs/architecture/jev-evaluation.md` for the runtime and rollout contract.
