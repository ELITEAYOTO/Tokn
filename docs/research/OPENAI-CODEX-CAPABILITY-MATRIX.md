# OpenAI / Codex Capability Matrix

Status: RESEARCH
Last reviewed: 2026-09-30

This file separates project observations from external capabilities that still
require current official verification.

Status vocabulary:
ACCEPTED / VERIFIED / OBSERVED / RESEARCH / HYPOTHESIS / UNKNOWN / SUPERSEDED.

| Capability | Status | Scope | Evidence / note |
| --- | --- | --- | --- |
| Codex hook lifecycle executed on the tested Desktop runtime | OBSERVED | Codex Desktop 0.158.0-alpha.2.1 | Local lifecycle logs showed hook/started and hook/completed |
| PreToolUse for unified exec is mapped to Bash | OBSERVED | Tagged/local P6 investigation | Runtime/source investigation used by P6 |
| PreToolUse Bash exposes nested max_output_tokens | OBSERVED: NO | Codex Desktop 0.158.0-alpha.2.1 | P6 found command but not max_output_tokens |
| Remaining product quota/budget is programmatically exposed | UNKNOWN | Codex/ChatGPT product | Do not infer from API pricing/rate-limit docs |
| Cache-write telemetry is available to current Codex Desktop | UNKNOWN | Current target runtime | Requires current official/runtime verification |
| Compaction/context-management events are directly observable | UNKNOWN | Current target runtime | Requires current official/runtime verification |
| Exact custom skill invocation syntax | UNKNOWN | Current Codex Desktop/CLI | Do not freeze $skill or /tokn syntax yet |
| Plugin bundle/manifest contract | RESEARCH | Current Codex plugin system | Needs fresh official verification before implementation |
| Plugin-installed hooks and trust/approval behavior | RESEARCH | Current Codex plugin system | Needs fresh official verification before implementation |
| MCP can call Tokn Engine directly without a service wrapper | UNKNOWN | Future integration | Requires prototype and official transport review |
| Runtime/model version can be captured reliably per run | PARTIAL / OBSERVED | Existing Tokn evidence | Runtime/model metadata already exists, capability completeness not frozen |
| Parent/subagent identity can be reconstructed | OBSERVED | V0.1 P2 | Golden Experiment 001: 1 parent + 3 descendants |
| Token usage can be attributed across RunGroup | OBSERVED | V0.1 | Golden replay and reducers |
| Actual output workspace can be resolved dynamically | OBSERVED | V0.1 P5 | Golden Experiment 001 selects B07-C |
| Experiment causal validity is machine-reduced | ACCEPTED / OBSERVED | Tokn P7 | ADR-004 + passing fixtures/golden replay |

## Interpretation rule

A capability marked OBSERVED is limited to the stated runtime/evidence.
It must not be generalized to later Codex versions or other models.

A capability marked UNKNOWN must stay UNKNOWN until a source or test resolves it.

API token economics, cached-token pricing and ChatGPT/Codex product quota
are separate concerns and must not be conflated.
