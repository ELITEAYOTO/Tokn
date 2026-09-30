# OpenAI / Codex Capability Matrix

Status: ACTIVE RESEARCH REFERENCE
Last reviewed: 2026-09-30

Status vocabulary:
ACCEPTED / VERIFIED / OBSERVED / RESEARCH / HYPOTHESIS / UNKNOWN / SUPERSEDED.

| Capability | Status | Scope | Evidence / note |
| --- | --- | --- | --- |
| Current local Codex CLI version | OBSERVED | Development PC | codex-cli 0.161.0-alpha.2 |
| Current local Codex app version | OBSERVED | Development PC | OpenAI.Codex 26.928.1915.0 |
| Standard session token_usage_record exists | OBSERVED | Recent local sessions | Direct JSONL inspection |
| Local usage exposes input/cached/cache-write/output/reasoning/total | OBSERVED | Recent local sessions | Direct JSONL inspection |
| Local per-turn and per-thread usage exists | OBSERVED | Recent local sessions | turn_token_usage + thread_token_usage |
| Local context_window metadata exists | OBSERVED | Recent local sessions | session_meta.context_window |
| Local model + effort metadata exists | OBSERVED | Recent local sessions | turn_context.model / effort |
| Parent/fork agent relationship metadata exists | OBSERVED | Recent local sessions | parent_thread_id / forked_from_id / inter-agent metadata |
| Top-level compacted record exists in compatible session shape | OBSERVED | Local session history | Seen in 2026-09-09 session |
| Agents API usage may be null/unknown | VERIFIED | Agents API | Official observability docs |
| Cached tokens are included in input tokens | VERIFIED | Agents API/API usage semantics | Official observability docs |
| Reasoning tokens are included in output tokens | VERIFIED | Agents API/API usage semantics | Official observability docs |
| Agents API turn usage can identify subagents | VERIFIED | Agents API | subagent_id documented |
| Portable plugin root plugin.json | VERIFIED | Current plugin system | Official plugin packaging docs |
| Portable skills/ directory | VERIFIED | Current plugin system | Official plugin packaging/skills docs |
| Portable root mcp.json | VERIFIED | Current plugin system | Official plugin packaging docs |
| .codex-plugin/plugin.json compatibility fallback | VERIFIED | Current plugin system | Official plugin packaging docs |
| Local/repo plugin marketplaces | VERIFIED | Codex local clients | Official docs + local CLI |
| codex plugin add/list/remove | OBSERVED | codex-cli 0.161.0-alpha.2 | Local CLI help |
| codex plugin marketplace add/list/upgrade/remove | VERIFIED + OBSERVED | Current docs/local CLI | Official docs + local CLI help |
| Codex /plugins plugin-browser flow | VERIFIED | Codex | Official OpenAI Developers plugin docs |
| Skills can activate from ordinary task wording | VERIFIED | Current plugin skills | Official skill testing docs |
| $skill-creator invocation | VERIFIED | Codex | Official skill docs |
| Arbitrary /tokn command namespace | UNKNOWN | Future Tokn UX | Do not assume support |
| Custom Tokn skill explicit syntax | RESEARCH | Future Tokn UX | Minimal installed-skill test still needed |
| Plugin hooks supported | VERIFIED | Current plugin system | Official plugin architecture/docs |
| hooks/hooks.json default discovery | VERIFIED | Current plugin system | Official packaging docs |
| Plugin hooks require review/trust | VERIFIED | Codex | Non-managed hooks are skipped until trusted |
| PLUGIN_ROOT / PLUGIN_DATA available | VERIFIED | Plugin hook runtime | Official packaging docs |
| PreCompact hook exists in current docs | VERIFIED | Current hook docs | Supersedes older no-PreCompact assumption |
| PreToolUse Bash exposes nested max_output_tokens | OBSERVED: NO | Codex Desktop 0.158.0-alpha.2.1 | P6 found command but not max_output_tokens |
| Remaining product quota visible to user via /status/dashboard | VERIFIED | Codex product | OpenAI Help Center |
| Local session rate_limits telemetry exists | OBSERVED | codex-cli 0.161.0-alpha.2 | token_count payload exposes versioned limit windows |
| Rate-limit window semantics exist in openai/codex source | VERIFIED SOURCE | Current openai/codex | used_percent, window duration, reset timestamp, credits/spend-control shapes |
| One universal remaining product budget field | UNKNOWN | Codex/ChatGPT product | Multiple limit identities/windows can coexist |
| Streamable HTTP MCP supported | VERIFIED | Plugin/MCP surfaces | Official plugin MCP docs |
| Stdio MCP supported in OpenAI agent execution environments | VERIFIED | Agents/plugin API surfaces | Official MCP docs |
| Command-launched local MCP in Codex compatibility plugins | OBSERVED | codex-cli 0.161.0-alpha.2 | OpenAI-installed .mcp.json files use command/args/cwd |
| Best Tokn Plugin-to-Engine transport | PREFERRED / UNVALIDATED | Future local integration | Process-bound MCP adapter is preferred prototype after P8/P9 |
| Parent/subagent identity can be reconstructed by Tokn | OBSERVED | V0.1 P2 | Golden Experiment 001 |
| Token usage can be attributed across RunGroup | OBSERVED | V0.1 | Golden replay and reducers |
| Actual output workspace can be resolved dynamically | OBSERVED | V0.1 P5 | Golden Experiment 001 |
| Experiment causal validity is machine-reduced | ACCEPTED / OBSERVED | Tokn P7 | ADR-004 + fixtures/golden replay |
| ModelRuntimeProfile minimum schema | ACCEPTED DESIGN | Tokn future analyzer | docs/design/MODEL-RUNTIME-PROFILE.md |
| Local-first plugin privacy boundary | ACCEPTED | Tokn integration | docs/reference/PRIVACY.md |

## Source anchors

Official OpenAI:
- https://developers.openai.com/plugins/build/plugins
- https://developers.openai.com/plugins/build/skills
- https://developers.openai.com/plugins/concepts/plugins
- https://developers.openai.com/docs/hooks
- https://developers.openai.com/api/docs/guides/agents-api/observability
- https://developers.openai.com/api/docs/guides/agents-api/tools/mcp
- https://help.openai.com/en/articles/11369540-using-codex-with-your-chatgpt-plan
- https://developers.openai.com/plugins/guides/security-privacy
- https://github.com/openai/codex/blob/main/codex-rs/codex-api/src/rate_limits.rs
- https://github.com/openai/codex/blob/main/codex-rs/protocol/src/protocol.rs

Local evidence:
development PC standard session JSONL + local codex CLI help on 2026-09-30.

## Interpretation rule

VERIFIED is scoped to the named official surface.
OBSERVED is scoped to the named runtime/evidence.
Neither may be generalized silently to another runtime, model or product surface.

UNKNOWN must stay UNKNOWN until a source or test resolves it.

API token economics, cached-token pricing, context occupancy and
ChatGPT/Codex product allowance are separate metrics.