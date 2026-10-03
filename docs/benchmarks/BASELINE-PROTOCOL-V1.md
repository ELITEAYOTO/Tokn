# Baseline Benchmark Protocol V1

Status: ACCEPTED PROTOCOL / BASELINE-ONLY
Date: 2026-10-03

## Purpose

Measure native coding-agent variance, quality and addressable efficiency surfaces before Tokn introduces an active optimization.

This protocol is **descriptive**. It must not be presented as evidence that Tokn saves tokens.
Experiment 003 remains the first causal A/B optimization experiment.

## Phase 1: native baseline

Current condition:
- `A = native agent behavior observed by Tokn`;
- no active Tokn intervention changes model/tool behavior.

Do not create a fake `B = passive observer` condition. A passive observer is expected to leave model token behavior essentially unchanged and would not test the future optimizer.

The baseline answers:
- how variable are repeated runs of the same task?;
- which token components dominate?;
- how often do reads/searches/retries/overlaps recur?;
- what quality failures and rate-limit effects occur?;
- what overhead does Tokn itself add?

## Corpus

Prepare 6-10 reproducible tasks across frozen repository commits.
Include a mix of:
- bug fixes;
- bounded features;
- refactors;
- repository-understanding questions.

Each task must declare before execution:
- repository + exact starting commit;
- task prompt reference/hash;
- setup/reset procedure;
- quality commands/tests;
- allowed/forbidden side effects;
- exclusion policy.

Prefer task manifests and external/frozen repos over copying large repositories into Tokn fixtures.
`fixtures/` remains primarily for parser/protocol/regression evidence.

## Run count and order

Start with a variance pilot of at least 3 native runs per task.
Do not make `3` a permanent sample-size rule: increase repeats when observed variance is high or the expected effect is small.

For future A/B experiments, randomize/interleave condition order instead of running all A then all B.
Record run order and time so provider/cache/rate-limit drift can be inspected.

## Quality is the constraint

Efficiency is evaluated only inside the acceptable-quality region.
Predeclare automatic gates first:
- terminal status compatible/completed;
- build passes;
- existing tests pass;
- task-specific regression tests pass;
- Clippy or equivalent project lint remains acceptable;
- no forbidden regression/side effect.

Use a blind human rubric only for dimensions that cannot be validated automatically.
Write that rubric before running the experiment.

A future B condition counts as an efficiency win only when its quality/success rate is non-inferior under the predeclared gate.

## Efficiency vector

Never collapse all token semantics into one ambiguous number.
Report separately when observed:
- input tokens;
- cached input tokens;
- ordinary uncached input (`input - cached` under current V1 semantics);
- cache-write input;
- output tokens;
- reasoning output tokens;
- tool calls;
- file reads/searches;
- exact repeat observations;
- cross-agent overlap/result identity coverage;
- wall-clock duration and rate-limit observations.

## Tokn overhead

Measure Tokn separately from agent cost whenever Tokn is injected or default-active:
- Tokn wall time;
- peak RSS / memory delta where measurable;
- CPU time where measurable;
- SQLite/database growth;
- ingestion/query latency;
- additional tool/runtime interception overhead.

A token reduction does not automatically justify excessive local CPU, RAM, latency or disk growth.

## Cache handling

Provider cache state can confound comparisons.
If the provider cache cannot be reset deterministically:
- preserve cached/uncached/cache-write metrics separately;
- randomize future A/B order;
- record cache condition as observed/unknown, never assume cold/warm equality;
- do not attribute an order effect to Tokn.

## Invalid/excluded runs

Declare exclusion rules before inspecting results.
Examples requiring an explicit policy:
- usage/rate limit hit;
- provider/network failure;
- runtime crash;
- missing tool/dependency;
- incomplete capture;
- invalid starting workspace;
- quality gate infrastructure failure.

Never remove a bad B run after seeing the outcome merely because it weakens the result.

## Statistics

For each task/condition report at minimum:
- run count;
- success/quality-pass rate;
- median;
- p25/p75 or IQR;
- min/max only as secondary diagnostics;
- coverage/UNKNOWN counts for evidence-dependent metrics.

For later causal experiments, add confidence intervals or an appropriate paired/resampling analysis after the variance pilot determines sample needs.

## Privacy and publication

Raw model rollouts, prompts, source trees and private tool outputs remain local by default.
Do not commit raw benchmark runs.
Publish only sanitized manifests, structured metrics and aggregates that pass the repository privacy gate.

Recommended layout:

```text
benchmarks/
  manifest.schema.json
  example-manifest.json
  results/          # sanitized aggregates only when accepted
```

Local raw run directories belong outside tracked repository paths or under ignored runtime locations.

## BenchmarkManifest V1

Every published/sanitized run record should identify at least:
- benchmark/task/run IDs;
- condition (`NATIVE_BASELINE` today, later explicit policy/intervention ID);
- repository reference + exact starting commit;
- prompt reference/hash, not an accidental private prompt dump;
- runtime kind/version;
- model/provider/reasoning configuration when observed;
- Tokn commit/version;
- policy/config version;
- quality-gate version;
- cache condition/coverage;
- run order and observation time;
- exclusion-policy version;
- evidence/coverage notes.

The machine-readable schema lives at `benchmarks/manifest.schema.json`.

## Transition to Experiment 003

Move from baseline to causal A/B only after:
1. M4/M5 evidence produces a reproducible addressable finding;
2. Opportunity Analyzer says the surface is worth targeting;
3. an actual Tokn intervention exists;
4. runtime/model/config compatibility is explicit;
5. quality and exclusion rules are frozen;
6. run-to-run variance is understood well enough to size the experiment.
