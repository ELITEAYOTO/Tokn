# ADR-006 - Multi-Runtime Core / Adapter Boundary

Status: ACCEPTED
Date: 2026-10-03

## Context

Tokn is currently validated first against Codex, but its long-term analytical value must not depend on one provider, model, trace shape, tool protocol or runtime version.

Future runtimes may expose different telemetry and different semantics for usage, cache, reasoning, compaction, subagents, quotas and tools.

A provider-specific analytical core would make every new runtime a partial rewrite and would contaminate AutoLab data with false equivalences.

## Decision

Tokn adopts one provider-neutral analytical Core with thin runtime adapters.

The Core reasons about generic concepts such as runs, agents, tool operations, context/evidence, usage, quality, terminal state, findings, policies and experiments.
Runtime adapters own provider/runtime-specific detection, parsing, normalization, capability mapping, usage-semantics translation, runtime-profile construction, provenance mapping, sanitization and adapter diagnostics.

Generic reducers must prefer capability/evidence checks over runtime-name branches.

Missing evidence remains explicit (`UNKNOWN`, `NOT_CAPTURED`, or another evidence-aware state) and must never be converted to zero, false or unsupported without proof.

Provider-specific concepts may remain adapter-local until they justify a genuinely generic, versioned Core concept.

## Token semantics

OpenAI/Codex accounting is not a universal token contract.

Before runtime #2 is treated as a serious analytical target, Tokn must define a provider-neutral Token Semantics V2 that preserves metric identity, unit, semantic meaning, source, scope and evidence/coverage.

## Runtime gates

Before runtime #2:
- Runtime Adapter Contract V1;
- Token Semantics V2;
- Runtime Capability Manifest V1;
- sanitized conformance fixtures and adapter contract tests.
Runtime #2 is an architectural learning exercise: validate which abstractions are genuinely generic and refactor only what real evidence requires.

Runtime #3 is the maturity test: adding it should require little or no change to the principal reducers.

A public adapter SDK must wait until at least 2-3 real adapters have exercised the internal contract.

## Normalized evidence rule

New durable evidence should be replay/rebuild friendly, but this ADR does not mandate an Event Store rewrite.
Store V2 remains valid until a measured need justifies migration.

Cross-runtime benchmarks are descriptive comparisons unless the causal experiment contract is actually satisfied. Runtime/model changes are not presented as proof of a Tokn policy effect.

## Consequences

Tokn remains local-first and offline-testable.
Runtime-specific dependencies stay outside generic reducers when possible.
Advanced analytics may remain capability-conditional instead of collapsing Tokn to the lowest common denominator.
AutoLab can later learn policies scoped by runtime, task, project and capabilities rather than assuming one universal policy.

See `docs/design/MULTI-RUNTIME-CORE-AND-DATA-ARCHITECTURE.md` for the long-term design contract.
