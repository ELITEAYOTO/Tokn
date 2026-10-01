# Experiment 002 - Instrumentation Validation

Status: DONE / ACCEPTED
Updated: 2026-10-01

## Purpose

Validate that Tokn V0.1 measures a real Astra run correctly end to end.

Experiment 002 does NOT attempt to prove token savings.

It validates:
- session/diagnostic fallback;
- parent/subagent grouping;
- terminal status;
- tool/policy evidence;
- dynamic workspace tracking;
- correct quality-gate target;
- experiment-validity reducer.

## Task selection

Use a genuine but bounded JEM development or maintenance task.

Requirements:
- useful real work;
- small enough that quota exhaustion is unlikely;
- clear automated acceptance gate;
- preferably one known candidate output workspace;
- no broad multi-phase architecture migration.

Do not ask Astra to be faster, shorter or less thorough.
## Required preflight

Before launch:
- V0.1 tests PASS;
- no active run;
- harness provisioned from a clean Git commit and SHA-256 manifest verified;
- release binary provenance points to the same commit and binary SHA-256;
- complete frozen baseline still matches 833 tests / 832 PASS / 1 known FAIL;
- runtime/app versions recorded;
- run-scoped model/config values captured after the run from diagnostic evidence;
- source workspace snapshot complete;
- expected output workspace recorded when known;
- policy placement paths recorded;
- session root selection confirmed;
- fallback discovery enabled.

## Success conditions

Capture:
- root session discovered;
- every descendant subagent discovered;
- token totals available;
- diagnostic/session source health visible;
- terminal status known.

Workspace:
- actual changed workspace identified;
- correct before/after diff;
- quality gate runs on the actual output.

Policy/evidence:
- policy level accurately reported when a policy is present;
- zero observations cannot pass;
- no hard-cap enforcement is required for this instrumentation run.

Experiment:
- machine-readable validity verdict generated;
- runtime validity is PASS when CLI/runtime evidence exists;
- model_recorded is PASS only when run-scoped model evidence is observed;
- configuration_recorded may remain UNKNOWN until the post-run ModelRuntimeProfile contract is frozen.

Harness source of truth:
- `scripts/experiment/002/` contains the versioned task and lifecycle scripts;
- `scripts/experiment/prepare-exp002-harness.ps1` provisions the external run workspace;
- real traces/runs remain outside Git.

## Real run result - 2026-10-01

Experiment 002 is accepted for instrumentation validation.

Measured result:
- runtime: `codex-cli 0.161.0-alpha.2`;
- observed model: `gpt-6.1-sol`;
- RunGroup: 1 parent + 1 subagent;
- usage records: 49;
- input tokens: 3,328,582;
- cached input tokens: 3,168,256;
- output tokens: 13,394;
- reasoning output tokens: 3,734;
- parent terminal: COMPLETED;
- Runner pipeline: COMPLETE;
- quality gate: PASS, 863/863 tests;
- workspace diff: 1 added / 2 modified / 0 removed;
- validity: INSTRUMENTATION_ONLY;
- causal claims: blocked;
- descriptive metrics: allowed;
- configuration completeness: UNKNOWN.

The live run exposed two FINISH harness defects:
1. `$LASTEXITCODE` was incorrectly treated as authoritative after an internal PowerShell script;
2. Codex CLI diagnostic capture stored the bundle one level below the configured trace root,
   while the V0.1 workspace resolver correctly rejected the parent folder.

No model rerun was used to repair the evidence.
Tokn reused the persisted healthy Codex parent rollout plus its descendant,
resolved the correct workspace, reran the Runner deterministically and reached COMPLETE.
The fallback path is now versioned and covered by `check-exp002-source-selection.ps1` in CI.

This recovery does not strengthen the experiment beyond instrumentation validation.
It does not prove savings, efficiency improvement or a causal intervention.

## Failure conditions

Repeat Experiment 002 after fixing Tokn if:
- source fallback requires semantic reconstruction, invented evidence or mutation of original run data;
- descendant sessions are missed;
- output workspace is wrong;
- quality gate runs on the wrong root;
- terminal status is UNKNOWN despite rollout evidence;
- compliance reports PASS with zero/unknown evidence;
- reports silently mix unrelated concurrent sessions.

## After Experiment 002

Instrumentation is valid and the Measurement Contract V1 is now frozen.

The minimal Store + ModelRuntimeProfile foundation is now implemented.

Next:
1. prototype the thin local MCP adapter over the shared Engine/Store ;
2. build Historical Analyzer + Context Ledger observation-only ;
3. produce recurring evidence-backed findings ;
4. select one reproducible low-risk finding ;
5. only then prepare Experiment 003 as the first controlled optimization A/B.

Experiment 003 requirements:
- choose one reproducible finding from Historical Analyzer / Context Efficiency analysis;
- frozen identical starting workspace;
- identical task;
- same Codex runtime/model/config;
- control run without the candidate intervention;
- candidate run with exactly one primary optimization variable;
- quality acceptance before token comparison;
- no default assumption that the variable is an output cap.
