# Experiment 002 - Frozen Task

Task SHA will be recorded by START before the real run.

<!-- ASTRA_PROMPT_START -->
You are working in the JEM Ultimate B07-C workspace already opened for this task.

Implement the missing B07-C Quick Fix workflow/composition module expected by the existing code, tests and ADR-034.

Known starting condition:
- `tests/b07c-quick-fix-workflow.test.cjs` currently cannot load `src/plugin/quick-fix-workflow.ts` after `build:test` because that source module is missing.
- The typed Quick Fix plan/action and preview pieces already exist.

Requirements:
1. Keep the change narrowly scoped to the missing B07-C workflow/composition behavior.
2. Follow ADR-034 and the contracts already expressed by the existing tests.
3. Preserve typed quick-fix authority, owning session/tab isolation, stale-plan rejection, history/Undo semantics, diagnostic ownership, and source-owned/JPM safety.
4. Do not add a generic patch executor, Fix All, live lint, animation rewriting, external JPM mutation, or unrelated refactors/features.
5. Do not weaken or delete existing tests merely to make the suite pass.
6. Prefer the smallest architecture-consistent implementation needed to satisfy the existing contract.
7. Run the targeted B07-C workflow test while working, then run `npm run verify:local` before declaring the task complete.

At the end, report the files changed, the reason for each change, and the final test results.
<!-- ASTRA_PROMPT_END -->
