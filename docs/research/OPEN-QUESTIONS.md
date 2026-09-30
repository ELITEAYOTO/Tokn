# Tokn Open Questions

Status: ACTIVE
Date: 2026-09-30

Only unresolved questions with architectural or analytical impact belong here.

## OQ-001 - Remaining product budget

Can Codex/ChatGPT expose remaining product usage or time-to-exhaustion
programmatically with enough reliability for Tokn?

Status: UNKNOWN
Impact: consumption-rate diagnostics only.
Blocks P8/P9: no.

## OQ-002 - Current plugin packaging

What is the exact current plugin bundle/manifest/install contract across
Codex Desktop and CLI?

Status: RESEARCH
Impact: future integration packaging.
Blocks P8/P9: no.

## OQ-003 - Invocation UX

What are the officially supported invocation paths for custom skills/plugin tools?
Do not assume `/tokn` or a specific `$skill` form before verification.

Status: UNKNOWN
Impact: UX only.
Blocks P8/P9: no.

## OQ-004 - Engine transport

What is the smallest reliable local boundary between Codex integration and Tokn:
process/stdio, MCP, IPC, or a local service?

Status: HYPOTHESIS
Impact: integration complexity and security.
Blocks P8/P9: no.

## OQ-005 - Context/cache telemetry

Which context occupancy, cache read/write, compaction and context-management
signals are observable in the current runtime?

Status: UNKNOWN
Impact: future Context Ledger and Cache Efficiency Analyzer.
Blocks P8/P9: no.

## OQ-006 - Model/runtime capability registry

Which identifiers and capability flags can be captured directly versus inferred?

Status: RESEARCH
Impact: cross-version/multi-model analysis.
Blocks P8/P9: no.

## OQ-007 - Hook payload drift

Do hook event names/payloads change across Codex versions, and how should Tokn
version adapters without silently reclassifying missing fields?

Status: HYPOTHESIS
Impact: adapter compatibility.
Blocks P8/P9: no.

## Closure rule

Close an open question only with:
- VERIFIED official source;
- OBSERVED reproducible runtime evidence; or
- ACCEPTED project decision when the question is under Tokn's control.

If evidence conflicts, preserve both provenances and reopen the question.
