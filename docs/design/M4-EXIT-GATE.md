# M4 Core Exit Gate

Status: ACCEPTED / CORE COMPLETE WITH EVIDENCE-GATED EXTENSIONS
Date: 2026-10-03

## Purpose

Define the exact boundary at which Historical Analyzer + Context Ledger can stop accumulating speculative observation primitives and M4.5 can proceed.

This gate does **not** mean every desirable signal is captured. It means the M4 core is stable, privacy-safe, fail-closed, and sufficient for the next observation-only shadow layer. Missing evidence remains explicitly missing instead of blocking the roadmap forever or being backfilled by heuristics.

## Exit decision

M4 core is **ACCEPTED**.

M4.5 Context Identity + Shadow Foundations may proceed observation-only from the accepted contracts.

Future direct evidence may still extend M4 additively. Those extensions do not reopen or weaken the accepted core unless they require a contract migration or change an existing semantic guarantee.

## Accepted M4 core

The accepted core includes:
- HistoricalSnapshot V1 and project/workspace/run/agent history;
- Context Ledger V1 with coverage-aware token accounting and integrity reporting;
- WorkspaceLineage, terminal, quality, validity, runtime profile and provenance;
- read-only CLI and local MCP Context Ledger surfaces;
- ToolActivityHistory V3 and ActivityTimeline V1;
- automatic ModelRuntimeProfile compatibility evidence;
- RateLimitHistory V1;
- Cross-Agent Evidence V2;
- exact privacy-safe result/content identity when directly observable;
- SourceStableId file V0 + SourceIdentityHistory V1;
- SourceMutationHistory V1 and Source Mutation Window V0;
- Run-Boundary Source Version V0 + SourceVersionHistory V1;
- Workspace Git Provenance V0/V1;
- Source Freshness Evidence V0 corroboration with freshness/invalidation still `NOT_PROVEN`;
- Cross-Run Comparability V0 with causal claims still `NOT_ESTABLISHED`;
- Task Input Identity V0 with runtime delivery still `NOT_PROVEN`;
- Cross-Run Source Re-read Evidence V0 with rediscovery/redundancy/freshness still `NOT_PROVEN`.

## Evidence-gated M4 extensions

The following remain intentionally outside the accepted core until direct evidence exists:
- authoritative per-turn token/context ledger: `NOT_CAPTURED`;
- current retained-context occupancy: `UNKNOWN`;
- semantic phase timeline: `NOT_CAPTURED` because current fixtures/adapters expose no authoritative phase marker;
- detailed compaction chronology: `NOT_CAPTURED` until a real sanitizable event sample exists;
- runtime task delivery: `NOT_PROVEN`;
- interpreted rediscovery: `NOT_PROVEN`;
- redundancy/waste from a repeated read or duplicate result: `NOT_PROVEN`;
- general freshness/staleness or invalidation correctness: `NOT_PROVEN`;
- broader source kinds such as symbol/range identity: evidence-gated until stable identity is directly provable.

These states are valid product outputs, not implementation failures.

## Prohibited inference shortcuts

M4 exit does not authorize any of the following:
- tool categories -> semantic phase labels;
- Store ingestion timestamps -> runtime chronology;
- repeated source read -> model forgot / rediscovery;
- exact-equal result -> unnecessary read / token waste;
- frozen task artifact -> runtime delivery;
- observed mutation call -> durable mutation effect;
- source change -> causal attribution to one tool call;
- cached input -> useful context;
- fewer tokens -> quality-preserving optimization.

If a future slice needs one of these claims, it must introduce direct evidence or keep the result `UNKNOWN` / `NOT_CAPTURED` / `NOT_PROVEN`.

## Why phase timeline is not implemented now

The current sanitized fixtures and Codex adapter contain no authoritative `phase`/`stage` marker for semantic states such as planning, implementation or validation.

Deriving phases from tool categories, gaps, filenames or command families would create an inferred taxonomy and then present it as runtime fact. That is outside M4's evidence discipline.

A future runtime may expose direct phase markers. If so, phase history can be added as a versioned evidence surface without changing this exit decision.

## M4.5 entry conditions

M4.5 must remain shadow/observation-only and preserve the provider-neutral Core boundary.

The first M4.5 work should:
- keep Astra/runtime semantic decisions untouched;
- keep the Measurement Contract V1 frozen unless new evidence truly requires a versioned change;
- keep shadow indexes rebuildable and separate from durable measurement evidence;
- prefer Git/hash incremental invalidation over full rescans;
- start lexical/repository indexing only with a measurable retrieval need;
- defer embeddings until measured recall/value justifies them;
- preserve provenance, freshness coverage and expand/fallback paths for future Context Packages.

The recommended first slice is **Shadow Repository Index V0 design + measurement contract**, before implementing active retrieval or context injection.

## M5 gate

M5 Findings + Opportunity Analyzer must not reinterpret the accepted M4 primitives as findings automatically.

In particular:
- re-read chronology is not rediscovery;
- duplicate identity is not waste;
- changed source evidence is not stale-context proof;
- opportunity estimates remain separate from causal savings claims.

Experiment 003 remains the first causal optimization A/B after a reproducible finding, compatible run scope, predeclared quality gates and one primary intervention.

## Operational status

M4 core exit changes documentation/status only. It introduces:
- no Store migration;
- no Measurement Contract change;
- no Runner contract change;
- no new runtime parser;
- no active optimizer behavior.

Repository governance remains separate housekeeping: `main` is currently unprotected, so PR + green CI + expected-head SHA remains an enforced team discipline until GitHub branch protection/rulesets are configured.