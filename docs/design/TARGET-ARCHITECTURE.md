# Tokn Target Architecture

Status: ACCEPTED DESIGN DIRECTION
Date: 2026-10-01
Current state: P0-P9 DONE / Experiment 002 ACCEPTED / Measurement Contract V1 FROZEN / Store V2 foundation DONE / Local MCP transport prototype ACCEPTED / Historical Analyzer + Context Ledger IN PROGRESS

## Principle

Tokn is organized around a runtime-independent analytical engine.

Runtime -> Adapter/Evidence -> Tokn Engine -> Tokn Store -> Reports/Findings

UI and plugin surfaces consume this architecture; they do not replace it.

## Components

### Tokn Engine

Owns deterministic analysis and reducers:
source health, RunGroup reconstruction, terminal status, tool/policy evidence,
workspace resolution, quality evidence, validity, and future context analyzers.

P8 must provide a self-contained runner that orchestrates existing P0-P7 logic
without reimplementing those reducers.

### Evidence layer

Evidence includes sessions/descendants, token usage, tools, terminal status,
workspace lineage, runtime/model/config metadata, policy evidence, quality gates
and provenance.

Missing evidence remains explicitly UNKNOWN.

### Runtime adapters

Adapters translate runtime-specific artifacts into normalized Tokn inputs.
Codex is the first target adapter, not a dependency of the domain model.

### ModelRuntimeProfile registry

Profiles should carry model/runtime identity, version, observed capabilities,
observability gaps, historical baselines, recurring patterns, provenance
and confidence.

A missing capability must not be inferred as zero, false or unsupported.

### Tokn Store

Store V2 foundation is implemented after the frozen Runner/Measurement Contract boundary.
It currently persists privacy-preserving project/workspace identity, runs, agents,
usage summaries, provenance and ModelRuntimeProfile V1.

Findings, Experiment Lab and richer historical structures remain deferred until their
consumer contracts are justified.

### Codex integration

Codex integration is a thin activation/observation/presentation layer.

Current OpenAI documentation verifies the core packaging surfaces:
portable plugins, skills, MCP configuration and lifecycle hooks.

The local transport prototype is now measured:
Codex 0.161.0-alpha.2 can launch `tokn-mcp` as a command-based stdio server and
discover its read-only Store tools. Production/plugin packaging and explicit UX
remain separate concerns.

### Future Desktop UI

Desktop UI may visualize runs, agent graphs, ledgers, findings and experiments
using the same Engine/Store contracts.

### Future Optimizer

The Optimizer is disabled by design until Tokn has:
reproducible findings, a quality gate, a controlled A/B and causal validity.

## Stable boundary created by P8

Evidence sources
-> capture/import
-> normalization
-> P1-P7 reducers
-> validity
-> self-contained evidence folder
-> machine-readable result

Future integrations should call this boundary instead of rebuilding analysis.

## Transport decision

For local Codex integration, the validated prototype is:
Codex host -> command-launched `tokn-mcp` stdio adapter -> shared Rust Engine/Store.

The transport prototype is locally accepted, but this is still not an irreversible
production/plugin packaging ADR. The standalone Runner remains independent.

No permanent localhost service is required unless a measured need later justifies it.
Public/remote plugin distribution remains a separate deployment concern.

Selection criteria remain:
simplicity, local-first behavior, security, observability, portability,
versionability and low operational cost.

## Sequence after V0.1

P8 Runner DONE
-> P9 golden/release validation DONE
-> Experiment 002 instrumentation validation ACCEPTED
-> Measurement Contract V1 FROZEN
-> Store V2 + ModelRuntimeProfile foundation DONE
-> local Codex MCP transport prototype ACCEPTED
-> Historical Analyzer + Context Ledger NEXT
-> recurring findings
-> causal experiments
-> optimizer candidates.
