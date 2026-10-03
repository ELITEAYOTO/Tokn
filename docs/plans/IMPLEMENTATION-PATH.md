# Tokn Implementation Path

Status: ACTIVE EXECUTION PLAN
Date: 2026-10-03
Current state: P0-P9 DONE / Experiment 002 ACCEPTED / Measurement Contract V1 FROZEN / Store foundation DONE / Local MCP transport prototype ACCEPTED / M4 core ACCEPTED / M4.5 OBSERVATION-ONLY STARTED / Context-Result Identity V0 ACCEPTED / Source Freshness Evidence V0 ACCEPTED CORROBORATION FOUNDATION / Cross-Run Comparability V0 ACCEPTED OBSERVATION FOUNDATION / Task Input Identity V0 ACCEPTED OBSERVATION FOUNDATION

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
-> Context/Result Identity + Context Twin seed
-> Shadow Retrieval/Edit foundations
-> Findings + Opportunity Analyzer
-> deterministic counterfactual + PolicyCandidate/Registry
-> Experiment 003 causal A/B
-> Advisor
-> selective Context Compiler / Project Memory
-> offline-first AutoLab only after evidence/experiment contracts stabilize.

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

The prepared rate-limit table is now populated from directly observed Codex token-count telemetry, with a privacy-minimized historical query contract.
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
- Codex app-server launches Tokn and discovers `tokn_status` + `tokn_recent_runs` + `tokn_context_ledger` with `toolsError=null` ;
- an ephemeral idle zero-turn Codex thread is created locally ;
- Codex directly invokes `tokn_status` and `tokn_context_ledger` through `mcpServer/tool/call` ;
- no user authentication material is copied or inspected ;
- standalone Runner remains independently usable.

## Phase G - Historical Analyzer + Context Ledger

Status: CORE ACCEPTED / EVIDENCE-GATED EXTENSIONS

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

Phase G evidence status:
- rate-limit snapshots over time: ACCEPTED FOUNDATION;
- tool/file activity timeline: ACCEPTED FOUNDATION;
- parent/subagent exact-operation overlap analysis: ACCEPTED FOUNDATION;
- Source Mutation Window V0: ACCEPTED CHRONOLOGY FOUNDATION;
- Source Freshness Evidence V0: ACCEPTED CORROBORATION FOUNDATION / observation-only over mutation-window + SourceVersion + Workspace Git, with freshness/invalidation still `NOT_PROVEN`;
- Cross-Run Comparability V0: ACCEPTED OBSERVATION FOUNDATION / observation-only scope comparison over project/contract/runtime/SourceVersion BEFORE/Git BEFORE; even PASS keeps causal claims `NOT_ESTABLISHED`;
- Task Input Identity V0: ACCEPTED OBSERVATION FOUNDATION; exact frozen task-artifact identity is project-scoped/privacy-safe, runtime delivery remains `NOT_PROVEN`, and Cross-Run causality remains `NOT_ESTABLISHED`;
- Cross-Run Source Re-read Evidence V0: ACCEPTED CHRONOLOGY FOUNDATION; same-source cross-run order requires complete parseable rollout timestamps, while rediscovery/redundancy/freshness stay `NOT_PROVEN`;
- detailed compaction chronology: `NOT_CAPTURED` until a real diagnostic event is observed/sanitized; diagnostic seq is not assumed comparable to rollout seq;
- repeated reads/searches/retries: exact-repeat foundation accepted, richer analysis remaining;
- exact result identity: foundation accepted; duplicate evidence remains allowed only when complete observed identity proves equality;
- interpreted rediscovery remains future work beyond the direct re-read chronology primitive.

## Phase G.5 - Context Identity + Shadow Foundations

Status: STARTED / OBSERVATION-ONLY

Accepted pre-implementation slice:
- Shadow Repository Index V0 design: file-level first, DERIVED/local/rebuildable, separate from Measurement Store;
- exact current-file hash verification before returning source text;
- Git/hash incremental invalidation with fail-closed verification/rebuild fallback;
- DIRECT_SCAN_V0 reference condition before persistent backend selection;
- SQLite FTS5 unicode61/trigram are measurement candidates only; no backend is selected;
- Shadow Index Benchmark Protocol V0 + ShadowIndexMeasurement V1 freeze retrieval/build/refresh/resource/privacy metrics before implementation;
- no MCP retrieval tool, no active runtime hook, no context injection and no token-savings claim.

Implementation prerequisites:
1. reuse the exact existing SourceStableId derivation without making the shadow index depend on Store internals or duplicating the algorithm;
2. define a separate versioned IndexContentHash domain for whole-file bytes;
3. define local index location/retention outside tracked/package artifacts;
4. implement bounded DIRECT_SCAN_V0 corpus/query semantics and sanitized gold fixtures;
5. capability-probe FTS5 and benchmark candidates only after the reference path exists.

Exit toward active shadow retrieval requires a backend to meet every predeclared quality/correctness/resource gate. Active context injection remains a later separate gate.

## Phase H - Findings + Opportunity Analyzer V0

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

Before engineering an optimizer, Opportunity Analyzer V0 estimates separately:
- addressable surface and theoretical upper bound;
- realistic bounded potential;
- frequency and confidence;
- quality/preservation risk;
- implementation complexity and experiment cost.

A valid conclusion is DEPRIORITIZE.

## Parallel baseline / benchmark readiness

Status: PROTOCOL PREPARED / MAY RUN DURING M4.

Use `docs/benchmarks/BASELINE-PROTOCOL-V1.md` to collect native-agent variance and cost surfaces without an active Tokn intervention.
Store raw runs locally only; commit sanitized manifests/aggregates only.
Do not create a fake passive-observer A/B. The baseline prepares sample sizing, quality/exclusion rules, cache handling and Tokn-overhead measurement for Experiment 003.

## Phase I - Counterfactual / Policy Foundation + Experiment 003

Status: ONLY AFTER ONE REPRODUCIBLE FINDING + OPPORTUNITY

Before the first causal optimization A/B:
- retain Astra native as explicit baseline;
- replay bounded deterministic variants offline when meaningful;
- introduce a versioned PolicyCandidate / PolicyGenome V0;
- record scope, runtime constraints, provenance, evidence references and lifecycle state;
- keep offline estimates distinct from causal outcomes.

Experiment 003 remains the first causal optimization A/B.

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

## Phase L - AutoLab V0 / offline data foundation

Future only after M4-M6 contracts are stable:
- Experience Bank derived from structured Store evidence where possible;
- Feature Store with observed features separated from derived labels;
- DatasetManifest and schema/runtime/task/privacy metadata;
- time/project/runtime holdouts where applicable;
- deterministic replay, Offline Fidelity and drift detection;
- Policy Registry and native baseline;
- bounded raw retention plus purge/export/migration rules.

No ML model is required for this phase.

## Phase M - Adaptive policy learning

Research only after enough data and after simpler methods plateau:
- contextual bandit first;
- Bayesian optimization;
- learning-to-rank;
- calibrated surrogate model;
- evolutionary search offline only;
- optional LLM analyst only if structured methods remain insufficient.

No learned policy self-deploys.

## What not to build now

Current non-goals while M4.5 shadow foundations are observation-only:
- production plugin package;
- permanent local daemon;
- GUI;
- Findings persistence before a Findings contract exists;
- Experiment Lab persistence before its consumer contract exists;
- active optimization;
- AutoLab / Experience Bank / Feature Store / learned policy controller;
- RAG/embeddings;
- speculative multi-runtime crate split or public Adapter SDK;
- runtime #2 implementation before its adapter/token/capability contracts and sanitized evidence exist;
- Project Memory;
- context rewriting.

The MCP prototype must remain process-bound and thin over the already validated
Runner/Store boundaries. Research documents may guide the prototype, but they are
not a reason to widen scope prematurely.

## Current decision

M4 Historical Analyzer + Context Ledger core is **ACCEPTED**. Immediate product work moves to **M4.5 Context Identity + Shadow Foundations** observation-only; unavailable M4 signals remain evidence-gated extensions.

The Measurement Contract V1, Store V2 foundation and local MCP transport boundary are stable inputs.

Completed in the current slice:
1. Store queries for project/workspace/run/agent history ;
2. normalized context/token ledger with coverage semantics preserved ;
3. terminal, quality, validity, runtime profile and provenance beside usage ;
4. workspace lineage ;
5. current-context occupancy explicitly UNKNOWN unless directly evidenced ;
6. synthetic history tests + full release/golden regressions ;
7. CLI + read-only MCP exposure, including direct target-Codex tool-call validation.

M4 core exit state:
1. Context/Result Identity Foundation V0 is ACCEPTED; keep duplicate-evidence classification gated on complete observed identity ;
2. SourceStableId file V0 + exact observed content-evolution history are ACCEPTED; broader source kinds remain evidence-gated ;
3. ToolActivityHistory V3 + SourceMutationHistory V1 are ACCEPTED for directly observed mutation-operation timing; effect remains `NOT_VERIFIED` ;
4. Run-Boundary Source Version V0 + SourceVersionHistory V1 are ACCEPTED from directly observed Runner BEFORE/AFTER snapshots ;
5. Workspace Git Provenance V0 + WorkspaceGitProvenanceHistory V1 are ACCEPTED from directly observed Runner boundary snapshots ;
6. Source Mutation Window V0 is ACCEPTED as a fail-closed same-thread read-before -> mutation -> read-after chronology primitive; exact equality/difference is observed, while mutation causality remains `NOT_PROVEN` ;
7. Source Freshness Evidence V0 is ACCEPTED CORROBORATION FOUNDATION: join the accepted chronology with exact SourceVersion + Workspace Git provenance, report only observed change/reread corroboration, and keep `freshness_status` / `invalidation_status` at `NOT_PROVEN` ;
8. Cross-Run Comparability V0 is ACCEPTED OBSERVATION FOUNDATION: compare explicitly selected runs only on captured project/contract/runtime/SourceVersion BEFORE/Git BEFORE scope and keep `causal_claims_status=NOT_ESTABLISHED` ;
9. Task Input Identity V0 is ACCEPTED OBSERVATION FOUNDATION: persist only project-scoped exact task-artifact identity/coverage, use the fail-closed Cross-Run task axis, and keep runtime delivery `NOT_PROVEN` ;
10. Cross-Run Source Re-read Evidence V0 is ACCEPTED CHRONOLOGY FOUNDATION: order same-source reads across runs only with complete direct rollout timestamps; never use Store ingestion time and never upgrade the sequence into rediscovery/redundancy/freshness ;
11. semantic phase timeline and detailed compaction remain `NOT_CAPTURED` until directly observed; interpreted rediscovery, runtime delivery and broader source kinds stay evidence-gated ;
12. `docs/design/M4-EXIT-GATE.md` accepts the M4 core without upgrading those missing signals; M4.5 may proceed shadow/observation-only while stronger stale/fresh semantics remain evidence-gated.

Parallel implementation hardening, only if measurement justifies it:
- benchmark current Codex raw-result retention with representative 1 MB / 10 MB / 50 MB and many-small-output fixtures;
- compare peak RSS, elapsed ingestion time, exact fingerprint equality and durable Store equality;
- if cost is non-trivial, replace retained raw result vectors with a private streaming/bounded fingerprint accumulator while preserving ambiguity and coverage semantics;
- defer immediately if the change requires Runner/Measurement/Store contract changes, a generic buffer crate or active tool-call deduplication.

## 2026-10-02 insertion - shadow before active optimizer

Current M4 activity foundation, runtime-profile compatibility reducer, rate-limit history, Cross-Agent Evidence V2, exact result identity, conservative SourceStableId file V0, mutation-operation timing, run-boundary source versions, Workspace Git Provenance V0 and Source Mutation Window V0 are accepted. Source Freshness Evidence V0 is the accepted corroboration foundation; it does not emit `FRESH` / `STALE`. Cross-Run Comparability V0 is accepted and does not establish causality. Task Input Identity V0 is an accepted extension: it can prove frozen task-artifact equality when directly captured, while runtime delivery remains `NOT_PROVEN`. Cross-Run Source Re-read Evidence V0 is an accepted chronology primitive and does not label later reads as rediscovery or redundancy. Detailed compaction remains `NOT_CAPTURED` until a real event can be fixture-tested; interpreted rediscovery and broader stable source kinds remain evidence-gated.

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

Runtime #2 comes after stable Codex V1 and is an architectural validation exercise, not only a support checkbox. Before it becomes analysis-ready Tokn requires Runtime Adapter Contract V1, Token Semantics V2, Runtime Capability Manifest V1, sanitized conformance fixtures and real runtime #2 evidence. Runtime #3 is the maturity test before any public Adapter SDK.

## 2026-10-03 long-term AutoLab / Intelligence Layer review

Accepted sequencing changes:
- Context Identity / Context Twin seed is a prerequisite for trustworthy duplicate-evidence and rediscovery analysis;
- Opportunity Analyzer V0 is part of M5 and may explicitly deprioritize low-addressable ideas;
- deterministic counterfactual replay + versioned policy schema/registry precede expensive causal experiments;
- active Context Compiler remains gated by repeated causal wins;
- AutoLab is a later offline-first track with Experience Bank, Feature Store, DatasetManifest, holdouts and drift handling;
- learned optimizers and any specialized LLM come only after simpler methods plateau and measured data is sufficient.

Event sourcing is a compatibility direction, not a current Store rewrite mandate: prefer versioned/rebuildable derived views and fail closed on unknown schemas.
## 2026-10-03 multi-runtime Core / Adapter review

Accepted architecture constraints:
- one provider-neutral analytical Core; runtime-specific details stay in thin adapters;
- new M4/M4.5 identities and evidence use runtime-neutral names where possible;
- capability/evidence checks are preferred to product/version branches in generic reducers;
- runtime, model, provider, host and configuration remain distinct identities;
- Context Identity separates stable source identity from privacy-safe content/result fingerprint;
- Query/UI/MCP layers should consume shared Core/Query contracts instead of coupling to SQLite;
- normalized events are a rebuildability direction, not a current Store rewrite;
- cross-runtime comparison is descriptive unless causal validity conditions are actually satisfied.

Canonical detail: `../design/MULTI-RUNTIME-CORE-AND-DATA-ARCHITECTURE.md` and ADR-006.
