# Tokn Open Questions

Status: ACTIVE
Date: 2026-09-30

Only unresolved questions with architectural or analytical impact belong here.

## OQ-001 - Rate-limit telemetry semantics

How should Tokn interpret multiple Codex rate-limit windows without collapsing
them into a false single "budget remaining" metric?

Status: OBSERVED LOCAL / SEMANTICS VERSIONED

Known:
- /status is officially documented for active Codex CLI sessions;
- local session token_count events expose structured rate_limits;
- primary/secondary windows expose used_percent, window_minutes and resets_at;
- current openai/codex source defines these as rate-limit snapshots/windows;
- credits and spend-control shapes may also be present.

Unknown:
- stable meaning of every limit_id across runtime/product versions;
- whether a given profile exposes all possible windows/credits;
- whether a single product-level allowance metric is meaningful at all.

Rule:
store each limit/window with identity, provenance and runtime version.
Do not derive one universal Astra budget unless evidence later supports it.

Impact: consumption-rate and exhaustion-risk diagnostics.
Blocks P8/P9: no.

## OQ-003 - Tokn invocation UX

What explicit invocation should Tokn use after installation?

Status: RESEARCH

Known:
- /plugins is a verified plugin-browser/install flow;
- ordinary task wording can activate skills;
- $skill-creator is officially documented in Codex.

Unknown:
- whether Tokn should expose one or more explicit $tokn-* skills;
- whether any custom slash-command namespace is appropriate.

Rule:
do not design around /tokn unless officially supported and locally verified.

Impact: UX only.
Blocks P8/P9: no.

## OQ-004 - Engine transport prototype

Can the preferred process-bound MCP adapter satisfy Tokn integration needs
without introducing a permanent local service?

Status: PREFERRED PROTOTYPE / NOT YET VALIDATED

Known:
- OpenAI surfaces support stdio and streamable HTTP MCP in appropriate environments;
- current local Codex compatibility plugins launch MCP processes with
  command/args/cwd declarations;
- installed OpenAI plugin manifests reference those .mcp.json files;
- P8 remains a standalone Runner/Engine boundary.

Preferred prototype:
Codex plugin -> local command-launched Tokn MCP adapter -> shared Rust Engine.

Unknown:
- actual startup/shutdown behavior of a Tokn adapter;
- error/approval ergonomics;
- whether a later Desktop UI creates a real need for a long-lived service.

Impact: integration complexity, lifecycle and security.
Blocks P8/P9: no.

## OQ-005 - Context occupancy semantics

Which observable signal should Tokn treat as current context occupancy rather
than cumulative logical usage?

Status: UNKNOWN

Known:
- local session metadata exposes context_window;
- local token usage exposes cumulative/response usage dimensions;
- compacted records are observable in at least one session shape;
- API compaction and token accounting are documented separately.

Unknown:
- stable Codex Desktop field/event that directly represents the effective
  retained context occupancy after tools and compaction;
- whether this field is consistent across models/runtime versions.

Impact: future Context Ledger and compaction findings.
Blocks P8/P9: no.

## OQ-007 - Hook payload/version drift

How should Tokn version hook adapters when event names or payload fields change?

Status: HYPOTHESIS

Known:
the current official hook model and trust lifecycle are verified.
Historical P6 evidence already shows a field required by Tokn can be absent
from a supported hook event.

Rule:
hook support and field support are separate capabilities.

Impact: adapter compatibility.
Blocks P8/P9: no.

## Recently closed

### OQ-002 - Current plugin packaging

Closed: VERIFIED on 2026-09-30.

Current official packaging supports root plugin.json, skills/, root mcp.json,
optional hooks/assets, extensions.com.openai, and optional
.codex-plugin/plugin.json compatibility fallback.

### OQ-006 - ModelRuntimeProfile minimum schema

Closed: ACCEPTED DESIGN on 2026-09-30.

Minimum identity/configuration/capability fields and privacy exclusions are
defined in ../design/MODEL-RUNTIME-PROFILE.md.

Implementation remains future work after the V0.1 measurement gate.

See:
2026-09-30-CODEX-PLUGIN-TELEMETRY-RESEARCH.md.

## Closure rule

Close an open question only with:
- VERIFIED official source;
- OBSERVED reproducible runtime evidence; or
- ACCEPTED project decision when the question is under Tokn's control.

If evidence conflicts, preserve both provenances and reopen the question.