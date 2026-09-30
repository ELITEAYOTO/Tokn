# Tokn Open Questions

Status: ACTIVE
Date: 2026-09-30

Only unresolved questions with architectural or analytical impact belong here.

## OQ-001 - Remaining product budget API

Can Tokn retrieve remaining ChatGPT/Codex allowance, credits or reset horizon
programmatically with a stable supported contract?

Status: UNKNOWN

Known:
- /status is officially documented for active Codex CLI sessions;
- Settings/usage dashboard may expose allowance, credits and reset time.

Unknown:
- stable machine-readable interface suitable for Tokn.

Impact: consumption-rate diagnostics only.
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

## OQ-004 - Engine transport

What is the smallest reliable local boundary between Codex integration and Tokn?

Status: RESEARCH

Candidates:
- direct child process / stdio;
- MCP stdio;
- streamable HTTP/local service only if justified.

Known:
OpenAI surfaces support stdio and streamable HTTP MCP in appropriate environments.

Unknown:
which option produces the smallest, most reliable Tokn integration on the
target Codex Desktop/CLI runtime.

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

## OQ-006 - ModelRuntimeProfile minimum schema

Which identifiers and capability flags should be persisted versus derived?

Status: RESEARCH

Observed candidates:
cli_version, model_provider, context_window, model, effort, workspace roots,
permission/config context, parent/fork identity and telemetry capability flags.

Impact: cross-version/multi-model analysis.
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

See:
2026-09-30-CODEX-PLUGIN-TELEMETRY-RESEARCH.md.

## Closure rule

Close an open question only with:
- VERIFIED official source;
- OBSERVED reproducible runtime evidence; or
- ACCEPTED project decision when the question is under Tokn's control.

If evidence conflicts, preserve both provenances and reopen the question.