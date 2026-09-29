# Experiment Validity Model

## Purpose

Tokn must decide whether a run can support an optimization conclusion
before displaying a winner or savings claim.

## Validity dimensions

### Capture validity
- model usage evidence available;
- tool evidence available when required;
- source health acceptable;
- parent/subagent graph resolved.

### Task validity
- exact task captured;
- terminal status known;
- required task completion achieved for experiments that require completion.

### Workspace validity
- actual output workspace identified;
- before/after state captured;
- quality gate executed on correct output.

### Policy validity
- policy identity captured;
- exposure known;
- compliance evidence sufficient;
- enforcement level accurately labeled.

### Runtime comparability
- model/runtime versions recorded;
- important configuration differences recorded.

### Quality validity
- automated gates executed;
- human/host gates recorded when required.
## Experiment verdicts

VALID_FOR_CAUSAL_AB
- same/frozen task;
- same starting workspace;
- same runtime/model configuration;
- both runs complete enough;
- evidence complete enough;
- quality accepted.

VALID_FOR_DESCRIPTIVE_COMPARISON
- useful metrics exist;
- one or more causal controls differ.

INSTRUMENTATION_ONLY
- run is useful to debug Tokn measurement;
- optimization conclusion is invalid.

INVALID_CAPTURE
- evidence insufficient to characterize model work.

INCOMPLETE_TASK
- usage limit/error/abort prevented required completion.

## Experiment 001 verdict

Capture via diagnostic trace alone: INVALID_CAPTURE.
Capture after session-rollout recovery: usable.

Task: INCOMPLETE_USAGE_LIMIT.

Policy observation: FAIL on recovered session evidence (58 targets / 17 violations).
Policy enforcement: SUPPORTED_INSUFFICIENT_INPUT for the historical max_output_tokens rule.

Workspace: original harness target invalid; post-mortem recovery successful.

Overall:
INSTRUMENTATION_ONLY.

No observed token-savings claim is permitted.
