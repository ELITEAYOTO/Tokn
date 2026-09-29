# Evidence, Coverage and Unknown Semantics

## Core rule

Absence of evidence is not evidence of compliance, zero cost, or success.

Tokn uses four result states for assertions:

- PASS: sufficient evidence supports the assertion.
- FAIL: sufficient evidence disproves the assertion.
- NO_EVIDENCE: no relevant observation exists.
- INCOMPLETE_EVIDENCE: some relevant observations exist but coverage is insufficient.

UNKNOWN remains a data value, not a synonym for zero.

## Coverage gate

A policy check with:
targeted_tools = 0

must return:
NO_EVIDENCE

It may not return PASS.

A policy check with:
targeted_tools > 0 and unknown_caps > 0

returns:
INCOMPLETE_EVIDENCE

unless an explicit experiment rule allows partial coverage.
## Diagnostic trace health gate

Expected for an active Astra development run:
- thread/session activity;
- inference or response activity;
- tool activity when tools were used;
- usage records when provider exposes them.

A trace with only rollout/thread/protocol bookkeeping is PARTIAL.

Health check fields:
- record_count;
- inference_count;
- usage_count;
- tool_count;
- terminal_event_count;
- payload_resolution_success;
- malformed_count;
- time-span coverage.

The health result is stored next to every imported source.

## Experiment claim gate

Tokn may only print a savings percentage as an observed candidate effect when:
- both compared runs are valid;
- same metric has sufficient evidence in both;
- terminal status satisfies experiment criteria;
- policy exposure/enforcement is known;
- task/workspace comparability requirements are satisfied.

Otherwise report:
DESCRIPTIVE_ONLY or NOT_EVALUABLE.
