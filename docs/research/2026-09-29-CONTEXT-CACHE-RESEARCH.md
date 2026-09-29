# Research Review - Context, Cache and Quality-Preserving Token Efficiency

Status: REVIEWED RESEARCH INPUT
Date: 2026-09-29

## Purpose

This document integrates the 2026-09-29 deep-research report with evidence already verified in the Tokn repository
and current official OpenAI documentation.

It is a research document, not a release plan.
Product decisions live in ADRs and ROADMAP.md.

## Evidence policy

Priority order:
1. Tokn run evidence and exact local runtime evidence;
2. official OpenAI documentation and tagged Codex source;
3. external technical research;
4. hypothesis.

Where the deep-research report conflicts with verified repository state, repository evidence wins and the discrepancy is recorded.

## Verified Tokn facts

Experiment 001:
- 4 sessions: 1 parent + 3 subagents;
- 80 usage records;
- input 5,285,737;
- cached input 5,059,712;
- uncached input 226,025;
- output 26,021;
- reasoning 3,018;
- logical total 5,311,758;
- parent about 47.1%;
- descendants about 52.9%;
- terminal INCOMPLETE_USAGE_LIMIT;
- diagnostic source PARTIAL;
- optimization conclusion invalid; instrumentation value retained.

P6:
- policy hint, observed behavior and enforcement are separate concepts;
- real Desktop hooks were trusted/active;
- Codex lifecycle logs showed hook execution around nested Code Mode exec calls;
- tagged runtime source rust-v0.158.0-alpha.2.1 maps unified exec to Bash PreToolUse;
- that callback forwards command but not max_output_tokens;
- enforcement for that cap policy is SUPPORTED_INSUFFICIENT_INPUT;
- absent cap input is UNOBSERVABLE/fail-open.

## Deep-research report: retained conclusions

Useful conclusions retained:
- prioritize passive profiling before active optimization;
- investigate context/cache behavior rather than reducing model capability;
- measure parent/subagent duplication;
- treat compaction as a quality-sensitive intervention;
- keep provenance and quality gates around any optimization.

Ideas retained as candidates, not commitments:
- context engine;
- external/project memory;
- tool-evidence compression;
- command rewriting;
- plugin packaging.

The report proposed changing P7-P9 into profiling/compaction/rewriting phases.
That recommendation is NOT adopted because it conflicts with the existing V0.1 measurement-hardening dependency chain.
P7 validity, P8 runner and P9 release validation remain required before active optimization.

The report also described the empty Tokn JSONL probe as proof of missing hook callbacks.
Local Codex lifecycle logs later proved hooks did execute, so that interpretation is superseded.

## Current OpenAI facts relevant to Tokn

### Prompt caching

Official OpenAI prompt-caching documentation states:
- caching reuses a matching prompt prefix;
- cached input is still input, but may have a lower API price;
- caching does not replay an old answer or reduce output-generation capability;
- changes to model, tools, tool schemas/order, instructions or earlier context can break prefix reuse;
- compaction can reduce cache reuse because it changes earlier context;
- usage exposes cached token counts; newer API surfaces may also expose cache-write accounting.

For GPT-5.6 and later, current API docs describe explicit/implicit cache breakpoints and a 30-minute minimum TTL.
These API controls must NOT be assumed to map one-for-one to ChatGPT Codex Desktop internals.

Reference:
https://developers.openai.com/api/docs/guides/prompt-caching

### Astra API economics are not Codex-plan economics

Current GPT-6 Astra API documentation lists separate rates for:
- uncached input;
- cached input;
- cache writes;
- output.

This is useful to understand economic asymmetry, but Tokn must not apply API dollar pricing directly to ChatGPT/Codex plan quota.
The user's Codex allowance is a product quota with its own accounting.

Reference:
https://developers.openai.com/api/docs/models/gpt-6-astra

### Astra context management

OpenAI states that Astra in Codex can experimentally preserve notes across context windows and search earlier context,
rather than repeatedly compressing all history into a single summary.

Current Codex configuration documentation exposes:
features.context_management.experimental_mode

This is highly relevant to Tokn, but Tokn should first OBSERVE whether the feature is active and how runs behave.
It should not enable/disable the feature automatically during measurement.

References:
https://openai.com/index/gpt-6-astra/
https://learn.chatgpt.com/docs/config-file/config-reference

### AGENTS.md and skills can create context overhead

OpenAI's September 2026 guidance recommends revisiting old instructions for Astra.
It specifically warns against forcing the model to read a stack of documents before every edit
and recommends contextual instructions rather than blanket mandatory reads.

This supports a future Tokn Instruction Footprint finding:
measure recurring instruction-driven reads before proposing any change.

Reference:
https://developers.openai.com/blog/rethinking-skills-and-prompts-for-gpt-6-astra

## Experiment 001 cache interpretation

cached/input = 5,059,712 / 5,285,737 = about 95.72%.

Safe conclusion:
the rollout classified most input tokens as cached.

Unsafe conclusions:
- "95.72% of context was useful";
- "the run was already optimal";
- "cached tokens cost nothing";
- "cached tokens do not consume Codex quota";
- "the remaining 4.28% is the only waste";
- "we should shrink the context";
- "subagents are inefficient because they used 52.9%".

A high cache ratio can coexist with a very large logical context.
It may indicate good prefix reuse while still carrying redundant or stale material.
Only additional attribution can separate useful reuse from repeated baggage.

## Questions Tokn must answer next

Per turn / agent:
- where does uncached input grow?
- what happened immediately before the growth?
- did tool definitions/instructions/runtime config change?
- did a compaction/context-management event occur?
- was the same file/search evidence already available?
- did another agent already acquire equivalent evidence?

Across runs:
- which project facts are repeatedly rediscovered?
- which instruction blocks trigger repeated work?
- which context remains stable and cache-friendly?
- which context changes frequently?
- which repeated evidence correlates with useful changes versus rework?

## Measurement backlog

Priority A - observable from existing rollouts:
- cached ratio by session/agent/turn;
- uncached delta timeline;
- repeated file reads;
- overlapping file reads;
- repeated searches;
- repeated commands/retries;
- parent/subagent shared evidence;
- tool category frequency;
- terminal/quality correlation.

Priority B - requires adapter/runtime investigation:
- compaction/context-management events;
- exact model-visible tool-output size;
- stable instruction/tool-schema fingerprints;
- cache-break diagnostics or equivalent evidence;
- Codex context-note/search events;
- cache-write accounting if exposed.

Priority C - active experiments only:
- instruction cleanup;
- context compiler;
- project memory;
- evidence compression;
- command rewriting;
- explicit cache-aware prompt construction.

## Recommended experiment sequence

Experiment 002:
instrumentation validation only.
Prove that one complete real run is captured end-to-end by V0.1.

Observation campaign:
collect several normal runs without changing Astra behavior.
Build historical/context findings and measure recurrence.

Experiment 003:
first causal A/B.
Select ONE low-risk, high-evidence context finding.
Freeze task/workspace/runtime as far as practical.
Quality gate before token comparison.

Possible first candidates, only if evidence supports them:
- remove one proven redundant mandatory instruction/read;
- reuse one stable project fact instead of rediscovering it;
- avoid one repeated equivalent search/read pattern.

Do not choose a candidate merely because it has the largest token count.

## Research conclusion

The strongest Tokn opportunity is not "make Astra speak less".
It is to make invisible context behavior measurable:
what is cached, what becomes uncached, what is duplicated, what is rediscovered,
what is carried across agents, and what actually contributes to useful work.

That measurement layer is the prerequisite for any safe Context Compiler or Project Memory.
