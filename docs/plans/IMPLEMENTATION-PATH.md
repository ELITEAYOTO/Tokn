# Tokn Implementation Path

Status: ACTIVE EXECUTION PLAN
Date: 2026-09-30
Current state: P0-P9 DONE / Experiment 002 ACCEPTED / Measurement Contract V1 FROZEN / Store foundation DONE / Local MCP transport prototype ACCEPTED / Historical Analyzer + Context Ledger IN PROGRESS

## Purpose

This document answers one question:
what should Tokn implement next, in what order, and what evidence is required
before moving to the following layer?

It complements:
- ROADMAP.md for milestone order;
- V0.1-IMPLEMENTATION-PLAN.md for V0.1 details;
- TARGET-ARCHITECTURE.md for long-term boundaries.

## Execution rule

Do not start the next layer because it is attractive.
Start it only when the previous boundary is stable enough to consume.

Current mandatory sequence:

P8 Runner
-> P9 release validation
-> Experiment 002
-> Measurement Contract Freeze
-> Store + ModelRuntimeProfile
-> Local MCP Integration Prototype
-> Historical Analyzer + Context Ledger
-> Findings Engine
-> Experiment 003 causal A/B
-> Advisor
-> optional active Context Compiler / Desktop expansion.

## Phase A - P8 Runner V0.1

Status: DONE
Priority: BLOCKER

Goal:
one command creates a complete, self-contained evidence folder
without manual forensic reconstruction.

### P8.1 - Runner contract

Define one machine-readable request and result contract.

Request must identify:
- task/run intent;
- root session or discovery source;
- initial workspace;
- evidence/output destination;
- quality command when applicable;
- experiment intent.

Result must expose:
- evidence folder;
- RunGroup identity;
- resolved workspace;
- terminal status;
- source health;
- policy evidence;
- quality result;
- experiment validity;
- runtime/model/task metadata;
- structured errors/warnings.

Do not add Store/plugin concerns here.

### P8.2 - Orchestration shell

Create a focused Runner orchestrator that calls existing P1-P7 logic.

Order:
1. initialize evidence folder safely;
2. identify/import root session;
3. discover/import descendants exactly once;
4. evaluate source health/fallback;
5. recover terminal/runtime/model/task metadata;
6. resolve actual output workspace;
7. compute diff against the correct initial root;
8. execute quality gate on the resolved output;
9. produce policy evidence;
10. reduce experiment validity;
11. write final manifest/result;
12. restore temporary placements even after interruption.

Existing reducers remain the source of truth.

### P8.3 - Evidence folder contract

The folder should be understandable without the original live session.

Minimum categories:
- request/metadata;
- normalized session evidence;
- RunGroup summary;
- usage summary;
- terminal status;
- workspace resolution;
- diff summary;
- quality result;
- policy evidence;
- validity result;
- provenance/source-health;
- runner manifest.

Raw private artifacts are referenced or sanitized according to privacy rules.

### P8.4 - Failure and recovery

Must fail conservatively for:
- missing root session;
- malformed/empty evidence with no fallback;
- ambiguous workspace;
- unavailable quality gate;
- partial policy evidence;
- interrupted execution.

Temporary policy/AGENTS placements must be restored on success or recovery.

UNKNOWN remains UNKNOWN.
A runner failure must not silently become a valid experiment.

### P8.5 - Offline acceptance

Before any new Astra run:
- sanitized fixture replay;
- Experiment 001 golden replay;
- all P8 failure cases;
- deterministic result/evidence schema tests.

Exit:
Experiment 001 can be represented end-to-end by the Runner
without manual timestamp heuristics or forensic repair.

## Phase B - P9 V0.1 release validation

Status: DONE
Priority: BLOCKER

Run:
- cargo fmt --check;
- clippy --workspace --all-targets -- -D warnings;
- cargo test --workspace;
- release build;
- sanitized fixture replay;
- Experiment 001 golden replay;
- package/privacy check;
- documentation consistency;
- git diff --check.

Any golden regression blocks release.

Exit:
V0.1 measurement pipeline is reproducible offline from a clean commit.

## Phase C - Experiment 002

Status: DONE / ACCEPTED

Purpose:
one small real Astra/Codex task validates V0.1 instrumentation end-to-end.

This is not an optimization experiment.

Observed acceptance:
- pipeline COMPLETE ;
- root terminal COMPLETED ;
- quality PASS (`npm run verify:local`, 863/863 tests) ;
- 1 parent + 1 descendant, 49 usage records ;
- selected workspace matches the expected B07-C workspace ;
- diff 1 added / 2 modified / 0 removed ;
- model `gpt-6.1-sol` recorded ;
- configuration completeness remains UNKNOWN ;
- validity INSTRUMENTATION_ONLY ;
- causal claims blocked, descriptive metrics allowed.

The first FINISH attempt exposed two harness defects, not a task failure:
PowerShell LASTEXITCODE handling and CLI diagnostic-bundle nesting.
The immutable persisted Codex rollout was replayed through Runner without another model run.
The deterministic session fallback is now versioned and covered by CI.

Required:
- small bounded task;
- known starting workspace;
- run-scoped runtime/model/config values captured with provenance;
- configuration completeness may remain UNKNOWN until ModelRuntimeProfile is frozen after this experiment;
- Runner used for the accepted evidence reduction;
- no invented or mutated evidence; deterministic harness-only recovery may reuse immutable persisted rollouts when fully documented and regression-tested;
- post-run evidence folder complete;
- validity expected to reflect instrumentation purpose honestly.

Exit:
V0.1 works on one new real run, not only the historical golden.

## Phase D - Measurement Contract Freeze

Status: DONE / FROZEN V1

Freeze the interfaces that downstream systems may depend on:
- Runner request/result schema;
- evidence folder layout;
- RunGroup identity;
- usage accounting semantics;
- source-health/provenance;
- terminal status;
- workspace lineage;
- validity contract;
- capability/profile representation.

Breaking changes remain possible later, but become explicit schema migrations.

## Phase E - Tokn Store + ModelRuntimeProfile

Status: FOUNDATION DONE / EXTENSIONS DEFERRED

Foundation implemented:
- projects/workspaces with privacy-preserving logical identities;
- runs;
- agents;
- runtime/model profiles;
- usage summaries with known counters;
- provenance fingerprints;
- versioned Store schema + fail-closed migrations;
- `store-evidence` ingestion from immutable Runner evidence.

The rate-limit table exists, but ingestion remains deferred.
Findings and Experiment Lab persistence remain deferred until their consumer contracts are justified.

Do not store raw prompts or secrets by default.
Legacy source paths are pseudonymized and physically removed on the one-time V2 migration.

ModelRuntimeProfile V1 persistence is implemented from observed fields,
not speculative universal model metadata.

## Phase F - Local Codex integration prototype

Status: DONE / TRANSPORT PROTOTYPE ACCEPTED
Direction: LOCALLY VALIDATED / PRODUCTION PLUGIN PACKAGING STILL SEPARATE

Preferred first prototype:
Codex plugin -> command-launched local MCP adapter -> shared Rust Engine/Store.

Why:
current local Codex already demonstrates this lifecycle with OpenAI plugins.

Initial surface should remain thin:
- doctor/status;
- analyze evidence;
- run/observe through Runner contract;
- fetch findings/history.

No analysis logic in plugin glue.
No permanent localhost daemon unless a measured requirement appears.

Observed exit evidence:
- `tokn-mcp` starts/stops cleanly over stdio ;
- version/capability reporting is explicit ;
- tools are read-only and backed by shared Store APIs ;
- errors are structured ;
- release-process smoke passes ;
- Codex 0.161.0-alpha.2 accepts stdio registration ;
- Codex app-server launches Tokn and discovers `tokn_status` + `tokn_recent_runs` with `toolsError=null` ;
- an ephemeral idle zero-turn Codex thread is created locally ;
- Codex directly invokes `tokn_status` through `mcpServer/tool/call` ;
- no user authentication material is copied or inspected ;
- standalone Runner remains independently usable.

## Phase G - Historical Analyzer + Context Ledger

Status: IN PROGRESS

Accepted current slice:
- HistoricalSnapshot V1 from Store V2 ;
- multi-run project/workspace/run/agent history ;
- WorkspaceLineage ;
- per-run/per-agent token ledger ;
- cached / uncached / cache-write / output / reasoning with explicit coverage ;
- terminal / quality / validity / runtime profile / provenance ;
- integrity mismatches reported rather than repaired ;
- `tokn-observe context-ledger` CLI ;
- read-only `tokn_context_ledger` MCP tool ;
- standalone stdio and target Codex 0.161.0-alpha.2 validation PASS.

Contract V1 boundaries:
- per-turn ledger is `NOT_CAPTURED` because the persisted V1 history does not carry authoritative per-turn usage ;
- current retained-context occupancy is `UNKNOWN` ;
- model_context_window, total usage and last usage must not be relabeled as current retained context.

Still required before Phase G is DONE:
- rate-limit snapshots over time;
- tool/file activity timeline;
- parent/subagent shared-evidence analysis;
- compaction events when observable;
- repeated reads/searches/retries;
- duplicate evidence;
- rediscovery and explicit cross-run comparison primitives.

## Phase H - Findings Engine

Convert repeated observations into evidence-backed findings.

Candidate types:
- context_amplification;
- duplicate_file_reads;
- subagent_context_duplication;
- cache_churn/collapse;
- retry_loop;
- compaction_rediscovery;
- oversized_tool_evidence;
- repeated_project_discovery.

Every finding requires:
evidence, provenance, compatible profile scope, frequency,
confidence, observed cost, bounded potential impact and quality risk.

## Phase I - Experiment 003

Status: ONLY AFTER ONE REPRODUCIBLE FINDING

First causal optimization A/B.

Requirements:
- identical frozen starting workspace;
- identical task;
- compatible model/runtime/config;
- one primary intervention;
- quality gate passes before token comparison;
- validity = VALID_FOR_CAUSAL_AB.

The intervention is chosen from evidence.
It is not assumed to be an output cap.

## Phase J - Advisor

Turn validated recurring findings into recommendations.

No automatic mutation yet.

Recommendations must include:
- why;
- evidence;
- confidence;
- expected bounded impact;
- quality risk;
- reversible experiment suggestion.

## Phase K - Active layers

Only after repeated causal wins:
- Context Compiler;
- Project Memory;
- selective evidence compression;
- cache-aware context construction;
- Desktop UX expansion;
- optional long-lived service if genuinely required.

These remain separate from the Analyzer.

## What not to build now

Current non-goals while Historical Analyzer + Context Ledger is IN PROGRESS:
- production plugin package;
- permanent local daemon;
- GUI;
- Findings persistence before a Findings contract exists;
- Experiment Lab persistence before its consumer contract exists;
- active optimization;
- RAG/embeddings;
- Project Memory;
- context rewriting.

The MCP prototype must remain process-bound and thin over the already validated
Runner/Store boundaries. Research documents may guide the prototype, but they are
not a reason to widen scope prematurely.

## Current decision

Immediate product work remains **Historical Analyzer + Context Ledger** observation-only.

The Measurement Contract V1, Store V2 foundation and local MCP transport boundary are stable inputs.

Completed in the current slice:
1. Store queries for project/workspace/run/agent history ;
2. normalized context/token ledger with coverage semantics preserved ;
3. terminal, quality, validity, runtime profile and provenance beside usage ;
4. workspace lineage ;
5. current-context occupancy explicitly UNKNOWN unless directly evidenced ;
6. synthetic history tests + full release/golden regressions ;
7. CLI + read-only MCP exposure, including direct target-Codex tool-call validation.

Next inside M4:
1. persist only privacy-safe tool/file activity evidence needed by history ;
2. build activity/phase timeline from direct evidence ;
3. add repeated reads/searches/retries and shared/duplicate evidence analysis ;
4. add compaction/rediscovery observations where directly supported ;
5. add explicit multi-run comparison primitives ;
6. only after those observations are reliable, begin M5 Findings.

## 2026-10-02 insertion - shadow before active optimizer

Current M4 activity foundation is accepted. Next: automatic runtime-profile compatibility, rate-limit snapshots, shared/duplicate evidence and rediscovery primitives.

Before Findings become interventions, introduce an observation-only shadow layer where useful:
- repo/symbol index + Git/hash invalidation;
- shadow context retrieval;
- shadow edit-strategy classification.

Before causal optimization:
- pilot run-to-run variance;
- quality acceptance defined before the run;
- automatic compatible runtime/model/config evidence;
- one primary intervention;
- measure Tokn's own injected/default-active overhead.

Before public distribution: threat model, retention/purge/export, dependency audit + SBOM, signing/update integrity.

Runtime #2 comes after stable Codex V1 and requires provider-neutral token semantics plus real sanitized Claude/Cowork evidence.
