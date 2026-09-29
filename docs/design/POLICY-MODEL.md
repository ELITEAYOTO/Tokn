# Policy Model: Hint, Observation, Enforcement

## Motivation

Experiment 001 placed a temporary AGENTS.md under PROJECT.
The Codex session cwd was E:\BlockBench-Plugin.

The policy was not present in session_meta.base_instructions.
The parent nevertheless found and read PROJECT/AGENTS.md manually.

This proves visibility is not enforcement.

## Three policy levels

### policy_hint

A model-facing instruction exists.

Evidence examples:
- AGENTS.md exists;
- prompt contains policy;
- session instructions include policy.

A hint may be ignored or only partially followed.

### policy_observed

Actual tool requests comply with the requested behavior.

Example:
file_read classified calls explicitly request max_output_tokens <= 5000.

Observation requires sufficient coverage.

### policy_enforced

A runtime mechanism prevents non-compliant requests from reaching the tool surface.

This requires an actual mediator/hook/proxy/configuration mechanism.
Tokn V0.1 does NOT claim this capability yet.
## Compliance states

For each policy rule:
- PASS;
- FAIL;
- NO_EVIDENCE;
- INCOMPLETE_EVIDENCE;
- NOT_APPLICABLE.

PASS requires:
- at least one applicable target observation;
- no violation;
- no unknown field that the rule requires.

Experiment 001 must evaluate to INCOMPLETE_EVIDENCE or FAIL,
never PASS.

## Policy placement

Until hard enforcement exists, Experiment runner should:
- place scoped policy at the actual Codex workspace root;
- optionally mirror it into the project root when safe;
- preserve/restore both independently;
- record hashes;
- verify the session saw the intended policy when observable.

A run must record:
- policy placement paths;
- base instruction evidence;
- explicit repository read evidence;
- observed compliance.

## Hard-enforcement research gate

Before implementing interception:
- probe supported Codex hooks/configuration locally;
- document whether exec_command output caps can be enforced externally;
- do not patch internal Codex binaries;
- do not depend on undocumented internals without adapter isolation.

If no stable enforcement point exists,
Tokn continues with measurement + soft policy experiments until a safe mechanism exists.
