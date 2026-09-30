# 2026-09-30 Codex Plugin and Telemetry Research

Status: REVIEWED RESEARCH INPUT
Scope: R1-R6 first pass
Date: 2026-09-30

## Purpose

Resolve current external capabilities with official OpenAI sources and local runtime evidence.
This document does not change Tokn architecture by itself.

Evidence labels:
- VERIFIED = current official OpenAI documentation.
- OBSERVED = reproduced on the local Tokn development machine.
- UNKNOWN = not established strongly enough for Tokn to depend on it.

## R1 - Codex telemetry

### Local runtime observation

OBSERVED on 2026-09-30:
- codex-cli: 0.161.0-alpha.2
- OpenAI.Codex app: 26.928.1915.0

Recent standard session JSONL files expose:
- top-level token_usage_record records;
- usage fields: input_tokens, cached_input_tokens, cache_write_input_tokens,
  output_tokens, reasoning_output_tokens, total_tokens;
- per-response usage plus turn_token_usage and thread_token_usage;
- session metadata including cli_version, context_window, model_provider,
  parent_thread_id, forked_from_id and runtime_workspace_roots;
- turn context including model, effort, permission profile, workspace roots
  and turn/root-turn identifiers;
- top-level compacted records exist in at least one observed session shape;
- parent/subagent relationship evidence remains directly observable.

These are local runtime observations, not a promise that every Codex version
or every session shape exposes all fields.

### Official external semantics

VERIFIED for OpenAI Agents API observability:
- usage is best-effort and may be null when unknown;
- missing usage does not mean zero;
- cached tokens are included in input tokens;
- reasoning tokens are included in output tokens;
- subagent turns have their own usage and subagent_id;
- cache-write usage may not be separately exposed by that API surface.

This supports Tokn's UNKNOWN and RunGroup rules but does not make Agents API
fields equivalent to Codex Desktop session JSONL.

Official source:
https://developers.openai.com/api/docs/guides/agents-api/observability

VERIFIED for Responses/API prompt caching:
cache performance can expose cached token counts and, on applicable surfaces,
cache-write token counts.

Official source:
https://developers.openai.com/api/docs/guides/prompt-caching

## R2 - Plugin packaging

VERIFIED current portable plugin structure:
- root plugin.json is the portable Agent Plugins entry point;
- skills live under skills/;
- portable MCP configuration lives in root mcp.json;
- optional hooks/ and assets/ are supported;
- extensions.com.openai carries OpenAI-specific presentation, app mappings
  and hook configuration;
- .codex-plugin/plugin.json remains a compatibility fallback.

The built-in plugin creator currently scaffolds the compatibility layout
(.codex-plugin/plugin.json, optional .mcp.json/.app.json/skills/hooks/scripts/assets).
That scaffold remains supported, but new portable packages should use root
plugin.json and mcp.json.

VERIFIED local/repo marketplace support:
- repo marketplace: $REPO_ROOT/.agents/plugins/marketplace.json;
- personal marketplace: ~/.agents/plugins/marketplace.json;
- codex plugin marketplace add/list/upgrade/remove are documented.

OBSERVED locally on codex-cli 0.161.0-alpha.2:
- codex plugin add
- codex plugin list
- codex plugin remove
- codex plugin marketplace add/list/upgrade/remove
are present in CLI help.
The local CLI currently sees openai-primary-runtime, openai-bundled and
openai-curated marketplace sources.

Official source:
https://developers.openai.com/plugins/build/plugins

## R3 - Skill and plugin invocation UX

VERIFIED:
- Codex has a /plugins flow for opening the plugin browser and installing plugins;
- skills may activate from ordinary task wording;
- OpenAI documents explicit $skill-creator invocation in Codex;
- skill descriptions are the primary signal for automatic activation.

NOT YET FROZEN:
- Tokn will not assume a custom /tokn command namespace;
- Tokn will not require explicit $tokn-* syntax until a minimal installed
  custom-skill test confirms the desired UX on the target runtime.

Official sources:
https://developers.openai.com/learn/developers-codex-plugin
https://developers.openai.com/plugins/build/skills
https://developers.openai.com/blog/rethinking-skills-and-prompts-for-gpt-6-astra

## R4 - Hooks and trust

VERIFIED current plugin hook model:
- enabled plugins can contribute lifecycle hooks;
- hooks/hooks.json is discovered by default when not overridden;
- plugin hooks use the regular Codex hook event schema;
- plugin hook commands receive PLUGIN_ROOT and PLUGIN_DATA;
- plugin-installed hooks are non-managed hooks;
- installing/enabling a plugin does not automatically trust its hooks;
- untrusted plugin hooks are skipped until reviewed/trusted.

Current hook documentation includes lifecycle events such as SessionStart,
PreToolUse, PostToolUse, PreCompact, SubagentStart and Stop.
Therefore the earlier assumption that there is no first-class PreCompact hook
is SUPERSEDED for current documentation.

This does not guarantee that every event payload exposes every Tokn field.
Payload-level capability remains versioned evidence.

Official sources:
https://developers.openai.com/plugins/build/plugins
https://developers.openai.com/docs/hooks

## R5 - MCP / Engine transport

VERIFIED:
- portable plugin MCP configuration supports streamable HTTP;
- OpenAI's plugin/Agents surfaces also support stdio MCP when the server
  executable runs in the session environment;
- public plugin submission expects a remote HTTPS MCP endpoint;
- local development can use local/tunneled MCP workflows depending on surface.

For Tokn, this does NOT yet choose the Engine boundary.
A direct process/stdio adapter may be simpler than a permanent localhost service,
but that remains a prototype decision.

Official sources:
https://developers.openai.com/plugins/build/plugins
https://developers.openai.com/api/docs/guides/agents-api/tools/plugins
https://developers.openai.com/api/docs/guides/agents-api/tools/mcp

## R6 - Product quota / remaining budget

VERIFIED user-facing behavior:
OpenAI Help documents that an active Codex CLI session can use /status and that
Settings/usage dashboard may show the exhausted allowance, credit balance
and reset time when applicable.

UNKNOWN:
there is no verified programmatic contract yet that Tokn can depend on for
remaining ChatGPT/Codex product allowance or time-to-exhaustion.

Do not derive product quota from API token pricing or API rate limits.

Official source:
https://help.openai.com/en/articles/11369540-using-codex-with-your-chatgpt-plan

## Architectural effect

No result in this pass blocks P8 or P9.

R2 is sufficiently verified to stop treating plugin packaging as an unknown.
R4 is sufficiently verified to design a future hook adapter, while individual
payload fields remain capability-gated.
R1 now has a stronger local telemetry baseline.
R3, R5 and R6 remain partially open where Tokn-specific behavior matters.

The accepted architecture remains:
Rust Engine = source of truth;
plugin/skills/MCP/hooks = integration surfaces;
Optimizer = later, after causal evidence.
