# Tokn External Research Roadmap

Status: ACTIVE
Date: 2026-09-30

Purpose:
close only external unknowns that materially affect Tokn architecture,
observability or future integrations.

P8/P9 remain unblocked unless research proves a required Runner datum
is unavailable through current evidence sources.

| Priority | Topic | Status | Exit criterion | Blocks P8/P9? |
| --- | --- | --- | --- | --- |
| R1 | Codex telemetry | PARTIAL / OBSERVED | Versioned capability table with provenance | No |
| R2 | Plugin packaging | VERIFIED | Portable + compatibility packaging contract documented | No |
| R3 | Skill/invocation UX | PARTIAL / VERIFIED | Minimal custom-skill invocation test on target runtime | No |
| R4 | Hooks/trust | VERIFIED CORE / PAYLOADS VERSIONED | Hook/trust model documented; payload gaps remain capability-gated | No |
| R5 | MCP/Engine transport | RESEARCH | Prototype direct process/stdio vs MCP before choosing | No |
| R6 | Product quota | PARTIAL / UNKNOWN PROGRAMMATIC | Verified programmatic method or explicit UNKNOWN | No |
| R7 | Model/runtime profiles | PARTIAL / OBSERVED | Minimal profile schema backed by current evidence | No |
| R8 | Privacy/threat model | TODO | Document local-only boundary and plugin/adapter risks | No |

## R1 telemetry - current result

Local codex-cli 0.161.0-alpha.2 / OpenAI.Codex 26.928.1915.0 exposes
standard session JSONL with direct token usage, per-turn/thread usage,
runtime/model/context-window metadata and agent relationship fields.

Compaction is observable in at least one current/compatible session shape.

Still open:
- stable context-occupancy semantics versus cumulative logical usage;
- exact event/payload stability across runtime upgrades;
- whether every model/runtime exposes the same cache-write fields.

## R2 plugin packaging - closed

Official current contract:
- portable root plugin.json;
- skills/;
- portable root mcp.json;
- optional hooks/ and assets/;
- extensions.com.openai for OpenAI-specific metadata/hooks/apps;
- optional .codex-plugin/plugin.json compatibility fallback;
- local/repo marketplaces for authoring/testing.

Local codex-cli 0.161.0-alpha.2 also exposes:
codex plugin add/list/remove and plugin marketplace add/list/upgrade/remove.

## R3 invocation - partial

Verified:
- /plugins opens the Codex plugin browser/install flow;
- skills may activate from normal task wording;
- OpenAI explicitly documents $skill-creator in Codex.

Still open:
- Tokn-specific explicit invocation ergonomics;
- do not freeze /tokn or require $tokn-* before a minimal installed-skill test.

## R4 hooks/trust - core closed

Verified current docs:
- plugin hooks are supported;
- hooks/hooks.json default discovery exists;
- plugin hooks are non-managed and require review/trust;
- PLUGIN_ROOT and PLUGIN_DATA are provided;
- regular Codex hook schema applies;
- current docs include PreCompact among supported lifecycle events.

Payload fields remain versioned evidence rather than universal guarantees.

## R5 transport - next research task

Official OpenAI surfaces support streamable HTTP MCP and stdio MCP in
appropriate execution environments.

Tokn still needs a small prototype to compare:
1. direct child-process/stdio call to Runner;
2. MCP stdio adapter;
3. persistent local service only if the first two are insufficient.

No permanent localhost daemon should be introduced by default.

## R6 quota - partial

Verified human-facing access:
Codex CLI /status and usage dashboards can expose allowance/credits/reset
information when applicable.

Programmatic contract for Tokn:
UNKNOWN.

Do not derive product allowance from API pricing or API rate-limit fields.

## R7 profiles - partial

Current session evidence can capture:
- cli_version;
- model_provider;
- context_window;
- model;
- effort;
- workspace roots;
- parent/fork relationships;
- permission/config context.

Next step:
define the minimal ModelRuntimeProfile schema only from fields Tokn can
actually source with provenance.

## R8 privacy - pending

Review plugin/MCP/hooks data boundaries after R5 chooses the integration shape.

## Research rules

1. Official OpenAI documentation and tagged openai/codex source are preferred.
2. Local runtime observations are OBSERVED, not universal truths.
3. Community/forum claims remain RESEARCH unless independently verified.
4. UNKNOWN is an acceptable result.
5. Research cannot silently change an ADR; durable changes require an ADR/update.
6. Do not create implementation dependencies on unverified plugin features.
7. API semantics and ChatGPT/Codex product semantics remain distinct.

## Immediate order

R5 transport prototype
-> R6 quota programmatic check
-> R7 ModelRuntimeProfile schema
-> R8 privacy/threat model
-> revisit R1 only where those tasks expose a concrete telemetry gap.

Research may run in parallel with P8/P9, but must not distract from them.