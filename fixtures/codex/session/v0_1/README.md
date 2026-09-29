# Tokn V0.1 Session Fixtures

Status: SYNTHETIC / SANITIZED
Created: 2026-09-27

These fixtures preserve only structural shapes observed during Experiment 001.

They do NOT contain:
- user prompts;
- JEM source code;
- real user paths;
- real session/thread UUIDs;
- email/account identifiers;
- real tool output.

All values are synthetic.

## Files

parent/rollout.jsonl
- SHA-256: 626fa57f9c7606e92e6096471d900956607f0aed46f4f4b9ebbe5892f08ee9c9
- bytes: 2258
- shapes: parent session_meta, task_started, token_usage_record, exec tool call, tool output, usage_limit_exceeded terminal.

subagent/rollout.jsonl
- SHA-256: 32b1cfc644526b15a6571863ee87124226907b359f2d0481d26fd8c21fe8400f
- bytes: 1893
- shapes: source.subagent.thread_spawn, parent_thread_id, forked_from_id, thread_settings_applied, token usage, usage_limit_exceeded terminal.

tool-caps/rollout.jsonl
- SHA-256: 496a57a8c3dd5a6b14b7428ebbce5759d83b03ecf908cc34b4d194731b257743
- bytes: 619
- shapes: exec call with max_output_tokens and exec call without a cap.

## Diagnostic companion fixture

../..\diagnostic\partial-protocol-only\manifest.json
- SHA-256: 3e5d3462fce5d79a2acbcdfa6109c3e84fb964ce3aecd1acb4c564c0366469e0

../..\diagnostic\partial-protocol-only\trace.jsonl
- SHA-256: 86a4c6facbfdd409a81b8a71156d6dfbe633529c04238b0709f6e37ee1ec3920
- bytes: 435
- exactly 4 bookkeeping/protocol events and no inference/tool usage surface.

## Privacy gate

The Rust test v0_1_fixtures_are_sanitized scans these fixtures and fails if known private/project markers appear.

Any future fixture derived from real evidence must be sanitized before entering this directory.
