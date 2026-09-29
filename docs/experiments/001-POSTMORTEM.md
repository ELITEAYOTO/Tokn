# Experiment 001 - Post-mortem

Date: 2026-09-27
Classification: **instrumentation discovery SUCCESS / optimization experiment INVALID**

## Executive summary

Experiment 001 produced useful engineering evidence but did not validly test
whether Tokn's read/search caps reduce Astra token consumption without quality loss.

The Astra task really ran and created a B07-B freeze plus a B07-C working copy.
However:

- the official diagnostic trace followed only 4 protocol-level events;
- the actual model/tool activity lived in standard Codex session rollouts;
- Tokn monitored the original B07-B PROJECT while Astra correctly worked in a new B07-C copy;
- the temporary policy was not present in session base instructions;
- most targeted exec calls did not declare max_output_tokens;
- the run ended because all parent/subagent threads hit usage_limit_exceeded;
- the requested B07-C implementation was not completed.

No token-efficiency claim may be derived from this run.
## Run identity

Candidate run:
E:\Tokn\V0-CodexTkn-Consume\experiments\001-runtime-output-caps\runs\20260927-192630-candidate

Main Codex session:
01a0e3ed-ea87-7a10-839b-c5da052f7eb4

Runtime:
- Codex Desktop: 26.924.2738.0
- codex-cli: 0.158.0-alpha.2.1
- Node: 22.15.1
- npm: 11.3.0

Main session cwd:
E:\BlockBench-Plugin

Temporary policy location:
E:\BlockBench-Plugin\JEM_Ultimate_v0.12.0_B07-B_WORKING\PROJECT\AGENTS.md

This path mismatch matters because the session root was above PROJECT.
## Actual token accounting recovered from session rollouts

Tokn imported 4 related session rollouts:
- parent thread: 29 usage records;
- budget_audit subagent: 19 usage records;
- fix_audit subagent: 16 usage records;
- ui_workflow subagent: 16 usage records.

Aggregate:
- usage records: 80;
- input tokens: 5,285,737;
- cached input: 5,059,712;
- uncached input: 226,025;
- output tokens: 26,021;
- reasoning output: 3,018;
- logical total: 5,311,758;
- duplicate suppression conflicts: 0;
- token invariant conflicts: 0.

Parent logical total:
2,499,523 (~47.1%).

Subagent logical total:
2,812,235 (~52.9%).

Cached share of input:
~95.72%.

The dominant cost is therefore not just the parent thread.
Subagent fan-out is a first-class optimization target.
## Completion status

All four threads terminated with:
usage_limit_exceeded

Main task duration before terminal error:
374,751 ms (~6 min 15 s).

Observed work completed before exhaustion:
- B07-B freeze created;
- B07-C working copy created;
- bundle baseline analysis produced;
- bundle budget policy/tests produced;
- ADR-034 produced;
- Quick Fix audit produced;
- Blockbench 5.1.6 host-source evidence captured.

The requested Quick Fix Framework implementation was not completed.

Therefore task status is:
**INCOMPLETE_USAGE_LIMIT**
## Workspace tracking failure

The original Experiment 001 snapshot watched:
E:\BlockBench-Plugin\JEM_Ultimate_v0.12.0_B07-B_WORKING\PROJECT

Astra correctly created and worked in:
E:\BlockBench-Plugin\JEM_Ultimate_v0.12.1_B07-C_WORKING

Result:
the old finisher reported 0 added / 0 modified / 0 removed.

Direct comparison between B07-B_FROZEN and B07-C_WORKING later found:
- 13 added files;
- 4 modified files;
- 0 removed files.

This is not an Astra failure. It is a Tokn workspace-observation failure.
## Diagnostic trace coverage failure

The candidate diagnostic bundle contained only:
- rollout_started: 1;
- thread_started: 1;
- protocol_event_observed: 2.

trace.jsonl contained only 4 lines and no inference/tool usage records.

Tokn nevertheless generated:
- 0 targeted tools;
- 0 violations;
- known caps compliant = YES;
- evidence complete = YES.

This is invalid semantics.

New invariant:
**zero observations can never prove compliance.**

When the expected target surface has zero observations, policy status must be:
NO_EVIDENCE.
## Policy-adherence findings from standard session rollouts

Across parent + 3 subagents:
- exec calls: 64;
- calls with no explicit max_output_tokens: 57;
- explicit 5000 cap: 4;
- explicit 6000 cap: 1;
- explicit 8000 cap: 1;
- explicit 3000 cap: 1.

Approximate targeted classification:

file_read:
- calls: 25;
- compliant <= 5000: 3;
- unknown/no explicit cap: 20;
- violations: 2.

search:
- calls: 25;
- compliant <= 3000: 1;
- unknown/no explicit cap: 23;
- violations: 1.

Conclusion:
the conservative cap policy was not reliably enforced.
The experiment did not test the simulated cap scenario.
## Policy injection finding

The temporary AGENTS policy was NOT present in session_meta.base_instructions.

The parent agent explicitly searched for and read PROJECT/AGENTS.md,
so the policy was visible as repository evidence, but it was not a reliable
session-level enforcement mechanism.

Soft instructions and hard enforcement must be treated as distinct mechanisms.

Tokn terminology going forward:
- policy_hint: model-facing instruction only;
- policy_observed: evidence that the model/tool request followed it;
- policy_enforced: runtime mechanism makes violation impossible.

Experiment 001 only attempted policy_hint.
## Real B07-C quality check

The old finisher accidentally ran verify:local in the original B07-B PROJECT.

A manual post-mortem verification was then run in:
E:\BlockBench-Plugin\JEM_Ultimate_v0.12.1_B07-C_WORKING\PROJECT

Result:
PASS.

Observed test count:
814 tests, up from the 809-test starting state.

The new B07-C bundle-budget tests pass.
The incomplete candidate work therefore remains technically healthy so far,
but it is not the completed requested deliverable.

## Decisions resulting from Experiment 001

1. Standard session rollouts become mandatory fallback evidence.
2. Parent/subagent session grouping becomes core domain logic.
3. Terminal task status is part of every run result.
4. NO_EVIDENCE is distinct from PASS.
5. Workspace tracking must support new/copy output roots.
6. Quality gates must run against the actual output workspace.
7. Policy hints cannot be described as enforcement.
8. Diagnostic trace health must be evaluated before its metrics are trusted.
9. Experiment validity must be machine-computable.
10. No new active optimization experiment until V0.1 measurement is hardened.
