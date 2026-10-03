# Tokn Multi-Runtime Core & Data Architecture

Status: ACCEPTED LONG-TERM DESIGN DIRECTION
Date: 2026-10-03

## Purpose

Define the durable boundary that lets Tokn support changing models, providers, coding agents, trace formats and tool protocols without duplicating the analytical engine.

Repository state remains authoritative for what exists today. This document is a design contract, not an implementation claim.

## North-star architecture

```text
Codex / Claude Code / OpenCode / future runtimes
                    |
                    v
             Runtime Adapters
                    |
                    v
            Normalized Contracts
                    |
                    v
                 Tokn Core
```

Tokn aims for **one analytical brain, many runtime translators**.

## Core rule

The Core reasons about runtime-neutral concepts:
- RUN / AGENT / TOOL OPERATION;
- CONTEXT / EVIDENCE / USAGE;
- SEARCH / READ / EDIT / TEST / ERROR;
- RUNTIME / MODEL / QUALITY / TERMINAL STATE;
- FINDING / OPPORTUNITY / POLICY / EXPERIMENT.

It must not reason directly on provider field names such as Codex JSONL keys or Claude-specific hooks.

Core stability is semantic, not immutability. A new runtime may reveal a genuinely generic concept; if so the Core may evolve through a versioned, tested and provider-neutral contract.

## Runtime Adapter responsibility

A runtime adapter is a translator, not a second analyzer.

Conceptual responsibilities:
1. runtime detection and identity;
2. artifact discovery;
3. evidence parsing;
4. event/evidence normalization;
5. capability probing/mapping;
6. usage semantics mapping;
7. runtime profile construction;
8. provenance mapping;
9. sanitization;
10. adapter diagnostics and gap reporting.
Adapters must not decide that repeated work is waste, rank opportunities, run causal comparisons or choose policies. Those decisions belong to the Core.

## Capability-driven analysis

Tokn should prefer observed capabilities over runtime-name checks.

Representative evidence-aware states:
- `OBSERVED`;
- `SUPPORTED`;
- `SUPPORTED_INSUFFICIENT_INPUT`;
- `NOT_CAPTURED`;
- `UNKNOWN`;
- `UNSUPPORTED_OBSERVED`;
- `NOT_APPLICABLE`.

Missing must never silently become false, zero or unsupported.

A versioned `RuntimeCapabilityManifest` may later describe run/profile capabilities such as total usage, per-agent usage, agent links, compaction evidence, tool-result identity and rate-limit telemetry.

Advanced analytics may be capability-conditional. Tokn must not collapse to the lowest common denominator across runtimes.

## Runtime / model / provider separation

Keep distinct:
- provider: OpenAI / Anthropic / other;
- model: model identity/version;
- runtime: Codex / Claude Code / OpenCode / other;
- host/app: CLI / Desktop / IDE;
- configuration: reasoning, sandbox, collaboration and feature flags.

## Token Semantics V2 gate

Current Codex/OpenAI token accounting must not be copied blindly to another provider.

Before runtime #2 is analysis-ready, define a provider-neutral usage metric contract carrying at least:
- metric identity;
- value and unit;
- semantic meaning/version;
- source;
- scope;
- evidence/coverage.

This allows metrics to remain different but comparable only where justified.

## Normalized evidence boundary

A future logical normalized-event vocabulary may include `RunStarted`, `AgentLinked`, `ReadObserved`, `SearchObserved`, `EditObserved`, `UsageObserved`, `RateLimitObserved`, `ContextEvidenceObserved`, `QualityObserved` and related events.

This is initially a logical boundary, not a mandate to rewrite Store V2 as an Event Store.
New data should be replay/rebuild friendly; derived views should be versioned or recalculable.

Unknown provider fields are never assigned invented semantics. They may be ignored with diagnostics, fingerprinted opaquely, kept adapter-local temporarily, or opened as a research question.

## Context identity contract

Context/Result Identity must be runtime-neutral.

Distinguish:
- `SourceStableId`: stable logical source identity when observable;
- `ContentFingerprint`: privacy-safe fingerprint of the observed version/content;
- source version/hash/commit when available;
- symbol/range when available;
- project scope, provenance, freshness and invalidation state.
This separation lets Tokn represent “same source, changed content” and supports deduplication, freshness, invalidation, rediscovery and cross-agent distribution without storing raw outputs by default.

## Data lifecycle

Use three conceptual classes:
- RAW: temporary/configurable retention;
- NORMALIZED: durable when privacy-safe;
- DERIVED: rebuildable/versioned whenever practical.

The normalized Store is Tokn's historical analytical record. Derived views such as Context Ledger, Activity Timeline, Cross-Agent Evidence, Findings and Opportunities should remain traceable to their source evidence/reducer version.

## Query / exposure boundary

UI, MCP and exports should not couple directly to SQLite schema.
A future internal Tokn Query API should expose goals such as run summary, activity timeline, context ledger, findings, opportunities, runtime-cohort comparison, policy performance and adapter health.

MCP should expose user/model goals (`tokn_context_query`, `tokn_findings`, `tokn_opportunities`, `tokn_compare_runs`) rather than internal algorithms such as BM25/AST/vector ranking.

## Retrieval/indexing direction

Start with measured needs:
- SQL indexes for common identities/time/runtime fields;
- activity/context fingerprints;
- SQLite FTS5/BM25 for lexical retrieval;
- LSP/Tree-sitter symbol data where valuable;
- dependency graph only where it improves retrieval/impact analysis;
- embeddings only after measured recall/value improvement.

Prefer incremental Git-diff invalidation over full rescans.

## Comparison semantics

Separate:
- causal intra-runtime policy A/B, where one primary intervention changes under compatible runtime/model/config evidence;
- descriptive inter-runtime benchmarks, where runtime/model differences are intentional.

Cross-runtime comparisons may report quality, observable cost, latency, tool calls, retries and findings, but must not be described as causal proof of a Tokn policy.

Future comparability states may distinguish `IDENTICAL_CONTRACT`, `SEMANTICALLY_COMPATIBLE`, `DESCRIPTIVELY_COMPARABLE`, `NOT_COMPARABLE` and `UNKNOWN`.

Always expose sample size, cohort/scope and uncertainty for aggregate comparisons.

## Adapter conformance and drift

Before serious support, each adapter should have sanitized versioned fixtures and contract tests for:
- identity extraction;
- unknown-field handling;
- privacy rules;
- run reconstruction;
- agent relationships;
- usage mapping;
- terminal/capability mapping;
- unsupported schema behavior;
- idempotent ingestion.

Adapter health should later track known runtime versions, parse failures, unknown records, capability changes and fixture coverage.
Schema drift must degrade explicitly instead of silently losing measurements.

## Multi-runtime maturity gates

Now / M4-M4.5:
- keep new identities/evidence runtime-neutral;
- finish Context/Result Identity, rediscovery and shadow foundations;
- do not perform a speculative adapter refactor.

Before runtime #2:
- Runtime Adapter Contract V1;
- Token Semantics V2;
- Runtime Capability Manifest V1;
- sanitized Conformance Fixture Kit.

Runtime #2 validates and corrects the abstractions using real evidence.
Runtime #3 is the maturity milestone: principal reducers should require little or no provider-specific change.
A public adapter SDK waits for 2-3 real adapters.

## Product and privacy invariants

Tokn remains local-first; raw code/prompts/paths/auth/reasoning are not default dashboard data.
Team/company views prefer privacy-safe project/runtime aggregates over developer scoring.

Long-term product progression is:
`Observability -> Diagnostics -> Opportunity -> Experiment -> Adaptive Policy -> Runtime Assistance`.

The broader North Star is same-or-better quality for lower total agent cost, where cost may include tokens, time, retries, duplication and failures.

## Current non-goals

Do not create speculative adapter crates, a public SDK, mandatory cloud, mass embeddings, or a Store rewrite during current M4 work.
Do not claim support for a runtime that lacks real sanitized evidence and contract coverage.
