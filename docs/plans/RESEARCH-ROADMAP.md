# Tokn External Research Roadmap

Status: ACTIVE
Date: 2026-09-30

Purpose:
close only external unknowns that materially affect Tokn architecture,
observability or future integrations.

P8/P9 remain unblocked unless research proves a required Runner datum
is unavailable through current evidence sources.

| Priority | Topic | Question | Preferred source | Exit criterion | Blocks P8/P9? |
| --- | --- | --- | --- | --- | --- |
| R1 | Codex telemetry | Which usage/context/cache/compaction/agent/tool fields are actually observable? | Official OpenAI docs + tagged openai/codex source + local evidence | Versioned capability table with provenance | No |
| R2 | Plugin packaging | What plugin bundle, manifest, install/import and lifecycle are officially supported now? | Official OpenAI plugin/Codex docs | VERIFIED packaging contract | No |
| R3 | Skill/invocation UX | How are custom skills/tools invoked in Codex Desktop/CLI today? | Official docs + minimal local test | VERIFIED invocation paths; no assumed slash syntax | No |
| R4 | Hooks/trust | Which hook events/payloads/trust rules exist for the target runtime? | Official docs + tagged source + local probe | Event/payload matrix with gaps marked UNKNOWN | No |
| R5 | MCP/Engine transport | Should Tokn use direct process/stdio, MCP or a service wrapper? | Official MCP docs + prototype | One minimal transport chosen from measured needs | No |
| R6 | Product quota | Is remaining Codex/Astra product budget observable programmatically? | Official product docs + local evidence | VERIFIED method or explicit UNKNOWN | No |
| R7 | Model/runtime profiles | Which runtime/model identifiers and capabilities can be captured reliably? | Official docs + local evidence | Minimal profile schema backed by evidence | No |
| R8 | Privacy/threat model | What data can adapters/plugins see, persist or transmit? | Official security/docs + Tokn privacy rules | Documented local-only boundary and risks | No |

## Research rules

1. Official OpenAI documentation and tagged openai/codex source are preferred.
2. Local runtime observations are OBSERVED, not universal truths.
3. Community/forum claims remain RESEARCH unless independently verified.
4. UNKNOWN is an acceptable result.
5. Research cannot silently change an ADR; durable changes require an ADR/update.
6. Do not create implementation dependencies on unverified plugin features.

## Immediate order

R1 telemetry
-> R2 packaging
-> R3 invocation
-> R4 hooks/trust
-> R5 transport
-> R6 quota
-> R7 profiles
-> R8 privacy review.

Research may run in parallel with P8/P9, but must not distract from them.
