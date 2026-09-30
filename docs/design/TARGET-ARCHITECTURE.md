# Tokn Target Architecture

Status: ACCEPTED DESIGN DIRECTION
Date: 2026-09-30
Current state: P0-P7 DONE / P8 NEXT

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

The future local Store persists runs, profiles, experiments, findings and history.
Its schema should follow the stable Runner boundary rather than precede it.

### Codex integration

Codex integration is a thin activation/observation/presentation layer.
Its exact primitives and transport remain RESEARCH until officially verified
for the target runtime/version.

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

Plugin-to-Engine transport is RESEARCH.

Candidates may include MCP, stdio/process invocation, IPC or a local service.
No permanent localhost service is required unless a measured need justifies it.

Selection criteria:
simplicity, local-first behavior, security, observability, portability,
versionability and low operational cost.

## Sequence after V0.1

P8 Runner
-> P9 golden/release validation
-> Experiment 002 instrumentation validation
-> Store + ModelRuntimeProfiles
-> Codex integration
-> historical/context analyzers
-> recurring findings
-> causal experiments
-> optimizer candidates.
