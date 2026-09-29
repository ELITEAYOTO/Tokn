# Standard Session Rollout Adapter V0.1

## Purpose

Standard session rollouts become a first-class evidence source, not only an import fallback.

Experiment 001 proved they can recover the complete run when diagnostic tracing is partial.

## Required records

Parser must support at minimum:
- session_meta;
- token_usage_record;
- event_msg/token_count;
- response_item/custom_tool_call;
- response_item/custom_tool_call_output;
- response_item/function_call;
- response_item/function_call_output;
- inter_agent_communication_metadata;
- task_started;
- task_complete;
- thread_settings_applied.

Unknown records are preserved structurally and ignored safely.
## Normalized session facts

SessionIdentity:
- thread_id;
- root session_id;
- parent_thread_id;
- forked_from_id;
- agent path/nickname;
- cwd;
- runtime_workspace_roots;
- model provider;
- cli version.

SessionUsage:
- input;
- cached input;
- cache write;
- output;
- reasoning;
- logical total;
- usage identity.

SessionTerminal:
- turn_id;
- completed_at;
- duration_ms;
- error code;
- error message classification.

SessionTool:
- call_id;
- tool name;
- raw encoded input;
- parsed command when supported;
- requested output cap when recoverable;
- output metadata when recoverable.
## Code Mode exec parsing

Experiment 001 encoded exec calls inside strings shaped like:
text(await tools.exec_command({...}))

V0.1 must parse this adapter conservatively.

Required extraction:
- command text;
- workdir if present;
- max_output_tokens if present;
- timeout if present;
- call identity.

Parser rule:
do not use brittle regex as the only parser.
Prefer a small tokenizer/structured extraction layer with fixtures.

Malformed/unrecognized input remains UNKNOWN and is counted.

## Fixtures required

Add sanitized fixtures for:
- parent Codex Desktop rollout;
- subagent rollout;
- task_complete usage_limit_exceeded;
- exec with max_output_tokens;
- exec without cap;
- nested/escaped PowerShell command;
- unknown future payload.

No real user prompt or proprietary source content in fixtures.
