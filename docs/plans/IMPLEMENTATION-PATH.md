# Tokn Implementation Path

Status: ACTIVE EXECUTION PLAN
Date: 2026-09-30
Current state: P0-P9 DONE / Experiment 002 NEXT

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

Status: NEXT

Purpose:
one small real Astra/Codex task validates V0.1 instrumentation end-to-end.

This is not an optimization experiment.

Required:
- small bounded task;
- known starting workspace;
- current runtime/model/profile captured;
- Runner used from start to finish;
- no manual evidence repair;
- post-run evidence folder complete;
- validity expected to reflect instrumentation purpose honestly.

Exit:
V0.1 works on one new real run, not only the historical golden.

## Phase D - Measurement Contract Freeze

Status: AFTER EXPERIMENT 002

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

Status: AFTER CONTRACT FREEZE

Build the smallest local persistent store for:
- projects/workspaces;
- runs;
- agents;
- runtime/model profiles;
- usage summaries;
- rate-limit snapshots;
- findings;
- experiments;
- provenance.

Do not store raw prompts or secrets by default.

Implement MODEL-RUNTIME-PROFILE.md from observed fields,
not speculative universal model metadata.

## Phase F - Local Codex integration prototype

Status: AFTER STORE FOUNDATION
Direction: RESEARCH-BACKED / NOT YET ACCEPTED AS FINAL TRANSPORT

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

Exit:
plugin integration starts/stops cleanly, reports capabilities/version,
uses shared Engine logic, exposes structured errors, and does not break
standalone Runner operation.

## Phase G - Historical Analyzer + Context Ledger

Status: AFTER RELIABLE STORE / CAN BEGIN BEFORE FULL UX POLISH

Build observation-only analytics:
- per-run/per-turn/per-agent token ledger;
- cached / uncached / cache-write / output / reasoning;
- rate-limit snapshots over time;
- tool/file activity timeline;
- parent/subagent graph;
- compaction events when observable;
- repeated reads/searches/retries;
- duplicate evidence;
- rediscovery across runs.

Important:
model_context_window, total usage and last usage must not be called
current retained-context occupancy until OQ-005 is resolved.

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

Do not start before Experiment 002 and the Measurement Contract Freeze:
- production plugin package;
- MCP adapter implementation;
- local daemon;
- GUI;
- persistent Store schema implementation;
- active optimization;
- RAG/embeddings;
- Project Memory;
- context rewriting.

The research documents may guide future design,
but they are not a reason to skip measurement hardening.

## Current decision

Immediate product work is Experiment 002 instrumentation validation only.

Order:
1. freeze the Experiment 002 task and starting workspace ;
2. validate the preflight without launching Astra ;
3. run one small real Codex/Astra task through Runner V0.1 ;
4. inspect the self-contained evidence folder ;
5. accept or reject instrumentation validity ;
6. if accepted, freeze the V0.1 Measurement Contract ;
7. only then begin Store/Profile and historical analysis layers.
