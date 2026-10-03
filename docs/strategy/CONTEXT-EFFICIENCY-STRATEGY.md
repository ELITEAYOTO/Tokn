# Tokn Context Efficiency Strategy

Status: PRODUCT DIRECTION
Date: 2026-09-29
Decision basis: ADR-003

## Objective

Improve useful work and quality per token without reducing Astra's reasoning or output capability.

Tokn first observes and explains context behavior.
It only changes context after a reproducible finding and a valid A/B experiment.

## What Tokn optimizes

The target is avoidable context work, not model intelligence.

Priority opportunity classes:
1. repeated file reads and repeated searches;
2. duplicate evidence loaded by parent and subagents;
3. stable context that repeatedly becomes uncached;
4. oversized tool evidence that is repeatedly reintroduced without added value;
5. stale or globally-applied instructions that trigger unnecessary work;
6. repeated project discovery across runs;
7. cache-breaking changes to otherwise stable prefixes when observable;
8. repeated tests/builds/retries without a relevant state change;
9. context compaction/retrieval behavior that loses useful state or causes expensive rediscovery.

## What Tokn does not optimize by default

- maximum output tokens;
- reasoning effort;
- number of subagents;
- context window size;
- test depth;
- verification depth.

Those may be experimental variables only when a finding justifies them and quality is protected.

## Measurement architecture after V0.1

### Context Ledger

For every observable model usage record:
- input tokens;
- cached input tokens;
- uncached input tokens;
- output tokens;
- reasoning tokens;
- session/agent/turn identity;
- timestamps and runtime/model/config provenance.

Derived metrics must be labeled DERIVED, never OBSERVED.

### Cache Efficiency Analyzer

Measure:
- cached/input ratio;
- uncached growth by turn and agent;
- cache-ratio changes around tool/schema/instruction/context changes;
- repeated stable material correlated with uncached growth;
- cache behavior before/after compaction when observable.

Do not infer an exact cache miss cause without request-prefix evidence or runtime diagnostics.

### Context Duplication Analyzer

Detect:
- same file/range read repeatedly;
- overlapping reads;
- repeated search query/pattern;
- same evidence loaded by multiple agents;
- repeated repository orientation;
- repeated AGENTS/instruction reads;
- repeated tool outputs or equivalent evidence.

Each finding records frequency, observed token footprint when available, and provenance.

### Parent/Subagent Context Graph

For every RunGroup:
- parent/child topology;
- per-agent token ledger;
- shared files/searches/tools;
- duplicated discovery;
- unique contribution;
- concurrent spans.

High descendant cost is not itself a defect.
The question is whether duplicated context produced additional useful work.

### Tool Evidence Footprint

Measure model-visible evidence produced by tools when observable:
- raw size;
- normalized category;
- repeated/overlapping content;
- downstream reuse;
- retry/rework relationship.

Compression or truncation remains observation-only until an A/B proves quality preservation.

### Project Memory Candidate Detector

Before building active memory, detect facts repeatedly rediscovered across runs:
- build/test commands;
- stable architecture boundaries;
- workspace conventions;
- recurring file locations;
- validated project-specific procedures.

A candidate becomes project memory only after freshness, provenance and invalidation rules exist.

## Experiment 001 interpretation

Experiment 001:
- input = 5,285,737;
- cached input = 5,059,712;
- uncached input = 226,025;
- cached/input = about 95.72%;
- logical total = 5,311,758;
- descendants = about 52.9%.

This proves that most reported input tokens were classified as cached in that run.
It does NOT prove:
- that the context was useful;
- that cached tokens were free;
- that cached tokens did not affect product quota;
- that the cache was optimal;
- that parent/subagent duplication was harmless;
- that compaction would improve the run;
- that removing context would preserve quality.

The next analyzer must explain where the 226,025 uncached tokens appeared,
which stable material was repeatedly carried, and which work was duplicated across agents.

## Evidence levels

OBSERVED:
directly present in rollouts, trace, runtime logs or verified workspace state.

DERIVED:
deterministic calculation from observed evidence.

INFERRED:
plausible explanation requiring validation.

UNKNOWN:
not observable with current evidence.

No finding may silently promote INFERRED to OBSERVED.

## Active optimization gate

An optimization candidate may reach Experiment Lab only when:
- the waste pattern is reproducible;
- its evidence is attributable to a project/run/agent/phase;
- the intervention is reversible;
- quality gates are defined before the run;
- one primary variable changes;
- terminal status and runtime comparability are valid;
- expected benefit is bounded rather than advertised as guaranteed.

## Product sequence

1. Finish V0.1 measurement hardening.
2. Validate the complete runner with Experiment 002.
3. Build Historical Analyzer + Context Ledger.
4. Finish M4 identity/rediscovery evidence and seed Context Twin semantics without active injection.
5. Build M4.5 Context Identity + Shadow Retrieval/Edit foundations.
6. Build M5 Findings + Opportunity Analyzer V0 and allow explicit deprioritization.
7. Add deterministic counterfactual replay + versioned policy candidates before expensive experiments.
8. Run the first causal A/B (Experiment 003) on one bounded intervention.
9. Advisor follows validated recurring evidence.
10. Only after repeated wins, consider active Context Compiler / Project Memory.
11. AutoLab/learned policies remain a later offline-first track.

Plugin packaging is distribution/activation infrastructure, not an optimization by itself.

## Context Compiler guardrails - 2026-10-02

Tokn should evolve from observer to selective context compiler only through:
observe -> classify -> shadow -> findings -> causal experiment -> advisor -> selective automation.

Tokn may take over deterministic/repetitive/verifiable mechanics (search, symbol lookup, hashes, deduplication, Git invalidation, mechanical transforms after Astra has decided the semantic change). Astra retains architecture, debugging judgement, semantic edits and final validation.

Priority candidates to study:
- repeated reads/searches and repository orientation;
- repo/symbol index + Git incremental invalidation;
- context hashes / already-known ranges;
- post-edit redundant reads;
- edit amplification and retry chains;
- tool-output footprint;
- parent/subagent duplicate evidence;
- stable instruction/tool-schema footprint.

Not default optimization targets:
- lower reasoning effort;
- arbitrary output/context caps;
- fewer subagents merely because they cost tokens;
- skipping final verification/tests;
- blocking Python or direct reads.

Any active retrieval must support expand/fallback. Any Project Memory fact must carry content/state provenance, freshness and invalidation.

## Long-term adaptive optimization gates - 2026-10-03

The long-term target is an evidence-driven adaptive efficiency layer, not a generic token compressor or a smaller coding agent.

New foundational rule: Context/Result Identity should be reusable infrastructure. When directly observable, record privacy-safe identity, source/version, provenance and freshness/invalidation without persisting raw outputs by default. This seeds future Context Twin semantics while preserving UNKNOWN/NOT_CAPTURED where host evidence is absent.

Opportunity-first rule: before implementing an optimizer, estimate addressable surface, bounded upper potential, frequency, confidence, quality risk, implementation complexity and experiment cost. `DEPRIORITIZE` is valid.

AutoLab is deliberately later and offline-first. Experience Bank / Feature Store / DatasetManifest / holdouts / drift handling precede any learned controller. Native Astra remains a baseline, and prediction/offline replay never becomes causal evidence by relabeling.

Algorithm order, if data later justifies it: deterministic/statistical methods -> contextual bandit -> Bayesian optimization -> learning-to-rank -> calibrated surrogate -> evolutionary search offline -> optional specialized LLM analyst.

No learned policy self-deploys. Shadow, causal A/B, quality gate, runtime compatibility, rollback and post-deploy monitoring remain mandatory for active promotion.
