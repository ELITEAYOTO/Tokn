# ADR-005 - Hybrid Tokn Architecture

Status: ACCEPTED
Date: 2026-09-30

## Context

Tokn must remain trustworthy across changing model/runtime integrations.
The analytical core must not depend on Codex UI details, plugin packaging,
or any one transport protocol.

## Decision

Tokn adopts a hybrid architecture:

- the local Rust Core/Engine is the analytical source of truth;
- a local Store persists runs, runtime/model profiles, baselines, findings,
  experiments, provenance and confidence;
- runtime adapters translate external evidence into Tokn's normalized domain;
- Codex integration is a thin UX/integration layer, not a second analyzer;
- a future Desktop UI consumes the same Engine/Store contracts;
- an Optimizer is future-only and remains blocked until causal evidence exists.

The exact Codex integration surface is intentionally not frozen here.
Skills, MCP, hooks, process invocation, IPC or a local service may be used
only when their current capabilities are VERIFIED for the target runtime.

## Quality policy

Tokn optimizes avoidable context work, not model capability.

Hard output-token caps, arbitrary reasoning reduction, arbitrary subagent limits,
reduced verification depth, or smaller context windows are not primary
optimization levers.

Any active transformation requires a controlled experiment with a quality gate.

## V0.1 dependency rule

P8 Runner V0.1 and P9 release validation do not depend on a plugin.

P8 should create the stable Runner/Engine boundary that later integrations call.
Plugin/service work must not duplicate P0-P7 reducers.

## Runtime/model rule

Model/runtime behavior is represented through versioned capability profiles.
Capabilities are detected or evidenced, never assumed universal.

UNKNOWN remains UNKNOWN.
NO_EVIDENCE is not PASS.
Correlation is not causation.

## Consequences

Tokn Core remains testable offline and independent of Codex.
Integration layers may evolve without rewriting analytical logic.
Historical analysis becomes possible through the local Store.
A future optimizer can only act on reproducible findings validated causally.
