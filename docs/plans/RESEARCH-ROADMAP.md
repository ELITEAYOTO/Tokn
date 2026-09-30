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
| R5 | MCP/Engine transport | PREFERRED LOCAL PROTOTYPE / UNVALIDATED | Process-bound MCP adapter test after P8/P9 | No |
| R6 | Rate limits / product allowance | OBSERVED LOCAL / SEMANTICS VERSIONED | Preserve limit identities/windows; avoid single-budget assumption | No |
| R7 | Model/runtime profiles | DESIGN DEFINED / IMPLEMENTATION PENDING | Implement accepted minimum schema after V0.1 | No |
| R8 | Privacy/threat model | LOCAL BOUNDARY DOCUMENTED | Revisit only for public/remote deployment | No |

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

## R5 transport - preferred local prototype

Official OpenAI surfaces support streamable HTTP MCP and stdio MCP in
appropriate execution environments.

Local Codex 0.161.0-alpha.2 also provides direct evidence:
OpenAI-installed compatibility plugins launch MCP servers with
command/args/cwd declarations from .mcp.json.

Preferred Tokn prototype after P8/P9:
Codex plugin -> process-bound Tokn MCP adapter -> shared Rust Engine.

A persistent local service is deferred unless multi-client/background
requirements later justify it.

## R6 rate limits - observed local telemetry

Verified/observed:
- Codex CLI /status and usage dashboards can expose allowance/reset information;
- local token_count events expose structured rate_limits;
- primary/secondary windows carry used_percent, window_minutes and resets_at;
- current openai/codex source defines the corresponding rate-limit snapshot;
- credits/spend-control fields may also exist.

Tokn can ingest these fields programmatically from local evidence.

Remaining rule:
do not collapse multiple limit identities/windows into one universal
"Astra budget remaining" value unless later evidence supports that mapping.

Do not derive product allowance from API pricing.

## R7 profiles - design defined

The accepted minimum schema now lives in:
`../design/MODEL-RUNTIME-PROFILE.md`.

It covers:
- runtime/model identity;
- material configuration;
- versioned capability observations;
- rate-limit snapshots;
- provenance;
- privacy exclusions.

Implementation waits until after the V0.1 measurement gate.

## R8 privacy - local boundary documented

Tokn remains local-first.

The privacy reference now defines plugin/MCP constraints:
least privilege, secret exclusion, minimal structured results,
no unnecessary account/user IDs, path minimization and explicit separation
between local Tokn integration and future public/remote distribution.

Reopen this topic when a public/remote plugin becomes a concrete goal.

## Research rules

1. Official OpenAI documentation and tagged openai/codex source are preferred.
2. Local runtime observations are OBSERVED, not universal truths.
3. Community/forum claims remain RESEARCH unless independently verified.
4. UNKNOWN is an acceptable result.
5. Research cannot silently change an ADR; durable changes require an ADR/update.
6. Do not create implementation dependencies on unverified plugin features.
7. API semantics and ChatGPT/Codex product semantics remain distinct.

## Immediate order

The current blocking research pass is sufficiently complete for V0.1.

Next product work remains P8 -> P9.

Research after that:
- prototype the process-bound MCP adapter;
- validate ModelRuntimeProfile ingestion on real V0.1 evidence;
- refine rate-limit semantics per runtime/limit_id;
- run the Tokn-specific skill invocation test;
- revisit public/remote privacy only if distribution requires it.

Research remains non-blocking for P8/P9.