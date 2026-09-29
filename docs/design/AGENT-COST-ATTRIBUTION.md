# Parent and Subagent Cost Attribution

## Why this is now P0

Experiment 001 aggregate logical tokens:
5,311,758.

Parent:
2,499,523 (~47.1%).

Three subagents combined:
2,812,235 (~52.9%).

Ignoring subagents would miss the majority of measured logical token cost.

## Run graph

RunGroup
- parent thread
  - budget_audit
  - fix_audit
  - ui_workflow
  - future nested agents...

Each node records:
- own token usage;
- own tool calls;
- own duration;
- own terminal status;
- spawn relationship;
- overlap interval;
- inherited/forked context metadata when observable.
## Metrics

Per node:
- input;
- cached;
- uncached;
- output;
- reasoning;
- logical total;
- cache share;
- usage records;
- tool calls;
- read/search calls;
- repeated resource accesses;
- terminal status.

Per RunGroup:
- aggregate tokens;
- parent share;
- descendant share;
- agent count;
- max depth;
- fan-out;
- concurrent overlap;
- completed vs failed/limited agents.

## Optimization questions V0.2+

Do not automatically conclude subagents are waste.

Measure:
- duplicated file reads across agents;
- duplicated searches;
- repeated repo-map/context loading;
- whether an agent result replaces more expensive parent work;
- whether parallel agents improve completed work before quota exhaustion.

Optimization target:
reduce duplicate context and redundant tool output,
not blindly reduce agent count.
